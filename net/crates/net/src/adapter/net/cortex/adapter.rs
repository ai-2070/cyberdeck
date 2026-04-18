//! `CortexAdapter<State>` — one RedEX file, one fold, one materialized
//! state.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use parking_lot::RwLock;
use tokio::sync::Notify;

use super::super::channel::ChannelName;
use super::super::redex::{Redex, RedexEvent, RedexFile, RedexFileConfig, RedexFold};
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
        self.inner.shutdown.notify_waiters();
        Ok(())
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
        let inner = Arc::new(AdapterInner {
            file: file.clone(),
            state: state.clone(),
            folded_through_seq: AtomicI64::new(initial_watermark),
            fold_errors: AtomicU64::new(0),
            running: AtomicBool::new(true),
            closed: AtomicBool::new(false),
            notify: Notify::new(),
            shutdown: Notify::new(),
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
    let result = {
        let mut state = inner.state.write();
        fold.apply(event, &mut state)
    };

    match result {
        Ok(()) => {
            inner
                .folded_through_seq
                .store(seq as i64, Ordering::Release);
            inner.notify.notify_waiters();
            false
        }
        Err(err) => {
            inner.fold_errors.fetch_add(1, Ordering::AcqRel);
            tracing::warn!(seq = seq, error = %err, "cortex fold error");
            match policy {
                FoldErrorPolicy::Stop => true,
                FoldErrorPolicy::LogAndContinue => {
                    // Skip this event; advance watermark so
                    // wait_for_seq doesn't hang on the skipped seq.
                    inner
                        .folded_through_seq
                        .store(seq as i64, Ordering::Release);
                    inner.notify.notify_waiters();
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
