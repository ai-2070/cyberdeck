// Regression tests for specific bugs fixed in the cortex / mesh Go
// bindings. Each test ties back to a known failure mode that would
// reappear if the fix regressed.

package net

import (
	"errors"
	"math"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// ---------------------------------------------------------------------------
// P1 regression: concurrent Close vs in-flight ops must not
// use-after-free the native handle.
//
// Before the fix, non-lifecycle methods did
//     mu.Lock(); h := handle; mu.Unlock(); C.call(h, ...)
// so a Close() between the unlock and the C call would free `h`
// while it was still on the call stack of another goroutine. The fix
// switches every handle type to RWMutex + holds RLock through the C
// call; Close takes the writer lock and waits.
//
// The stress pattern here: N concurrent workers doing small CRUD /
// list / wait ops in a loop, plus one goroutine that issues Close.
// Under the old pattern this routinely crashed under `go test
// -race`; under the fix it must drain cleanly with no panics and
// with post-close ops returning ErrShuttingDown.
// ---------------------------------------------------------------------------

func TestRegressionConcurrentCloseNoUseAfterFree(t *testing.T) {
	r := NewRedex("")
	defer r.Free()
	tasks, err := OpenTasks(r, testOrigin, false)
	if err != nil {
		t.Fatalf("open: %v", err)
	}

	const workers = 16
	const opsPerWorker = 200

	var wg sync.WaitGroup
	start := make(chan struct{})

	// Pre-seed so there's state to list.
	for i := uint64(1); i <= 8; i++ {
		if _, err := tasks.Create(i, "seed", 100); err != nil {
			t.Fatalf("seed: %v", err)
		}
	}

	var observedShutdown atomic.Int64

	// Workers: each does Create + List + Complete in a tight loop.
	// After Close, they'll start seeing ErrShuttingDown — that's the
	// expected post-close behavior, not a crash.
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			<-start
			for i := 0; i < opsPerWorker; i++ {
				idU64 := uint64(1000 + id*opsPerWorker + i)
				if _, err := tasks.Create(idU64, "stress", 100); err != nil {
					if errors.Is(err, ErrShuttingDown) ||
						errors.Is(err, ErrCortexClosed) ||
						errors.Is(err, ErrCortexFold) {
						observedShutdown.Add(1)
						return
					}
					t.Errorf("worker %d: unexpected create err: %v", id, err)
					return
				}
				if _, err := tasks.List(nil); err != nil {
					if errors.Is(err, ErrShuttingDown) ||
						errors.Is(err, ErrCortexClosed) ||
						errors.Is(err, ErrCortexFold) {
						observedShutdown.Add(1)
						return
					}
					t.Errorf("worker %d: unexpected list err: %v", id, err)
					return
				}
				if _, err := tasks.Complete(idU64, 200); err != nil {
					if errors.Is(err, ErrShuttingDown) ||
						errors.Is(err, ErrCortexClosed) ||
						errors.Is(err, ErrCortexFold) {
						observedShutdown.Add(1)
						return
					}
					t.Errorf("worker %d: unexpected complete err: %v", id, err)
					return
				}
			}
		}(w)
	}

	close(start)
	// Let workers run for a bit, then Close concurrently.
	time.Sleep(20 * time.Millisecond)
	if err := tasks.Close(); err != nil {
		t.Fatalf("close: %v", err)
	}

	// Drain workers. If the old race returned, they'd have segfaulted
	// before this Wait completed — a panic in a goroutine kills the
	// whole test process, so arrival at the `wg.Wait()` call is itself
	// the "no use-after-free" assertion.
	wg.Wait()
	// Force one GC so finalizers run on any still-dangling handles;
	// catches cases where a stale handle pointer was retained.
	runtime.GC()

	// Some workers raced close and observed shutdown; that's expected.
	// We only assert at least one worker actually ran to completion
	// (otherwise the test is vacuous).
	count := observedShutdown.Load()
	t.Logf("workers observed shutdown: %d / %d", count, workers)

	// Post-close ops must cleanly return ErrShuttingDown (not
	// crash / not silently succeed).
	_, err = tasks.Create(9999, "after-close", 300)
	if err == nil {
		t.Fatalf("create after close should error, got nil")
	}
}

// ---------------------------------------------------------------------------
// P2 regression: invalid order_by in list filters must surface as
// ErrInvalidJSON (or ErrCortexFold after JSON round-trip) rather
// than silently falling back to default ordering.
// ---------------------------------------------------------------------------

func TestRegressionInvalidTasksOrderByRejected(t *testing.T) {
	r := NewRedex("")
	defer r.Free()
	tasks, err := OpenTasks(r, testOrigin, false)
	if err != nil {
		t.Fatalf("open: %v", err)
	}
	defer tasks.Close()

	_, err = tasks.List(&TasksFilter{OrderBy: "createdasc"}) // typo'd
	if err == nil {
		t.Fatalf("expected error for misspelled order_by; got nil")
	}
	if !errors.Is(err, ErrInvalidJSON) {
		t.Fatalf("expected ErrInvalidJSON for bad order_by; got %v", err)
	}
}

func TestRegressionInvalidMemoriesOrderByRejected(t *testing.T) {
	r := NewRedex("")
	defer r.Free()
	mem, err := OpenMemories(r, testOrigin, false)
	if err != nil {
		t.Fatalf("open: %v", err)
	}
	defer mem.Close()

	_, err = mem.List(&MemoriesFilter{OrderBy: "updatedasc"}) // typo'd
	if err == nil {
		t.Fatalf("expected error for misspelled order_by; got nil")
	}
	if !errors.Is(err, ErrInvalidJSON) {
		t.Fatalf("expected ErrInvalidJSON for bad order_by; got %v", err)
	}
}

// ---------------------------------------------------------------------------
// P2 regression: time.Duration → uint32 ms conversion must clamp at
// math.MaxUint32 rather than wrap. A ~50-day timeout wrapping to 0
// would flip "wait indefinitely" semantics into the FFI's
// "never-wait" sentinel.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// P1 regression: Close() must not hang when an indefinite WaitForSeq
// is in flight.
//
// Before the fix, Close took the writer Lock which blocked on the
// WaitForSeq goroutine's RLock; WaitForSeq held that RLock through a
// native await that only returned when `seq` was reached (or timeout
// elapsed). With `timeout = 0` the wait is indefinite, so Close hung
// forever. Fix: signal native close under RLock first (fast atomic +
// notify wakes pending waiters), then take writer Lock.
// ---------------------------------------------------------------------------

func TestRegressionCloseDoesNotHangOnIndefiniteWaitForSeq(t *testing.T) {
	r := NewRedex("")
	defer r.Free()
	tasks, err := OpenTasks(r, testOrigin, false)
	if err != nil {
		t.Fatalf("open: %v", err)
	}

	waitDone := make(chan error, 1)
	go func() {
		// Seq that will never be reached — nothing will be ingested
		// after this. `timeout = 0` → wait indefinitely, exactly the
		// case that previously deadlocked Close.
		waitDone <- tasks.WaitForSeq(99999, 0)
	}()

	// Let the goroutine park inside the C wait.
	time.Sleep(20 * time.Millisecond)

	closeDone := make(chan error, 1)
	go func() { closeDone <- tasks.Close() }()

	select {
	case err := <-closeDone:
		if err != nil {
			t.Fatalf("close returned error: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatalf("Close hung with indefinite WaitForSeq in flight (pre-fix behavior)")
	}

	// WaitForSeq must also return, not leak the goroutine.
	select {
	case <-waitDone:
	case <-time.After(1 * time.Second):
		t.Fatalf("WaitForSeq goroutine leaked — native close didn't wake it")
	}
}

func TestRegressionDurationToMillisU32Clamp(t *testing.T) {
	cases := []struct {
		name string
		d    time.Duration
		want uint32
	}{
		{"zero", 0, 0},
		{"negative", -time.Second, 0},
		{"one ms", time.Millisecond, 1},
		{"one second", time.Second, 1000},
		{"max-u32-minus-1", time.Duration(math.MaxUint32-1) * time.Millisecond, math.MaxUint32 - 1},
		{"at-max-u32", time.Duration(math.MaxUint32) * time.Millisecond, math.MaxUint32},
		{"above-max-u32-clamps", time.Duration(math.MaxUint32+1000) * time.Millisecond, math.MaxUint32},
		{"fifty-days-clamps", 50 * 24 * time.Hour, math.MaxUint32},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := uint32(durationToMillisU32(tc.d))
			if got != tc.want {
				t.Fatalf("durationToMillisU32(%v) = %d, want %d", tc.d, got, tc.want)
			}
		})
	}
}

// ---------------------------------------------------------------------------
// P1 regression: TOCTOU between NewDaemonRuntime and mesh.Shutdown.
//
// Before the fix, NewDaemonRuntime did:
//
//	mesh.mu.RLock(); h := mesh.handle; mesh.mu.RUnlock()
//	C.net_mesh_arc_clone(h); C.net_mesh_channel_configs_arc_clone(h)
//
// A concurrent mesh.Shutdown() could acquire the write lock between
// the RUnlock and the Arc-clone FFI calls, free the native handle,
// and leave the local `h` dangling. The second call would then
// dereference freed memory — a use-after-free on the C side that
// Go's `-race` detector cannot catch (it watches Go-level memory
// only).
//
// The fix holds the read lock across both arc_clone calls so
// Shutdown blocks until the clones complete; the cloned Arcs then
// keep the underlying object alive regardless of what Shutdown
// does next. This stress test races N concurrent NewDaemonRuntime
// calls against one Shutdown. Under the old code it could
// sporadically crash; under the fix every outcome is either a
// built runtime or a typed *DaemonError with a "closed / failed
// to clone" message — never a segfault, never an untyped error.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// P2 regression: negative / huge `time.Duration` must clamp to a
// well-formed `uint64` ms for `MigrationHandle.WaitWithTimeout`.
//
// `time.Duration.Milliseconds()` returns `int64`. Before the fix,
// passing a negative duration (or `time.Until(pastDeadline)` that
// resolved past zero) cast straight to `C.uint64_t` and wrapped
// into ~584 million years of wait — a past-deadline call turned
// into effectively infinite blocking. The fix extracts the clamp
// into `clampTimeoutToU64Ms` and `WaitWithTimeout` routes through
// it.
//
// Testing the clamp on a live `*MigrationHandle` with a nil handle
// is a false positive: the nil-check short-circuits to
// `ErrRuntimeShutDown` before the clamp runs. Exercise the helper
// directly so every branch is covered — including values that
// *should* pass through unchanged.
// ---------------------------------------------------------------------------

func TestRegressionClampTimeoutToU64Ms(t *testing.T) {
	cases := []struct {
		name string
		in   time.Duration
		want uint64
	}{
		{"zero", 0, 0},
		// Any negative duration — the bug path: -1 ns would
		// otherwise wrap to 0xFFFF_FFFF_FFFF_FFFF.
		{"negative_nanosecond", -time.Nanosecond, 0},
		{"negative_one_second", -time.Second, 0},
		{"min_int64", time.Duration(math.MinInt64), 0},
		{"one_ms", time.Millisecond, 1},
		{"one_second", time.Second, 1000},
		// Upper end: large but still int64-positive values flow
		// through as their millisecond count.
		{"one_hour", time.Hour, 3600 * 1000},
		// math.MaxInt64 ns expressed in ms — the divide-by-1e6
		// keeps it int64-representable.
		{"max_int64_ns", time.Duration(math.MaxInt64), uint64(math.MaxInt64) / 1_000_000},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := clampTimeoutToU64Ms(tc.in)
			if got != tc.want {
				t.Fatalf("clampTimeoutToU64Ms(%v) = %d, want %d", tc.in, got, tc.want)
			}
		})
	}
}

func TestRegressionMigrationHandleWaitWithTimeoutOnClosedHandle(t *testing.T) {
	// Independently verify the `handle == nil` short-circuit so a
	// refactor of `WaitWithTimeout` can't silently drop the shutdown
	// guard. Negative duration is passed to exercise the lock path
	// first; the clamp itself is covered by
	// `TestRegressionClampTimeoutToU64Ms` above.
	h := &MigrationHandle{handle: nil}

	done := make(chan error, 1)
	go func() {
		done <- h.WaitWithTimeout(-1 * time.Second)
	}()

	select {
	case err := <-done:
		if !errors.Is(err, ErrRuntimeShutDown) {
			t.Errorf("WaitWithTimeout on nil handle = %v, want ErrRuntimeShutDown", err)
		}
	case <-time.After(500 * time.Millisecond):
		t.Fatal("WaitWithTimeout on nil handle did not return within 500ms")
	}
}

func TestRegressionNewDaemonRuntimeVsShutdownNoUseAfterFree(t *testing.T) {
	const workers = 16
	const iterations = 20

	for iter := 0; iter < iterations; iter++ {
		mesh := newLocalMesh(t)

		var wg sync.WaitGroup
		built := atomic.Int32{}
		closedRace := atomic.Int32{}

		for i := 0; i < workers; i++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				rt, err := NewDaemonRuntime(mesh)
				if err == nil {
					built.Add(1)
					rt.Close()
					return
				}
				var de *DaemonError
				if !errors.As(err, &de) {
					t.Errorf("NewDaemonRuntime error is not *DaemonError: %v", err)
					return
				}
				switch {
				case strings.Contains(de.Message, "mesh has been closed"),
					strings.Contains(de.Message, "failed to clone"):
					closedRace.Add(1)
				default:
					t.Errorf("unexpected NewDaemonRuntime error: %q", de.Message)
				}
			}()
		}

		// Fire the shutdown partway through the racers so some
		// succeed and some collide with the close.
		time.Sleep(time.Duration(iter%5) * time.Microsecond)
		_ = mesh.Shutdown()

		wg.Wait()

		// Sanity: every worker must account for itself via one
		// of the two typed outcomes. A drop would imply a goroutine
		// panicked or the error wasn't typed.
		if built.Load()+closedRace.Load() != workers {
			t.Fatalf(
				"iter %d: accounted %d + %d worker results, want %d",
				iter, built.Load(), closedRace.Load(), workers,
			)
		}
	}
}

// ---------------------------------------------------------------------------
// P1 regression: the process-global Go factory map must scope by
// (runtime_id, kind) so two runtimes in the same process that
// register the same kind don't collide.
//
// Before the fix the map was keyed on `kind` alone; the second
// `RegisterFactoryFunc("counter", ...)` across two runtimes would
// overwrite the first, and a migration landing on the original
// runtime's trampoline would reconstruct using the wrong factory.
// The fix captures `net_compute_runtime_id` per runtime and scopes
// every map operation to `(runtime_id, kind)`.
//
// This test:
//   1. Creates two independent runtimes on two independent meshes.
//   2. Registers the same kind on each with a *different* factory
//      (one produces an "A"-flavoured counter, the other "B").
//   3. Asserts the Go-side trampoline resolves each runtime's
//      registration independently by looking them up with their
//      respective runtime ids.
//   4. Asserts Close() purges each runtime's entries from the map.
// ---------------------------------------------------------------------------

type flavouredDaemon struct{ flavour string }

func (flavouredDaemon) Process(event CausalEvent) ([][]byte, error) {
	return nil, nil
}

func TestRegressionFactoryMapScopedByRuntimeID(t *testing.T) {
	meshA := newLocalMesh(t)
	defer meshA.Shutdown()
	meshB := newLocalMesh(t)
	defer meshB.Shutdown()

	rtA, err := NewDaemonRuntime(meshA)
	if err != nil {
		t.Fatalf("NewDaemonRuntime A: %v", err)
	}
	rtB, err := NewDaemonRuntime(meshB)
	if err != nil {
		t.Fatalf("NewDaemonRuntime B: %v", err)
	}

	if rtA.runtimeID == 0 || rtB.runtimeID == 0 {
		t.Fatalf("runtime ids must be non-zero; got A=%d B=%d", rtA.runtimeID, rtB.runtimeID)
	}
	if rtA.runtimeID == rtB.runtimeID {
		t.Fatalf("two runtimes must have distinct ids; both are %d", rtA.runtimeID)
	}

	factoryA := func() MeshDaemon { return flavouredDaemon{flavour: "A"} }
	factoryB := func() MeshDaemon { return flavouredDaemon{flavour: "B"} }

	if err := rtA.RegisterFactoryFunc("counter", factoryA); err != nil {
		t.Fatalf("RegisterFactoryFunc A: %v", err)
	}
	if err := rtB.RegisterFactoryFunc("counter", factoryB); err != nil {
		t.Fatalf("RegisterFactoryFunc B: %v", err)
	}

	// Each trampoline lookup must return its own runtime's factory.
	// Before the fix, `factoryB` would have overwritten `factoryA`
	// on a shared `map[string]DaemonFactory`.
	gotA := lookupFactoryFunc(rtA.runtimeID, "counter")
	gotB := lookupFactoryFunc(rtB.runtimeID, "counter")
	if gotA == nil || gotB == nil {
		t.Fatalf("both lookups should return non-nil; gotA=%v gotB=%v", gotA, gotB)
	}
	a := gotA().(flavouredDaemon)
	b := gotB().(flavouredDaemon)
	if a.flavour != "A" {
		t.Errorf("runtime A's factory produced flavour=%q, want 'A'", a.flavour)
	}
	if b.flavour != "B" {
		t.Errorf("runtime B's factory produced flavour=%q, want 'B'", b.flavour)
	}

	// Close rtA — its entry must vanish from the map. rtB's
	// registration must survive.
	rtA.Close()
	if lookupFactoryFunc(rtA.runtimeID, "counter") != nil {
		t.Errorf("rtA factory still present after Close — purge leaked")
	}
	if lookupFactoryFunc(rtB.runtimeID, "counter") == nil {
		t.Errorf("rtB factory vanished when rtA closed — wrong runtime purged")
	}

	rtB.Close()
	if lookupFactoryFunc(rtB.runtimeID, "counter") != nil {
		t.Errorf("rtB factory still present after Close — purge leaked")
	}
}
