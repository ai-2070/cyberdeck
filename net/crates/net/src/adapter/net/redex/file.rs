//! `RedexFile` — the append / tail / read_range primitive.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use futures::Stream;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::super::channel::ChannelName;
use super::config::RedexFileConfig;
use super::entry::{payload_checksum, RedexEntry, INLINE_PAYLOAD_SIZE};
use super::error::RedexError;
use super::event::RedexEvent;
use super::retention::compute_eviction_count;
use super::segment::{HeapSegment, MAX_SEGMENT_BYTES};

#[cfg(feature = "redex-disk")]
use super::disk::DiskSegment;
#[cfg(feature = "redex-disk")]
use std::path::Path;

/// A live tail subscription waiting for new events.
struct TailWatcher {
    /// Minimum seq to deliver (inclusive).
    from_seq: u64,
    /// Channel back to the subscriber.
    sender: mpsc::UnboundedSender<Result<RedexEvent, RedexError>>,
}

/// Mutable state: index, parallel timestamps, segment, and live
/// watchers. All held behind a single lock so the backfill→register
/// handoff is atomic.
struct FileState {
    index: Vec<RedexEntry>,
    /// Per-entry unix-nanos timestamps captured at append time.
    /// Same length as `index`. Used by age-based retention.
    /// In-memory only — not persisted to disk in v1; on reopen of
    /// a persistent file, recovered entries get "now" as their
    /// fake timestamp.
    timestamps: Vec<u64>,
    segment: HeapSegment,
    watchers: Vec<TailWatcher>,
}

/// Shared inner state. Handles are cheap `Arc` clones of this.
struct RedexFileInner {
    name: ChannelName,
    config: RedexFileConfig,
    next_seq: AtomicU64,
    state: Mutex<FileState>,
    closed: AtomicBool,
    #[cfg(feature = "redex-disk")]
    disk: Option<Arc<DiskSegment>>,
}

/// A handle to a RedEX file. Cheap to clone.
///
/// Created via [`super::Redex::open_file`].
#[derive(Clone)]
pub struct RedexFile {
    inner: Arc<RedexFileInner>,
}

impl RedexFile {
    /// Create a fresh, empty file. Called by `Redex::open_file`.
    pub(super) fn new(name: ChannelName, config: RedexFileConfig) -> Self {
        let capacity = config.max_memory_bytes.min(64 * 1024 * 1024);
        Self {
            inner: Arc::new(RedexFileInner {
                name,
                config,
                next_seq: AtomicU64::new(0),
                state: Mutex::new(FileState {
                    index: Vec::new(),
                    timestamps: Vec::new(),
                    segment: HeapSegment::with_capacity(capacity),
                    watchers: Vec::new(),
                }),
                closed: AtomicBool::new(false),
                #[cfg(feature = "redex-disk")]
                disk: None,
            }),
        }
    }

    /// Open (or recover) a file with disk-backed durability.
    ///
    /// Reads `<base_dir>/<channel_path>/idx` and `.../dat` if they
    /// exist, replays the full dat file into the heap segment, and
    /// sets `next_seq` to one past the last recovered entry. New
    /// appends are mirrored to disk.
    ///
    /// A partial trailing record in `idx` (torn write from a crash)
    /// is truncated on reopen.
    #[cfg(feature = "redex-disk")]
    pub(super) fn open_persistent(
        name: ChannelName,
        config: RedexFileConfig,
        base_dir: &Path,
    ) -> Result<Self, RedexError> {
        let recovered = DiskSegment::open(base_dir, &name)?;
        let next_seq = recovered.index.last().map(|e| e.seq + 1).unwrap_or(0);

        let segment = HeapSegment::from_existing(recovered.payload_bytes);
        // Recovered entries get "now" as their fake timestamp. v1
        // age-retention limitation: persistent files lose age info
        // across reopen. v2 mmap tier will persist timestamps.
        let now = now_ns();
        let timestamps = vec![now; recovered.index.len()];
        let state = FileState {
            index: recovered.index,
            timestamps,
            segment,
            watchers: Vec::new(),
        };

        Ok(Self {
            inner: Arc::new(RedexFileInner {
                name,
                config,
                next_seq: AtomicU64::new(next_seq),
                state: Mutex::new(state),
                closed: AtomicBool::new(false),
                disk: Some(Arc::new(recovered.disk)),
            }),
        })
    }

    /// The channel name this file is bound to.
    #[inline]
    pub fn name(&self) -> &ChannelName {
        &self.inner.name
    }

    /// The config this file was opened with.
    #[inline]
    pub fn config(&self) -> &RedexFileConfig {
        &self.inner.config
    }

    /// Number of currently retained entries.
    pub fn len(&self) -> usize {
        self.inner.state.lock().index.len()
    }

    /// True if no entries are retained.
    pub fn is_empty(&self) -> bool {
        self.inner.state.lock().index.is_empty()
    }

    /// Next sequence to be assigned (== total append count since open,
    /// including any evicted head).
    pub fn next_seq(&self) -> u64 {
        self.inner.next_seq.load(Ordering::Acquire)
    }

    /// Append one event. Returns the assigned sequence.
    ///
    /// Failure modes: `PayloadTooLarge` if the segment is full or the
    /// offset would overflow u32; `Io(_)` if the disk mirror fails
    /// under `redex-disk`. Failure atomicity: memory is committed
    /// only after the disk write succeeds (for persistent files);
    /// `next_seq` is rolled back on disk failure so no seq number is
    /// burnt and no in-memory entry diverges from disk.
    pub fn append(&self, payload: &[u8]) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        let cks = payload_checksum(payload);
        let ts = now_ns();

        let mut state = self.inner.state.lock();

        // Pre-validate capacity + offset width under the state lock —
        // no side effects yet.
        let current_live = state.segment.live_bytes();
        if current_live.saturating_add(payload.len()) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: payload.len(),
                max: MAX_SEGMENT_BYTES.saturating_sub(current_live),
            });
        }
        let offset = state
            .segment
            .base_offset()
            .saturating_add(current_live as u64);
        let offset_u32 = offset_to_u32(offset)?;

        // Allocate seq only after validation, under the state lock.
        // Concurrent writers also take the lock, so if we roll back
        // on disk failure no other writer has observed our value.
        let seq = self.inner.next_seq.fetch_add(1, Ordering::AcqRel);
        let entry = RedexEntry::new_heap(seq, offset_u32, payload.len() as u32, 0, cks);

        // Disk FIRST. If it fails, roll back the seq allocation and
        // leave memory untouched so callers don't observe a record
        // that was never durably persisted.
        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            if let Err(e) = disk.append_entry(&entry, payload) {
                self.inner.next_seq.fetch_sub(1, Ordering::AcqRel);
                return Err(e);
            }
        }

        // Commit to memory. `segment.append` is infallible here —
        // we pre-validated its capacity above.
        state
            .segment
            .append(payload)
            .expect("pre-validated capacity; segment append cannot fail");
        state.index.push(entry);
        state.timestamps.push(ts);

        let event = RedexEvent {
            entry,
            payload: Bytes::copy_from_slice(payload),
        };
        notify_watchers(&mut state.watchers, &event);

        Ok(seq)
    }

    /// Append a fixed-length 8-byte inline payload. Skips the segment
    /// indirection. Returns the assigned sequence. Same failure-
    /// atomicity contract as [`Self::append`].
    pub fn append_inline(&self, payload: &[u8; INLINE_PAYLOAD_SIZE]) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        let cks = payload_checksum(payload);
        let ts = now_ns();

        let mut state = self.inner.state.lock();

        let seq = self.inner.next_seq.fetch_add(1, Ordering::AcqRel);
        let entry = RedexEntry::new_inline(seq, payload, cks);

        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            if let Err(e) = disk.append_entry(&entry, payload) {
                self.inner.next_seq.fetch_sub(1, Ordering::AcqRel);
                return Err(e);
            }
        }

        state.index.push(entry);
        state.timestamps.push(ts);

        let event = RedexEvent {
            entry,
            payload: Bytes::copy_from_slice(payload),
        };
        notify_watchers(&mut state.watchers, &event);

        Ok(seq)
    }

    /// Append many payloads. Returns the sequence of the FIRST event
    /// in the batch. All entries land contiguously in the index.
    ///
    /// Failure atomicity:
    /// - seq numbers are allocated **after** the batch is validated
    ///   to fit (both segment capacity and u32 offset width);
    /// - for persistent files, the batch is written to disk **before**
    ///   any in-memory commit — on disk failure the seq allocation
    ///   rolls back and neither memory nor subscribers observe the
    ///   batch.
    pub fn append_batch(&self, payloads: &[Bytes]) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        if payloads.is_empty() {
            return Ok(self.inner.next_seq.load(Ordering::Acquire));
        }

        let ts = now_ns();
        let mut state = self.inner.state.lock();

        // Pre-validate: every payload must fit in the remaining
        // segment capacity, and the final offset must fit in a u32.
        // Under the state lock, nothing else can write to the
        // segment, so the check-then-act is atomic.
        let total_bytes: usize = payloads.iter().map(|p| p.len()).sum();
        let current_live = state.segment.live_bytes();
        if current_live.saturating_add(total_bytes) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: total_bytes,
                max: MAX_SEGMENT_BYTES.saturating_sub(current_live),
            });
        }
        let base = state
            .segment
            .base_offset()
            .saturating_add(current_live as u64);
        let final_offset = base.saturating_add(total_bytes as u64);
        offset_to_u32(final_offset)?;

        // Allocate the contiguous seq range under the lock so that
        // a rollback on disk failure is safe — no other writer can
        // advance past our allocation while we hold the state lock.
        let first_seq = self
            .inner
            .next_seq
            .fetch_add(payloads.len() as u64, Ordering::AcqRel);

        // Pre-compute entries + running offsets without touching
        // the segment yet. This lets us write the batch to disk
        // before committing any in-memory state.
        let mut events: Vec<RedexEvent> = Vec::with_capacity(payloads.len());
        let mut running = base;
        for (i, payload) in payloads.iter().enumerate() {
            let seq = first_seq + i as u64;
            let cks = payload_checksum(payload);
            let entry = RedexEntry::new_heap(
                seq,
                offset_to_u32(running).expect("pre-validated final offset fits u32"),
                payload.len() as u32,
                0,
                cks,
            );
            running = running.saturating_add(payload.len() as u64);
            events.push(RedexEvent {
                entry,
                payload: payload.clone(),
            });
        }

        // Disk FIRST. Roll back the seq range on failure so no seq
        // is burnt and memory stays clean.
        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            let pairs: Vec<(RedexEntry, &[u8])> = events
                .iter()
                .map(|e| (e.entry, e.payload.as_ref()))
                .collect();
            if let Err(e) = disk.append_entries(&pairs) {
                self.inner
                    .next_seq
                    .fetch_sub(payloads.len() as u64, Ordering::AcqRel);
                return Err(e);
            }
        }

        // Commit to memory. Pre-validated capacity → infallible.
        for event in &events {
            state
                .segment
                .append(&event.payload)
                .expect("pre-validated capacity; segment append cannot fail under the state lock");
            state.index.push(event.entry);
            state.timestamps.push(ts);
        }

        for event in &events {
            notify_watchers(&mut state.watchers, event);
        }
        Ok(first_seq)
    }

    /// Like [`Self::append`] but holds the state lock across seq
    /// allocation. Guarantees that the index records entries in
    /// strict seq order, even under concurrent writers that use the
    /// ordered path. The non-ordered `append` path allocates seq via
    /// a lock-free `fetch_add` before taking the lock — fast but can
    /// produce out-of-seq-order index insertions under contention.
    ///
    /// Used by [`super::OrderedAppender`] for replay determinism.
    /// Same failure-atomicity contract as [`Self::append`].
    pub fn append_ordered(&self, payload: &[u8]) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        let cks = payload_checksum(payload);
        let ts = now_ns();

        let mut state = self.inner.state.lock();

        let current_live = state.segment.live_bytes();
        if current_live.saturating_add(payload.len()) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: payload.len(),
                max: MAX_SEGMENT_BYTES.saturating_sub(current_live),
            });
        }
        let offset = state
            .segment
            .base_offset()
            .saturating_add(current_live as u64);
        let offset_u32 = offset_to_u32(offset)?;

        let seq = self.inner.next_seq.fetch_add(1, Ordering::AcqRel);
        let entry = RedexEntry::new_heap(seq, offset_u32, payload.len() as u32, 0, cks);

        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            if let Err(e) = disk.append_entry(&entry, payload) {
                self.inner.next_seq.fetch_sub(1, Ordering::AcqRel);
                return Err(e);
            }
        }

        state
            .segment
            .append(payload)
            .expect("pre-validated capacity; segment append cannot fail");
        state.index.push(entry);
        state.timestamps.push(ts);

        let event = RedexEvent {
            entry,
            payload: Bytes::copy_from_slice(payload),
        };
        notify_watchers(&mut state.watchers, &event);

        Ok(seq)
    }

    /// Ordered variant of [`Self::append_inline`]. See
    /// [`Self::append_ordered`]. Same failure-atomicity contract.
    pub fn append_inline_ordered(
        &self,
        payload: &[u8; INLINE_PAYLOAD_SIZE],
    ) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        let cks = payload_checksum(payload);
        let ts = now_ns();

        let mut state = self.inner.state.lock();
        let seq = self.inner.next_seq.fetch_add(1, Ordering::AcqRel);
        let entry = RedexEntry::new_inline(seq, payload, cks);

        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            if let Err(e) = disk.append_entry(&entry, payload) {
                self.inner.next_seq.fetch_sub(1, Ordering::AcqRel);
                return Err(e);
            }
        }

        state.index.push(entry);
        state.timestamps.push(ts);

        let event = RedexEvent {
            entry,
            payload: Bytes::copy_from_slice(payload),
        };
        notify_watchers(&mut state.watchers, &event);

        Ok(seq)
    }

    /// Ordered variant of [`Self::append_batch`]. The whole batch is
    /// appended under one state-lock acquisition, so it's both
    /// atomic (all-or-nothing within the batch) and strictly
    /// seq-ordered relative to any other ordered writers. Same
    /// failure-atomicity contract as [`Self::append_batch`].
    pub fn append_batch_ordered(&self, payloads: &[Bytes]) -> Result<u64, RedexError> {
        self.check_not_closed()?;
        if payloads.is_empty() {
            return Ok(self.inner.next_seq.load(Ordering::Acquire));
        }
        let ts = now_ns();
        let mut state = self.inner.state.lock();

        // Pre-validate capacity + offset width before allocating
        // seq numbers — see [`Self::append_batch`] for rationale.
        let total_bytes: usize = payloads.iter().map(|p| p.len()).sum();
        let current_live = state.segment.live_bytes();
        if current_live.saturating_add(total_bytes) > MAX_SEGMENT_BYTES {
            return Err(RedexError::PayloadTooLarge {
                size: total_bytes,
                max: MAX_SEGMENT_BYTES.saturating_sub(current_live),
            });
        }
        let base = state
            .segment
            .base_offset()
            .saturating_add(current_live as u64);
        let final_offset = base.saturating_add(total_bytes as u64);
        offset_to_u32(final_offset)?;

        let first_seq = self
            .inner
            .next_seq
            .fetch_add(payloads.len() as u64, Ordering::AcqRel);

        // Pre-compute entries without touching the segment yet.
        let mut events: Vec<RedexEvent> = Vec::with_capacity(payloads.len());
        let mut running = base;
        for (i, payload) in payloads.iter().enumerate() {
            let seq = first_seq + i as u64;
            let cks = payload_checksum(payload);
            let entry = RedexEntry::new_heap(
                seq,
                offset_to_u32(running).expect("pre-validated final offset fits u32"),
                payload.len() as u32,
                0,
                cks,
            );
            running = running.saturating_add(payload.len() as u64);
            events.push(RedexEvent {
                entry,
                payload: payload.clone(),
            });
        }

        // Disk FIRST. Roll back seq range on failure.
        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            let pairs: Vec<(RedexEntry, &[u8])> = events
                .iter()
                .map(|e| (e.entry, e.payload.as_ref()))
                .collect();
            if let Err(e) = disk.append_entries(&pairs) {
                self.inner
                    .next_seq
                    .fetch_sub(payloads.len() as u64, Ordering::AcqRel);
                return Err(e);
            }
        }

        // Commit to memory.
        for event in &events {
            state
                .segment
                .append(&event.payload)
                .expect("pre-validated capacity; segment append cannot fail under the state lock");
            state.index.push(event.entry);
            state.timestamps.push(ts);
        }

        for event in &events {
            notify_watchers(&mut state.watchers, event);
        }
        Ok(first_seq)
    }

    /// Append `value` (bincode-serialized) AND run `fold_fn` against
    /// caller-supplied `state` in the same call. Returns the
    /// assigned seq.
    ///
    /// This is the common "log-an-event and update a materialized
    /// view in one step" pattern, without spinning up a full
    /// [`super::super::cortex`] adapter. Callers maintain `state`
    /// themselves; the fold closure sees the just-appended value
    /// and mutates state in place.
    ///
    /// Note: the fold runs AFTER the RedEX append. If the append
    /// succeeds and the fold panics, the log advances but state is
    /// out of sync — callers who need crash-consistency should use
    /// the CortEX adapter's durable `snapshot` + `open_from_snapshot`
    /// instead.
    pub fn append_and_fold<T, F, S>(
        &self,
        value: &T,
        state: &mut S,
        fold_fn: F,
    ) -> Result<u64, RedexError>
    where
        T: serde::Serialize,
        F: FnOnce(&T, &mut S),
    {
        let bytes = bincode::serialize(value)
            .map_err(|e| RedexError::Encode(format!("append_and_fold serialize: {}", e)))?;
        let seq = self.append(&bytes)?;
        fold_fn(value, state);
        Ok(seq)
    }

    /// Convenience: serialize `value` with bincode and append.
    pub fn append_bincode<T: serde::Serialize>(&self, value: &T) -> Result<u64, RedexError> {
        let bytes = bincode::serialize(value).map_err(|e| RedexError::Encode(e.to_string()))?;
        self.append(&bytes)
    }

    /// Subscribe to all events with `seq >= from_seq`, including those
    /// already in the index at call time.
    ///
    /// Backfill and live registration happen atomically under the
    /// state lock: no event can interleave between backfill delivery
    /// and live subscription.
    pub fn tail(
        &self,
        from_seq: u64,
    ) -> impl Stream<Item = Result<RedexEvent, RedexError>> + Send + 'static {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut state = self.inner.state.lock();

        // Check `closed` inside the state lock: close() drains the
        // watcher list under this same lock, so any watcher we register
        // after clearing this check is guaranteed to either (a) be
        // seen by a future close() drain, or (b) predate the close.
        // Checking outside the lock is a TOCTOU — close() could flip
        // the flag + drain before we register here, leaving the
        // subscriber hanging with no `Closed` signal.
        if self.inner.closed.load(Ordering::Acquire) {
            drop(state);
            let _ = tx.send(Err(RedexError::Closed));
            return UnboundedReceiverStream::new(rx);
        }

        // Backfill.
        for entry in state.index.iter() {
            if entry.seq < from_seq {
                continue;
            }
            let event = match materialize(entry, &state.segment) {
                Some(e) => e,
                None => continue, // payload evicted between index retain and read
            };
            if tx.send(Ok(event)).is_err() {
                // Receiver dropped before stream was even used.
                return UnboundedReceiverStream::new(rx);
            }
        }

        // Register for live events.
        state.watchers.push(TailWatcher {
            from_seq,
            sender: tx,
        });
        drop(state);

        UnboundedReceiverStream::new(rx)
    }

    /// One-shot read of the half-open range `[start, end)` from the
    /// in-memory index. Returns only entries currently retained;
    /// silently skips any seqs that have been evicted.
    pub fn read_range(&self, start: u64, end: u64) -> Vec<RedexEvent> {
        if end <= start {
            return Vec::new();
        }
        let state = self.inner.state.lock();
        let mut out = Vec::new();
        for entry in state.index.iter() {
            if entry.seq < start {
                continue;
            }
            if entry.seq >= end {
                break;
            }
            if let Some(ev) = materialize(entry, &state.segment) {
                out.push(ev);
            }
        }
        out
    }

    /// Run the retention policy synchronously. Exposed so a background
    /// task (heartbeat loop) can drive it; no hot-path cost.
    pub fn sweep_retention(&self) {
        let cfg = self.inner.config;
        if cfg.retention_max_events.is_none()
            && cfg.retention_max_bytes.is_none()
            && cfg.retention_max_age_ns.is_none()
        {
            return;
        }
        let now = now_ns();
        let mut state = self.inner.state.lock();
        let drop = compute_eviction_count(&state.index, &state.timestamps, now, &cfg);
        if drop == 0 {
            return;
        }

        // Determine the new segment base: first offset of the entry
        // that survives. Inline entries don't consume segment bytes,
        // so skip past them when finding the boundary.
        let mut new_base: Option<u64> = None;
        for e in state.index.iter().skip(drop) {
            if !e.is_inline() {
                new_base = Some(e.payload_offset as u64);
                break;
            }
        }

        // Drop the head of the index AND the parallel timestamps.
        state.index.drain(..drop);
        state.timestamps.drain(..drop);

        // Evict the payload prefix up to new_base (or everything if
        // surviving entries are all inline).
        if let Some(base) = new_base {
            state.segment.evict_prefix_to(base);
        } else {
            // All surviving entries are inline; reclaim the whole
            // payload segment.
            let cur = state.segment.base_offset() + state.segment.live_bytes() as u64;
            state.segment.evict_prefix_to(cur);
        }
    }

    /// Close the file. Outstanding tail streams receive `RedexError::Closed`.
    /// For persistent files, fsyncs the disk segment before returning.
    pub fn close(&self) -> Result<(), RedexError> {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let mut state = self.inner.state.lock();
        for w in state.watchers.drain(..) {
            let _ = w.sender.send(Err(RedexError::Closed));
        }
        drop(state);

        #[cfg(feature = "redex-disk")]
        if let Some(disk) = self.disk() {
            disk.sync()?;
        }

        Ok(())
    }

    /// Fsync the disk segment (no-op for heap-only files).
    #[cfg(feature = "redex-disk")]
    pub fn sync(&self) -> Result<(), RedexError> {
        if let Some(disk) = self.disk() {
            disk.sync()?;
        }
        Ok(())
    }

    #[cfg(feature = "redex-disk")]
    #[inline]
    fn disk(&self) -> Option<&Arc<DiskSegment>> {
        self.inner.disk.as_ref()
    }

    #[inline]
    fn check_not_closed(&self) -> Result<(), RedexError> {
        if self.inner.closed.load(Ordering::Acquire) {
            Err(RedexError::Closed)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Debug for RedexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedexFile")
            .field("name", &self.inner.name)
            .field("len", &self.len())
            .field("next_seq", &self.next_seq())
            .finish()
    }
}

// -- helpers ---------------------------------------------------------------

fn materialize(entry: &RedexEntry, segment: &HeapSegment) -> Option<RedexEvent> {
    let payload = if entry.is_inline() {
        Bytes::copy_from_slice(&entry.inline_payload()?)
    } else {
        segment.read(entry.payload_offset as u64, entry.payload_len)?
    };
    Some(RedexEvent {
        entry: *entry,
        payload,
    })
}

fn notify_watchers(watchers: &mut Vec<TailWatcher>, event: &RedexEvent) {
    // Walk watchers; drop those whose receiver is gone.
    watchers.retain(|w| {
        if event.entry.seq < w.from_seq {
            return true; // keep, but don't deliver
        }
        w.sender.send(Ok(event.clone())).is_ok()
    });
}

/// Convert a segment offset to the `u32` field that lives in
/// `RedexEntry::payload_offset`. Returns
/// `Err(SegmentOffsetOverflow { offset })` if the absolute segment
/// offset has passed `u32::MAX` — this can only happen on a
/// persistent file whose lifetime heap bytes (append + eviction +
/// re-append) have crossed the 4 GB threshold. The segment itself
/// caps at 3 GB live, so this fires on `base_offset` growth, not
/// live-data growth. The right long-term fix is a sweep-time offset
/// renormalization (v2); until then we surface the overflow instead
/// of silently truncating.
#[inline]
fn offset_to_u32(offset: u64) -> Result<u32, RedexError> {
    u32::try_from(offset).map_err(|_| RedexError::SegmentOffsetOverflow { offset })
}

/// Unix nanoseconds from `SystemTime::now`. Used as the per-entry
/// timestamp for age-based retention. Non-monotonic (wall clock)
/// is acceptable here — retention only needs rough ordering, not
/// strict monotonicity.
#[inline]
fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::super::super::channel::ChannelName;
    use super::*;
    use futures::StreamExt;

    fn make_file(name: &str) -> RedexFile {
        RedexFile::new(ChannelName::new(name).unwrap(), RedexFileConfig::default())
    }

    #[test]
    fn test_append_assigns_monotonic_seq() {
        let f = make_file("t1");
        assert_eq!(f.append(b"a").unwrap(), 0);
        assert_eq!(f.append(b"b").unwrap(), 1);
        assert_eq!(f.append(b"c").unwrap(), 2);
        assert_eq!(f.next_seq(), 3);
    }

    #[test]
    fn test_read_range_returns_events_in_order() {
        let f = make_file("t2");
        for i in 0..10u64 {
            f.append(format!("e{}", i).as_bytes()).unwrap();
        }
        let events = f.read_range(2, 5);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].entry.seq, 2);
        assert_eq!(events[0].payload.as_ref(), b"e2");
        assert_eq!(events[2].entry.seq, 4);
    }

    #[test]
    fn test_read_range_empty_when_end_le_start() {
        let f = make_file("t2e");
        f.append(b"x").unwrap();
        assert!(f.read_range(5, 5).is_empty());
        assert!(f.read_range(10, 3).is_empty());
    }

    #[test]
    fn test_append_batch_sequential() {
        let f = make_file("t3");
        let start = f
            .append_batch(&[Bytes::from_static(b"one"), Bytes::from_static(b"two")])
            .unwrap();
        assert_eq!(start, 0);
        assert_eq!(f.next_seq(), 2);
        let events = f.read_range(0, 2);
        assert_eq!(events[0].payload.as_ref(), b"one");
        assert_eq!(events[1].payload.as_ref(), b"two");
    }

    #[test]
    fn test_append_inline_roundtrip() {
        let f = make_file("t4");
        let bytes = *b"abcdefgh";
        let seq = f.append_inline(&bytes).unwrap();
        assert_eq!(seq, 0);
        let events = f.read_range(0, 1);
        assert_eq!(events.len(), 1);
        assert!(events[0].entry.is_inline());
        assert_eq!(events[0].payload.as_ref(), &bytes);
    }

    #[test]
    fn test_append_bincode_roundtrip() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Foo {
            a: u32,
            b: String,
        }
        let f = make_file("t5");
        let v = Foo {
            a: 42,
            b: "hi".into(),
        };
        let seq = f.append_bincode(&v).unwrap();
        assert_eq!(seq, 0);
        let events = f.read_range(0, 1);
        let decoded: Foo = bincode::deserialize(&events[0].payload).unwrap();
        assert_eq!(decoded, v);
    }

    #[tokio::test]
    async fn test_tail_backfills_then_lives() {
        let f = make_file("t6");
        for i in 0..5u64 {
            f.append(format!("e{}", i).as_bytes()).unwrap();
        }

        let mut stream = Box::pin(f.tail(0));

        // First 5 events are backfill.
        for i in 0..5u64 {
            let ev = stream.next().await.unwrap().unwrap();
            assert_eq!(ev.entry.seq, i);
            assert_eq!(ev.payload.as_ref(), format!("e{}", i).as_bytes());
        }

        // New appends should be delivered live.
        f.append(b"live").unwrap();
        let ev = stream.next().await.unwrap().unwrap();
        assert_eq!(ev.entry.seq, 5);
        assert_eq!(ev.payload.as_ref(), b"live");
    }

    #[tokio::test]
    async fn test_tail_boundary_no_dupes_no_drops() {
        // Regression: backfill → register handoff must be gapless.
        // We append N events, open a tail, and in parallel the test
        // drives more appends. Every event must arrive exactly once.
        let f = make_file("t7");
        for i in 0..100u64 {
            f.append(format!("e{}", i).as_bytes()).unwrap();
        }

        let mut stream = Box::pin(f.tail(0));

        // Append 50 more after tail registration.
        let f2 = f.clone();
        let handle = tokio::spawn(async move {
            for i in 100..150u64 {
                f2.append(format!("e{}", i).as_bytes()).unwrap();
            }
        });

        let mut seen = Vec::new();
        for _ in 0..150 {
            let ev = stream.next().await.unwrap().unwrap();
            seen.push(ev.entry.seq);
        }
        handle.await.unwrap();

        assert_eq!(seen.len(), 150);
        for (i, &seq) in seen.iter().enumerate() {
            assert_eq!(seq, i as u64, "event {} arrived out of order or missing", i);
        }
    }

    #[tokio::test]
    async fn test_tail_from_mid_sequence() {
        let f = make_file("t8");
        for i in 0..10u64 {
            f.append(format!("e{}", i).as_bytes()).unwrap();
        }
        let mut stream = Box::pin(f.tail(7));
        for i in 7..10u64 {
            let ev = stream.next().await.unwrap().unwrap();
            assert_eq!(ev.entry.seq, i);
        }
    }

    #[test]
    fn test_retention_count() {
        let f = RedexFile::new(
            ChannelName::new("t9").unwrap(),
            RedexFileConfig::default().with_retention_max_events(3),
        );
        for i in 0..10u64 {
            f.append(format!("e{}", i).as_bytes()).unwrap();
        }
        f.sweep_retention();
        assert_eq!(f.len(), 3);
        // Surviving events are the newest 3.
        let events = f.read_range(0, 100);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].entry.seq, 7);
        assert_eq!(events[2].entry.seq, 9);
    }

    #[test]
    fn test_retention_respects_payload_slicing() {
        let f = RedexFile::new(
            ChannelName::new("t10").unwrap(),
            RedexFileConfig::default().with_retention_max_events(2),
        );
        f.append(b"first").unwrap();
        f.append(b"second").unwrap();
        f.append(b"third").unwrap();
        f.sweep_retention();
        let events = f.read_range(0, 100);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].payload.as_ref(), b"second");
        assert_eq!(events[1].payload.as_ref(), b"third");
    }

    #[test]
    fn test_close_rejects_further_append() {
        let f = make_file("t11");
        f.append(b"a").unwrap();
        f.close().unwrap();
        assert!(matches!(f.append(b"b"), Err(RedexError::Closed)));
    }

    #[tokio::test]
    async fn test_close_signals_outstanding_tails() {
        let f = make_file("t12");
        f.append(b"a").unwrap();
        let mut stream = Box::pin(f.tail(0));

        // First event is backfill.
        let ev = stream.next().await.unwrap().unwrap();
        assert_eq!(ev.entry.seq, 0);

        f.close().unwrap();

        // Next yield is the Closed error.
        let err = stream.next().await.unwrap().unwrap_err();
        assert!(matches!(err, RedexError::Closed));
    }

    // ---- Regression tests ----

    #[test]
    fn test_regression_offset_to_u32_boundary() {
        // Regression: `offset_to_u32` used to be a silent truncation
        // (`offset as u32`), which corrupted `RedexEntry::payload_offset`
        // on long-running persistent files whose base_offset crossed
        // `u32::MAX`. The fix converts the truncation into a
        // `SegmentOffsetOverflow` error at the exact boundary and
        // surfaces the overflowing offset value.
        assert!(offset_to_u32(0).is_ok());
        assert!(offset_to_u32(u32::MAX as u64).is_ok());
        let err = offset_to_u32(u32::MAX as u64 + 1).unwrap_err();
        assert!(matches!(
            err,
            RedexError::SegmentOffsetOverflow { offset } if offset == u32::MAX as u64 + 1
        ));
        assert!(matches!(
            offset_to_u32(u64::MAX).unwrap_err(),
            RedexError::SegmentOffsetOverflow { offset: u64::MAX }
        ));
    }

    #[test]
    fn test_regression_append_fails_when_base_offset_overflows_u32() {
        // Regression: single appends must surface the offset overflow
        // rather than write a truncated `payload_offset`. We force
        // `base_offset` past `u32::MAX` via the test-only hook and
        // verify the next append returns `SegmentOffsetOverflow`.
        let f = make_file("t_off_overflow");
        {
            let mut state = f.inner.state.lock();
            state.segment.force_base_offset(u32::MAX as u64 + 1);
        }
        // Any further append computes a start offset > u32::MAX and
        // must surface `SegmentOffsetOverflow` from `offset_to_u32`.
        let err = f.append(b"x").unwrap_err();
        assert!(matches!(err, RedexError::SegmentOffsetOverflow { .. }));
    }

    #[test]
    fn test_regression_batch_seq_gap_on_offset_overflow() {
        // Regression: `append_batch` used to `fetch_add(batch_size)`
        // before calling `segment.append`. If any append mid-batch
        // failed, the seq range was allocated but no index entries
        // were written — producing permanent gaps in seq space.
        //
        // The fix pre-validates capacity + final offset under the
        // state lock, advancing `next_seq` only when every append is
        // guaranteed to succeed. A failing batch must leave
        // `next_seq` unchanged.
        let f = make_file("t_batch_gap");
        // One real append so `next_seq` starts at 1.
        f.append(b"a").unwrap();
        assert_eq!(f.next_seq(), 1);

        // Push base_offset so a 2-payload batch of 8 bytes each
        // would overflow u32.
        {
            let mut state = f.inner.state.lock();
            state.segment.force_base_offset(u32::MAX as u64 - 4);
        }

        let err = f
            .append_batch(&[
                Bytes::from_static(b"aaaaaaaa"),
                Bytes::from_static(b"bbbbbbbb"),
            ])
            .unwrap_err();
        assert!(matches!(err, RedexError::SegmentOffsetOverflow { .. }));
        // Critical assertion: next_seq did NOT advance. A naive
        // pre-fix implementation would have `next_seq == 3` here.
        assert_eq!(
            f.next_seq(),
            1,
            "failing batch must not advance next_seq (would leak gap)"
        );
    }

    #[test]
    fn test_regression_ordered_batch_seq_gap_on_offset_overflow() {
        // Same contract as `test_regression_batch_seq_gap_on_offset_overflow`
        // but for `append_batch_ordered`.
        let f = make_file("t_obatch_gap");
        f.append(b"a").unwrap();
        assert_eq!(f.next_seq(), 1);

        {
            let mut state = f.inner.state.lock();
            state.segment.force_base_offset(u32::MAX as u64 - 4);
        }

        let err = f
            .append_batch_ordered(&[
                Bytes::from_static(b"aaaaaaaa"),
                Bytes::from_static(b"bbbbbbbb"),
            ])
            .unwrap_err();
        assert!(matches!(err, RedexError::SegmentOffsetOverflow { .. }));
        assert_eq!(f.next_seq(), 1);
    }

    // ---- Durability-first append regressions (persistent files) ----

    #[cfg(feature = "redex-disk")]
    fn tmp_persistent_dir(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "redex_persist_{}_{}_{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[cfg(feature = "redex-disk")]
    fn make_persistent(name: &str, dir: &std::path::Path) -> RedexFile {
        use super::super::manager::Redex;
        let r = Redex::new().with_persistent_dir(dir);
        r.open_file(
            &ChannelName::new(name).unwrap(),
            RedexFileConfig::default().with_persistent(true),
        )
        .unwrap()
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_regression_append_rolls_back_on_disk_failure() {
        // Regression: `append` used to commit in-memory state (segment
        // buf, index, timestamps) and advance `next_seq` BEFORE
        // attempting the disk mirror write. A disk failure left the
        // caller with an `Err` but memory diverged from disk — a
        // retry would duplicate the event and a reopen would miss it.
        // The fix writes to disk FIRST and rolls back the seq + leaves
        // memory untouched on disk failure.
        let dir = tmp_persistent_dir("append_rollback");
        let f = make_persistent("t_rollback/append", &dir);
        // One real append to prime the file.
        f.append(b"a").unwrap();
        assert_eq!(f.next_seq(), 1);
        assert_eq!(f.len(), 1);

        // Arm a one-shot failure on the next disk write.
        f.inner.disk.as_ref().unwrap().arm_next_append_failure();

        let err = f.append(b"b").unwrap_err();
        assert!(matches!(err, RedexError::Io(_)));

        // Invariants that MUST hold after a failed append:
        // - next_seq was rolled back (no seq burnt)
        // - in-memory index unchanged (no ghost entry)
        // - segment bytes unchanged (no orphaned payload)
        assert_eq!(
            f.next_seq(),
            1,
            "disk failure must roll back next_seq (no burnt seq)"
        );
        assert_eq!(f.len(), 1, "index must not grow on disk failure");
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_regression_append_batch_rolls_back_on_disk_failure() {
        // Same contract as above, for `append_batch`. A mid-batch
        // disk failure must roll back the whole seq range and leave
        // memory + index untouched.
        let dir = tmp_persistent_dir("batch_rollback");
        let f = make_persistent("t_rollback/batch", &dir);
        f.append(b"a").unwrap();
        assert_eq!(f.next_seq(), 1);

        f.inner.disk.as_ref().unwrap().arm_next_append_failure();

        let err = f
            .append_batch(&[
                Bytes::from_static(b"x"),
                Bytes::from_static(b"y"),
                Bytes::from_static(b"z"),
            ])
            .unwrap_err();
        assert!(matches!(err, RedexError::Io(_)));

        assert_eq!(
            f.next_seq(),
            1,
            "batch disk failure must roll back the full seq range"
        );
        assert_eq!(f.len(), 1, "index must not grow on batch disk failure");
    }
}
