//! Adapter trait and implementations for durable event storage.
//!
//! Adapters provide the persistence layer for the event bus. They receive
//! batches of events from the ingestion core and store them durably.
//!
//! # Adapter Contract
//!
//! Adapters must:
//! - Append batches in received order
//! - Never block ingestion indefinitely
//! - Fail fast on internal errors
//! - Be idempotent under retry
//! - Preserve per-shard FIFO order
//! - NOT allocate memory per-event (only per-batch or static)
//!
//! # Available Adapters
//!
//! - `NoopAdapter`: Discards events (for testing/benchmarking)
//! - `RedisAdapter`: Redis Streams backend (requires `redis` feature)
//! - `JetStreamAdapter`: NATS JetStream backend (requires `jetstream` feature)
//! - `NltpAdapter`: High-performance UDP transport (requires `nltp` feature)

mod noop;

#[cfg(feature = "redis")]
mod redis;

#[cfg(feature = "jetstream")]
mod jetstream;

#[cfg(feature = "nltp")]
pub mod nltp;

pub use noop::NoopAdapter;

#[cfg(feature = "redis")]
pub use self::redis::RedisAdapter;

#[cfg(feature = "jetstream")]
pub use self::jetstream::JetStreamAdapter;

#[cfg(feature = "nltp")]
pub use self::nltp::{NltpAdapter, NltpAdapterConfig};

use async_trait::async_trait;

use crate::error::AdapterError;
use crate::event::{Batch, StoredEvent};

/// Result of polling a single shard.
#[derive(Debug, Clone)]
pub struct ShardPollResult {
    /// Events retrieved from the shard.
    pub events: Vec<StoredEvent>,
    /// Cursor for the next poll (backend-specific).
    /// None if no events were returned.
    pub next_id: Option<String>,
    /// True if there are more events available.
    pub has_more: bool,
}

impl ShardPollResult {
    /// Create an empty poll result.
    pub fn empty() -> Self {
        Self {
            events: Vec::new(),
            next_id: None,
            has_more: false,
        }
    }
}

/// Adapter trait for durable event storage.
///
/// # Memory Allocation Constraint
///
/// Adapters **MUST NOT** allocate memory per-event. Allowed allocations:
/// - Per-batch buffer allocation (reusable)
/// - Static/pooled buffers
/// - Connection resources
///
/// Forbidden:
/// - `Vec::push` per event in hot path
/// - String allocation per event
/// - Any heap allocation scaling with event count
#[async_trait]
pub trait Adapter: Send + Sync {
    /// Initialize the adapter.
    ///
    /// Called once before any other methods. Use this to establish
    /// connections, validate configuration, etc.
    async fn init(&mut self) -> Result<(), AdapterError>;

    /// Process a batch of events.
    ///
    /// The adapter must persist all events in the batch atomically
    /// (all or nothing). Events must be stored in order within the batch.
    ///
    /// # Errors
    ///
    /// - `AdapterError::Transient`: Temporary failure, retry is safe
    /// - `AdapterError::Fatal`: Unrecoverable error, adapter is broken
    /// - `AdapterError::Backpressure`: Backend overloaded, slow down
    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError>;

    /// Force flush any buffered data.
    ///
    /// Some adapters may buffer writes for efficiency. This method
    /// forces all buffered data to be persisted.
    async fn flush(&self) -> Result<(), AdapterError>;

    /// Gracefully shut down the adapter.
    ///
    /// This should flush any pending data and close connections.
    async fn shutdown(&self) -> Result<(), AdapterError>;

    /// Poll events from a single shard.
    ///
    /// # Parameters
    ///
    /// - `shard_id`: The shard to poll
    /// - `from_id`: Start cursor (exclusive). None means from the beginning.
    /// - `limit`: Maximum number of events to return
    ///
    /// # Returns
    ///
    /// A `ShardPollResult` containing the events and pagination info.
    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError>;

    /// Get the adapter name (for logging/metrics).
    fn name(&self) -> &'static str;

    /// Check if the adapter is healthy.
    ///
    /// Returns true if the adapter can accept batches.
    async fn is_healthy(&self) -> bool {
        true
    }
}

/// Wrapper to make `Box<dyn Adapter>` implement Adapter.
#[async_trait]
impl Adapter for Box<dyn Adapter> {
    async fn init(&mut self) -> Result<(), AdapterError> {
        (**self).init().await
    }

    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        (**self).on_batch(batch).await
    }

    async fn flush(&self) -> Result<(), AdapterError> {
        (**self).flush().await
    }

    async fn shutdown(&self) -> Result<(), AdapterError> {
        (**self).shutdown().await
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        (**self).poll_shard(shard_id, from_id, limit).await
    }

    fn name(&self) -> &'static str {
        (**self).name()
    }

    async fn is_healthy(&self) -> bool {
        (**self).is_healthy().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::InternalEvent;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_noop_adapter() {
        let mut adapter = NoopAdapter::new();
        adapter.init().await.unwrap();

        let events = vec![
            InternalEvent::from_value(json!({"test": 1}), 1, 0),
            InternalEvent::from_value(json!({"test": 2}), 2, 0),
        ];
        let batch = Batch::new(0, events, 0);

        adapter.on_batch(batch).await.unwrap();
        adapter.flush().await.unwrap();

        // Noop adapter doesn't store anything
        let result = adapter.poll_shard(0, None, 10).await.unwrap();
        assert!(result.events.is_empty());

        adapter.shutdown().await.unwrap();
    }

    #[test]
    fn test_shard_poll_result_empty() {
        let result = ShardPollResult::empty();
        assert!(result.events.is_empty());
        assert!(result.next_id.is_none());
        assert!(!result.has_more);
    }

    #[test]
    fn test_shard_poll_result_debug() {
        let result = ShardPollResult::empty();
        let debug = format!("{:?}", result);
        assert!(debug.contains("ShardPollResult"));
    }

    #[test]
    fn test_shard_poll_result_clone() {
        let mut result = ShardPollResult::empty();
        result.next_id = Some("cursor".to_string());
        result.has_more = true;

        let cloned = result.clone();
        assert_eq!(cloned.next_id, Some("cursor".to_string()));
        assert!(cloned.has_more);
    }

    #[tokio::test]
    async fn test_noop_adapter_name() {
        let adapter = NoopAdapter::new();
        assert_eq!(adapter.name(), "noop");
    }

    #[tokio::test]
    async fn test_noop_adapter_is_healthy() {
        let mut adapter = NoopAdapter::new();
        // Not healthy before init
        assert!(!adapter.is_healthy().await);
        // Healthy after init
        adapter.init().await.unwrap();
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn test_boxed_adapter() {
        let mut adapter: Box<dyn Adapter> = Box::new(NoopAdapter::new());

        // Test all trait methods through Box
        adapter.init().await.unwrap();
        assert_eq!(adapter.name(), "noop");
        assert!(adapter.is_healthy().await);

        let events = vec![InternalEvent::from_value(json!({"test": 1}), 1, 0)];
        let batch = Batch::new(0, events, 0);
        adapter.on_batch(batch).await.unwrap();

        adapter.flush().await.unwrap();

        let result = adapter.poll_shard(0, None, 10).await.unwrap();
        assert!(result.events.is_empty());

        adapter.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_arc_adapter() {
        let mut adapter = NoopAdapter::new();
        adapter.init().await.unwrap();

        let adapter: Arc<dyn Adapter> = Arc::new(adapter);

        // Test methods through Arc
        assert_eq!(adapter.name(), "noop");
        assert!(adapter.is_healthy().await);

        let events = vec![InternalEvent::from_value(json!({"test": 1}), 1, 0)];
        let batch = Batch::new(0, events, 0);
        adapter.on_batch(batch).await.unwrap();

        adapter.flush().await.unwrap();
        adapter.shutdown().await.unwrap();
    }
}
