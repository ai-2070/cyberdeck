//! `RedexFile` — the append / tail / read_range primitive.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use futures::Stream;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::super::channel::ChannelName;
use super::config::RedexFileConfig;
use super::entry::{payload_checksum, RedexEntry, INLINE_PAYLOAD_SIZE};
use super::error::RedexError;
use super::event::RedexEvent;
use super::retention::compute_eviction_count;
use super::segment::{HeapSegment, MAX_SEGMENT_BYTES};

#[cfg(feature = "redex-disk")]
use super::config::FsyncPolicy;
#[cfg(feature = "redex-disk")]
use super::disk::DiskSegment;
#[cfg(feature = "redex-disk")]
use std::path::Path;
#[cfg(feature = "redex-disk")]
use tokio::sync::Notify;

/// A live tail subscription waiting for new events.
struct TailWatcher {
    /// Minimum seq to deliver (inclusive).
    from_seq: u64,
    /// Channel back to the subscriber. Bounded — see [`TAIL_BUFFER_SIZE`].
    sender: mpsc::Sender<Result<RedexEvent, RedexError>>,
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
    /// Shutdown signal for the `FsyncPolicy::Interval` background
    /// task. `Some` iff an Interval task was spawned at
    /// `open_persistent` time. `close()` calls `notify_one()` so a
    /// permit is stored even if the task hasn't yet registered a
    /// waiter; the task observes it, exits, and releases its
    /// DiskSegment reference.
    #[cfg(feature = "redex-disk")]
    interval_shutdown: Option<Arc<Notify>>,
}

/// Dropping the last `RedexFile` clone without calling `close()`
/// previously leaked the `FsyncPolicy::Interval` background task —
/// it kept a strong `Arc<DiskSegment>` and a shutdown `Notify` whose
/// only firing site was inside `close()`. `redex/index.rs` already
/// had a Drop impl that mirrored this pattern; we mirror it here so
/// a misbehaving caller (or a panic path that bypasses the explicit
/// close) doesn't leak the task for the lifetime of the runtime.
///
/// Drop is best-effort: it fires the notify so the spawned task
/// observes the signal and exits at the next select. We do NOT
/// flush or fsync from Drop because that would require an async
/// runtime context that may not be available.
impl Drop for RedexFileInner {
    fn drop(&mut self) {
        #[cfg(feature = "redex-disk")]
        if let Some(notify) = self.interval_shutdown.as_ref() {
            // `notify_one` stores a permit even if no waiter is
            // currently parked, so a task that hasn't yet reached
            // its first `notified().await` will still observe the
            // signal on its next select.
            notify.notify_one();
        }
    }
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
                #[cfg(feature = "redex-disk")]
                interval_shutdown: None,
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
        // Derive the DiskSegment's append-side fsync cadence from
        // the caller's policy. EveryN is honored on the append path;
        // Never and Interval both pass 0 (Interval runs via the
        // per-file background task spawned below).
        let fsync_every_n = match config.fsync_policy {
            FsyncPolicy::Never | FsyncPolicy::Interval(_) => 0,
            FsyncPolicy::EveryN(n) => n.max(1),
        };
        let recovered = DiskSegment::open(base_dir, &name, fsync_every_n)?;
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

        let disk = Arc::new(recovered.disk);

        // Spawn the Interval background task (if requested). It holds
        // an Arc<DiskSegment> and a clone of the shutdown Notify; on
        // close() the notify fires and the task exits. Dropping the
        // last RedexFile clone WITHOUT calling close() leaks the
        // task until the runtime shuts down — consistent with the
        // rest of the codebase's lifecycle expectations (callers are
        // expected to `close()` persistent files).
        let interval_shutdown = match config.fsync_policy {
            FsyncPolicy::Interval(d) if d > std::time::Duration::ZERO => {
                let shutdown = Arc::new(Notify::new());
                let task_shutdown = shutdown.clone();
                let task_disk = disk.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = task_shutdown.notified() => return,
                            _ = tokio::time::sleep(d) => {
                                if let Err(e) = task_disk.sync() {
                                    tracing::warn!(
                                        error = %e,
                                        "Interval fsync failed; continuing"
                                    );
                                }
                            }
                        }
                    }
                });
                Some(shutdown)
            }
            _ => None,
        };

        Ok(Self {
            inner: Arc::new(RedexFileInner {
                name,
                config,
                next_seq: AtomicU64::new(next_seq),
                state: Mutex::new(state),
                closed: AtomicBool::new(false),
                disk: Some(disk),
                interval_shutdown,
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

        // The `Bytes::copy_from_slice` below is purely for watcher
        // delivery; durable storage already landed via `segment.append`.
        // Skip the copy entirely when nobody is tailing.
        if !state.watchers.is_empty() {
            let event = RedexEvent {
                entry,
                payload: Bytes::copy_from_slice(payload),
            };
            notify_watchers(&mut state.watchers, &event);
        }

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

        if !state.watchers.is_empty() {
            let event = RedexEvent {
                entry,
                payload: Bytes::copy_from_slice(payload),
            };
            notify_watchers(&mut state.watchers, &event);
        }

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

    /// Strictly-ordered variant of [`Self::append`].
    ///
    /// Both [`Self::append`] and this method now take the state lock
    /// before allocating a sequence number (the failure-atomicity
    /// fix required moving `fetch_add` inside the lock so rollback
    /// on disk-write failure is safe). That means `append` already
    /// produces in-seq-order index insertions under contention, and
    /// the two paths are functionally equivalent for single writes.
    ///
    /// The real distinction is at the wrapper level: this method
    /// pairs with [`Self::append_batch_ordered`], which holds ONE
    /// lock across an entire batch, whereas [`Self::append_batch`]
    /// also holds one lock per batch today. In v1 the non-ordered
    /// and ordered paths are nearly identical. The distinction is
    /// kept so that a future optimization of [`Self::append`] (e.g.
    /// moving the seq allocation back outside the lock with a
    /// different rollback scheme) doesn't affect callers who need
    /// guaranteed-ordered appends.
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

        if !state.watchers.is_empty() {
            let event = RedexEvent {
                entry,
                payload: Bytes::copy_from_slice(payload),
            };
            notify_watchers(&mut state.watchers, &event);
        }

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

        if !state.watchers.is_empty() {
            let event = RedexEvent {
                entry,
                payload: Bytes::copy_from_slice(payload),
            };
            notify_watchers(&mut state.watchers, &event);
        }

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

    /// Append `value` (postcard-serialized) AND run `fold_fn` against
    /// caller-supplied `state` in the same call. Returns the
    /// assigned seq.
    ///
    /// This is the common "log-an-event and update a materialized
    /// view in one step" pattern, without spinning up a full CortEX
    /// adapter. Callers maintain `state` themselves; the fold
    /// closure sees the just-appended value and mutates state in
    /// place.
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
        let bytes = postcard::to_allocvec(value)
            .map_err(|e| RedexError::Encode(format!("append_and_fold serialize: {}", e)))?;
        let seq = self.append(&bytes)?;
        fold_fn(value, state);
        Ok(seq)
    }

    /// Convenience: serialize `value` with postcard and append.
    pub fn append_postcard<T: serde::Serialize>(&self, value: &T) -> Result<u64, RedexError> {
        let bytes = postcard::to_allocvec(value).map_err(|e| RedexError::Encode(e.to_string()))?;
        self.append(&bytes)
    }

    /// Subscribe to all events with `seq >= from_seq`, including those
    /// already in the index at call time.
    ///
    /// Backfill and live registration happen atomically under the
    /// state lock: no event can interleave between backfill delivery
    /// and live subscription.
    ///
    /// Delivery is backed by a per-subscription bounded channel of
    /// depth [`RedexFileConfig::tail_buffer_size`].
    ///
    /// - **Backfill overflow** (requested `from_seq` produces more
    ///   retained events than the buffer can hold): pre-flighted
    ///   under the state lock; the subscriber observes
    ///   [`RedexError::Lagged`] as the first stream item and no
    ///   truncated history. Guaranteed deliverable because the
    ///   channel is empty at that point.
    /// - **Live overflow** (subscriber falls behind during live
    ///   delivery): disconnected with a best-effort
    ///   [`RedexError::Lagged`]. Under sustained saturation the
    ///   signal itself may be dropped (the channel is full when we
    ///   try to enqueue it), in which case the subscriber sees a
    ///   clean stream end.
    pub fn tail(
        &self,
        from_seq: u64,
    ) -> impl Stream<Item = Result<RedexEvent, RedexError>> + Send + 'static {
        // mpsc::channel panics on 0; clamp to a minimum of 1.
        let buffer = self.inner.config.tail_buffer_size.max(1);
        let (tx, rx) = mpsc::channel(buffer);

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
            let _ = tx.try_send(Err(RedexError::Closed));
            return ReceiverStream::new(rx);
        }

        // Backfill pre-flight. The index is in seq order so we can
        // binary-search for the first matching entry and compute the
        // backfill size in O(log n). If it can't fit in the channel,
        // we signal `Lagged` *before* enqueuing any events — the
        // channel is empty at this point so the signal is guaranteed
        // to land, and the subscriber sees a clean "you missed
        // history" error rather than a silently-truncated prefix
        // (the case the prior best-effort `try_send(Err(Lagged))`
        // could not deliver if the buffer was already saturated).
        let start = state.index.partition_point(|e| e.seq < from_seq);
        let backfill_count = state.index.len() - start;
        if backfill_count > buffer {
            drop(state);
            let _ = tx.try_send(Err(RedexError::Lagged));
            return ReceiverStream::new(rx);
        }

        // Backfill. Uses `try_send` under the state lock so backfill +
        // registration is still atomic with respect to concurrent
        // appends. The pre-flight above guarantees we won't hit the
        // `Full` path here — defensively handle it anyway in case a
        // payload evicts between the count and the materialize.
        for entry in state.index[start..].iter() {
            let event = match materialize(entry, &state.segment) {
                Some(e) => e,
                None => continue, // payload evicted between index retain and read
            };
            match tx.try_send(Ok(event)) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    // Defensive: pre-flight should have prevented this.
                    let _ = tx.try_send(Err(RedexError::Lagged));
                    return ReceiverStream::new(rx);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // Receiver dropped before stream was polled.
                    return ReceiverStream::new(rx);
                }
            }
        }

        // Register for live events.
        state.watchers.push(TailWatcher {
            from_seq,
            sender: tx,
        });
        drop(state);

        ReceiverStream::new(rx)
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
    /// For persistent files, fsyncs the disk segment before returning
    /// and signals any `FsyncPolicy::Interval` background task to
    /// exit. `close()` always fsyncs regardless of the per-file
    /// `FsyncPolicy` — this is the caller's explicit durability
    /// barrier.
    pub fn close(&self) -> Result<(), RedexError> {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let mut state = self.inner.state.lock();
        for w in state.watchers.drain(..) {
            // Best-effort `Closed` signal. If the per-subscriber
            // buffer is saturated the signal is dropped, but the
            // sender is dropped at end-of-iteration regardless, so
            // the subscriber still observes a clean stream end.
            let _ = w.sender.try_send(Err(RedexError::Closed));
        }
        drop(state);

        #[cfg(feature = "redex-disk")]
        {
            // Signal the Interval task to exit before fsyncing so
            // `close()` isn't racing the task's own sync.
            //
            // `notify_one` stores a permit if the task hasn't yet
            // parked on `notified()` — e.g. a `close()` that races a
            // just-spawned task before it reaches the select, or one
            // that fires while the task is between sleep and the
            // next poll. `notify_waiters` would be lost in that
            // window and the fsync loop would keep running after
            // close.
            if let Some(shutdown) = self.inner.interval_shutdown.as_ref() {
                shutdown.notify_one();
            }
            if let Some(disk) = self.disk() {
                disk.sync()?;
            }
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

    /// Test-only: cumulative successful `sync()` count on the disk
    /// segment. `None` for heap-only files. Used by `FsyncPolicy`
    /// tests to assert cadence without racing real I/O.
    #[cfg(all(test, feature = "redex-disk"))]
    pub fn sync_count(&self) -> Option<u64> {
        self.disk().map(|d| d.sync_count())
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

/// Verify the entry's stored checksum against the actual payload
/// bytes on every read. The 28-bit xxh3 is computed at append
/// time; without verification at read time, on-disk corruption
/// (torn writes, bit-rot, external tampering) would flow through
/// `materialize` as a valid event and silently poison every
/// downstream consumer. We surface a checksum mismatch by
/// returning `None` (same channel `materialize` already uses for
/// "couldn't construct an event" signals).
fn materialize(entry: &RedexEntry, segment: &HeapSegment) -> Option<RedexEvent> {
    let payload = if entry.is_inline() {
        Bytes::copy_from_slice(&entry.inline_payload()?)
    } else {
        segment.read(entry.payload_offset as u64, entry.payload_len)?
    };

    let stored = entry.checksum();
    let computed = super::entry::payload_checksum(&payload);
    if stored != computed {
        tracing::error!(
            seq = entry.seq,
            stored_checksum = format_args!("{:#x}", stored),
            computed_checksum = format_args!("{:#x}", computed),
            "RedexFile::materialize: checksum mismatch — payload corrupt; dropping entry"
        );
        return None;
    }

    Some(RedexEvent {
        entry: *entry,
        payload,
    })
}

fn notify_watchers(watchers: &mut Vec<TailWatcher>, event: &RedexEvent) {
    // Walk watchers; drop those whose receiver is gone or whose
    // buffer is saturated. On `Full` we make a best-effort attempt
    // to signal `Lagged` before dropping — under true saturation
    // that signal may itself fail, in which case the subscriber
    // sees a plain stream end.
    watchers.retain(|w| {
        if event.entry.seq < w.from_seq {
            return true; // keep, but don't deliver
        }
        match w.sender.try_send(Ok(event.clone())) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                let _ = w.sender.try_send(Err(RedexError::Lagged));
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => false,
        }
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
    fn test_append_postcard_roundtrip() {
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
        let seq = f.append_postcard(&v).unwrap();
        assert_eq!(seq, 0);
        let events = f.read_range(0, 1);
        let decoded: Foo = postcard::from_bytes(&events[0].payload).unwrap();
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

    // ---- FsyncPolicy tests (Stage 1 of v2 closeout) ----

    #[cfg(feature = "redex-disk")]
    fn make_persistent_with_policy(
        name: &str,
        base: &std::path::Path,
        policy: super::FsyncPolicy,
    ) -> RedexFile {
        let r = super::super::manager::Redex::new().with_persistent_dir(base);
        r.open_file(
            &ChannelName::new(name).unwrap(),
            RedexFileConfig::default()
                .with_persistent(true)
                .with_fsync_policy(policy),
        )
        .unwrap()
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_fsync_policy_never_skips_append_syncs() {
        // Never: no append-path fsync at all. Counter stays at 0 until
        // an explicit `sync()` or `close()` triggers one.
        let dir = tmp_persistent_dir("fsync_never");
        let f = make_persistent_with_policy("fsync/never", &dir, super::FsyncPolicy::Never);
        for i in 0..20u64 {
            f.append(format!("n-{}", i).as_bytes()).unwrap();
        }
        assert_eq!(
            f.sync_count(),
            Some(0),
            "Never must not fsync on the append path"
        );
        f.sync().unwrap();
        assert_eq!(f.sync_count(), Some(1), "explicit sync() still works");
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_fsync_policy_every_n_syncs_on_cadence() {
        // EveryN(5): exactly one sync per 5 appends. 23 appends → 4
        // append-driven syncs (at append-count 5, 10, 15, 20); the
        // trailing 3 don't trigger. close() adds one more.
        let dir = tmp_persistent_dir("fsync_every_n");
        let f = make_persistent_with_policy("fsync/everyn", &dir, super::FsyncPolicy::EveryN(5));
        for i in 0..23u64 {
            f.append(format!("e-{}", i).as_bytes()).unwrap();
        }
        assert_eq!(
            f.sync_count(),
            Some(4),
            "EveryN(5) over 23 appends = 4 append-driven syncs"
        );
        f.close().unwrap();
        assert_eq!(f.sync_count(), Some(5), "close() adds one more sync");
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_fsync_policy_every_n_clamps_zero_to_one() {
        // EveryN(0) would never fire with a naïve implementation; the
        // clamp at `open_persistent` maps 0 (and 1) to "fsync every
        // append."
        let dir = tmp_persistent_dir("fsync_every_n_zero");
        let f =
            make_persistent_with_policy("fsync/everyn_zero", &dir, super::FsyncPolicy::EveryN(0));
        for i in 0..3u64 {
            f.append(format!("e-{}", i).as_bytes()).unwrap();
        }
        assert_eq!(
            f.sync_count(),
            Some(3),
            "EveryN(0) must fsync on every append (clamped to 1)"
        );
    }

    #[cfg(feature = "redex-disk")]
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn test_fsync_policy_interval_fires_on_timer() {
        // Interval drives fsync from a tokio background task. We use
        // paused-time so the test is deterministic — advance time
        // through three intervals, expect three syncs (give or take
        // one scheduler hop).
        let dir = tmp_persistent_dir("fsync_interval");
        let f = make_persistent_with_policy(
            "fsync/interval",
            &dir,
            super::FsyncPolicy::Interval(std::time::Duration::from_millis(50)),
        );
        // No appends yet, but the timer ticks anyway.
        for _ in 0..3 {
            tokio::time::advance(std::time::Duration::from_millis(50)).await;
            tokio::task::yield_now().await;
        }
        // The background task may be slightly ahead or behind the
        // advance cursor; require at least 2 to avoid flakes on a
        // busy runner.
        let observed = f.sync_count().unwrap_or(0);
        assert!(
            observed >= 2,
            "Interval(50ms) after 150ms of advance expected ≥ 2 syncs, got {}",
            observed,
        );
        f.close().unwrap();
    }

    #[cfg(feature = "redex-disk")]
    #[test]
    fn test_regression_close_still_syncs_under_never_policy() {
        // Regression guard: `Never` means "no fsync on append," NOT
        // "no durability at all." `close()` is the caller's explicit
        // durability barrier and must always sync regardless of
        // policy; otherwise a clean shutdown of a Never-configured
        // file would silently lose its tail.
        let dir = tmp_persistent_dir("fsync_close_syncs");
        let base = dir.clone();

        {
            let f =
                make_persistent_with_policy("fsync/close_syncs", &base, super::FsyncPolicy::Never);
            for i in 0..10u64 {
                f.append(format!("x-{}", i).as_bytes()).unwrap();
            }
            assert_eq!(f.sync_count(), Some(0), "Never skips append syncs");
            f.close().unwrap();
            assert_eq!(
                f.sync_count(),
                Some(1),
                "close() under Never still syncs once"
            );
        }

        // Reopen and verify every entry survived the close.
        let r = super::super::manager::Redex::new().with_persistent_dir(&base);
        let f2 = r
            .open_file(
                &ChannelName::new("fsync/close_syncs").unwrap(),
                RedexFileConfig::default().with_persistent(true),
            )
            .unwrap();
        assert_eq!(f2.len(), 10, "all 10 entries must persist across close");
    }

    /// Regression: previously, the 28-bit xxh3 stored on every
    /// `RedexEntry` was computed at append but never verified at
    /// read. On-disk corruption (torn writes, bit-rot, external
    /// tampering) flowed through `materialize` as a valid event.
    /// The fix verifies the stored checksum matches the recomputed
    /// payload checksum on every read; mismatched entries are
    /// dropped from the result with an error log.
    ///
    /// We simulate corruption by mutating the in-memory segment
    /// bytes after append and asserting `read_range` no longer
    /// returns the corrupt entry.
    #[test]
    fn read_path_drops_entries_with_bad_checksum() {
        let f = make_file("checksum_verify");

        // Append three heap-stored entries (payload > inline size of
        // 8 bytes, so they live in the segment).
        f.append(b"first-payload-bytes").unwrap();
        f.append(b"second-payload-bytes").unwrap();
        f.append(b"third-payload-bytes").unwrap();

        // Sanity: all three round-trip cleanly.
        let events = f.read_range(0, 100);
        assert_eq!(events.len(), 3);

        // Corrupt the second entry's bytes in the heap segment.
        // We mutate the byte at the entry's offset directly via the
        // shared state lock — same access pattern `materialize` will
        // use, just with a write.
        {
            let mut state = f.inner.state.lock();
            // Find the second entry (seq == 1) and flip a byte at
            // its payload_offset.
            let entry = state.index.iter().find(|e| e.seq == 1).copied().unwrap();
            assert!(!entry.is_inline(), "test premise: seq=1 must be heap-stored");
            // Flip the first byte of the payload.
            let off = entry.payload_offset as usize;
            let old = state.segment.bytes_for_test_mut()[off];
            state.segment.bytes_for_test_mut()[off] = old.wrapping_add(1);
        }

        // The corrupted entry must be dropped on read; the other
        // two survive.
        let events = f.read_range(0, 100);
        assert_eq!(
            events.len(),
            2,
            "corrupt entry must be dropped from read_range result"
        );
        let surviving_seqs: Vec<u64> = events.iter().map(|e| e.entry.seq).collect();
        assert_eq!(surviving_seqs, vec![0, 2]);
    }
}
