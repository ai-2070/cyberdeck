//! Integration tests for the MIKOSHI live migration system.
//!
//! These tests exercise the full migration lifecycle across the orchestrator,
//! source handler, and target handler — verifying that the components compose
//! correctly end-to-end.

#![cfg(feature = "net")]

use std::sync::Arc;

use bytes::Bytes;
use net::adapter::net::behavior::capability::{
    CapabilityAnnouncement, CapabilityFilter, CapabilityIndex, CapabilitySet,
};
use net::adapter::net::compute::{
    chunk_snapshot, DaemonError, DaemonHost, DaemonHostConfig, DaemonRegistry, MeshDaemon,
    MigrationMessage, MigrationOrchestrator, MigrationPhase, MigrationSourceHandler,
    MigrationTargetHandler, Scheduler, SnapshotReassembler, MAX_SNAPSHOT_CHUNK_SIZE,
};
use net::adapter::net::identity::EntityKeypair;
use net::adapter::net::state::causal::{CausalEvent, CausalLink};
use net::adapter::net::state::snapshot::StateSnapshot;
use net::adapter::net::subprotocol::SubprotocolRegistry;

// ── Test daemon ──────────────────────────────────────────────────────────────

/// A stateful counter daemon for testing. Each event increments the counter.
/// Snapshot serializes the count as 8 LE bytes; restore deserializes it.
struct CounterDaemon {
    count: u64,
}

impl CounterDaemon {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn with_count(count: u64) -> Self {
        Self { count }
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

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_event(origin: u32, seq: u64) -> CausalEvent {
    CausalEvent {
        link: CausalLink {
            origin_hash: origin,
            horizon_encoded: 0,
            sequence: seq,
            parent_hash: 0,
        },
        payload: Bytes::from(format!("event-{}", seq)),
        received_at: seq * 1000,
    }
}

fn register_counter_daemon(
    registry: &DaemonRegistry,
    initial_count: u64,
) -> (EntityKeypair, u32) {
    let kp = EntityKeypair::generate();
    let origin = kp.origin_hash();
    let host = DaemonHost::new(
        Box::new(CounterDaemon::with_count(initial_count)),
        kp.clone(),
        DaemonHostConfig::default(),
    );
    registry.register(host).unwrap();
    (kp, origin)
}

// ── 1. Orchestrator full phase progression ───────────────────────────────────

#[test]
fn test_orchestrator_full_phase_chain() {
    let reg = Arc::new(DaemonRegistry::new());
    let (_kp, origin) = register_counter_daemon(&reg, 42);
    let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);

    // Phase 0→1: Start migration (local source)
    let msg = orch.start_migration(origin, 0x1111, 0x2222).unwrap();
    assert_eq!(orch.status(origin), Some(MigrationPhase::Transfer));
    let _snapshot_bytes = match msg {
        MigrationMessage::SnapshotReady { snapshot_bytes, .. } => snapshot_bytes,
        _ => panic!("expected SnapshotReady"),
    };

    // Phase 1→2: Snapshot ready → forward (simulating orchestrator receiving it back)
    // Since we're the source and already advanced, on_snapshot_ready is for remote case.
    // For local source, we're already in Transfer. Simulate restore complete from target.
    let buffered = orch.on_restore_complete(origin, 42).unwrap();
    assert_eq!(orch.status(origin), Some(MigrationPhase::Replay));
    assert!(buffered.is_none()); // no events buffered yet

    // Phase 3→4: Replay complete → cutover
    let cutover_msg = orch.on_replay_complete(origin, 42).unwrap();
    assert_eq!(orch.status(origin), Some(MigrationPhase::Cutover));
    match cutover_msg {
        MigrationMessage::CutoverNotify { target_node, .. } => {
            assert_eq!(target_node, 0x2222);
        }
        _ => panic!("expected CutoverNotify"),
    }

    // Phase 4→5: Cutover acknowledged
    orch.on_cutover_acknowledged(origin).unwrap();
    assert_eq!(orch.status(origin), Some(MigrationPhase::Complete));

    // Phase 5: Cleanup complete
    orch.on_cleanup_complete(origin).unwrap();
    assert!(!orch.is_migrating(origin));
    assert_eq!(orch.active_count(), 0);
}

// ── 2. Orchestrator phase chain with buffered events ─────────────────────────

#[test]
fn test_orchestrator_phase_chain_with_buffered_events() {
    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 10);
    let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);

    // Start migration
    orch.start_migration(origin, 0x1111, 0x2222).unwrap();

    // Buffer events while snapshot is in flight
    for seq in 1..=5 {
        assert!(orch.buffer_event(origin, make_event(0xBBBB, seq)));
    }

    // Restore complete → should drain buffered events
    let buffered = orch.on_restore_complete(origin, 10).unwrap();
    assert!(buffered.is_some());
    match buffered.unwrap() {
        MigrationMessage::BufferedEvents { events, .. } => {
            assert_eq!(events.len(), 5);
            assert_eq!(events[0].link.sequence, 1);
            assert_eq!(events[4].link.sequence, 5);
        }
        _ => panic!("expected BufferedEvents"),
    }

    // Continue through remaining phases
    orch.on_replay_complete(origin, 15).unwrap();
    orch.on_cutover_acknowledged(origin).unwrap();
    orch.on_cleanup_complete(origin).unwrap();
    assert!(!orch.is_migrating(origin));
}

// ── 3. End-to-end: source → orchestrator → target ───────────────────────────

#[test]
fn test_end_to_end_migration_local_source() {
    // Setup: source node (0x1111) with a daemon, target registry (0x2222)
    let source_reg = Arc::new(DaemonRegistry::new());
    let target_reg = Arc::new(DaemonRegistry::new());

    let (kp, origin) = register_counter_daemon(&source_reg, 100);

    // Process some events on source to advance state
    for seq in 1..=5 {
        source_reg.deliver(origin, &make_event(0xFFFF, seq)).unwrap();
    }
    // Daemon count is now 105 (100 initial + 5 events)

    let source_handler = MigrationSourceHandler::new(source_reg.clone());
    let target_handler = MigrationTargetHandler::new(target_reg.clone());
    let orch = MigrationOrchestrator::new(source_reg.clone(), 0x1111);

    // Phase 0: Orchestrator starts migration → takes snapshot locally
    let msg = orch.start_migration(origin, 0x1111, 0x2222).unwrap();
    let (snapshot_bytes, seq_through) = match msg {
        MigrationMessage::SnapshotReady {
            snapshot_bytes,
            seq_through,
            ..
        } => (snapshot_bytes, seq_through),
        _ => panic!("expected SnapshotReady"),
    };

    // Phase 2: Target restores from snapshot
    let snapshot = StateSnapshot::from_bytes(&snapshot_bytes).unwrap();
    target_handler
        .restore_snapshot(
            origin,
            &snapshot,
            0x1111,
            kp.clone(),
            || Box::new(CounterDaemon::new()),
            DaemonHostConfig::default(),
        )
        .unwrap();
    assert!(target_reg.contains(origin));

    // Simulate events arriving during transfer
    source_handler.start_snapshot(origin, 0x2222).unwrap();
    source_handler
        .buffer_event(origin, make_event(0xFFFF, 6))
        .unwrap();
    source_handler
        .buffer_event(origin, make_event(0xFFFF, 7))
        .unwrap();

    // Phase 2→3: Notify orchestrator restore is complete
    let _buffered_msg = orch.on_restore_complete(origin, seq_through).unwrap();
    // Orchestrator may have its own buffered events (none in this case since
    // we buffered on the source handler directly)

    // Phase 3: Replay buffered events from source on target
    let buffered_events = source_handler.take_buffered_events(origin).unwrap();
    assert_eq!(buffered_events.len(), 2);
    let replayed_through = target_handler
        .replay_events(origin, buffered_events)
        .unwrap();

    // Phase 3→4: Replay complete
    let cutover_msg = orch.on_replay_complete(origin, replayed_through).unwrap();
    match &cutover_msg {
        MigrationMessage::CutoverNotify { target_node, .. } => {
            assert_eq!(*target_node, 0x2222);
        }
        _ => panic!("expected CutoverNotify"),
    }

    // Phase 4: Cutover — source stops accepting writes
    let final_events = source_handler.on_cutover(origin).unwrap();
    assert!(final_events.is_empty()); // already drained

    // Phase 4: Activate target
    target_handler.activate(origin).unwrap();

    // Phase 4→5: Cutover acknowledged
    orch.on_cutover_acknowledged(origin).unwrap();

    // Phase 5: Source cleanup
    source_handler.cleanup(origin).unwrap();
    assert!(!source_reg.contains(origin)); // daemon removed from source

    // Target completes
    target_handler.complete(origin).unwrap();
    orch.on_cleanup_complete(origin).unwrap();

    // Verify: daemon lives on target, not on source
    assert!(target_reg.contains(origin));
    assert!(!source_reg.contains(origin));
    assert!(!orch.is_migrating(origin));
}

// ── 4. start_migration_auto ──────────────────────────────────────────────────

#[test]
fn test_start_migration_auto() {
    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 50);

    let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);

    // Create an index with a migration-capable target
    let index = Arc::new(CapabilityIndex::new());
    let target_caps = CapabilitySet::new().add_tag("subprotocol:0x0500");
    index.index(CapabilityAnnouncement::new(0x2222, 1, target_caps));

    let local_caps = CapabilitySet::new();
    let scheduler = Scheduler::new(index, 0x1111, local_caps);

    let (target_node, msg) = orch
        .start_migration_auto(origin, 0x1111, &scheduler, &CapabilityFilter::default())
        .unwrap();

    assert_eq!(target_node, 0x2222);
    assert_eq!(orch.target_node(origin), Some(0x2222));
    match msg {
        MigrationMessage::SnapshotReady { daemon_origin, .. } => {
            assert_eq!(daemon_origin, origin);
        }
        _ => panic!("expected SnapshotReady"),
    }
}

#[test]
fn test_start_migration_auto_no_targets() {
    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 50);

    let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);

    // Empty index — no migration-capable nodes
    let index = Arc::new(CapabilityIndex::new());
    let scheduler = Scheduler::new(index, 0x1111, CapabilitySet::new());

    let err = orch
        .start_migration_auto(origin, 0x1111, &scheduler, &CapabilityFilter::default())
        .unwrap_err();
    match err {
        net::adapter::net::MigrationError::TargetUnavailable(_) => {}
        _ => panic!("expected TargetUnavailable, got {:?}", err),
    }
}

// ── 5. Subprotocol handler full message chain ────────────────────────────────

#[test]
fn test_subprotocol_handler_snapshot_ready_dispatch() {
    use net::adapter::net::compute::orchestrator::wire;
    use net::adapter::net::subprotocol::MigrationSubprotocolHandler;

    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 25);

    let orch = Arc::new(MigrationOrchestrator::new(reg.clone(), 0x1111));
    let source = Arc::new(MigrationSourceHandler::new(reg.clone()));
    let target = Arc::new(MigrationTargetHandler::new(reg.clone()));

    let handler = MigrationSubprotocolHandler::new(orch.clone(), source, target, 0x1111);

    // Send TakeSnapshot → should get SnapshotReady back
    let take_msg = MigrationMessage::TakeSnapshot {
        daemon_origin: origin,
        target_node: 0x2222,
    };
    let outbound = handler
        .handle_message(&wire::encode(&take_msg), 0x3333)
        .unwrap();
    assert!(!outbound.is_empty());

    // Decode the reply — should be SnapshotReady
    let reply = wire::decode(&outbound[0].payload).unwrap();
    match reply {
        MigrationMessage::SnapshotReady {
            daemon_origin,
            chunk_index,
            total_chunks,
            ..
        } => {
            assert_eq!(daemon_origin, origin);
            assert_eq!(chunk_index, 0);
            assert_eq!(total_chunks, 1); // small daemon = single chunk
        }
        _ => panic!("expected SnapshotReady"),
    }
}

#[test]
fn test_subprotocol_handler_buffered_events_dispatch() {
    use net::adapter::net::compute::orchestrator::wire;
    use net::adapter::net::subprotocol::MigrationSubprotocolHandler;

    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 10);

    let orch = Arc::new(MigrationOrchestrator::new(reg.clone(), 0x3333));
    let source = Arc::new(MigrationSourceHandler::new(reg.clone()));
    let target = Arc::new(MigrationTargetHandler::new(reg.clone()));

    let handler = MigrationSubprotocolHandler::new(orch.clone(), source, target, 0x3333);

    // Start migration on the orchestrator (remote source at 0x1111)
    orch.start_migration(origin, 0x1111, 0x2222).unwrap();

    // Buffer some events on the orchestrator
    orch.buffer_event(origin, make_event(0xCCCC, 1));
    orch.buffer_event(origin, make_event(0xCCCC, 2));

    // Send RestoreComplete → should get BufferedEvents back
    let restore_msg = MigrationMessage::RestoreComplete {
        daemon_origin: origin,
        restored_seq: 10,
    };
    let outbound = handler
        .handle_message(&wire::encode(&restore_msg), 0x2222)
        .unwrap();

    // Should have BufferedEvents response
    assert!(!outbound.is_empty());
    let reply = wire::decode(&outbound[0].payload).unwrap();
    match reply {
        MigrationMessage::BufferedEvents { events, .. } => {
            assert_eq!(events.len(), 2);
        }
        _ => panic!("expected BufferedEvents, got {:?}", reply),
    }
}

#[test]
fn test_subprotocol_handler_cutover_notify_dispatch() {
    use net::adapter::net::compute::orchestrator::wire;
    use net::adapter::net::subprotocol::MigrationSubprotocolHandler;

    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 5);

    let orch = Arc::new(MigrationOrchestrator::new(reg.clone(), 0x1111));
    let source = Arc::new(MigrationSourceHandler::new(reg.clone()));
    let target = Arc::new(MigrationTargetHandler::new(reg.clone()));

    let handler = MigrationSubprotocolHandler::new(orch.clone(), source.clone(), target, 0x1111);

    // Setup: source starts snapshot
    source.start_snapshot(origin, 0x2222).unwrap();

    // Buffer an event on source
    source
        .buffer_event(origin, make_event(0xFFFF, 1))
        .unwrap();

    // Orchestrator starts migration
    orch.start_migration(origin, 0x1111, 0x2222).unwrap();
    // Advance to replay→cutover
    orch.on_restore_complete(origin, 5).unwrap();
    orch.on_replay_complete(origin, 5).unwrap();

    // Now send CutoverNotify to the source via handler
    let cutover_msg = MigrationMessage::CutoverNotify {
        daemon_origin: origin,
        target_node: 0x2222,
    };
    let outbound = handler
        .handle_message(&wire::encode(&cutover_msg), 0x3333)
        .unwrap();

    // Should have: BufferedEvents (final events) + CleanupComplete
    assert!(!outbound.is_empty());

    // Check that CleanupComplete was sent
    let has_cleanup = outbound.iter().any(|o| {
        matches!(
            wire::decode(&o.payload),
            Ok(MigrationMessage::CleanupComplete { .. })
        )
    });
    assert!(has_cleanup, "expected CleanupComplete in outbound");
}

#[test]
fn test_subprotocol_handler_cleanup_complete_dispatch() {
    use net::adapter::net::compute::orchestrator::wire;
    use net::adapter::net::subprotocol::MigrationSubprotocolHandler;

    let reg = Arc::new(DaemonRegistry::new());
    let (_, origin) = register_counter_daemon(&reg, 1);

    let orch = Arc::new(MigrationOrchestrator::new(reg.clone(), 0x1111));
    let source = Arc::new(MigrationSourceHandler::new(reg.clone()));
    let target = Arc::new(MigrationTargetHandler::new(reg.clone()));

    let handler = MigrationSubprotocolHandler::new(orch.clone(), source, target, 0x1111);

    // Setup: start and advance migration to Complete
    orch.start_migration(origin, 0x1111, 0x2222).unwrap();
    orch.on_restore_complete(origin, 1).unwrap();
    orch.on_replay_complete(origin, 1).unwrap();
    orch.on_cutover_acknowledged(origin).unwrap();
    assert!(orch.is_migrating(origin));

    // Send CleanupComplete
    let cleanup_msg = MigrationMessage::CleanupComplete {
        daemon_origin: origin,
    };
    let outbound = handler
        .handle_message(&wire::encode(&cleanup_msg), 0x1111)
        .unwrap();
    assert!(outbound.is_empty()); // no response needed
    assert!(!orch.is_migrating(origin)); // migration removed
}

// ── 6. Reassembler with out-of-order chunks ──────────────────────────────────

#[test]
fn test_reassembler_out_of_order_chunks() {
    let data = vec![0xABu8; MAX_SNAPSHOT_CHUNK_SIZE * 3 + 500];
    let total_len = data.len();
    let chunks = chunk_snapshot(0xAAAA, data, 99);
    assert_eq!(chunks.len(), 4);

    let mut reassembler = SnapshotReassembler::new();

    // Feed in reverse order: 3, 1, 0, 2
    let feed_order = [3, 1, 0, 2];
    for &i in &feed_order[..3] {
        let chunk = &chunks[i as usize];
        if let MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes,
            seq_through,
            chunk_index,
            total_chunks,
        } = chunk
        {
            let result = reassembler.feed(
                *daemon_origin,
                snapshot_bytes.clone(),
                *seq_through,
                *chunk_index,
                *total_chunks,
            );
            assert!(result.is_none(), "chunk {} should not complete", i);
        }
    }

    // Feed the last one (index 2) — should complete
    let last = &chunks[feed_order[3] as usize];
    if let MigrationMessage::SnapshotReady {
        daemon_origin,
        snapshot_bytes,
        seq_through,
        chunk_index,
        total_chunks,
    } = last
    {
        let result = reassembler
            .feed(
                *daemon_origin,
                snapshot_bytes.clone(),
                *seq_through,
                *chunk_index,
                *total_chunks,
            )
            .expect("last chunk should complete reassembly");
        assert_eq!(result.len(), total_len);
        assert!(result.iter().all(|&b| b == 0xAB));
    }

    assert_eq!(reassembler.pending_count(), 0);
}

#[test]
fn test_reassembler_duplicate_chunks_handled() {
    let data = vec![0xCDu8; MAX_SNAPSHOT_CHUNK_SIZE * 2 + 100];
    let total_len = data.len();
    let chunks = chunk_snapshot(0xBBBB, data, 50);
    assert_eq!(chunks.len(), 3);

    let mut reassembler = SnapshotReassembler::new();

    // Feed chunk 0 twice — second should overwrite, not cause issues
    if let MigrationMessage::SnapshotReady {
        daemon_origin,
        snapshot_bytes,
        seq_through,
        chunk_index,
        total_chunks,
    } = &chunks[0]
    {
        reassembler.feed(*daemon_origin, snapshot_bytes.clone(), *seq_through, *chunk_index, *total_chunks);
        reassembler.feed(*daemon_origin, snapshot_bytes.clone(), *seq_through, *chunk_index, *total_chunks);
    }

    // Feed remaining chunks
    for chunk in &chunks[1..] {
        if let MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes,
            seq_through,
            chunk_index,
            total_chunks,
        } = chunk
        {
            let result = reassembler.feed(
                *daemon_origin,
                snapshot_bytes.clone(),
                *seq_through,
                *chunk_index,
                *total_chunks,
            );
            if *chunk_index == *total_chunks - 1 {
                let full = result.expect("last chunk should complete");
                assert_eq!(full.len(), total_len);
            }
        }
    }
}

// ── 7. Event buffer → replay integration ─────────────────────────────────────

#[test]
fn test_event_buffer_flows_to_target_replay() {
    let source_reg = Arc::new(DaemonRegistry::new());
    let target_reg = Arc::new(DaemonRegistry::new());

    let (kp, origin) = register_counter_daemon(&source_reg, 0);

    // Process 10 events on source
    for seq in 1..=10 {
        source_reg
            .deliver(origin, &make_event(0xFFFF, seq))
            .unwrap();
    }

    let source_handler = MigrationSourceHandler::new(source_reg.clone());
    let target_handler = MigrationTargetHandler::new(target_reg.clone());

    // Source takes snapshot (daemon count = 10)
    let snapshot = source_handler.start_snapshot(origin, 0x2222).unwrap();

    // Events arrive during migration
    for seq in 11..=15 {
        source_handler
            .buffer_event(origin, make_event(0xFFFF, seq))
            .unwrap();
    }

    // Target restores from snapshot
    target_handler
        .restore_snapshot(
            origin,
            &snapshot,
            0x1111,
            kp.clone(),
            || Box::new(CounterDaemon::new()),
            DaemonHostConfig::default(),
        )
        .unwrap();

    // Drain buffered events from source
    let buffered = source_handler.take_buffered_events(origin).unwrap();
    assert_eq!(buffered.len(), 5);

    // Replay on target
    let replayed = target_handler.replay_events(origin, buffered).unwrap();
    assert_eq!(replayed, 15); // replayed through seq 15

    // Verify target daemon processed the events
    let target_stats = target_reg.stats(origin).unwrap();
    assert_eq!(target_stats.events_processed, 5); // 5 replayed events

    // Activate and complete
    target_handler.activate(origin).unwrap();
    target_handler.complete(origin).unwrap();
    assert!(target_reg.contains(origin));
}

// ── 8. Concurrent migrations ─────────────────────────────────────────────────

#[test]
fn test_concurrent_migrations_no_interference() {
    let reg = Arc::new(DaemonRegistry::new());

    let (_, origin_a) = register_counter_daemon(&reg, 100);
    let (_, origin_b) = register_counter_daemon(&reg, 200);

    let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);

    // Start both migrations
    orch.start_migration(origin_a, 0x1111, 0x2222).unwrap();
    orch.start_migration(origin_b, 0x1111, 0x3333).unwrap();
    assert_eq!(orch.active_count(), 2);

    // Advance A through all phases
    orch.on_restore_complete(origin_a, 100).unwrap();
    orch.on_replay_complete(origin_a, 100).unwrap();
    orch.on_cutover_acknowledged(origin_a).unwrap();
    orch.on_cleanup_complete(origin_a).unwrap();

    // B should still be active
    assert!(!orch.is_migrating(origin_a));
    assert!(orch.is_migrating(origin_b));
    assert_eq!(orch.status(origin_b), Some(MigrationPhase::Transfer));

    // Advance B
    orch.on_restore_complete(origin_b, 200).unwrap();
    orch.on_replay_complete(origin_b, 200).unwrap();
    orch.on_cutover_acknowledged(origin_b).unwrap();
    orch.on_cleanup_complete(origin_b).unwrap();

    assert_eq!(orch.active_count(), 0);
}

// ── 9. Capability enrichment end-to-end ──────────────────────────────────────

#[test]
fn test_enriched_capabilities_discoverable_by_scheduler() {
    // Node A: registers defaults and enriches its capabilities
    let subproto_reg = SubprotocolRegistry::with_defaults();
    let node_a_caps = subproto_reg.enrich_capabilities(CapabilitySet::new());
    assert!(node_a_caps.has_tag("subprotocol:0x0500"));

    // Index node A's capabilities
    let index = Arc::new(CapabilityIndex::new());
    index.index(CapabilityAnnouncement::new(0xAAAA, 1, node_a_caps));

    // Node B: no migration support
    let node_b_caps = CapabilitySet::new();
    index.index(CapabilityAnnouncement::new(0xBBBB, 1, node_b_caps));

    // Scheduler on node C should find A but not B
    let scheduler = Scheduler::new(index, 0xCCCC, CapabilitySet::new());
    let targets = scheduler.find_migration_targets(&CapabilityFilter::default(), 0xCCCC);
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0], 0xAAAA);
}

// ── 10. Wire roundtrip for all message types ─────────────────────────────────

#[test]
fn test_wire_roundtrip_all_message_types() {
    use net::adapter::net::compute::orchestrator::wire;

    let messages: Vec<MigrationMessage> = vec![
        MigrationMessage::TakeSnapshot {
            daemon_origin: 0x1111,
            target_node: 0x2222,
        },
        MigrationMessage::SnapshotReady {
            daemon_origin: 0x3333,
            snapshot_bytes: vec![1, 2, 3, 4, 5],
            seq_through: 42,
            chunk_index: 0,
            total_chunks: 1,
        },
        MigrationMessage::SnapshotReady {
            daemon_origin: 0x3333,
            snapshot_bytes: vec![6, 7, 8],
            seq_through: 42,
            chunk_index: 2,
            total_chunks: 5,
        },
        MigrationMessage::RestoreComplete {
            daemon_origin: 0x4444,
            restored_seq: 100,
        },
        MigrationMessage::ReplayComplete {
            daemon_origin: 0x5555,
            replayed_seq: 200,
        },
        MigrationMessage::CutoverNotify {
            daemon_origin: 0x6666,
            target_node: 0x7777,
        },
        MigrationMessage::CleanupComplete {
            daemon_origin: 0x8888,
        },
        MigrationMessage::MigrationFailed {
            daemon_origin: 0x9999,
            reason: "test failure".into(),
        },
        MigrationMessage::BufferedEvents {
            daemon_origin: 0xAAAA,
            events: vec![make_event(0xBBBB, 1), make_event(0xBBBB, 2)],
        },
    ];

    for msg in &messages {
        let encoded = wire::encode(msg);
        let decoded = wire::decode(&encoded).unwrap();

        // Verify message type matches by checking discriminant
        assert_eq!(
            std::mem::discriminant(msg),
            std::mem::discriminant(&decoded),
            "roundtrip failed for {:?}",
            msg,
        );
    }
}

// ── 11. Migration abort at each phase ────────────────────────────────────────

#[test]
fn test_abort_at_each_phase() {
    let reg = Arc::new(DaemonRegistry::new());

    // Abort during Snapshot phase (remote source)
    {
        let (_, origin) = register_counter_daemon(&reg, 1);
        let orch = MigrationOrchestrator::new(reg.clone(), 0x3333);
        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        assert_eq!(orch.status(origin), Some(MigrationPhase::Snapshot));
        orch.abort_migration(origin, "abort at snapshot".into())
            .unwrap();
        assert!(!orch.is_migrating(origin));
    }

    // Abort during Transfer phase (local source)
    {
        let (_, origin) = register_counter_daemon(&reg, 2);
        let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);
        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        assert_eq!(orch.status(origin), Some(MigrationPhase::Transfer));
        orch.abort_migration(origin, "abort at transfer".into())
            .unwrap();
        assert!(!orch.is_migrating(origin));
    }

    // Abort during Replay phase
    {
        let (_, origin) = register_counter_daemon(&reg, 3);
        let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);
        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        orch.on_restore_complete(origin, 3).unwrap();
        assert_eq!(orch.status(origin), Some(MigrationPhase::Replay));
        orch.abort_migration(origin, "abort at replay".into())
            .unwrap();
        assert!(!orch.is_migrating(origin));
    }

    // Abort during Cutover phase
    {
        let (_, origin) = register_counter_daemon(&reg, 4);
        let orch = MigrationOrchestrator::new(reg.clone(), 0x1111);
        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        orch.on_restore_complete(origin, 4).unwrap();
        orch.on_replay_complete(origin, 4).unwrap();
        assert_eq!(orch.status(origin), Some(MigrationPhase::Cutover));
        orch.abort_migration(origin, "abort at cutover".into())
            .unwrap();
        assert!(!orch.is_migrating(origin));
    }
}
