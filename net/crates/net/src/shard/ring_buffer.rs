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
use std::sync::atomic::{AtomicUsize, Ordering};

/// Error returned when the ring buffer is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFullError;

impl std::fmt::Display for BufferFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ring buffer is full")
    }
}

impl std::error::Error for BufferFullError {}

/// Lock-free SPSC ring buffer with cache-line padding.
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
pub struct RingBuffer<T> {
    /// Pre-allocated buffer storage.
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    /// Capacity (power of 2).
    capacity: usize,
    /// Mask for fast modulo (capacity - 1).
    mask: usize,
    /// Write position (producer).
    head: CachePadded<AtomicUsize>,
    /// Read position (consumer).
    tail: CachePadded<AtomicUsize>,
}

// Safety: We ensure proper synchronization via atomics.
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
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    /// Try to push an element into the buffer.
    ///
    /// Returns `Ok(())` if successful, or `Err(BufferFullError)` if the buffer is full.
    ///
    /// # Safety
    ///
    /// This is safe for a single producer thread. Multiple producers require
    /// external synchronization.
    #[inline]
    pub fn try_push(&self, value: T) -> Result<(), BufferFullError> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Check if buffer is full
        let next_head = head.wrapping_add(1);
        if (next_head & self.mask) == (tail & self.mask) && next_head != tail {
            // Actually check the count
            let len = head.wrapping_sub(tail);
            if len >= self.capacity - 1 {
                return Err(BufferFullError);
            }
        }

        // Calculate actual length
        let len = head.wrapping_sub(tail);
        if len >= self.capacity - 1 {
            return Err(BufferFullError);
        }

        // Write the value
        let index = head & self.mask;
        unsafe {
            (*self.buffer[index].get()).write(value);
        }

        // Publish the write
        self.head.store(head.wrapping_add(1), Ordering::Release);

        Ok(())
    }

    /// Try to pop an element from the buffer.
    ///
    /// Returns `Some(value)` if successful, or `None` if the buffer is empty.
    ///
    /// # Safety
    ///
    /// This is safe for a single consumer thread. Multiple consumers require
    /// external synchronization.
    #[inline]
    pub fn try_pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Check if buffer is empty
        if tail == head {
            return None;
        }

        // Read the value
        let index = tail & self.mask;
        let value = unsafe { (*self.buffer[index].get()).assume_init_read() };

        // Publish the read
        self.tail.store(tail.wrapping_add(1), Ordering::Release);

        Some(value)
    }

    /// Pop up to `max` elements from the buffer into a vector.
    ///
    /// This is more efficient than calling `try_pop` repeatedly as it
    /// reduces atomic operations.
    #[inline]
    pub fn pop_batch(&self, max: usize) -> Vec<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Calculate how many elements are available
        let available = head.wrapping_sub(tail);
        let count = available.min(max);

        if count == 0 {
            return Vec::new();
        }

        // Pre-allocate the result vector
        let mut result = Vec::with_capacity(count);

        // Read all elements
        for i in 0..count {
            let index = (tail.wrapping_add(i)) & self.mask;
            let value = unsafe { (*self.buffer[index].get()).assume_init_read() };
            result.push(value);
        }

        // Publish all reads at once
        self.tail.store(tail.wrapping_add(count), Ordering::Release);

        result
    }

    /// Get the current number of elements in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
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
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of free slots in the buffer.
    #[inline]
    pub fn free_slots(&self) -> usize {
        self.capacity - 1 - self.len()
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        // Drop any remaining elements
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
}
