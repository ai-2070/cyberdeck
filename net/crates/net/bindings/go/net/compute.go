// Compute surface — MeshDaemon + migration. Stage 6 of
// SDK_COMPUTE_SURFACE_PLAN.md.
//
// Sub-step 1 covers lifecycle: a Go caller can build a
// DaemonRuntime against an existing MeshNode, transition it to
// Ready, register a placeholder kind, and shut it down. Event
// dispatch, spawn, snapshot/restore, and migration land in
// sub-steps 2-4.
package net

/*
#cgo LDFLAGS: -L${SRCDIR}/../../../target/release -lnet_compute
#include "net.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"sync"
	"unsafe"
)

// DaemonError is the base error type the compute surface surfaces.
// All errors from this package carry messages prefixed with
// "daemon:" so callers can dispatch programmatically if desired;
// matches the convention used by the Node and Python bindings.
type DaemonError struct {
	// Message is the free-form detail surfaced by the Rust layer
	// (everything after the "daemon: " prefix).
	Message string
}

// Error implements the built-in error interface.
func (e *DaemonError) Error() string {
	return "daemon: " + e.Message
}

// DuplicateKindError is returned by RegisterFactory when the same
// kind is registered twice.
type DuplicateKindError struct {
	Kind string
}

// Error implements the built-in error interface.
func (e *DuplicateKindError) Error() string {
	return fmt.Sprintf("daemon: factory for kind '%s' is already registered", e.Kind)
}

// ErrRuntimeShutDown is returned by DaemonRuntime methods called
// after Shutdown / Close.
var ErrRuntimeShutDown = errors.New("daemon: runtime handle freed")

// DaemonRuntime is the Go handle to the compute runtime. Construct
// via NewDaemonRuntime(mesh); each runtime shares the given
// MeshNode's socket + handshake table (no second socket).
//
// Lifecycle: NewDaemonRuntime → Start → (spawn/deliver/... — future
// sub-steps) → Shutdown. Close() is an idempotent alias for
// Shutdown() + freeing the native handle, safe to call from a
// finalizer.
type DaemonRuntime struct {
	handle *C.net_compute_runtime_t
	mu     sync.RWMutex
}

// NewDaemonRuntime builds a DaemonRuntime bound to the given
// MeshNode. The returned runtime shares the MeshNode's Arc — the
// mesh stays alive as long as the runtime holds its reference.
//
// Shutting down the runtime does NOT shut down the MeshNode;
// callers own the mesh lifecycle separately.
func NewDaemonRuntime(mesh *MeshNode) (*DaemonRuntime, error) {
	if mesh == nil {
		return nil, &DaemonError{Message: "mesh is nil"}
	}
	mesh.mu.RLock()
	meshHandle := mesh.handle
	mesh.mu.RUnlock()
	if meshHandle == nil {
		return nil, &DaemonError{Message: "mesh has been closed"}
	}

	// Clone Arcs from the mesh. These pointers are consumed by
	// net_compute_runtime_new on success — we only need to free
	// them on the error path.
	nodeArc := C.net_mesh_arc_clone(meshHandle)
	if nodeArc == nil {
		return nil, &DaemonError{Message: "failed to clone mesh Arc"}
	}
	ccArc := C.net_mesh_channel_configs_arc_clone(meshHandle)
	if ccArc == nil {
		C.net_mesh_arc_free(nodeArc)
		return nil, &DaemonError{Message: "failed to clone channel configs Arc"}
	}

	handle := C.net_compute_runtime_new(nodeArc, ccArc)
	if handle == nil {
		// Constructor consumed the Arcs on success only. On a NULL
		// return they'd still be intact — but we can't tell which
		// input Rust faulted on, so we conservatively free both.
		C.net_mesh_arc_free(nodeArc)
		C.net_mesh_channel_configs_arc_free(ccArc)
		return nil, &DaemonError{Message: "failed to build runtime"}
	}
	rt := &DaemonRuntime{handle: handle}
	runtime.SetFinalizer(rt, (*DaemonRuntime).Close)
	return rt, nil
}

// Start transitions the runtime to Ready and installs the migration
// subprotocol handler on the underlying mesh. Idempotent on an
// already-ready runtime; returns an error if the runtime has been
// shut down.
func (rt *DaemonRuntime) Start() error {
	rt.mu.RLock()
	defer rt.mu.RUnlock()
	if rt.handle == nil {
		return ErrRuntimeShutDown
	}
	var errOut *C.char
	code := C.net_compute_runtime_start(rt.handle, &errOut)
	return computeErr(code, errOut)
}

// Shutdown tears down the runtime (drains daemons, clears factory
// registrations, uninstalls the migration handler) but leaves the
// underlying MeshNode running. Idempotent; calling on an
// already-shut-down handle returns ErrRuntimeShutDown.
//
// After Shutdown, the native handle is still allocated — call
// Close to release it, or let the finalizer do so.
func (rt *DaemonRuntime) Shutdown() error {
	rt.mu.RLock()
	defer rt.mu.RUnlock()
	if rt.handle == nil {
		return ErrRuntimeShutDown
	}
	var errOut *C.char
	code := C.net_compute_runtime_shutdown(rt.handle, &errOut)
	return computeErr(code, errOut)
}

// IsReady reports whether the runtime has transitioned to Ready
// and not yet begun shutting down. Returns false for handles that
// have been Close()d.
func (rt *DaemonRuntime) IsReady() bool {
	rt.mu.RLock()
	defer rt.mu.RUnlock()
	if rt.handle == nil {
		return false
	}
	return C.net_compute_runtime_is_ready(rt.handle) == 1
}

// DaemonCount returns the number of daemons currently registered.
// Returns 0 for Close()d runtimes.
func (rt *DaemonRuntime) DaemonCount() int64 {
	rt.mu.RLock()
	defer rt.mu.RUnlock()
	if rt.handle == nil {
		return 0
	}
	n := int64(C.net_compute_runtime_daemon_count(rt.handle))
	if n < 0 {
		return 0
	}
	return n
}

// RegisterFactory registers a placeholder entry under kind.
// Sub-step 1 only tracks the kind string; sub-step 2 will wire
// the Go callback table so Rust can construct daemons on demand.
// Second registration of the same kind returns a
// *DuplicateKindError.
func (rt *DaemonRuntime) RegisterFactory(kind string) error {
	rt.mu.RLock()
	defer rt.mu.RUnlock()
	if rt.handle == nil {
		return ErrRuntimeShutDown
	}
	// C.CString allocates; avoid it for short-lived args by using
	// the byte-slice + size_t pattern that the Rust side prefers.
	kindBytes := []byte(kind)
	var ptr *C.char
	if len(kindBytes) > 0 {
		ptr = (*C.char)(unsafe.Pointer(&kindBytes[0]))
	}
	code := C.net_compute_register_factory(rt.handle, ptr, C.size_t(len(kindBytes)))
	// Keep `kindBytes` alive until after the call returns — Go's
	// escape analysis would otherwise free the slice backing
	// store if the closure below referenced only `ptr`.
	runtime.KeepAlive(kindBytes)
	switch code {
	case C.NET_COMPUTE_OK:
		return nil
	case C.NET_COMPUTE_ERR_DUPLICATE_KIND:
		return &DuplicateKindError{Kind: kind}
	case C.NET_COMPUTE_ERR_NULL:
		return &DaemonError{Message: "register_factory: null argument"}
	default:
		return &DaemonError{Message: fmt.Sprintf("register_factory: unexpected code %d", code)}
	}
}

// Close releases the native handle. Idempotent — a second call is
// a no-op. If Shutdown has not been called, Close attempts it
// best-effort (discarding the error) before freeing.
func (rt *DaemonRuntime) Close() {
	rt.mu.Lock()
	defer rt.mu.Unlock()
	if rt.handle == nil {
		return
	}
	var errOut *C.char
	_ = C.net_compute_runtime_shutdown(rt.handle, &errOut)
	if errOut != nil {
		C.net_compute_free_cstring(errOut)
	}
	C.net_compute_runtime_free(rt.handle)
	rt.handle = nil
	runtime.SetFinalizer(rt, nil)
}

// computeErr turns a compute-ffi return code + optional err_out
// CString into a Go error. OK returns nil; all other codes return
// a *DaemonError, with Message populated from err_out when set.
func computeErr(code C.int, errOut *C.char) error {
	if code == C.NET_COMPUTE_OK {
		if errOut != nil {
			// Shouldn't happen on success, but be defensive.
			C.net_compute_free_cstring(errOut)
		}
		return nil
	}
	msg := ""
	if errOut != nil {
		msg = C.GoString(errOut)
		C.net_compute_free_cstring(errOut)
	}
	if msg == "" {
		msg = fmt.Sprintf("compute call failed (code %d)", code)
	}
	return &DaemonError{Message: msg}
}
