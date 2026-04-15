//! Simplified configuration builder for the SDK.

use std::time::Duration;

use net::config::{
    AdapterConfig, BackpressureMode, BatchConfig, EventBusConfig, EventBusConfigBuilder,
    ScalingPolicy,
};

/// Backpressure strategy.
#[derive(Debug, Clone, Copy)]
pub enum Backpressure {
    /// Drop newest events when buffer is full.
    DropNewest,
    /// Drop oldest events when buffer is full (default).
    DropOldest,
    /// Fail the producer when buffer is full.
    FailProducer,
    /// Sample events at the given rate (1 in N).
    Sample(u32),
}

impl From<Backpressure> for BackpressureMode {
    fn from(bp: Backpressure) -> Self {
        match bp {
            Backpressure::DropNewest => BackpressureMode::DropNewest,
            Backpressure::DropOldest => BackpressureMode::DropOldest,
            Backpressure::FailProducer => BackpressureMode::FailProducer,
            Backpressure::Sample(rate) => BackpressureMode::Sample { rate },
        }
    }
}

/// Builder for constructing a [`Net`](crate::Net) node.
pub struct NetBuilder {
    pub(crate) inner: EventBusConfigBuilder,
    pub(crate) adapter: Option<AdapterConfig>,
}

impl NetBuilder {
    pub(crate) fn new() -> Self {
        Self {
            inner: EventBusConfig::builder(),
            adapter: None,
        }
    }

    /// Set the number of shards.
    pub fn shards(mut self, n: u16) -> Self {
        self.inner = self.inner.num_shards(n);
        self
    }

    /// Set the ring buffer capacity per shard (must be power of 2).
    pub fn buffer_capacity(mut self, capacity: usize) -> Self {
        self.inner = self.inner.ring_buffer_capacity(capacity);
        self
    }

    /// Set the backpressure strategy.
    pub fn backpressure(mut self, bp: Backpressure) -> Self {
        self.inner = self.inner.backpressure_mode(bp.into());
        self
    }

    /// Use the high-throughput batch preset.
    pub fn high_throughput(mut self) -> Self {
        self.inner = self.inner.batch(BatchConfig::high_throughput());
        self
    }

    /// Use the low-latency batch preset.
    pub fn low_latency(mut self) -> Self {
        self.inner = self.inner.batch(BatchConfig::low_latency());
        self
    }

    /// Set a custom batch configuration.
    pub fn batch(mut self, batch: BatchConfig) -> Self {
        self.inner = self.inner.batch(batch);
        self
    }

    /// Enable dynamic scaling.
    pub fn scaling(mut self, policy: ScalingPolicy) -> Self {
        self.inner = self.inner.scaling(policy);
        self
    }

    /// Set the adapter timeout.
    pub fn adapter_timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.adapter_timeout(timeout);
        self
    }

    /// Use in-memory transport (no persistence, single process).
    pub fn memory(mut self) -> Self {
        self.adapter = Some(AdapterConfig::Noop);
        self
    }

    /// Use Redis Streams transport.
    #[cfg(feature = "redis")]
    pub fn redis(mut self, config: net::config::RedisAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::Redis(config));
        self
    }

    /// Use NATS JetStream transport.
    #[cfg(feature = "jetstream")]
    pub fn jetstream(mut self, config: net::config::JetStreamAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::JetStream(config));
        self
    }

    /// Use Net encrypted UDP mesh transport.
    #[cfg(feature = "net")]
    pub fn mesh(mut self, config: net::adapter::net::NetAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::Net(Box::new(config)));
        self
    }

    /// Build the configuration, consuming the builder.
    pub(crate) fn build_config(mut self) -> crate::error::Result<EventBusConfig> {
        if let Some(adapter) = self.adapter {
            self.inner = self.inner.adapter(adapter);
        }
        self.inner
            .build()
            .map_err(|e| crate::error::SdkError::Config(e.to_string()))
    }
}
