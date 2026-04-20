package net

import (
	"errors"
	"strings"
	"sync"
	"testing"
	"time"
)

const meshPsk = "4242424242424242424242424242424242424242424242424242424242424242"

// pairPorts returns two unique ephemeral-ish ports per test case.
// The Go binding doesn't yet expose `local_addr()`, so we can't let
// the OS pick — allocate from a shared counter instead.
var (
	portMu   sync.Mutex
	portBase = uint16(29200)
)

func allocPortPair(t *testing.T) (string, string) {
	t.Helper()
	portMu.Lock()
	defer portMu.Unlock()
	a := portBase
	b := portBase + 1
	portBase += 2
	return portAddr(a), portAddr(b)
}

func portAddr(p uint16) string { return "127.0.0.1:" + itoa(p) }

func itoa(p uint16) string {
	if p == 0 {
		return "0"
	}
	var buf [5]byte
	i := len(buf)
	for p > 0 {
		i--
		buf[i] = byte('0' + p%10)
		p /= 10
	}
	return string(buf[i:])
}

func meshHandshakePair(t *testing.T) (*MeshNode, *MeshNode, func()) {
	t.Helper()
	aAddr, bAddr := allocPortPair(t)
	a, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new mesh a: %v", err)
	}
	b, err := NewMeshNode(MeshConfig{BindAddr: bAddr, PskHex: meshPsk})
	if err != nil {
		a.Shutdown()
		t.Fatalf("new mesh b: %v", err)
	}

	bPub, err := b.PublicKey()
	if err != nil {
		a.Shutdown()
		b.Shutdown()
		t.Fatalf("public key: %v", err)
	}
	aID := a.NodeID()
	bID := b.NodeID()

	acceptDone := make(chan error, 1)
	go func() {
		_, err := b.Accept(aID)
		acceptDone <- err
	}()

	// Give Accept a chance to start polling.
	time.Sleep(50 * time.Millisecond)
	if err := a.Connect(bAddr, bPub, bID); err != nil {
		a.Shutdown()
		b.Shutdown()
		t.Fatalf("connect: %v", err)
	}
	if err := <-acceptDone; err != nil {
		a.Shutdown()
		b.Shutdown()
		t.Fatalf("accept: %v", err)
	}
	if err := a.Start(); err != nil {
		a.Shutdown()
		b.Shutdown()
		t.Fatalf("start a: %v", err)
	}
	if err := b.Start(); err != nil {
		a.Shutdown()
		b.Shutdown()
		t.Fatalf("start b: %v", err)
	}

	cleanup := func() {
		a.Shutdown()
		b.Shutdown()
	}
	return a, b, cleanup
}

func TestNewMeshNode(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	pk, err := m.PublicKey()
	if err != nil {
		t.Fatalf("public_key: %v", err)
	}
	if len(pk) != 64 {
		t.Fatalf("expected 64-char hex pubkey, got %d chars", len(pk))
	}
	if m.NodeID() == 0 {
		t.Fatalf("node_id should be non-zero")
	}
	if err := m.Shutdown(); err != nil {
		t.Fatalf("shutdown: %v", err)
	}
}

func TestMeshHandshake(t *testing.T) {
	_, _, cleanup := meshHandshakePair(t)
	cleanup()
}

func TestMeshOpenStreamAndSend(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	bID := b.NodeID()
	stream, err := a.OpenStream(bID, 0x1337, StreamConfig{
		Reliability: "reliable",
		WindowBytes: 1 << 14, // 16 KiB
	})
	if err != nil {
		t.Fatalf("open_stream: %v", err)
	}
	defer stream.Close()

	if err := stream.Send([][]byte{[]byte("hello")}); err != nil {
		t.Fatalf("send: %v", err)
	}

	// Drain B's inbound shards — the payload should land on one.
	deadline := time.Now().Add(2 * time.Second)
	var received []RecvdEvent
	for time.Now().Before(deadline) && len(received) == 0 {
		for shard := uint16(0); shard < 4; shard++ {
			events, err := b.RecvShard(shard, 16)
			if err != nil {
				t.Fatalf("recv_shard: %v", err)
			}
			received = append(received, events...)
		}
		if len(received) == 0 {
			time.Sleep(20 * time.Millisecond)
		}
	}
	if len(received) == 0 {
		t.Fatalf("no events received within timeout")
	}
}

func TestMeshSendBackpressureSurfaces(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	bID := b.NodeID()
	// Tiny window → concurrent sends will race the admission check
	// and at least one should surface ErrBackpressure on the first
	// loop iteration. v2 accounting charges ~86 B per 6-byte payload
	// (64 B header + 16 B tag + 6 B payload), so a 96-byte window
	// admits exactly one packet at a time.
	stream, err := a.OpenStream(bID, 0x2001, StreamConfig{
		Reliability: "fire_and_forget",
		WindowBytes: 96,
	})
	if err != nil {
		t.Fatalf("open_stream: %v", err)
	}
	defer stream.Close()

	sawBackpressure := false
	var wg sync.WaitGroup
	for i := 0; i < 16; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			err := stream.Send([][]byte{{}})
			if errors.Is(err, ErrBackpressure) {
				sawBackpressure = true
			}
		}()
	}
	wg.Wait()
	if !sawBackpressure {
		t.Fatalf("expected at least one ErrBackpressure from 16 concurrent window-1 sends")
	}
}

func TestMeshSendWithRetryAbsorbsBackpressure(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	bID := b.NodeID()
	stream, err := a.OpenStream(bID, 0x2002, StreamConfig{
		Reliability: "fire_and_forget",
		WindowBytes: 96,
	})
	if err != nil {
		t.Fatalf("open_stream: %v", err)
	}
	defer stream.Close()

	// SendWithRetry should succeed despite the tight window.
	if err := stream.SendWithRetry([][]byte{[]byte("xyz")}, 8); err != nil {
		t.Fatalf("send_with_retry: %v", err)
	}
}

func TestMeshStreamStats(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	bID := b.NodeID()
	_, err := a.OpenStream(bID, 0x3000, StreamConfig{Reliability: "reliable"})
	if err != nil {
		t.Fatalf("open_stream: %v", err)
	}
	stats, err := a.StreamStats(bID, 0x3000)
	if err != nil {
		t.Fatalf("stream_stats: %v", err)
	}
	if stats == nil {
		t.Fatalf("stream_stats returned nil for an opened stream")
	}
	if !stats.Active {
		t.Fatalf("opened stream should be active")
	}
}

func TestMeshStreamStatsNilForUnknown(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()
	stats, err := m.StreamStats(0xDEAD, 0xBEEF)
	if err != nil {
		t.Fatalf("stream_stats: %v", err)
	}
	if stats != nil {
		t.Fatalf("expected nil stats for unknown (peer, stream)")
	}
}

func TestMeshInvalidPskHex(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	_, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: "bad-psk"})
	if !errors.Is(err, ErrMeshInit) {
		t.Fatalf("expected ErrMeshInit for bad PSK; got %v", err)
	}
}

func TestMeshInvalidBindAddr(t *testing.T) {
	_, err := NewMeshNode(MeshConfig{BindAddr: "not-an-address", PskHex: meshPsk})
	if !errors.Is(err, ErrMeshInit) {
		t.Fatalf("expected ErrMeshInit for bad bind_addr; got %v", err)
	}
}

func TestMeshInvalidPeerPubkeyHex(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()
	err = m.Connect("127.0.0.1:9999", "not-hex", 0xDEAD)
	if !errors.Is(err, ErrMeshHandshake) {
		t.Fatalf("expected ErrMeshHandshake for bad pubkey; got %v", err)
	}
}

func TestMeshErrorMessageShape(t *testing.T) {
	// Sanity-check error message formatting so downstream tests that
	// rely on `errors.Is` keep working.
	if !strings.Contains(ErrBackpressure.Error(), "backpressure") {
		t.Fatalf("ErrBackpressure should mention 'backpressure'")
	}
}
