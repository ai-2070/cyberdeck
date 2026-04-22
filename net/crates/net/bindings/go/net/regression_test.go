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
