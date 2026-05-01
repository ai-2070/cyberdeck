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
