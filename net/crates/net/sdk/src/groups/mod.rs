//! HA / scaling overlays on top of [`DaemonRuntime`](crate::compute::DaemonRuntime).
//!
//! Three coordination primitives implemented by the core compute
//! layer (`adapter::net::compute::{replica_group, fork_group,
//! standby_group}`), wrapped here with SDK-ergonomic surfaces:
//!
//! - [`ReplicaGroup`] — N interchangeable copies of a daemon.
//!   Deterministic identity per replica index; LB-routed inbound
//!   events; auto-replacement on node failure.
//! - [`ForkGroup`] — N independent daemons forked from a common
//!   parent at a specific sequence. Unique identities, shared
//!   ancestry via `ForkRecord`.
//! - [`StandbyGroup`] — active-passive replication. One active
//!   processes events; N−1 standbys hold snapshots and catch up
//!   via `sync_standbys`; on active failure `promote` picks the
//!   most-synced standby.
//!
//! Each group takes a `&DaemonRuntime` (to reach the shared
//! scheduler / registry / factory map) and a `kind` string that
//! must have been registered via
//! [`DaemonRuntime::register_factory`](crate::compute::DaemonRuntime::register_factory).
//! The kind factory is invoked once per group member — at spawn,
//! on `scale_to` growth, and on `on_node_failure` replacement.
//!
//! # Thread safety
//!
//! Each group wraps its inner core state in an `Arc<Mutex<_>>`, so
//! all SDK methods take `&self`. Contention is negligible because
//! group operations happen on seconds-to-minutes timescales
//! (scale-ups, failover); event routing (`route_event`) reads the
//! coordinator's live-member snapshot without contending on a hot
//! path.

pub mod common;
pub mod error;
pub mod fork;
pub mod replica;
pub mod standby;

pub use common::{ForkRecord, GroupHealth, MemberInfo, MemberRole, RequestContext};
pub use error::GroupError;
pub use fork::{ForkGroup, ForkGroupConfig};
pub use replica::{ReplicaGroup, ReplicaGroupConfig};
pub use standby::{StandbyGroup, StandbyGroupConfig};
