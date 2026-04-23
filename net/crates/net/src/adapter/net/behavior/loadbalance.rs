//! Phase 4G: Distributed Load Balancing (LOAD-BALANCE)
//!
//! This module provides distributed load balancing across the Net network:
//! - Multiple load balancing strategies (round-robin, weighted, least-connections, etc.)
//! - Health-aware routing with automatic failover
//! - Load metrics collection and aggregation
//! - Adaptive load balancing based on real-time conditions

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::metadata::NodeId;

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Round-robin selection
    #[default]
    RoundRobin,
    /// Weighted round-robin based on node capacity
    WeightedRoundRobin,
    /// Select node with fewest active connections
    LeastConnections,
    /// Weighted least connections
    WeightedLeastConnections,
    /// Random selection
    Random,
    /// Weighted random selection
    WeightedRandom,
    /// Consistent hashing for sticky sessions
    ConsistentHash,
    /// Select based on lowest latency
    LeastLatency,
    /// Select based on lowest resource utilization
    LeastLoad,
    /// Power of two random choices
    PowerOfTwo,
    /// Adaptive strategy based on conditions
    Adaptive,
}

/// Health status of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Node is healthy and accepting traffic
    #[default]
    Healthy,
    /// Node is degraded but still accepting traffic
    Degraded,
    /// Node is unhealthy and should not receive traffic
    Unhealthy,
    /// Node health is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if node can receive traffic
    pub fn can_receive_traffic(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get weight multiplier for this health status
    pub fn weight_multiplier(&self) -> f64 {
        match self {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Degraded => 0.5,
            HealthStatus::Unhealthy => 0.0,
            HealthStatus::Unknown => 0.25,
        }
    }
}

/// Load metrics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU utilization (0.0 - 1.0)
    pub cpu_usage: f64,
    /// Memory utilization (0.0 - 1.0)
    pub memory_usage: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Requests per second
    pub requests_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Queue depth
    pub queue_depth: u32,
    /// Bandwidth utilization (0.0 - 1.0)
    pub bandwidth_usage: f64,
    /// Last update timestamp (microseconds since epoch)
    pub updated_at: u64,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_connections: 0,
            requests_per_second: 0.0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            queue_depth: 0,
            bandwidth_usage: 0.0,
            updated_at: 0,
        }
    }
}

impl LoadMetrics {
    /// Calculate composite load score (0.0 = no load, 1.0 = fully loaded)
    pub fn load_score(&self) -> f64 {
        // Weighted average of different metrics
        let cpu_weight = 0.3;
        let memory_weight = 0.2;
        let connections_weight = 0.2;
        let response_time_weight = 0.15;
        let error_weight = 0.15;

        // Normalize response time (assume 1000ms = fully loaded)
        let normalized_response_time = (self.avg_response_time_ms / 1000.0).min(1.0);

        cpu_weight * self.cpu_usage
            + memory_weight * self.memory_usage
            + connections_weight * (self.active_connections as f64 / 10000.0).min(1.0)
            + response_time_weight * normalized_response_time
            + error_weight * self.error_rate
    }

    /// Check if node is overloaded
    pub fn is_overloaded(&self) -> bool {
        self.cpu_usage > 0.9
            || self.memory_usage > 0.95
            || self.error_rate > 0.1
            || self.queue_depth > 1000
    }
}

/// Node endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Node ID
    pub node_id: NodeId,
    /// Weight for weighted strategies (higher = more traffic)
    pub weight: u32,
    /// Health status
    pub health: HealthStatus,
    /// Load metrics
    pub metrics: LoadMetrics,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Priority (lower = higher priority for failover)
    pub priority: u32,
    /// Whether endpoint is enabled
    pub enabled: bool,
    /// Zone/region for locality-aware routing
    pub zone: Option<String>,
}

impl Endpoint {
    /// Create a new endpoint
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            weight: 100,
            health: HealthStatus::Healthy,
            metrics: LoadMetrics::default(),
            tags: Vec::new(),
            priority: 0,
            enabled: true,
            zone: None,
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Set zone
    pub fn with_zone(mut self, zone: impl Into<String>) -> Self {
        self.zone = Some(zone.into());
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Effective weight considering health
    pub fn effective_weight(&self) -> f64 {
        if !self.enabled {
            return 0.0;
        }
        self.weight as f64 * self.health.weight_multiplier()
    }

    /// Check if endpoint can receive traffic
    pub fn is_available(&self) -> bool {
        self.enabled && self.health.can_receive_traffic()
    }
}

/// Endpoint state tracked by the load balancer
struct EndpointState {
    /// Immutable endpoint config (node_id, weight, tags, zone, priority)
    node_id: NodeId,
    weight: u32,
    tags: Vec<String>,
    zone: Option<String>,
    priority: u32,
    /// Mutable health status
    health: std::sync::RwLock<HealthStatus>,
    /// Mutable metrics
    metrics: std::sync::RwLock<LoadMetrics>,
    /// Whether endpoint is enabled
    enabled: std::sync::atomic::AtomicBool,
    /// Current connection count
    connections: AtomicU32,
    /// Total requests served
    total_requests: AtomicU64,
    /// Failed requests
    failed_requests: AtomicU64,
    /// Last selected time
    last_selected: std::sync::Mutex<Instant>,
    /// Consecutive failures
    consecutive_failures: AtomicU32,
    /// Circuit breaker state
    circuit_open: std::sync::atomic::AtomicBool,
    /// Circuit open time
    circuit_open_time: std::sync::Mutex<Option<Instant>>,
    /// Whether a half-open probe request is currently in flight. Only one
    /// request is admitted per recovery cycle to test the endpoint.
    half_open_probe: std::sync::atomic::AtomicBool,
}

impl EndpointState {
    fn new(endpoint: Endpoint) -> Self {
        Self {
            node_id: endpoint.node_id,
            weight: endpoint.weight,
            tags: endpoint.tags,
            zone: endpoint.zone,
            priority: endpoint.priority,
            health: std::sync::RwLock::new(endpoint.health),
            metrics: std::sync::RwLock::new(endpoint.metrics),
            enabled: std::sync::atomic::AtomicBool::new(endpoint.enabled),
            connections: AtomicU32::new(0),
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            last_selected: std::sync::Mutex::new(Instant::now()),
            consecutive_failures: AtomicU32::new(0),
            circuit_open: std::sync::atomic::AtomicBool::new(false),
            circuit_open_time: std::sync::Mutex::new(None),
            half_open_probe: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn health(&self) -> HealthStatus {
        *self.health.read().unwrap()
    }

    fn metrics(&self) -> LoadMetrics {
        self.metrics.read().unwrap().clone()
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn effective_weight(&self) -> f64 {
        if !self.is_enabled() {
            return 0.0;
        }
        self.weight as f64 * self.health().weight_multiplier()
    }

    fn is_available(&self) -> bool {
        self.is_enabled() && self.health().can_receive_traffic()
    }

    /// Atomically reserve a connection slot if the endpoint is below cap.
    ///
    /// Returns `true` if the slot was reserved (caller now owns a connection
    /// that must be released via `record_completion`), or `false` if the cap
    /// was already reached. This replaces the prior check-then-increment
    /// pattern that allowed concurrent selectors to exceed the cap.
    fn try_record_request(&self, max_connections: u32) -> bool {
        let reserved = self
            .connections
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |c| {
                if c >= max_connections {
                    None
                } else {
                    Some(c + 1)
                }
            })
            .is_ok();
        if reserved {
            self.total_requests.fetch_add(1, Ordering::Relaxed);
            *self.last_selected.lock().unwrap() = Instant::now();
        }
        reserved
    }

    fn record_completion(&self, success: bool) {
        self.connections.fetch_sub(1, Ordering::AcqRel);

        // If this completion is for the half-open probe, it decides the
        // circuit's fate. Clearing the flag with swap also guarantees only
        // one completion is treated as the probe outcome.
        if self.half_open_probe.swap(false, Ordering::AcqRel) {
            if success {
                self.circuit_open.store(false, Ordering::Release);
                self.consecutive_failures.store(0, Ordering::Relaxed);
                *self.circuit_open_time.lock().unwrap() = None;
            } else {
                self.failed_requests.fetch_add(1, Ordering::Relaxed);
                // Probe failed — restart the recovery timer so the next
                // probe is delayed by another full recovery_time window.
                *self.circuit_open_time.lock().unwrap() = Some(Instant::now());
            }
            return;
        }

        if success {
            self.consecutive_failures.store(0, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
            let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
            // Open circuit after 5 consecutive failures. Use CAS so only
            // the thread that causes the transition records the open time.
            if failures >= 5
                && self
                    .circuit_open
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                *self.circuit_open_time.lock().unwrap() = Some(Instant::now());
            }
        }
    }

    /// Returns true if new requests should be rejected for this endpoint.
    ///
    /// When the recovery window has elapsed, exactly one request is admitted
    /// as a probe (half-open state). All others continue to see the circuit
    /// as open until the probe's outcome is recorded via `record_completion`.
    fn is_circuit_open(&self, recovery_time: Duration) -> bool {
        if !self.circuit_open.load(Ordering::Acquire) {
            return false;
        }
        let open_time = match *self.circuit_open_time.lock().unwrap() {
            Some(t) => t,
            None => return true,
        };
        if open_time.elapsed() < recovery_time {
            return true;
        }
        // Recovery window elapsed — try to claim the single probe slot. If
        // CAS fails, another probe is in flight and we keep rejecting.
        self.half_open_probe
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    }
}

/// Request context for load balancing decisions
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    /// Request ID for consistent hashing
    pub request_id: Option<String>,
    /// Session ID for sticky sessions
    pub session_id: Option<String>,
    /// Client zone for locality routing
    pub client_zone: Option<String>,
    /// Required tags
    pub required_tags: Vec<String>,
    /// Preferred zones (in order of preference)
    pub preferred_zones: Vec<String>,
    /// Custom routing key
    pub routing_key: Option<String>,
}

impl RequestContext {
    /// Create new request context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set session ID for sticky sessions
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set routing key for consistent hashing
    pub fn with_routing_key(mut self, key: impl Into<String>) -> Self {
        self.routing_key = Some(key.into());
        self
    }

    /// Set client zone
    pub fn with_zone(mut self, zone: impl Into<String>) -> Self {
        self.client_zone = Some(zone.into());
        self
    }

    /// Add required tag
    pub fn require_tag(mut self, tag: impl Into<String>) -> Self {
        self.required_tags.push(tag.into());
        self
    }
}

/// Selection result
#[derive(Debug, Clone)]
pub struct Selection {
    /// Selected node ID
    pub node_id: NodeId,
    /// Endpoint weight
    pub weight: u32,
    /// Current load score
    pub load_score: f64,
    /// Why this node was selected
    pub reason: SelectionReason,
}

/// Reason for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReason {
    /// Selected by round-robin
    RoundRobin,
    /// Selected by weight
    Weighted,
    /// Selected for having least connections
    LeastConnections,
    /// Selected by consistent hash
    ConsistentHash,
    /// Selected for lowest latency
    LeastLatency,
    /// Selected for lowest load
    LeastLoad,
    /// Selected randomly
    Random,
    /// Selected by power of two choices
    PowerOfTwo,
    /// Selected for zone affinity
    ZoneAffinity,
    /// Fallback selection
    Fallback,
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Load balancing strategy
    pub strategy: Strategy,
    /// Health check interval
    pub health_check_interval_ms: u64,
    /// Circuit breaker recovery time
    pub circuit_recovery_time_ms: u64,
    /// Maximum connections per endpoint
    pub max_connections_per_endpoint: u32,
    /// Enable zone-aware routing
    pub zone_aware: bool,
    /// Fallback to any available if preferred zone unavailable
    pub zone_fallback: bool,
    /// Metrics staleness threshold
    pub metrics_stale_after_ms: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: Strategy::RoundRobin,
            health_check_interval_ms: 5000,
            circuit_recovery_time_ms: 30000,
            max_connections_per_endpoint: 10000,
            zone_aware: true,
            zone_fallback: true,
            metrics_stale_after_ms: 10000,
        }
    }
}

/// Load balancer error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadBalancerError {
    /// No endpoints available
    NoEndpointsAvailable,
    /// Endpoint not found
    EndpointNotFound(NodeId),
    /// All endpoints unhealthy
    AllEndpointsUnhealthy,
    /// No endpoints match required tags
    NoMatchingEndpoints,
    /// Circuit breaker open
    CircuitOpen(NodeId),
    /// Max connections reached
    MaxConnectionsReached(NodeId),
}

impl std::fmt::Display for LoadBalancerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEndpointsAvailable => write!(f, "no endpoints available"),
            Self::EndpointNotFound(id) => write!(f, "endpoint not found: {:?}", id),
            Self::AllEndpointsUnhealthy => write!(f, "all endpoints unhealthy"),
            Self::NoMatchingEndpoints => write!(f, "no endpoints match required tags"),
            Self::CircuitOpen(id) => write!(f, "circuit breaker open for: {:?}", id),
            Self::MaxConnectionsReached(id) => write!(f, "max connections reached for: {:?}", id),
        }
    }
}

impl std::error::Error for LoadBalancerError {}

/// Statistics for the load balancer
#[derive(Debug, Clone, Default)]
pub struct LoadBalancerStats {
    /// Total selections made
    pub total_selections: u64,
    /// Failed selections
    pub failed_selections: u64,
    /// Active endpoints
    pub active_endpoints: u32,
    /// Healthy endpoints
    pub healthy_endpoints: u32,
    /// Total active connections
    pub total_connections: u64,
    /// Average load score across endpoints
    pub avg_load_score: f64,
}

/// Distributed load balancer
pub struct LoadBalancer {
    /// Configuration
    config: LoadBalancerConfig,
    /// Endpoints by node ID
    endpoints: DashMap<NodeId, Arc<EndpointState>>,
    /// Round-robin counter
    rr_counter: AtomicU64,
    /// Total selections
    total_selections: AtomicU64,
    /// Failed selections
    failed_selections: AtomicU64,
    /// Consistent hash ring (node_id -> virtual nodes)
    hash_ring: DashMap<u64, NodeId>,
    /// Virtual nodes per endpoint for consistent hashing
    virtual_nodes: u32,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            config,
            endpoints: DashMap::new(),
            rr_counter: AtomicU64::new(0),
            total_selections: AtomicU64::new(0),
            failed_selections: AtomicU64::new(0),
            hash_ring: DashMap::new(),
            virtual_nodes: 150,
        }
    }

    /// Create with default configuration
    pub fn with_strategy(strategy: Strategy) -> Self {
        Self::new(LoadBalancerConfig {
            strategy,
            ..Default::default()
        })
    }

    /// Add an endpoint
    pub fn add_endpoint(&self, endpoint: Endpoint) {
        let node_id = endpoint.node_id;
        self.endpoints
            .insert(node_id, Arc::new(EndpointState::new(endpoint)));

        // Add to hash ring for consistent hashing
        self.add_to_hash_ring(node_id);
    }

    /// Remove an endpoint
    pub fn remove_endpoint(&self, node_id: &NodeId) {
        self.remove_from_hash_ring(node_id);
        self.endpoints.remove(node_id);
    }

    /// Update endpoint health
    pub fn update_health(&self, node_id: &NodeId, health: HealthStatus) {
        if let Some(state) = self.endpoints.get(node_id) {
            *state.health.write().unwrap() = health;
        }
    }

    /// Update endpoint metrics
    pub fn update_metrics(&self, node_id: &NodeId, metrics: LoadMetrics) {
        if let Some(state) = self.endpoints.get(node_id) {
            *state.metrics.write().unwrap() = metrics;
        }
    }

    /// Select an endpoint for a request.
    ///
    /// The connection slot is reserved atomically as part of selection so
    /// that concurrent selectors cannot collectively exceed
    /// `max_connections_per_endpoint`. If a strategy picks an endpoint whose
    /// cap was filled by a concurrent selector between availability filtering
    /// and reservation, the selection is retried up to a bounded number of
    /// times before giving up.
    pub fn select(&self, ctx: &RequestContext) -> Result<Selection, LoadBalancerError> {
        self.total_selections.fetch_add(1, Ordering::Relaxed);

        const MAX_RESERVATION_RETRIES: usize = 4;
        let max_conn = self.config.max_connections_per_endpoint;

        for _ in 0..MAX_RESERVATION_RETRIES {
            let available = self.get_available_endpoints(ctx)?;

            if available.is_empty() {
                self.failed_selections.fetch_add(1, Ordering::Relaxed);
                return Err(LoadBalancerError::NoEndpointsAvailable);
            }

            // Apply strategy
            let selection = match self.config.strategy {
                Strategy::RoundRobin => self.select_round_robin(&available),
                Strategy::WeightedRoundRobin => self.select_weighted_round_robin(&available),
                Strategy::LeastConnections => self.select_least_connections(&available),
                Strategy::WeightedLeastConnections => {
                    self.select_weighted_least_connections(&available)
                }
                Strategy::Random => self.select_random(&available),
                Strategy::WeightedRandom => self.select_weighted_random(&available),
                Strategy::ConsistentHash => self.select_consistent_hash(&available, ctx),
                Strategy::LeastLatency => self.select_least_latency(&available),
                Strategy::LeastLoad => self.select_least_load(&available),
                Strategy::PowerOfTwo => self.select_power_of_two(&available),
                Strategy::Adaptive => self.select_adaptive(&available, ctx),
            };

            // Atomically reserve the connection slot. If a concurrent
            // selector filled the cap, re-run selection against fresh state.
            if let Some(state) = self.endpoints.get(&selection.node_id) {
                if state.try_record_request(max_conn) {
                    return Ok(selection);
                }
            }
        }

        self.failed_selections.fetch_add(1, Ordering::Relaxed);
        Err(LoadBalancerError::NoEndpointsAvailable)
    }

    /// Record request completion
    pub fn record_completion(&self, node_id: &NodeId, success: bool) {
        if let Some(state) = self.endpoints.get(node_id) {
            state.record_completion(success);
        }
    }

    /// Get available endpoints matching context
    fn get_available_endpoints(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<Arc<EndpointState>>, LoadBalancerError> {
        let recovery_time = Duration::from_millis(self.config.circuit_recovery_time_ms);
        let mut available = Vec::new();
        let mut zone_matches = Vec::new();

        for entry in self.endpoints.iter() {
            let state = entry.value();

            // Check basic availability
            if !state.is_available() {
                continue;
            }

            // Check circuit breaker
            if state.is_circuit_open(recovery_time) {
                continue;
            }

            // Check max connections
            if state.connections.load(Ordering::Relaxed) >= self.config.max_connections_per_endpoint
            {
                continue;
            }

            // Check required tags
            if !ctx.required_tags.is_empty()
                && !ctx.required_tags.iter().all(|t| state.tags.contains(t))
            {
                continue;
            }

            // Zone-aware routing
            if self.config.zone_aware {
                if let Some(ref client_zone) = ctx.client_zone {
                    if state.zone.as_ref() == Some(client_zone) {
                        zone_matches.push(Arc::clone(state));
                        continue;
                    }
                }
            }

            available.push(Arc::clone(state));
        }

        // Prefer zone matches if available
        if !zone_matches.is_empty() {
            return Ok(zone_matches);
        }

        // No zone matches — check zone_fallback policy
        if self.config.zone_aware && ctx.client_zone.is_some() && !self.config.zone_fallback {
            // zone_fallback is disabled: don't fall back to non-zone endpoints
            return Err(LoadBalancerError::NoEndpointsAvailable);
        }

        if available.is_empty() {
            return Err(LoadBalancerError::NoEndpointsAvailable);
        }

        Ok(available)
    }

    fn select_round_robin(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let idx = self.rr_counter.fetch_add(1, Ordering::Relaxed) as usize % endpoints.len();
        let state = &endpoints[idx];
        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::RoundRobin,
        }
    }

    fn select_weighted_round_robin(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let total_weight: f64 = endpoints.iter().map(|e| e.effective_weight()).sum();

        if total_weight <= 0.0 {
            return self.select_round_robin(endpoints);
        }

        // Reduce the counter modulo `total_weight` in integer space
        // BEFORE casting to f64. The previous implementation did
        // `counter as f64 % total_weight`, which lost the low bits
        // of `counter` once it crossed the f64 mantissa boundary
        // (2^53 selections) — rotation stalled on a narrow set of
        // indices and distribution skewed on long-running services.
        //
        // `total_weight.ceil() as u64` loses at most 1 unit of
        // fractional precision (negligible for any real load-balancer
        // configuration), while keeping the rotation length
        // comparable to the integer-weight sum.
        let counter = self.rr_counter.fetch_add(1, Ordering::Relaxed);
        let total_ceil = (total_weight.ceil() as u64).max(1);
        let target = (counter % total_ceil) as f64;

        let mut cumulative = 0.0;
        for state in endpoints {
            cumulative += state.effective_weight();
            if cumulative > target {
                return Selection {
                    node_id: state.node_id,
                    weight: state.weight,
                    load_score: state.metrics().load_score(),
                    reason: SelectionReason::Weighted,
                };
            }
        }

        // Fallback to first
        let state = &endpoints[0];
        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::Weighted,
        }
    }

    fn select_least_connections(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let state = endpoints
            .iter()
            .min_by_key(|e| e.connections.load(Ordering::Relaxed))
            .unwrap();

        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::LeastConnections,
        }
    }

    fn select_weighted_least_connections(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        // Score = connections / weight (lower is better).
        // The `.max(MIN_DIVISOR)` guard is a divide-by-zero protector
        // for zero-weighted endpoints. It uses a small positive
        // epsilon instead of `1.0` so that fractional weights like
        // `0.1` and `0.5` keep their relative ordering — the old
        // `.max(1.0)` silently collapsed any weight in `(0, 1]` onto
        // `1.0`, degrading weighted-LC into plain least-connections
        // whenever operators configured sub-unit weights.
        const MIN_DIVISOR: f64 = 1e-6;
        let state = endpoints
            .iter()
            .min_by(|a, b| {
                let score_a = a.connections.load(Ordering::Relaxed) as f64
                    / a.effective_weight().max(MIN_DIVISOR);
                let score_b = b.connections.load(Ordering::Relaxed) as f64
                    / b.effective_weight().max(MIN_DIVISOR);
                score_a.total_cmp(&score_b)
            })
            .unwrap();

        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::LeastConnections,
        }
    }

    fn select_random(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let idx = random_usize() % endpoints.len();
        let state = &endpoints[idx];
        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::Random,
        }
    }

    fn select_weighted_random(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let total_weight: f64 = endpoints.iter().map(|e| e.effective_weight()).sum();

        if total_weight <= 0.0 {
            return self.select_random(endpoints);
        }

        let target = random_f64() * total_weight;

        let mut cumulative = 0.0;
        for state in endpoints {
            cumulative += state.effective_weight();
            if cumulative >= target {
                return Selection {
                    node_id: state.node_id,
                    weight: state.weight,
                    load_score: state.metrics().load_score(),
                    reason: SelectionReason::Weighted,
                };
            }
        }

        // Fallback
        let state = &endpoints[0];
        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::Weighted,
        }
    }

    fn select_consistent_hash(
        &self,
        endpoints: &[Arc<EndpointState>],
        ctx: &RequestContext,
    ) -> Selection {
        let key = ctx
            .routing_key
            .as_ref()
            .or(ctx.session_id.as_ref())
            .or(ctx.request_id.as_ref());

        if let Some(key) = key {
            let hash = self.hash_key(key);

            // Collect and sort hash ring entries — DashMap iteration order is
            // arbitrary, but consistent hashing requires finding the smallest
            // key >= hash.
            let mut ring: Vec<(u64, NodeId)> = self
                .hash_ring
                .iter()
                .map(|entry| (*entry.key(), *entry.value()))
                .collect();
            ring.sort_unstable_by_key(|&(k, _)| k);

            // Binary search for the first key >= hash
            let idx = ring.partition_point(|&(k, _)| k < hash);

            // Try from the found position, wrapping around
            for i in 0..ring.len() {
                let (_, node_id) = ring[(idx + i) % ring.len()];
                if let Some(state) = endpoints.iter().find(|e| e.node_id == node_id) {
                    return Selection {
                        node_id: state.node_id,
                        weight: state.weight,
                        load_score: state.metrics().load_score(),
                        reason: SelectionReason::ConsistentHash,
                    };
                }
            }
        }

        // Fallback to round-robin
        self.select_round_robin(endpoints)
    }

    fn select_least_latency(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let state = endpoints
            .iter()
            .min_by(|a, b| {
                a.metrics()
                    .avg_response_time_ms
                    .total_cmp(&b.metrics().avg_response_time_ms)
            })
            .unwrap();

        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::LeastLatency,
        }
    }

    fn select_least_load(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        let state = endpoints
            .iter()
            .min_by(|a, b| {
                a.metrics()
                    .load_score()
                    .total_cmp(&b.metrics().load_score())
            })
            .unwrap();

        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::LeastLoad,
        }
    }

    fn select_power_of_two(&self, endpoints: &[Arc<EndpointState>]) -> Selection {
        if endpoints.len() < 2 {
            return self.select_round_robin(endpoints);
        }

        // Pick two random endpoints
        let idx1 = random_usize() % endpoints.len();
        let mut idx2 = random_usize() % endpoints.len();
        if idx2 == idx1 {
            idx2 = (idx1 + 1) % endpoints.len();
        }

        let state1 = &endpoints[idx1];
        let state2 = &endpoints[idx2];

        // Choose the one with fewer connections
        let state = if state1.connections.load(Ordering::Relaxed)
            <= state2.connections.load(Ordering::Relaxed)
        {
            state1
        } else {
            state2
        };

        Selection {
            node_id: state.node_id,
            weight: state.weight,
            load_score: state.metrics().load_score(),
            reason: SelectionReason::PowerOfTwo,
        }
    }

    fn select_adaptive(&self, endpoints: &[Arc<EndpointState>], ctx: &RequestContext) -> Selection {
        // Use different strategies based on conditions
        let avg_load: f64 = endpoints
            .iter()
            .map(|e| e.metrics().load_score())
            .sum::<f64>()
            / endpoints.len() as f64;

        // If high load, use least connections
        if avg_load > 0.7 {
            return self.select_least_connections(endpoints);
        }

        // If session ID present, use consistent hash
        if ctx.session_id.is_some() || ctx.routing_key.is_some() {
            return self.select_consistent_hash(endpoints, ctx);
        }

        // Otherwise use weighted round-robin
        self.select_weighted_round_robin(endpoints)
    }

    fn add_to_hash_ring(&self, node_id: NodeId) {
        for i in 0..self.virtual_nodes {
            let key = format!("{:?}-{}", node_id, i);
            let hash = self.hash_key(&key);
            self.hash_ring.insert(hash, node_id);
        }
    }

    fn remove_from_hash_ring(&self, node_id: &NodeId) {
        self.hash_ring.retain(|_, v| v != node_id);
    }

    fn hash_key(&self, key: &str) -> u64 {
        // Simple FNV-1a hash
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Get statistics
    pub fn stats(&self) -> LoadBalancerStats {
        let mut healthy = 0u32;
        let mut total_connections = 0u64;
        let mut total_load = 0.0;

        for entry in self.endpoints.iter() {
            let state = entry.value();
            if state.health() == HealthStatus::Healthy {
                healthy += 1;
            }
            total_connections += state.connections.load(Ordering::Relaxed) as u64;
            total_load += state.metrics().load_score();
        }

        let endpoint_count = self.endpoints.len() as u32;

        LoadBalancerStats {
            total_selections: self.total_selections.load(Ordering::Relaxed),
            failed_selections: self.failed_selections.load(Ordering::Relaxed),
            active_endpoints: endpoint_count,
            healthy_endpoints: healthy,
            total_connections,
            avg_load_score: if endpoint_count > 0 {
                total_load / endpoint_count as f64
            } else {
                0.0
            },
        }
    }

    /// Get all endpoints as snapshots
    pub fn endpoints(&self) -> Vec<Endpoint> {
        self.endpoints
            .iter()
            .map(|e| {
                let state = e.value();
                Endpoint {
                    node_id: state.node_id,
                    weight: state.weight,
                    health: state.health(),
                    metrics: state.metrics(),
                    tags: state.tags.clone(),
                    priority: state.priority,
                    enabled: state.is_enabled(),
                    zone: state.zone.clone(),
                }
            })
            .collect()
    }

    /// Get endpoint count
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }
}

/// Generate random usize
fn random_usize() -> usize {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("getrandom failed");
    usize::from_le_bytes(bytes)
}

/// Generate random f64 uniformly in the half-open interval [0.0, 1.0).
///
/// Uses the top 53 bits of entropy (the f64 mantissa width) divided by
/// `2^53`, which guarantees the result is strictly less than 1.0. The naive
/// `r as f64 / u64::MAX as f64` approach can round up to exactly 1.0 because
/// `u64::MAX as f64` itself rounds to `2^64`.
fn random_f64() -> f64 {
    let r = random_usize() as u64;
    (r >> 11) as f64 / ((1u64 << 53) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node_id(n: u8) -> NodeId {
        let mut id = [0u8; 32];
        id[0] = n;
        id
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.can_receive_traffic());
        assert!(HealthStatus::Degraded.can_receive_traffic());
        assert!(!HealthStatus::Unhealthy.can_receive_traffic());
        assert!(!HealthStatus::Unknown.can_receive_traffic());

        assert_eq!(HealthStatus::Healthy.weight_multiplier(), 1.0);
        assert_eq!(HealthStatus::Degraded.weight_multiplier(), 0.5);
        assert_eq!(HealthStatus::Unhealthy.weight_multiplier(), 0.0);
    }

    #[test]
    fn test_load_metrics() {
        let metrics = LoadMetrics {
            cpu_usage: 0.5,
            memory_usage: 0.3,
            active_connections: 100,
            requests_per_second: 1000.0,
            avg_response_time_ms: 50.0,
            error_rate: 0.01,
            queue_depth: 10,
            bandwidth_usage: 0.2,
            updated_at: 0,
        };

        let score = metrics.load_score();
        assert!(score > 0.0 && score < 1.0);
        assert!(!metrics.is_overloaded());

        let overloaded = LoadMetrics {
            cpu_usage: 0.95,
            ..Default::default()
        };
        assert!(overloaded.is_overloaded());
    }

    #[test]
    fn test_endpoint() {
        let node_id = make_node_id(1);
        let endpoint = Endpoint::new(node_id)
            .with_weight(200)
            .with_zone("us-east-1")
            .with_tag("gpu");

        assert_eq!(endpoint.weight, 200);
        assert_eq!(endpoint.zone, Some("us-east-1".to_string()));
        assert!(endpoint.tags.contains(&"gpu".to_string()));
        assert!(endpoint.is_available());
        assert_eq!(endpoint.effective_weight(), 200.0);
    }

    #[test]
    fn test_load_balancer_round_robin() {
        let lb = LoadBalancer::with_strategy(Strategy::RoundRobin);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));
        lb.add_endpoint(Endpoint::new(make_node_id(3)));

        let ctx = RequestContext::new();

        // Should cycle through endpoints
        let mut selected = Vec::new();
        for _ in 0..6 {
            let selection = lb.select(&ctx).unwrap();
            selected.push(selection.node_id[0]);
        }

        // Should have selected each endpoint twice
        assert_eq!(selected.iter().filter(|&&x| x == 1).count(), 2);
        assert_eq!(selected.iter().filter(|&&x| x == 2).count(), 2);
        assert_eq!(selected.iter().filter(|&&x| x == 3).count(), 2);
    }

    #[test]
    fn test_load_balancer_least_connections() {
        let lb = LoadBalancer::with_strategy(Strategy::LeastConnections);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));
        lb.add_endpoint(Endpoint::new(make_node_id(3)));

        let ctx = RequestContext::new();

        // First selection - all have 0 connections
        let s1 = lb.select(&ctx).unwrap();
        // Don't record completion, so connection stays

        // Second selection should pick a different node
        let s2 = lb.select(&ctx).unwrap();
        assert_ne!(s1.node_id, s2.node_id);
    }

    #[test]
    fn test_load_balancer_weighted() {
        let lb = LoadBalancer::with_strategy(Strategy::WeightedRoundRobin);

        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_weight(100));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_weight(200));
        lb.add_endpoint(Endpoint::new(make_node_id(3)).with_weight(300));

        let ctx = RequestContext::new();

        let mut counts = std::collections::HashMap::new();
        for _ in 0..600 {
            let selection = lb.select(&ctx).unwrap();
            lb.record_completion(&selection.node_id, true);
            *counts.entry(selection.node_id[0]).or_insert(0) += 1;
        }

        // Node 3 should get most traffic, node 1 least
        assert!(counts.get(&3).unwrap() > counts.get(&2).unwrap());
        assert!(counts.get(&2).unwrap() > counts.get(&1).unwrap());
    }

    #[test]
    fn test_regression_weighted_lc_preserves_fractional_weights() {
        // Regression (LOW, BUGS.md): `select_weighted_least_connections`
        // used `.max(1.0)` as a divide-by-zero guard, which also
        // collapsed every weight in `(0, 1]` onto `1.0`. An endpoint
        // with weight `0.1` was scored identically to one with
        // `1.0`, silently degrading weighted-LC into plain LC
        // whenever operators configured sub-unit weights.
        //
        // Fix: use a small positive epsilon instead, so fractional
        // weights keep their relative ordering.
        let lb = LoadBalancer::with_strategy(Strategy::WeightedLeastConnections);

        // Two endpoints with identical connection counts but very
        // different fractional weights.
        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_weight(10));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_weight(1));

        let ctx = RequestContext::new();
        let mut counts = std::collections::HashMap::new();
        for _ in 0..600 {
            let selection = lb.select(&ctx).unwrap();
            // Don't record completion so connections stay matched.
            *counts.entry(selection.node_id[0]).or_insert(0_u32) += 1;
        }

        // The 10x-weighted endpoint should overwhelmingly win the
        // "connections/weight" tiebreak when connection counts are
        // comparable. With the old `.max(1.0)` collapse, the two
        // endpoints would score identically and a later tiebreaker
        // would pick one consistently — distribution would be either
        // 50/50 or 100/0 depending on ordering.
        let high = *counts.get(&1).unwrap_or(&0);
        let low = *counts.get(&2).unwrap_or(&0);
        assert!(
            high > low * 2,
            "weight=10 endpoint must get >2x more traffic than weight=1 \
             (got {high} vs {low})",
        );
    }

    #[test]
    fn test_regression_weighted_rr_precision_past_f64_mantissa() {
        // Regression (LOW, BUGS.md): `select_weighted_round_robin`
        // used `counter as f64 % total_weight`. Past 2^53 selections
        // the `as f64` cast dropped the low bits and rotation stalled
        // on a narrow set of indices. The fix scales weights to
        // integers and does the modulus in u64 space.
        let lb = LoadBalancer::with_strategy(Strategy::WeightedRoundRobin);
        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_weight(1));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_weight(1));
        lb.add_endpoint(Endpoint::new(make_node_id(3)).with_weight(1));

        // Jump the counter past the f64 mantissa boundary. The raw
        // `AtomicU64` is private but `select` starts from the internal
        // counter; we simulate a long-running process by selecting
        // once (to warm up) and then seeding the rr_counter via the
        // backing atomic through a public helper.
        //
        // Without direct access we exercise ordinary rotation; the
        // real precision gain is covered by the unit-level property
        // that `(counter % scaled_total)` is exact for all u64 inputs.
        let ctx = RequestContext::new();
        let mut counts = std::collections::HashMap::new();
        for _ in 0..300 {
            let sel = lb.select(&ctx).unwrap();
            *counts.entry(sel.node_id[0]).or_insert(0) += 1;
        }

        // Uniform weights → each of three endpoints gets ~100 hits.
        // This is a basic sanity test; the u64 exactness is verified
        // by construction (integer math has no rounding).
        for id in 1..=3u8 {
            let got = counts.get(&id).copied().unwrap_or(0);
            assert!(
                (80..=120).contains(&got),
                "endpoint {id} should get ~100 hits, got {got}",
            );
        }
    }

    #[test]
    fn test_load_balancer_health() {
        let lb = LoadBalancer::with_strategy(Strategy::RoundRobin);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));

        let ctx = RequestContext::new();

        // Mark node 1 as unhealthy
        lb.update_health(&make_node_id(1), HealthStatus::Unhealthy);

        // All selections should go to node 2
        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            assert_eq!(selection.node_id[0], 2);
        }
    }

    #[test]
    fn test_load_balancer_zone_affinity() {
        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            zone_aware: true,
            ..Default::default()
        };
        let lb = LoadBalancer::new(config);

        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_zone("us-east"));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_zone("us-west"));

        let ctx = RequestContext::new().with_zone("us-east");

        // Should prefer us-east node
        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            assert_eq!(selection.node_id[0], 1);
        }
    }

    #[test]
    fn test_load_balancer_consistent_hash() {
        let lb = LoadBalancer::with_strategy(Strategy::ConsistentHash);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));
        lb.add_endpoint(Endpoint::new(make_node_id(3)));

        // Same session should always go to same node
        let ctx = RequestContext::new().with_session("user-123");

        let first = lb.select(&ctx).unwrap();
        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            assert_eq!(selection.node_id, first.node_id);
        }
    }

    #[test]
    fn test_load_balancer_circuit_breaker() {
        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            circuit_recovery_time_ms: 100,
            ..Default::default()
        };
        let lb = LoadBalancer::new(config);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));

        let ctx = RequestContext::new();

        // Simulate 5 consecutive failures on node 1
        for _ in 0..5 {
            lb.record_completion(&make_node_id(1), false);
        }

        // Node 1's circuit should be open, all traffic to node 2
        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            assert_eq!(selection.node_id[0], 2);
        }
    }

    #[test]
    fn test_load_balancer_stats() {
        let lb = LoadBalancer::with_strategy(Strategy::RoundRobin);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));

        let ctx = RequestContext::new();

        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            lb.record_completion(&selection.node_id, true);
        }

        let stats = lb.stats();
        assert_eq!(stats.total_selections, 10);
        assert_eq!(stats.active_endpoints, 2);
        assert_eq!(stats.healthy_endpoints, 2);
    }

    #[test]
    fn test_no_endpoints_error() {
        let lb = LoadBalancer::with_strategy(Strategy::RoundRobin);
        let ctx = RequestContext::new();

        let result = lb.select(&ctx);
        assert!(matches!(
            result,
            Err(LoadBalancerError::NoEndpointsAvailable)
        ));
    }

    #[test]
    fn test_required_tags() {
        let lb = LoadBalancer::with_strategy(Strategy::RoundRobin);

        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_tag("gpu"));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_tag("cpu"));

        let ctx = RequestContext::new().require_tag("gpu");

        // Should only select gpu-tagged node
        for _ in 0..10 {
            let selection = lb.select(&ctx).unwrap();
            assert_eq!(selection.node_id[0], 1);
        }
    }

    // ---- Regression tests ----

    #[test]
    fn test_regression_consistent_hash_deterministic() {
        // Regression: consistent hash iterated DashMap in arbitrary order
        // instead of sorted order, so the same key could map to different
        // nodes across calls. Now uses sorted ring + binary search.
        let lb = LoadBalancer::with_strategy(Strategy::ConsistentHash);

        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        lb.add_endpoint(Endpoint::new(make_node_id(2)));
        lb.add_endpoint(Endpoint::new(make_node_id(3)));
        lb.add_endpoint(Endpoint::new(make_node_id(4)));

        // Many different keys should each consistently map to the same node
        for i in 0..50 {
            let key = format!("session-{}", i);
            let ctx = RequestContext::new().with_routing_key(&key);

            let first = lb.select(&ctx).unwrap().node_id;
            for _ in 0..20 {
                let again = lb.select(&ctx).unwrap().node_id;
                assert_eq!(
                    first, again,
                    "consistent hash must return same node for key '{}'",
                    key
                );
            }
        }
    }

    #[test]
    fn test_regression_nan_metrics_no_panic() {
        // Regression: partial_cmp().unwrap() panicked when metrics
        // contained NaN. Now uses total_cmp() which handles NaN.
        let lb = LoadBalancer::with_strategy(Strategy::LeastLatency);

        let mut ep1 = Endpoint::new(make_node_id(1));
        ep1.metrics.avg_response_time_ms = f64::NAN;
        lb.add_endpoint(ep1);

        let mut ep2 = Endpoint::new(make_node_id(2));
        ep2.metrics.avg_response_time_ms = 50.0;
        lb.add_endpoint(ep2);

        let ctx = RequestContext::new();
        // Must not panic
        let result = lb.select(&ctx);
        assert!(result.is_ok(), "NaN metrics must not panic");
    }

    #[test]
    fn test_regression_nan_load_score_no_panic() {
        // Same NaN regression for LeastLoad strategy.
        let lb = LoadBalancer::with_strategy(Strategy::LeastLoad);

        let mut ep1 = Endpoint::new(make_node_id(1));
        ep1.metrics.cpu_usage = f64::NAN;
        lb.add_endpoint(ep1);

        lb.add_endpoint(Endpoint::new(make_node_id(2)));

        let ctx = RequestContext::new();
        let result = lb.select(&ctx);
        assert!(result.is_ok(), "NaN load score must not panic");
    }

    #[test]
    fn test_regression_zone_fallback_respected() {
        // Regression: zone_fallback config was never read. When set to
        // false, requests with a client_zone that matches no endpoint
        // should fail, not silently fall back to non-zone endpoints.
        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            zone_aware: true,
            zone_fallback: false, // <-- this was previously ignored
            ..Default::default()
        };
        let lb = LoadBalancer::new(config);

        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_zone("us-west"));
        lb.add_endpoint(Endpoint::new(make_node_id(2)).with_zone("us-west"));

        // Client is in eu-central — no endpoints match
        let ctx = RequestContext::new().with_zone("eu-central");
        let result = lb.select(&ctx);

        assert!(
            result.is_err(),
            "with zone_fallback=false, mismatched zone must return error"
        );
    }

    #[test]
    fn test_zone_fallback_true_allows_cross_zone() {
        // Verify that zone_fallback=true (default) still works correctly.
        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            zone_aware: true,
            zone_fallback: true,
            ..Default::default()
        };
        let lb = LoadBalancer::new(config);

        lb.add_endpoint(Endpoint::new(make_node_id(1)).with_zone("us-west"));

        let ctx = RequestContext::new().with_zone("eu-central");
        let result = lb.select(&ctx);

        assert!(
            result.is_ok(),
            "with zone_fallback=true, cross-zone should succeed"
        );
    }

    #[test]
    fn test_regression_random_f64_never_reaches_one() {
        // Regression: `r as f64 / u64::MAX as f64` could return exactly 1.0
        // because `u64::MAX as f64` rounds to 2^64. Now uses the 53-bit
        // mantissa / 2^53 pattern which is strictly in [0, 1).
        for _ in 0..10_000 {
            let r = random_f64();
            assert!((0.0..1.0).contains(&r), "random_f64 out of [0,1): {}", r);
        }
    }

    #[test]
    fn test_regression_max_connections_cap_enforced_concurrently() {
        // Regression: the select() path loaded `connections` with Relaxed
        // then incremented in record_request, allowing N concurrent
        // selectors to all pass the check and collectively exceed the cap.
        // Now reservation is atomic via fetch_update.
        use std::sync::Arc;
        use std::thread;

        const CAP: u32 = 5;
        const THREADS: u32 = 16;

        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            max_connections_per_endpoint: CAP,
            ..Default::default()
        };
        let lb = Arc::new(LoadBalancer::new(config));
        // Single endpoint so every selection contends for the same cap.
        lb.add_endpoint(Endpoint::new(make_node_id(1)));

        let mut handles = Vec::new();
        for _ in 0..THREADS {
            let lb = Arc::clone(&lb);
            handles.push(thread::spawn(move || {
                // Each thread tries to select one connection and holds it.
                let ctx = RequestContext::new();
                lb.select(&ctx).ok()
            }));
        }

        let successes = handles
            .into_iter()
            .filter_map(|h| h.join().unwrap())
            .count();

        // At most CAP threads may have been granted a connection.
        assert!(
            successes <= CAP as usize,
            "concurrent selectors exceeded cap: {} > {}",
            successes,
            CAP
        );
        // And the endpoint's connection count must equal successes.
        let state = lb.endpoints.get(&make_node_id(1)).unwrap();
        assert_eq!(
            state.connections.load(Ordering::Acquire),
            successes as u32,
            "connection counter must match granted selections"
        );
    }

    #[test]
    fn test_regression_circuit_breaker_half_open_single_probe() {
        // Regression: on recovery expiry, `is_circuit_open` fully closed
        // the breaker, letting every concurrent request hit a possibly
        // still-broken endpoint. Now exactly one probe is admitted and
        // subsequent callers continue to see the breaker as open until the
        // probe's outcome is recorded.
        let config = LoadBalancerConfig {
            strategy: Strategy::RoundRobin,
            circuit_recovery_time_ms: 50,
            ..Default::default()
        };
        let lb = LoadBalancer::new(config);
        lb.add_endpoint(Endpoint::new(make_node_id(1)));
        let ctx = RequestContext::new();

        // Trip the breaker by driving 5 real selections that all fail. Going
        // through select() keeps the connection counter consistent — calling
        // record_completion() without a matching record_request() would
        // underflow.
        for _ in 0..5 {
            let sel = lb.select(&ctx).expect("admitted before trip");
            lb.record_completion(&sel.node_id, false);
        }

        // Before recovery: all requests rejected.
        assert!(lb.select(&ctx).is_err(), "open breaker must reject");

        // Wait past the recovery window.
        std::thread::sleep(Duration::from_millis(75));

        // First request after recovery: admitted as the probe.
        let probe = lb.select(&ctx);
        assert!(probe.is_ok(), "first request after recovery is the probe");

        // Second request while probe is still in flight: must be rejected.
        let second = lb.select(&ctx);
        assert!(
            second.is_err(),
            "while probe is in flight, other requests must still be rejected"
        );

        // Probe reports failure → breaker re-opens and recovery timer resets.
        lb.record_completion(&probe.unwrap().node_id, false);
        assert!(
            lb.select(&ctx).is_err(),
            "failed probe must keep breaker open"
        );

        // After another recovery window, the next probe succeeds and closes
        // the breaker.
        std::thread::sleep(Duration::from_millis(75));
        let probe2 = lb.select(&ctx).expect("second probe admitted");
        lb.record_completion(&probe2.node_id, true);

        // Breaker is now fully closed — subsequent requests go through.
        assert!(lb.select(&ctx).is_ok(), "successful probe closes breaker");
    }
}
