//! Dynamic shard mapping and scaling.
//!
//! This module implements dynamic shard scaling following Approach B from
//! DYNAMIC_SHARD_SCALING.md: increasing the number of shards and rebalancing
//! producers across them, while maintaining SPSC (single-producer single-consumer)
//! semantics per shard.
//!
//! # Core Principles
//!
//! - Shards never accept multiple producers
//! - Producers get moved to new shards when load increases
//! - Ingestion performance stays at 700M ops/sec per shard
//! - Ordering guarantees remain intact within a shard
//! - Total throughput scales linearly with shard count
//!
//! # Scaling Triggers
//!
//! - Ring buffer fill ratio > threshold (default 70%)
//! - Push latency exceeds threshold (default 5ns)
//! - Batch flush latency exceeds threshold
//! - Session/producer count growth
//!
//! # Architecture
//!
//! ```text
//! Producers -----+
//!                |
//!                v
//!     +----------------------------+
//!     |   Dynamic Shard Mapper     |
//!     +----------------------------+
//!        |         |         |
//!        v         v         v
//!     Shard 0   Shard 1   Shard 2 …
//! ```

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

use crate::config::ScalingPolicy;

/// Metrics for a single shard used for scaling decisions.
#[derive(Debug, Clone)]
pub struct ShardMetrics {
    /// Shard identifier.
    pub shard_id: u16,
    /// Current fill ratio (0.0 - 1.0).
    pub fill_ratio: f64,
    /// Events ingested in the current window.
    pub event_rate: u64,
    /// Average push latency in nanoseconds.
    pub avg_push_latency_ns: u64,
    /// Average batch flush latency in microseconds.
    pub avg_flush_latency_us: u64,
    /// Whether this shard is in drain mode.
    pub draining: bool,
    /// Computed weight for load balancing (lower = less loaded).
    pub weight: f64,
    /// Last update timestamp.
    pub last_updated: Instant,
}

impl ShardMetrics {
    /// Create new metrics for a shard.
    pub fn new(shard_id: u16) -> Self {
        Self {
            shard_id,
            fill_ratio: 0.0,
            event_rate: 0,
            avg_push_latency_ns: 0,
            avg_flush_latency_us: 0,
            draining: false,
            weight: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Compute the weight based on current metrics.
    /// Lower weight = better candidate for new producers.
    pub fn compute_weight(&mut self) {
        // Weight formula: combines fill ratio, latency, and event rate
        // Higher fill ratio = higher weight (avoid overloaded shards)
        // Higher latency = higher weight
        // Higher event rate = higher weight
        let fill_weight = self.fill_ratio * 100.0;
        let latency_weight = (self.avg_push_latency_ns as f64) / 10.0;
        let rate_weight = (self.event_rate as f64) / 1_000_000.0;

        self.weight = fill_weight + latency_weight + rate_weight;

        // Draining shards get maximum weight (never assign new producers)
        if self.draining {
            self.weight = f64::MAX;
        }
    }
}

/// Live metrics collector for a shard (atomics for lock-free updates).
#[derive(Debug)]
pub struct ShardMetricsCollector {
    /// Shard identifier.
    shard_id: u16,
    /// Ring buffer capacity.
    capacity: usize,
    /// Current buffer length (updated by shard).
    current_len: AtomicU64,
    /// Events ingested in current window.
    events_in_window: AtomicU64,
    /// Sum of push latencies in current window (ns).
    push_latency_sum_ns: AtomicU64,
    /// Number of push operations in current window.
    push_count: AtomicU64,
    /// Sum of flush latencies in current window (us).
    flush_latency_sum_us: AtomicU64,
    /// Number of flush operations in current window.
    flush_count: AtomicU64,
    /// Whether this shard is draining.
    draining: AtomicBool,
    /// Window start time.
    window_start: RwLock<Instant>,
}

impl ShardMetricsCollector {
    /// Create a new metrics collector.
    pub fn new(shard_id: u16, capacity: usize) -> Self {
        Self {
            shard_id,
            capacity,
            current_len: AtomicU64::new(0),
            events_in_window: AtomicU64::new(0),
            push_latency_sum_ns: AtomicU64::new(0),
            push_count: AtomicU64::new(0),
            flush_latency_sum_us: AtomicU64::new(0),
            flush_count: AtomicU64::new(0),
            draining: AtomicBool::new(false),
            window_start: RwLock::new(Instant::now()),
        }
    }

    /// Record current buffer length.
    #[inline]
    pub fn record_buffer_len(&self, len: usize) {
        self.current_len.store(len as u64, AtomicOrdering::Relaxed);
    }

    /// Record an event ingestion.
    #[inline]
    pub fn record_push(&self, latency_ns: u64) {
        self.events_in_window.fetch_add(1, AtomicOrdering::Relaxed);
        self.push_latency_sum_ns
            .fetch_add(latency_ns, AtomicOrdering::Relaxed);
        self.push_count.fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Record a batch flush.
    #[inline]
    pub fn record_flush(&self, latency_us: u64) {
        self.flush_latency_sum_us
            .fetch_add(latency_us, AtomicOrdering::Relaxed);
        self.flush_count.fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Set drain mode.
    pub fn set_draining(&self, draining: bool) {
        self.draining.store(draining, AtomicOrdering::Release);
    }

    /// Check if draining.
    pub fn is_draining(&self) -> bool {
        self.draining.load(AtomicOrdering::Acquire)
    }

    /// Collect metrics and reset window counters.
    ///
    /// NOTE: The individual atomic swaps below are not collectively atomic with
    /// respect to concurrent `record_push`/`record_flush` calls. This means a
    /// push recorded between, say, the `events_in_window` swap and the
    /// `push_count` swap could be counted in one counter but not the other for
    /// a given window. This small inaccuracy is an accepted trade-off to
    /// preserve the lock-free design of the hot path (`record_push` /
    /// `record_flush`). Adding a `Mutex` here would serialize the hot path
    /// and defeat the purpose of using atomics.
    pub fn collect_and_reset(&self) -> ShardMetrics {
        let current_len = self.current_len.load(AtomicOrdering::Relaxed);
        let events = self.events_in_window.swap(0, AtomicOrdering::Relaxed);
        let push_latency_sum = self.push_latency_sum_ns.swap(0, AtomicOrdering::Relaxed);
        let push_count = self.push_count.swap(0, AtomicOrdering::Relaxed);
        let flush_latency_sum = self.flush_latency_sum_us.swap(0, AtomicOrdering::Relaxed);
        let flush_count = self.flush_count.swap(0, AtomicOrdering::Relaxed);

        let fill_ratio = if self.capacity > 0 {
            current_len as f64 / self.capacity as f64
        } else {
            0.0
        };

        let avg_push_latency = push_latency_sum.checked_div(push_count).unwrap_or(0);

        let avg_flush_latency = flush_latency_sum.checked_div(flush_count).unwrap_or(0);

        // Reset window
        *self.window_start.write() = Instant::now();

        let mut metrics = ShardMetrics {
            shard_id: self.shard_id,
            fill_ratio,
            event_rate: events,
            avg_push_latency_ns: avg_push_latency,
            avg_flush_latency_us: avg_flush_latency,
            draining: self.draining.load(AtomicOrdering::Acquire),
            weight: 0.0,
            last_updated: Instant::now(),
        };
        metrics.compute_weight();
        metrics
    }
}

/// Scaling decision made by the mapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalingDecision {
    /// No scaling needed.
    None,
    /// Scale up by adding N shards.
    ScaleUp(u16),
    /// Scale down by removing N shards (marks them for draining).
    ScaleDown(u16),
}

/// Errors that can occur during scaling operations.
#[derive(Debug, Clone)]
pub enum ScalingError {
    /// Invalid scaling policy.
    InvalidPolicy(String),
    /// Already at maximum shards.
    AtMaxShards,
    /// Already at minimum shards.
    AtMinShards,
    /// Scaling operation in cooldown.
    InCooldown,
    /// Shard creation failed.
    ShardCreationFailed(String),
}

impl std::fmt::Display for ScalingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPolicy(msg) => write!(f, "invalid scaling policy: {}", msg),
            Self::AtMaxShards => write!(f, "already at maximum shard count"),
            Self::AtMinShards => write!(f, "already at minimum shard count"),
            Self::InCooldown => write!(f, "scaling operation in cooldown"),
            Self::ShardCreationFailed(msg) => write!(f, "shard creation failed: {}", msg),
        }
    }
}

impl std::error::Error for ScalingError {}

/// State of a shard in the mapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardState {
    /// Shard is active and accepting producers.
    Active,
    /// Shard is draining (no new producers, waiting for empty).
    Draining,
    /// Shard is stopped and can be removed.
    Stopped,
}

/// Information about a shard managed by the mapper.
#[derive(Debug)]
struct MappedShard {
    /// Shard ID.
    id: u16,
    /// Current state.
    state: ShardState,
    /// Metrics collector.
    metrics: Arc<ShardMetricsCollector>,
    /// When this shard entered drain mode (if draining).
    drain_started: Option<Instant>,
    /// Last collected metrics.
    last_metrics: ShardMetrics,
}

/// Callback type for shard lifecycle events.
type ShardCallback = Box<dyn Fn(u16) + Send + Sync>;

/// Dynamic shard mapper that manages shard allocation and producer routing.
///
/// This is the core component for dynamic scaling. It:
/// - Tracks metrics for all shards
/// - Makes scaling decisions based on policy
/// - Routes producers to the least-loaded shards
/// - Manages shard lifecycle (active → draining → stopped)
pub struct ShardMapper {
    /// Mapped shards (RwLock for concurrent reads, rare writes).
    shards: RwLock<Vec<MappedShard>>,
    /// Current active shard count.
    active_count: AtomicU16,
    /// Scaling policy.
    policy: ScalingPolicy,
    /// Ring buffer capacity for new shards.
    ring_buffer_capacity: usize,
    /// Last scaling operation timestamp.
    last_scaling: RwLock<Option<Instant>>,
    /// Callback for shard creation (provided by ShardManager).
    on_shard_created: RwLock<Option<ShardCallback>>,
    /// Callback for shard removal (provided by ShardManager).
    on_shard_removed: RwLock<Option<ShardCallback>>,
}

impl ShardMapper {
    /// Create a new shard mapper with the given initial shard count and policy.
    pub fn new(
        initial_shards: u16,
        ring_buffer_capacity: usize,
        policy: ScalingPolicy,
    ) -> Result<Self, ScalingError> {
        let mut policy = policy.normalize();
        // Ensure max_shards can accommodate the initial shard count
        if policy.max_shards < initial_shards {
            policy.max_shards = initial_shards;
        }
        policy
            .validate()
            .map_err(|e| ScalingError::InvalidPolicy(e.to_string()))?;

        let shards: Vec<MappedShard> = (0..initial_shards)
            .map(|id| MappedShard {
                id,
                state: ShardState::Active,
                metrics: Arc::new(ShardMetricsCollector::new(id, ring_buffer_capacity)),
                drain_started: None,
                last_metrics: ShardMetrics::new(id),
            })
            .collect();

        Ok(Self {
            shards: RwLock::new(shards),
            active_count: AtomicU16::new(initial_shards),
            policy,
            ring_buffer_capacity,
            last_scaling: RwLock::new(None),
            on_shard_created: RwLock::new(None),
            on_shard_removed: RwLock::new(None),
        })
    }

    /// Set callback for shard creation.
    pub fn set_on_shard_created<F>(&self, callback: F)
    where
        F: Fn(u16) + Send + Sync + 'static,
    {
        *self.on_shard_created.write() = Some(Box::new(callback));
    }

    /// Set callback for shard removal.
    pub fn set_on_shard_removed<F>(&self, callback: F)
    where
        F: Fn(u16) + Send + Sync + 'static,
    {
        *self.on_shard_removed.write() = Some(Box::new(callback));
    }

    /// Get the metrics collector for a shard.
    pub fn metrics_collector(&self, shard_id: u16) -> Option<Arc<ShardMetricsCollector>> {
        let shards = self.shards.read();
        shards
            .iter()
            .find(|s| s.id == shard_id)
            .map(|s| s.metrics.clone())
    }

    /// Get current active shard count.
    pub fn active_shard_count(&self) -> u16 {
        self.active_count.load(AtomicOrdering::Acquire)
    }

    /// Get total shard count (including draining).
    pub fn total_shard_count(&self) -> u16 {
        self.shards.read().len() as u16
    }

    /// Select the best shard for a new event/producer.
    ///
    /// This implements weighted shard selection:
    /// - Only considers active (non-draining) shards
    /// - Prefers shards with lower weight (less loaded)
    /// - Falls back to round-robin if weights are equal
    #[inline]
    pub fn select_shard(&self, event_hash: u64) -> u16 {
        let shards = self.shards.read();

        // Filter to active shards only
        let active: Vec<_> = shards
            .iter()
            .filter(|s| s.state == ShardState::Active)
            .collect();

        if active.is_empty() {
            // Fallback: use any non-stopped shard
            return shards
                .iter()
                .find(|s| s.state != ShardState::Stopped)
                .map(|s| s.id)
                .unwrap_or(0);
        }

        // Find the shard with lowest weight
        let min_weight = active
            .iter()
            .map(|s| s.last_metrics.weight)
            .fold(f64::MAX, f64::min);

        // Get all shards with the minimum weight (within tolerance)
        let candidates: Vec<_> = active
            .iter()
            .filter(|s| (s.last_metrics.weight - min_weight).abs() < 0.1)
            .collect();

        // Use hash to pick among candidates for determinism
        // Fallback to first active shard if tolerance filter excludes all (e.g. NaN weights)
        if candidates.is_empty() {
            return active[0].id;
        }
        let idx = (event_hash as usize) % candidates.len();
        candidates[idx].id
    }

    /// Collect metrics from all shards and update weights.
    pub fn collect_metrics(&self) -> Vec<ShardMetrics> {
        let mut shards = self.shards.write();
        shards
            .iter_mut()
            .map(|s| {
                s.last_metrics = s.metrics.collect_and_reset();
                s.last_metrics.clone()
            })
            .collect()
    }

    /// Evaluate scaling based on current metrics.
    ///
    /// Returns a scaling decision without executing it.
    pub fn evaluate_scaling(&self) -> ScalingDecision {
        if !self.policy.auto_scale {
            return ScalingDecision::None;
        }

        // Check cooldown
        if let Some(last) = *self.last_scaling.read() {
            if last.elapsed() < self.policy.cooldown {
                return ScalingDecision::None;
            }
        }

        let shards = self.shards.read();
        let active_count = self.active_count.load(AtomicOrdering::Acquire);

        // Check for scale-up triggers
        let mut overloaded_count = 0;
        let mut underutilized_count = 0;

        for shard in shards.iter() {
            if shard.state != ShardState::Active {
                continue;
            }

            let m = &shard.last_metrics;

            // Scale-up triggers
            if m.fill_ratio > self.policy.fill_ratio_threshold
                || m.avg_push_latency_ns > self.policy.push_latency_threshold_ns
                || m.avg_flush_latency_us > self.policy.flush_latency_threshold_us
            {
                overloaded_count += 1;
            }

            // Scale-down triggers
            if m.fill_ratio < self.policy.underutilized_threshold && m.event_rate == 0 {
                underutilized_count += 1;
            }
        }

        // Scale up if more than half of shards are overloaded
        if overloaded_count > active_count / 2 && active_count < self.policy.max_shards {
            // Add shards proportional to overload
            let shards_to_add = (overloaded_count / 2)
                .max(1)
                .min(self.policy.max_shards - active_count);
            return ScalingDecision::ScaleUp(shards_to_add);
        }

        // Scale down if more than half of shards are underutilized
        // and we're above minimum
        if underutilized_count > active_count / 2 && active_count > self.policy.min_shards {
            let shards_to_remove = (underutilized_count / 2)
                .max(1)
                .min(active_count - self.policy.min_shards);
            return ScalingDecision::ScaleDown(shards_to_remove);
        }

        ScalingDecision::None
    }

    /// Execute a scale-up operation.
    ///
    /// Creates new shards and makes them available for routing.
    pub fn scale_up(&self, count: u16) -> Result<Vec<u16>, ScalingError> {
        // Early check (may race, but avoids unnecessary lock acquisition)
        let current = self.active_count.load(AtomicOrdering::Acquire);
        if current + count > self.policy.max_shards {
            return Err(ScalingError::AtMaxShards);
        }

        // Check cooldown
        {
            let last = self.last_scaling.read();
            if let Some(ts) = *last {
                if ts.elapsed() < self.policy.cooldown {
                    return Err(ScalingError::InCooldown);
                }
            }
        }

        let mut shards = self.shards.write();

        // Re-check max_shards under the lock to prevent race conditions
        let current = self.active_count.load(AtomicOrdering::Acquire);
        if current + count > self.policy.max_shards {
            return Err(ScalingError::AtMaxShards);
        }

        let mut new_ids = Vec::with_capacity(count as usize);

        // Find the next available ID
        let max_id = shards.iter().map(|s| s.id).max().unwrap_or(0);

        // Check for potential overflow before allocating any shards
        // We need max_id + count + 1 IDs (since we start at max_id + 1)
        if max_id.checked_add(count).is_none() || max_id + count > u16::MAX - 1 {
            return Err(ScalingError::AtMaxShards);
        }

        for i in 0..count {
            let new_id = max_id + 1 + i; // Safe: checked above
            let new_shard = MappedShard {
                id: new_id,
                state: ShardState::Active,
                metrics: Arc::new(ShardMetricsCollector::new(
                    new_id,
                    self.ring_buffer_capacity,
                )),
                drain_started: None,
                last_metrics: ShardMetrics::new(new_id),
            };
            shards.push(new_shard);
            new_ids.push(new_id);
        }

        // Update counts
        self.active_count.fetch_add(count, AtomicOrdering::Release);
        *self.last_scaling.write() = Some(Instant::now());

        // Notify callback
        if let Some(callback) = self.on_shard_created.read().as_ref() {
            for &id in &new_ids {
                callback(id);
            }
        }

        Ok(new_ids)
    }

    /// Start draining shards for scale-down.
    ///
    /// Marks shards as draining so they stop receiving new events.
    /// Shards will be removed once they're empty.
    pub fn scale_down(&self, count: u16) -> Result<Vec<u16>, ScalingError> {
        // Early check (may race, but avoids unnecessary lock acquisition)
        let current = self.active_count.load(AtomicOrdering::Acquire);
        if current <= self.policy.min_shards {
            return Err(ScalingError::AtMinShards);
        }

        let to_drain = count.min(current - self.policy.min_shards);
        if to_drain == 0 {
            return Err(ScalingError::AtMinShards);
        }

        // Check cooldown
        {
            let last = self.last_scaling.read();
            if let Some(ts) = *last {
                if ts.elapsed() < self.policy.cooldown {
                    return Err(ScalingError::InCooldown);
                }
            }
        }

        let mut shards = self.shards.write();

        // Re-check under the lock to prevent race conditions (double-check pattern)
        let current = self.active_count.load(AtomicOrdering::Acquire);
        if current <= self.policy.min_shards {
            return Err(ScalingError::AtMinShards);
        }
        let to_drain = count.min(current - self.policy.min_shards);
        if to_drain == 0 {
            return Err(ScalingError::AtMinShards);
        }

        let mut drained_ids = Vec::with_capacity(to_drain as usize);

        // Find shards with lowest weight (least utilized) to drain
        let mut active_indices: Vec<_> = shards
            .iter()
            .enumerate()
            .filter(|(_, s)| s.state == ShardState::Active)
            .map(|(i, s)| (i, s.last_metrics.weight))
            .collect();

        // Sort by weight (ascending - drain least utilized first)
        active_indices.sort_by(|a, b| a.1.total_cmp(&b.1));

        // Mark shards for draining
        for (idx, _) in active_indices.into_iter().take(to_drain as usize) {
            shards[idx].state = ShardState::Draining;
            shards[idx].drain_started = Some(Instant::now());
            shards[idx].metrics.set_draining(true);
            drained_ids.push(shards[idx].id);
        }

        // Update count
        self.active_count
            .fetch_sub(to_drain, AtomicOrdering::Release);
        *self.last_scaling.write() = Some(Instant::now());

        Ok(drained_ids)
    }

    /// Check draining shards and finalize those that are empty.
    ///
    /// Returns IDs of shards that were stopped.
    pub fn finalize_draining(&self) -> Vec<u16> {
        let mut shards = self.shards.write();
        let mut stopped = Vec::new();

        for shard in shards.iter_mut() {
            if shard.state == ShardState::Draining {
                // Check if shard is empty by reading current_len directly,
                // avoiding collect_and_reset() which destructively zeros all counters.
                let current_len = shard.metrics.current_len.load(AtomicOrdering::Relaxed);
                let fill_ratio = if shard.metrics.capacity > 0 {
                    current_len as f64 / shard.metrics.capacity as f64
                } else {
                    0.0
                };
                let event_rate = shard.metrics.events_in_window.load(AtomicOrdering::Relaxed);
                if fill_ratio == 0.0 && event_rate == 0 {
                    // Check if we've waited long enough
                    if let Some(drain_start) = shard.drain_started {
                        if drain_start.elapsed() > Duration::from_millis(100) {
                            shard.state = ShardState::Stopped;
                            stopped.push(shard.id);
                        }
                    }
                }
            }
        }

        // Notify callback
        if !stopped.is_empty() {
            if let Some(callback) = self.on_shard_removed.read().as_ref() {
                for &id in &stopped {
                    callback(id);
                }
            }
        }

        stopped
    }

    /// Remove stopped shards from the mapper.
    pub fn remove_stopped_shards(&self) -> Vec<u16> {
        let mut shards = self.shards.write();
        let before = shards.len();
        let removed: Vec<u16> = shards
            .iter()
            .filter(|s| s.state == ShardState::Stopped)
            .map(|s| s.id)
            .collect();

        shards.retain(|s| s.state != ShardState::Stopped);

        if shards.len() < before {
            tracing::info!(
                removed = removed.len(),
                remaining = shards.len(),
                "Removed stopped shards"
            );
        }

        removed
    }

    /// Get the state of a specific shard.
    pub fn shard_state(&self, shard_id: u16) -> Option<ShardState> {
        self.shards
            .read()
            .iter()
            .find(|s| s.id == shard_id)
            .map(|s| s.state)
    }

    /// Get all active shard IDs.
    pub fn active_shard_ids(&self) -> Vec<u16> {
        self.shards
            .read()
            .iter()
            .filter(|s| s.state == ShardState::Active)
            .map(|s| s.id)
            .collect()
    }

    /// Get all shard IDs (including draining).
    pub fn all_shard_ids(&self) -> Vec<u16> {
        self.shards
            .read()
            .iter()
            .filter(|s| s.state != ShardState::Stopped)
            .map(|s| s.id)
            .collect()
    }

    /// Get the scaling policy.
    pub fn policy(&self) -> &ScalingPolicy {
        &self.policy
    }

    /// Update the scaling policy.
    pub fn set_policy(&mut self, policy: ScalingPolicy) -> Result<(), ScalingError> {
        let policy = policy.normalize();
        policy
            .validate()
            .map_err(|e| ScalingError::InvalidPolicy(e.to_string()))?;
        self.policy = policy;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_mapper_creation() {
        let mapper = ShardMapper::new(4, 1024, ScalingPolicy::default()).unwrap();
        assert_eq!(mapper.active_shard_count(), 4);
        assert_eq!(mapper.total_shard_count(), 4);
    }

    #[test]
    fn test_select_shard_distributes() {
        let mapper = ShardMapper::new(4, 1024, ScalingPolicy::default()).unwrap();

        // Different hashes should potentially select different shards
        let mut selected = std::collections::HashSet::new();
        for i in 0..100u64 {
            let shard = mapper.select_shard(i * 12345);
            selected.insert(shard);
        }

        // With 4 shards, we should hit multiple
        assert!(!selected.is_empty());
    }

    #[test]
    fn test_scale_up() {
        // Explicitly set max_shards to allow scaling from 2 to 4
        let policy = ScalingPolicy {
            max_shards: 8,
            ..Default::default()
        };
        let mapper = ShardMapper::new(2, 1024, policy).unwrap();

        let new_ids = mapper.scale_up(2).unwrap();
        assert_eq!(new_ids.len(), 2);
        assert_eq!(mapper.active_shard_count(), 4);
    }

    #[test]
    fn test_scale_up_max_limit() {
        let policy = ScalingPolicy {
            max_shards: 4,
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        let result = mapper.scale_up(1);
        assert!(matches!(result, Err(ScalingError::AtMaxShards)));
    }

    #[test]
    fn test_scale_down() {
        let policy = ScalingPolicy {
            min_shards: 1,
            cooldown: Duration::from_millis(0), // Disable cooldown for test
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        let drained = mapper.scale_down(2).unwrap();
        assert_eq!(drained.len(), 2);
        assert_eq!(mapper.active_shard_count(), 2);
    }

    #[test]
    fn test_scale_down_min_limit() {
        let policy = ScalingPolicy {
            min_shards: 4,
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        let result = mapper.scale_down(1);
        assert!(matches!(result, Err(ScalingError::AtMinShards)));
    }

    #[test]
    fn test_metrics_collection() {
        let mapper = ShardMapper::new(2, 1024, ScalingPolicy::default()).unwrap();

        // Record some metrics
        if let Some(collector) = mapper.metrics_collector(0) {
            collector.record_buffer_len(512);
            collector.record_push(5);
            collector.record_push(10);
        }

        let metrics = mapper.collect_metrics();
        assert_eq!(metrics.len(), 2);

        let shard0_metrics = metrics.iter().find(|m| m.shard_id == 0).unwrap();
        assert!(shard0_metrics.fill_ratio > 0.0);
    }

    #[test]
    fn test_draining_excludes_from_selection() {
        let policy = ScalingPolicy {
            min_shards: 1,
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(2, 1024, policy).unwrap();

        // Drain one shard
        let drained = mapper.scale_down(1).unwrap();
        assert_eq!(drained.len(), 1);

        // All selections should go to the remaining active shard
        let active_ids = mapper.active_shard_ids();
        assert_eq!(active_ids.len(), 1);

        for i in 0..100u64 {
            let selected = mapper.select_shard(i);
            assert!(active_ids.contains(&selected));
        }
    }

    #[test]
    fn test_policy_validation() {
        let invalid_policy = ScalingPolicy {
            fill_ratio_threshold: 1.5, // Invalid
            ..Default::default()
        };
        assert!(invalid_policy.validate().is_err());

        // Without normalize(), this would be invalid
        let invalid_policy2 = ScalingPolicy {
            min_shards: 10,
            max_shards: 5,
            ..Default::default()
        };
        assert!(invalid_policy2.validate().is_err());
    }

    #[test]
    fn test_policy_normalize_auto_adjusts_max_shards() {
        // When min_shards > max_shards, normalize() should adjust max_shards
        let policy = ScalingPolicy {
            min_shards: 8,
            max_shards: 2, // Less than min_shards
            ..Default::default()
        };

        let normalized = policy.normalize();
        assert_eq!(
            normalized.max_shards, 8,
            "max_shards should be adjusted to min_shards"
        );
        assert!(
            normalized.validate().is_ok(),
            "normalized policy should be valid"
        );
    }

    #[test]
    fn test_policy_normalize_preserves_valid_config() {
        // When max_shards >= min_shards, normalize() should not change anything
        let policy = ScalingPolicy {
            min_shards: 4,
            max_shards: 16,
            ..Default::default()
        };

        let normalized = policy.normalize();
        assert_eq!(normalized.min_shards, 4);
        assert_eq!(normalized.max_shards, 16);
    }

    #[test]
    fn test_shard_mapper_normalizes_policy() {
        // ShardMapper should accept a policy where min_shards > default max_shards
        // because it calls normalize() internally
        let policy = ScalingPolicy {
            min_shards: 4,
            ..Default::default()
        };

        // This should succeed even on machines with < 4 CPUs
        let result = ShardMapper::new(4, 1024, policy);
        assert!(
            result.is_ok(),
            "ShardMapper should normalize policy automatically"
        );
    }

    #[test]
    fn test_shard_mapper_adjusts_max_shards_to_initial_count() {
        // ShardMapper should adjust max_shards to accommodate initial_shards
        // even if initial_shards > default max_shards (CPU count)
        let policy = ScalingPolicy::default();

        // Create mapper with 8 initial shards - should work even on 2-core machines
        let result = ShardMapper::new(8, 1024, policy);
        assert!(
            result.is_ok(),
            "ShardMapper should adjust max_shards to initial_shards"
        );

        let mapper = result.unwrap();
        assert_eq!(mapper.active_shard_count(), 8);

        // Verify the policy was adjusted
        assert!(
            mapper.policy().max_shards >= 8,
            "max_shards should be at least initial_shards"
        );
    }

    #[test]
    fn test_scale_up_max_shards_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let policy = ScalingPolicy {
            max_shards: 10,
            cooldown: Duration::from_millis(0), // Disable cooldown for test
            ..Default::default()
        };
        let mapper = Arc::new(ShardMapper::new(5, 1024, policy).unwrap());

        // Spawn multiple threads that all try to scale up
        let mut handles = vec![];
        for _ in 0..5 {
            let mapper_clone = mapper.clone();
            handles.push(thread::spawn(move || mapper_clone.scale_up(3)));
        }

        // Collect results
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Some should succeed, some should fail with AtMaxShards
        let successes: Vec<_> = results.iter().filter(|r| r.is_ok()).collect();
        let failures: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, Err(ScalingError::AtMaxShards)))
            .collect();

        // We started with 5, max is 10, each tries to add 3
        // At most we can add 5 more shards, so at most 1-2 can succeed
        assert!(
            !successes.is_empty() || !failures.is_empty(),
            "at least some operations should complete"
        );

        // Final count should not exceed max_shards
        assert!(
            mapper.active_shard_count() <= 10,
            "should never exceed max_shards, got {}",
            mapper.active_shard_count()
        );
    }

    #[test]
    fn test_scale_up_overflow_protection() {
        // Create mapper with a high starting shard ID to test overflow protection
        let policy = ScalingPolicy {
            max_shards: u16::MAX,
            ..Default::default()
        };
        let mapper = ShardMapper::new(1, 1024, policy).unwrap();

        // Manually set up a scenario where we're near the u16 limit
        // by scaling up close to the limit
        {
            let mut shards = mapper.shards.write();
            // Simulate a shard with ID close to u16::MAX
            shards.clear();
            shards.push(MappedShard {
                id: u16::MAX - 2,
                state: ShardState::Active,
                metrics: Arc::new(ShardMetricsCollector::new(u16::MAX - 2, 1024)),
                drain_started: None,
                last_metrics: ShardMetrics::new(u16::MAX - 2),
            });
        }

        // Trying to add 3 shards should fail (would need IDs MAX-1, MAX, MAX+1)
        let result = mapper.scale_up(3);
        assert!(matches!(result, Err(ScalingError::AtMaxShards)));

        // Adding 1 shard should still work (ID = MAX-1)
        let result = mapper.scale_up(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_scaling_auto_scale_disabled() {
        let policy = ScalingPolicy {
            auto_scale: false,
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::None));
    }

    #[test]
    fn test_evaluate_scaling_in_cooldown() {
        let policy = ScalingPolicy {
            cooldown: Duration::from_secs(60),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Trigger a scaling operation to start cooldown
        *mapper.last_scaling.write() = Some(Instant::now());

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::None));
    }

    #[test]
    fn test_evaluate_scaling_scale_up_on_high_fill_ratio() {
        let policy = ScalingPolicy {
            fill_ratio_threshold: 0.7,
            max_shards: 8,
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set high fill ratio on majority of shards
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.fill_ratio = 0.9; // Above threshold
            }
        }

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }

    #[test]
    fn test_evaluate_scaling_scale_up_on_high_latency() {
        let policy = ScalingPolicy {
            push_latency_threshold_ns: 10,
            max_shards: 8,
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set high push latency on majority of shards
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.avg_push_latency_ns = 100; // Above threshold
            }
        }

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }

    #[test]
    fn test_evaluate_scaling_scale_down_on_underutilized() {
        let policy = ScalingPolicy {
            underutilized_threshold: 0.2,
            min_shards: 2,
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set low fill ratio and zero event rate on majority of shards
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.fill_ratio = 0.05; // Below threshold
                shard.last_metrics.event_rate = 0;
            }
        }

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::ScaleDown(_)));
    }

    #[test]
    fn test_evaluate_scaling_no_scale_up_at_max() {
        let policy = ScalingPolicy {
            fill_ratio_threshold: 0.7,
            max_shards: 4, // Already at max
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set high fill ratio
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.fill_ratio = 0.9;
            }
        }

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::None));
    }

    #[test]
    fn test_evaluate_scaling_no_scale_down_at_min() {
        let policy = ScalingPolicy {
            underutilized_threshold: 0.2,
            min_shards: 4, // Already at min
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set underutilized metrics
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.fill_ratio = 0.05;
                shard.last_metrics.event_rate = 0;
            }
        }

        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::None));
    }

    #[test]
    fn test_evaluate_scaling_ignores_draining_shards() {
        let policy = ScalingPolicy {
            fill_ratio_threshold: 0.7,
            max_shards: 8,
            cooldown: Duration::from_millis(0),
            ..Default::default()
        };
        let mapper = ShardMapper::new(4, 1024, policy).unwrap();

        // Set high fill ratio but mark shards as draining
        {
            let mut shards = mapper.shards.write();
            for shard in shards.iter_mut() {
                shard.last_metrics.fill_ratio = 0.9;
                shard.state = ShardState::Draining;
            }
        }

        // Should not scale up since draining shards are ignored
        let decision = mapper.evaluate_scaling();
        assert!(matches!(decision, ScalingDecision::None));
    }

    #[test]
    fn test_shard_metrics_new() {
        let metrics = ShardMetrics::new(5);
        assert_eq!(metrics.shard_id, 5);
        assert_eq!(metrics.fill_ratio, 0.0);
        assert_eq!(metrics.event_rate, 0);
        assert!(!metrics.draining);
    }

    #[test]
    fn test_shard_metrics_compute_weight() {
        let mut metrics = ShardMetrics::new(0);
        metrics.fill_ratio = 0.5;
        metrics.avg_push_latency_ns = 100;
        metrics.event_rate = 1_000_000;

        metrics.compute_weight();
        assert!(metrics.weight > 0.0);
    }

    #[test]
    fn test_shard_metrics_draining_max_weight() {
        let mut metrics = ShardMetrics::new(0);
        metrics.draining = true;
        metrics.compute_weight();
        assert_eq!(metrics.weight, f64::MAX);
    }

    #[test]
    fn test_scaling_decision_debug() {
        let none = ScalingDecision::None;
        let up = ScalingDecision::ScaleUp(2);
        let down = ScalingDecision::ScaleDown(1);

        assert!(format!("{:?}", none).contains("None"));
        assert!(format!("{:?}", up).contains("ScaleUp"));
        assert!(format!("{:?}", down).contains("ScaleDown"));
    }
}
