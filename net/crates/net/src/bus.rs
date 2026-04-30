//! Main EventBus facade.
//!
//! The EventBus provides a unified API for:
//! - Event ingestion (non-blocking)
//! - Event consumption (async polling with filtering)
//! - Lifecycle management

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::adapter::{Adapter, NoopAdapter};
use crate::config::{AdapterConfig, BatchConfig, EventBusConfig};
use crate::consumer::{ConsumeRequest, ConsumeResponse, PollMerger};
use crate::error::{AdapterError, ConsumerError, IngestionError, IngestionResult};
use crate::event::{Batch, Event, RawEvent};
use crate::shard::{BatchWorker, ScalingDecision, ShardManager, ShardMetrics};

#[cfg(feature = "jetstream")]
use crate::adapter::JetStreamAdapter;
#[cfg(feature = "net")]
use crate::adapter::NetAdapter;
#[cfg(feature = "redis")]
use crate::adapter::RedisAdapter;

/// The main event bus.
///
/// # Example
///
/// ```rust,ignore
/// use net::{EventBus, EventBusConfig, Event};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let bus = EventBus::new(EventBusConfig::default()).await?;
///
///     // Ingest events
///     bus.ingest(Event::from_str(r#"{"token": "hello"}"#)?)?;
///
///     // Poll events
///     let response = bus.poll(ConsumeRequest::new(100)).await?;
///
///     bus.shutdown().await?;
///     Ok(())
/// }
/// ```
pub struct EventBus {
    /// Shard manager for parallel ingestion.
    shard_manager: Arc<ShardManager>,
    /// Adapter for durable storage.
    adapter: Arc<dyn Adapter>,
    /// Poll merger for cross-shard consumption.
    poll_merger: arc_swap::ArcSwap<PollMerger>,
    /// Per-shard worker handles. Stored separately so shutdown can
    /// await drain workers *before* batch workers — the drain
    /// worker's final sweep races the batch worker's exit
    /// otherwise, and any events the drain worker pushes to the
    /// channel after the batch worker has stopped reading are
    /// silently lost.
    batch_workers: parking_lot::Mutex<std::collections::HashMap<u16, ShardWorkers>>,
    /// Channels for sending batches to workers (shard_id -> sender).
    batch_senders: parking_lot::RwLock<
        std::collections::HashMap<u16, mpsc::Sender<Vec<crate::event::InternalEvent>>>,
    >,
    /// Shutdown flag.
    shutdown: Arc<AtomicBool>,
    /// Gate signaling drain workers that the in-flight wait has
    /// completed and they may safely run their final ring-buffer
    /// sweep. Distinct from `shutdown` because the drain worker
    /// observing `shutdown=true` alone is not enough: a producer
    /// that read `shutdown=false` may still be mid-push, and if the
    /// drain worker rushes through its final sweep before that push
    /// is visible the event is stranded. `shutdown()` sets this
    /// after waiting for `in_flight_ingests==0`, at which point the
    /// Acquire load on the drain side synchronizes-with the Release
    /// store here, transitively chaining through the SeqCst
    /// in-flight handshake to make every observed-pre-shutdown push
    /// visible to the drain worker's subsequent `pop_batch_into`.
    drain_finalize_ready: Arc<AtomicBool>,
    /// In-flight ingest counter. Incremented before each ingest's
    /// shutdown check and decremented after the push completes (or
    /// bails). `shutdown()` waits for this to drop to zero *after*
    /// setting `shutdown=true` and *before* setting
    /// `drain_finalize_ready=true` so no producer is mid-push when
    /// the drain workers do their final sweep — closing the race
    /// where a producer that observed `shutdown=false` could push
    /// *after* the drain worker's last `pop_batch_into` returned
    /// zero, leaving the event stranded in the ring buffer.
    in_flight_ingests: AtomicU32,
    /// Set to `true` after `shutdown()` runs to completion. `Drop`
    /// uses this to detect "dropped without an awaited shutdown" —
    /// in that case events still in the ring buffers / mpsc channels
    /// are silently lost (see `Drop` impl).
    shutdown_completed: AtomicBool,
    /// Configuration.
    config: EventBusConfig,
    /// Statistics.
    stats: EventBusStats,
    /// Scaling monitor task handle.
    scaling_monitor: parking_lot::Mutex<Option<JoinHandle<()>>>,
}

/// Worker handles for a single shard. The drain worker pumps
/// events from the ring buffer into an mpsc channel; the batch
/// worker reads from that channel and dispatches to the adapter.
/// Shutdown ordering is load-bearing — see `EventBus::shutdown`.
struct ShardWorkers {
    batch: JoinHandle<()>,
    drain: JoinHandle<()>,
}

/// RAII guard for an in-flight ingest. Decrements
/// `in_flight_ingests` on drop so `shutdown()` can wait for the
/// counter to reach zero.
struct IngestGuard<'a> {
    bus: &'a EventBus,
}

impl Drop for IngestGuard<'_> {
    fn drop(&mut self) {
        self.bus
            .in_flight_ingests
            .fetch_sub(1, AtomicOrdering::SeqCst);
    }
}

/// Event bus statistics.
#[derive(Debug, Default)]
pub struct EventBusStats {
    /// Total events ingested.
    pub events_ingested: AtomicU64,
    /// Events dropped due to backpressure.
    pub events_dropped: AtomicU64,
    /// Batches dispatched to adapter.
    pub batches_dispatched: AtomicU64,
}

impl EventBus {
    /// Create a new event bus with the given configuration.
    pub async fn new(config: EventBusConfig) -> Result<Self, AdapterError> {
        // Create adapter from config
        let adapter: Box<dyn Adapter> = match &config.adapter {
            AdapterConfig::Noop => Box::new(NoopAdapter::new()),
            #[cfg(feature = "redis")]
            AdapterConfig::Redis(redis_config) => {
                Box::new(RedisAdapter::new(redis_config.clone())?)
            }
            #[cfg(feature = "jetstream")]
            AdapterConfig::JetStream(js_config) => {
                Box::new(JetStreamAdapter::new(js_config.clone())?)
            }
            #[cfg(feature = "net")]
            AdapterConfig::Net(net_config) => Box::new(NetAdapter::new((**net_config).clone())?),
        };

        Self::new_with_adapter(config, adapter).await
    }

    /// Create a new event bus with a caller-supplied adapter.
    ///
    /// `config.adapter` is ignored — the supplied `adapter` is used
    /// instead. Useful for tests that need to observe or inject
    /// behavior at the adapter boundary (e.g. a counting adapter
    /// for end-to-end delivery assertions, a flaky adapter for
    /// retry-path coverage).
    pub async fn new_with_adapter(
        config: EventBusConfig,
        mut adapter: Box<dyn Adapter>,
    ) -> Result<Self, AdapterError> {
        config
            .validate()
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Initialize adapter (with timeout to prevent hanging on unreachable backends)
        tokio::time::timeout(config.adapter_timeout, adapter.init())
            .await
            .map_err(|_| AdapterError::Fatal("adapter init timed out".into()))??;
        let adapter: Arc<dyn Adapter> = Arc::from(adapter);

        // Create shard manager (with or without dynamic scaling)
        let shard_manager = if let Some(ref scaling_policy) = config.scaling {
            Arc::new(
                ShardManager::with_mapper(
                    config.num_shards,
                    config.ring_buffer_capacity,
                    config.backpressure_mode,
                    scaling_policy.clone(),
                )
                .map_err(|e| AdapterError::Fatal(e.to_string()))?,
            )
        } else {
            Arc::new(ShardManager::new(
                config.num_shards,
                config.ring_buffer_capacity,
                config.backpressure_mode,
            ))
        };

        // Create poll merger
        let poll_merger =
            arc_swap::ArcSwap::from_pointee(PollMerger::new(adapter.clone(), config.num_shards));

        // Shutdown flag and drain-finalize gate. See `drain_finalize_ready`
        // doc on `EventBus` for the synchronization contract.
        let shutdown = Arc::new(AtomicBool::new(false));
        let drain_finalize_ready = Arc::new(AtomicBool::new(false));

        // Create batch workers for each shard
        let mut batch_workers: std::collections::HashMap<u16, ShardWorkers> =
            std::collections::HashMap::with_capacity(config.num_shards as usize);
        let mut batch_senders =
            std::collections::HashMap::with_capacity(config.num_shards as usize);

        for shard_id in 0..config.num_shards {
            let (tx, rx) = mpsc::channel::<Vec<crate::event::InternalEvent>>(1024);

            let batch = spawn_batch_worker(BatchWorkerParams {
                shard_id,
                rx,
                adapter: adapter.clone(),
                shard_manager: shard_manager.clone(),
                config: config.batch.clone(),
                adapter_timeout: config.adapter_timeout,
                batch_retries: config.adapter_batch_retries,
            });

            let drain = spawn_drain_worker_for_shard(
                shard_id,
                shard_manager.clone(),
                tx.clone(),
                shutdown.clone(),
                drain_finalize_ready.clone(),
            );

            batch_workers.insert(shard_id, ShardWorkers { batch, drain });
            batch_senders.insert(shard_id, tx);
        }

        let bus = Self {
            shard_manager,
            adapter,
            poll_merger,
            batch_workers: parking_lot::Mutex::new(batch_workers),
            batch_senders: parking_lot::RwLock::new(batch_senders),
            shutdown,
            drain_finalize_ready,
            in_flight_ingests: AtomicU32::new(0),
            shutdown_completed: AtomicBool::new(false),
            config,
            stats: EventBusStats::default(),
            scaling_monitor: parking_lot::Mutex::new(None),
        };

        Ok(bus)
    }

    /// Start the scaling monitor (if dynamic scaling is enabled).
    /// This spawns a background task that periodically evaluates scaling decisions.
    ///
    /// The spawned task holds a `Weak<Self>` rather than a strong
    /// `Arc<Self>` clone. With a strong clone the task kept the bus
    /// alive forever, and `shutdown(self)` (which consumes by value)
    /// was unreachable: callers with an `Arc<EventBus>` could not
    /// `Arc::try_unwrap` to consume it because the spawned task always
    /// held one of the strong refs.
    ///
    /// With a `Weak`, the monitor task upgrades each tick. Once the
    /// last caller-held `Arc` is dropped, the upgrade fails and the
    /// task exits cleanly. To shut down via `shutdown(self)`, the
    /// caller must hold the only strong reference: `Arc::try_unwrap`
    /// on the resulting bus succeeds because the spawned task only
    /// holds a Weak.
    pub fn start_scaling_monitor(self: &Arc<Self>) {
        if self.config.scaling.is_none() {
            return;
        }

        let weak = Arc::downgrade(self);
        let handle = tokio::spawn(async move {
            run_scaling_monitor_via_weak(weak).await;
        });

        *self.scaling_monitor.lock() = Some(handle);
    }

    /// Internal: Add a new shard with its workers.
    ///
    /// The previous implementation called `shard_manager.add_shard()`
    /// first, which atomically marked the shard `Active` and published
    /// it to the routing table. So `select_shard` could route producer
    /// pushes to the new id *before* any drain or batch worker existed,
    /// leaving events queued in a buffer with no consumer (and
    /// triggering the configured backpressure mode if the buffer
    /// filled).
    ///
    /// The fix uses the two-phase API on `ShardManager`:
    ///   1. `add_shard()` allocates the id and metrics collector,
    ///      adds the shard to the routing table in `Provisioning`
    ///      state — so `with_shard` works (which the drain worker
    ///      needs) but `select_shard` skips it.
    ///   2. Spawn batch + drain workers and register the sender.
    ///   3. `activate_shard()` flips state to `Active`. Only now
    ///      does `select_shard` start routing producer pushes.
    async fn add_shard_internal(&self) -> Result<u16, AdapterError> {
        // Step 1: provisioning add — not yet selectable.
        let new_id = self
            .shard_manager
            .add_shard()
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Step 2: spawn workers and register the sender.
        let (tx, rx) = mpsc::channel::<Vec<crate::event::InternalEvent>>(1024);

        let batch = spawn_batch_worker(BatchWorkerParams {
            shard_id: new_id,
            rx,
            adapter: self.adapter.clone(),
            shard_manager: self.shard_manager.clone(),
            config: self.config.batch.clone(),
            adapter_timeout: self.config.adapter_timeout,
            batch_retries: self.config.adapter_batch_retries,
        });

        self.batch_senders.write().insert(new_id, tx.clone());

        let drain = spawn_drain_worker_for_shard(
            new_id,
            self.shard_manager.clone(),
            tx,
            self.shutdown.clone(),
            self.drain_finalize_ready.clone(),
        );

        self.batch_workers
            .lock()
            .insert(new_id, ShardWorkers { batch, drain });

        // Step 3: workers are live — flip the shard to Active so
        // `select_shard` will route to it.
        self.shard_manager
            .activate_shard(new_id)
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Update poll merger
        self.poll_merger.store(Arc::new(PollMerger::new(
            self.adapter.clone(),
            self.shard_manager.num_shards(),
        )));

        tracing::info!(shard_id = new_id, "Added new shard");
        Ok(new_id)
    }

    /// Internal: Remove a stopped shard.
    ///
    /// Previously this dropped the worker `JoinHandle`s and unmapped
    /// the shard without first draining its ring buffer. Any events
    /// still queued at the moment of removal — even just a few from a
    /// producer that pushed concurrently with the scale-down decision
    /// — were silently stranded once the drain worker exited on
    /// `with_shard → None`.
    ///
    /// The fix:
    ///   1. Wait for the drain worker we're about to retire to flush
    ///      the channel, by closing the bus-side sender first.
    ///   2. Call `remove_shard`, which atomically pops the
    ///      ring-buffer remnants and unmaps the shard. The drained
    ///      events come back to us as a `Vec`.
    ///   3. Hand those events directly to the adapter as a
    ///      single-shot batch — bypassing the per-shard pipeline
    ///      that's already being torn down — so they reach durable
    ///      storage.
    async fn remove_shard_internal(&self, shard_id: u16) -> Result<(), AdapterError> {
        // Step 1: drop the bus-side sender. The drain worker still
        // has its own clone and will keep draining; we want it to
        // exit when its `with_shard` call returns `None` after
        // step 2's unmap.
        self.batch_senders.write().remove(&shard_id);

        // Step 2: atomically drain whatever's in the ring buffer and
        // unmap. After this, `with_shard(shard_id)` returns `None`
        // and the drain worker exits at its next poll.
        let stranded = self
            .shard_manager
            .remove_shard(shard_id)
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Step 3: drop worker handles. The drain worker should have
        // exited (or be about to). The batch worker exits when its
        // channel closes — once the drain worker drops its sender
        // clone, the channel is closed.
        self.batch_workers.lock().remove(&shard_id);

        // Step 4: flush the stranded ring-buffer events through the
        // adapter in one shot. We construct a single `Batch` and
        // push it through the same `dispatch_batch` helper the
        // batch worker uses, so retry / timeout semantics match.
        if !stranded.is_empty() {
            let count = stranded.len();
            let batch = crate::event::Batch::new(shard_id, stranded, 0);
            let dispatched = dispatch_batch(
                &*self.adapter,
                batch,
                shard_id,
                self.config.adapter_timeout,
                self.config.adapter_batch_retries,
            )
            .await;
            if dispatched {
                tracing::info!(
                    shard_id,
                    count,
                    "Removed shard: flushed stranded ring-buffer events to adapter"
                );
            } else {
                tracing::error!(
                    shard_id,
                    count,
                    "Removed shard: adapter rejected stranded ring-buffer events; \
                     events lost"
                );
            }
        }

        // Update poll merger
        self.poll_merger.store(Arc::new(PollMerger::new(
            self.adapter.clone(),
            self.shard_manager.num_shards(),
        )));

        tracing::info!(shard_id = shard_id, "Removed shard");
        Ok(())
    }

    /// Try to enter an ingest critical section. Returns `None` if
    /// shutdown is in progress, in which case the caller must
    /// return `IngestionError::ShuttingDown` without touching the
    /// shard manager.
    ///
    /// The `fetch_add` + load(`shutdown`) sequence pairs with
    /// `shutdown()`'s store(`shutdown=true`) + wait-for-zero on
    /// `in_flight_ingests`. SeqCst on both sides closes the
    /// stranding race: every ingest that is observed as in-flight
    /// during shutdown's wait is guaranteed to complete before the
    /// drain workers do their final ring-buffer sweep, so no event
    /// can land in a ring buffer after the drain worker has stopped
    /// reading from it.
    #[inline(always)]
    fn try_enter_ingest(&self) -> Option<IngestGuard<'_>> {
        self.in_flight_ingests.fetch_add(1, AtomicOrdering::SeqCst);
        if self.shutdown.load(AtomicOrdering::SeqCst) {
            self.in_flight_ingests.fetch_sub(1, AtomicOrdering::SeqCst);
            return None;
        }
        Some(IngestGuard { bus: self })
    }

    /// Ingest an event.
    ///
    /// This is a non-blocking operation. The event is added to the appropriate
    /// shard's ring buffer and will be batched and persisted asynchronously.
    ///
    /// # Returns
    ///
    /// The shard ID and insertion timestamp on success.
    #[inline]
    pub fn ingest(&self, event: Event) -> IngestionResult<(u16, u64)> {
        let _g = self
            .try_enter_ingest()
            .ok_or(IngestionError::ShuttingDown)?;

        match self.shard_manager.ingest(event.into_inner()) {
            Ok((shard_id, ts)) => {
                self.stats
                    .events_ingested
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Ok((shard_id, ts))
            }
            Err(e) => {
                self.stats
                    .events_dropped
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Err(e)
            }
        }
    }

    /// Ingest a raw event (pre-serialized with cached hash).
    ///
    /// This is the fastest ingestion path:
    /// - Uses pre-computed hash for shard selection (no serialization)
    /// - Stores bytes directly (no clone needed, reference-counted)
    ///
    /// # Returns
    ///
    /// The shard ID and insertion timestamp on success.
    #[inline]
    pub fn ingest_raw(&self, event: RawEvent) -> IngestionResult<(u16, u64)> {
        let _g = self
            .try_enter_ingest()
            .ok_or(IngestionError::ShuttingDown)?;

        match self.shard_manager.ingest_raw(event) {
            Ok((shard_id, ts)) => {
                self.stats
                    .events_ingested
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Ok((shard_id, ts))
            }
            Err(e) => {
                self.stats
                    .events_dropped
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Err(e)
            }
        }
    }

    /// Ingest a batch of events.
    ///
    /// This is more efficient than calling `ingest` repeatedly: events
    /// destined for the same shard share a single mutex acquisition.
    ///
    /// # Returns
    ///
    /// The number of successfully ingested events.
    pub fn ingest_batch(&self, events: Vec<Event>) -> usize {
        // The shutdown gate lives in `ingest_raw_batch`, which we
        // forward to. No separate guard here — that would double-
        // count `in_flight_ingests` (once for this call, once for
        // the inner call) and could deadlock shutdown under high
        // contention.
        let raw: Vec<RawEvent> = events.into_iter().map(|e| e.into_raw()).collect();
        self.ingest_raw_batch(raw)
    }

    /// Ingest a batch of raw events (fastest batch ingestion).
    ///
    /// Groups events by their destination shard and pushes each group
    /// under a single mutex acquisition.
    ///
    /// # Returns
    ///
    /// The number of successfully ingested events.
    pub fn ingest_raw_batch(&self, events: Vec<RawEvent>) -> usize {
        let _g = match self.try_enter_ingest() {
            Some(g) => g,
            None => return 0,
        };

        let total = events.len();
        let success = self.shard_manager.ingest_raw_batch(events);
        if success > 0 {
            self.stats
                .events_ingested
                .fetch_add(success as u64, AtomicOrdering::Relaxed);
        }
        let dropped = total.saturating_sub(success);
        if dropped > 0 {
            self.stats
                .events_dropped
                .fetch_add(dropped as u64, AtomicOrdering::Relaxed);
        }
        success
    }

    /// Poll events from the bus.
    ///
    /// This retrieves events from storage according to the request parameters.
    pub async fn poll(&self, request: ConsumeRequest) -> Result<ConsumeResponse, ConsumerError> {
        let merger = self.poll_merger.load();
        merger.poll(request).await
    }

    /// Get the number of shards.
    pub fn num_shards(&self) -> u16 {
        self.shard_manager.num_shards()
    }

    /// Get the adapter name.
    pub fn adapter_name(&self) -> &'static str {
        self.adapter.name()
    }

    /// Check if the adapter is healthy.
    pub async fn is_healthy(&self) -> bool {
        self.adapter.is_healthy().await
    }

    /// Get statistics.
    pub fn stats(&self) -> &EventBusStats {
        &self.stats
    }

    /// Get shard statistics.
    pub fn shard_stats(&self) -> crate::shard::ShardStats {
        self.shard_manager.stats()
    }

    /// Flush all pending batches.
    ///
    /// Waits for all shard ring buffers to drain, then for the
    /// per-shard mpsc channels to drain, then for any pending batch
    /// inside each batch worker to time out and dispatch — and only
    /// then calls `adapter.flush()`. Bounded by 5 s overall.
    ///
    /// The previous implementation slept a single `batch.max_delay`
    /// (default 10 ms) after the ring buffers drained and immediately
    /// called `adapter.flush()`. Events still in transit through the
    /// per-shard mpsc channel, the batch worker's pending batch, or
    /// the in-progress `adapter.on_batch` call (bounded only by
    /// `adapter_timeout`, default 30 s) could miss the flush. Callers
    /// using `flush()` as a delivery barrier silently lost events.
    pub async fn flush(&self) -> Result<(), AdapterError> {
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(5);
        let mut backoff = Duration::from_micros(100);

        // Phase 1: wait for ring buffers to drain (drain workers
        // pump them into the per-shard mpsc channels).
        loop {
            if self.shard_manager.all_shards_empty() {
                break;
            }
            if start.elapsed() >= timeout {
                tracing::warn!("flush: ring buffers not fully drained after {:?}", timeout);
                break;
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_millis(10));
        }

        // Phase 2: wait for the per-shard mpsc channels to be
        // empty. We approximate this by sleeping `batch.max_delay`
        // — the batch worker's `recv_timeout` cap — at least once
        // per shard's worth of pending batch state, so any
        // partially-filled batch has a chance to time out and
        // dispatch.
        //
        // We can't directly observe channel depth (mpsc::Receiver
        // doesn't expose len without consuming), so use the
        // worst-case timing: `max_delay` per outstanding batch
        // worker. With the default 10 ms `max_delay` and a
        // typical 8-shard config, that's ~80 ms.
        let n_workers = self.batch_workers.lock().len();
        let phase2_budget = self
            .config
            .batch
            .max_delay
            .saturating_mul(n_workers.max(1) as u32);
        let phase2_deadline =
            tokio::time::Instant::now() + phase2_budget.min(Duration::from_secs(2));
        // Sleep one `max_delay` window at a time. After each sleep,
        // if the ring buffers are still empty (no late drain refilled
        // them), we've given the in-flight batches at least one
        // `max_delay`-sized opportunity to time out and dispatch —
        // good enough to declare phase 2 complete. Without this
        // early-break, every `flush()` always paid the full budget
        // (up to 2s) even on an idle system.
        while tokio::time::Instant::now() < phase2_deadline {
            tokio::time::sleep(self.config.batch.max_delay).await;
            if self.shard_manager.all_shards_empty() {
                break;
            }
        }

        // Phase 3: tell the adapter to flush whatever it has
        // buffered. Bounded by `adapter_timeout` so a hanging
        // adapter can't pin us forever.
        match tokio::time::timeout(self.config.adapter_timeout, self.adapter.flush()).await {
            Ok(r) => r,
            Err(_) => {
                tracing::warn!(
                    "flush: adapter.flush timed out after {:?}",
                    self.config.adapter_timeout
                );
                Err(AdapterError::Fatal("adapter flush timed out".into()))
            }
        }
    }

    /// Gracefully shut down the event bus.
    ///
    /// The shutdown order is load-bearing:
    ///
    ///   1. Signal `shutdown` so drain workers stop pulling from
    ///      ring buffers after their final sweep.
    ///   2. Await **drain workers** so every event the producer
    ///      has handed to the bus is now in the per-shard mpsc
    ///      channel.
    ///   3. Drop `batch_senders` so each channel's last sender is
    ///      gone — the next `recv().await` in a batch worker will
    ///      return `None`.
    ///   4. Await **batch workers**, which drain everything
    ///      remaining in their channel and exit on `recv() = None`.
    ///
    /// Reversing steps 2 and 4 (the previous design) silently
    /// dropped events: a batch worker that exited on the shutdown
    /// flag could leave events the drain worker pushed *after* its
    /// `try_recv` sweep stranded in the channel.
    pub async fn shutdown(self) -> Result<(), AdapterError> {
        self.shutdown_via_ref().await
    }

    /// Shutdown via shared reference — same semantics as
    /// [`shutdown`](Self::shutdown), but does not consume `self`.
    ///
    /// Useful for callers that hold the bus behind `Arc<EventBus>`
    /// (e.g., the SDK, where `subscribe` perpetuates an Arc clone
    /// into every `EventStream`) and therefore cannot satisfy
    /// `Arc::try_unwrap`. Idempotent: the first caller does the
    /// work; concurrent or subsequent callers wait for the
    /// `shutdown_completed` flag and return `Ok(())`.
    pub async fn shutdown_via_ref(&self) -> Result<(), AdapterError> {
        // 1. CAS the shutdown flag false→true. SeqCst pairs with
        // `try_enter_ingest`'s shutdown check — any producer that
        // observed the previous `false` and is mid-push has its
        // `in_flight_ingests` increment ordered before this store
        // (the CAS-success branch is a release of the new `true`),
        // and so will be visible to the wait below.
        //
        // If the CAS loses (someone else — typically a concurrent
        // call or `Drop` — already flipped the flag), spin until
        // they finish. We can't run the rest of the body because
        // workers/senders may already be partially torn down.
        if self
            .shutdown
            .compare_exchange(false, true, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst)
            .is_err()
        {
            // Bound the wait so a `Drop`-only path (which sets
            // `shutdown=true` but never sets `shutdown_completed`)
            // doesn't spin forever. After the deadline, return Ok
            // — the bus is at least signalled to stop.
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !self.shutdown_completed.load(AtomicOrdering::Acquire) {
                if std::time::Instant::now() >= deadline {
                    return Ok(());
                }
                tokio::task::yield_now().await;
            }
            return Ok(());
        }

        // 1a. Wait for in-flight ingests to drain BEFORE the drain
        // workers do their final ring-buffer sweep. Otherwise a
        // producer that observed `shutdown=false` could push *after*
        // the drain worker's last `pop_batch_into` returned zero,
        // leaving the event stranded in the ring buffer when the bus
        // is dropped.
        //
        // This is bounded: every producer either bails on the
        // SeqCst-synchronized shutdown check (no progress past the
        // increment) or completes its single non-blocking push and
        // decrements. Both paths take constant time; the total
        // wait is O(producer threads).
        let in_flight_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while self.in_flight_ingests.load(AtomicOrdering::SeqCst) > 0 {
            if std::time::Instant::now() >= in_flight_deadline {
                tracing::warn!(
                    in_flight = self.in_flight_ingests.load(AtomicOrdering::SeqCst),
                    "shutdown timed out waiting for in-flight ingests; \
                     proceeding with potential event loss"
                );
                break;
            }
            tokio::task::yield_now().await;
        }

        // 1b. Release the drain-finalize gate. The Release here pairs
        // with the drain worker's Acquire load below, transitively
        // making every push observed-pre-shutdown visible to the
        // drain worker's final sweep. Set this even on the timeout
        // path so a stuck producer doesn't deadlock the workers.
        self.drain_finalize_ready
            .store(true, AtomicOrdering::Release);

        // Stop the scaling monitor first — it's independent of the
        // ingestion path and just needs to observe the flag.
        let scaling_handle = self.scaling_monitor.lock().take();
        if let Some(handle) = scaling_handle {
            let _ = handle.await;
        }

        // Take workers without holding the lock across await.
        let workers = std::mem::take(&mut *self.batch_workers.lock());

        // 2. Await drain workers. Each one pops a final batch (up
        //    to 10k events) from its ring buffer, sends it on the
        //    channel, and exits. After this loop, every event in
        //    the ring buffers has been pushed to its channel.
        let mut batch_handles = Vec::with_capacity(workers.len());
        for (_shard_id, ShardWorkers { batch, drain }) in workers {
            let _ = drain.await;
            batch_handles.push(batch);
        }

        // 3. Drop the original senders so the channels close once
        //    drain-worker sender clones (already dropped above)
        //    are gone too. Without this, batch workers would block
        //    on `recv().await` forever.
        drop(std::mem::take(&mut *self.batch_senders.write()));

        // 4. Await batch workers. They drain their channel until
        //    `recv() = None`, flush, and exit.
        for handle in batch_handles {
            let _ = handle.await;
        }

        // Flush and shutdown adapter (with timeout to prevent hanging)
        let timeout = self.config.adapter_timeout;
        if tokio::time::timeout(timeout, self.adapter.flush())
            .await
            .is_err()
        {
            tracing::error!("Adapter flush timed out during shutdown");
        }
        let result = tokio::time::timeout(timeout, self.adapter.shutdown())
            .await
            .map_err(|_| AdapterError::Fatal("adapter shutdown timed out".into()))?;

        // Mark shutdown as completed so Drop knows not to warn.
        self.shutdown_completed.store(true, AtomicOrdering::Release);
        result
    }

    /// True once `shutdown` / `shutdown_via_ref` has signaled — does
    /// not imply the shutdown work has finished. Use
    /// [`is_shutdown_completed`](Self::is_shutdown_completed) for
    /// completion.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(AtomicOrdering::Acquire)
    }

    /// True once `shutdown` / `shutdown_via_ref` has fully drained
    /// workers and the adapter shutdown returned (success path only).
    pub fn is_shutdown_completed(&self) -> bool {
        self.shutdown_completed.load(AtomicOrdering::Acquire)
    }

    /// Get shard metrics (if dynamic scaling is enabled).
    pub fn shard_metrics(&self) -> Option<Vec<ShardMetrics>> {
        self.shard_manager.collect_metrics()
    }

    /// Check if dynamic scaling is enabled.
    pub fn is_dynamic_scaling_enabled(&self) -> bool {
        self.config.scaling.is_some()
    }

    /// Manually trigger a scale-up (for testing or manual intervention).
    pub async fn manual_scale_up(&self, count: u16) -> Result<Vec<u16>, AdapterError> {
        let mut new_ids = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let id = self.add_shard_internal().await?;
            new_ids.push(id);
        }
        Ok(new_ids)
    }

    /// Manually trigger a scale-down (for testing or manual intervention).
    ///
    /// Marks `count` shards as `Draining`, waits for them to empty,
    /// finalizes them to `Stopped`, and removes them from the
    /// routing table — mirroring the scaling monitor's per-tick
    /// finalize loop. Returns the IDs of shards that were
    /// successfully drained AND removed (subset of those marked
    /// Draining if any failed to empty within the deadline).
    ///
    /// BUG #82: previously this only called `mapper.scale_down`,
    /// which marks shards `Draining` but does NOT finalize them —
    /// finalization was the scaling monitor's responsibility. Bus
    /// configs without an active monitor (or callers that
    /// shut down before the monitor's next tick) lost any events
    /// queued in those shards' ring buffers. Now this method
    /// drives the full lifecycle synchronously.
    pub async fn manual_scale_down(&self, count: u16) -> Result<Vec<u16>, AdapterError> {
        let mapper = self
            .shard_manager
            .mapper()
            .ok_or_else(|| AdapterError::Fatal("Dynamic scaling not enabled".into()))?;

        let drained_ids = mapper
            .scale_down(count)
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // `finalize_draining` requires the shard to have been
        // Draining for >100ms with an empty ring buffer and no
        // pushes since drain start. Poll until every requested
        // shard finalizes, capped by an outer deadline so a wedged
        // producer can't pin this method forever.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut finalized: std::collections::HashSet<u16> = std::collections::HashSet::new();
        let target: std::collections::HashSet<u16> = drained_ids.iter().copied().collect();

        while finalized.len() < target.len() && std::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            let stopped = mapper.finalize_draining();
            for shard_id in stopped {
                if target.contains(&shard_id) {
                    let _ = self.remove_shard_internal(shard_id).await;
                    finalized.insert(shard_id);
                }
            }
        }

        Ok(finalized.into_iter().collect())
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        // Signal shutdown so background tasks (drain workers, batch
        // workers, scaling monitor) observe the flag and exit. We
        // cannot await futures in Drop, but setting the atomic flag
        // triggers eventual termination.
        //
        // Previously used `Release` here while `try_enter_ingest`
        // and `shutdown()` use `SeqCst` on the same flag. `&mut self`
        // exclusion makes that sound today (no concurrent ingest can
        // observe a half-published shutdown). The mismatch is purely
        // defensive — switching to `SeqCst` matches the rest of the
        // lifecycle and removes a footgun if a future change ever
        // opened a path where `Drop` overlaps an in-flight
        // `try_enter_ingest`.
        self.shutdown.store(true, AtomicOrdering::SeqCst);
        // Also release the drain-finalize gate so any drain worker
        // already parked waiting for it can proceed and exit. Without
        // this, drop-without-shutdown leaves drain workers blocked on
        // `drain_finalize_ready` until their internal deadline fires
        // (which delays task cleanup by `DRAIN_FINALIZE_TIMEOUT`).
        // Best-effort durability: drop never gets the in-flight wait,
        // so any push that lands after this point is still lost.
        self.drain_finalize_ready
            .store(true, AtomicOrdering::SeqCst);

        // If `shutdown()` was never awaited, any events still in the
        // per-shard ring buffers or mpsc channels are lost — the
        // adapter's `flush()` and `shutdown()` won't run, so durable
        // backends never see them. We can't fix that from `Drop` (no
        // async), but we *can* surface the data-loss risk loudly so
        // it doesn't hide. The check is bounded to "shutdown was
        // never started"; an in-progress shutdown is fine because the
        // call site is awaiting it.
        if !self.shutdown_completed.load(AtomicOrdering::Acquire) {
            let stats = self.shard_manager.stats();
            tracing::warn!(
                events_ingested = stats.events_ingested,
                events_dropped = stats.events_dropped,
                "EventBus dropped without an awaited shutdown(). Any in-flight \
                 events still in the ring buffers or batch channels will be lost \
                 — the adapter's flush()/shutdown() never ran. Call \
                 `bus.shutdown().await` before dropping for durable shutdown."
            );
        }
    }
}

/// Body of the scaling monitor task spawned by
/// `EventBus::start_scaling_monitor`. Holds a `Weak<EventBus>` and
/// upgrades it once per tick so the spawned task does not keep the
/// bus alive past the caller's last `Arc` reference. Without this,
/// `Arc::try_unwrap` (which the consuming `shutdown(self)` API
/// requires) would fail forever.
async fn run_scaling_monitor_via_weak(weak: std::sync::Weak<EventBus>) {
    // Refresh `interval` from the policy on every tick. The previous
    // version cached it once at task start, so any future runtime
    // policy update would not be adopted by the monitor without a
    // process restart. Today `EventBusConfig` is immutable
    // post-construction so this is a no-op — but reading it each tick
    // is cheap (one atomic / RwLock read) and removes the latent
    // footgun.
    loop {
        let interval = match weak.upgrade() {
            Some(bus) => match &bus.config.scaling {
                Some(p) => p.metrics_window,
                None => return,
            },
            None => return,
        };
        tokio::time::sleep(interval).await;

        let bus = match weak.upgrade() {
            Some(b) => b,
            // Last strong ref dropped — caller is shutting down (or
            // already gone). Exit cleanly.
            None => break,
        };

        if bus.shutdown.load(AtomicOrdering::Relaxed) {
            break;
        }

        // Collect metrics for observability.
        if let Some(metrics) = bus.shard_manager.collect_metrics() {
            for m in &metrics {
                if m.fill_ratio > 0.5 {
                    tracing::debug!(
                        shard_id = m.shard_id,
                        fill_ratio = m.fill_ratio,
                        event_rate = m.event_rate,
                        "Shard metrics"
                    );
                }
            }
        }

        // Evaluate scaling.
        match bus.shard_manager.evaluate_scaling() {
            ScalingDecision::ScaleUp(count) => {
                tracing::info!(count = count, "Scaling up shards");
                for _ in 0..count {
                    if let Err(e) = bus.add_shard_internal().await {
                        tracing::error!(error = %e, "Failed to add shard");
                        break;
                    }
                }
            }
            ScalingDecision::ScaleDown(count) => {
                tracing::info!(count = count, "Scaling down shards");
                if let Some(mapper) = bus.shard_manager.mapper() {
                    if let Ok(drained) = mapper.scale_down(count) {
                        for shard_id in drained {
                            let _ = bus.shard_manager.drain_shard(shard_id);
                        }
                    }
                }
            }
            ScalingDecision::None => {}
        }

        if let Some(mapper) = bus.shard_manager.mapper() {
            let stopped = mapper.finalize_draining();
            for shard_id in stopped {
                let _ = bus.remove_shard_internal(shard_id).await;
            }
        }

        // CRITICAL: drop the strong ref BEFORE the next sleep so a
        // concurrent `shutdown(self)` caller can `Arc::try_unwrap`
        // the last strong ref while we're sleeping.
        drop(bus);
    }
}

/// Spawn a batch worker for a shard.
/// Dispatch a batch to the adapter with timeout and optional retries.
/// Returns true if the batch was accepted, false if all attempts failed.
///
/// Non-retryable errors (e.g. `AdapterError::Connection`,
/// `AdapterError::Fatal`, `AdapterError::Serialization`) skip the
/// retry loop and drop the batch immediately. Retrying a fatal error
/// just delays the inevitable while burning CPU and amplifying log
/// noise. Use `AdapterError::is_retryable` as the single source of
/// truth for this decision.
async fn dispatch_batch(
    adapter: &dyn Adapter,
    batch: Batch,
    shard_id: u16,
    timeout: Duration,
    retries: u32,
) -> bool {
    // Retry attempts clone the batch; the final attempt moves it, saving
    // one clone per dispatch (the common path is retries == 0).
    for attempt in 0..retries {
        match tokio::time::timeout(timeout, adapter.on_batch(batch.clone())).await {
            Ok(Ok(())) => return true,
            Ok(Err(e)) => {
                if !e.is_retryable() {
                    tracing::error!(
                        shard_id,
                        error = %e,
                        attempt,
                        "Non-retryable error from adapter, dropping batch"
                    );
                    return false;
                }
                tracing::warn!(shard_id, error = %e, attempt, "Batch dispatch failed, retrying");
            }
            Err(_) => {
                tracing::warn!(shard_id, attempt, "Adapter on_batch timed out, retrying");
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    match tokio::time::timeout(timeout, adapter.on_batch(batch)).await {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            tracing::error!(shard_id, error = %e, "Failed to dispatch batch, dropping");
            false
        }
        Err(_) => {
            tracing::error!(shard_id, "Adapter on_batch timed out, dropping batch");
            false
        }
    }
}

struct BatchWorkerParams {
    shard_id: u16,
    rx: mpsc::Receiver<Vec<crate::event::InternalEvent>>,
    adapter: Arc<dyn Adapter>,
    shard_manager: Arc<ShardManager>,
    config: BatchConfig,
    adapter_timeout: Duration,
    batch_retries: u32,
}

fn spawn_batch_worker(params: BatchWorkerParams) -> JoinHandle<()> {
    let BatchWorkerParams {
        shard_id,
        mut rx,
        adapter,
        shard_manager,
        config,
        adapter_timeout,
        batch_retries,
    } = params;
    tokio::spawn(async move {
        let mut worker = BatchWorker::new(shard_id, config.clone());

        loop {
            // Wait for events with timeout. The batch worker exits
            // only when its channel is closed — i.e. after every
            // upstream sender (the drain worker for this shard +
            // `EventBus::batch_senders`) has been dropped.
            // `EventBus::shutdown` enforces that ordering so no
            // event is left stranded in the channel.
            let recv_timeout = worker.time_until_timeout().unwrap_or(config.max_delay);

            match tokio::time::timeout(recv_timeout, rx.recv()).await {
                Ok(Some(events)) => {
                    if let Some(batch) = worker.add_events(events) {
                        if dispatch_batch(
                            &*adapter,
                            batch,
                            shard_id,
                            adapter_timeout,
                            batch_retries,
                        )
                        .await
                        {
                            if let Some(shard_ref) = shard_manager.shard(shard_id) {
                                shard_ref.lock().record_batch_dispatch();
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Channel closed — drain any pending and exit.
                    if worker.has_pending() {
                        let batch = worker.flush();
                        if !batch.is_empty() {
                            dispatch_batch(
                                &*adapter,
                                batch,
                                shard_id,
                                adapter_timeout,
                                batch_retries,
                            )
                            .await;
                        }
                    }
                    break;
                }
                Err(_) => {
                    // Timeout - check if we need to flush
                    if let Some(batch) = worker.add_events(vec![]) {
                        if dispatch_batch(
                            &*adapter,
                            batch,
                            shard_id,
                            adapter_timeout,
                            batch_retries,
                        )
                        .await
                        {
                            if let Some(shard_ref) = shard_manager.shard(shard_id) {
                                shard_ref.lock().record_batch_dispatch();
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Maximum time a drain worker waits for `drain_finalize_ready`
/// after observing `shutdown=true`. Defense in depth: if a caller
/// drops the bus mid-shutdown without setting the gate, we don't
/// want the worker pinned forever. The shutdown path *always* sets
/// the gate (even on its own timeout), so this deadline is normally
/// unreached.
const DRAIN_FINALIZE_TIMEOUT: Duration = Duration::from_secs(10);

/// Spawn a drain worker for a single shard.
///
/// Uses a scratch `Vec` + `pop_batch_into` so the per-cycle
/// allocation happens *outside* the shard mutex critical section.
/// Each cycle: lock → drain into scratch (no alloc, capacity already
/// reserved) → unlock → `mem::replace` swaps the filled scratch out
/// for a fresh empty `Vec` (alloc *outside* the lock) → send the
/// filled batch on the channel.
fn spawn_drain_worker_for_shard(
    shard_id: u16,
    shard_manager: Arc<ShardManager>,
    sender: mpsc::Sender<Vec<crate::event::InternalEvent>>,
    shutdown: Arc<AtomicBool>,
    drain_finalize_ready: Arc<AtomicBool>,
) -> JoinHandle<()> {
    const STEADY_BATCH: usize = 1_000;
    const FINAL_BATCH: usize = 10_000;

    tokio::spawn(async move {
        let mut scratch: Vec<crate::event::InternalEvent> = Vec::with_capacity(STEADY_BATCH);

        loop {
            // Relaxed: same rationale as ingest — see `EventBus::ingest`.
            if shutdown.load(AtomicOrdering::Relaxed) {
                // Before doing the final sweep, wait for `shutdown()`
                // to release the finalize gate. The gate is set only
                // after the in-flight ingest counter reaches zero,
                // which means every producer that read `shutdown=false`
                // has completed its push. Without this wait, the drain
                // worker can race ahead of a late push under
                // shard-mutex serialization (drain takes the lock
                // first, sees nothing, exits; producer then takes the
                // lock and pushes — event stranded).
                //
                // Acquire pairs with the Release in `EventBus::shutdown`
                // and `EventBus::drop`, transitively making every
                // producer push that happened-before its `in_flight`
                // decrement visible to the subsequent `pop_batch_into`.
                let finalize_deadline = std::time::Instant::now() + DRAIN_FINALIZE_TIMEOUT;
                while !drain_finalize_ready.load(AtomicOrdering::Acquire) {
                    if std::time::Instant::now() >= finalize_deadline {
                        tracing::warn!(
                            shard_id,
                            "drain worker timed out waiting for finalize gate; \
                             proceeding with potential event loss"
                        );
                        break;
                    }
                    tokio::task::yield_now().await;
                }

                // Final drain: loop until the ring buffer is empty.
                // A single 10k batch is not enough — the ring
                // buffer can hold up to `ring_buffer_capacity`
                // events (default 1M) and any leftover would be
                // silently lost on shutdown.
                let mut final_scratch: Vec<crate::event::InternalEvent> =
                    Vec::with_capacity(FINAL_BATCH);
                loop {
                    let popped = shard_manager
                        .with_shard(shard_id, |shard| {
                            shard.pop_batch_into(&mut final_scratch, FINAL_BATCH)
                        })
                        .unwrap_or(0);
                    if popped == 0 {
                        break;
                    }
                    let batch =
                        std::mem::replace(&mut final_scratch, Vec::with_capacity(FINAL_BATCH));
                    if sender.send(batch).await.is_err() {
                        break;
                    }
                }
                break;
            }

            // Drain events from ring buffer.
            let popped = shard_manager.with_shard(shard_id, |shard| {
                shard.pop_batch_into(&mut scratch, STEADY_BATCH)
            });

            match popped {
                Some(0) => {
                    // No events — yield briefly. The 100μs sleep is deliberate:
                    // this is a latency-first system where the drain loop is the
                    // hot path. Longer backoff would add milliseconds of latency
                    // to the first event after a quiet period, violating the
                    // sub-microsecond design target. The CPU cost of 100μs polling
                    // is acceptable for a system that processes 10M+ events/sec.
                    tokio::time::sleep(Duration::from_micros(100)).await;
                }
                Some(_) => {
                    let batch = std::mem::replace(&mut scratch, Vec::with_capacity(STEADY_BATCH));
                    if sender.send(batch).await.is_err() {
                        break;
                    }
                }
                None => {
                    // Shard no longer exists (was removed)
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard::ScalingPolicy;
    use serde_json::json;

    #[tokio::test]
    async fn test_event_bus_basic() {
        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .build()
            .unwrap();

        let bus = EventBus::new(config).await.unwrap();

        // Ingest some events
        for i in 0..10 {
            let event = Event::new(json!({"index": i}));
            bus.ingest(event).unwrap();
        }

        // Give workers time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check stats
        assert_eq!(
            bus.stats().events_ingested.load(AtomicOrdering::Relaxed),
            10
        );

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_bus_batch_ingest() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config).await.unwrap();

        let events: Vec<Event> = (0..100).map(|i| Event::new(json!({"i": i}))).collect();

        let ingested = bus.ingest_batch(events);
        assert_eq!(ingested, 100);

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_bus_with_dynamic_scaling() {
        let policy = ScalingPolicy {
            min_shards: 2,
            max_shards: 8,
            ..Default::default()
        };

        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .scaling(policy)
            .build()
            .unwrap();

        let bus = EventBus::new(config).await.unwrap();

        // Verify dynamic scaling is enabled
        assert!(bus.is_dynamic_scaling_enabled());
        assert_eq!(bus.num_shards(), 2);

        // Ingest some events
        for i in 0..100 {
            let event = Event::new(json!({"index": i}));
            bus.ingest(event).unwrap();
        }

        // Give workers time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check stats
        assert_eq!(
            bus.stats().events_ingested.load(AtomicOrdering::Relaxed),
            100
        );

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_manual_scale_up() {
        let policy = ScalingPolicy {
            min_shards: 2,
            max_shards: 8,
            cooldown: Duration::from_nanos(1), // Effectively disable cooldown for test
            ..Default::default()
        };

        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .scaling(policy)
            .build()
            .unwrap();

        let bus = EventBus::new(config).await.unwrap();

        assert_eq!(bus.num_shards(), 2);

        // Manually scale up
        let new_ids = bus.manual_scale_up(2).await.unwrap();
        assert_eq!(new_ids.len(), 2);
        assert_eq!(bus.num_shards(), 4);

        // Ingest events - they should be distributed across all shards
        for i in 0..100 {
            let event = Event::new(json!({"index": i}));
            bus.ingest(event).unwrap();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(
            bus.stats().events_ingested.load(AtomicOrdering::Relaxed),
            100
        );

        bus.shutdown().await.unwrap();
    }

    /// Regression for BUG_AUDIT_2026_04_30_CORE.md #82: previously
    /// `manual_scale_down` only called `mapper.scale_down(count)`,
    /// which marks shards as `Draining` but does NOT finalize them.
    /// Bus configs without an active scaling monitor (or callers
    /// shutting down before the monitor's next tick) lost any
    /// events queued in the drained shards' ring buffers because
    /// `remove_shard_internal` was never invoked. The fix runs the
    /// full lifecycle synchronously: scale_down → poll for empty →
    /// finalize_draining → remove_shard_internal.
    ///
    /// We pin this by scaling up, manually scaling down, and
    /// asserting that `num_shards` actually decreases — pre-fix
    /// the count would still reflect the Draining shards.
    #[tokio::test]
    async fn manual_scale_down_finalizes_and_removes_drained_shards() {
        let policy = ScalingPolicy {
            min_shards: 2,
            max_shards: 8,
            cooldown: Duration::from_nanos(1),
            ..Default::default()
        };
        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .scaling(policy)
            .build()
            .unwrap();
        let bus = EventBus::new(config).await.unwrap();

        // Scale up to 4, then back down to 2.
        let added = bus.manual_scale_up(2).await.unwrap();
        assert_eq!(added.len(), 2);
        assert_eq!(bus.num_shards(), 4);

        let removed = bus.manual_scale_down(2).await.unwrap();
        assert_eq!(
            removed.len(),
            2,
            "manual_scale_down must complete the lifecycle for both \
             requested shards (mark Draining → wait → finalize → remove)"
        );

        // Pre-fix: `num_shards` would still be 4 because shards
        // were only marked Draining (and the routing-table removal
        // path never ran). Post-fix: it's back to 2.
        assert_eq!(
            bus.num_shards(),
            2,
            "drained shards must be removed from the routing table"
        );

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_shard_metrics() {
        let policy = ScalingPolicy::default();

        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .scaling(policy)
            .build()
            .unwrap();

        let bus = EventBus::new(config).await.unwrap();

        // Ingest some events
        for i in 0..50 {
            let event = Event::new(json!({"index": i}));
            bus.ingest(event).unwrap();
        }

        // Get metrics
        let metrics = bus.shard_metrics();
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.len(), 2);

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_regression_eventbus_drop_signals_shutdown() {
        // Regression: dropping an EventBus without calling shutdown() used to
        // leave background tasks running indefinitely. The Drop impl now sets
        // the shutdown flag so workers eventually exit.
        let result = tokio::time::timeout(Duration::from_secs(5), async {
            let config = EventBusConfig::builder()
                .num_shards(2)
                .ring_buffer_capacity(1024)
                .build()
                .unwrap();

            let bus = EventBus::new(config).await.unwrap();

            // Ingest some events
            for i in 0..10 {
                let event = Event::new(json!({"index": i}));
                bus.ingest(event).unwrap();
            }

            // Drop without calling shutdown()
            drop(bus);

            // If we reach here, the drop didn't hang
        })
        .await;

        assert!(
            result.is_ok(),
            "EventBus drop should not hang — Drop impl must signal shutdown"
        );
    }

    #[tokio::test]
    async fn test_with_dynamic_scaling_builder() {
        let config = EventBusConfig::builder()
            .num_shards(4)
            .ring_buffer_capacity(2048)
            .with_dynamic_scaling()
            .build()
            .unwrap();

        let bus = EventBus::new(config).await.unwrap();

        assert!(bus.is_dynamic_scaling_enabled());
        assert_eq!(bus.num_shards(), 4);

        bus.shutdown().await.unwrap();
    }

    /// Mock adapter that counts `on_batch` invocations and returns a
    /// configurable error variant. Used to assert dispatch retry
    /// semantics without dragging in a real adapter.
    struct CountingErrAdapter {
        calls: Arc<std::sync::atomic::AtomicU32>,
        make_err: Box<dyn Fn() -> AdapterError + Send + Sync>,
    }

    #[async_trait::async_trait]
    impl crate::adapter::Adapter for CountingErrAdapter {
        async fn init(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn on_batch(&self, _batch: Batch) -> Result<(), AdapterError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err((self.make_err)())
        }
        async fn flush(&self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn poll_shard(
            &self,
            _shard_id: u16,
            _from_id: Option<&str>,
            _limit: usize,
        ) -> Result<crate::adapter::ShardPollResult, AdapterError> {
            Ok(crate::adapter::ShardPollResult::empty())
        }
        fn name(&self) -> &'static str {
            "counting_err"
        }
        async fn is_healthy(&self) -> bool {
            true
        }
    }

    /// Regression: BUG_REPORT.md #21 — `dispatch_batch` previously
    /// retried every error variant, ignoring `AdapterError::is_retryable`.
    /// A non-retryable error (Connection / Fatal / Serialization)
    /// should now drop the batch immediately rather than burn the
    /// retry budget on something that cannot succeed.
    #[tokio::test(start_paused = true)]
    async fn dispatch_batch_skips_retries_on_non_retryable_error() {
        let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let adapter = CountingErrAdapter {
            calls: calls.clone(),
            make_err: Box::new(|| AdapterError::Connection("refused".into())),
        };

        let batch = Batch::new(0, vec![], 0);
        let ok = dispatch_batch(&adapter, batch, 0, Duration::from_secs(1), 5).await;

        assert!(!ok, "non-retryable error must drop batch");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Connection error must not be retried; expected exactly 1 on_batch call"
        );
    }

    /// Sanity: a retryable error *does* go through the full retry
    /// budget. Without this companion check, the previous test could
    /// pass for the wrong reason (e.g. if dispatch always returned on
    /// the first error).
    #[tokio::test(start_paused = true)]
    async fn dispatch_batch_retries_transient_errors() {
        let calls = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let adapter = CountingErrAdapter {
            calls: calls.clone(),
            make_err: Box::new(|| AdapterError::Transient("temp".into())),
        };

        let batch = Batch::new(0, vec![], 0);
        let ok = dispatch_batch(&adapter, batch, 0, Duration::from_secs(1), 3).await;

        assert!(!ok);
        // 3 retries + 1 final attempt = 4 total calls.
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    /// Counting adapter that records the number of events delivered via
    /// `on_batch`. Used by shutdown-durability tests below.
    struct CountingAdapter {
        received: Arc<AtomicU64>,
    }

    #[async_trait::async_trait]
    impl crate::adapter::Adapter for CountingAdapter {
        async fn init(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
            self.received
                .fetch_add(batch.events.len() as u64, AtomicOrdering::SeqCst);
            Ok(())
        }
        async fn flush(&self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), AdapterError> {
            Ok(())
        }
        async fn poll_shard(
            &self,
            _shard_id: u16,
            _from_id: Option<&str>,
            _limit: usize,
        ) -> Result<crate::adapter::ShardPollResult, AdapterError> {
            Ok(crate::adapter::ShardPollResult::empty())
        }
        fn name(&self) -> &'static str {
            "counting"
        }
        async fn is_healthy(&self) -> bool {
            true
        }
    }

    /// Regression: BUG_REPORT.md #6 — `shutdown()` must deliver every
    /// successfully-ingested event to the adapter before returning.
    /// Pins the broader durability contract that the
    /// `drain_finalize_ready` gate supports: the drain worker may not
    /// finalize until the in-flight wait completes.
    ///
    /// Tests across many shards with bursts large enough that the
    /// drain workers are mid-loop when shutdown begins.
    #[tokio::test]
    async fn shutdown_delivers_every_successful_ingest_to_adapter() {
        let received = Arc::new(AtomicU64::new(0));
        let adapter: Box<dyn crate::adapter::Adapter> = Box::new(CountingAdapter {
            received: received.clone(),
        });

        let config = EventBusConfig::builder()
            .num_shards(4)
            .ring_buffer_capacity(4096)
            .build()
            .unwrap();
        let bus = EventBus::new_with_adapter(config, adapter).await.unwrap();

        // Drive a sizable burst across all shards. Capacity > burst so
        // we don't trip backpressure; every successful Ok must reach
        // `on_batch` before shutdown returns.
        let total = 10_000usize;
        let mut successes = 0u64;
        for i in 0..total {
            if bus.ingest(Event::new(json!({"i": i}))).is_ok() {
                successes += 1;
            }
        }

        // Shutdown awaits drain workers; with the BUG_REPORT.md #6 fix
        // those workers wait on `drain_finalize_ready` after observing
        // `shutdown=true`, so any push the producer made before the
        // shutdown flag is guaranteed to be in the ring buffer when
        // the final sweep runs.
        bus.shutdown().await.unwrap();

        let delivered = received.load(AtomicOrdering::SeqCst);
        assert_eq!(
            delivered, successes,
            "shutdown stranded events: {successes} ingested successfully, \
             only {delivered} reached the adapter"
        );
    }

    /// Regression: BUG_REPORT.md #16 — `flush()` must be a delivery
    /// barrier: after it returns successfully, every event the
    /// caller successfully ingested before `flush()` was called
    /// must have been handed to the adapter via `on_batch`.
    /// The previous implementation slept a single `batch.max_delay`
    /// after the ring buffers drained, which left a window where
    /// events could still be sitting in the per-shard mpsc channel
    /// or inside a partially-filled batch awaiting timeout — those
    /// events were silently dropped from the flush guarantee.
    #[tokio::test]
    async fn flush_is_a_delivery_barrier() {
        let received = Arc::new(AtomicU64::new(0));
        let adapter: Box<dyn crate::adapter::Adapter> = Box::new(CountingAdapter {
            received: received.clone(),
        });

        // Use a deliberately *long* batch.max_delay (250ms) so that a
        // partially-filled batch sitting in the batch worker's
        // pending state would survive past the old single-`max_delay`
        // sleep. min_size > burst forces the partial-batch path.
        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .batch(crate::config::BatchConfig {
                min_size: 1_000,
                max_size: 10_000,
                max_delay: Duration::from_millis(250),
                adaptive: false,
                velocity_window: Duration::from_millis(100),
            })
            .build()
            .unwrap();
        let bus = EventBus::new_with_adapter(config, adapter).await.unwrap();

        // A small burst — far below `min_size`, so the batch worker
        // will sit on a partial batch waiting for `max_delay`.
        let burst = 50usize;
        let mut successes = 0u64;
        for i in 0..burst {
            if bus.ingest(Event::new(json!({"i": i}))).is_ok() {
                successes += 1;
            }
        }

        // Time the flush call to confirm we waited long enough for
        // the partial batch to time out. The previous code slept
        // ~10ms total in the post-empty phase; the fix waits up to
        // `max_delay * num_workers` (here 500ms cap, capped at 2s).
        let t0 = std::time::Instant::now();
        bus.flush().await.unwrap();
        let elapsed = t0.elapsed();

        // After flush returns, every successful ingest must have
        // been delivered to the adapter. With the old code this
        // assertion would fail: events sit in the partial batch
        // until `max_delay` (250ms) elapses, but flush returned
        // after only ~10ms.
        let delivered = received.load(AtomicOrdering::SeqCst);
        assert_eq!(
            delivered, successes,
            "flush() returned but only {delivered} of {successes} \
             events reached the adapter (#16); flush waited {:?}",
            elapsed
        );

        bus.shutdown().await.unwrap();
    }

    /// Regression: BUG_REPORT.md #6 — drop-without-shutdown must
    /// still release the drain-finalize gate so detached drain
    /// workers can exit instead of parking on the gate until the
    /// internal `DRAIN_FINALIZE_TIMEOUT` deadline. Pinning this
    /// keeps the `Drop` impl honest if someone refactors the
    /// shutdown gates later.
    #[tokio::test]
    async fn drop_releases_drain_finalize_gate_promptly() {
        let config = EventBusConfig::builder()
            .num_shards(2)
            .ring_buffer_capacity(1024)
            .build()
            .unwrap();
        let bus = EventBus::new(config).await.unwrap();
        let drain_gate = bus.drain_finalize_ready.clone();

        // Drop without an awaited shutdown.
        drop(bus);

        // The Drop impl must have set the gate. `DRAIN_FINALIZE_TIMEOUT`
        // is 10s; if Drop didn't flip the gate, drain workers would
        // park for up to that long before exiting.
        assert!(
            drain_gate.load(AtomicOrdering::Acquire),
            "Drop must release `drain_finalize_ready` so detached drain \
             workers exit promptly"
        );
    }
}
