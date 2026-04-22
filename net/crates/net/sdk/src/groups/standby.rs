//! SDK wrapper around `adapter::net::compute::StandbyGroup`.
//!
//! Active-passive replication. One member processes events; N−1
//! standbys hold snapshots and catch up via `sync_standbys`. On
//! active failure, `promote` picks the standby with the highest
//! `synced_through` sequence, re-keys it to active, and replays
//! any events that arrived after the last sync.
//!
//! # Usage pattern
//!
//! ```ignore
//! let group = StandbyGroup::spawn(&rt, "counter", cfg)?;
//!
//! // Every event deliver:
//! rt.deliver(group.active_origin(), &event)?;
//! group.on_event_delivered(event);    // buffer for replay
//!
//! // Periodic catchup:
//! group.sync_standbys()?;
//!
//! // On active node failure:
//! group.on_node_failure(failed_node)?;  // may auto-promote
//! ```
//!
//! The `on_event_delivered` call is manual — SDK doesn't
//! auto-hook into `DaemonRuntime::deliver`. See the plan doc's
//! "Open questions" section for the rationale.

use std::sync::{Arc, Mutex};

use ::net::adapter::net::compute::DaemonHostConfig;
use ::net::adapter::net::compute::{
    standby_group::StandbyGroup as CoreStandbyGroup,
    standby_group::StandbyGroupConfig as CoreStandbyGroupConfig,
};
use ::net::adapter::net::state::causal::CausalEvent;

use crate::compute::DaemonRuntime;
use crate::groups::common::{GroupHealth, MemberInfo, MemberRole};
use crate::groups::error::GroupError;

/// Configuration for a standby group.
#[derive(Debug, Clone)]
pub struct StandbyGroupConfig {
    /// Total members (1 active + N−1 standbys). Must be ≥ 2 or
    /// the core rejects with `InvalidConfig`.
    pub member_count: u8,
    /// 32-byte seed for deterministic per-member keypair derivation.
    pub group_seed: [u8; 32],
    /// Daemon host configuration applied to every member.
    pub host_config: DaemonHostConfig,
}

impl From<StandbyGroupConfig> for CoreStandbyGroupConfig {
    fn from(cfg: StandbyGroupConfig) -> Self {
        CoreStandbyGroupConfig {
            member_count: cfg.member_count,
            group_seed: cfg.group_seed,
            host_config: cfg.host_config,
        }
    }
}

/// A standby group. See module docs for the usage pattern.
pub struct StandbyGroup {
    inner: Arc<Mutex<CoreStandbyGroup>>,
    runtime: DaemonRuntime,
}

impl StandbyGroup {
    /// Spawn a standby group. Member 0 starts as active; the rest
    /// start as standbys with no snapshot (`synced_through == 0`).
    pub fn spawn(
        runtime: &DaemonRuntime,
        kind: &str,
        config: StandbyGroupConfig,
    ) -> Result<Self, GroupError> {
        if !runtime.is_ready_pub() {
            return Err(GroupError::NotReady);
        }
        let factory = runtime
            .factory_for_kind_pub(kind)
            .map_err(|_| GroupError::FactoryNotFound(kind.to_string()))?;
        let scheduler = runtime.scheduler_arc();
        let registry = runtime.registry_arc();
        let core =
            CoreStandbyGroup::spawn(config.into(), move || (factory)(), &scheduler, &registry)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(core)),
            runtime: runtime.clone(),
        })
    }

    /// `origin_hash` of the current active member. Events always
    /// go through the active; standbys don't process inputs.
    pub fn active_origin(&self) -> u32 {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .active_origin()
    }

    /// Buffer an event for standby replay. Call after every
    /// `DaemonRuntime::deliver` to the active — on promotion the
    /// new active replays events that arrived between the last
    /// successful `sync_standbys` and the failure.
    pub fn on_event_delivered(&self, event: CausalEvent) {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .on_event_delivered(event);
    }

    /// Snapshot the active and push to every standby. Returns the
    /// sequence number through which the sync caught up.
    pub fn sync_standbys(&self) -> Result<u64, GroupError> {
        let registry = self.runtime.registry_arc();
        let mut guard = self.inner.lock().expect("StandbyGroup mutex poisoned");
        Ok(guard.sync_standbys(&registry)?)
    }

    /// Promote the most-synced standby to active. Used
    /// automatically by `on_node_failure` when the active fails;
    /// can also be called manually for planned failover.
    /// Returns the promoted member's new `origin_hash` (stays
    /// the same as before — keypair is re-derived deterministically).
    pub fn promote(&self, kind: &str) -> Result<u32, GroupError> {
        let factory = self
            .runtime
            .factory_for_kind_pub(kind)
            .map_err(|_| GroupError::FactoryNotFound(kind.to_string()))?;
        let scheduler = self.runtime.scheduler_arc();
        let registry = self.runtime.registry_arc();
        let mut guard = self.inner.lock().expect("StandbyGroup mutex poisoned");
        Ok(guard.promote(move || (factory)(), &registry, &scheduler)?)
    }

    /// Handle node failure. If the active was on `failed_node_id`,
    /// auto-promotes the most-synced standby and returns its
    /// `origin_hash`. If only standbys were affected, returns
    /// `None` — the caller can re-sync those standbys later.
    pub fn on_node_failure(
        &self,
        failed_node_id: u64,
        kind: &str,
    ) -> Result<Option<u32>, GroupError> {
        let factory = self
            .runtime
            .factory_for_kind_pub(kind)
            .map_err(|_| GroupError::FactoryNotFound(kind.to_string()))?;
        let scheduler = self.runtime.scheduler_arc();
        let registry = self.runtime.registry_arc();
        let mut guard = self.inner.lock().expect("StandbyGroup mutex poisoned");
        Ok(guard.on_node_failure(failed_node_id, move || (factory)(), &scheduler, &registry)?)
    }

    pub fn on_node_recovery(&self, recovered_node_id: u64) {
        let registry = self.runtime.registry_arc();
        let mut guard = self.inner.lock().expect("StandbyGroup mutex poisoned");
        guard.on_node_recovery(recovered_node_id, &registry);
    }

    pub fn health(&self) -> GroupHealth {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .health()
    }

    pub fn active_healthy(&self) -> bool {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .active_healthy()
    }

    pub fn active_index(&self) -> u8 {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .active_index()
    }

    pub fn member_role(&self, index: u8) -> Option<MemberRole> {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .member_role(index)
    }

    pub fn synced_through(&self, index: u8) -> Option<u64> {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .synced_through(index)
    }

    pub fn buffered_event_count(&self) -> usize {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .buffered_event_count()
    }

    pub fn group_id(&self) -> u32 {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .group_id()
    }

    pub fn members(&self) -> Vec<MemberInfo> {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .members()
            .to_vec()
    }

    pub fn member_count(&self) -> u8 {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .member_count()
    }

    pub fn standby_count(&self) -> u8 {
        self.inner
            .lock()
            .expect("StandbyGroup mutex poisoned")
            .standby_count()
    }
}

impl std::fmt::Debug for StandbyGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.inner.lock().expect("StandbyGroup mutex poisoned");
        f.debug_struct("StandbyGroup")
            .field("group_id", &format_args!("{:#x}", guard.group_id()))
            .field("active_index", &guard.active_index())
            .field("member_count", &guard.member_count())
            .field("buffered_events", &guard.buffered_event_count())
            .finish()
    }
}
