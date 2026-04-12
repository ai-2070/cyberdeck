//! # Blackstream
//!
//! High-performance, schema-agnostic, backend-agnostic event bus designed for
//! AI runtime workloads.
//!
//! ## Primary Use Case
//!
//! Blackstream is fundamentally designed to **ingest, relay, and replay
//! AI-generated streaming output** at GPU-native speeds. Target workloads include:
//!
//! - **Token streams**: LLM output tokens as they're generated
//! - **Multi-agent event flows**: Inter-agent communication in agentic systems
//! - **Tool-use streams**: Function calls, API invocations, tool results
//! - **Guardrail streams**: Safety checks, content filtering, policy enforcement
//! - **Consensus streams**: Multi-model voting, ensemble decisions
//! - **Structured-output parsing events**: JSON mode, schema validation
//! - **Retry/fallback trees**: Failure handling, alternative paths
//! - **Drift detection events**: Model behavior monitoring
//! - **Session lifecycle streams**: Conversation state, context management
//!
//! ## Performance Targets
//!
//! - **≥ 10 million events/sec sustained ingestion**
//! - **≥ 100 million events/sec microburst tolerance**
//! - **< 1μs p99 ingestion latency**
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use blackstream::{EventBus, EventBusConfig, Event};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create event bus with default configuration
//!     let bus = EventBus::new(EventBusConfig::default()).await?;
//!
//!     // Ingest events (non-blocking)
//!     let event = Event::from_str(r#"{"token": "hello", "index": 0}"#)?;
//!     bus.ingest(event)?;
//!
//!     // Poll events
//!     let response = bus.poll(Default::default()).await?;
//!     for event in response.events {
//!         println!("{:?}", event.raw);
//!     }
//!
//!     bus.shutdown().await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]
#![recursion_limit = "256"]

pub mod config;
pub mod error;
pub mod event;
pub mod timestamp;

pub mod adapter;
pub mod bus;
pub mod consumer;
pub mod shard;

pub use timestamp::TimestampGenerator;

pub mod ffi;

// Re-exports for convenience
pub use bus::{EventBus, EventBusStats};
#[cfg(feature = "jetstream")]
pub use config::JetStreamAdapterConfig;
#[cfg(feature = "redis")]
pub use config::RedisAdapterConfig;
pub use config::ScalingPolicy;
pub use config::{
    AdapterConfig, BackpressureMode, BatchConfig, ConfigError, EventBusConfig,
    EventBusConfigBuilder,
};
pub use consumer::{ConsumeRequest, ConsumeResponse, Filter, Ordering};
pub use error::{
    AdapterError, AdapterResult, ConsumerError, ConsumerResult, IngestionError, IngestionResult,
};
pub use event::{Batch, Event, InternalEvent, RawEvent, StoredEvent};
pub use shard::{ScalingDecision, ScalingError, ShardMetrics};
