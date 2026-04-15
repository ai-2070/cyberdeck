//! Failure detection and recovery for Net.
//!
//! This module provides:
//! - `FailureDetector` - Heartbeat-based failure detection
//! - `LossSimulator` - Packet loss simulation for testing
//! - `RecoveryManager` - Route recovery and failover
//! - `CircuitBreaker` - Prevent cascading failures

use dashmap::DashMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Failure detector configuration
#[derive(Debug, Clone)]
pub struct FailureDetectorConfig {
    /// Heartbeat timeout before considering node failed
    pub timeout: Duration,
    /// Number of missed heartbeats before declaring failure
    pub miss_threshold: u32,
    /// Suspicion threshold (soft failure)
    pub suspicion_threshold: u32,
    /// Cleanup interval for stale entries
    pub cleanup_interval: Duration,
}

impl Default for FailureDetectorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            miss_threshold: 3,
            suspicion_threshold: 2,
            cleanup_interval: Duration::from_secs(30),
        }
    }
}

/// Node health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is healthy (receiving heartbeats)
    Healthy,
    /// Node is suspected (missed some heartbeats)
    Suspected,
    /// Node is considered failed
    Failed,
    /// Node status is unknown (never seen)
    Unknown,
}

/// Per-node failure tracking state
#[derive(Debug)]
struct NodeState {
    /// Last heartbeat timestamp
    last_heartbeat: Instant,
    /// Number of consecutive missed heartbeats
    missed_count: u32,
    /// Current status
    status: NodeStatus,
    /// Node address
    #[allow(dead_code)]
    addr: SocketAddr,
    /// Total heartbeats received
    total_heartbeats: u64,
    /// Time node was first seen
    #[allow(dead_code)]
    first_seen: Instant,
}

impl NodeState {
    fn new(addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            last_heartbeat: now,
            missed_count: 0,
            status: NodeStatus::Healthy,
            addr,
            total_heartbeats: 1,
            first_seen: now,
        }
    }

    fn on_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.missed_count = 0;
        self.status = NodeStatus::Healthy;
        self.total_heartbeats += 1;
    }

    fn check(&mut self, timeout: Duration, suspicion_threshold: u32, miss_threshold: u32) {
        let elapsed = self.last_heartbeat.elapsed();

        if elapsed > timeout {
            // Compute how many heartbeat intervals have been missed based on
            // actual elapsed time, not just how many times check() was called.
            // This prevents both under- and over-counting when check_all()
            // runs at a different frequency than the heartbeat interval.
            let timeout_nanos = timeout.as_nanos().max(1);
            self.missed_count = (elapsed.as_nanos() / timeout_nanos) as u32;

            if self.missed_count >= miss_threshold {
                self.status = NodeStatus::Failed;
            } else if self.missed_count >= suspicion_threshold {
                self.status = NodeStatus::Suspected;
            }
        }
    }
}

/// Failure detection statistics
#[derive(Debug, Clone, Default)]
pub struct FailureStats {
    /// Total nodes tracked
    pub nodes_tracked: usize,
    /// Healthy nodes
    pub nodes_healthy: usize,
    /// Suspected nodes
    pub nodes_suspected: usize,
    /// Failed nodes
    pub nodes_failed: usize,
    /// Total failures detected
    pub total_failures: u64,
    /// Total recoveries
    pub total_recoveries: u64,
}

/// Heartbeat-based failure detector.
///
/// Tracks node health via heartbeat messages and detects failures.
pub struct FailureDetector {
    /// Configuration
    config: FailureDetectorConfig,
    /// Per-node state
    nodes: DashMap<u64, NodeState>,
    /// Failure callback (node_id)
    on_failure: Option<Arc<dyn Fn(u64) + Send + Sync>>,
    /// Recovery callback (node_id)
    on_recovery: Option<Arc<dyn Fn(u64) + Send + Sync>>,
    /// Total failures detected
    total_failures: AtomicU64,
    /// Total recoveries
    total_recoveries: AtomicU64,
    /// Last cleanup time
    last_cleanup: std::sync::Mutex<Instant>,
}

impl FailureDetector {
    /// Create a new failure detector with default config
    pub fn new() -> Self {
        Self::with_config(FailureDetectorConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: FailureDetectorConfig) -> Self {
        Self {
            config,
            nodes: DashMap::new(),
            on_failure: None,
            on_recovery: None,
            total_failures: AtomicU64::new(0),
            total_recoveries: AtomicU64::new(0),
            last_cleanup: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Set failure callback
    pub fn on_failure<F>(mut self, f: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.on_failure = Some(Arc::new(f));
        self
    }

    /// Set recovery callback
    pub fn on_recovery<F>(mut self, f: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.on_recovery = Some(Arc::new(f));
        self
    }

    /// Record a heartbeat from a node
    pub fn heartbeat(&self, node_id: u64, addr: SocketAddr) {
        self.nodes
            .entry(node_id)
            .and_modify(|state| {
                let was_failed = state.status == NodeStatus::Failed;
                state.on_heartbeat();

                // Check for recovery
                if was_failed {
                    self.total_recoveries.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref cb) = self.on_recovery {
                        cb(node_id);
                    }
                }
            })
            .or_insert_with(|| NodeState::new(addr));
    }

    /// Check all nodes for failures
    pub fn check_all(&self) -> Vec<u64> {
        let mut newly_failed = Vec::new();

        for mut entry in self.nodes.iter_mut() {
            let prev_status = entry.status;
            entry.check(
                self.config.timeout,
                self.config.suspicion_threshold,
                self.config.miss_threshold,
            );

            // Detect new failures
            if entry.status == NodeStatus::Failed && prev_status != NodeStatus::Failed {
                newly_failed.push(*entry.key());
                self.total_failures.fetch_add(1, Ordering::Relaxed);

                if let Some(ref cb) = self.on_failure {
                    cb(*entry.key());
                }
            }
        }

        newly_failed
    }

    /// Get node status
    pub fn status(&self, node_id: u64) -> NodeStatus {
        self.nodes
            .get(&node_id)
            .map(|s| s.status)
            .unwrap_or(NodeStatus::Unknown)
    }

    /// Get all failed nodes
    pub fn failed_nodes(&self) -> Vec<u64> {
        self.nodes
            .iter()
            .filter(|r| r.status == NodeStatus::Failed)
            .map(|r| *r.key())
            .collect()
    }

    /// Get all suspected nodes
    pub fn suspected_nodes(&self) -> Vec<u64> {
        self.nodes
            .iter()
            .filter(|r| r.status == NodeStatus::Suspected)
            .map(|r| *r.key())
            .collect()
    }

    /// Get all healthy nodes
    pub fn healthy_nodes(&self) -> Vec<u64> {
        self.nodes
            .iter()
            .filter(|r| r.status == NodeStatus::Healthy)
            .map(|r| *r.key())
            .collect()
    }

    /// Remove a node from tracking
    pub fn remove(&self, node_id: u64) {
        self.nodes.remove(&node_id);
    }

    /// Clean up stale entries (nodes that have been failed for too long)
    pub fn cleanup(&self) -> usize {
        let mut last = self.last_cleanup.lock().unwrap();
        if last.elapsed() < self.config.cleanup_interval {
            return 0;
        }
        *last = Instant::now();
        drop(last);

        let stale_threshold = self.config.timeout * 10; // 10x timeout = stale
        let mut removed = 0;

        self.nodes.retain(|_, state| {
            if state.status == NodeStatus::Failed
                && state.last_heartbeat.elapsed() > stale_threshold
            {
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }

    /// Get statistics
    pub fn stats(&self) -> FailureStats {
        let mut healthy = 0;
        let mut suspected = 0;
        let mut failed = 0;

        for entry in self.nodes.iter() {
            match entry.status {
                NodeStatus::Healthy => healthy += 1,
                NodeStatus::Suspected => suspected += 1,
                NodeStatus::Failed => failed += 1,
                NodeStatus::Unknown => {}
            }
        }

        FailureStats {
            nodes_tracked: self.nodes.len(),
            nodes_healthy: healthy,
            nodes_suspected: suspected,
            nodes_failed: failed,
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_recoveries: self.total_recoveries.load(Ordering::Relaxed),
        }
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for FailureDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet loss simulator for testing.
///
/// Simulates various network failure conditions.
pub struct LossSimulator {
    /// Base loss rate (0.0 - 1.0)
    loss_rate: f32,
    /// Burst loss state
    in_burst: AtomicBool,
    /// Burst probability
    burst_prob: f32,
    /// Burst length (packets)
    burst_length: u32,
    /// Current burst remaining
    burst_remaining: AtomicU64,
    /// Random state (simple LCG)
    rng_state: AtomicU64,
    /// Total packets seen
    total_packets: AtomicU64,
    /// Total packets dropped
    total_dropped: AtomicU64,
}

impl LossSimulator {
    /// Create a new loss simulator with given loss rate
    pub fn new(loss_rate: f32) -> Self {
        Self {
            loss_rate: loss_rate.clamp(0.0, 1.0),
            in_burst: AtomicBool::new(false),
            burst_prob: 0.0,
            burst_length: 0,
            burst_remaining: AtomicU64::new(0),
            rng_state: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            ),
            total_packets: AtomicU64::new(0),
            total_dropped: AtomicU64::new(0),
        }
    }

    /// Create with burst loss behavior
    pub fn with_bursts(mut self, burst_prob: f32, burst_length: u32) -> Self {
        self.burst_prob = burst_prob.clamp(0.0, 1.0);
        self.burst_length = burst_length;
        self
    }

    /// Check if a packet should be dropped
    pub fn should_drop(&self) -> bool {
        self.total_packets.fetch_add(1, Ordering::Relaxed);

        // Check burst state
        let remaining = self.burst_remaining.load(Ordering::Relaxed);
        if remaining > 0 {
            self.burst_remaining.fetch_sub(1, Ordering::Relaxed);
            self.total_dropped.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        // Generate random value
        let r = self.next_random();

        // Check for burst start
        if self.burst_prob > 0.0 && r < self.burst_prob {
            // The triggering packet counts as the first drop in the burst,
            // so only burst_length - 1 additional packets remain.
            self.burst_remaining.store(
                self.burst_length.saturating_sub(1) as u64,
                Ordering::Relaxed,
            );
            self.in_burst.store(true, Ordering::Relaxed);
            self.total_dropped.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        // Normal loss
        if r < self.loss_rate {
            self.total_dropped.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        false
    }

    /// Get current effective loss rate
    pub fn effective_loss_rate(&self) -> f32 {
        let total = self.total_packets.load(Ordering::Relaxed);
        let dropped = self.total_dropped.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        dropped as f32 / total as f32
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.total_packets.store(0, Ordering::Relaxed);
        self.total_dropped.store(0, Ordering::Relaxed);
        self.burst_remaining.store(0, Ordering::Relaxed);
        self.in_burst.store(false, Ordering::Relaxed);
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.total_packets.load(Ordering::Relaxed),
            self.total_dropped.load(Ordering::Relaxed),
        )
    }

    // Simple LCG random number generator (0.0 - 1.0)
    fn next_random(&self) -> f32 {
        let mut state = self.rng_state.load(Ordering::Relaxed);
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.rng_state.store(state, Ordering::Relaxed);
        (state >> 33) as f32 / (1u64 << 31) as f32
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (blocking requests)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures.
pub struct CircuitBreaker {
    /// Current state
    state: std::sync::RwLock<CircuitState>,
    /// Failure count in current window
    failure_count: AtomicU64,
    /// Success count in current window
    success_count: AtomicU64,
    /// Failure threshold to trip
    failure_threshold: u64,
    /// Success threshold to close
    success_threshold: u64,
    /// Time to wait before half-open
    reset_timeout: Duration,
    /// Last state change time
    last_state_change: std::sync::Mutex<Instant>,
    /// Total trips
    total_trips: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u64, success_threshold: u64, reset_timeout: Duration) -> Self {
        Self {
            state: std::sync::RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_threshold,
            success_threshold,
            reset_timeout,
            last_state_change: std::sync::Mutex::new(Instant::now()),
            total_trips: AtomicU64::new(0),
        }
    }

    /// Check if request should be allowed
    pub fn allow(&self) -> bool {
        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                let last = self.last_state_change.lock().unwrap();
                if last.elapsed() >= self.reset_timeout {
                    drop(last);
                    self.transition_to(CircuitState::HalfOpen);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a success
    pub fn record_success(&self) {
        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.success_threshold {
                    self.transition_to(CircuitState::Closed);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failure
    pub fn record_failure(&self) {
        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.failure_threshold {
                    self.transition_to(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open trips back to open
                self.transition_to(CircuitState::Open);
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// Get total trip count
    pub fn total_trips(&self) -> u64 {
        self.total_trips.load(Ordering::Relaxed)
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        self.transition_to(CircuitState::Closed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    fn transition_to(&self, new_state: CircuitState) {
        let mut state = self.state.write().unwrap();
        let old_state = *state;

        if old_state != new_state {
            *state = new_state;
            *self.last_state_change.lock().unwrap() = Instant::now();

            // Reset counters on transition
            self.failure_count.store(0, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);

            // Track trips
            if new_state == CircuitState::Open {
                self.total_trips.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Recovery action for a failed node
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Reroute traffic through alternate path
    Reroute {
        /// Node IDs forming the alternate path
        via: Vec<u64>,
    },
    /// Retry with backoff
    Retry {
        /// Delay before retry in milliseconds
        delay_ms: u64,
    },
    /// Drop and notify
    Drop {
        /// Reason for dropping the message
        reason: String,
    },
    /// Queue for later delivery
    Queue,
}

/// Recovery statistics
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// Reroutes performed
    pub reroutes: u64,
    /// Retries performed
    pub retries: u64,
    /// Packets dropped
    pub dropped: u64,
    /// Packets queued
    pub queued: u64,
    /// Average recovery time (ms)
    pub avg_recovery_ms: u64,
}

/// Recovery manager for handling node failures.
pub struct RecoveryManager {
    /// Failed nodes and their recovery state
    failed_nodes: DashMap<u64, FailedNodeState>,
    /// Pending recovery queue
    recovery_queue: std::sync::Mutex<VecDeque<(u64, Instant)>>,
    /// Stats
    reroutes: AtomicU64,
    retries: AtomicU64,
    dropped: AtomicU64,
    queued: AtomicU64,
    total_recovery_time_ms: AtomicU64,
    recovery_count: AtomicU64,
}

#[derive(Debug)]
struct FailedNodeState {
    /// When failure was detected
    failed_at: Instant,
    /// Retry count
    retry_count: u32,
    /// Alternate routes
    alternates: Vec<u64>,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        Self {
            failed_nodes: DashMap::new(),
            recovery_queue: std::sync::Mutex::new(VecDeque::new()),
            reroutes: AtomicU64::new(0),
            retries: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            queued: AtomicU64::new(0),
            total_recovery_time_ms: AtomicU64::new(0),
            recovery_count: AtomicU64::new(0),
        }
    }

    /// Handle a node failure
    pub fn on_failure(&self, node_id: u64, alternates: Vec<u64>) -> RecoveryAction {
        self.failed_nodes.insert(
            node_id,
            FailedNodeState {
                failed_at: Instant::now(),
                retry_count: 0,
                alternates: alternates.clone(),
            },
        );

        if !alternates.is_empty() {
            self.reroutes.fetch_add(1, Ordering::Relaxed);
            RecoveryAction::Reroute { via: alternates }
        } else {
            self.queued.fetch_add(1, Ordering::Relaxed);
            self.recovery_queue
                .lock()
                .unwrap()
                .push_back((node_id, Instant::now()));
            RecoveryAction::Queue
        }
    }

    /// Handle node recovery
    pub fn on_recovery(&self, node_id: u64) {
        if let Some((_, state)) = self.failed_nodes.remove(&node_id) {
            let recovery_time = state.failed_at.elapsed().as_millis() as u64;
            self.total_recovery_time_ms
                .fetch_add(recovery_time, Ordering::Relaxed);
            self.recovery_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get recovery action for a node
    pub fn get_action(&self, node_id: u64, max_retries: u32) -> RecoveryAction {
        if let Some(mut state) = self.failed_nodes.get_mut(&node_id) {
            if !state.alternates.is_empty() {
                return RecoveryAction::Reroute {
                    via: state.alternates.clone(),
                };
            }

            if state.retry_count < max_retries {
                state.retry_count += 1;
                self.retries.fetch_add(1, Ordering::Relaxed);
                let delay = 100 * (1 << state.retry_count.min(6)); // Exponential backoff
                return RecoveryAction::Retry { delay_ms: delay };
            }

            self.dropped.fetch_add(1, Ordering::Relaxed);
            RecoveryAction::Drop {
                reason: "max retries exceeded".into(),
            }
        } else {
            // Node not in failed list, normal operation
            RecoveryAction::Retry { delay_ms: 0 }
        }
    }

    /// Check if a node is failed
    pub fn is_failed(&self, node_id: u64) -> bool {
        self.failed_nodes.contains_key(&node_id)
    }

    /// Get statistics
    pub fn stats(&self) -> RecoveryStats {
        let count = self.recovery_count.load(Ordering::Relaxed);
        let total_time = self.total_recovery_time_ms.load(Ordering::Relaxed);
        let avg = if count > 0 { total_time / count } else { 0 };

        RecoveryStats {
            reroutes: self.reroutes.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            queued: self.queued.load(Ordering::Relaxed),
            avg_recovery_ms: avg,
        }
    }

    /// Get failed node count
    pub fn failed_count(&self) -> usize {
        self.failed_nodes.len()
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_detector_basic() {
        let detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: Duration::from_millis(100),
            miss_threshold: 2,
            suspicion_threshold: 1,
            cleanup_interval: Duration::from_secs(60),
        });

        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        detector.heartbeat(0x1234, addr);

        assert_eq!(detector.status(0x1234), NodeStatus::Healthy);
        assert_eq!(detector.node_count(), 1);
    }

    #[test]
    fn test_failure_detector_failure() {
        let detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: Duration::from_millis(10),
            miss_threshold: 2,
            suspicion_threshold: 1,
            cleanup_interval: Duration::from_secs(60),
        });

        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        detector.heartbeat(0x1234, addr);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(15));

        // First check - should be suspected
        detector.check_all();
        assert_eq!(detector.status(0x1234), NodeStatus::Suspected);

        // Wait more
        std::thread::sleep(Duration::from_millis(15));

        // Second check - should be failed
        let failed = detector.check_all();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0], 0x1234);
        assert_eq!(detector.status(0x1234), NodeStatus::Failed);
    }

    #[test]
    fn test_failure_detector_recovery() {
        let detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: Duration::from_millis(10),
            miss_threshold: 1,
            suspicion_threshold: 1,
            cleanup_interval: Duration::from_secs(60),
        });

        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        detector.heartbeat(0x1234, addr);

        std::thread::sleep(Duration::from_millis(15));
        detector.check_all();
        assert_eq!(detector.status(0x1234), NodeStatus::Failed);

        // Recovery
        detector.heartbeat(0x1234, addr);
        assert_eq!(detector.status(0x1234), NodeStatus::Healthy);

        let stats = detector.stats();
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.total_recoveries, 1);
    }

    #[test]
    fn test_failure_detector_elapsed_based_missed_count() {
        // Regression: check() incremented missed_count by 1 per call regardless
        // of elapsed time. If check_all() ran infrequently, a node could stay
        // healthy much longer than the configured timeout. Now missed_count is
        // computed from elapsed / timeout.
        let detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: Duration::from_millis(10),
            miss_threshold: 3,
            suspicion_threshold: 2,
            cleanup_interval: Duration::from_secs(60),
        });

        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        detector.heartbeat(0x1234, addr);

        // Wait long enough that multiple timeouts have elapsed
        std::thread::sleep(Duration::from_millis(35));

        // A single check_all() call should detect the node as failed
        // because ~35ms / 10ms = 3 missed heartbeats >= miss_threshold(3).
        // With the old code (increment by 1), this would only be missed_count=1.
        let failed = detector.check_all();
        assert_eq!(
            detector.status(0x1234),
            NodeStatus::Failed,
            "node should be Failed after 3+ timeout intervals, even with one check call"
        );
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn test_loss_simulator() {
        let sim = LossSimulator::new(0.5);

        let mut dropped = 0;
        for _ in 0..1000 {
            if sim.should_drop() {
                dropped += 1;
            }
        }

        // Should be roughly 50% (allow wide margin for randomness)
        assert!(dropped > 300 && dropped < 700);
    }

    #[test]
    fn test_loss_simulator_burst() {
        let sim = LossSimulator::new(0.0).with_bursts(0.1, 5);

        let mut total_bursts = 0;
        let mut in_burst = false;
        for _ in 0..1000 {
            if sim.should_drop() {
                if !in_burst {
                    in_burst = true;
                    total_bursts += 1;
                }
            } else {
                in_burst = false;
            }
        }

        // Should have had some bursts
        assert!(total_bursts > 0);
    }

    #[test]
    fn test_burst_drops_exactly_burst_length_packets() {
        // Regression: a burst starting dropped the triggering packet AND then
        // burst_length more, for burst_length + 1 total drops per burst.
        //
        // We verify by directly inspecting burst_remaining after triggering.
        // With burst_prob = 1.0, the first call always starts a burst.
        let burst_len = 5u32;
        let sim = LossSimulator::new(0.0).with_bursts(1.0, burst_len);

        // First call: triggers burst, drops the triggering packet.
        assert!(sim.should_drop());
        // burst_remaining should be burst_length - 1 (since the trigger was the 1st drop)
        let remaining = sim.burst_remaining.load(Ordering::Relaxed);
        assert_eq!(
            remaining,
            (burst_len - 1) as u64,
            "after trigger, burst_remaining should be burst_length - 1, \
             not burst_length (which would cause burst_length + 1 total drops)"
        );

        // Drain the remaining burst
        for _ in 0..remaining {
            assert!(sim.should_drop());
        }

        // After exactly burst_length total drops, burst_remaining should be 0
        assert_eq!(sim.burst_remaining.load(Ordering::Relaxed), 0);
        assert_eq!(sim.total_dropped.load(Ordering::Relaxed), burst_len as u64);
    }

    #[test]
    fn test_circuit_breaker() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_millis(50));

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow());

        // Trip the breaker
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();

        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow());

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(60));

        // Should transition to half-open
        assert!(cb.allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Successes should close it
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_recovery_manager() {
        let mgr = RecoveryManager::new();

        // Failure with alternates
        let action = mgr.on_failure(0x1234, vec![0x5678, 0x9ABC]);
        match action {
            RecoveryAction::Reroute { via } => {
                assert_eq!(via, vec![0x5678, 0x9ABC]);
            }
            _ => panic!("expected reroute"),
        }

        // Failure without alternates
        let action = mgr.on_failure(0x2222, vec![]);
        match action {
            RecoveryAction::Queue => {}
            _ => panic!("expected queue"),
        }

        assert!(mgr.is_failed(0x1234));
        assert!(mgr.is_failed(0x2222));

        // Recovery
        mgr.on_recovery(0x1234);
        assert!(!mgr.is_failed(0x1234));

        let stats = mgr.stats();
        assert_eq!(stats.reroutes, 1);
        assert_eq!(stats.queued, 1);
    }
}
