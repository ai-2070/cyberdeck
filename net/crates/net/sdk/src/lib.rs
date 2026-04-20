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
//!     println!("{}", event.raw_str().unwrap_or("<non-utf8>"));
//! }
//!
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
#[cfg(feature = "cortex")]
pub mod cortex;
pub mod error;
#[cfg(feature = "net")]
pub mod mesh;
mod net;
pub mod stream;

// Security surface — each sub-surface is independently gated so
// consumers can pick only what they need. The `security` meta-feature
// enables all three together.
#[cfg(feature = "capabilities")]
pub mod capabilities;
#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "subnets")]
pub mod subnets;

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
pub use ::net::adapter::net::{
    CloseBehavior, Reliability, Stream as MeshStream, StreamConfig, StreamStats,
};

// Channel (distributed pub/sub) types. Ship alongside `net` because
// they live on the mesh transport — subscribing / publishing require
// a live `Mesh`.
#[cfg(feature = "net")]
pub use ::net::adapter::net::{
    AckReason, ChannelConfig, ChannelId, ChannelName, OnFailure, PublishConfig, PublishReport,
    Visibility,
};

#[cfg(feature = "net")]
pub use crate::mesh::{Mesh, MeshBuilder};

// Convenience re-exports for the common security types, so users can
// `use net_sdk::{Identity, TokenScope};` without reaching for a
// sub-module path.
#[cfg(feature = "capabilities")]
pub use crate::capabilities::{CapabilityFilter, CapabilitySet};
#[cfg(feature = "identity")]
pub use crate::identity::{Identity, PermissionToken, TokenError, TokenScope};
#[cfg(feature = "subnets")]
pub use crate::subnets::{SubnetId, SubnetPolicy};

impl NetBuilder {
    /// Build and start the node.
    pub async fn build(self) -> error::Result<Net> {
        Net::from_builder(self).await
    }
}
