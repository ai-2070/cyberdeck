//! `CortexAdapter<State>` — one RedEX file, one fold, one materialized
//! state.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

/// Item type yielded by [`CortexAdapter::changes_with_lag`].
///
/// Pre-fix the `changes()` stream used `filter_map(|r| r.ok())`
/// which silently dropped `BroadcastStream::Lagged(n)` errors —
/// downstream telemetry consumers had no way to surface "you
/// missed N changes." This enum exposes both shapes; subscribers
/// who need only the latest sequence can stay on
/// [`CortexAdapter::changes`] (BUG #144).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeEvent {
    /// A successful fold apply produced this RedEX sequence.
    Seq(u64),
    /// The subscriber fell `n` events behind the broadcast channel
    /// and `n` change notifications were dropped. By the time the
    /// subscriber sees this, `state()` already reflects past those
    /// events — the lag value is purely observability.
    Lagged(u64),
}

struct AdapterInner<State> {
    file: RedexFile,
    state: Arc<RwLock<State>>,
    /// Highest RedEX seq applied to state, as a signed i64 so we can
    /// sentinel "nothing folded yet" with `start_seq - 1` (can be
    /// negative when `start_seq == 0`).
    /// Highest RedEX seq folded into state, or `u64::MAX` as the
    /// "nothing folded yet" sentinel. `u64::MAX` is safe as a
    /// sentinel because the `open_from_snapshot` overflow guard
    /// rejects `last_seq == u64::MAX`, so no real event can ever
    /// occupy that slot.
    folded_through_seq: AtomicU64,
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
        if v == u64::MAX {
            None
        } else {
            Some(v)
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
        loop {
            let notified = self.inner.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            let watermark = self.inner.folded_through_seq.load(Ordering::Acquire);
            // `u64::MAX` is the "nothing folded yet" sentinel — any
            // other value is a real applied seq, so `watermark >= seq`
            // after the sentinel check returns exactly when seq has
            // been applied.
            if watermark != u64::MAX && watermark >= seq {
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
    /// how many signals were missed. Subscribers that need to
    /// observe lag (e.g. for telemetry or reactive-backpressure)
    /// should use [`Self::changes_with_lag`] instead.
    ///
    /// The stream ends when all adapter handles have been dropped
    /// and the fold task has exited.
    pub fn changes(&self) -> impl Stream<Item = u64> + Send + 'static {
        BroadcastStream::new(self.inner.changes_tx.subscribe())
            .filter_map(|r| async move { r.ok() })
    }

    /// Stream of changes that surfaces broadcast-channel lag as a
    /// `Lagged(n)` event interleaved with the sequence emissions.
    ///
    /// The yielded items are [`ChangeEvent`]s — `Seq(u64)` for a
    /// successful fold-apply notification, and `Lagged(n)` when the
    /// subscriber fell `n` events behind the broadcast channel
    /// (capacity 64). Pre-fix [`Self::changes`] silently dropped
    /// `Lagged` errors via `filter_map(|r| r.ok())`; downstream
    /// telemetry consumers had no way to surface "you missed N
    /// changes." This method is the lossless counterpart — by the
    /// time a subscriber sees `Lagged(n)`, `state()` already
    /// reflects past those n events, so the subscriber can react
    /// (re-read state, log lag, apply backpressure) without
    /// missing data.
    ///
    /// BUG #144.
    pub fn changes_with_lag(&self) -> impl Stream<Item = ChangeEvent> + Send + 'static {
        use futures::StreamExt;
        BroadcastStream::new(self.inner.changes_tx.subscribe()).map(|r| match r {
            Ok(seq) => ChangeEvent::Seq(seq),
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                ChangeEvent::Lagged(n)
            }
        })
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
        fold: F,
        initial_state: State,
    ) -> Result<Self, CortexAdapterError>
    where
        F: RedexFold<State> + Send + 'static,
    {
        // BUG #134: positions that skip a non-empty event prefix
        // require externally-rehydrated state — the watermark
        // would otherwise advance past events the adapter never
        // saw, making `wait_for_seq(k)` return immediately for
        // skipped k while `state` has never observed those events.
        // Callers using these positions must use
        // `open_from_snapshot` (which carries the matching
        // `last_seq` + serialized state) and routes through
        // `open_unchecked` below.
        match adapter_config.start {
            StartPosition::FromBeginning => {}
            StartPosition::LiveOnly => {
                return Err(CortexAdapterError::InvalidStartPosition("LiveOnly"));
            }
            StartPosition::FromSeq(n) if n > 0 => {
                return Err(CortexAdapterError::InvalidStartPosition("FromSeq(n>0)"));
            }
            StartPosition::FromSeq(_) => {} // FromSeq(0) is equivalent to FromBeginning
        }
        Self::open_unchecked(redex, name, redex_config, adapter_config, fold, initial_state)
    }

    /// Internal open path that bypasses the BUG #134 start-position
    /// guard. Used by `open_from_snapshot`, where the externally-
    /// rehydrated state is the legitimate reason to skip the event
    /// prefix.
    fn open_unchecked<F>(
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
        // Initial watermark encodes "applied through start_seq-1", so
        // `wait_for_seq(start_seq-1)` returns immediately after open
        // (those seqs are conceptually behind us) while
        // `wait_for_seq(start_seq)` blocks until the first event
        // actually folds. `start_seq == 0` encodes the "nothing
        // folded yet" state with the `u64::MAX` sentinel.
        let initial_watermark: u64 = if start_seq == 0 {
            u64::MAX
        } else {
            start_seq - 1
        };
        let (changes_tx, _) = broadcast::channel(CHANGES_BROADCAST_CAP);
        let inner = Arc::new(AdapterInner {
            file: file.clone(),
            state: state.clone(),
            folded_through_seq: AtomicU64::new(initial_watermark),
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
    /// postcard-serialized state and `last_seq` is the highest RedEX
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
        let bytes = postcard::to_allocvec(&*state).map_err(|e| {
            CortexAdapterError::Redex(RedexError::Encode(format!("snapshot serialize: {}", e)))
        })?;
        let watermark = self.inner.folded_through_seq.load(Ordering::Acquire);
        let last_seq = if watermark == u64::MAX {
            None
        } else {
            Some(watermark)
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
        let initial_state: State = postcard::from_bytes(state_bytes).map_err(|e| {
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
        // BUG #134: route through `open_unchecked` so the
        // externally-rehydrated state can skip its event prefix.
        Self::open_unchecked(redex, name, redex_config, config, fold, initial_state)
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
            inner.folded_through_seq.store(seq, Ordering::Release);
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

    // ========================================================================
    // BUG #134: open must reject FromSeq(n>0) / LiveOnly
    // ========================================================================

    /// `open` rejects `StartPosition::FromSeq(n)` for n > 0
    /// because the watermark would advance past events the adapter
    /// never folded, leaving `wait_for_seq` lying about applied
    /// state. Callers that intentionally skip a prefix must use
    /// `open_from_snapshot`.
    #[test]
    fn open_rejects_from_seq_n_greater_than_zero() {
        let redex = Redex::new();
        let cfg = CortexAdapterConfig::new().with_start(StartPosition::FromSeq(5));
        let result = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/from-seq-guard"),
            RedexFileConfig::default(),
            cfg,
            CountFold,
            0u64,
        );
        assert!(
            matches!(
                result,
                Err(CortexAdapterError::InvalidStartPosition(_))
            ),
            "open must reject FromSeq(n>0) (BUG #134), got {:?}",
            result.map(|_| "Ok"),
        );
    }

    /// `open` rejects `StartPosition::LiveOnly` for the same
    /// reason — the start_seq is `file.next_seq()`, so any prior
    /// events go un-folded but the watermark advances past them.
    #[test]
    fn open_rejects_live_only_start_position() {
        let redex = Redex::new();
        let cfg = CortexAdapterConfig::new().with_start(StartPosition::LiveOnly);
        let result = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/live-only-guard"),
            RedexFileConfig::default(),
            cfg,
            CountFold,
            0u64,
        );
        assert!(
            matches!(
                result,
                Err(CortexAdapterError::InvalidStartPosition(_))
            ),
            "open must reject LiveOnly (BUG #134), got {:?}",
            result.map(|_| "Ok"),
        );
    }

    /// `FromSeq(0)` is equivalent to `FromBeginning` (no events
    /// skipped) and must still be accepted — pins the boundary so
    /// the BUG #134 guard doesn't accidentally lock out the
    /// degenerate-but-valid `FromSeq(0)` form.
    #[tokio::test]
    async fn open_accepts_from_seq_zero() {
        let redex = Redex::new();
        let cfg = CortexAdapterConfig::new().with_start(StartPosition::FromSeq(0));
        CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/from-seq-zero"),
            RedexFileConfig::default(),
            cfg,
            CountFold,
            0u64,
        )
        .expect("FromSeq(0) is equivalent to FromBeginning and must be accepted");
    }

    // ========================================================================
    // BUG #144: changes_with_lag must surface BroadcastStream::Lagged
    // ========================================================================

    /// `changes_with_lag` yields a `ChangeEvent::Lagged(n)` when a
    /// subscriber falls behind the broadcast channel capacity. Pre-
    /// fix `changes()` silently dropped these events via
    /// `filter_map(|r| r.ok())`; downstream telemetry consumers had
    /// no way to detect or count missed change notifications.
    ///
    /// Setup: subscribe, then ingest more than CHANGES_BROADCAST_CAP
    /// (64) events without polling the stream. The broadcast channel
    /// drops the oldest, and the next stream poll surfaces a
    /// `Lagged(n)` for the dropped count.
    #[tokio::test]
    async fn changes_with_lag_yields_lagged_when_subscriber_falls_behind() {
        let redex = Redex::new();
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/lag"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            CountFold,
            0u64,
        )
        .unwrap();

        let stream = adapter.changes_with_lag();
        tokio::pin!(stream);

        // Ingest CHANGES_BROADCAST_CAP + 16 events without polling
        // the stream. The broadcast channel will drop the oldest 16
        // (or thereabouts — the exact count depends on broadcast
        // semantics; we just need at least one Lagged emission).
        let total = (CHANGES_BROADCAST_CAP + 16) as u64;
        for i in 0..total {
            let meta = EventMeta::new(0, 0, 0, i, 0);
            let env = EventEnvelope::new(meta, Bytes::from_static(b""));
            let seq = adapter.ingest(env).unwrap();
            adapter.wait_for_seq(seq).await;
        }

        // First poll should see a Lagged event (the broadcast channel
        // has overflowed). Drain the stream up to a reasonable cap and
        // assert at least one Lagged event was observed.
        let mut saw_lagged = false;
        let mut saw_seq = false;
        for _ in 0..(total as usize + 4) {
            match tokio::time::timeout(std::time::Duration::from_millis(50), stream.next()).await {
                Ok(Some(ChangeEvent::Lagged(n))) => {
                    saw_lagged = true;
                    assert!(n > 0, "Lagged count must be positive");
                }
                Ok(Some(ChangeEvent::Seq(_))) => {
                    saw_seq = true;
                }
                Ok(None) | Err(_) => break,
            }
        }
        assert!(
            saw_lagged,
            "subscriber that fell behind {} events must observe Lagged (BUG #144)",
            CHANGES_BROADCAST_CAP + 16,
        );
        assert!(
            saw_seq,
            "the stream should still emit Seq events after the lag",
        );
    }

    /// `changes()` continues to silently drop lag (the documented
    /// best-effort behavior) — pins the contract so a future
    /// refactor doesn't accidentally surface `Lagged` through the
    /// simple stream and break consumers that don't want it.
    #[tokio::test]
    async fn changes_filters_out_lag_silently() {
        let redex = Redex::new();
        let adapter = CortexAdapter::<u64>::open(
            &redex,
            &cn("cortex/lag-silent"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            CountFold,
            0u64,
        )
        .unwrap();

        let stream = adapter.changes();
        tokio::pin!(stream);

        // Same overflow setup.
        let total = (CHANGES_BROADCAST_CAP + 16) as u64;
        for i in 0..total {
            let meta = EventMeta::new(0, 0, 0, i, 0);
            let env = EventEnvelope::new(meta, Bytes::from_static(b""));
            let seq = adapter.ingest(env).unwrap();
            adapter.wait_for_seq(seq).await;
        }

        // Drain everything we can from the stream. Item type is `u64`
        // (not Result), so we can't observe Lagged in any form. Just
        // verify the stream still produces some seqs without errors.
        let mut got_seq = false;
        for _ in 0..(total as usize + 4) {
            match tokio::time::timeout(std::time::Duration::from_millis(50), stream.next()).await {
                Ok(Some(_seq)) => {
                    got_seq = true;
                }
                Ok(None) | Err(_) => break,
            }
        }
        assert!(got_seq, "changes() must still emit seqs after lag");
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
