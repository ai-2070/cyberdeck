//! `CortexAdapter<State>` — one RedEX file, one fold, one materialized
//! state.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

use futures::{Stream, StreamExt};
use parking_lot::RwLock;
use tokio::sync::{broadcast, Notify};
use tokio_stream::wrappers::BroadcastStream;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::super::channel::ChannelName;
use super::super::redex::{Redex, RedexError, RedexEvent, RedexFile, RedexFileConfig, RedexFold};
use super::config::{CortexAdapterConfig, FoldErrorPolicy, StartPosition};
use super::envelope::IntoRedexPayload;
use super::error::CortexAdapterError;
use super::meta::EVENT_META_SIZE;

/// One-file CortEX adapter: projects envelopes into RedEX payloads,
/// tails the same file, drives a [`RedexFold`] implementation, and
/// exposes the materialized state as a read handle.
///
/// Created via [`Self::open`].
pub struct CortexAdapter<State> {
    inner: Arc<AdapterInner<State>>,
}

/// Capacity of the post-fold change-notification broadcast channel.
/// A slow subscriber that falls more than this many events behind
/// gets a `Lagged` signal and should re-read state fresh.
const CHANGES_BROADCAST_CAP: usize = 64;

struct AdapterInner<State> {
    file: RedexFile,
    state: Arc<RwLock<State>>,
    /// Highest RedEX seq applied to state, as a signed i64 so we can
    /// sentinel "nothing folded yet" with `start_seq - 1` (can be
    /// negative when `start_seq == 0`).
    folded_through_seq: AtomicI64,
    fold_errors: AtomicU64,
    running: AtomicBool,
    closed: AtomicBool,
    notify: Notify,
    shutdown: Notify,
    /// Broadcast of RedEX seqs after each successful (or LogAndContinue-skipped)
    /// fold apply. Subscribers: see [`CortexAdapter::changes`].
    changes_tx: broadcast::Sender<u64>,
}

impl<State> CortexAdapter<State> {
    /// Read-only access to the materialized state. The returned `Arc`
    /// is cheap to clone; all readers and the fold task share the
    /// same `RwLock`.
    pub fn state(&self) -> Arc<RwLock<State>> {
        self.inner.state.clone()
    }

    /// Highest RedEX sequence that has been folded into state.
    /// `None` if no event has been folded yet since open.
    pub fn folded_through_seq(&self) -> Option<u64> {
        let v = self.inner.folded_through_seq.load(Ordering::Acquire);
        if v < 0 {
            None
        } else {
            Some(v as u64)
        }
    }

    /// Cumulative count of fold errors (only ever increases under
    /// [`FoldErrorPolicy::LogAndContinue`]; under `Stop` it is 0 or
    /// 1, with the task exiting after the first error).
    pub fn fold_errors(&self) -> u64 {
        self.inner.fold_errors.load(Ordering::Acquire)
    }

    /// True if the fold task is currently running (has not observed
    /// shutdown, an error under `Stop`, or a tail-end signal).
    pub fn is_running(&self) -> bool {
        self.inner.running.load(Ordering::Acquire)
    }

    /// Block until the fold task has applied every event up through
    /// `seq`, or until the fold task stops (e.g. close, fold error
    /// under `Stop`). Returning after the task has stopped is
    /// correct behavior — callers should re-check
    /// [`Self::is_running`] if they need to distinguish.
    ///
    /// Use pattern:
    /// ```ignore
    /// let seq = adapter.ingest(envelope)?;
    /// adapter.wait_for_seq(seq).await;
    /// let state = adapter.state().read();
    /// // state reflects the ingest.
    /// ```
    pub async fn wait_for_seq(&self, seq: u64) {
        let target = seq as i64;
        loop {
            let notified = self.inner.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            if self.inner.folded_through_seq.load(Ordering::Acquire) >= target {
                return;
            }
            if !self.inner.running.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }

    /// Close the adapter. Stops the fold task (after it finishes any
    /// in-progress apply), leaves the RedEX file open so other
    /// adapters / callers can continue using it, and leaves the
    /// state handle readable. Idempotent.
    pub fn close(&self) -> Result<(), CortexAdapterError> {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        // `notify_one()` stores a permit if the fold task hasn't yet
        // reached its `shutdown.notified()` poll, so a close that
        // races the spawn → first-select window is still observed.
        // `notify_waiters()` would drop the signal in that window.
        self.inner.shutdown.notify_one();
        Ok(())
    }

    /// Stream of RedEX sequences, one per successful (or
    /// `LogAndContinue`-skipped) fold application. Used by reactive
    /// queries: on each emission, the caller re-reads
    /// [`Self::state`] to compute its current view.
    ///
    /// Lag semantics: if a subscriber falls more than 64 events
    /// behind (the internal broadcast channel capacity), the channel
    /// drops intermediate events. This implementation filters lag
    /// errors out silently — by the time the subscriber catches up,
    /// `state()` reflects the latest applied events regardless of
    /// how many signals were missed.
    ///
    /// The stream ends when all adapter handles have been dropped
    /// and the fold task has exited.
    pub fn changes(&self) -> impl Stream<Item = u64> + Send + 'static {
        BroadcastStream::new(self.inner.changes_tx.subscribe())
            .filter_map(|r| async move { r.ok() })
    }

    /// Append an envelope. Projects to `(EventMeta, tail)`, builds the
    /// concatenated payload, calls [`RedexFile::append`], and returns
    /// the assigned RedEX sequence.
    pub fn ingest<E: IntoRedexPayload>(&self, envelope: E) -> Result<u64, CortexAdapterError> {
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(CortexAdapterError::Closed);
        }
        let (meta, tail) = envelope.into_redex_payload();
        let mut buf = Vec::with_capacity(EVENT_META_SIZE + tail.len());
        buf.extend_from_slice(&meta.to_bytes());
        buf.extend_from_slice(&tail);
        Ok(self.inner.file.append(&buf)?)
    }
}

impl<State: Send + Sync + 'static> CortexAdapter<State> {
    /// Open an adapter against a RedEX file.
    ///
    /// Opens (or reuses) `<redex>/<name>` via
    /// [`Redex::open_file`](super::super::redex::Redex::open_file),
    /// spawns a background task that tails the file and drives
    /// `fold`, and returns the handle.
    pub fn open<F>(
        redex: &Redex,
        name: &ChannelName,
        redex_config: RedexFileConfig,
        adapter_config: CortexAdapterConfig,
        mut fold: F,
        initial_state: State,
    ) -> Result<Self, CortexAdapterError>
    where
        F: RedexFold<State> + Send + 'static,
    {
        let file = redex.open_file(name, redex_config)?;

        let start_seq = match adapter_config.start {
            StartPosition::FromBeginning => 0,
            StartPosition::LiveOnly => file.next_seq(),
            StartPosition::FromSeq(n) => n,
        };

        let state = Arc::new(RwLock::new(initial_state));
        // Initial watermark is start_seq - 1 (can be -1 for FromBeginning)
        // so that `folded_through_seq >= start_seq` holds iff the seq
        // has actually been applied.
        let initial_watermark = (start_seq as i64).wrapping_sub(1);
        let (changes_tx, _) = broadcast::channel(CHANGES_BROADCAST_CAP);
        let inner = Arc::new(AdapterInner {
            file: file.clone(),
            state: state.clone(),
            folded_through_seq: AtomicI64::new(initial_watermark),
            fold_errors: AtomicU64::new(0),
            running: AtomicBool::new(true),
            closed: AtomicBool::new(false),
            notify: Notify::new(),
            shutdown: Notify::new(),
            changes_tx,
        });

        let policy = adapter_config.on_fold_error;
        let inner_task = inner.clone();
        let mut stream = Box::pin(file.tail(start_seq));

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = inner_task.shutdown.notified() => {
                        break;
                    }
                    next = stream.next() => {
                        match next {
                            None => break,
                            Some(Err(_)) => {
                                // Tail yielded an error (e.g. file
                                // closed). Stop cleanly.
                                break;
                            }
                            Some(Ok(event)) => {
                                if handle_event(
                                    &inner_task,
                                    &mut fold,
                                    &event,
                                    policy,
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            inner_task.running.store(false, Ordering::Release);
            inner_task.notify.notify_waiters();
        });

        Ok(Self { inner })
    }
}

impl<State> CortexAdapter<State>
where
    State: Serialize + Send + Sync + 'static,
{
    /// Capture a point-in-time snapshot of the materialized state.
    ///
    /// Returns `(state_bytes, last_seq)` where `state_bytes` is the
    /// bincode-serialized state and `last_seq` is the highest RedEX
    /// sequence folded into it. Persist both together — they form a
    /// consistent pair, guaranteed by the adapter holding the state
    /// write lock while advancing the watermark.
    ///
    /// Restore via [`Self::open_from_snapshot`] on a State that also
    /// implements `DeserializeOwned`.
    ///
    /// `last_seq` is `None` if no event has been folded yet since
    /// open (the snapshot is still meaningful — it represents the
    /// initial State — but callers typically wait until
    /// [`Self::wait_for_seq`] has returned before snapshotting).
    pub fn snapshot(&self) -> Result<(Vec<u8>, Option<u64>), CortexAdapterError> {
        let state = self.inner.state.read();
        let bytes = bincode::serialize(&*state).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("snapshot serialize: {}", e)))
        })?;
        let watermark = self.inner.folded_through_seq.load(Ordering::Acquire);
        let last_seq = if watermark < 0 {
            None
        } else {
            Some(watermark as u64)
        };
        Ok((bytes, last_seq))
    }
}

impl<State> CortexAdapter<State>
where
    State: DeserializeOwned + Send + Sync + 'static,
{
    /// Open an adapter from a previously-captured snapshot, skipping
    /// the `[0, last_seq]` replay.
    ///
    /// `state_bytes` is the blob returned from [`Self::snapshot`].
    /// `last_seq` is its companion sequence. The tail starts at
    /// `last_seq + 1`; the initial state is deserialized from the
    /// blob; the fold task is spawned as usual.
    ///
    /// If `last_seq` is `None` (no events had been folded at
    /// snapshot time), the tail starts at seq 0 — equivalent to
    /// `StartPosition::FromBeginning` with the deserialized initial
    /// state.
    pub fn open_from_snapshot<F>(
        redex: &Redex,
        name: &ChannelName,
        redex_config: RedexFileConfig,
        adapter_config: CortexAdapterConfig,
        fold: F,
        state_bytes: &[u8],
        last_seq: Option<u64>,
    ) -> Result<Self, CortexAdapterError>
    where
        F: RedexFold<State> + Send + 'static,
    {
        let initial_state: State = bincode::deserialize(state_bytes).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("deserialize snapshot: {}", e)))
        })?;
        let start = match last_seq {
            Some(n) => {
                let next = n.checked_add(1).ok_or_else(|| {
                    CortexAdapterError::Redex(RedexError::Encode(
                        "snapshot last_seq at u64::MAX; cannot resume".to_string(),
                    ))
                })?;
                StartPosition::FromSeq(next)
            }
            None => StartPosition::FromBeginning,
        };
        let config = CortexAdapterConfig {
            start,
            on_fold_error: adapter_config.on_fold_error,
        };
        Self::open(redex, name, redex_config, config, fold, initial_state)
    }
}

impl<State> Clone for CortexAdapter<State> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<State> std::fmt::Debug for CortexAdapter<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CortexAdapter")
            .field("folded_through_seq", &self.folded_through_seq())
            .field("fold_errors", &self.fold_errors())
            .field("running", &self.is_running())
            .field("closed", &self.inner.closed.load(Ordering::Acquire))
            .finish()
    }
}

/// Apply one event. Returns `true` if the task should exit
/// (Stop policy + error).
fn handle_event<State, F>(
    inner: &Arc<AdapterInner<State>>,
    fold: &mut F,
    event: &RedexEvent,
    policy: FoldErrorPolicy,
) -> bool
where
    F: RedexFold<State>,
{
    let seq = event.entry.seq;
    // Hold the write lock across both the fold and the watermark
    // update so that a `snapshot()` holding `state.read()` observes
    // a consistent `(state, folded_through_seq)` pair — otherwise
    // the state could reflect seq N while the watermark still reads
    // N-1, causing restore to double-apply event N.
    let result = {
        let mut state = inner.state.write();
        let r = fold.apply(event, &mut state);
        let advance = matches!(
            (&r, policy),
            (Ok(()), _) | (Err(_), FoldErrorPolicy::LogAndContinue)
        );
        if advance {
            inner
                .folded_through_seq
                .store(seq as i64, Ordering::Release);
        }
        r
    };

    match result {
        Ok(()) => {
            inner.notify.notify_waiters();
            let _ = inner.changes_tx.send(seq);
            false
        }
        Err(err) => {
            inner.fold_errors.fetch_add(1, Ordering::AcqRel);
            tracing::warn!(seq = seq, error = %err, "cortex fold error");
            match policy {
                FoldErrorPolicy::Stop => true,
                FoldErrorPolicy::LogAndContinue => {
                    // Watermark was already advanced inside the lock
                    // above; just notify waiters.
                    inner.notify.notify_waiters();
                    let _ = inner.changes_tx.send(seq);
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::channel::ChannelName;
    use super::super::super::redex::{RedexError, RedexFold};
    use super::super::envelope::EventEnvelope;
    use super::super::meta::EventMeta;
    use super::*;
    use bytes::Bytes;

    fn cn(s: &str) -> ChannelName {
        ChannelName::new(s).unwrap()
    }

    struct CountFold;
    impl RedexFold<u64> for CountFold {
        fn apply(&mut self, _ev: &RedexEvent, state: &mut u64) -> Result<(), RedexError> {
            *state += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_open_ingest_wait_query() {
        let redex = Redex::new();
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/counts"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            CountFold,
            0u64,
        )
        .unwrap();

        for i in 0..10u64 {
            let meta = EventMeta::new(1, 0, 1, i, 0);
            let env = EventEnvelope::new(meta, Bytes::from_static(b""));
            let seq = adapter.ingest(env).unwrap();
            adapter.wait_for_seq(seq).await;
        }

        assert_eq!(*adapter.state().read(), 10);
        assert_eq!(adapter.fold_errors(), 0);
        assert!(adapter.is_running());
    }

    #[tokio::test]
    async fn test_close_stops_fold_task() {
        let redex = Redex::new();
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/close"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            CountFold,
            0u64,
        )
        .unwrap();

        adapter.close().unwrap();
        // Close is idempotent.
        adapter.close().unwrap();

        // Ingest after close returns Closed.
        let meta = EventMeta::new(0, 0, 0, 0, 0);
        let env = EventEnvelope::new(meta, Bytes::from_static(b""));
        let err = adapter.ingest(env).unwrap_err();
        assert!(matches!(err, CortexAdapterError::Closed));

        // State handle still readable.
        assert_eq!(*adapter.state().read(), 0);
    }

    struct FailAtSeq(u64);
    impl RedexFold<u64> for FailAtSeq {
        fn apply(&mut self, ev: &RedexEvent, state: &mut u64) -> Result<(), RedexError> {
            if ev.entry.seq == self.0 {
                Err(RedexError::Encode(format!(
                    "deliberate failure at seq {}",
                    ev.entry.seq
                )))
            } else {
                *state += 1;
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_stop_policy_halts_on_first_error() {
        let redex = Redex::new();
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/stop"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(), // Stop is default
            FailAtSeq(3),
            0u64,
        )
        .unwrap();

        for i in 0..10u64 {
            let meta = EventMeta::new(0, 0, 0, i, 0);
            let env = EventEnvelope::new(meta, Bytes::from_static(b""));
            adapter.ingest(env).unwrap();
        }

        // Wait until fold task stops. wait_for_seq returns on stop
        // even if the seq isn't reached.
        adapter.wait_for_seq(10).await;
        assert!(!adapter.is_running());
        assert_eq!(adapter.fold_errors(), 1);
        // Seqs 0..=2 folded; seq 3 errored; seqs 4..=9 never folded.
        assert_eq!(*adapter.state().read(), 3);
    }

    #[tokio::test]
    async fn test_log_and_continue_skips_errors() {
        let redex = Redex::new();
        let cfg =
            CortexAdapterConfig::new().with_fold_error_policy(FoldErrorPolicy::LogAndContinue);
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/lc"),
            RedexFileConfig::default(),
            cfg,
            FailAtSeq(3),
            0u64,
        )
        .unwrap();

        for i in 0..10u64 {
            let meta = EventMeta::new(0, 0, 0, i, 0);
            let env = EventEnvelope::new(meta, Bytes::from_static(b""));
            let seq = adapter.ingest(env).unwrap();
            adapter.wait_for_seq(seq).await;
        }

        assert!(adapter.is_running());
        assert_eq!(adapter.fold_errors(), 1);
        // All seqs except 3 were folded → state == 9.
        assert_eq!(*adapter.state().read(), 9);
    }
}
