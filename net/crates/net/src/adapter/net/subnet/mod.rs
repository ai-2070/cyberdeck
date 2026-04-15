//! Layer 3: Subnets & Hierarchy for Net.
//!
//! Subnets are label-based — nodes belong by identity/capability, not
//! static configuration. The hierarchy is encoded as 4 levels of 8 bits
//! each in the `subnet_id: u32` header field. Gateway nodes enforce
//! visibility policy at subnet boundaries.

mod assignment;
mod gateway;
mod id;

pub use assignment::{SubnetPolicy, SubnetRule};
pub use gateway::{DropReason, ForwardDecision, SubnetGateway};
pub use id::SubnetId;
