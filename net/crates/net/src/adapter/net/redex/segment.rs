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

    /// Append every payload in `payloads` in order. Returns the
    /// absolute offset the FIRST payload was written at; subsequent
    /// payloads land at successive offsets.
    ///
    /// Performs one bounds check against the total size and one
    /// `reserve` before extending — equivalent to N `append` calls
    /// but with a single capacity check and a single allocation
    /// when the buffer needs to grow.
    pub fn append_many(&mut self, payloads: &[Bytes]) -> Result<u64, RedexError> {
        let total: usize = payloads.iter().map(|p| p.len()).sum();
        if self.buf.len().saturating_add(total) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: total,
                max: MAX_SEGMENT_BYTES.saturating_sub(self.buf.len()),
            });
        }
        let first = self.base_offset + self.buf.len() as u64;
        self.buf.reserve(total);
        for p in payloads {
            self.buf.extend_from_slice(p);
        }
        Ok(first)
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

    /// Reset `base_offset` to zero without touching the buffer.
    ///
    /// Used by `RedexFile::sweep_retention` after a successful
    /// `disk.compact_to`: the on-disk dat is now rewritten to start
    /// at byte 0, so the in-memory segment must follow the same
    /// renormalization or subsequent appends will compute absolute
    /// offsets that index past the end of the new on-disk dat (BUG
    /// #92). Caller is responsible for renormalizing any external
    /// offsets (e.g. `RedexEntry::payload_offset` values stored in
    /// the index) by the prior `base_offset` value before calling
    /// this — otherwise reads through `read_at` will misalign.
    pub(super) fn rebase_to_zero(&mut self) {
        self.base_offset = 0;
    }

    /// Test-only: mutable access to the underlying byte buffer.
    /// Used by checksum-on-read regression tests to simulate
    /// on-disk corruption without going through a real I/O path.
    #[cfg(test)]
    pub(super) fn bytes_for_test_mut(&mut self) -> &mut [u8] {
        &mut self.buf
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

    #[test]
    fn test_append_many_basic() {
        let mut seg = HeapSegment::new();
        // Pre-fill so the returned offset isn't 0 — pins that
        // `append_many` honors the existing buffer length.
        let pre = seg.append(b"prefix").unwrap();
        assert_eq!(pre, 0);

        let payloads: Vec<Bytes> = vec![
            Bytes::from_static(b"alpha"),
            Bytes::from_static(b"beta"),
            Bytes::from_static(b"gamma"),
        ];
        let first = seg.append_many(&payloads).unwrap();
        assert_eq!(first, 6, "first payload offset == prefix len");

        // Each payload must be readable at first + sum(prev lens).
        assert_eq!(seg.read(6, 5).unwrap().as_ref(), b"alpha");
        assert_eq!(seg.read(11, 4).unwrap().as_ref(), b"beta");
        assert_eq!(seg.read(15, 5).unwrap().as_ref(), b"gamma");
        assert_eq!(seg.live_bytes(), 6 + 5 + 4 + 5);
    }

    #[test]
    fn test_append_many_capacity_exceeded() {
        let mut seg = HeapSegment::new();
        // One huge payload (1 GiB) so the first append succeeds, then
        // a batch whose total pushes us past `MAX_SEGMENT_BYTES` (3
        // GiB). This is the multi-payload bounds-check path that a
        // per-payload loop would not catch until partway through.
        let big = vec![0u8; 1024 * 1024 * 1024];
        seg.append(&big).unwrap();
        seg.append(&big).unwrap();
        seg.append(&big).unwrap();
        // Now at MAX exactly. A two-payload batch totaling 2 bytes
        // must still be rejected.
        let payloads: Vec<Bytes> = vec![Bytes::from_static(b"x"), Bytes::from_static(b"y")];
        assert!(matches!(
            seg.append_many(&payloads),
            Err(RedexError::PayloadTooLarge { .. })
        ));
        // And the buffer state stayed clean — no partial extension.
        assert_eq!(seg.live_bytes(), MAX_SEGMENT_BYTES);
    }

    #[test]
    fn test_append_many_empty_returns_current_end() {
        let mut seg = HeapSegment::new();
        seg.append(b"xyz").unwrap();
        // Empty batch is a no-op that returns the current end offset.
        let off = seg.append_many(&[]).unwrap();
        assert_eq!(off, 3);
        assert_eq!(seg.live_bytes(), 3);
    }
}
