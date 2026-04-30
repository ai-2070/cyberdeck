//! `TasksAdapter` — a typed wrapper around `CortexAdapter<TasksState>`
//! with domain-level ingest helpers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::super::super::channel::ChannelName;
use super::super::super::redex::{Redex, RedexError, RedexFileConfig};
use super::super::adapter::CortexAdapter;
use super::super::config::CortexAdapterConfig;
use super::super::envelope::EventEnvelope;
use super::super::error::CortexAdapterError;
use super::super::meta::{compute_checksum, EventMeta, EVENT_META_SIZE};
use super::dispatch::{
    DISPATCH_TASK_COMPLETED, DISPATCH_TASK_CREATED, DISPATCH_TASK_DELETED, DISPATCH_TASK_RENAMED,
    TASKS_CHANNEL,
};
use super::fold::TasksFold;
use super::state::TasksState;
use super::types::{
    TaskCompletedPayload, TaskCreatedPayload, TaskDeletedPayload, TaskId, TaskRenamedPayload,
};
use super::watch::TasksWatcher;

/// Return shape of [`TasksAdapter::snapshot_and_watch`]: the
/// initial filter result plus a boxed stream that emits every
/// subsequent change (dedup'd, with the initial skipped so the
/// caller doesn't double-render).
pub type TasksSnapshotAndWatch = (
    Vec<super::types::Task>,
    std::pin::Pin<Box<dyn futures::Stream<Item = Vec<super::types::Task>> + Send + 'static>>,
);

use futures::StreamExt;

/// Wire format for [`TasksAdapter::snapshot`]: wraps the `TasksState`
/// postcard blob produced by the underlying [`CortexAdapter`] alongside
/// the typed adapter's own `app_seq` counter so restore preserves
/// per-origin monotonicity of `EventMeta::seq_or_ts`.
#[derive(Serialize, Deserialize)]
struct TasksSnapshotPayload {
    /// Next-to-assign `app_seq` value at snapshot time — the adapter
    /// restores its counter to this so post-restore `EventMeta`
    /// records continue with monotonic per-origin sequencing.
    app_seq: u64,
    /// The `CortexAdapter::snapshot` blob (postcard of `TasksState`).
    inner: Vec<u8>,
}

/// Typed wrapper around `CortexAdapter<TasksState>` that exposes
/// domain-level operations (`create`, `rename`, `complete`, `delete`)
/// and hides the `EventMeta` + postcard plumbing.
pub struct TasksAdapter {
    inner: CortexAdapter<TasksState>,
    /// Producer identity stamped on every `EventMeta`.
    origin_hash: u32,
    /// Monotonic per-origin counter for `EventMeta::seq_or_ts`.
    /// Starts at 0 and increments on every ingest through this
    /// handle. This gives deterministic fold order for the stream of
    /// events produced by this TasksAdapter instance.
    app_seq: AtomicU64,
}

impl TasksAdapter {
    /// Open the tasks adapter against a `Redex` manager.
    ///
    /// Uses [`TASKS_CHANNEL`] (`"cortex/tasks"`). Replays the full
    /// history into state on open; subsequent events are appended to
    /// the same channel.
    pub fn open(redex: &Redex, origin_hash: u32) -> Result<Self, CortexAdapterError> {
        Self::open_with_config(redex, origin_hash, RedexFileConfig::default())
    }

    /// Like [`Self::open`] but with a caller-supplied `RedexFileConfig`
    /// (useful for `persistent: true` or custom retention).
    pub fn open_with_config(
        redex: &Redex,
        origin_hash: u32,
        redex_config: RedexFileConfig,
    ) -> Result<Self, CortexAdapterError> {
        let name = ChannelName::new(TASKS_CHANNEL).map_err(|e| {
            CortexAdapterError::Redex(super::super::super::redex::RedexError::Channel(
                e.to_string(),
            ))
        })?;
        let inner = CortexAdapter::open(
            redex,
            &name,
            redex_config,
            CortexAdapterConfig::default(),
            TasksFold,
            TasksState::new(),
        )?;
        Ok(Self {
            inner,
            origin_hash,
            app_seq: AtomicU64::new(0),
        })
    }

    /// Create a new task. Returns the RedEX seq of the append.
    pub fn create(
        &self,
        id: TaskId,
        title: impl Into<String>,
        now_ns: u64,
    ) -> Result<u64, CortexAdapterError> {
        let payload = TaskCreatedPayload {
            id,
            title: title.into(),
            now_ns,
        };
        self.ingest_typed(DISPATCH_TASK_CREATED, &payload)
    }

    /// Rename an existing task. No-op at fold time if `id` is unknown.
    pub fn rename(
        &self,
        id: TaskId,
        new_title: impl Into<String>,
        now_ns: u64,
    ) -> Result<u64, CortexAdapterError> {
        let payload = TaskRenamedPayload {
            id,
            new_title: new_title.into(),
            now_ns,
        };
        self.ingest_typed(DISPATCH_TASK_RENAMED, &payload)
    }

    /// Mark a task completed. No-op at fold time if `id` is unknown.
    pub fn complete(&self, id: TaskId, now_ns: u64) -> Result<u64, CortexAdapterError> {
        let payload = TaskCompletedPayload { id, now_ns };
        self.ingest_typed(DISPATCH_TASK_COMPLETED, &payload)
    }

    /// Delete a task. No-op at fold time if `id` is unknown.
    pub fn delete(&self, id: TaskId) -> Result<u64, CortexAdapterError> {
        let payload = TaskDeletedPayload { id };
        self.ingest_typed(DISPATCH_TASK_DELETED, &payload)
    }

    /// Read-only access to the materialized state.
    pub fn state(&self) -> Arc<RwLock<TasksState>> {
        self.inner.state()
    }

    /// Total task count in the current state. Cheap; acquires the
    /// state read lock briefly. Matches the Node/Python SDK surface.
    pub fn count(&self) -> usize {
        self.inner.state().read().len()
    }

    /// Block until every event up through `seq` has been folded.
    pub async fn wait_for_seq(&self, seq: u64) {
        self.inner.wait_for_seq(seq).await;
    }

    /// Close the adapter. See [`CortexAdapter::close`].
    pub fn close(&self) -> Result<(), CortexAdapterError> {
        self.inner.close()
    }

    /// True if the fold task is currently running.
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Access the wrapped [`CortexAdapter`] for cases that need the
    /// lower-level surface.
    pub fn as_cortex(&self) -> &CortexAdapter<TasksState> {
        &self.inner
    }

    /// Start building a reactive watcher. See
    /// [`TasksWatcher::stream`] for emission semantics (initial +
    /// deduplicated on filter-result change).
    pub fn watch(&self) -> TasksWatcher {
        TasksWatcher::new(self.inner.state(), self.inner.changes().boxed())
    }

    /// One-shot combo: a snapshot of the current filter result PLUS
    /// a stream that emits every **subsequent** change to that
    /// filter. The stream skips the initial emission so the caller
    /// doesn't see the snapshot twice — the snapshot is the initial
    /// state; the stream carries deltas from there forward.
    ///
    /// Useful for UI-style consumers: "paint what's there now, then
    /// react to changes" without a manual dedup against the first
    /// emission.
    pub fn snapshot_and_watch(&self, watcher: TasksWatcher) -> TasksSnapshotAndWatch {
        use futures::StreamExt;
        // Compute the snapshot from the adapter's current state,
        // reusing the watcher's configured filter. Holding the read
        // lock only for the execute call keeps it brief.
        let initial = {
            let state = self.inner.state();
            let guard = state.read();
            watcher.spec_for_snapshot().execute(&guard)
        };
        // BUG #143: pre-fix this used
        // `skip_while(|c| c == &initial)`, which is "sticky" — once
        // the predicate evaluates `false` it never re-skips. That
        // handled the snapshot-vs-watcher race (state changes
        // between snapshot read and `watcher.stream()` start, so
        // the watcher's first emission ≠ snapshot — we want to
        // forward it) but introduced a starvation hazard: under an
        // (A → B → A) state oscillation that the single-slot
        // `tokio::sync::watch` collapses into final A, the
        // surviving A equals `initial` so it's skipped — the
        // consumer is silent until state diverges from A.
        //
        // The fix: skip ONLY the first emission, and only if it
        // equals the snapshot. Subsequent emissions always
        // forward. This handles both cases:
        //   - leading match (no state change since snapshot): skip
        //     the first emission → consumer sees no duplicate
        //   - leading divergence (state changed during the race):
        //     first emission ≠ snapshot → forwarded
        //   - oscillation back to initial (A → B → A): the watch's
        //     surviving A is forwarded as the first item if state
        //     hadn't changed since snapshot — caller can dedup
        //     against their snapshot if they care, or treat it as
        //     "fold tick observed" signal.
        // Implemented via `enumerate().filter(...)` rather than a
        // separate state-carrying skip primitive, since
        // `futures::StreamExt::filter` doesn't accept a `FnMut`.
        let initial_for_stream = initial.clone();
        let stream = watcher
            .stream()
            .enumerate()
            .filter(move |(i, current)| {
                let drop_first = *i == 0 && current == &initial_for_stream;
                futures::future::ready(!drop_first)
            })
            .map(|(_, current)| current)
            .boxed();
        (initial, stream)
    }

    /// Capture a snapshot suitable for restore. Returns
    /// `(state_bytes, last_seq)` — persist both together.
    pub fn snapshot(&self) -> Result<(Vec<u8>, Option<u64>), CortexAdapterError> {
        let (inner, last_seq) = self.inner.snapshot()?;
        let payload = TasksSnapshotPayload {
            app_seq: self.app_seq.load(Ordering::Acquire),
            inner,
        };
        let bytes = postcard::to_allocvec(&payload).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("tasks snapshot wrap: {}", e)))
        })?;
        Ok((bytes, last_seq))
    }

    /// Open the tasks adapter from a snapshot, skipping replay of
    /// events up through `last_seq`.
    pub fn open_from_snapshot(
        redex: &Redex,
        origin_hash: u32,
        state_bytes: &[u8],
        last_seq: Option<u64>,
    ) -> Result<Self, CortexAdapterError> {
        Self::open_from_snapshot_with_config(
            redex,
            origin_hash,
            RedexFileConfig::default(),
            state_bytes,
            last_seq,
        )
    }

    /// Like [`Self::open_from_snapshot`] but with a caller-supplied
    /// `RedexFileConfig` (e.g. for `persistent: true`).
    pub fn open_from_snapshot_with_config(
        redex: &Redex,
        origin_hash: u32,
        redex_config: RedexFileConfig,
        state_bytes: &[u8],
        last_seq: Option<u64>,
    ) -> Result<Self, CortexAdapterError> {
        let payload: TasksSnapshotPayload = postcard::from_bytes(state_bytes).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("tasks snapshot unwrap: {}", e)))
        })?;
        let name = ChannelName::new(TASKS_CHANNEL)
            .map_err(|e| CortexAdapterError::Redex(RedexError::Channel(e.to_string())))?;
        let inner = CortexAdapter::open_from_snapshot(
            redex,
            &name,
            redex_config,
            CortexAdapterConfig::default(),
            TasksFold,
            &payload.inner,
            last_seq,
        )?;

        // Restore the app_seq counter so post-restore events continue
        // per-origin monotonic sequencing. If the file has events for
        // this origin with seq > last_seq (periodic-snapshot-while-
        // ingesting pattern), the fold task will replay them, but
        // THEIR seq_or_ts values have already been assigned — we
        // must bump the counter past the highest one of our origin
        // to avoid duplicates on the next ingest.
        let mut app_seq = payload.app_seq;
        let replay_start = last_seq.map(|s| s + 1).unwrap_or(0);
        let file = redex.open_file(&name, redex_config)?;
        let replay_end = file.next_seq();
        if replay_start < replay_end {
            for ev in file.read_range(replay_start, replay_end) {
                if ev.payload.len() < EVENT_META_SIZE {
                    continue;
                }
                if let Some(meta) = EventMeta::from_bytes(&ev.payload[..EVENT_META_SIZE]) {
                    if meta.origin_hash == origin_hash && meta.seq_or_ts >= app_seq {
                        app_seq = meta.seq_or_ts + 1;
                    }
                }
            }
        }

        Ok(Self {
            inner,
            origin_hash,
            app_seq: AtomicU64::new(app_seq),
        })
    }

    /// Build the `EventEnvelope` + ingest. Keeps postcard serialization
    /// and `EventMeta` assembly in one place.
    ///
    /// BUG #126: pre-fix this called `app_seq.fetch_add(1, ...)`
    /// FIRST, then `inner.ingest`. If `inner.ingest` failed (closed
    /// adapter, RedEX append error, fold error under `Stop` policy),
    /// the local counter was permanently advanced past a `seq_or_ts`
    /// that never made it to the log. The next snapshot persisted
    /// the higher counter; on restore, future ingests picked up at
    /// the inflated value, leaving a permanent gap (and producing
    /// `seq_or_ts` collisions when a second adapter sharing the
    /// same `origin_hash` recovered via on-disk scan).
    ///
    /// Now: load the current counter, build the envelope at that
    /// value, attempt the ingest, and only if it succeeds do we
    /// CAS-commit the counter advance. On CAS contention (a
    /// concurrent ingest moved the counter past us), we retry with
    /// the new value — the inner ingest IS retried because each
    /// attempt produces a fresh `EventEnvelope` with the right
    /// `seq_or_ts`. The Redex log is the source of truth; counter
    /// drift never escapes this function.
    fn ingest_typed<T: serde::Serialize>(
        &self,
        dispatch: u8,
        payload: &T,
    ) -> Result<u64, CortexAdapterError> {
        let tail = postcard::to_allocvec(payload).map_err(|e| {
            CortexAdapterError::Redex(super::super::super::redex::RedexError::Encode(
                e.to_string(),
            ))
        })?;
        let checksum = compute_checksum(&tail);
        let payload_bytes = Bytes::from(tail);

        loop {
            let app_seq = self.app_seq.load(Ordering::Acquire);
            let meta = EventMeta::new(dispatch, 0, self.origin_hash, app_seq, checksum);
            let env = EventEnvelope::new(meta, payload_bytes.clone());
            let seq = self.inner.ingest(env)?;

            // Commit the counter advance now that the log has
            // accepted the entry. CAS to defend against a concurrent
            // ingest that moved the counter past `app_seq` between
            // our load and the inner ingest — in that case we'd be
            // stamping the SAME `seq_or_ts` as another envelope,
            // producing a dup. The inner.ingest above already
            // succeeded, so a CAS failure means our envelope landed
            // at a stale `seq_or_ts`; we surface this as an Encode
            // error so the caller knows the ingest is not
            // counter-monotonic.
            //
            // In practice every external caller goes through a
            // single shared `&TasksAdapter` instance and this
            // function is the only counter writer, so contention is
            // dominated by single-thread sequential ingest where the
            // CAS always succeeds on the first try. Multi-threaded
            // ingest produces ordering uncertainty regardless of
            // counter primitive — the user-visible app_seq just
            // tracks insertion order at the call site.
            match self.app_seq.compare_exchange(
                app_seq,
                app_seq + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(seq),
                Err(_actual) => {
                    // A concurrent ingest stamped the same
                    // `seq_or_ts`. The log already has our event,
                    // so we can't roll back; surface a recoverable
                    // error. The caller's options are: rebuild the
                    // adapter from snapshot (scans the log to
                    // reconcile app_seq), or accept the duplicate.
                    return Err(CortexAdapterError::Redex(
                        super::super::super::redex::RedexError::Encode(format!(
                            "concurrent ingest_typed produced duplicate app_seq={}; \
                             rebuild adapter from snapshot to reconcile",
                            app_seq
                        )),
                    ));
                }
            }
        }
    }
}

impl std::fmt::Debug for TasksAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TasksAdapter")
            .field("origin_hash", &self.origin_hash)
            .field("app_seq", &self.app_seq.load(Ordering::Acquire))
            .field("inner", &self.inner)
            .finish()
    }
}
