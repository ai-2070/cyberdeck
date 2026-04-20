//! Capability declarations — what a node can do.
//!
//! `CapabilitySet` describes hardware (CPU / GPU / memory), software
//! (OS / runtimes / frameworks), loaded models, available tools, and
//! free-form tags. `CapabilityFilter` lets peers query for nodes that
//! match a requirement; `CapabilityAnnouncement` is the signed
//! on-wire form.
//!
//! # Example
//!
//! ```
//! use net_sdk::capabilities::{
//!     CapabilityFilter, CapabilitySet, GpuInfo, GpuVendor, HardwareCapabilities,
//! };
//!
//! // Declare: "this node has one RTX 4090 + 64 GB RAM, tagged `prod`".
//! let hw = HardwareCapabilities::new()
//!     .with_cpu(16, 32)
//!     .with_memory(65_536)
//!     .with_gpu(GpuInfo::new(GpuVendor::Nvidia, "RTX 4090", 24_576));
//! let caps = CapabilitySet::new().with_hardware(hw).add_tag("prod");
//!
//! // Match-ability: does this node satisfy "needs GPU ≥ 16 GB VRAM"?
//! let filter = CapabilityFilter::new().require_gpu().with_min_vram(16_384);
//! assert!(filter.matches(&caps));
//! ```
//!
//! # Cross-node (direct-peer, one-hop)
//!
//! With `--features "net capabilities"`, `Mesh` has
//! [`announce_capabilities`](crate::mesh::Mesh::announce_capabilities)
//! and [`find_peers`](crate::mesh::Mesh::find_peers). Announce-side
//! self-indexes, so a single-node test is round-trippable:
//!
//! ```
//! # #[cfg(all(feature = "net", feature = "capabilities"))]
//! # async fn doc() -> net_sdk::error::Result<()> {
//! use net_sdk::capabilities::{CapabilityFilter, CapabilitySet};
//! use net_sdk::mesh::MeshBuilder;
//!
//! let node = MeshBuilder::new("127.0.0.1:0", &[0x42u8; 32])?
//!     .build()
//!     .await?;
//!
//! // Announce; also self-indexes.
//! node.announce_capabilities(CapabilitySet::new().add_tag("gpu"))
//!     .await?;
//!
//! // Self-match hits.
//! let hits = node.find_peers(&CapabilityFilter::new().require_tag("gpu"));
//! assert!(hits.contains(&node.node_id()));
//!
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Multi-hop propagation is deferred — peers more than one hop away
//! will not see the announcement. TTL + GC eviction match the core
//! defaults (`capability_gc_interval` on [`net::adapter::net::MeshNodeConfig`]).

pub use net::adapter::net::behavior::capability::{
    AcceleratorInfo, AcceleratorType, CapabilityAnnouncement, CapabilityFilter, CapabilityIndex,
    CapabilityIndexStats, CapabilityRequirement, CapabilitySet, GpuInfo, GpuVendor,
    HardwareCapabilities, IndexedNode, Modality, ModelCapability, ResourceLimits, Signature64,
    SoftwareCapabilities, ToolCapability,
};
