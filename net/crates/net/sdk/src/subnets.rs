//! Subnet assignment — hierarchical 4-level grouping for routing and
//! visibility.
//!
//! `SubnetId` is a bit-packed `u32` encoding up to four levels of
//! nesting. `SubnetPolicy` maps capability tags onto a subnet id so
//! assignment can derive from what a node declares it is.
//! `SubnetGateway` decides at forwarding time whether a packet
//! addressed to one subnet should be forwarded to another.
//!
//! # Example
//!
//! ```
//! use net_sdk::subnets::{SubnetId, SubnetPolicy, SubnetRule};
//!
//! // Policy: put nodes tagged `region:eu` into subnet level 0 bucket 1.
//! let policy = SubnetPolicy::new().add_rule(
//!     SubnetRule::new("region:", 0).map("eu", 1).map("us", 2),
//! );
//! let _ = policy;
//!
//! // Global subnet = "no restriction".
//! assert!(SubnetId::GLOBAL.is_global());
//! ```
//!
//! # Scope in this stage
//!
//! Re-exports only. The builder hook (`MeshBuilder::subnet(...)`) and
//! gateway wiring land in a later stage; today users can construct a
//! `SubnetPolicy` but the mesh does not yet consume it.

pub use net::adapter::net::subnet::{
    DropReason, ForwardDecision, SubnetGateway, SubnetId, SubnetPolicy, SubnetRule,
};

// `Visibility` isn't strictly in the subnet module — it lives on
// channel config — but it pairs with subnets semantically. Re-export
// here so security-surface users get it from one place.
pub use net::adapter::net::Visibility;
