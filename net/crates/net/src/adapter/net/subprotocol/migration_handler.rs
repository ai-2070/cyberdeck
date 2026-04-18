//! Migration subprotocol message handler.
//!
//! Dispatches inbound migration messages (subprotocol 0x0500) to the
//! appropriate handler: orchestrator, source, or target.

use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::adapter::net::compute::orchestrator::wire;
use crate::adapter::net::compute::{
    MigrationError, MigrationMessage, MigrationOrchestrator, MigrationSourceHandler,
    MigrationTargetHandler, SnapshotReassembler,
};
use crate::adapter::net::state::snapshot::StateSnapshot;

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
    /// Per-target reassemblers for incoming snapshot chunks. One entry
    /// per in-flight inbound migration. Lazily created on first chunk,
    /// torn down after successful restore or failure.
    ///
    /// Wrapped in `Mutex` because `SnapshotReassembler::feed` requires
    /// `&mut self`.
    reassemblers: DashMap<u32, Mutex<SnapshotReassembler>>,
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
            reassemblers: DashMap::new(),
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

                // Chunk the snapshot for transport
                let chunks = crate::adapter::net::compute::orchestrator::chunk_snapshot(
                    daemon_origin,
                    snapshot.to_bytes(),
                    snapshot.through_seq,
                )?;
                for chunk in chunks {
                    outbound.push(OutboundMigrationMessage {
                        dest_node: from_node, // reply to orchestrator
                        payload: wire::encode(&chunk)?,
                    });
                }
            }

            MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
                chunk_index,
                total_chunks,
            } => {
                // If the orchestrator is local, let it record this chunk and
                // forward to target. `target_node` identifies where the
                // snapshot should be restored; if that's us, we reassemble
                // and restore instead of forwarding.
                let orch_target = self.orchestrator.target_node(daemon_origin);

                match orch_target {
                    Some(target) if target == self.local_node_id => {
                        // We are the target — advance orchestrator state
                        // (safe: on_snapshot_ready is idempotent on the
                        // target side because it just re-derives the
                        // forward message we ignore), then reassemble.
                        //
                        // Errors are informational: the restore path below
                        // is the authoritative check for whether the
                        // snapshot is usable. We log at `debug` rather
                        // than ignore so non-idempotent failures (e.g.,
                        // unexpected phase transitions on a stale record)
                        // are observable during triage.
                        if let Err(e) = self.orchestrator.on_snapshot_ready(
                            daemon_origin,
                            snapshot_bytes.clone(),
                            seq_through,
                            chunk_index,
                            total_chunks,
                        ) {
                            tracing::debug!(
                                ?e,
                                origin = format!("{:#x}", daemon_origin),
                                "on_snapshot_ready (local target): ignored"
                            );
                        }

                        if let Some(out) = self.restore_on_target(
                            daemon_origin,
                            snapshot_bytes,
                            seq_through,
                            chunk_index,
                            total_chunks,
                            from_node,
                        )? {
                            outbound.extend(out);
                        }
                    }
                    Some(target) => {
                        // Middle of the chain (or orchestrator node forwarding
                        // to a remote target). Let the orchestrator update its
                        // own phase state and emit the forward.
                        let forward = self.orchestrator.on_snapshot_ready(
                            daemon_origin,
                            snapshot_bytes,
                            seq_through,
                            chunk_index,
                            total_chunks,
                        )?;
                        if let MigrationMessage::SnapshotReady { .. } = &forward {
                            outbound.push(OutboundMigrationMessage {
                                dest_node: target,
                                payload: wire::encode(&forward)?,
                            });
                        }
                    }
                    None => {
                        // No local migration record — this node may be a
                        // target that has no orchestrator-side state (the
                        // orchestrator lives on a different node). Try to
                        // restore anyway; the factory registry is the
                        // authority on whether this node should accept.
                        if let Some(out) = self.restore_on_target(
                            daemon_origin,
                            snapshot_bytes,
                            seq_through,
                            chunk_index,
                            total_chunks,
                            from_node,
                        )? {
                            outbound.extend(out);
                        }
                    }
                }
            }

            MigrationMessage::RestoreComplete {
                daemon_origin,
                restored_seq,
            } => {
                // Target has restored — orchestrator may send buffered events.
                // If there are no buffered events, send an empty BufferedEvents
                // anyway: the target's reply (ReplayComplete) is what drives
                // the chain forward into Cutover. Dropping the message here
                // would stall any migration whose source never buffered.
                let buffered_msg = self
                    .orchestrator
                    .on_restore_complete(daemon_origin, restored_seq)?
                    .unwrap_or(MigrationMessage::BufferedEvents {
                        daemon_origin,
                        events: Vec::new(),
                    });
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node, // send back to target
                    payload: wire::encode(&buffered_msg)?,
                });
            }

            MigrationMessage::ReplayComplete {
                daemon_origin,
                replayed_seq,
            } => {
                // Target finished replay — orchestrator initiates cutover
                let cutover_msg = self
                    .orchestrator
                    .on_replay_complete(daemon_origin, replayed_seq)?;

                // Send CutoverNotify to source (from_node is the target that reported)
                if let MigrationMessage::CutoverNotify { .. } = &cutover_msg {
                    let source_node = self
                        .orchestrator
                        .source_node(daemon_origin)
                        .unwrap_or(from_node);

                    outbound.push(OutboundMigrationMessage {
                        dest_node: source_node,
                        payload: wire::encode(&cutover_msg)?,
                    });
                }
            }

            MigrationMessage::CutoverNotify {
                daemon_origin,
                target_node,
            } => {
                // We are the source — stop accepting writes.
                //
                // `on_cutover` returns `DaemonNotFound` if this node didn't
                // handle a `TakeSnapshot` (the orchestrator took the snapshot
                // locally and never involved `source_handler`). Treat that as
                // "no buffered events to drain" rather than a hard error so
                // local-source migrations can still reach cleanup.
                let final_events = match self.source_handler.on_cutover(daemon_origin) {
                    Ok(events) => events,
                    Err(MigrationError::DaemonNotFound(_)) => Vec::new(),
                    Err(e) => return Err(e),
                };

                // If there are last-moment events, send them to target
                if !final_events.is_empty() {
                    let events_msg = MigrationMessage::BufferedEvents {
                        daemon_origin,
                        events: final_events,
                    };
                    outbound.push(OutboundMigrationMessage {
                        dest_node: target_node,
                        payload: wire::encode(&events_msg)?,
                    });
                }

                // Acknowledge cutover to the local orchestrator. When the
                // orchestrator lives on a different node, this local call
                // has no record to advance; the remote orchestrator learns
                // about cutover from `CleanupComplete`, which does the same
                // phase advance there.
                match self.orchestrator.on_cutover_acknowledged(daemon_origin) {
                    Ok(()) => {}
                    Err(MigrationError::DaemonNotFound(_)) => {}
                    Err(e) => return Err(e),
                }

                // Cleanup source — also tolerant of missing source_handler
                // state, and unregisters the local daemon even if no
                // `source_handler.start_snapshot` was called.
                let _ = self.source_handler.cleanup(daemon_origin);

                let cleanup_msg = MigrationMessage::CleanupComplete { daemon_origin };
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node,
                    payload: wire::encode(&cleanup_msg)?,
                });
            }

            MigrationMessage::CleanupComplete { daemon_origin } => {
                // Source reports its cleanup done. The orchestrator now
                // tells the target to activate.
                let activate = self.orchestrator.on_cleanup_complete(daemon_origin)?;
                // Route the ActivateTarget to whichever node is the target.
                let target = self
                    .orchestrator
                    .target_node(daemon_origin)
                    .unwrap_or(from_node);
                outbound.push(OutboundMigrationMessage {
                    dest_node: target,
                    payload: wire::encode(&activate)?,
                });
            }

            MigrationMessage::ActivateTarget { daemon_origin } => {
                // We are the target — drain remaining events and go live.
                let replayed_seq = self.target_handler.activate(daemon_origin)?;
                // Clean up local target-side migration tracking (daemon
                // stays registered in the daemon registry).
                let _ = self.target_handler.complete(daemon_origin);
                let ack = MigrationMessage::ActivateAck {
                    daemon_origin,
                    replayed_seq,
                };
                outbound.push(OutboundMigrationMessage {
                    dest_node: from_node,
                    payload: wire::encode(&ack)?,
                });
            }

            MigrationMessage::ActivateAck {
                daemon_origin,
                replayed_seq,
            } => {
                // Target acknowledged — migration terminus on the
                // orchestrator.
                self.orchestrator
                    .on_activate_ack(daemon_origin, replayed_seq)?;
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
                    payload: wire::encode(&reply)?,
                });
            }
        }

        Ok(outbound)
    }

    /// Feed a snapshot chunk into the target-side reassembler. When the
    /// full snapshot is assembled, resolve a factory for the daemon and
    /// call `restore_snapshot`, then emit `RestoreComplete` back to the
    /// source node (`from_node`).
    ///
    /// Returns `Ok(None)` while waiting for more chunks, `Ok(Some(outbound))`
    /// with the `RestoreComplete` (or a `MigrationFailed`) once restore has
    /// been attempted.
    fn restore_on_target(
        &self,
        daemon_origin: u32,
        snapshot_bytes: Vec<u8>,
        seq_through: u64,
        chunk_index: u32,
        total_chunks: u32,
        from_node: u64,
    ) -> Result<Option<Vec<OutboundMigrationMessage>>, MigrationError> {
        let reassembler_entry = self
            .reassemblers
            .entry(daemon_origin)
            .or_insert_with(|| Mutex::new(SnapshotReassembler::new()));

        let assembled = {
            let mut reassembler = reassembler_entry.lock();
            reassembler
                .feed(
                    daemon_origin,
                    snapshot_bytes,
                    seq_through,
                    chunk_index,
                    total_chunks,
                )
                .map_err(|e| {
                    MigrationError::StateFailed(format!("snapshot reassembly failed: {:?}", e))
                })?
        };
        drop(reassembler_entry); // release DashMap read lock

        let assembled_bytes = match assembled {
            Some(bytes) => bytes,
            None => return Ok(None), // still waiting for more chunks
        };

        // Drop the reassembler entry now that we've completed.
        self.reassemblers.remove(&daemon_origin);

        // Parse the snapshot. A parse failure is a hard migration failure.
        let snapshot = match StateSnapshot::from_bytes(&assembled_bytes) {
            Some(s) => s,
            None => {
                return Ok(Some(self.fail_migration(
                    daemon_origin,
                    from_node,
                    "failed to parse snapshot bytes on target",
                )?));
            }
        };

        // Build restore inputs without removing the registration. If the
        // restore fails (e.g., parse error revealed at a later layer, or
        // any other recoverable failure), the factory stays in place so a
        // retry can use it without the caller re-registering.
        let inputs = match self.target_handler.factories().construct(daemon_origin) {
            Some(i) => i,
            None => {
                return Ok(Some(self.fail_migration(
                    daemon_origin,
                    from_node,
                    &format!(
                        "no daemon factory registered for origin {:#x}",
                        daemon_origin
                    ),
                )?));
            }
        };

        // `source_node` is the daemon's pre-migration host — tracked here
        // only for the target-handler's internal bookkeeping. It is NOT
        // where `RestoreComplete` gets sent (see below).
        let source_node = self
            .orchestrator
            .source_node(daemon_origin)
            .unwrap_or(from_node);

        let daemon = inputs.daemon;
        if let Err(e) = self.target_handler.restore_snapshot(
            daemon_origin,
            &snapshot,
            source_node,
            inputs.keypair,
            move || daemon,
            inputs.config,
        ) {
            // Factory is still registered — next `SnapshotReady` for this
            // origin (e.g., from an orchestrator retry) can try again.
            return Ok(Some(self.fail_migration(
                daemon_origin,
                from_node,
                &format!("restore_snapshot failed: {:?}", e),
            )?));
        }

        // Restore succeeded — the registration is now single-shot. Remove
        // it so a second migration for the same origin, without explicit
        // re-registration, is refused.
        self.target_handler.factories().remove(daemon_origin);

        // Route `RestoreComplete` to the orchestrator, not the source.
        // Only the orchestrator holds the migration record; sending to
        // the source would fail with `DaemonNotFound` whenever the source
        // and orchestrator are different nodes (including the collocated
        // orchestrator+target case). `from_node` is the node that
        // forwarded us this `SnapshotReady` — in the common topology
        // (orchestrator → target) that IS the orchestrator.
        let reply = MigrationMessage::RestoreComplete {
            daemon_origin,
            restored_seq: seq_through,
        };
        Ok(Some(vec![OutboundMigrationMessage {
            dest_node: from_node,
            payload: wire::encode(&reply)?,
        }]))
    }

    /// Build a `MigrationFailed` outbound message and clean up local state.
    /// Used when the target can't accept a migration (no factory, parse
    /// failure, etc).
    fn fail_migration(
        &self,
        daemon_origin: u32,
        from_node: u64,
        reason: &str,
    ) -> Result<Vec<OutboundMigrationMessage>, MigrationError> {
        tracing::warn!(
            daemon_origin = format!("{:#x}", daemon_origin),
            reason = reason,
            "migration failed on target"
        );
        self.reassemblers.remove(&daemon_origin);
        let _ = self.target_handler.abort(daemon_origin);
        let msg = MigrationMessage::MigrationFailed {
            daemon_origin,
            reason: reason.to_string(),
        };
        Ok(vec![OutboundMigrationMessage {
            dest_node: from_node,
            payload: wire::encode(&msg)?,
        }])
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
        let encoded = wire::encode(&msg).unwrap();

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
        let encoded = wire::encode(&msg).unwrap();

        // Should not error — just cleans up
        let outbound = handler.handle_message(&encoded, 0x3333).unwrap();
        assert!(outbound.is_empty());
    }
}
