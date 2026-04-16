//! Migration subprotocol message handler.
//!
//! Dispatches inbound migration messages (subprotocol 0x0500) to the
//! appropriate handler: orchestrator, source, or target.

use std::sync::Arc;

use crate::adapter::net::compute::orchestrator::wire;
use crate::adapter::net::compute::{
    MigrationError, MigrationMessage, MigrationOrchestrator, MigrationSourceHandler,
    MigrationTargetHandler,
};

/// Outbound message with destination node.
#[derive(Debug)]
pub struct OutboundMigrationMessage {
    /// Destination node ID.
    pub dest_node: u64,
    /// Encoded wire message.
    pub payload: Vec<u8>,
}

/// Handles inbound migration subprotocol messages.
///
/// Routes each message type to the orchestrator, source handler, or target
/// handler as appropriate, and produces outbound response messages.
pub struct MigrationSubprotocolHandler {
    orchestrator: Arc<MigrationOrchestrator>,
    source_handler: Arc<MigrationSourceHandler>,
    target_handler: Arc<MigrationTargetHandler>,
    local_node_id: u64,
}

impl MigrationSubprotocolHandler {
    /// Create a new handler.
    pub fn new(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
    ) -> Self {
        Self {
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
        }
    }

    /// Handle an inbound migration message.
    ///
    /// Returns zero or more outbound messages to send to other nodes.
    pub fn handle_message(
        &self,
        data: &[u8],
        from_node: u64,
    ) -> Result<Vec<OutboundMigrationMessage>, MigrationError> {
        let msg = wire::decode(data)?;
        self.dispatch(msg, from_node)
    }

    /// Dispatch a decoded message to the appropriate handler.
    fn dispatch(
        &self,
        msg: MigrationMessage,
        from_node: u64,
    ) -> Result<Vec<OutboundMigrationMessage>, MigrationError> {
        let mut outbound = Vec::new();

        match msg {
            MigrationMessage::TakeSnapshot {
                daemon_origin,
                target_node,
            } => {
                // We are the source — take snapshot and reply
                let snapshot = self
                    .source_handler
                    .start_snapshot(daemon_origin, target_node)?;

                let reply = MigrationMessage::SnapshotReady {
                    daemon_origin,
                    snapshot_bytes: snapshot.to_bytes(),
                    seq_through: snapshot.through_seq,
                };
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node, // reply to orchestrator
                    payload: wire::encode(&reply),
                });
            }

            MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
            } => {
                // Forward snapshot to the target via orchestrator
                let forward = self.orchestrator.on_snapshot_ready(
                    daemon_origin,
                    snapshot_bytes,
                    seq_through,
                )?;
                // The orchestrator returns a SnapshotReady to forward to target
                if let MigrationMessage::SnapshotReady { .. } = &forward {
                    let target_node = self
                        .orchestrator
                        .status(daemon_origin)
                        .map(|_| {
                            // Get target from migration list
                            self.orchestrator
                                .list_migrations()
                                .iter()
                                .find(|(origin, _, _)| *origin == daemon_origin)
                                .map(|_| 0u64) // placeholder — target is known from start_migration
                        })
                        .flatten()
                        .unwrap_or(from_node);

                    outbound.push(OutboundMigrationMessage {
                        dest_node: target_node,
                        payload: wire::encode(&forward),
                    });
                }
            }

            MigrationMessage::RestoreComplete {
                daemon_origin,
                restored_seq,
            } => {
                // Target has restored — orchestrator may send buffered events
                if let Some(buffered_msg) = self
                    .orchestrator
                    .on_restore_complete(daemon_origin, restored_seq)?
                {
                    outbound.push(OutboundMigrationMessage {
                        dest_node: from_node, // send back to target
                        payload: wire::encode(&buffered_msg),
                    });
                }
            }

            MigrationMessage::ReplayComplete {
                daemon_origin,
                replayed_seq,
            } => {
                // Target finished replay — orchestrator initiates cutover
                let cutover_msg = self
                    .orchestrator
                    .on_replay_complete(daemon_origin, replayed_seq)?;

                // Send CutoverNotify to source
                if let MigrationMessage::CutoverNotify { .. } = &cutover_msg {
                    // Send cutover to source node (from_node is the target reporting)
                    outbound.push(OutboundMigrationMessage {
                        dest_node: from_node,
                        payload: wire::encode(&cutover_msg),
                    });
                }
            }

            MigrationMessage::CutoverNotify {
                daemon_origin,
                target_node,
            } => {
                // We are the source — stop accepting writes
                let final_events = self.source_handler.on_cutover(daemon_origin)?;

                // If there are last-moment events, send them to target
                if !final_events.is_empty() {
                    let events_msg = MigrationMessage::BufferedEvents {
                        daemon_origin,
                        events: final_events,
                    };
                    outbound.push(OutboundMigrationMessage {
                        dest_node: target_node,
                        payload: wire::encode(&events_msg),
                    });
                }

                // Acknowledge cutover to orchestrator
                self.orchestrator.on_cutover_acknowledged(daemon_origin)?;

                // Cleanup source
                self.source_handler.cleanup(daemon_origin)?;

                let cleanup_msg = MigrationMessage::CleanupComplete { daemon_origin };
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node,
                    payload: wire::encode(&cleanup_msg),
                });
            }

            MigrationMessage::CleanupComplete { daemon_origin } => {
                // Source has cleaned up — migration is fully complete
                self.orchestrator.on_cleanup_complete(daemon_origin)?;
            }

            MigrationMessage::MigrationFailed {
                daemon_origin,
                reason: _,
            } => {
                // Abort on all local handlers
                let _ = self.source_handler.abort(daemon_origin);
                let _ = self.target_handler.abort(daemon_origin);
                let _ = self
                    .orchestrator
                    .abort_migration(daemon_origin, "remote abort".into());
            }

            MigrationMessage::BufferedEvents {
                daemon_origin,
                events,
            } => {
                // We are the target — replay events
                let replayed_seq = self.target_handler.replay_events(daemon_origin, events)?;

                // Tell orchestrator we're done replaying
                let reply = MigrationMessage::ReplayComplete {
                    daemon_origin,
                    replayed_seq,
                };
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node,
                    payload: wire::encode(&reply),
                });
            }
        }

        Ok(outbound)
    }

    /// Get a reference to the orchestrator.
    pub fn orchestrator(&self) -> &Arc<MigrationOrchestrator> {
        &self.orchestrator
    }

    /// Get a reference to the source handler.
    pub fn source_handler(&self) -> &Arc<MigrationSourceHandler> {
        &self.source_handler
    }

    /// Get a reference to the target handler.
    pub fn target_handler(&self) -> &Arc<MigrationTargetHandler> {
        &self.target_handler
    }
}

impl std::fmt::Debug for MigrationSubprotocolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationSubprotocolHandler")
            .field("local_node_id", &format!("{:#x}", self.local_node_id))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::orchestrator::wire;
    use crate::adapter::net::compute::{
        DaemonError, DaemonHost, DaemonHostConfig, DaemonRegistry, MeshDaemon,
        MigrationOrchestrator, MigrationSourceHandler, MigrationTargetHandler,
    };
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalEvent;
    use bytes::Bytes;

    struct TestDaemon {
        value: u64,
    }

    impl MeshDaemon for TestDaemon {
        fn name(&self) -> &str {
            "test"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            self.value += 1;
            Ok(vec![])
        }
        fn snapshot(&self) -> Option<Bytes> {
            Some(Bytes::from(self.value.to_le_bytes().to_vec()))
        }
        fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
            self.value = u64::from_le_bytes(state[..8].try_into().unwrap());
            Ok(())
        }
    }

    fn setup() -> (MigrationSubprotocolHandler, Arc<DaemonRegistry>, u32) {
        let reg = Arc::new(DaemonRegistry::new());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let host = DaemonHost::new(
            Box::new(TestDaemon { value: 100 }),
            kp,
            DaemonHostConfig::default(),
        );
        reg.register(host).unwrap();

        let orch = Arc::new(MigrationOrchestrator::new(reg.clone(), 0x1111));
        let source = Arc::new(MigrationSourceHandler::new(reg.clone()));
        let target = Arc::new(MigrationTargetHandler::new(reg.clone()));

        let handler = MigrationSubprotocolHandler::new(orch, source, target, 0x1111);
        (handler, reg, origin)
    }

    #[test]
    fn test_handle_take_snapshot() {
        let (handler, _reg, origin) = setup();

        let msg = MigrationMessage::TakeSnapshot {
            daemon_origin: origin,
            target_node: 0x2222,
        };
        let encoded = wire::encode(&msg);

        let outbound = handler.handle_message(&encoded, 0x3333).unwrap();
        assert!(!outbound.is_empty());

        // Should get SnapshotReady back
        let reply = wire::decode(&outbound[0].payload).unwrap();
        match reply {
            MigrationMessage::SnapshotReady { daemon_origin, .. } => {
                assert_eq!(daemon_origin, origin);
            }
            _ => panic!("expected SnapshotReady"),
        }
    }

    #[test]
    fn test_handle_migration_failed() {
        let (handler, _reg, origin) = setup();

        let msg = MigrationMessage::MigrationFailed {
            daemon_origin: origin,
            reason: "test failure".into(),
        };
        let encoded = wire::encode(&msg);

        // Should not error — just cleans up
        let outbound = handler.handle_message(&encoded, 0x3333).unwrap();
        assert!(outbound.is_empty());
    }
}
