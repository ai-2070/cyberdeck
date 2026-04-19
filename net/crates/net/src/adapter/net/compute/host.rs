//! DaemonHost — runtime wrapper for a MeshDaemon.
//!
//! Owns the causal infrastructure (chain builder, horizon) and wraps
//! daemon outputs in CausalLinks. The daemon only sees events and
//! produces payloads — all chain management is the host's job.

use super::daemon::{DaemonError, DaemonHostConfig, DaemonStats, MeshDaemon, ResourceUsage};
use crate::adapter::net::behavior::capability::CapabilityFilter;
use crate::adapter::net::identity::EntityKeypair;
use crate::adapter::net::state::causal::{CausalChainBuilder, CausalEvent};
use crate::adapter::net::state::horizon::ObservedHorizon;
use crate::adapter::net::state::snapshot::StateSnapshot;

/// Runtime wrapper for a `MeshDaemon`.
///
/// Manages the daemon's causal chain, observed horizon, and snapshot lifecycle.
/// Each `DaemonHost` has its own `EntityKeypair` — its identity in the mesh.
pub struct DaemonHost {
    /// The daemon implementation.
    daemon: Box<dyn MeshDaemon>,
    /// Daemon's identity in the mesh.
    keypair: EntityKeypair,
    /// Produces causally-linked output events.
    chain: CausalChainBuilder,
    /// Tracks what this daemon has observed from other entities.
    horizon: ObservedHorizon,
    /// Host configuration.
    config: DaemonHostConfig,
    /// Runtime statistics.
    stats: DaemonStats,
    /// Host construction time, for uptime.
    created_at: quanta::Instant,
    /// Cumulative wall-clock nanoseconds spent in `MeshDaemon::process`.
    cumulative_process_ns: u64,
}

impl DaemonHost {
    /// Create a new host with a genesis chain.
    pub fn new(
        daemon: Box<dyn MeshDaemon>,
        keypair: EntityKeypair,
        config: DaemonHostConfig,
    ) -> Self {
        let chain = CausalChainBuilder::new(keypair.origin_hash());
        Self {
            daemon,
            keypair,
            chain,
            horizon: ObservedHorizon::new(),
            config,
            stats: DaemonStats::default(),
            created_at: quanta::Instant::now(),
            cumulative_process_ns: 0,
        }
    }

    /// Create a host from a fork.
    ///
    /// Uses a pre-built `CausalChainBuilder` from `fork_entity()` whose
    /// genesis link carries the fork sentinel as `parent_hash`. The daemon
    /// starts fresh (no state to restore) but its chain documents lineage.
    ///
    /// Validates that the chain's origin_hash matches the keypair to prevent
    /// identity/chain mismatches.
    pub fn from_fork(
        daemon: Box<dyn MeshDaemon>,
        keypair: EntityKeypair,
        chain: CausalChainBuilder,
        config: DaemonHostConfig,
    ) -> Self {
        assert_eq!(
            chain.origin_hash(),
            keypair.origin_hash(),
            "fork chain origin {:#x} does not match keypair origin {:#x}",
            chain.origin_hash(),
            keypair.origin_hash(),
        );
        Self {
            daemon,
            keypair,
            chain,
            horizon: ObservedHorizon::new(),
            config,
            stats: DaemonStats::default(),
            created_at: quanta::Instant::now(),
            cumulative_process_ns: 0,
        }
    }

    /// Restore from an L4 `StateSnapshot`.
    ///
    /// Rebuilds the causal chain from the snapshot's head link and calls
    /// `daemon.restore()` with the serialized state.
    pub fn from_snapshot(
        mut daemon: Box<dyn MeshDaemon>,
        keypair: EntityKeypair,
        snapshot: &StateSnapshot,
        config: DaemonHostConfig,
    ) -> Result<Self, DaemonError> {
        // Validate snapshot belongs to this keypair
        if snapshot.entity_id != *keypair.entity_id() {
            return Err(DaemonError::RestoreFailed(format!(
                "snapshot entity {:?} does not match keypair entity {:?}",
                snapshot.entity_id,
                keypair.entity_id()
            )));
        }

        // Restore daemon state
        daemon.restore(snapshot.state.clone())?;

        // Rebuild chain from snapshot head. Use the snapshot state as the
        // head payload so the next event's parent_hash is computed correctly.
        let chain = CausalChainBuilder::from_head(snapshot.chain_link, snapshot.state.clone());

        Ok(Self {
            daemon,
            keypair,
            chain,
            horizon: snapshot.horizon.clone(),
            config,
            stats: DaemonStats::default(),
            created_at: quanta::Instant::now(),
            cumulative_process_ns: 0,
        })
    }

    /// Deliver an inbound causal event to the daemon.
    ///
    /// Updates the observed horizon, calls `daemon.process()`, and wraps
    /// any outputs in CausalLinks via the chain builder.
    ///
    /// Returns the wrapped output events (ready to send on the mesh).
    pub fn deliver(&mut self, event: &CausalEvent) -> Result<Vec<CausalEvent>, DaemonError> {
        // Update horizon with what we've observed
        self.horizon
            .observe(event.link.origin_hash, event.link.sequence);

        // Process the event
        let started = quanta::Instant::now();
        let result = self.daemon.process(event);
        let elapsed_ns = started.elapsed().as_nanos() as u64;
        self.cumulative_process_ns = self.cumulative_process_ns.saturating_add(elapsed_ns);

        let outputs = match result {
            Ok(outputs) => outputs,
            Err(e) => {
                self.stats.errors += 1;
                return Err(e);
            }
        };

        self.stats.events_processed += 1;

        // Wrap each output payload in a causal link
        let horizon_encoded = self.horizon.encode();
        let mut causal_outputs = Vec::with_capacity(outputs.len());
        for payload in outputs {
            let event = self
                .chain
                .append(payload, horizon_encoded)
                .ok_or_else(|| DaemonError::ProcessFailed("causal sequence overflow".into()))?;
            self.stats.events_emitted += 1;
            causal_outputs.push(event);
        }

        Ok(causal_outputs)
    }

    /// Take a snapshot of the daemon's current state.
    ///
    /// Returns `None` if the daemon is stateless (`snapshot()` returns `None`).
    pub fn take_snapshot(&self) -> Option<StateSnapshot> {
        let state = self.daemon.snapshot()?;
        Some(StateSnapshot::new(
            self.keypair.entity_id().clone(),
            *self.chain.head(),
            state,
            self.horizon.clone(),
        ))
    }

    /// Get the daemon's entity ID.
    #[inline]
    pub fn entity_id(&self) -> &crate::adapter::net::identity::EntityId {
        self.keypair.entity_id()
    }

    /// Get the daemon's origin hash.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.keypair.origin_hash()
    }

    /// Get the daemon's capability requirements.
    #[inline]
    pub fn requirements(&self) -> CapabilityFilter {
        self.daemon.requirements()
    }

    /// Get the daemon's name.
    #[inline]
    pub fn name(&self) -> &str {
        self.daemon.name()
    }

    /// Get the current chain sequence number.
    #[inline]
    pub fn sequence(&self) -> u64 {
        self.chain.sequence()
    }

    /// Get runtime statistics.
    #[inline]
    pub fn stats(&self) -> &DaemonStats {
        &self.stats
    }

    /// Snapshot of daemon consumption.
    pub fn resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            events_processed: self.stats.events_processed,
            events_emitted: self.stats.events_emitted,
            errors: self.stats.errors,
            uptime_secs: self.created_at.elapsed().as_secs(),
            cumulative_process_ns: self.cumulative_process_ns,
        }
    }

    /// Get the daemon host configuration.
    #[inline]
    pub fn config(&self) -> &DaemonHostConfig {
        &self.config
    }
}

impl std::fmt::Debug for DaemonHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonHost")
            .field("name", &self.daemon.name())
            .field("origin_hash", &format!("{:#x}", self.origin_hash()))
            .field("sequence", &self.sequence())
            .field("stats", &self.stats)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::state::causal::CausalLink;
    use bytes::Bytes;

    /// A simple stateless echo daemon for testing.
    struct EchoDaemon;

    impl MeshDaemon for EchoDaemon {
        fn name(&self) -> &str {
            "echo"
        }

        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }

        fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            // Echo the payload back
            Ok(vec![event.payload.clone()])
        }
    }

    /// A stateful counter daemon for testing.
    struct CounterDaemon {
        count: u64,
    }

    impl CounterDaemon {
        fn new() -> Self {
            Self { count: 0 }
        }
    }

    impl MeshDaemon for CounterDaemon {
        fn name(&self) -> &str {
            "counter"
        }

        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }

        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            self.count += 1;
            Ok(vec![Bytes::from(self.count.to_le_bytes().to_vec())])
        }

        fn snapshot(&self) -> Option<Bytes> {
            Some(Bytes::from(self.count.to_le_bytes().to_vec()))
        }

        fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
            if state.len() != 8 {
                return Err(DaemonError::RestoreFailed("bad state size".into()));
            }
            self.count = u64::from_le_bytes(state[..8].try_into().unwrap());
            Ok(())
        }
    }

    fn make_event(origin: u32, seq: u64, payload: &[u8]) -> CausalEvent {
        CausalEvent {
            link: CausalLink {
                origin_hash: origin,
                horizon_encoded: 0,
                sequence: seq,
                parent_hash: 0,
            },
            payload: Bytes::copy_from_slice(payload),
            received_at: 0,
        }
    }

    #[test]
    fn test_echo_daemon() {
        let kp = EntityKeypair::generate();
        let mut host = DaemonHost::new(Box::new(EchoDaemon), kp, DaemonHostConfig::default());

        let event = make_event(0xAAAA, 1, b"hello");
        let outputs = host.deliver(&event).unwrap();

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].payload, Bytes::from_static(b"hello"));
        assert_eq!(outputs[0].link.sequence, 1); // first output
        assert_eq!(host.stats().events_processed, 1);
        assert_eq!(host.stats().events_emitted, 1);
    }

    #[test]
    fn test_counter_daemon() {
        let kp = EntityKeypair::generate();
        let mut host = DaemonHost::new(
            Box::new(CounterDaemon::new()),
            kp,
            DaemonHostConfig::default(),
        );

        for i in 1..=5 {
            let event = make_event(0xBBBB, i, b"tick");
            let outputs = host.deliver(&event).unwrap();
            assert_eq!(outputs.len(), 1);

            let count = u64::from_le_bytes(outputs[0].payload[..8].try_into().unwrap());
            assert_eq!(count, i);
        }

        assert_eq!(host.sequence(), 5);
        assert_eq!(host.stats().events_processed, 5);
    }

    #[test]
    fn test_stateless_snapshot_is_none() {
        let kp = EntityKeypair::generate();
        let host = DaemonHost::new(Box::new(EchoDaemon), kp, DaemonHostConfig::default());

        assert!(host.take_snapshot().is_none());
    }

    #[test]
    fn test_stateful_snapshot_and_restore() {
        let kp = EntityKeypair::generate();
        let mut host = DaemonHost::new(
            Box::new(CounterDaemon::new()),
            kp.clone(),
            DaemonHostConfig::default(),
        );

        // Process some events
        for i in 1..=10 {
            let event = make_event(0xCCCC, i, b"tick");
            host.deliver(&event).unwrap();
        }

        // Take snapshot
        let snapshot = host.take_snapshot().unwrap();
        assert_eq!(snapshot.through_seq, 10);

        // Restore on a new host
        let kp2 = kp.clone();
        let mut restored = DaemonHost::from_snapshot(
            Box::new(CounterDaemon::new()),
            kp2,
            &snapshot,
            DaemonHostConfig::default(),
        )
        .unwrap();

        // Next event should continue counting from 10
        let event = make_event(0xCCCC, 11, b"tick");
        let outputs = restored.deliver(&event).unwrap();
        let count = u64::from_le_bytes(outputs[0].payload[..8].try_into().unwrap());
        assert_eq!(count, 11);
    }

    #[test]
    fn test_chain_continuity_across_events() {
        let kp = EntityKeypair::generate();
        let mut host = DaemonHost::new(Box::new(EchoDaemon), kp, DaemonHostConfig::default());

        let mut prev_link = None;
        for i in 1..=5 {
            let event = make_event(0xDDDD, i, b"data");
            let outputs = host.deliver(&event).unwrap();

            let link = outputs[0].link;
            assert_eq!(link.sequence, i);
            assert_eq!(link.origin_hash, host.origin_hash());

            if let Some(prev) = prev_link {
                // parent_hash should link to previous
                assert_ne!(link.parent_hash, 0);
                assert_ne!(link.parent_hash, prev);
            }
            prev_link = Some(link.parent_hash);
        }
    }

    #[test]
    fn test_horizon_updated_before_process() {
        let kp = EntityKeypair::generate();
        let mut host = DaemonHost::new(Box::new(EchoDaemon), kp, DaemonHostConfig::default());

        let event = make_event(0xEEEE, 42, b"test");
        let outputs = host.deliver(&event).unwrap();

        // Output should carry horizon info about the observed event
        assert_ne!(outputs[0].link.horizon_encoded, 0);
    }

    #[test]
    fn test_resource_usage_accumulates_process_time() {
        // A daemon that sleeps a known amount in process(), so we can sanity-check
        // the cumulative_process_ns accumulator.
        struct SleepyDaemon {
            sleep_ns: u64,
        }
        impl MeshDaemon for SleepyDaemon {
            fn name(&self) -> &str {
                "sleepy"
            }
            fn requirements(&self) -> CapabilityFilter {
                CapabilityFilter::default()
            }
            fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
                std::thread::sleep(std::time::Duration::from_nanos(self.sleep_ns));
                Ok(vec![])
            }
        }

        let kp = EntityKeypair::generate();
        let sleep_ns = 2_000_000; // 2 ms per call, well above timer jitter
        let mut host = DaemonHost::new(
            Box::new(SleepyDaemon { sleep_ns }),
            kp,
            DaemonHostConfig::default(),
        );

        let u0 = host.resource_usage();
        assert_eq!(u0.cumulative_process_ns, 0);
        assert_eq!(u0.events_processed, 0);

        for i in 1..=3 {
            let event = make_event(0xABCD, i, b"tick");
            host.deliver(&event).unwrap();
        }

        let u1 = host.resource_usage();
        assert_eq!(u1.events_processed, 3);
        // Three sleeps of `sleep_ns` each — wall clock must meet at least that.
        assert!(
            u1.cumulative_process_ns >= 3 * sleep_ns,
            "cumulative_process_ns {} should be >= {}",
            u1.cumulative_process_ns,
            3 * sleep_ns,
        );

        // Monotonic: next read must not go backwards.
        let u2 = host.resource_usage();
        assert!(u2.cumulative_process_ns >= u1.cumulative_process_ns);
        assert!(u2.uptime_secs >= u1.uptime_secs);
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_from_snapshot_rejects_wrong_keypair() {
        // Regression: from_snapshot accepted any snapshot regardless of
        // entity identity, allowing chain/identity divergence.
        let kp_a = EntityKeypair::generate();
        let kp_b = EntityKeypair::generate();

        // Create snapshot for entity A
        let chain = CausalChainBuilder::new(kp_a.origin_hash());
        let snapshot = StateSnapshot::new(
            kp_a.entity_id().clone(),
            *chain.head(),
            Bytes::from_static(b"state"),
            ObservedHorizon::new(),
        );

        // Try to restore on entity B — must fail
        let result = DaemonHost::from_snapshot(
            Box::new(EchoDaemon),
            kp_b,
            &snapshot,
            DaemonHostConfig::default(),
        );
        assert!(
            result.is_err(),
            "from_snapshot must reject snapshot from a different entity"
        );
    }
}
