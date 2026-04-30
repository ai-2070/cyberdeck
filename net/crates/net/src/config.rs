//! Configuration types for the Net event bus.

use std::time::Duration;

/// Top-level configuration for the event bus.
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Number of shards for parallel ingestion.
    /// Each shard has its own ring buffer and batch worker.
    /// Default: number of CPU cores.
    ///
    /// Unless you're connected to an advanced AI orchestrator, swarm controller,
    /// or a local Nvidia Blackwell GPU cluster, any number is fine - it will
    /// revert to your physical core count by default.
    pub num_shards: u16,

    /// Capacity of each shard's ring buffer (number of events).
    /// Must be a power of 2 for efficient modulo operations.
    /// Default: 1,048,576 (1M events per shard).
    pub ring_buffer_capacity: usize,

    /// Backpressure policy when ring buffers are full.
    pub backpressure_mode: BackpressureMode,

    /// Batch aggregation configuration.
    pub batch: BatchConfig,

    /// Adapter configuration.
    pub adapter: AdapterConfig,

    /// Dynamic scaling configuration.
    /// If None, dynamic scaling is disabled.
    pub scaling: Option<ScalingPolicy>,

    /// Timeout for adapter operations (init, on_batch, flush, shutdown).
    /// Prevents a hanging adapter from blocking the event bus.
    /// Default: 30 seconds.
    pub adapter_timeout: Duration,

    /// Number of times to retry a failed on_batch before dropping the batch.
    /// 0 = no retries (drop immediately on failure, default).
    /// Retries use a fixed 100ms delay between attempts.
    pub adapter_batch_retries: u32,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        let cpus = num_cpus();
        Self {
            num_shards: cpus,
            ring_buffer_capacity: 1 << 20, // 1M events
            backpressure_mode: BackpressureMode::DropOldest,
            batch: BatchConfig::default(),
            adapter: AdapterConfig::Noop,
            scaling: Some(ScalingPolicy::default_for_cpus(cpus)),
            adapter_timeout: Duration::from_secs(30),
            adapter_batch_retries: 0,
        }
    }
}

impl EventBusConfig {
    /// Create a new configuration builder.
    pub fn builder() -> EventBusConfigBuilder {
        EventBusConfigBuilder::default()
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.num_shards == 0 {
            return Err(ConfigError::InvalidValue("num_shards must be > 0".into()));
        }
        if !self.ring_buffer_capacity.is_power_of_two() {
            return Err(ConfigError::InvalidValue(
                "ring_buffer_capacity must be a power of 2".into(),
            ));
        }
        if self.ring_buffer_capacity < 1024 {
            return Err(ConfigError::InvalidValue(
                "ring_buffer_capacity must be >= 1024".into(),
            ));
        }
        // Adapter timeout of zero would make every adapter call time
        // out instantly. Reject at config time.
        if self.adapter_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "adapter_timeout must be > 0".into(),
            ));
        }
        // `Sample { rate: 0 }` was accepted but crashed downstream
        // sampling (counter % 0).
        if let BackpressureMode::Sample { rate: 0 } = self.backpressure_mode {
            return Err(ConfigError::InvalidValue(
                "BackpressureMode::Sample.rate must be > 0".into(),
            ));
        }
        self.batch.validate()?;
        if let Some(ref scaling) = self.scaling {
            scaling
                .validate()
                .map_err(|e| ConfigError::InvalidValue(format!("scaling policy: {}", e)))?;
        }
        // Recurse into adapter configs. Previously these were
        // accepted blindly and zero-divisor fields like
        // `RedisAdapterConfig::pipeline_size: 0` shipped through to
        // runtime panics.
        match &self.adapter {
            AdapterConfig::Noop => {}
            #[cfg(feature = "redis")]
            AdapterConfig::Redis(c) => c
                .validate()
                .map_err(|e| ConfigError::InvalidValue(format!("redis adapter: {}", e)))?,
            #[cfg(feature = "jetstream")]
            AdapterConfig::JetStream(c) => c
                .validate()
                .map_err(|e| ConfigError::InvalidValue(format!("jetstream adapter: {}", e)))?,
            #[cfg(feature = "net")]
            AdapterConfig::Net(_) => {} // Net adapter has its own
                                        // validation pipeline, not in scope here.
        }
        Ok(())
    }
}

/// Scaling configuration for the builder.
#[derive(Debug, Clone)]
enum ScalingConfig {
    /// Use default policy based on num_shards (resolved at build time).
    Default,
    /// Explicitly disabled.
    Disabled,
    /// Explicit policy.
    Policy(ScalingPolicy),
}

/// Builder for EventBusConfig.
#[derive(Debug, Default)]
pub struct EventBusConfigBuilder {
    num_shards: Option<u16>,
    ring_buffer_capacity: Option<usize>,
    backpressure_mode: Option<BackpressureMode>,
    batch: Option<BatchConfig>,
    adapter: Option<AdapterConfig>,
    scaling: Option<ScalingConfig>,
    adapter_timeout: Option<Duration>,
    adapter_batch_retries: Option<u32>,
}

impl EventBusConfigBuilder {
    /// Set the number of shards.
    pub fn num_shards(mut self, n: u16) -> Self {
        self.num_shards = Some(n);
        self
    }

    /// Set the ring buffer capacity per shard.
    pub fn ring_buffer_capacity(mut self, cap: usize) -> Self {
        self.ring_buffer_capacity = Some(cap);
        self
    }

    /// Set the backpressure mode.
    pub fn backpressure_mode(mut self, mode: BackpressureMode) -> Self {
        self.backpressure_mode = Some(mode);
        self
    }

    /// Set the batch configuration.
    pub fn batch(mut self, config: BatchConfig) -> Self {
        self.batch = Some(config);
        self
    }

    /// Set the adapter configuration.
    pub fn adapter(mut self, config: AdapterConfig) -> Self {
        self.adapter = Some(config);
        self
    }

    /// Enable dynamic scaling with the given policy.
    pub fn scaling(mut self, policy: ScalingPolicy) -> Self {
        self.scaling = Some(ScalingConfig::Policy(policy));
        self
    }

    /// Enable dynamic scaling with default policy.
    /// The policy's max_shards will be based on num_shards (resolved at build time).
    pub fn with_dynamic_scaling(mut self) -> Self {
        self.scaling = Some(ScalingConfig::Default);
        self
    }

    /// Disable dynamic scaling (use fixed shard count).
    pub fn without_scaling(mut self) -> Self {
        self.scaling = Some(ScalingConfig::Disabled);
        self
    }

    /// Set the adapter operation timeout.
    pub fn adapter_timeout(mut self, timeout: Duration) -> Self {
        self.adapter_timeout = Some(timeout);
        self
    }

    /// Set the number of retries for failed on_batch calls.
    /// 0 = no retries (default). Useful for Redis/JetStream under intermittent failures.
    pub fn adapter_batch_retries(mut self, retries: u32) -> Self {
        self.adapter_batch_retries = Some(retries);
        self
    }

    /// Build the configuration, validating all settings.
    pub fn build(self) -> Result<EventBusConfig, ConfigError> {
        let num_shards = self.num_shards.unwrap_or_else(num_cpus);
        let scaling = match self.scaling {
            Some(ScalingConfig::Policy(policy)) => Some(policy),
            Some(ScalingConfig::Default) | None => {
                Some(ScalingPolicy::default_for_cpus(num_shards))
            }
            Some(ScalingConfig::Disabled) => None,
        };
        let config = EventBusConfig {
            num_shards,
            ring_buffer_capacity: self.ring_buffer_capacity.unwrap_or(1 << 20),
            backpressure_mode: self
                .backpressure_mode
                .unwrap_or(BackpressureMode::DropOldest),
            batch: self.batch.unwrap_or_default(),
            adapter: self.adapter.unwrap_or(AdapterConfig::Noop),
            scaling,
            adapter_timeout: self.adapter_timeout.unwrap_or(Duration::from_secs(30)),
            adapter_batch_retries: self.adapter_batch_retries.unwrap_or(0),
        };
        config.validate()?;
        Ok(config)
    }
}

/// Backpressure policy when ring buffers are full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureMode {
    /// Drop the newest event (the one being inserted).
    DropNewest,

    /// Drop the oldest event in the buffer to make room.
    DropOldest,

    /// Return an error to the producer.
    FailProducer,

    /// Sample events: keep 1 in N events.
    Sample {
        /// Keep 1 event for every `rate` events.
        rate: u32,
    },
}

/// Batch aggregation configuration.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Minimum batch size (floor for adaptive sizing).
    /// Default: 1,000 events.
    pub min_size: usize,

    /// Maximum batch size (ceiling for adaptive sizing).
    /// Default: 10,000 events.
    pub max_size: usize,

    /// Maximum time to wait before flushing a partial batch.
    /// Default: 10ms.
    pub max_delay: Duration,

    /// Enable adaptive batch sizing based on ingestion velocity.
    /// Default: true.
    pub adaptive: bool,

    /// Window size for velocity calculation (adaptive mode).
    /// Default: 100ms.
    pub velocity_window: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            min_size: 1_000,
            max_size: 10_000,
            max_delay: Duration::from_millis(10),
            adaptive: true,
            velocity_window: Duration::from_millis(100),
        }
    }
}

impl BatchConfig {
    /// Validate the batch configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_size == 0 {
            return Err(ConfigError::InvalidValue("min_size must be > 0".into()));
        }
        if self.max_size < self.min_size {
            return Err(ConfigError::InvalidValue(
                "max_size must be >= min_size".into(),
            ));
        }
        if self.max_delay.is_zero() {
            return Err(ConfigError::InvalidValue("max_delay must be > 0".into()));
        }
        // Zero `velocity_window` div-by-zeros the throughput
        // calculator when adaptive batching is enabled. Validate
        // only when the field is actually consulted.
        if self.adaptive && self.velocity_window.is_zero() {
            return Err(ConfigError::InvalidValue(
                "velocity_window must be > 0 when adaptive batching is enabled".into(),
            ));
        }
        Ok(())
    }

    /// Create a high-throughput configuration optimized for Blackwell workloads.
    pub fn high_throughput() -> Self {
        Self {
            min_size: 5_000,
            max_size: 50_000,
            max_delay: Duration::from_millis(5),
            adaptive: true,
            velocity_window: Duration::from_millis(50),
        }
    }

    /// Create a low-latency configuration for interactive workloads.
    pub fn low_latency() -> Self {
        Self {
            min_size: 100,
            max_size: 1_000,
            max_delay: Duration::from_millis(1),
            adaptive: true,
            velocity_window: Duration::from_millis(20),
        }
    }
}

/// Adapter configuration.
#[derive(Debug, Clone)]
pub enum AdapterConfig {
    /// No-op adapter (events are discarded after batching).
    /// Useful for testing and benchmarking.
    Noop,

    /// Redis Streams adapter.
    #[cfg(feature = "redis")]
    Redis(RedisAdapterConfig),

    /// NATS JetStream adapter.
    #[cfg(feature = "jetstream")]
    JetStream(JetStreamAdapterConfig),

    /// Net (Net L0 Transport Protocol) adapter.
    /// High-performance UDP transport for GPU-to-GPU communication.
    #[cfg(feature = "net")]
    Net(Box<crate::adapter::net::NetAdapterConfig>),
}

/// Redis adapter configuration.
#[cfg(feature = "redis")]
#[derive(Debug, Clone)]
pub struct RedisAdapterConfig {
    /// Redis connection URL.
    /// Example: "redis://localhost:6379"
    pub url: String,

    /// Stream key prefix.
    /// Streams are named: "{prefix}:shard:{shard_id}"
    /// Default: "net"
    pub prefix: String,

    /// Maximum commands per pipeline.
    /// Default: 1000.
    pub pipeline_size: usize,

    /// Connection pool size.
    /// Default: number of shards.
    pub pool_size: Option<usize>,

    /// Connection timeout.
    /// Default: 5 seconds.
    pub connect_timeout: Duration,

    /// Command timeout.
    /// Default: 1 second.
    pub command_timeout: Duration,

    /// Maximum stream length (MAXLEN for XADD).
    /// None = unlimited.
    pub max_stream_len: Option<usize>,
}

#[cfg(feature = "redis")]
impl RedisAdapterConfig {
    /// Create a new Redis adapter configuration.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            prefix: "net".into(),
            pipeline_size: 1000,
            pool_size: None,
            connect_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(1),
            max_stream_len: None,
        }
    }

    /// Set the stream key prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set the pipeline size.
    pub fn with_pipeline_size(mut self, size: usize) -> Self {
        self.pipeline_size = size;
        self
    }

    /// Set the connection pool size.
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = Some(size);
        self
    }

    /// Set the connection timeout.
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the command timeout.
    pub fn with_command_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }

    /// Set the maximum stream length.
    pub fn with_max_stream_len(mut self, len: usize) -> Self {
        self.max_stream_len = Some(len);
        self
    }

    /// Validate the configuration. Called from
    /// `EventBusConfig::validate` so adapter misconfiguration is
    /// caught at startup rather than at the first batch dispatch.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::InvalidValue(
                "redis url must be non-empty".into(),
            ));
        }
        if self.pipeline_size == 0 {
            return Err(ConfigError::InvalidValue(
                "redis pipeline_size must be > 0".into(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "redis connect_timeout must be > 0".into(),
            ));
        }
        if self.command_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "redis command_timeout must be > 0".into(),
            ));
        }
        Ok(())
    }
}

/// NATS JetStream adapter configuration.
#[cfg(feature = "jetstream")]
#[derive(Debug, Clone)]
pub struct JetStreamAdapterConfig {
    /// NATS server URL.
    /// Example: "nats://localhost:4222"
    pub url: String,

    /// Stream name prefix.
    /// Streams are named: "{prefix}_shard_{shard_id}"
    /// Default: "net"
    pub prefix: String,

    /// Connection timeout.
    /// Default: 5 seconds.
    pub connect_timeout: Duration,

    /// Request timeout for publish/fetch operations.
    /// Default: 5 seconds.
    pub request_timeout: Duration,

    /// Maximum messages per stream (oldest are discarded when exceeded).
    /// None = unlimited.
    pub max_messages: Option<i64>,

    /// Maximum bytes per stream.
    /// None = unlimited.
    pub max_bytes: Option<i64>,

    /// Maximum age for messages in the stream.
    /// None = unlimited.
    pub max_age: Option<Duration>,

    /// Number of stream replicas for fault tolerance.
    /// Default: 1 (no replication).
    pub replicas: usize,
}

#[cfg(feature = "jetstream")]
impl JetStreamAdapterConfig {
    /// Create a new JetStream adapter configuration.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            prefix: "net".into(),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(5),
            max_messages: None,
            max_bytes: None,
            max_age: None,
            replicas: 1,
        }
    }

    /// Set the stream name prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set the connection timeout.
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the maximum messages per stream.
    pub fn with_max_messages(mut self, max: i64) -> Self {
        self.max_messages = Some(max);
        self
    }

    /// Set the maximum bytes per stream.
    pub fn with_max_bytes(mut self, max: i64) -> Self {
        self.max_bytes = Some(max);
        self
    }

    /// Set the maximum age for messages.
    pub fn with_max_age(mut self, age: Duration) -> Self {
        self.max_age = Some(age);
        self
    }

    /// Set the number of replicas.
    pub fn with_replicas(mut self, replicas: usize) -> Self {
        self.replicas = replicas;
        self
    }

    /// Validate the configuration. Called from
    /// `EventBusConfig::validate` so adapter misconfiguration is
    /// caught at startup rather than at the first batch dispatch.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::InvalidValue(
                "jetstream url must be non-empty".into(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "jetstream connect_timeout must be > 0".into(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "jetstream request_timeout must be > 0".into(),
            ));
        }
        if self.replicas == 0 {
            return Err(ConfigError::InvalidValue(
                "jetstream replicas must be >= 1".into(),
            ));
        }
        Ok(())
    }
}

/// Configuration errors.
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Invalid configuration value.
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue(msg) => write!(f, "invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Policy configuration for dynamic shard scaling.
#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    /// Fill ratio threshold to trigger scale-up (0.0 - 1.0).
    /// Default: 0.7 (70%)
    pub fill_ratio_threshold: f64,

    /// Push latency threshold in nanoseconds to trigger scale-up.
    /// Default: 5ns (after this, we're seeing contention)
    pub push_latency_threshold_ns: u64,

    /// Batch flush latency threshold in microseconds to trigger scale-up.
    /// Default: 1000μs (1ms)
    pub flush_latency_threshold_us: u64,

    /// Minimum number of shards (floor for scaling down).
    pub min_shards: u16,

    /// Maximum number of shards (ceiling for scaling up).
    pub max_shards: u16,

    /// Cooldown period between scaling operations.
    /// Default: 1 second
    pub cooldown: Duration,

    /// How long a shard must be underutilized before scaling down.
    /// Default: 10 seconds
    pub scale_down_delay: Duration,

    /// Fill ratio below which a shard is considered underutilized.
    /// Default: 0.1 (10%)
    pub underutilized_threshold: f64,

    /// Metrics collection window.
    /// Default: 100ms
    pub metrics_window: Duration,

    /// Enable automatic scaling (if false, scaling is manual only).
    pub auto_scale: bool,
}

impl Default for ScalingPolicy {
    fn default() -> Self {
        Self::default_for_cpus(num_cpus())
    }
}

impl ScalingPolicy {
    /// Create a default scaling policy based on CPU count.
    /// Scales from 1 shard up to the number of physical cores.
    pub fn default_for_cpus(cpus: u16) -> Self {
        Self {
            fill_ratio_threshold: 0.7,
            push_latency_threshold_ns: 5,
            flush_latency_threshold_us: 1000,
            min_shards: 1,
            max_shards: cpus,
            cooldown: Duration::from_secs(1),
            scale_down_delay: Duration::from_secs(10),
            underutilized_threshold: 0.1,
            metrics_window: Duration::from_millis(100),
            auto_scale: true,
        }
    }

    /// Create a policy optimized for high-throughput GPU workloads.
    /// Uses more aggressive scaling with higher max shard count.
    pub fn high_throughput() -> Self {
        let cpus = num_cpus();
        Self {
            fill_ratio_threshold: 0.6,
            push_latency_threshold_ns: 3,
            flush_latency_threshold_us: 500,
            min_shards: 4.min(cpus),
            max_shards: cpus.saturating_mul(2), // Allow up to 2x CPU count for GPU workloads
            cooldown: Duration::from_millis(500),
            scale_down_delay: Duration::from_secs(30),
            underutilized_threshold: 0.05,
            metrics_window: Duration::from_millis(50),
            auto_scale: true,
        }
    }

    /// Create a conservative policy for stable workloads.
    pub fn conservative() -> Self {
        let cpus = num_cpus();
        Self {
            fill_ratio_threshold: 0.8,
            push_latency_threshold_ns: 10,
            flush_latency_threshold_us: 2000,
            min_shards: 1,
            max_shards: cpus,
            cooldown: Duration::from_secs(5),
            scale_down_delay: Duration::from_secs(60),
            underutilized_threshold: 0.05,
            metrics_window: Duration::from_millis(200),
            auto_scale: true,
        }
    }

    /// Normalize the policy by auto-adjusting conflicting values.
    ///
    /// This allows users to set either `min_shards` or `max_shards` independently
    /// without worrying about the other. If `max_shards < min_shards`, `max_shards`
    /// is adjusted to equal `min_shards`.
    pub fn normalize(mut self) -> Self {
        if self.max_shards < self.min_shards {
            self.max_shards = self.min_shards;
        }
        self
    }

    /// Validate the policy.
    ///
    /// BUG #63: NaN thresholds slip past raw `<=` / `>` comparisons
    /// (every comparison against `f64::NaN` returns `false`), so a
    /// config deserialized from `0.0/0.0`-style arithmetic or an
    /// unfortunate environment-templated string used to "validate"
    /// successfully and then sit inert at runtime — `mapper.rs:560`
    /// does `m.fill_ratio > self.policy.fill_ratio_threshold`,
    /// which is always `false` against NaN, so the scaler never
    /// fires (mirror hazard for scale-down). The pre-fix range
    /// checks below were necessary but not sufficient; the new
    /// `is_finite()` guards reject NaN and ±∞ explicitly before
    /// the range check runs.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.fill_ratio_threshold.is_finite() {
            return Err(ConfigError::InvalidValue(
                "fill_ratio_threshold must be finite (NaN/±inf rejected)".into(),
            ));
        }
        if self.fill_ratio_threshold <= 0.0 || self.fill_ratio_threshold > 1.0 {
            return Err(ConfigError::InvalidValue(
                "fill_ratio_threshold must be in (0.0, 1.0]".into(),
            ));
        }
        if !self.underutilized_threshold.is_finite() {
            return Err(ConfigError::InvalidValue(
                "underutilized_threshold must be finite (NaN/±inf rejected)".into(),
            ));
        }
        if self.underutilized_threshold < 0.0 || self.underutilized_threshold > 1.0 {
            return Err(ConfigError::InvalidValue(
                "underutilized_threshold must be in [0.0, 1.0]".into(),
            ));
        }
        if self.min_shards == 0 {
            return Err(ConfigError::InvalidValue("min_shards must be > 0".into()));
        }
        if self.max_shards < self.min_shards {
            return Err(ConfigError::InvalidValue(
                "max_shards must be >= min_shards".into(),
            ));
        }
        // Zero durations on the scaling path either div-by-zero
        // (`metrics_window`), thrash the scaler (`cooldown`), or scale
        // down on the first underutilized sample (`scale_down_delay`).
        // Reject all three at config time.
        if self.cooldown.is_zero() {
            return Err(ConfigError::InvalidValue("cooldown must be > 0".into()));
        }
        if self.metrics_window.is_zero() {
            return Err(ConfigError::InvalidValue(
                "metrics_window must be > 0".into(),
            ));
        }
        if self.scale_down_delay.is_zero() {
            return Err(ConfigError::InvalidValue(
                "scale_down_delay must be > 0".into(),
            ));
        }
        Ok(())
    }
}

/// Get the number of CPU cores (fallback to 1).
fn num_cpus() -> u16 {
    std::thread::available_parallelism()
        .map(|n| u16::try_from(n.get()).unwrap_or(u16::MAX))
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EventBusConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.num_shards > 0);
        assert!(config.ring_buffer_capacity.is_power_of_two());
    }

    #[test]
    fn test_builder() {
        let config = EventBusConfig::builder()
            .num_shards(8)
            .ring_buffer_capacity(1 << 16)
            .backpressure_mode(BackpressureMode::FailProducer)
            .build()
            .unwrap();

        assert_eq!(config.num_shards, 8);
        assert_eq!(config.ring_buffer_capacity, 65536);
        assert_eq!(config.backpressure_mode, BackpressureMode::FailProducer);
    }

    #[test]
    fn test_invalid_ring_buffer_capacity() {
        let result = EventBusConfig::builder()
            .ring_buffer_capacity(1000) // Not a power of 2
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_batch_config_presets() {
        let high = BatchConfig::high_throughput();
        assert!(high.validate().is_ok());
        assert!(high.max_size > high.min_size);

        let low = BatchConfig::low_latency();
        assert!(low.validate().is_ok());
        assert!(low.max_delay < high.max_delay);
    }

    #[test]
    fn test_scaling_enabled_by_default() {
        let config = EventBusConfig::default();
        assert!(config.scaling.is_some());

        let policy = config.scaling.unwrap();
        assert_eq!(policy.max_shards, config.num_shards);
        assert!(policy.auto_scale);
    }

    #[test]
    fn test_builder_enables_scaling_by_default() {
        let config = EventBusConfig::builder().num_shards(8).build().unwrap();

        assert!(config.scaling.is_some());
        let policy = config.scaling.unwrap();
        assert_eq!(policy.max_shards, 8);
    }

    #[test]
    fn test_builder_without_scaling() {
        let config = EventBusConfig::builder()
            .num_shards(4)
            .without_scaling()
            .build()
            .unwrap();

        assert!(config.scaling.is_none());
    }

    #[test]
    fn test_with_dynamic_scaling_respects_num_shards() {
        // with_dynamic_scaling() should use num_shards for max_shards, not CPU count
        let config = EventBusConfig::builder()
            .num_shards(8)
            .with_dynamic_scaling()
            .build()
            .unwrap();

        assert!(config.scaling.is_some());
        let policy = config.scaling.unwrap();
        assert_eq!(policy.max_shards, 8);

        // Order shouldn't matter
        let config2 = EventBusConfig::builder()
            .with_dynamic_scaling()
            .num_shards(16)
            .build()
            .unwrap();

        assert!(config2.scaling.is_some());
        let policy2 = config2.scaling.unwrap();
        assert_eq!(policy2.max_shards, 16);
    }

    #[test]
    fn test_scaling_policy_presets() {
        let high = ScalingPolicy::high_throughput();
        assert!(high.validate().is_ok());
        assert!(high.max_shards >= high.min_shards);

        let conservative = ScalingPolicy::conservative();
        assert!(conservative.validate().is_ok());
        assert!(conservative.cooldown > high.cooldown);
    }

    #[test]
    fn test_scaling_policy_validation() {
        let mut policy = ScalingPolicy {
            underutilized_threshold: 0.0,
            ..Default::default()
        };

        // Valid underutilized_threshold
        assert!(policy.validate().is_ok());
        policy.underutilized_threshold = 0.5;
        assert!(policy.validate().is_ok());
        policy.underutilized_threshold = 1.0;
        assert!(policy.validate().is_ok());

        // Invalid underutilized_threshold
        policy.underutilized_threshold = -0.1;
        assert!(policy.validate().is_err());
        policy.underutilized_threshold = 1.1;
        assert!(policy.validate().is_err());

        // Reset and test fill_ratio_threshold
        policy.underutilized_threshold = 0.1;
        policy.fill_ratio_threshold = 0.0;
        assert!(policy.validate().is_err());
        policy.fill_ratio_threshold = 1.1;
        assert!(policy.validate().is_err());
    }

    // ========================================================================
    // BUG #63: validate() must reject NaN / ±inf thresholds
    // ========================================================================

    /// `validate()` rejects `f64::NaN` for both threshold fields.
    /// Pre-fix the raw `<=` / `>` range checks accepted NaN
    /// because every comparison with NaN returns `false`; the
    /// "validated" config then sat inert at runtime since the
    /// scaler's `m.fill_ratio > policy.fill_ratio_threshold` was
    /// always false against NaN.
    #[test]
    fn validate_rejects_nan_fill_ratio_threshold() {
        let policy = ScalingPolicy {
            fill_ratio_threshold: f64::NAN,
            ..Default::default()
        };
        assert!(
            policy.validate().is_err(),
            "NaN fill_ratio_threshold must be rejected (BUG #63)",
        );
    }

    #[test]
    fn validate_rejects_nan_underutilized_threshold() {
        let policy = ScalingPolicy {
            underutilized_threshold: f64::NAN,
            ..Default::default()
        };
        assert!(
            policy.validate().is_err(),
            "NaN underutilized_threshold must be rejected (BUG #63)",
        );
    }

    /// `validate()` also rejects `±inf` for both threshold fields.
    /// A config that arithmetic'd to infinity (e.g. divide-by-tiny)
    /// would have slipped through the `> 1.0` check on positive
    /// infinity (which IS rejected) but not on a negative infinity
    /// against the lower bound for `fill_ratio_threshold` —
    /// `-inf <= 0.0` is true, so it would have been rejected
    /// already; for `underutilized_threshold` the lower bound
    /// `-inf < 0.0` is also true. The explicit `is_finite()` guard
    /// pins these edge cases regardless of which bound check would
    /// have fired.
    #[test]
    fn validate_rejects_infinity_thresholds() {
        let p1 = ScalingPolicy {
            fill_ratio_threshold: f64::INFINITY,
            ..Default::default()
        };
        assert!(p1.validate().is_err());

        let p2 = ScalingPolicy {
            fill_ratio_threshold: f64::NEG_INFINITY,
            ..Default::default()
        };
        assert!(p2.validate().is_err());

        let p3 = ScalingPolicy {
            underutilized_threshold: f64::INFINITY,
            ..Default::default()
        };
        assert!(p3.validate().is_err());

        let p4 = ScalingPolicy {
            underutilized_threshold: f64::NEG_INFINITY,
            ..Default::default()
        };
        assert!(p4.validate().is_err());
    }

    #[test]
    fn test_config_validates_scaling_policy() {
        // Invalid scaling policy should cause config build to fail
        let invalid_policy = ScalingPolicy {
            min_shards: 10,
            max_shards: 5, // Invalid: min > max
            ..Default::default()
        };

        let result = EventBusConfig::builder()
            .num_shards(4)
            .scaling(invalid_policy)
            .build();

        assert!(result.is_err());

        // Another invalid policy
        let invalid_policy2 = ScalingPolicy {
            fill_ratio_threshold: 1.5, // Invalid: > 1.0
            ..Default::default()
        };

        let result2 = EventBusConfig::builder()
            .num_shards(4)
            .scaling(invalid_policy2)
            .build();

        assert!(result2.is_err());
    }

    // Regression: high_throughput() used cpus * 2 which overflows u16
    // on machines with >32K CPUs (BUGS_3 #7).
    #[test]
    fn test_high_throughput_max_shards_no_overflow() {
        let policy = ScalingPolicy::high_throughput();
        assert!(policy.max_shards >= policy.min_shards);
        assert!(policy.validate().is_ok());
    }

    /// Regression: BUG_REPORT.md #3 — zero-rate `Sample` previously
    /// passed validation but div-by-zero'd downstream.
    #[test]
    fn test_validate_rejects_sample_rate_zero() {
        let result = EventBusConfig::builder()
            .backpressure_mode(BackpressureMode::Sample { rate: 0 })
            .build();
        assert!(
            result.is_err(),
            "BackpressureMode::Sample.rate == 0 must reject"
        );
    }

    /// Regression: BUG_REPORT.md #3 — zero `velocity_window` with
    /// adaptive batching div-by-zero'd the throughput calculator.
    #[test]
    fn test_validate_rejects_zero_velocity_window_when_adaptive() {
        let bad = BatchConfig {
            adaptive: true,
            velocity_window: Duration::ZERO,
            ..Default::default()
        };
        assert!(bad.validate().is_err());

        // Non-adaptive ignores the field.
        let ok = BatchConfig {
            adaptive: false,
            velocity_window: Duration::ZERO,
            ..Default::default()
        };
        assert!(ok.validate().is_ok());
    }

    /// Regression: BUG_REPORT.md #3 — zero `adapter_timeout` made
    /// every adapter call time out instantly.
    #[test]
    fn test_validate_rejects_zero_adapter_timeout() {
        let config = EventBusConfig {
            adapter_timeout: Duration::ZERO,
            ..EventBusConfig::default()
        };
        assert!(config.validate().is_err());
    }

    /// Regression: BUG_REPORT.md #3 — `ScalingPolicy` durations of
    /// zero either div-by-zero'd, thrashed the scaler, or scaled
    /// down on the first underutilized sample.
    #[test]
    fn test_validate_rejects_zero_scaling_durations() {
        let base = ScalingPolicy::default();

        let mut p = base.clone();
        p.cooldown = Duration::ZERO;
        assert!(p.validate().is_err());

        let mut p = base.clone();
        p.metrics_window = Duration::ZERO;
        assert!(p.validate().is_err());

        let mut p = base;
        p.scale_down_delay = Duration::ZERO;
        assert!(p.validate().is_err());
    }

    /// Regression: BUG_REPORT.md #3 — `RedisAdapterConfig` had no
    /// `validate()` and `pipeline_size: 0` shipped through to a
    /// runtime panic.
    #[cfg(feature = "redis")]
    #[test]
    fn test_validate_redis_pipeline_size_zero_rejected() {
        let mut redis = RedisAdapterConfig::new("redis://localhost:6379");
        redis.pipeline_size = 0;

        let result = EventBusConfig::builder()
            .adapter(AdapterConfig::Redis(redis))
            .build();
        assert!(result.is_err(), "redis pipeline_size == 0 must reject");
    }

    /// Regression: BUG_REPORT.md #3 — `JetStreamAdapterConfig` had
    /// no `validate()` either.
    #[cfg(feature = "jetstream")]
    #[test]
    fn test_validate_jetstream_replicas_zero_rejected() {
        let mut js = JetStreamAdapterConfig::new("nats://localhost:4222");
        js.replicas = 0;

        let result = EventBusConfig::builder()
            .adapter(AdapterConfig::JetStream(js))
            .build();
        assert!(result.is_err(), "jetstream replicas == 0 must reject");
    }
}
