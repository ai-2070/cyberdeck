//! Shard management for parallel event ingestion.
//!
//! The shard module provides:
//! - Lock-free ring buffers for high-throughput event queuing
//! - Per-shard timestamp generation (no cross-shard contention)
//! - Batch assembly with adaptive sizing
//! - Shard manager for coordinating multiple shards
//! - Dynamic shard scaling with weighted producer routing

mod batch;
mod mapper;
mod ring_buffer;

pub use batch::{AdaptiveBatcher, BatchWorker};
pub use mapper::{
    ScalingDecision, ScalingError, ShardMapper, ShardMetrics, ShardMetricsCollector, ShardState,
};
pub use ring_buffer::{BufferFullError, RingBuffer};

// Re-export ScalingPolicy from config for convenience
pub use crate::config::ScalingPolicy;

use bytes::Bytes;

use crate::config::BackpressureMode;
use crate::error::IngestionError;
use crate::event::{InternalEvent, RawEvent};
use crate::timestamp::TimestampGenerator;

use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Statistics for a single shard.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShardStats {
    /// Total events ingested.
    pub events_ingested: u64,
    /// Events dropped due to backpressure.
    pub events_dropped: u64,
    /// Batches dispatched to adapter.
    pub batches_dispatched: u64,
}

/// A single shard with its own ring buffer and timestamp generator.
pub struct Shard {
    /// Shard identifier.
    pub id: u16,
    /// Ring buffer for event queuing.
    ring_buffer: RingBuffer<InternalEvent>,
    /// Shard-local timestamp generator (no contention).
    timestamp_gen: TimestampGenerator,
    /// Shard statistics.
    stats: ShardStats,
    /// Optional metrics collector for dynamic scaling.
    metrics_collector: Option<Arc<ShardMetricsCollector>>,
    /// Ring buffer capacity (for metrics).
    capacity: usize,
}

impl Shard {
    /// Create a new shard.
    pub fn new(id: u16, capacity: usize) -> Self {
        Self {
            id,
            ring_buffer: RingBuffer::new(capacity),
            timestamp_gen: TimestampGenerator::new(),
            stats: ShardStats::default(),
            metrics_collector: None,
            capacity,
        }
    }

    /// Create a new shard with a metrics collector for dynamic scaling.
    pub fn with_metrics(id: u16, capacity: usize, metrics: Arc<ShardMetricsCollector>) -> Self {
        Self {
            id,
            ring_buffer: RingBuffer::new(capacity),
            timestamp_gen: TimestampGenerator::new(),
            stats: ShardStats::default(),
            metrics_collector: Some(metrics),
            capacity,
        }
    }

    /// Set the metrics collector.
    pub fn set_metrics_collector(&mut self, metrics: Arc<ShardMetricsCollector>) {
        self.metrics_collector = Some(metrics);
    }

    /// Try to push a raw event (pre-serialized bytes) into the shard's ring buffer.
    /// Returns the assigned insertion timestamp on success.
    ///
    /// This is the fastest ingestion path - no serialization or hashing needed.
    #[inline]
    pub fn try_push_raw(&mut self, raw: Bytes) -> Result<u64, IngestionError> {
        let ts = self.timestamp_gen.next();
        let event = InternalEvent::new(raw, ts, self.id);

        match self.ring_buffer.try_push(event) {
            Ok(()) => {
                self.stats.events_ingested += 1;
                Ok(ts)
            }
            Err(_) => {
                self.stats.events_dropped += 1;
                Err(IngestionError::Backpressure)
            }
        }
    }

    /// Try to push a JSON value into the shard's ring buffer.
    /// Returns the assigned insertion timestamp on success.
    ///
    /// This serializes the value once before storing.
    #[inline]
    pub fn try_push(&mut self, raw: JsonValue) -> Result<u64, IngestionError> {
        let ts = self.timestamp_gen.next();
        let event = InternalEvent::from_value(raw, ts, self.id);

        match self.ring_buffer.try_push(event) {
            Ok(()) => {
                self.stats.events_ingested += 1;
                Ok(ts)
            }
            Err(_) => {
                self.stats.events_dropped += 1;
                Err(IngestionError::Backpressure)
            }
        }
    }

    /// Pop a batch of events from the ring buffer.
    #[inline]
    pub fn pop_batch(&mut self, max: usize) -> Vec<InternalEvent> {
        self.ring_buffer.pop_batch(max)
    }

    /// Try to pop a single event from the ring buffer.
    #[inline]
    pub fn try_pop(&mut self) -> Option<InternalEvent> {
        self.ring_buffer.try_pop()
    }

    /// Get the current buffer length.
    #[inline]
    pub fn len(&self) -> usize {
        self.ring_buffer.len()
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ring_buffer.is_empty()
    }

    /// Check if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.ring_buffer.is_full()
    }

    /// Get the fill ratio (0.0 - 1.0).
    #[inline]
    pub fn fill_ratio(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.ring_buffer.len() as f64 / self.capacity as f64
        }
    }

    /// Get the ring buffer capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get shard statistics.
    pub fn stats(&self) -> &ShardStats {
        &self.stats
    }

    /// Record a batch dispatch.
    pub fn record_batch_dispatch(&mut self) {
        self.stats.batches_dispatched += 1;
    }
}

/// Manager for multiple shards.
///
/// The ShardManager can operate in two modes:
/// 1. Static mode (default): Fixed number of shards, simple hash-based routing
/// 2. Dynamic mode: Shards can be added/removed based on load, weighted routing
pub struct ShardManager {
    /// All shards (indexed by position, not shard ID).
    shards: parking_lot::RwLock<Vec<parking_lot::Mutex<Shard>>>,
    /// Map from shard ID to index in shards vector.
    shard_index: parking_lot::RwLock<std::collections::HashMap<u16, usize>>,
    /// Current number of active shards.
    num_shards: std::sync::atomic::AtomicU16,
    /// Backpressure mode.
    backpressure_mode: BackpressureMode,
    /// Ring buffer capacity for new shards.
    ring_buffer_capacity: usize,
    /// Optional shard mapper for dynamic scaling.
    mapper: Option<Arc<ShardMapper>>,
}

impl ShardManager {
    /// Create a new shard manager (static mode).
    pub fn new(
        num_shards: u16,
        ring_buffer_capacity: usize,
        backpressure_mode: BackpressureMode,
    ) -> Self {
        let shards: Vec<_> = (0..num_shards)
            .map(|id| parking_lot::Mutex::new(Shard::new(id, ring_buffer_capacity)))
            .collect();

        let shard_index: std::collections::HashMap<u16, usize> =
            (0..num_shards).map(|id| (id, id as usize)).collect();

        Self {
            shards: parking_lot::RwLock::new(shards),
            shard_index: parking_lot::RwLock::new(shard_index),
            num_shards: std::sync::atomic::AtomicU16::new(num_shards),
            backpressure_mode,
            ring_buffer_capacity,
            mapper: None,
        }
    }

    /// Create a new shard manager with dynamic scaling enabled.
    pub fn with_mapper(
        num_shards: u16,
        ring_buffer_capacity: usize,
        backpressure_mode: BackpressureMode,
        policy: ScalingPolicy,
    ) -> Result<Self, ScalingError> {
        let mapper = Arc::new(ShardMapper::new(num_shards, ring_buffer_capacity, policy)?);

        let shards: Vec<_> = (0..num_shards)
            .map(|id| {
                let metrics = mapper.metrics_collector(id).ok_or_else(|| {
                    ScalingError::InvalidPolicy(format!("no metrics collector for shard {}", id))
                })?;
                Ok(parking_lot::Mutex::new(Shard::with_metrics(
                    id,
                    ring_buffer_capacity,
                    metrics,
                )))
            })
            .collect::<Result<Vec<_>, ScalingError>>()?;

        let shard_index: std::collections::HashMap<u16, usize> =
            (0..num_shards).map(|id| (id, id as usize)).collect();

        Ok(Self {
            shards: parking_lot::RwLock::new(shards),
            shard_index: parking_lot::RwLock::new(shard_index),
            num_shards: std::sync::atomic::AtomicU16::new(num_shards),
            backpressure_mode,
            ring_buffer_capacity,
            mapper: Some(mapper),
        })
    }

    /// Get the shard mapper (if dynamic scaling is enabled).
    pub fn mapper(&self) -> Option<&Arc<ShardMapper>> {
        self.mapper.as_ref()
    }

    /// Get the number of active shards.
    #[inline]
    pub fn num_shards(&self) -> u16 {
        self.num_shards.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get the backpressure mode.
    #[inline]
    pub fn backpressure_mode(&self) -> BackpressureMode {
        self.backpressure_mode
    }

    /// Select a shard for an event based on its content hash.
    /// Uses weighted selection if dynamic scaling is enabled.
    #[inline]
    pub fn select_shard(&self, event: &JsonValue) -> u16 {
        // Use xxhash for fast, deterministic hashing. `to_vec` avoids the
        // extra UTF-8 validation that `to_string` performs on the serialized
        // buffer, since we only need the bytes for hashing.
        let bytes = serde_json::to_vec(event).expect("Value serialization is infallible");
        let hash = xxhash_rust::xxh3::xxh3_64(&bytes);

        if let Some(ref mapper) = self.mapper {
            // Dynamic mode: use weighted selection
            mapper.select_shard(hash)
        } else {
            // Static mode: simple modulo
            let num_shards = self.num_shards.load(std::sync::atomic::Ordering::Acquire);
            (hash % num_shards as u64) as u16
        }
    }

    /// Select a shard using a pre-computed hash.
    ///
    /// This is faster than `select_shard` when you already have the hash.
    #[inline]
    pub fn select_shard_by_hash(&self, hash: u64) -> u16 {
        if let Some(ref mapper) = self.mapper {
            // Dynamic mode: use weighted selection
            mapper.select_shard(hash)
        } else {
            // Static mode: simple modulo
            let num_shards = self.num_shards.load(std::sync::atomic::Ordering::Acquire);
            (hash % num_shards as u64) as u16
        }
    }

    /// Ingest an event into the appropriate shard.
    pub fn ingest(&self, event: JsonValue) -> Result<(u16, u64), IngestionError> {
        // Serialize once upfront - avoids clone on retry
        let raw = Bytes::from(
            serde_json::to_vec(&event).map_err(|e| IngestionError::Serialization(e.to_string()))?,
        );
        let hash = xxhash_rust::xxh3::xxh3_64(&raw);
        let shard_id = self.select_shard_by_hash(hash);

        // Fast path for static mode: shard_id == index, skip HashMap lookup
        let shards = self.shards.read();
        let idx = if self.mapper.is_none() {
            shard_id as usize
        } else {
            let shard_index = self.shard_index.read();
            match shard_index.get(&shard_id).copied() {
                Some(idx) => idx,
                None => return Err(IngestionError::Backpressure),
            }
        };

        let Some(shard_lock) = shards.get(idx) else {
            return Err(IngestionError::Backpressure);
        };

        let mut shard = shard_lock.lock();

        match shard.try_push_raw(raw.clone()) {
            Ok(ts) => Ok((shard_id, ts)),
            Err(IngestionError::Backpressure) => {
                match self.backpressure_mode {
                    BackpressureMode::DropNewest => Err(IngestionError::Backpressure),
                    BackpressureMode::DropOldest => {
                        // The failed try_push_raw incremented events_dropped for the
                        // *new* event, but the new event isn't actually dropped — the
                        // oldest is. Correct the stats: undo the spurious drop count,
                        // pop the oldest (which is the real drop), and retry.
                        shard.stats.events_dropped -= 1;
                        let _ = shard.try_pop();
                        shard.stats.events_dropped += 1;
                        // Retry with the same bytes (Bytes clone is ref-counted)
                        shard.try_push_raw(raw).map(|ts| (shard_id, ts))
                    }
                    BackpressureMode::FailProducer => Err(IngestionError::Backpressure),
                    BackpressureMode::Sample { .. } => {
                        // Sampling is handled at a higher level
                        Err(IngestionError::Sampled)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Ingest a raw event (pre-serialized with cached hash).
    ///
    /// This is the fastest ingestion path:
    /// - Uses pre-computed hash for shard selection (no serialization)
    /// - Stores bytes directly (no clone needed, reference-counted)
    #[inline]
    pub fn ingest_raw(&self, event: RawEvent) -> Result<(u16, u64), IngestionError> {
        let shard_id = self.select_shard_by_hash(event.hash());

        // Fast path for static mode: shard_id == index, skip HashMap lookup
        let shards = self.shards.read();
        let idx = if self.mapper.is_none() {
            shard_id as usize
        } else {
            let shard_index = self.shard_index.read();
            match shard_index.get(&shard_id).copied() {
                Some(idx) => idx,
                None => return Err(IngestionError::Backpressure),
            }
        };

        let Some(shard_lock) = shards.get(idx) else {
            return Err(IngestionError::Backpressure);
        };

        let mut shard = shard_lock.lock();

        match shard.try_push_raw(event.bytes()) {
            Ok(ts) => Ok((shard_id, ts)),
            Err(IngestionError::Backpressure) => {
                match self.backpressure_mode {
                    BackpressureMode::DropNewest => Err(IngestionError::Backpressure),
                    BackpressureMode::DropOldest => {
                        shard.stats.events_dropped -= 1;
                        let _ = shard.try_pop();
                        shard.stats.events_dropped += 1;
                        // Retry the push
                        shard.try_push_raw(event.bytes()).map(|ts| (shard_id, ts))
                    }
                    BackpressureMode::FailProducer => Err(IngestionError::Backpressure),
                    BackpressureMode::Sample { .. } => {
                        // Sampling is handled at a higher level
                        Err(IngestionError::Sampled)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Get a reference to a shard by ID.
    pub fn shard(&self, id: u16) -> Option<ShardRef<'_>> {
        let shards = self.shards.read();
        let shard_index = self.shard_index.read();

        let idx = shard_index.get(&id).copied()?;
        drop(shard_index);

        if idx < shards.len() {
            Some(ShardRef { guard: shards, idx })
        } else {
            None
        }
    }

    /// Execute a function with exclusive access to a shard.
    pub fn with_shard<F, R>(&self, id: u16, f: F) -> Option<R>
    where
        F: FnOnce(&mut Shard) -> R,
    {
        let shards = self.shards.read();
        let shard_index = self.shard_index.read();

        let idx = shard_index.get(&id).copied()?;
        drop(shard_index);

        shards.get(idx).map(|shard_lock| {
            let mut shard = shard_lock.lock();
            f(&mut shard)
        })
    }

    /// Iterate over all active shard IDs.
    pub fn shard_ids(&self) -> Vec<u16> {
        self.shard_index.read().keys().copied().collect()
    }

    /// Get aggregated statistics from all shards.
    pub fn stats(&self) -> ShardStats {
        let mut total = ShardStats::default();
        let shards = self.shards.read();
        for shard_lock in shards.iter() {
            let stats = *shard_lock.lock().stats();
            total.events_ingested += stats.events_ingested;
            total.events_dropped += stats.events_dropped;
            total.batches_dispatched += stats.batches_dispatched;
        }
        total
    }

    /// Add a new shard (for dynamic scaling).
    /// Returns the new shard ID.
    pub fn add_shard(&self) -> Result<u16, ScalingError> {
        let mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        // Scale up through the mapper
        let new_ids = mapper.scale_up(1)?;
        let new_id = new_ids[0];

        // Create the actual shard
        let metrics = mapper.metrics_collector(new_id).ok_or_else(|| {
            ScalingError::InvalidPolicy(format!("no metrics collector for shard {}", new_id))
        })?;
        let new_shard = Shard::with_metrics(new_id, self.ring_buffer_capacity, metrics);

        // Add to our collections
        let mut shards = self.shards.write();
        let mut shard_index = self.shard_index.write();

        let idx = shards.len();
        shards.push(parking_lot::Mutex::new(new_shard));
        shard_index.insert(new_id, idx);

        self.num_shards
            .fetch_add(1, std::sync::atomic::Ordering::Release);

        Ok(new_id)
    }

    /// Start draining a shard (for dynamic scaling).
    pub fn drain_shard(&self, shard_id: u16) -> Result<(), ScalingError> {
        let mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        // Mark as draining in the mapper
        if let Some(collector) = mapper.metrics_collector(shard_id) {
            collector.set_draining(true);
        }

        Ok(())
    }

    /// Remove a stopped shard (for dynamic scaling).
    pub fn remove_shard(&self, shard_id: u16) -> Result<(), ScalingError> {
        // Ensure dynamic scaling is enabled
        let _mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        let mut shards = self.shards.write();
        let mut shard_index = self.shard_index.write();

        if let Some(idx) = shard_index.remove(&shard_id) {
            // Remove the shard (swap_remove for efficiency)
            shards.swap_remove(idx);

            // Update indices for any shard that was swapped
            if idx < shards.len() {
                // Find the shard that was at the end and is now at idx
                let moved_shard_id = shards[idx].lock().id;
                shard_index.insert(moved_shard_id, idx);
            }

            self.num_shards
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        }

        Ok(())
    }

    /// Collect metrics from all shards (for dynamic scaling decisions).
    pub fn collect_metrics(&self) -> Option<Vec<ShardMetrics>> {
        self.mapper.as_ref().map(|m| m.collect_metrics())
    }

    /// Evaluate and optionally execute scaling.
    pub fn evaluate_scaling(&self) -> ScalingDecision {
        self.mapper
            .as_ref()
            .map(|m| m.evaluate_scaling())
            .unwrap_or(ScalingDecision::None)
    }
}

/// A reference to a shard that holds the read lock.
pub struct ShardRef<'a> {
    guard: parking_lot::RwLockReadGuard<'a, Vec<parking_lot::Mutex<Shard>>>,
    idx: usize,
}

impl<'a> ShardRef<'a> {
    /// Lock the shard for exclusive access.
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, Shard> {
        self.guard[self.idx].lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shard_push_pop() {
        let mut shard = Shard::new(0, 1024);

        let ts = shard.try_push(json!({"test": 1})).unwrap();
        assert!(ts > 0);
        assert_eq!(shard.len(), 1);

        let event = shard.try_pop().unwrap();
        assert_eq!(event.shard_id, 0);
        assert_eq!(event.insertion_ts, ts);
        assert!(shard.is_empty());
    }

    #[test]
    fn test_shard_manager_routing() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Same event should always go to the same shard
        let event = json!({"key": "value"});
        let shard1 = manager.select_shard(&event);
        let shard2 = manager.select_shard(&event);
        assert_eq!(shard1, shard2);

        // Different events may go to different shards
        let events: Vec<_> = (0..100).map(|i| json!({"i": i})).collect();
        let shards: std::collections::HashSet<_> =
            events.iter().map(|e| manager.select_shard(e)).collect();

        // With 100 random events and 4 shards, we should hit multiple shards
        assert!(shards.len() > 1);
    }

    #[test]
    fn test_shard_manager_ingest() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        for i in 0..100 {
            let event = json!({"i": i});
            let result = manager.ingest(event);
            assert!(result.is_ok());
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 100);
        assert_eq!(stats.events_dropped, 0);
    }

    #[test]
    fn test_backpressure_drop_newest() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropNewest);

        // Fill the buffer (capacity 4, usable 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Next insert should fail
        let result = manager.ingest(json!({"i": 999}));
        assert!(matches!(result, Err(IngestionError::Backpressure)));

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 3);
        assert_eq!(stats.events_dropped, 1);
    }

    #[test]
    fn test_backpressure_drop_oldest() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Next insert should succeed by dropping oldest
        let result = manager.ingest(json!({"i": 999}));
        assert!(result.is_ok());

        // Verify the oldest was dropped
        let shard = manager.shard(0).unwrap();
        let events = shard.lock().pop_batch(10);

        // Should have events 1, 2, 999 (0 was dropped)
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].parse().unwrap(), json!({"i": 1}));
        assert_eq!(events[2].parse().unwrap(), json!({"i": 999}));
    }

    #[test]
    fn test_raw_event_ingestion() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        for i in 0..100 {
            let raw = RawEvent::from_str(&format!(r#"{{"i": {}}}"#, i));
            let result = manager.ingest_raw(raw);
            assert!(result.is_ok());
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 100);
        assert_eq!(stats.events_dropped, 0);
    }

    #[test]
    fn test_remove_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.remove_shard(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_add_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.add_shard();
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_drain_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.drain_shard(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_drop_oldest_counts_dropped_events() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer (capacity 4, usable 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // This should succeed by dropping the oldest event
        manager.ingest(json!({"i": 999})).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 4);
        // The initial push fails (counted as dropped), then retry succeeds
        assert_eq!(
            stats.events_dropped, 1,
            "DropOldest cycle should count exactly one drop"
        );
    }

    #[test]
    fn test_drop_oldest_raw_counts_dropped_events() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer
        for i in 0..3 {
            let raw = RawEvent::from_str(&format!(r#"{{"i": {}}}"#, i));
            manager.ingest_raw(raw).unwrap();
        }

        // This should succeed by dropping the oldest event
        let raw = RawEvent::from_str(r#"{"i": 999}"#);
        manager.ingest_raw(raw).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 4);
        assert_eq!(
            stats.events_dropped, 1,
            "DropOldest cycle should count exactly one drop"
        );
    }

    /// Pin the current contract for `BackpressureMode::Sample`:
    /// it returns `IngestionError::Sampled` once the buffer fills,
    /// indistinguishable in shape from a `Backpressure` rejection.
    /// Sampling itself ("keep 1 in N events") is **not implemented**
    /// — the comments in `ingest` / `ingest_raw` defer it to "a
    /// higher level" that does not exist. A consumer setting this
    /// mode today gets a rejection signal, never probabilistic
    /// admission.
    ///
    /// This test pins that contract so it cannot quietly change
    /// without an explicit decision. If sampling is ever wired up,
    /// this test will fail and force an update — at which point
    /// the implementer should also add coverage for the
    /// rate-proportional admission rate.
    #[test]
    fn sample_mode_currently_returns_sampled_after_buffer_fills() {
        // TODO(coverage round 2): `BackpressureMode::Sample` is
        // dead-on-arrival until "higher level" sampling lands;
        // see comments at `ShardManager::ingest` / `ingest_raw`.
        let manager = ShardManager::new(1, 4, BackpressureMode::Sample { rate: 2 });

        // Fill the buffer (capacity 4, usable 3).
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Both ingest paths must report `Sampled` — not `Backpressure`,
        // not `Ok` — so callers can distinguish the (currently
        // unused) sampling rejection from a hard backpressure
        // rejection in case sampling is wired up later.
        let json_result = manager.ingest(json!({"i": 999}));
        assert!(
            matches!(json_result, Err(IngestionError::Sampled)),
            "Sample mode must return Sampled on a full buffer (got {:?})",
            json_result
        );

        let raw_result = manager.ingest_raw(RawEvent::from_str(r#"{"i": 999}"#));
        assert!(
            matches!(raw_result, Err(IngestionError::Sampled)),
            "Sample mode must return Sampled on a full buffer via ingest_raw (got {:?})",
            raw_result
        );
    }

    #[test]
    fn test_drop_oldest_multiple_cycles() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer (usable capacity 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Push 5 more events, each triggers a DropOldest cycle
        for i in 3..8 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 8);
        assert_eq!(
            stats.events_dropped, 5,
            "each DropOldest cycle should count one drop"
        );
    }
}
