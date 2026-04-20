package net

import (
	"errors"
	"testing"
	"time"
)

func TestMeshChannelRegister(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()

	if err := m.RegisterChannel(ChannelConfig{
		Name:       "sensors/temp",
		Visibility: "global",
		Reliable:   true,
		Priority:   3,
		MaxRatePps: 1000,
	}); err != nil {
		t.Fatalf("register: %v", err)
	}

	for _, v := range []string{"subnet-local", "parent-visible", "exported", "global"} {
		if err := m.RegisterChannel(ChannelConfig{Name: "v/" + v, Visibility: v}); err != nil {
			t.Fatalf("register %q: %v", v, err)
		}
	}
}

func TestMeshChannelInvalidName(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()
	err = m.RegisterChannel(ChannelConfig{Name: "has spaces"})
	if !errors.Is(err, ErrChannel) {
		t.Fatalf("expected ErrChannel for bad name; got %v", err)
	}
}

func TestMeshChannelInvalidVisibility(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()
	err = m.RegisterChannel(ChannelConfig{Name: "x", Visibility: "nonsense"})
	if !errors.Is(err, ErrChannel) {
		t.Fatalf("expected ErrChannel for bad visibility; got %v", err)
	}
}

func TestMeshChannelPublishEmptyRoster(t *testing.T) {
	aAddr, _ := allocPortPair(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: aAddr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new: %v", err)
	}
	defer m.Shutdown()

	if err := m.RegisterChannel(ChannelConfig{Name: "quiet", Visibility: "global"}); err != nil {
		t.Fatalf("register: %v", err)
	}
	report, err := m.Publish("quiet", []byte("hello"), PublishConfig{
		Reliability: "reliable",
		OnFailure:   "best_effort",
	})
	if err != nil {
		t.Fatalf("publish: %v", err)
	}
	if report.Attempted != 0 || report.Delivered != 0 {
		t.Fatalf("expected empty report, got attempted=%d delivered=%d",
			report.Attempted, report.Delivered)
	}
}

func TestMeshChannelSubscribeAndPublish(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	if err := b.RegisterChannel(ChannelConfig{
		Name:       "sensors/temp",
		Visibility: "global",
		Reliable:   true,
	}); err != nil {
		t.Fatalf("register: %v", err)
	}

	bID := b.NodeID()
	if err := a.SubscribeChannel(bID, "sensors/temp"); err != nil {
		t.Fatalf("subscribe: %v", err)
	}

	// `SubscribeChannel` blocks on the membership Ack, and the
	// publisher adds the subscriber to its roster *before* sending
	// that Ack (see `handle_membership_message` in adapter/net/mesh.rs),
	// so the first Publish should see Attempted=1. Poll with a bounded
	// deadline anyway so the test degrades to a clear timeout rather
	// than a cryptic assertion miss if that ordering ever changes.
	var report *PublishReport
	deadline := time.Now().Add(2 * time.Second)
	for {
		r, err := b.Publish("sensors/temp", []byte("22.5"), PublishConfig{
			Reliability: "reliable",
			OnFailure:   "best_effort",
		})
		if err != nil {
			t.Fatalf("publish: %v", err)
		}
		if r.Attempted == 1 {
			report = r
			break
		}
		if time.Now().After(deadline) {
			t.Fatalf("publisher roster never observed subscriber (attempted=%d)", r.Attempted)
		}
		time.Sleep(10 * time.Millisecond)
	}
	if report.Delivered != 1 {
		t.Fatalf("delivered: got %d, want 1", report.Delivered)
	}
	if len(report.Errors) != 0 {
		t.Fatalf("unexpected errors: %+v", report.Errors)
	}

	// Subscriber observes the payload on the event bus.
	recvDeadline := time.Now().Add(2 * time.Second)
	var received []RecvdEvent
	for time.Now().Before(recvDeadline) && len(received) == 0 {
		for shard := uint16(0); shard < 4; shard++ {
			events, err := a.RecvShard(shard, 16)
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
		t.Fatalf("subscriber didn't observe published payload")
	}
}

func TestMeshChannelSubscribeUnknownRejected(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	// Register `foo`, then ask for `bar` — must be rejected.
	if err := b.RegisterChannel(ChannelConfig{Name: "foo"}); err != nil {
		t.Fatalf("register: %v", err)
	}

	bID := b.NodeID()
	err := a.SubscribeChannel(bID, "bar")
	if !errors.Is(err, ErrChannel) {
		t.Fatalf("expected ErrChannel for unknown channel; got %v", err)
	}
}

func TestMeshChannelUnsubscribeIdempotent(t *testing.T) {
	a, b, cleanup := meshHandshakePair(t)
	defer cleanup()

	if err := b.RegisterChannel(ChannelConfig{Name: "chan/x"}); err != nil {
		t.Fatalf("register: %v", err)
	}
	bID := b.NodeID()
	// Unsubscribing a non-member should succeed on the publisher side.
	if err := a.UnsubscribeChannel(bID, "chan/x"); err != nil {
		t.Fatalf("unsubscribe: %v", err)
	}
}
