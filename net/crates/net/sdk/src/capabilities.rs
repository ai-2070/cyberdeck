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
//! # Scope in this stage
//!
//! This module is re-exports only — network-side primitives
//! (`Mesh::announce_capabilities`, `Mesh::find_peers`) land in a
//! later stage. Today users can build and pattern-match
//! `CapabilitySet`s locally (e.g. for a scheduler) but can't emit
//! them over the mesh.

pub use net::adapter::net::behavior::capability::{
    AcceleratorInfo, AcceleratorType, CapabilityAnnouncement, CapabilityFilter, CapabilityIndex,
    CapabilityIndexStats, CapabilityRequirement, CapabilitySet, GpuInfo, GpuVendor,
    HardwareCapabilities, IndexedNode, Modality, ModelCapability, ResourceLimits, Signature64,
    SoftwareCapabilities, ToolCapability,
};
