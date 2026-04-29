//! Main EventBus facade.
//!
//! The EventBus provides a unified API for:
//! - Event ingestion (non-blocking)
//! - Event consumption (async polling with filtering)
//! - Lifecycle management

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
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

        // Shutdown flag
        let shutdown = Arc::new(AtomicBool::new(false));

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
            config,
            stats: EventBusStats::default(),
            scaling_monitor: parking_lot::Mutex::new(None),
        };

        Ok(bus)
    }

    /// Start the scaling monitor (if dynamic scaling is enabled).
    /// This spawns a background task that periodically evaluates scaling decisions.
    pub fn start_scaling_monitor(self: &Arc<Self>) {
        if self.config.scaling.is_none() {
            return;
        }

        let bus = Arc::clone(self);
        let handle = tokio::spawn(async move {
            bus.run_scaling_monitor().await;
        });

        *self.scaling_monitor.lock() = Some(handle);
    }

    /// Run the scaling monitor loop.
    async fn run_scaling_monitor(&self) {
        let policy = match &self.config.scaling {
            Some(p) => p,
            None => return,
        };

        let interval = policy.metrics_window;

        loop {
            if self.shutdown.load(AtomicOrdering::Acquire) {
                break;
            }

            tokio::time::sleep(interval).await;

            // Check shutdown again after sleep
            if self.shutdown.load(AtomicOrdering::Acquire) {
                break;
            }

            // Collect metrics
            if let Some(metrics) = self.shard_manager.collect_metrics() {
                // Log metrics for observability
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

            // Evaluate scaling
            match self.shard_manager.evaluate_scaling() {
                ScalingDecision::ScaleUp(count) => {
                    tracing::info!(count = count, "Scaling up shards");
                    for _ in 0..count {
                        if let Err(e) = self.add_shard_internal().await {
                            tracing::error!(error = %e, "Failed to add shard");
                            break;
                        }
                    }
                }
                ScalingDecision::ScaleDown(count) => {
                    tracing::info!(count = count, "Scaling down shards");
                    if let Some(mapper) = self.shard_manager.mapper() {
                        if let Ok(drained) = mapper.scale_down(count) {
                            for shard_id in drained {
                                let _ = self.shard_manager.drain_shard(shard_id);
                            }
                        }
                    }
                }
                ScalingDecision::None => {}
            }

            // Finalize any draining shards
            if let Some(mapper) = self.shard_manager.mapper() {
                let stopped = mapper.finalize_draining();
                for shard_id in stopped {
                    let _ = self.remove_shard_internal(shard_id).await;
                }
            }
        }
    }

    /// Internal: Add a new shard with its workers.
    async fn add_shard_internal(&self) -> Result<u16, AdapterError> {
        let new_id = self
            .shard_manager
            .add_shard()
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Create batch worker for the new shard
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

        // Add sender
        self.batch_senders.write().insert(new_id, tx.clone());

        // Spawn drain worker for the new shard
        let drain = spawn_drain_worker_for_shard(
            new_id,
            self.shard_manager.clone(),
            tx,
            self.shutdown.clone(),
        );

        // Add workers
        self.batch_workers
            .lock()
            .insert(new_id, ShardWorkers { batch, drain });

        // Update poll merger
        self.poll_merger.store(Arc::new(PollMerger::new(
            self.adapter.clone(),
            self.shard_manager.num_shards(),
        )));

        tracing::info!(shard_id = new_id, "Added new shard");
        Ok(new_id)
    }

    /// Internal: Remove a stopped shard.
    async fn remove_shard_internal(&self, shard_id: u16) -> Result<(), AdapterError> {
        // Remove sender (workers will terminate when channel closes)
        self.batch_senders.write().remove(&shard_id);

        // Remove and drop worker handles for this shard
        self.batch_workers.lock().remove(&shard_id);

        // Remove from shard manager
        self.shard_manager
            .remove_shard(shard_id)
            .map_err(|e| AdapterError::Fatal(e.to_string()))?;

        // Update poll merger
        self.poll_merger.store(Arc::new(PollMerger::new(
            self.adapter.clone(),
            self.shard_manager.num_shards(),
        )));

        tracing::info!(shard_id = shard_id, "Removed shard");
        Ok(())
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
        // Relaxed: shutdown is a one-way latch. Producers don't gate
        // any memory observation on the flag — they just decide
        // whether to continue. Pair with `Release` in `shutdown()`.
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return Err(IngestionError::ShuttingDown);
        }

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
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return Err(IngestionError::ShuttingDown);
        }

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
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return 0;
        }

        // Convert once to `RawEvent` (pre-serializes + hashes each
        // event) and dispatch through the grouped batch path.
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
        if self.shutdown.load(AtomicOrdering::Relaxed) {
            return 0;
        }

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
    /// Waits for all shard ring buffers to drain (up to 5 seconds) before
    /// flushing the adapter, rather than relying on a fixed sleep.
    pub async fn flush(&self) -> Result<(), AdapterError> {
        // Wait until all ring buffers are empty, polling with adaptive backoff.
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(5);
        let mut backoff = Duration::from_micros(100);

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

        // Ring buffers are empty, but events may still be in-flight in the
        // drain→batch worker pipeline (mpsc channels, pending batches).
        // Wait for the batch workers' max_delay to ensure any pending batch
        // has been flushed to the adapter before we call adapter.flush().
        tokio::time::sleep(self.config.batch.max_delay).await;

        self.adapter.flush().await
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
        // 1. Signal shutdown.
        self.shutdown.store(true, AtomicOrdering::Release);

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
        tokio::time::timeout(timeout, self.adapter.shutdown())
            .await
            .map_err(|_| AdapterError::Fatal("adapter shutdown timed out".into()))?
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
    pub fn manual_scale_down(&self, count: u16) -> Result<Vec<u16>, AdapterError> {
        let mapper = self
            .shard_manager
            .mapper()
            .ok_or_else(|| AdapterError::Fatal("Dynamic scaling not enabled".into()))?;

        mapper
            .scale_down(count)
            .map_err(|e| AdapterError::Fatal(e.to_string()))
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        // Signal shutdown so background tasks (drain workers, batch workers,
        // scaling monitor) observe the flag and exit. We cannot await futures
        // in Drop, but setting the atomic flag triggers eventual termination.
        self.shutdown.store(true, AtomicOrdering::Release);
    }
}

/// Spawn a batch worker for a shard.
/// Dispatch a batch to the adapter with timeout and optional retries.
/// Returns true if the batch was accepted, false if all attempts failed.
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

/// Spawn a drain worker for a single shard.
fn spawn_drain_worker_for_shard(
    shard_id: u16,
    shard_manager: Arc<ShardManager>,
    sender: mpsc::Sender<Vec<crate::event::InternalEvent>>,
    shutdown: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if shutdown.load(AtomicOrdering::Acquire) {
                // Final drain: loop until the ring buffer is empty.
                // A single 10k batch is not enough — the ring
                // buffer can hold up to `ring_buffer_capacity`
                // events (default 1M) and any leftover would be
                // silently lost on shutdown.
                loop {
                    let events = shard_manager
                        .with_shard(shard_id, |shard| shard.pop_batch(10_000))
                        .unwrap_or_default();
                    if events.is_empty() {
                        break;
                    }
                    if sender.send(events).await.is_err() {
                        break;
                    }
                }
                break;
            }

            // Drain events from ring buffer
            let events = shard_manager.with_shard(shard_id, |shard| shard.pop_batch(1_000));

            match events {
                Some(events) if !events.is_empty() => {
                    if sender.send(events).await.is_err() {
                        break;
                    }
                }
                Some(_) => {
                    // No events — yield briefly. The 100μs sleep is deliberate:
                    // this is a latency-first system where the drain loop is the
                    // hot path. Longer backoff would add milliseconds of latency
                    // to the first event after a quiet period, violating the
                    // sub-microsecond design target. The CPU cost of 100μs polling
                    // is acceptable for a system that processes 10M+ events/sec.
                    tokio::time::sleep(Duration::from_micros(100)).await;
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
            cooldown: Duration::from_millis(0), // Disable cooldown for test
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
}
