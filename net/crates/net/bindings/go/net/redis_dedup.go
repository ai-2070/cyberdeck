// Consumer-side dedup helper for the Redis Streams adapter.
//
// The Net Redis adapter writes a stable `dedup_id` field on every
// XADD entry of the form
//
//	{producer_nonce:hex}:{shard_id}:{sequence_start}:{i}
//
// When the producer's MULTI/EXEC times out client-side but runs
// server-side anyway, the retry produces a duplicate stream entry
// with a distinct server-generated `*` id but an IDENTICAL
// `dedup_id`. This helper maintains an LRU-bounded set of seen
// `dedup_id`s and answers a test-and-insert query — consumers
// filter the duplicate at consume time.
//
// See [docs/BUG_AUDIT_2026_04_30_CORE.md] BUG #57 for background
// and `adapter/redis.rs` for the producer-side contract.
//
// # Example
//
//	dedup := net.NewRedisStreamDedup(0) // 0 → default 4096
//	defer dedup.Close()
//
//	// Read entries from your Redis client of choice. Pull
//	// `dedup_id` from each entry's field map.
//	for _, entry := range entries {
//	    id := entry.Fields["dedup_id"]
//	    if !dedup.IsDuplicate(id) {
//	        process(entry)
//	    }
//	}
//
// # Thread safety
//
// The C-side handle wraps a `Mutex<RedisStreamDedup>`, so concurrent
// `IsDuplicate` calls from multiple goroutines on the same helper
// serialize through the mutex. The expected shape is one helper per
// consumer goroutine — each with its own LRU window — for parallel
// throughput.
//
// # Sizing
//
// LRU capacity bounds memory and the dedup window. For ~10k
// events/sec with a 1 min out-of-order tolerance, size to
// ~600,000. The default of 4096 is suited to low-throughput /
// short-window deployments; production callers should set
// explicitly.

package net

/*
#include "net.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"runtime"
	"strings"
	"sync"
	"unsafe"
)

// ErrInvalidDedupID indicates the dedup_id passed to
// [RedisStreamDedup.IsDuplicate] was not valid UTF-8 OR contained
// an embedded NUL byte. Should never happen for ids produced by
// the Net Redis adapter (which emits ASCII-only `nonce:shard:seq:i`
// tuples); typically signals a corrupted entry on the consumer
// side.
//
// Cubic-ai P2: pre-fix the helper called `C.CString(dedupID)`
// without validating embedded NULs. `C.CString` truncates at the
// first NUL, so two distinct Go strings `"foo\x00bar"` and
// `"foo\x00baz"` both become C string `"foo"` and the helper
// would treat them as the same dedup_id — silent collision.
var ErrInvalidDedupID = errors.New("invalid dedup_id (non-UTF-8 or embedded NUL)")

// RedisStreamDedup is a consumer-side dedup helper that filters
// duplicate Redis Streams entries by their `dedup_id` field.
//
// Construct with [NewRedisStreamDedup]; release with
// [RedisStreamDedup.Close]. Concurrent use across goroutines is
// safe (each call serializes through an internal mutex), but the
// expected pattern is one helper per consumer goroutine.
//
// Cubic-ai P1: the Go-side `mu` lock guards `handle` against
// use-after-free. Pre-fix, a goroutine reading `d.handle` for a
// C call could race a concurrent `Close()` (or the
// `runtime.SetFinalizer` backstop) that frees the handle and
// nils the field; the C-side mutex inside the handle is no help
// once the handle's backing memory has been freed. The Go lock
// is held across each C call so that for the duration of the
// call, the handle cannot be freed.
type RedisStreamDedup struct {
	mu     sync.Mutex
	handle *C.net_redis_dedup_t
}

// NewRedisStreamDedup creates a helper with the given LRU
// capacity. `0` selects the default (4096). The returned helper
// must be released with [RedisStreamDedup.Close]; for callers
// that prefer GC-driven cleanup, [runtime.SetFinalizer] is wired
// up automatically as a backstop, but explicit `Close` is
// preferred (the finalizer's scheduling is non-deterministic).
func NewRedisStreamDedup(capacity uint) *RedisStreamDedup {
	h := C.net_redis_dedup_new(C.size_t(capacity))
	d := &RedisStreamDedup{handle: h}
	// Backstop: free the C handle if the caller forgets to Close.
	// Production callers should always Close explicitly — the
	// finalizer's scheduling is left to the runtime and a
	// long-lived helper that drops out of scope without Close
	// could leak its LRU memory until the next GC sweep.
	runtime.SetFinalizer(d, func(d *RedisStreamDedup) { d.Close() })
	return d
}

// Close releases the helper's C-side handle. After Close, all
// methods on this instance are no-ops (the underlying handle is
// NULL). Safe to call multiple times.
//
// Cubic-ai P1: takes the Go mutex so a concurrent in-flight
// method (already past its handle-nil guard but mid-C-call)
// completes before we free.
func (d *RedisStreamDedup) Close() {
	if d == nil {
		return
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return
	}
	C.net_redis_dedup_free(d.handle)
	d.handle = nil
	runtime.SetFinalizer(d, nil)
}

// IsDuplicate is the primary consumer entry point. Returns
// `true` if the caller should treat the entry as a DUPLICATE
// (skip it), `false` if it's the first time we've seen this
// `dedupID` (process it AND we've now marked it seen).
//
// Returns `false` (not a duplicate) on NULL handle or invalid
// UTF-8 — fail-open is the right direction here, since a missed
// dedup just means the application sees the duplicate (whatever
// safety the dedup_id provides is lost), but a false-true would
// silently drop a legitimate event. Callers that want to treat
// invalid input as a hard error can use [RedisStreamDedup.IsDuplicateChecked].
func (d *RedisStreamDedup) IsDuplicate(dedupID string) bool {
	dup, _ := d.IsDuplicateChecked(dedupID)
	return dup
}

// IsDuplicateChecked is like [RedisStreamDedup.IsDuplicate] but
// returns the underlying error from the C-side check (NULL
// handle, embedded NUL, NULL dedup_id, or invalid UTF-8) so
// callers can branch on it.
func (d *RedisStreamDedup) IsDuplicateChecked(dedupID string) (bool, error) {
	if d == nil {
		return false, ErrNullPointer
	}
	// Cubic-ai P2: reject embedded NULs BEFORE the C-string
	// conversion. `C.CString` truncates at the first NUL, so
	// without this guard `"foo\x00bar"` and `"foo\x00baz"` would
	// both serialize to the C string `"foo"` and the helper
	// would silently treat them as the same dedup_id (false
	// duplicate → dropped event). We reject upfront with a
	// dedicated error rather than letting the C call see a
	// truncated id.
	if strings.IndexByte(dedupID, 0) >= 0 {
		return false, ErrInvalidDedupID
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return false, ErrNullPointer
	}
	cID := C.CString(dedupID)
	defer C.free(unsafe.Pointer(cID))
	rc := C.net_redis_dedup_is_duplicate(d.handle, cID)
	switch rc {
	case 1:
		return true, nil
	case 0:
		return false, nil
	case -1:
		return false, ErrNullPointer
	case -2:
		return false, ErrInvalidDedupID
	default:
		return false, ErrUnknown
	}
}

// Len returns the number of distinct ids currently tracked.
// Returns 0 on a closed/nil helper.
func (d *RedisStreamDedup) Len() uint {
	if d == nil {
		return 0
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return 0
	}
	return uint(C.net_redis_dedup_len(d.handle))
}

// Capacity returns the configured LRU capacity. Returns 0 on a
// closed/nil helper.
func (d *RedisStreamDedup) Capacity() uint {
	if d == nil {
		return 0
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return 0
	}
	return uint(C.net_redis_dedup_capacity(d.handle))
}

// IsEmpty returns true if no ids are tracked yet. Also returns
// true on a closed/nil helper (mirrors the "no ids" semantic).
func (d *RedisStreamDedup) IsEmpty() bool {
	if d == nil {
		return true
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return true
	}
	return C.net_redis_dedup_is_empty(d.handle) == 1
}

// Clear drops all tracked ids. Use after a consumer-group
// rebalance to reset the dedup window without losing the helper
// instance.
func (d *RedisStreamDedup) Clear() {
	if d == nil {
		return
	}
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.handle == nil {
		return
	}
	C.net_redis_dedup_clear(d.handle)
}
