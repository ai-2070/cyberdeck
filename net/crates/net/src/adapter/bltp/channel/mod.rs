//! Layer 2: Channels & Authorization for BLTP.
//!
//! Channels are named, policy-bearing logical endpoints. Access control
//! uses the existing capability system (`CapabilityFilter`) combined with
//! L1 permission tokens. Wire-speed authorization via bloom filter.

mod config;
mod guard;
mod name;

pub use config::{ChannelConfig, ChannelConfigRegistry, Visibility};
pub use guard::{AuthGuard, AuthVerdict};
pub use name::{channel_hash, ChannelError, ChannelId, ChannelName, ChannelRegistry};
