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
