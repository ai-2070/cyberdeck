// Tests for the compute surface — Stage 6 of
// SDK_COMPUTE_SURFACE_PLAN.md.
//
// Sub-step 1 covers lifecycle only: a Go caller can build a
// DaemonRuntime against a NetMesh, register a kind (stored but
// not yet invoked), start the runtime, and shut it down. Event
// dispatch, spawn, snapshot/restore, and migration land in
// sub-steps 2-4.
package net

import (
	"errors"
	"fmt"
	"strings"
	"testing"
)

// newLocalMesh builds a single-node mesh on an OS-assigned
// localhost port. Caller is responsible for `m.Close()`.
func newLocalMesh(t *testing.T) *MeshNode {
	t.Helper()
	addr := reserveLocalUDPPort(t)
	m, err := NewMeshNode(MeshConfig{
		BindAddr: addr,
		PskHex:   meshPsk,
	})
	if err != nil {
		t.Fatalf("NewMeshNode(%q) failed: %v", addr, err)
	}
	return m
}

func TestDaemonRuntime_BuildsAndReportsNotReadyBeforeStart(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()

	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if rt.IsReady() {
		t.Errorf("IsReady() = true before Start(), want false")
	}
	if n := rt.DaemonCount(); n != 0 {
		t.Errorf("DaemonCount() = %d, want 0", n)
	}
}

func TestDaemonRuntime_StartFlipsToReady(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()

	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if err := rt.Start(); err != nil {
		t.Fatalf("Start failed: %v", err)
	}
	if !rt.IsReady() {
		t.Errorf("IsReady() = false after Start, want true")
	}
}

func TestDaemonRuntime_ShutdownFlipsBack(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()

	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if err := rt.Start(); err != nil {
		t.Fatalf("Start failed: %v", err)
	}
	if err := rt.Shutdown(); err != nil {
		t.Fatalf("Shutdown failed: %v", err)
	}
	if rt.IsReady() {
		t.Errorf("IsReady() = true after Shutdown, want false")
	}
}

func TestDaemonRuntime_ShutdownIsIdempotent(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()

	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if err := rt.Start(); err != nil {
		t.Fatalf("Start failed: %v", err)
	}
	if err := rt.Shutdown(); err != nil {
		t.Fatalf("first Shutdown failed: %v", err)
	}
	// Second Shutdown on an already-shut-down SDK runtime is a
	// no-op at the Rust layer. Accept either nil or a clean error
	// as long as nothing panics.
	_ = rt.Shutdown()
}

func TestDaemonRuntime_RegisterFactory_AcceptsKind(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if err := rt.RegisterFactory("echo"); err != nil {
		t.Errorf("RegisterFactory('echo') failed: %v", err)
	}
}

func TestDaemonRuntime_RegisterFactory_DuplicateKindFails(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	if err := rt.RegisterFactory("echo"); err != nil {
		t.Fatalf("first RegisterFactory failed: %v", err)
	}
	err = rt.RegisterFactory("echo")
	if err == nil {
		t.Fatalf("duplicate RegisterFactory('echo') succeeded, want error")
	}
	var dup *DuplicateKindError
	if !errors.As(err, &dup) {
		t.Errorf("err = %v (type %T), want *DuplicateKindError", err, err)
	} else if dup.Kind != "echo" {
		t.Errorf("DuplicateKindError.Kind = %q, want 'echo'", dup.Kind)
	}
}

func TestDaemonRuntime_RegisterFactory_DifferentKindsCoexist(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()

	for _, k := range []string{"echo", "counter", "router"} {
		if err := rt.RegisterFactory(k); err != nil {
			t.Errorf("RegisterFactory(%q) failed: %v", k, err)
		}
	}
}

func TestDaemonRuntime_NewWithNilMeshErrors(t *testing.T) {
	_, err := NewDaemonRuntime(nil)
	if err == nil {
		t.Fatalf("NewDaemonRuntime(nil) succeeded, want error")
	}
	if !strings.HasPrefix(err.Error(), "daemon:") {
		t.Errorf("err = %q, want prefix 'daemon:'", err.Error())
	}
}

func TestDaemonRuntime_DoesNotShutDownUnderlyingMesh(t *testing.T) {
	// Shutting down the runtime tears down daemons + migration
	// handler but leaves the NetMesh alive.
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	if err := rt.Start(); err != nil {
		t.Fatalf("Start failed: %v", err)
	}
	if err := rt.Shutdown(); err != nil {
		t.Fatalf("Shutdown failed: %v", err)
	}
	rt.Close()
	// Mesh should still be usable.
	if id := m.NodeID(); id == 0 {
		t.Errorf("mesh.NodeID() = 0 after runtime shutdown, mesh should still be alive")
	}
}

// -------------------------------------------------------------------------
// Sub-step 2: spawn + stop + event dispatch via Go callbacks
// -------------------------------------------------------------------------

// EchoDaemon — trivial stateless daemon that echoes the payload.
type echoDaemon struct{}

func (echoDaemon) Process(event CausalEvent) ([][]byte, error) {
	out := make([]byte, len(event.Payload))
	copy(out, event.Payload)
	return [][]byte{out}, nil
}

// counterDaemon — stateful, increments per event, snapshot/restore
// round-trip the 4-byte LE count.
type counterDaemon struct {
	count uint32
}

func (c *counterDaemon) Process(event CausalEvent) ([][]byte, error) {
	c.count++
	buf := make([]byte, 4)
	buf[0] = byte(c.count)
	buf[1] = byte(c.count >> 8)
	buf[2] = byte(c.count >> 16)
	buf[3] = byte(c.count >> 24)
	return [][]byte{buf}, nil
}

func (c *counterDaemon) Snapshot() ([]byte, error) {
	buf := make([]byte, 4)
	buf[0] = byte(c.count)
	buf[1] = byte(c.count >> 8)
	buf[2] = byte(c.count >> 16)
	buf[3] = byte(c.count >> 24)
	return buf, nil
}

func (c *counterDaemon) Restore(state []byte) error {
	if len(state) != 4 {
		return fmt.Errorf("expected 4-byte state, got %d", len(state))
	}
	c.count = uint32(state[0]) | uint32(state[1])<<8 | uint32(state[2])<<16 | uint32(state[3])<<24
	return nil
}

func readU32LE(b []byte) uint32 {
	return uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 | uint32(b[3])<<24
}

func TestDaemonSpawn_ReturnsHandleWithOriginHash(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime failed: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start failed: %v", err)
	}

	id, err := GenerateIdentity()
	if err != nil {
		t.Fatalf("IdentityGenerate failed: %v", err)
	}
	defer id.Close()

	h, err := rt.Spawn("echo", id, echoDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn failed: %v", err)
	}
	defer h.Close()

	if h.OriginHash() != id.OriginHash() {
		t.Errorf("OriginHash mismatch: handle=%x identity=%x", h.OriginHash(), id.OriginHash())
	}
	expectedEID, err := id.EntityID()
	if err != nil {
		t.Fatalf("identity.EntityID failed: %v", err)
	}
	if !bytesEqual(h.EntityID(), expectedEID) {
		t.Errorf("EntityID mismatch: handle=%x identity=%x", h.EntityID(), expectedEID)
	}
}

func TestDaemonSpawn_StopReducesDaemonCount(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	id, _ := GenerateIdentity()
	defer id.Close()

	h, err := rt.Spawn("echo", id, echoDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn: %v", err)
	}
	defer h.Close()
	if rt.DaemonCount() != 1 {
		t.Errorf("DaemonCount = %d after Spawn, want 1", rt.DaemonCount())
	}

	if err := rt.Stop(h.OriginHash()); err != nil {
		t.Fatalf("Stop: %v", err)
	}
	if rt.DaemonCount() != 0 {
		t.Errorf("DaemonCount = %d after Stop, want 0", rt.DaemonCount())
	}
}

func TestDeliver_EchoDaemonRoundTrip(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	id, _ := GenerateIdentity()
	defer id.Close()
	h, err := rt.Spawn("echo", id, echoDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn: %v", err)
	}
	defer h.Close()

	payload := []byte("hello from go")
	outputs, err := rt.Deliver(h.OriginHash(), CausalEvent{
		OriginHash: id.OriginHash(),
		Sequence:   1,
		Payload:    payload,
	})
	if err != nil {
		t.Fatalf("Deliver: %v", err)
	}
	if len(outputs) != 1 {
		t.Fatalf("len(outputs) = %d, want 1", len(outputs))
	}
	if !bytesEqual(outputs[0], payload) {
		t.Errorf("output = %q, want %q", outputs[0], payload)
	}
}

func TestDeliver_CounterDaemonAccumulatesState(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	id, _ := GenerateIdentity()
	defer id.Close()
	h, err := rt.Spawn("counter", id, &counterDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn: %v", err)
	}
	defer h.Close()

	for i := uint64(1); i <= 5; i++ {
		out, err := rt.Deliver(h.OriginHash(), CausalEvent{
			OriginHash: id.OriginHash(),
			Sequence:   i,
			Payload:    nil,
		})
		if err != nil {
			t.Fatalf("Deliver(%d): %v", i, err)
		}
		if len(out) != 1 {
			t.Fatalf("Deliver(%d): len(outputs) = %d", i, len(out))
		}
		if got := readU32LE(out[0]); got != uint32(i) {
			t.Errorf("Deliver(%d): counter = %d, want %d", i, got, i)
		}
	}
}

func TestDeliver_FanoutReturnsMultipleOutputs(t *testing.T) {
	type fanout struct{}
	// Local daemon — can be declared inline because `MeshDaemon`
	// interface is satisfied by a single Process method.
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	id, _ := GenerateIdentity()
	defer id.Close()

	// Use a closure-based daemon to avoid another top-level type.
	h, err := rt.Spawn("fanout", id, &fanoutDaemon{
		outs: [][]byte{[]byte("a"), []byte("bb"), []byte("ccc")},
	}, nil)
	if err != nil {
		t.Fatalf("Spawn: %v", err)
	}
	defer h.Close()

	outputs, err := rt.Deliver(h.OriginHash(), CausalEvent{
		OriginHash: id.OriginHash(),
		Sequence:   1,
	})
	if err != nil {
		t.Fatalf("Deliver: %v", err)
	}
	if len(outputs) != 3 {
		t.Fatalf("len(outputs) = %d, want 3", len(outputs))
	}
	want := []string{"a", "bb", "ccc"}
	for i, o := range outputs {
		if string(o) != want[i] {
			t.Errorf("outputs[%d] = %q, want %q", i, o, want[i])
		}
	}
	_ = fanout{}
}

type fanoutDaemon struct {
	outs [][]byte
}

func (f *fanoutDaemon) Process(_ CausalEvent) ([][]byte, error) {
	return f.outs, nil
}

func TestDeliver_ProcessErrorSurfaces(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	id, _ := GenerateIdentity()
	defer id.Close()

	h, err := rt.Spawn("buggy", id, &buggyDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn: %v", err)
	}
	defer h.Close()

	_, err = rt.Deliver(h.OriginHash(), CausalEvent{OriginHash: id.OriginHash(), Sequence: 1})
	if err == nil {
		t.Fatalf("expected error from Deliver, got nil")
	}
	var de *DaemonError
	if !errors.As(err, &de) {
		t.Errorf("err = %v (type %T), want *DaemonError", err, err)
	}
}

type buggyDaemon struct{}

func (buggyDaemon) Process(_ CausalEvent) ([][]byte, error) {
	return nil, errors.New("deliberate failure")
}

func TestDeliver_UnknownOriginReturnsError(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}
	_, err = rt.Deliver(0xDEADBEEF, CausalEvent{OriginHash: 0xDEADBEEF, Sequence: 1, Payload: []byte("x")})
	if err == nil {
		t.Fatalf("expected error from Deliver to unknown origin, got nil")
	}
}

func TestDeliver_TwoDaemonsIndependentState(t *testing.T) {
	m := newLocalMesh(t)
	defer m.Shutdown()
	rt, err := NewDaemonRuntime(m)
	if err != nil {
		t.Fatalf("NewDaemonRuntime: %v", err)
	}
	defer rt.Close()
	if err := rt.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}

	idA, _ := GenerateIdentity()
	defer idA.Close()
	idB, _ := GenerateIdentity()
	defer idB.Close()

	hA, err := rt.Spawn("counter", idA, &counterDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn A: %v", err)
	}
	defer hA.Close()
	hB, err := rt.Spawn("counter", idB, &counterDaemon{}, nil)
	if err != nil {
		t.Fatalf("Spawn B: %v", err)
	}
	defer hB.Close()

	for i := uint64(1); i <= 3; i++ {
		out, err := rt.Deliver(hA.OriginHash(), CausalEvent{OriginHash: idA.OriginHash(), Sequence: i})
		if err != nil {
			t.Fatalf("Deliver A %d: %v", i, err)
		}
		if got := readU32LE(out[0]); got != uint32(i) {
			t.Errorf("A[%d] = %d, want %d", i, got, i)
		}
	}
	out, err := rt.Deliver(hB.OriginHash(), CausalEvent{OriginHash: idB.OriginHash(), Sequence: 1})
	if err != nil {
		t.Fatalf("Deliver B 1: %v", err)
	}
	if got := readU32LE(out[0]); got != 1 {
		t.Errorf("B[1] = %d, want 1", got)
	}
}

func bytesEqual(a, b []byte) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
