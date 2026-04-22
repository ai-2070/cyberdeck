//! Shared types surfaced by all three groups.
//!
//! These are thin re-exports of core types that cross the SDK
//! boundary in return positions — callers need them to inspect
//! group membership / health / routing, so they're part of the
//! public surface.

pub use ::net::adapter::net::behavior::loadbalance::{RequestContext, Strategy};
pub use ::net::adapter::net::compute::{fork_group::ForkInfo, GroupHealth, MemberInfo, MemberRole};
pub use ::net::adapter::net::continuity::discontinuity::ForkRecord;

/// Convenience constructor for a [`RequestContext`] with a
/// routing-key for stickiness. Most SDK callers only want
/// consistent-hash routing on a single key; everything else on
/// the context (session, zones, tags) has a sensible default.
/// Callers who need finer control build `RequestContext` directly.
pub fn sticky_context(routing_key: impl Into<String>) -> RequestContext {
    RequestContext::new().with_routing_key(routing_key)
}
