//! # Net SDK
//!
//! Ergonomic Rust SDK for the Net mesh network.
//!
//! The core `net` crate is the engine. This SDK is what developers actually import.
//!
//! # Example
//!
//! ```rust,no_run
//! use net_sdk::{Net, Backpressure};
//! use futures::StreamExt;
//!
//! # async fn example() -> net_sdk::error::Result<()> {
//! let node = Net::builder()
//!     .shards(4)
//!     .backpressure(Backpressure::DropOldest)
//!     .memory()
//!     .build()
//!     .await?;
//!
//! // Emit events
//! node.emit(&serde_json::json!({"token": "hello"}))?;
//! node.emit_raw(b"{\"token\": \"world\"}" as &[u8])?;
//!
//! // Subscribe to a stream
//! let mut stream = node.subscribe(Default::default());
//! while let Some(event) = stream.next().await {
//!     let event = event?;
//!     println!("{}", event.raw_str());
//! }
//!
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
#[cfg(feature = "net")]
pub mod mesh;
mod net;
pub mod stream;

// Re-export the main handle.
pub use crate::net::{Net, PollRequest, PollResponse, Receipt, Stats};

// Re-export config types.
pub use crate::config::{Backpressure, NetBuilder};

// Re-export stream types.
pub use crate::stream::{EventStream, SubscribeOpts, TypedEventStream};

// Re-export core types that users will need.
pub use ::net::config::{BatchConfig, ScalingPolicy};
pub use ::net::consumer::Ordering;
pub use ::net::event::{Event, RawEvent, StoredEvent};
pub use ::net::Filter;

// Feature-gated re-exports.
#[cfg(feature = "redis")]
pub use ::net::config::RedisAdapterConfig;

#[cfg(feature = "jetstream")]
pub use ::net::config::JetStreamAdapterConfig;

#[cfg(feature = "net")]
pub use ::net::adapter::net::NetAdapterConfig;

#[cfg(feature = "net")]
pub use crate::mesh::{Mesh, MeshBuilder};

impl NetBuilder {
    /// Build and start the node.
    pub async fn build(self) -> error::Result<Net> {
        Net::from_builder(self).await
    }
}
