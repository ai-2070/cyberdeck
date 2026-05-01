// Smoke tests for the Go RedisStreamDedup helper. The Rust core
// (net::adapter::redis_dedup) and the Rust SDK have exhaustive
// coverage; here we verify the cgo shape and the canonical
// BUG #57 producer-retry scenario.

package net

import (
	"fmt"
	"testing"
)

func TestRedisStreamDedup_Lifecycle(t *testing.T) {
	d := NewRedisStreamDedup(0) // 0 → default 4096
	defer d.Close()

	if got, want := d.Capacity(), uint(4096); got != want {
		t.Errorf("default capacity: got %d, want %d", got, want)
	}
	if !d.IsEmpty() {
		t.Errorf("expected empty helper after construction")
	}
	if got, want := d.Len(), uint(0); got != want {
		t.Errorf("initial len: got %d, want %d", got, want)
	}
}

func TestRedisStreamDedup_FiltersProducerRetryDuplicates(t *testing.T) {
	d := NewRedisStreamDedup(64)
	defer d.Close()

	const batchSize = 5
	mkID := func(i int) string {
		return fmt.Sprintf("deadbeef:0:0:%d", i)
	}

	// First pass: every id is new.
	for i := 0; i < batchSize; i++ {
		id := mkID(i)
		if d.IsDuplicate(id) {
			t.Errorf("first observation of %q reported as duplicate", id)
		}
	}
	if got, want := d.Len(), uint(batchSize); got != want {
		t.Errorf("len after batch: got %d, want %d", got, want)
	}

	// Producer-side retry path: same dedup_ids reappear. All filtered.
	for i := 0; i < batchSize; i++ {
		id := mkID(i)
		if !d.IsDuplicate(id) {
			t.Errorf("retry observation of %q not flagged as duplicate", id)
		}
	}
	// Length unchanged on duplicate hits.
	if got, want := d.Len(), uint(batchSize); got != want {
		t.Errorf("len after retry: got %d, want %d", got, want)
	}
}

func TestRedisStreamDedup_Clear(t *testing.T) {
	d := NewRedisStreamDedup(8)
	defer d.Close()

	d.IsDuplicate("a")
	d.IsDuplicate("b")
	if got := d.Len(); got != 2 {
		t.Errorf("before clear: got %d, want 2", got)
	}

	d.Clear()
	if !d.IsEmpty() {
		t.Errorf("expected empty after Clear")
	}
	// Post-clear, previously-seen ids look new again.
	if d.IsDuplicate("a") {
		t.Errorf("post-clear: %q should look new", "a")
	}
}

// LRU eviction: inserting capacity+1 distinct ids drops the
// oldest. Pin both halves of the bounded-window contract: the
// non-evicted ids stay tracked, and the evicted id is reported as
// new on re-observation.
func TestRedisStreamDedup_LRUEviction(t *testing.T) {
	d := NewRedisStreamDedup(2)
	defer d.Close()

	d.IsDuplicate("a")
	d.IsDuplicate("b")
	d.IsDuplicate("c") // evicts "a"

	// "b" and "c" are still tracked.
	if !d.IsDuplicate("b") {
		t.Errorf("after eviction: %q should still be tracked", "b")
	}
	if !d.IsDuplicate("c") {
		t.Errorf("after eviction: %q should still be tracked", "c")
	}

	// "a" was evicted — re-observation looks new. (This re-inserts
	// "a", evicting "b" — which is fine for this assertion; the
	// "non-evicted ids stay" half above ran first against the
	// untouched LRU.)
	if d.IsDuplicate("a") {
		t.Errorf("after eviction: %q should be reported as new", "a")
	}
}

func TestRedisStreamDedup_NilSafety(t *testing.T) {
	var d *RedisStreamDedup
	// Every method on a nil helper is a safe no-op / sane default.
	if d.IsDuplicate("x") {
		t.Errorf("IsDuplicate on nil should return false")
	}
	if !d.IsEmpty() {
		t.Errorf("IsEmpty on nil should return true")
	}
	if d.Len() != 0 {
		t.Errorf("Len on nil should return 0")
	}
	if d.Capacity() != 0 {
		t.Errorf("Capacity on nil should return 0")
	}
	d.Clear()
	d.Close()
}

func TestRedisStreamDedup_Checked_NullHandleAfterClose(t *testing.T) {
	d := NewRedisStreamDedup(0)
	d.Close()

	dup, err := d.IsDuplicateChecked("anything")
	if dup {
		t.Errorf("IsDuplicateChecked after Close: should not report duplicate")
	}
	if err != ErrNullPointer {
		t.Errorf("IsDuplicateChecked after Close: got err=%v, want ErrNullPointer", err)
	}
}

// Cubic-ai P2: embedded NUL bytes in dedupID must be rejected
// before the C-string conversion. `C.CString` truncates at the
// first NUL, so two distinct Go strings that share a prefix and
// differ after a NUL would otherwise collide on the C side and
// be reported as the same dedup_id — silently dropping the
// second event as a "duplicate."
func TestRedisStreamDedup_RejectsEmbeddedNul(t *testing.T) {
	d := NewRedisStreamDedup(8)
	defer d.Close()

	// Two ids that share a prefix and differ AFTER a NUL.
	// Pre-fix both became `"foo"` on the C side; post-fix both
	// surface ErrInvalidDedupID before the C call.
	withNulA := "foo\x00bar"
	withNulB := "foo\x00baz"

	dup, err := d.IsDuplicateChecked(withNulA)
	if err != ErrInvalidDedupID {
		t.Errorf("expected ErrInvalidDedupID for embedded NUL, got err=%v", err)
	}
	if dup {
		t.Errorf("embedded NUL must surface as error, not duplicate (fail-open)")
	}

	dup, err = d.IsDuplicateChecked(withNulB)
	if err != ErrInvalidDedupID {
		t.Errorf("expected ErrInvalidDedupID for embedded NUL (B), got err=%v", err)
	}
	if dup {
		t.Errorf("embedded NUL (B) must surface as error, not silent duplicate")
	}

	// Neither call should have mutated the helper.
	if got := d.Len(); got != 0 {
		t.Errorf("rejected NUL ids must not enter the LRU; got len %d", got)
	}
}

// Cubic-ai P1: the Go-side mutex must protect handle lifetime
// against a concurrent Close() while a method is mid-C-call.
// Pre-fix `Close()` could free `d.handle` and nil it while another
// goroutine had already loaded the raw pointer for an in-flight
// `C.net_redis_dedup_*` call — use-after-free, UB.
//
// We can't deterministically reproduce the exact race in a unit
// test (it's microseconds wide), but a stress test running N
// goroutines hammering IsDuplicate while another races Close()
// reliably triggers the UB pre-fix on Linux/macOS under the
// race detector. Post-fix the lock makes the test deterministically
// safe — it must complete without panic, double-free, or
// data-race report.
func TestRedisStreamDedup_CloseRaceSafety(t *testing.T) {
	const goroutines = 8
	const iters = 200

	for trial := 0; trial < 5; trial++ {
		d := NewRedisStreamDedup(64)
		done := make(chan struct{}, goroutines)

		// Hammer goroutines: each does many IsDuplicate calls.
		for g := 0; g < goroutines; g++ {
			go func(g int) {
				defer func() { done <- struct{}{} }()
				for i := 0; i < iters; i++ {
					id := fmt.Sprintf("trial%d-g%d-i%d", trial, g, i)
					// Don't fail the test on results — once Close
					// races in, the answer is "false, ErrNullPointer"
					// and that's fine. We're testing for absence
					// of UB, not specific return values.
					_ = d.IsDuplicate(id)
				}
			}(g)
		}

		// Closer: races against the hammers. Sleep is omitted so
		// the close lands at an arbitrary point in the workload —
		// exactly the race we want to exercise.
		go func() {
			defer func() { done <- struct{}{} }()
			d.Close()
		}()

		// Drain. The closer counts as the last (goroutines+1)th
		// finisher.
		for i := 0; i < goroutines+1; i++ {
			<-done
		}

		// Idempotent close: must not double-free.
		d.Close()
	}
}
