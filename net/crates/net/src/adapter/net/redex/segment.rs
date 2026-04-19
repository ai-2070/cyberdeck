//! Payload storage segments for RedEX.
//!
//! v1 has one segment type: `HeapSegment`, a grow-only `Vec<u8>` payload
//! store returning `(offset, len)` for each append. Range reads return
//! `Bytes` slices over the underlying buffer.
//!
//! Reclamation (for retention) is deferred: truncation of the head of
//! the buffer happens on the retention sweep by rewriting the segment
//! plus adjusting a `base_offset` that callers subtract from stored
//! entry offsets. v1 keeps that machinery inside `RedexFile`; this
//! module just provides the primitive append+read surface.

use bytes::Bytes;

use super::error::RedexError;

/// Maximum heap segment size before `append` fails with
/// `PayloadTooLarge`. 32-bit offsets imply 4 GB hard max; we stay 1 GB
/// below to leave room for concurrent appends during a retention sweep.
pub(super) const MAX_SEGMENT_BYTES: usize = 3 * 1024 * 1024 * 1024; // 3 GB

/// In-memory payload segment backed by `Vec<u8>`.
///
/// Append-only from the caller's perspective. The retention sweep may
/// rewrite the buffer and advance `base_offset` to drop evicted heads;
/// all live offsets stored in `RedexEntry` records are absolute over
/// the logical seq-space and translated through `base_offset` on read.
#[derive(Debug)]
pub struct HeapSegment {
    buf: Vec<u8>,
    /// The absolute offset of the first byte currently in `buf`.
    /// Starts at 0 and increases as eviction compacts the head.
    base_offset: u64,
}

impl HeapSegment {
    /// Create an empty segment.
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            base_offset: 0,
        }
    }

    /// Create an empty segment with `capacity` bytes reserved.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            base_offset: 0,
        }
    }

    /// Build a segment from pre-existing payload bytes (e.g. replayed
    /// from disk). The bytes become the live region starting at
    /// absolute offset 0.
    #[cfg(feature = "redex-disk")]
    pub(super) fn from_existing(buf: Vec<u8>) -> Self {
        Self {
            buf,
            base_offset: 0,
        }
    }

    /// Append `payload` and return the absolute offset it was written
    /// at (offset in the logical seq-space, not in the backing `Vec`).
    pub fn append(&mut self, payload: &[u8]) -> Result<u64, RedexError> {
        if self.buf.len().saturating_add(payload.len()) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: payload.len(),
                max: MAX_SEGMENT_BYTES.saturating_sub(self.buf.len()),
            });
        }
        let offset = self.base_offset + self.buf.len() as u64;
        self.buf.extend_from_slice(payload);
        Ok(offset)
    }

    /// Read `len` bytes starting at absolute `offset`. Returns `None`
    /// if the slice is not fully contained in the live region
    /// (evicted or past end).
    pub fn read(&self, offset: u64, len: u32) -> Option<Bytes> {
        let len = len as usize;
        if offset < self.base_offset {
            return None;
        }
        let rel = (offset - self.base_offset) as usize;
        let end = rel.checked_add(len)?;
        if end > self.buf.len() {
            return None;
        }
        Some(Bytes::copy_from_slice(&self.buf[rel..end]))
    }

    /// Number of live bytes currently in the segment.
    pub fn live_bytes(&self) -> usize {
        self.buf.len()
    }

    /// Absolute offset of the first live byte. Anything below this has
    /// been evicted.
    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    /// Test-only: forcibly set the absolute base offset without
    /// touching the buffer. Used to simulate a long-lifetime file
    /// where eviction has pushed `base_offset` near `u32::MAX`,
    /// triggering the overflow path in `file.rs::offset_to_u32` and
    /// the pre-validation in `append_batch` / `append_batch_ordered`.
    #[cfg(test)]
    pub(super) fn force_base_offset(&mut self, base: u64) {
        self.base_offset = base;
    }

    /// Evict the prefix of the segment strictly below `new_base` in
    /// the absolute offset space.
    ///
    /// Returns the number of bytes evicted.
    pub fn evict_prefix_to(&mut self, new_base: u64) -> u64 {
        if new_base <= self.base_offset {
            return 0;
        }
        let delta = (new_base - self.base_offset) as usize;
        let delta = delta.min(self.buf.len());
        self.buf.drain(..delta);
        self.base_offset += delta as u64;
        delta as u64
    }
}

impl Default for HeapSegment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_read() {
        let mut seg = HeapSegment::new();
        let o1 = seg.append(b"hello").unwrap();
        let o2 = seg.append(b"world").unwrap();
        assert_eq!(o1, 0);
        assert_eq!(o2, 5);

        assert_eq!(seg.read(o1, 5).unwrap().as_ref(), b"hello");
        assert_eq!(seg.read(o2, 5).unwrap().as_ref(), b"world");
    }

    #[test]
    fn test_read_out_of_range_returns_none() {
        let mut seg = HeapSegment::new();
        seg.append(b"abc").unwrap();
        assert!(seg.read(10, 3).is_none()); // offset past end
        assert!(seg.read(0, 10).is_none()); // len overruns
    }

    #[test]
    fn test_evict_prefix() {
        let mut seg = HeapSegment::new();
        seg.append(b"aaaa").unwrap();
        let o2 = seg.append(b"bbbb").unwrap();
        assert_eq!(o2, 4);

        let evicted = seg.evict_prefix_to(4);
        assert_eq!(evicted, 4);
        assert_eq!(seg.base_offset(), 4);
        assert_eq!(seg.live_bytes(), 4);

        // Old offset 0 is now below base
        assert!(seg.read(0, 4).is_none());
        // New read from o2 still works
        assert_eq!(seg.read(o2, 4).unwrap().as_ref(), b"bbbb");
    }

    #[test]
    fn test_evict_below_base_is_noop() {
        let mut seg = HeapSegment::new();
        seg.append(b"abc").unwrap();
        seg.evict_prefix_to(10);
        assert_eq!(seg.base_offset(), 3);
        // Further eviction below current base does nothing.
        assert_eq!(seg.evict_prefix_to(1), 0);
    }

    #[test]
    fn test_evict_beyond_live_clamps() {
        let mut seg = HeapSegment::new();
        seg.append(b"xyz").unwrap();
        // Eviction beyond the tail should clamp to tail without panic.
        let evicted = seg.evict_prefix_to(100);
        assert_eq!(evicted, 3);
        assert_eq!(seg.base_offset(), 3);
        assert_eq!(seg.live_bytes(), 0);
    }
}
