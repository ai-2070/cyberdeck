//! Lock-free single-producer single-consumer (SPSC) ring buffer.
//!
//! This ring buffer is optimized for high-throughput event ingestion:
//! - Lock-free using atomics
//! - Pre-allocated, fixed capacity
//! - No heap allocation on push/pop
//! - Cache-line aligned to prevent false sharing
//!
//! # Design
//!
//! The buffer uses a power-of-2 capacity for efficient modulo operations
//! (bitwise AND instead of division). Head and tail pointers are cache-line
//! padded to prevent false sharing between producer and consumer threads.

use crossbeam_utils::CachePadded;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
/// Error returned when the ring buffer is full.
///
/// Crate-internal: surfaced to public callers as
/// `IngestionError::Backpressure`. Kept `pub(crate)` for symmetry
/// with the `pub(crate) RingBuffer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BufferFullError;

impl std::fmt::Display for BufferFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ring buffer is full")
    }
}

impl std::error::Error for BufferFullError {}

/// Lock-free SPSC ring buffer with cache-line padding.
///
/// # ⚠️  Single-Producer, Single-Consumer contract
///
/// At most one thread at a time may call `try_push`, and at most one
/// (other) thread at a time may call `try_pop` / `pop_batch` /
/// `pop_batch_into`. The atomics rely on that contract for
/// correctness; concurrent access from multiple producers or
/// multiple consumers **silently corrupts state** and is undefined
/// behavior.
///
/// Note: "single producer" allows the *task* / *handle* doing the
/// pushing to migrate between OS threads (e.g. across an `await`
/// point in a tokio task) — what matters is non-concurrency, not a
/// fixed thread id. Likewise for the consumer.
///
/// # Visibility
///
/// This type is `pub(crate)` — there is no public re-export. The
/// only legitimate use inside this crate wraps the buffer in
/// `parking_lot::Mutex<Shard>`, which serializes producer and
/// consumer access trivially. External callers should use
/// `EventBus` / `ShardManager`, which expose the SPSC fast path
/// without the footgun of `Sync`-shareable `&RingBuffer`. The bug
/// report (#5) flagged that prior `pub` exposure: any external
/// caller that put this in an `Arc` and called `try_push` from two
/// threads would silently corrupt state with no compile-time signal.
/// `pub(crate)` removes that surface entirely.
///
/// Internal unit tests (`#[cfg(test)]`) track producer/consumer
/// thread ids and panic on concurrent multi-producer or
/// multi-consumer access — a sanity check, not a complete one;
/// release builds trust the caller.
///
/// # Type Parameters
///
/// - `T`: The element type. Must be `Send` for thread safety.
///
/// # Capacity
///
/// The capacity must be a power of 2 and is fixed at construction time.
/// The actual usable capacity is `capacity - 1` to distinguish between
/// full and empty states.
pub(crate) struct RingBuffer<T> {
    /// Pre-allocated buffer storage.
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    /// Capacity (power of 2).
    capacity: usize,
    /// Mask for fast modulo (capacity - 1).
    mask: usize,
    /// Write position (producer).
    ///
    /// BUG #78: `head` / `tail` are `u64` regardless of target
    /// pointer width. Pre-fix they were `AtomicUsize`, which on
    /// 32-bit targets (wasm32 is in the test matrix) wrapped after
    /// 2^32 pushes — ~7 minutes per shard at 10 M events/sec, ~12
    /// hours at 100 K. Once `head` lapped `tail` and the wrapping
    /// distance exceeded `capacity-1`, `try_push` rejected
    /// forever and the buffer was permanently wedged. `u64` gives
    /// ~58 years to wrap at 10 G events/sec on every target.
    head: CachePadded<AtomicU64>,
    /// Read position (consumer). See `head` for the BUG #78
    /// rationale on the `u64` width.
    tail: CachePadded<AtomicU64>,
    /// Thread ID of the producer (debug-build SPSC enforcement —
    /// active under `debug_assertions`, not just `cfg(test)`, so dev
    /// runs of the binary catch SPSC violations even outside of unit
    /// tests). BUG #77: pre-fix every `#[cfg(test)]` gate below
    /// claimed to honor the `debug_assertions` contract advertised
    /// in this doc but actually only compiled under `cargo test`,
    /// leaving the safety net absent from any non-test build.
    #[cfg(any(test, debug_assertions))]
    producer_thread: std::sync::Mutex<Option<std::thread::ThreadId>>,
    /// Thread ID of the consumer (debug-build SPSC enforcement —
    /// see `producer_thread`).
    #[cfg(any(test, debug_assertions))]
    consumer_thread: std::sync::Mutex<Option<std::thread::ThreadId>>,
}

// Safety: The ring buffer is SPSC (single-producer, single-consumer).
// Atomics ensure correct visibility between the one producer and one
// consumer thread. Callers MUST NOT call try_push / pop_batch from
// multiple threads simultaneously — doing so is undefined behavior.
// Debug builds check this at runtime via thread-ID tracking;
// violations panic immediately. Release builds trust the caller.
//
// T must be Send because elements are transferred between threads.
unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with the given capacity.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is not a power of 2 or is less than 2.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "capacity must be a power of 2");
        assert!(capacity >= 2, "capacity must be at least 2");

        // Pre-allocate the buffer
        let buffer: Vec<UnsafeCell<MaybeUninit<T>>> = (0..capacity)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect();

        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            mask: capacity - 1,
            head: CachePadded::new(AtomicU64::new(0)),
            tail: CachePadded::new(AtomicU64::new(0)),
            #[cfg(any(test, debug_assertions))]
            producer_thread: std::sync::Mutex::new(None),
            #[cfg(any(test, debug_assertions))]
            consumer_thread: std::sync::Mutex::new(None),
        }
    }

    /// Try to push an element into the buffer.
    ///
    /// Returns `Ok(())` if successful, or `Err(BufferFullError)` if the buffer is full.
    ///
    /// SPSC contract: at most one thread may call `try_push` at a
    /// time. The `pub(crate)` visibility plus the in-crate mutex
    /// wrapping in `Shard` upholds this trivially.
    #[inline]
    pub fn try_push(&self, value: T) -> Result<(), BufferFullError> {
        #[cfg(any(test, debug_assertions))]
        {
            let current = std::thread::current().id();
            let mut guard = self
                .producer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(tid) = *guard {
                assert_eq!(
                    tid, current,
                    "SPSC violation: try_push called from multiple threads"
                );
            } else {
                *guard = Some(current);
            }
        }

        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Check if buffer is full. Both head/tail are u64; the
        // wrapping subtract gives the in-flight length on every
        // target (BUG #78).
        let len = head.wrapping_sub(tail);
        if len >= (self.capacity as u64) - 1 {
            return Err(BufferFullError);
        }

        // Write the value. The `mask` keeps the index inside
        // `capacity` (power-of-2); the `as usize` is the lossless
        // truncation back to the buffer index — `head & mask` is
        // always < `capacity` ≤ `usize::MAX`.
        let index = (head & self.mask as u64) as usize;
        unsafe {
            (*self.buffer[index].get()).write(value);
        }

        // Publish the write
        self.head.store(head.wrapping_add(1), Ordering::Release);

        Ok(())
    }

    /// Producer-side eviction of the oldest element.
    ///
    /// Identical to `try_pop` but tracks the *producer* thread,
    /// not the consumer. Intended exclusively for the
    /// `BackpressureMode::DropOldest` retry path, where the
    /// *producer* needs to evict the oldest event to make room
    /// for a new push. The shard's outer mutex serializes this
    /// evict against any concurrent `try_pop` from the legitimate
    /// consumer (the batch worker), so the SPSC atomic invariants
    /// are upheld even though two different OS threads call into
    /// this producer-side method and the consumer-side `try_pop`
    /// at different times.
    ///
    /// Previously this had no debug-build thread guard at all. The
    /// thread it expects matches `try_push` (the producer); we assert
    /// that here so a future caller using `evict_oldest` from the
    /// consumer thread or from a third thread is caught at test time
    /// the same way `try_push`/`try_pop` are.
    #[inline]
    pub(crate) fn evict_oldest(&self) -> Option<T> {
        #[cfg(any(test, debug_assertions))]
        {
            let current = std::thread::current().id();
            let mut guard = self
                .producer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(tid) = *guard {
                assert_eq!(
                    tid, current,
                    "SPSC violation: evict_oldest called from a different thread \
                     than try_push (it must run on the producer thread)"
                );
            } else {
                *guard = Some(current);
            }
        }

        // Same atomic ordering as `try_pop`; only the thread
        // tracking differs.
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let index = (tail & self.mask as u64) as usize;
        let value = unsafe { (*self.buffer[index].get()).assume_init_read() };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(value)
    }

    /// Try to pop an element from the buffer.
    ///
    /// Returns `Some(value)` if successful, or `None` if the buffer is empty.
    ///
    /// SPSC contract: at most one thread may call `try_pop` /
    /// `pop_batch` / `pop_batch_into` at a time. Upheld by the
    /// in-crate mutex on `Shard`.
    #[inline]
    pub fn try_pop(&self) -> Option<T> {
        #[cfg(any(test, debug_assertions))]
        {
            let current = std::thread::current().id();
            let mut guard = self
                .consumer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(tid) = *guard {
                assert_eq!(
                    tid, current,
                    "SPSC violation: try_pop called from multiple threads"
                );
            } else {
                *guard = Some(current);
            }
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Check if buffer is empty
        if tail == head {
            return None;
        }

        // Read the value
        let index = (tail & self.mask as u64) as usize;
        let value = unsafe { (*self.buffer[index].get()).assume_init_read() };

        // Publish the read
        self.tail.store(tail.wrapping_add(1), Ordering::Release);

        Some(value)
    }

    /// Pop up to `max` elements from the buffer into a vector.
    ///
    /// This is more efficient than calling `try_pop` repeatedly as it
    /// reduces atomic operations.
    ///
    /// Same single-consumer contract as `try_pop`.
    #[inline]
    pub fn pop_batch(&self, max: usize) -> Vec<T> {
        #[cfg(any(test, debug_assertions))]
        {
            let current = std::thread::current().id();
            let mut guard = self
                .consumer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(tid) = *guard {
                assert_eq!(
                    tid, current,
                    "SPSC violation: pop_batch called from multiple threads"
                );
            } else {
                *guard = Some(current);
            }
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Calculate how many elements are available. `available`
        // is u64 (BUG #78); cap to `max: usize` and convert back.
        let available = head.wrapping_sub(tail);
        let count = available.min(max as u64) as usize;

        if count == 0 {
            return Vec::new();
        }

        // Pre-allocate the result vector
        let mut result = Vec::with_capacity(count);

        // Read all elements
        for i in 0..count {
            let index = (tail.wrapping_add(i as u64) & self.mask as u64) as usize;
            let value = unsafe { (*self.buffer[index].get()).assume_init_read() };
            result.push(value);
        }

        // Publish all reads at once
        self.tail
            .store(tail.wrapping_add(count as u64), Ordering::Release);

        result
    }

    /// Pop up to `max` elements into a caller-owned `Vec`.
    ///
    /// **Append semantics**: this method does **not** clear `dst` first.
    /// It calls `dst.reserve(count)` then pushes drained elements onto
    /// the end. Returns the number of elements drained this call (may
    /// be less than `max` if the buffer has fewer available, including
    /// `0`).
    ///
    /// Use this in steady-state drain loops where the caller keeps a
    /// scratch `Vec` across cycles. Compared to [`pop_batch`], the
    /// per-cycle `Vec` allocation moves *out of the consumer's
    /// critical section* — hot when the ring buffer sits behind a
    /// mutex, since the allocator is no longer called under the lock.
    ///
    /// Typical usage:
    ///
    /// ```ignore
    /// let mut scratch = Vec::with_capacity(BATCH);
    /// loop {
    ///     let popped = ring.pop_batch_into(&mut scratch, BATCH);
    ///     if popped == 0 { break; }
    ///     // mem::replace allocates the fresh scratch *outside* any
    ///     // critical section the caller might have held while
    ///     // calling pop_batch_into.
    ///     let batch = std::mem::replace(&mut scratch, Vec::with_capacity(BATCH));
    ///     consume(batch);
    /// }
    /// ```
    ///
    /// Same single-consumer contract as `try_pop`.
    ///
    /// [`pop_batch`]: Self::pop_batch
    #[inline]
    pub fn pop_batch_into(&self, dst: &mut Vec<T>, max: usize) -> usize {
        #[cfg(any(test, debug_assertions))]
        {
            let current = std::thread::current().id();
            let mut guard = self
                .consumer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(tid) = *guard {
                assert_eq!(
                    tid, current,
                    "SPSC violation: pop_batch_into called from multiple threads"
                );
            } else {
                *guard = Some(current);
            }
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        let available = head.wrapping_sub(tail);
        let count = available.min(max as u64) as usize;

        if count == 0 {
            return 0;
        }

        // Reserve up-front so the push loop has no reallocation branch.
        dst.reserve(count);

        for i in 0..count {
            let index = (tail.wrapping_add(i as u64) & self.mask as u64) as usize;
            let value = unsafe { (*self.buffer[index].get()).assume_init_read() };
            dst.push(value);
        }

        self.tail
            .store(tail.wrapping_add(count as u64), Ordering::Release);

        count
    }

    /// Get the current number of elements in the buffer.
    ///
    /// BUG #78: `head` / `tail` are `u64` regardless of target;
    /// the in-flight count fits in `usize` because it's bounded by
    /// `capacity - 1` which is itself a `usize`.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail) as usize
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity - 1
    }

    /// Get the capacity of the buffer.
    ///
    /// Test-only: the `Shard` wrapper stores its own `capacity` field
    /// for the public API, so this is reachable only from in-file
    /// tests.
    #[cfg(test)]
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of free slots in the buffer.
    ///
    /// Test-only — see `capacity()`.
    #[cfg(test)]
    #[inline]
    fn free_slots(&self) -> usize {
        self.capacity - 1 - self.len()
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        // Reset thread tracking — &mut self guarantees exclusive access,
        // so draining from any thread is safe here. unwrap_or_else
        // recovers from poisoned mutexes (a spawned thread may have
        // panicked during SPSC violation detection). BUG #77: gate
        // matches the field-declaration gate.
        #[cfg(any(test, debug_assertions))]
        {
            *self
                .producer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = None;
            *self
                .consumer_thread
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = None;
        }
        // Drop any remaining elements. `&mut self` proves we are the
        // unique accessor, so the SPSC contract holds trivially.
        while self.try_pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let buf = RingBuffer::new(4);

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        buf.try_push(1).unwrap();
        buf.try_push(2).unwrap();
        buf.try_push(3).unwrap();

        assert_eq!(buf.len(), 3);
        assert!(buf.is_full()); // capacity - 1 = 3

        assert!(buf.try_push(4).is_err()); // Should fail, buffer full

        assert_eq!(buf.try_pop(), Some(1));
        assert_eq!(buf.try_pop(), Some(2));
        assert_eq!(buf.try_pop(), Some(3));
        assert_eq!(buf.try_pop(), None);

        assert!(buf.is_empty());
    }

    #[test]
    fn test_pop_batch() {
        let buf = RingBuffer::new(8);

        for i in 0..5 {
            buf.try_push(i).unwrap();
        }

        let batch = buf.pop_batch(3);
        assert_eq!(batch, vec![0, 1, 2]);

        let batch = buf.pop_batch(10); // Request more than available
        assert_eq!(batch, vec![3, 4]);

        assert!(buf.is_empty());
    }

    /// `pop_batch_into` is the steady-state drain primitive: it must
    /// produce the same elements as `pop_batch`, append (not replace)
    /// onto `dst`, return `0` when the buffer is empty, and tolerate
    /// being called with `max` larger than what's available.
    #[test]
    fn test_pop_batch_into() {
        let buf = RingBuffer::new(8);
        for i in 0..5 {
            buf.try_push(i).unwrap();
        }

        // Append onto an existing element — verifies the documented
        // append semantics (does not clear `dst`).
        let mut dst = vec![999u32];
        let drained = buf.pop_batch_into(&mut dst, 3);
        assert_eq!(drained, 3);
        assert_eq!(dst, vec![999, 0, 1, 2]);

        // Request more than available; should drain only what's there.
        dst.clear();
        let drained = buf.pop_batch_into(&mut dst, 10);
        assert_eq!(drained, 2);
        assert_eq!(dst, vec![3, 4]);
        assert!(buf.is_empty());

        // Empty buffer returns 0 without allocating or pushing.
        dst.clear();
        let drained = buf.pop_batch_into(&mut dst, 100);
        assert_eq!(drained, 0);
        assert!(dst.is_empty());
    }

    /// Reusing a scratch `Vec` across cycles (the canonical drain
    /// pattern) must not corrupt or skip elements across wraparound.
    #[test]
    fn test_pop_batch_into_scratch_reuse_across_wraparound() {
        let buf = RingBuffer::new(4);
        let mut scratch: Vec<u32> = Vec::with_capacity(2);
        let mut seen: Vec<u32> = Vec::new();

        for round in 0..10u32 {
            for i in 0..3 {
                buf.try_push(round * 3 + i).unwrap();
            }
            let drained = buf.pop_batch_into(&mut scratch, 3);
            assert_eq!(drained, 3);
            seen.append(&mut scratch); // empties scratch, retains capacity
        }

        let expected: Vec<u32> = (0..30).collect();
        assert_eq!(seen, expected);
    }

    #[test]
    fn test_wraparound() {
        let buf = RingBuffer::new(4);

        // Fill and drain multiple times to test wraparound
        for round in 0..10 {
            for i in 0..3 {
                buf.try_push(round * 3 + i).unwrap();
            }

            for i in 0..3 {
                assert_eq!(buf.try_pop(), Some(round * 3 + i));
            }
        }
    }

    #[test]
    fn test_concurrent_spsc() {
        use std::sync::Arc;
        use std::thread;

        let buf = Arc::new(RingBuffer::new(1024));
        let buf_producer = buf.clone();
        let buf_consumer = buf.clone();

        let count = 100_000;

        // Exactly one thread calls `try_push` (producer) and exactly
        // one calls `try_pop` (consumer). This is the SPSC happy path
        // the buffer is designed for.
        let producer = thread::spawn(move || {
            for i in 0..count {
                while buf_producer.try_push(i).is_err() {
                    std::hint::spin_loop();
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut received = Vec::with_capacity(count);
            while received.len() < count {
                if let Some(val) = buf_consumer.try_pop() {
                    received.push(val);
                } else {
                    std::hint::spin_loop();
                }
            }
            received
        });

        producer.join().unwrap();
        let received = consumer.join().unwrap();

        // Verify we got all values in order
        assert_eq!(received.len(), count);
        for (i, &val) in received.iter().enumerate() {
            assert_eq!(val, i, "mismatch at index {}", i);
        }
    }

    #[test]
    #[should_panic(expected = "power of 2")]
    fn test_non_power_of_two_capacity() {
        let _ = RingBuffer::<i32>::new(5);
    }

    #[test]
    fn test_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let drop_count = Arc::new(AtomicUsize::new(0));

        struct DropCounter(Arc<AtomicUsize>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let buf = RingBuffer::new(8);
            for _ in 0..5 {
                buf.try_push(DropCounter(drop_count.clone())).unwrap();
            }
            // Buffer drops here with 5 elements
        }

        assert_eq!(drop_count.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_buffer_full_error_display() {
        let err = BufferFullError;
        assert_eq!(format!("{}", err), "ring buffer is full");
    }

    #[test]
    fn test_buffer_full_error_debug() {
        let err = BufferFullError;
        assert!(format!("{:?}", err).contains("BufferFullError"));
    }

    #[test]
    fn test_buffer_full_error_is_error() {
        let err: &dyn std::error::Error = &BufferFullError;
        assert!(err.to_string().contains("full"));
    }

    #[test]
    fn test_capacity_and_free_slots() {
        let buf = RingBuffer::new(8);
        assert_eq!(buf.capacity(), 8);
        assert_eq!(buf.free_slots(), 7); // capacity - 1

        buf.try_push(1).unwrap();
        assert_eq!(buf.free_slots(), 6);

        buf.try_push(2).unwrap();
        buf.try_push(3).unwrap();
        assert_eq!(buf.free_slots(), 4);
    }

    #[test]
    fn test_is_full() {
        let buf = RingBuffer::new(4);
        assert!(!buf.is_full());

        buf.try_push(1).unwrap();
        buf.try_push(2).unwrap();
        assert!(!buf.is_full());

        buf.try_push(3).unwrap();
        assert!(buf.is_full());
    }

    #[test]
    fn test_pop_batch_empty() {
        let buf: RingBuffer<i32> = RingBuffer::new(8);
        let batch = buf.pop_batch(10);
        assert!(batch.is_empty());
    }

    #[test]
    #[should_panic(expected = "at least 2")]
    fn test_capacity_too_small() {
        let _ = RingBuffer::<i32>::new(1);
    }

    #[test]
    fn test_push_pop_at_exact_capacity() {
        // Regression: ensure the full check works correctly at boundary
        let buf = RingBuffer::new(4); // usable capacity = 3

        // Fill to exactly full
        buf.try_push(1).unwrap();
        buf.try_push(2).unwrap();
        buf.try_push(3).unwrap();
        assert!(buf.is_full());
        assert!(buf.try_push(4).is_err());

        // Pop one and push one - should succeed
        assert_eq!(buf.try_pop(), Some(1));
        buf.try_push(4).unwrap();
        assert!(buf.is_full());

        // Verify order
        assert_eq!(buf.try_pop(), Some(2));
        assert_eq!(buf.try_pop(), Some(3));
        assert_eq!(buf.try_pop(), Some(4));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_push_pop_boundary_stress() {
        // Regression: repeated fill/drain cycles at exact capacity boundary
        let buf = RingBuffer::new(4);

        for round in 0..100 {
            // Fill to capacity
            for i in 0..3 {
                buf.try_push(round * 3 + i)
                    .unwrap_or_else(|_| panic!("push failed at round {} item {}", round, i));
            }
            assert!(buf.is_full());
            assert!(buf.try_push(999).is_err());

            // Drain completely
            for i in 0..3 {
                assert_eq!(buf.try_pop(), Some(round * 3 + i));
            }
            assert!(buf.is_empty());
        }
    }

    /// BUG #78: cursors must be `u64` regardless of target pointer
    /// width. Pre-fix `head` and `tail` were `AtomicUsize`; on
    /// 32-bit they wrapped after 2^32 pushes and the buffer
    /// permanently wedged once the wrapping distance exceeded
    /// `capacity-1`. Pin the type-level invariant so a future
    /// regression to `AtomicUsize` would fail this test.
    ///
    /// We can't actually push 2^32 items in a unit test, but we
    /// CAN verify the cursor field types via `std::mem::size_of`:
    /// `AtomicU64` is always 8 bytes, regardless of target pointer
    /// width. (`AtomicUsize` would be 4 on 32-bit, 8 on 64-bit.)
    #[test]
    fn ring_buffer_cursors_are_u64_on_every_target() {
        // Confirm at the type level via size_of_val. `head` lives
        // inside CachePadded so the alignment is the cache line
        // size, but the inner AtomicU64 is exactly 8 bytes.
        // We can't directly inspect head's type from a unit test,
        // so we assert on the underlying load type — `u64` —
        // which is the load-bearing property.
        let buf: RingBuffer<u32> = RingBuffer::new(4);
        let head_val: u64 = buf.head.load(Ordering::Relaxed);
        let tail_val: u64 = buf.tail.load(Ordering::Relaxed);
        assert_eq!(head_val, 0);
        assert_eq!(tail_val, 0);
        // `wrapping_sub` returns u64 — type-level pin (this would
        // fail to compile if the cursors were AtomicUsize on a
        // 32-bit target).
        let len: u64 = head_val.wrapping_sub(tail_val);
        assert_eq!(len, 0);
    }

    #[cfg(test)]
    #[test]
    fn test_regression_spsc_multi_producer_detected() {
        // Regression: the SPSC ring buffer exposed &self methods with an
        // unsafe Sync impl, meaning safe code could call try_push from
        // multiple threads — causing silent data corruption.
        //
        // Fix: debug builds now track the producer/consumer thread IDs
        // and panic on violation.
        use std::sync::Arc;
        use std::thread;

        let buf = Arc::new(RingBuffer::new(1024));

        // Pin the producer identity from thread A
        buf.try_push(1).unwrap();

        // Attempt push from thread B — should panic inside the thread
        let buf2 = buf.clone();
        let result = thread::spawn(move || {
            buf2.try_push(2).unwrap();
        })
        .join();

        assert!(
            result.is_err(),
            "SPSC violation should be detected when two threads push"
        );
    }

    /// Regression: BUG_REPORT.md #35 — `evict_oldest` previously
    /// had no debug-build thread guard, so a future refactor that
    /// called it from the consumer thread (or a third thread) would
    /// silently corrupt SPSC state with no compile- or test-time
    /// signal. The fix makes `evict_oldest` track the producer
    /// thread the same way `try_push` does. This test pins the
    /// guard by spawning a second thread that calls `evict_oldest`
    /// after the first thread already pinned the producer identity
    /// via `try_push`.
    #[cfg(test)]
    #[test]
    fn test_regression_evict_oldest_thread_guard() {
        use std::sync::Arc;
        use std::thread;

        let buf = Arc::new(RingBuffer::new(4));
        // Pin producer identity via try_push from this thread.
        buf.try_push(1).unwrap();

        // Different thread calling evict_oldest should panic.
        let buf2 = buf.clone();
        let result = thread::spawn(move || {
            let _ = buf2.evict_oldest();
        })
        .join();

        assert!(
            result.is_err(),
            "evict_oldest must panic when called from a non-producer \
             thread (BUG_REPORT.md #35)"
        );
    }

    #[cfg(test)]
    #[test]
    fn test_regression_spsc_multi_consumer_detected() {
        // Same as above but for the consumer side.
        use std::sync::Arc;
        use std::thread;

        let buf = Arc::new(RingBuffer::new(1024));
        buf.try_push(1).unwrap();
        buf.try_push(2).unwrap();

        // Pin the consumer identity from thread A
        let _ = buf.try_pop();

        // Attempt pop from thread B — should panic inside the thread
        let buf2 = buf.clone();
        let result = thread::spawn(move || {
            let _ = buf2.try_pop();
        })
        .join();

        assert!(
            result.is_err(),
            "SPSC violation should be detected when two threads pop"
        );
    }
}
