//! `MemoriesAdapter` — typed wrapper over `CortexAdapter<MemoriesState>`
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
use super::super::meta::{compute_checksum, EventMeta};
use super::super::watermark::WatermarkingFold;
use super::dispatch::{
    DISPATCH_MEMORY_DELETED, DISPATCH_MEMORY_PINNED, DISPATCH_MEMORY_RETAGGED,
    DISPATCH_MEMORY_STORED, DISPATCH_MEMORY_UNPINNED, MEMORIES_CHANNEL,
};
use super::fold::MemoriesFold;
use super::state::MemoriesState;
use super::types::{
    MemoryDeletedPayload, MemoryId, MemoryPinTogglePayload, MemoryRetaggedPayload,
    MemoryStoredPayload,
};
use super::watch::MemoriesWatcher;

/// Return shape of [`MemoriesAdapter::snapshot_and_watch`]: the
/// initial filter result plus a boxed stream that emits every
/// subsequent change (dedup'd, with the initial skipped so the
/// caller doesn't double-render).
pub type MemoriesSnapshotAndWatch = (
    Vec<super::types::Memory>,
    std::pin::Pin<Box<dyn futures::Stream<Item = Vec<super::types::Memory>> + Send + 'static>>,
);

use futures::StreamExt;

/// Wire format for [`MemoriesAdapter::snapshot`]: wraps the
/// `MemoriesState` postcard blob produced by the underlying
/// [`CortexAdapter`] alongside the typed adapter's own `app_seq`
/// counter so restore preserves per-origin monotonicity of
/// `EventMeta::seq_or_ts`.
#[derive(Serialize, Deserialize)]
struct MemoriesSnapshotPayload {
    /// Next-to-assign `app_seq` value at snapshot time — the adapter
    /// restores its counter to this so post-restore `EventMeta`
    /// records continue with monotonic per-origin sequencing.
    app_seq: u64,
    /// The `CortexAdapter::snapshot` blob (postcard of `MemoriesState`).
    inner: Vec<u8>,
}

/// Typed wrapper around `CortexAdapter<MemoriesState>` that exposes
/// domain-level operations (`store`, `retag`, `pin`, `unpin`,
/// `delete`) and hides the `EventMeta` + postcard plumbing.
pub struct MemoriesAdapter {
    inner: CortexAdapter<MemoriesState>,
    /// Producer identity stamped on every `EventMeta`.
    origin_hash: u32,
    /// Monotonic per-origin counter for `EventMeta::seq_or_ts`.
    /// Shared with the inner `WatermarkingFold` wrapper around
    /// [`MemoriesFold`] (BUG #148): the fold task advances this
    /// counter via `fetch_max(seq_or_ts + 1)` for every replayed
    /// event whose `origin_hash` matches ours, so `ingest_typed`
    /// after open can never collide with an already-stored event.
    app_seq: Arc<AtomicU64>,
}

impl MemoriesAdapter {
    /// Open the memories adapter against a `Redex` manager.
    ///
    /// Uses [`MEMORIES_CHANNEL`] (`"cortex/memories"`). Replays the
    /// full history into state on open.
    ///
    /// `async` because the constructor awaits the fold task's catch-up
    /// before returning (BUG #148): the inner `WatermarkingFold`
    /// observes every replayed event's `EventMeta` and advances
    /// `app_seq` past any pre-existing same-origin `seq_or_ts`, so
    /// the first `ingest_typed` after `open` cannot collide with an
    /// already-stored event.
    pub async fn open(redex: &Redex, origin_hash: u32) -> Result<Self, CortexAdapterError> {
        Self::open_with_config(redex, origin_hash, RedexFileConfig::default()).await
    }

    /// Like [`Self::open`] but with a caller-supplied `RedexFileConfig`.
    pub async fn open_with_config(
        redex: &Redex,
        origin_hash: u32,
        redex_config: RedexFileConfig,
    ) -> Result<Self, CortexAdapterError> {
        let name = ChannelName::new(MEMORIES_CHANNEL).map_err(|e| {
            CortexAdapterError::Redex(super::super::super::redex::RedexError::Channel(
                e.to_string(),
            ))
        })?;
        let app_seq = Arc::new(AtomicU64::new(0));
        let fold = WatermarkingFold::new(MemoriesFold, app_seq.clone(), origin_hash);
        let inner = CortexAdapter::open(
            redex,
            &name,
            redex_config,
            CortexAdapterConfig::default(),
            fold,
            MemoriesState::new(),
        )?;

        // Wait for the fold task to catch up so the wrapper has
        // observed every pre-existing event before any caller-driven
        // ingest can race against it. `redex.open_file` is idempotent
        // (returns the same handle the inner adapter already holds),
        // so re-opening here is cheap.
        let file = redex.open_file(&name, redex_config)?;
        let next_seq = file.next_seq();
        if next_seq > 0 {
            inner.wait_for_seq(next_seq - 1).await;
        }

        Ok(Self {
            inner,
            origin_hash,
            app_seq,
        })
    }

    /// Store a new memory. Returns the RedEX seq of the append.
    pub fn store(
        &self,
        id: MemoryId,
        content: impl Into<String>,
        tags: impl IntoIterator<Item = String>,
        source: impl Into<String>,
        now_ns: u64,
    ) -> Result<u64, CortexAdapterError> {
        let payload = MemoryStoredPayload {
            id,
            content: content.into(),
            tags: tags.into_iter().collect(),
            source: source.into(),
            now_ns,
        };
        self.ingest_typed(DISPATCH_MEMORY_STORED, &payload)
    }

    /// Replace the tag set on an existing memory. No-op at fold time
    /// if `id` is unknown.
    pub fn retag(
        &self,
        id: MemoryId,
        tags: impl IntoIterator<Item = String>,
        now_ns: u64,
    ) -> Result<u64, CortexAdapterError> {
        let payload = MemoryRetaggedPayload {
            id,
            tags: tags.into_iter().collect(),
            now_ns,
        };
        self.ingest_typed(DISPATCH_MEMORY_RETAGGED, &payload)
    }

    /// Pin a memory.
    pub fn pin(&self, id: MemoryId, now_ns: u64) -> Result<u64, CortexAdapterError> {
        let payload = MemoryPinTogglePayload { id, now_ns };
        self.ingest_typed(DISPATCH_MEMORY_PINNED, &payload)
    }

    /// Unpin a memory.
    pub fn unpin(&self, id: MemoryId, now_ns: u64) -> Result<u64, CortexAdapterError> {
        let payload = MemoryPinTogglePayload { id, now_ns };
        self.ingest_typed(DISPATCH_MEMORY_UNPINNED, &payload)
    }

    /// Delete a memory.
    pub fn delete(&self, id: MemoryId) -> Result<u64, CortexAdapterError> {
        let payload = MemoryDeletedPayload { id };
        self.ingest_typed(DISPATCH_MEMORY_DELETED, &payload)
    }

    /// Read-only access to the materialized state.
    pub fn state(&self) -> Arc<RwLock<MemoriesState>> {
        self.inner.state()
    }

    /// Total memory count in the current state.
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
    pub fn as_cortex(&self) -> &CortexAdapter<MemoriesState> {
        &self.inner
    }

    /// Start building a reactive watcher.
    pub fn watch(&self) -> MemoriesWatcher {
        MemoriesWatcher::new(self.inner.state(), self.inner.changes().boxed())
    }

    /// One-shot combo: a snapshot of the current filter result PLUS
    /// a stream that emits every **subsequent** change to that
    /// filter. The stream skips the initial emission so the caller
    /// doesn't see the snapshot twice — the snapshot is the initial
    /// state; the stream carries deltas from there forward.
    pub fn snapshot_and_watch(&self, watcher: MemoriesWatcher) -> MemoriesSnapshotAndWatch {
        use futures::StreamExt;
        let initial = {
            let state = self.inner.state();
            let guard = state.read();
            watcher.spec_for_snapshot().execute(&guard)
        };
        // BUG #143: pre-fix used the sticky `skip_while`, which
        // starves consumers under (A → B → A) state oscillations
        // collapsed by the single-slot `tokio::sync::watch` — see
        // `tasks/adapter.rs::snapshot_and_watch` for the full
        // rationale. The fix here is identical: skip ONLY the first
        // emission if it still equals the snapshot, forward
        // everything after that.
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
        let payload = MemoriesSnapshotPayload {
            app_seq: self.app_seq.load(Ordering::Acquire),
            inner,
        };
        let bytes = postcard::to_allocvec(&payload).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("memories snapshot wrap: {}", e)))
        })?;
        Ok((bytes, last_seq))
    }

    /// Open the memories adapter from a snapshot.
    ///
    /// See [`Self::open`] for why this is `async`.
    pub async fn open_from_snapshot(
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
        .await
    }

    /// Like [`Self::open_from_snapshot`] but with a caller-supplied
    /// `RedexFileConfig`.
    pub async fn open_from_snapshot_with_config(
        redex: &Redex,
        origin_hash: u32,
        redex_config: RedexFileConfig,
        state_bytes: &[u8],
        last_seq: Option<u64>,
    ) -> Result<Self, CortexAdapterError> {
        let payload: MemoriesSnapshotPayload = postcard::from_bytes(state_bytes).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!(
                "memories snapshot unwrap: {}",
                e
            )))
        })?;
        let name = ChannelName::new(MEMORIES_CHANNEL)
            .map_err(|e| CortexAdapterError::Redex(RedexError::Channel(e.to_string())))?;

        // Pre-load the snapshot's persisted counter into the shared
        // atomic. The wrapper fold then advances it via fetch_max as
        // it catches up over the post-`last_seq` tail — replacing
        // the synchronous `read_range` walk that pre-fix doubled
        // startup IO/CPU on large logs (BUG #148).
        let app_seq = Arc::new(AtomicU64::new(payload.app_seq));
        let fold = WatermarkingFold::new(MemoriesFold, app_seq.clone(), origin_hash);
        let inner = CortexAdapter::open_from_snapshot(
            redex,
            &name,
            redex_config,
            CortexAdapterConfig::default(),
            fold,
            &payload.inner,
            last_seq,
        )?;

        let file = redex.open_file(&name, redex_config)?;
        let next_seq = file.next_seq();
        if next_seq > 0 {
            inner.wait_for_seq(next_seq - 1).await;
        }

        Ok(Self {
            inner,
            origin_hash,
            app_seq,
        })
    }

    /// BUG #126: see `tasks/adapter.rs::ingest_typed` for the full
    /// rationale. Same hazard pattern: pre-fix `app_seq` advanced
    /// before the inner ingest, leaving a phantom seq on
    /// `inner.ingest` failure. Now: load → build envelope → ingest
    /// → CAS-commit, mirroring the tasks adapter.
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

        // See `tasks/adapter.rs::ingest_typed` for the full
        // load → ingest → CAS-commit rationale.
        let app_seq = self.app_seq.load(Ordering::Acquire);
        let meta = EventMeta::new(dispatch, 0, self.origin_hash, app_seq, checksum);
        let env = EventEnvelope::new(meta, payload_bytes);
        let seq = self.inner.ingest(env)?;

        match self.app_seq.compare_exchange(
            app_seq,
            app_seq + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(seq),
            Err(_actual) => Err(CortexAdapterError::Redex(
                super::super::super::redex::RedexError::Encode(format!(
                    "concurrent ingest_typed produced duplicate app_seq={}; \
                     rebuild adapter from snapshot to reconcile",
                    app_seq
                )),
            )),
        }
    }
}

impl std::fmt::Debug for MemoriesAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoriesAdapter")
            .field("origin_hash", &self.origin_hash)
            .field("app_seq", &self.app_seq.load(Ordering::Acquire))
            .field("inner", &self.inner)
            .finish()
    }
}
