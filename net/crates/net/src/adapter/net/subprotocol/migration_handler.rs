//! Migration subprotocol message handler.
//!
//! Dispatches inbound migration messages (subprotocol 0x0500) to the
//! appropriate handler: orchestrator, source, or target.

use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::adapter::net::compute::migration_target::RestoreContext;
use crate::adapter::net::compute::orchestrator::wire;
use crate::adapter::net::compute::{
    MigrationError, MigrationMessage, MigrationOrchestrator, MigrationSourceHandler,
    MigrationTargetHandler, SnapshotReassembler,
};
use crate::adapter::net::identity::EntityKeypair;
use crate::adapter::net::state::snapshot::StateSnapshot;

/// Identity-transport context for automatic envelope seal/open in
/// the migration dispatcher.
///
/// When populated, the handler:
/// - **Source path**: after taking a snapshot, if the target's
///   X25519 static pub is known (via `peer_static_lookup`) AND the
///   local daemon's keypair is available, seal the envelope into
///   the snapshot before chunking.
/// - **Target path**: on `SnapshotReady` with an attached envelope,
///   unseal using `local_x25519_priv`, derive the daemon's keypair,
///   and pass that into `restore_snapshot` instead of whatever the
///   factory registry has pre-registered.
///
/// A `None` context means the dispatcher ignores envelopes — the
/// pre-identity-envelope fallback path where both nodes pre-register
/// matching keypairs.
#[derive(Clone)]
pub struct MigrationIdentityContext {
    /// Local Noise static X25519 private key. Used to unseal
    /// envelopes sealed to our public half.
    pub local_x25519_priv: [u8; 32],
    /// Callback: given a peer node_id, return its X25519 static
    /// public key if we have an active session with it. Used by the
    /// source path to pick the seal recipient.
    pub peer_static_lookup: Arc<dyn Fn(u64) -> Option<[u8; 32]> + Send + Sync>,
}

impl std::fmt::Debug for MigrationIdentityContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationIdentityContext")
            .field("local_x25519_priv", &"[REDACTED]")
            .field("peer_static_lookup", &"<fn>")
            .finish()
    }
}

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
/// Callback fired after the target-side dispatcher successfully
/// restores a daemon from a migration snapshot. Invoked with the
/// daemon's `origin_hash`. Used by the SDK to drive channel
/// re-bind replay (`DAEMON_CHANNEL_REBIND_PLAN.md` Stage 3): the
/// callback walks the restored daemon's subscription ledger and
/// spawns asynchronous `subscribe_channel` calls so publishers
/// start fanning out to the target before the source tears down.
///
/// The callback runs synchronously on the dispatcher thread; it
/// should kick off any async work via `tokio::spawn` rather than
/// blocking the dispatch loop.
pub type PostRestoreCallback = Arc<dyn Fn(u32) + Send + Sync>;

/// Callback fired on the source side at `CutoverNotify` handling,
/// immediately before `source_handler.cleanup` unregisters the
/// daemon. Stage 4 of `DAEMON_CHANNEL_REBIND_PLAN.md`: the SDK's
/// hook snapshots the daemon's subscription ledger here and spawns
/// async `unsubscribe_channel` calls to each publisher so rosters
/// drop the source immediately rather than aging out over the
/// session-timeout window.
///
/// Sync on the dispatcher thread; async work must `tokio::spawn`.
pub type PreCleanupCallback = Arc<dyn Fn(u32) + Send + Sync>;

/// Readiness predicate — returns `true` when the local runtime is
/// prepared to accept inbound migrations (target path). When
/// populated and the predicate returns `false`, the dispatcher
/// responds to `SnapshotReady` with
/// [`MigrationFailureReason::NotReady`](crate::adapter::net::compute::MigrationFailureReason::NotReady)
/// so the source can retry with backoff rather than surfacing a
/// terminal error while the target is still booting.
///
/// The callback is consulted synchronously on the dispatcher
/// thread — it must return quickly.
pub type ReadinessCallback = Arc<dyn Fn() -> bool + Send + Sync>;

/// Callback fired on the source side whenever the dispatcher
/// observes an inbound `MigrationFailed` message. The SDK uses this
/// to surface the structured reason code to the
/// [`MigrationHandle::wait`](crate) caller so they can distinguish
/// retriable from terminal failures.
///
/// Sync on the dispatcher thread.
pub type FailureCallback =
    Arc<dyn Fn(u32, crate::adapter::net::compute::MigrationFailureReason) + Send + Sync>;

/// Dispatcher for migration subprotocol (`0x0500`) messages.
///
/// Wraps the three handler halves — orchestrator, source, target —
/// plus the optional cross-cutting hooks that let the SDK drive
/// identity-envelope seal/open, channel-re-bind replay,
/// source-side Unsubscribe teardown, runtime-readiness gating, and
/// source-side failure observation. Constructed by the SDK's
/// `DaemonRuntime::start` via [`Self::new_fully_hooked`]; tests
/// use [`Self::new`] / [`Self::new_with_hooks`] with the subset of
/// hooks they care about.
///
/// Install onto a `MeshNode` via `MeshNode::set_migration_handler`.
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
    /// Identity-transport context. `None` = envelopes ignored
    /// (pre-identity-envelope fallback).
    identity_context: Option<MigrationIdentityContext>,
    /// Post-restore callback, fired on the target side after
    /// `restore_snapshot` succeeds. Used by the SDK to drive
    /// subscription replay. `None` = no hook (used by tests and
    /// pre-Stage-3 callers).
    post_restore_callback: Option<PostRestoreCallback>,
    /// Pre-cleanup callback, fired on the source side at
    /// `CutoverNotify` handling just before the daemon is
    /// unregistered. Drives source-side Unsubscribe teardown
    /// (Stage 4 of the channel re-bind plan). `None` = no hook.
    pre_cleanup_callback: Option<PreCleanupCallback>,
    /// Target-side readiness predicate. When `Some` and returns
    /// `false`, the dispatcher responds to inbound migration
    /// attempts with `NotReady` instead of attempting restore.
    /// Drives the runtime-readiness retry path
    /// (`DAEMON_RUNTIME_READINESS_PLAN.md` Stage 3). `None` = always
    /// treat target as ready (pre-Stage-3 behaviour).
    readiness_callback: Option<ReadinessCallback>,
    /// Source-side failure observer — fired when the dispatcher
    /// receives an inbound `MigrationFailed` message. Lets the SDK
    /// hand the structured reason to the caller via
    /// [`MigrationHandle::wait`] rather than swallowing it at the
    /// dispatcher layer. `None` = no hook; failures are still
    /// processed (orchestrator aborted) but the SDK can't
    /// distinguish retriable (NotReady) from terminal.
    failure_callback: Option<FailureCallback>,
}

impl MigrationSubprotocolHandler {
    /// Create a new handler with no identity-transport context.
    /// Envelopes on inbound snapshots are ignored; outbound
    /// snapshots don't get an envelope attached. Matches the
    /// pre-Stage-5b behaviour.
    pub fn new(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
    ) -> Self {
        Self::new_with_identity(
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            None,
        )
    }

    /// Create a new handler with an optional identity-transport
    /// context. When provided, the dispatcher auto-seals envelopes
    /// on the source path and auto-opens them on the target path.
    pub fn new_with_identity(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
        identity_context: Option<MigrationIdentityContext>,
    ) -> Self {
        Self::new_with_hooks(
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            identity_context,
            None,
        )
    }

    /// Create a new handler with both an identity-transport context
    /// and a post-restore callback. Convenience alias for
    /// [`Self::new_with_all_hooks`] with the pre-cleanup callback
    /// unset — callers that need both hooks should use the fuller
    /// constructor.
    pub fn new_with_hooks(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
        identity_context: Option<MigrationIdentityContext>,
        post_restore_callback: Option<PostRestoreCallback>,
    ) -> Self {
        Self::new_with_all_hooks(
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            identity_context,
            post_restore_callback,
            None,
        )
    }

    /// Create a handler with every hook populated: identity context,
    /// post-restore callback (target-side subscription replay), and
    /// pre-cleanup callback (source-side unsubscribe teardown).
    /// Equivalent to [`Self::new_with_full_hooks`] with no
    /// readiness predicate.
    pub fn new_with_all_hooks(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
        identity_context: Option<MigrationIdentityContext>,
        post_restore_callback: Option<PostRestoreCallback>,
        pre_cleanup_callback: Option<PreCleanupCallback>,
    ) -> Self {
        Self::new_with_full_hooks(
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            identity_context,
            post_restore_callback,
            pre_cleanup_callback,
            None,
        )
    }

    /// Create a handler with every hook + the readiness predicate.
    /// Convenience — falls through to [`Self::new_fully_hooked`]
    /// with the failure observer unset.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_full_hooks(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
        identity_context: Option<MigrationIdentityContext>,
        post_restore_callback: Option<PostRestoreCallback>,
        pre_cleanup_callback: Option<PreCleanupCallback>,
        readiness_callback: Option<ReadinessCallback>,
    ) -> Self {
        Self::new_fully_hooked(
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            identity_context,
            post_restore_callback,
            pre_cleanup_callback,
            readiness_callback,
            None,
        )
    }

    /// Fully-hooked constructor. The SDK's `DaemonRuntime::start`
    /// uses this form to supply every callback at once; the
    /// progressive-disclosure variants above are convenience
    /// entry points for tests and pre-Stage-X callers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_fully_hooked(
        orchestrator: Arc<MigrationOrchestrator>,
        source_handler: Arc<MigrationSourceHandler>,
        target_handler: Arc<MigrationTargetHandler>,
        local_node_id: u64,
        identity_context: Option<MigrationIdentityContext>,
        post_restore_callback: Option<PostRestoreCallback>,
        pre_cleanup_callback: Option<PreCleanupCallback>,
        readiness_callback: Option<ReadinessCallback>,
        failure_callback: Option<FailureCallback>,
    ) -> Self {
        Self {
            orchestrator,
            source_handler,
            target_handler,
            local_node_id,
            reassemblers: DashMap::new(),
            identity_context,
            post_restore_callback,
            pre_cleanup_callback,
            readiness_callback,
            failure_callback,
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
                // We are the source — take snapshot and reply.
                // Record `from_node` as the orchestrator: it's the node
                // that sent us TakeSnapshot, and replies (SnapshotReady,
                // CleanupComplete) must reach it. The source-side handler
                // stores this so subsequent replies don't drift if a
                // future forwarding layer rewrites `from_node`.
                let mut snapshot =
                    self.source_handler
                        .start_snapshot(daemon_origin, target_node, from_node)?;

                // Identity envelope: if we have a transport context
                // and can find the target's X25519 pubkey, seal the
                // daemon's keypair into the snapshot before chunking
                // so the target can reconstruct identity without an
                // out-of-band pre-registration.
                snapshot = self.maybe_seal_envelope(snapshot, daemon_origin, target_node);

                // Chunk the snapshot for transport
                let chunks = crate::adapter::net::compute::orchestrator::chunk_snapshot(
                    daemon_origin,
                    snapshot.to_bytes(),
                    snapshot.through_seq,
                )?;
                let orch = self
                    .source_handler
                    .orchestrator_node(daemon_origin)
                    .unwrap_or(from_node);
                for chunk in chunks {
                    outbound.push(OutboundMigrationMessage {
                        dest_node: orch,
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

                // Capture the orchestrator BEFORE `cleanup()` clears the
                // source-side migration record — once it's gone,
                // `orchestrator_node()` returns None and we'd silently
                // fall back to `from_node`, defeating the whole point of
                // recording the orchestrator at `start_snapshot` time.
                let dest = self
                    .source_handler
                    .orchestrator_node(daemon_origin)
                    .unwrap_or(from_node);

                // Fire the pre-cleanup callback BEFORE unregistering
                // the daemon — the host still holds the subscription
                // ledger, which the SDK's hook snapshots here so it
                // can send `Unsubscribe` messages to every publisher
                // after cleanup. This is Stage 4 of the channel
                // re-bind plan: without it, the publishers' rosters
                // keep pointing at the source until their session
                // timeout (~30 s), causing duplicate deliveries to
                // a now-gone daemon and unnecessary bandwidth.
                if let Some(cb) = &self.pre_cleanup_callback {
                    cb(daemon_origin);
                }

                // Cleanup source — also tolerant of missing source_handler
                // state, and unregisters the local daemon even if no
                // `source_handler.start_snapshot` was called.
                let _ = self.source_handler.cleanup(daemon_origin);

                let cleanup_msg = MigrationMessage::CleanupComplete { daemon_origin };
                outbound.push(OutboundMigrationMessage {
                    dest_node: dest,
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
                // Retry-safe: `activate()` is idempotent once the migration
                // has been completed, and we route the ack to the recorded
                // orchestrator BEFORE `complete()` transitions state to the
                // idempotency index. If the ack packet is lost, a retried
                // ActivateTarget will find the completed record, return the
                // same replayed_seq, and re-send the ack. The orchestrator
                // therefore can't get wedged waiting for a completion that
                // already happened.
                let replayed_seq = self.target_handler.activate(daemon_origin)?;
                let ack_dest = self
                    .target_handler
                    .orchestrator_node(daemon_origin)
                    .unwrap_or(from_node);
                let ack = MigrationMessage::ActivateAck {
                    daemon_origin,
                    replayed_seq,
                };
                outbound.push(OutboundMigrationMessage {
                    dest_node: ack_dest,
                    payload: wire::encode(&ack)?,
                });
                // `complete()` is idempotent: a retried ActivateTarget
                // after a lost ack re-runs `activate()` (idempotent) and
                // `complete()` (no-op once Complete).
                let _ = self.target_handler.complete(daemon_origin);
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
                reason,
            } => {
                // Fire the SDK's observer BEFORE abort, so the
                // observer sees the structured reason while the
                // migration record is still alive — the SDK uses
                // this to surface NotReady vs terminal to the
                // caller of `MigrationHandle::wait`.
                if let Some(cb) = &self.failure_callback {
                    cb(daemon_origin, reason.clone());
                }
                // Abort on all local handlers. This is correct for
                // terminal reasons; for `NotReady` the SDK may
                // elect to retry, which will re-`start_migration`
                // from scratch (re-snapshotting on local source,
                // re-sending TakeSnapshot on remote source).
                let _ = self.source_handler.abort(daemon_origin);
                let _ = self.target_handler.abort(daemon_origin);
                let _ = self
                    .orchestrator
                    .abort_migration_with_reason(daemon_origin, reason);
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
                let dest = self
                    .target_handler
                    .orchestrator_node(daemon_origin)
                    .unwrap_or(from_node);
                outbound.push(OutboundMigrationMessage {
                    dest_node: dest,
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

        // `source_node` is the daemon's pre-migration host — tracked here
        // only for the target-handler's internal bookkeeping. It is NOT
        // where `RestoreComplete` gets sent (see below).
        let source_node = self
            .orchestrator
            .source_node(daemon_origin)
            .unwrap_or(from_node);

        // If this origin is already under migration here, the source is
        // retrying because our earlier `RestoreComplete` didn't make it
        // back. Don't touch the already-restored daemon; just re-emit
        // `RestoreComplete` so the orchestrator can advance. This also
        // means we DO NOT consume the factory on the retry — the factory
        // registration must survive until the migration is truly complete
        // (`ActivateAck`), not just until the first locally-successful
        // restore.
        if !self.target_handler.is_migrating(daemon_origin) {
            // Readiness check first: if the runtime is still in
            // `Registering`, respond `NotReady` so the source can
            // retry with backoff rather than burning the attempt
            // on a target that's still booting.
            if let Some(readiness) = &self.readiness_callback {
                if !readiness() {
                    return Ok(Some(self.fail_migration_with_reason(
                        daemon_origin,
                        from_node,
                        crate::adapter::net::compute::MigrationFailureReason::NotReady,
                    )?));
                }
            }

            let inputs = match self.target_handler.factories().construct(daemon_origin) {
                Some(i) => i,
                None => {
                    return Ok(Some(self.fail_migration_with_reason(
                        daemon_origin,
                        from_node,
                        crate::adapter::net::compute::MigrationFailureReason::FactoryNotFound,
                    )?));
                }
            };

            // Identity envelope: if the snapshot carries one and we
            // have the X25519 private key to unseal it, the envelope
            // supplies the real daemon keypair. Otherwise fall back
            // to whatever keypair the factory was registered with —
            // which, for public-identity migrations or pre-Stage-5b
            // callers, is either a placeholder or a manually-shared
            // keypair. A present-but-invalid envelope is a hard
            // failure, not a fallback — otherwise an attacker who
            // tampers with the envelope could downgrade identity
            // transport silently.
            let keypair = match self.resolve_restore_keypair(&snapshot, inputs.keypair.as_ref()) {
                Ok(kp) => kp,
                Err(e) => {
                    return Ok(Some(self.fail_migration(
                        daemon_origin,
                        from_node,
                        &format!("identity envelope open failed: {e}"),
                    )?));
                }
            };

            let daemon = inputs.daemon;
            if let Err(e) = self.target_handler.restore_snapshot(
                RestoreContext {
                    daemon_origin,
                    snapshot: &snapshot,
                    source_node,
                    // orchestrator: whoever forwarded SnapshotReady to us
                    orchestrator_node: from_node,
                },
                keypair,
                move || daemon,
                inputs.config,
            ) {
                // Factory is still registered — next `SnapshotReady` for
                // this origin (e.g., from an orchestrator retry) can try
                // again. On successful completion (`complete()`), the
                // factory is auto-removed so a stale or replayed
                // SnapshotReady can't re-trigger restore against what is
                // already the authoritative copy.
                return Ok(Some(self.fail_migration(
                    daemon_origin,
                    from_node,
                    &format!("restore_snapshot failed: {:?}", e),
                )?));
            }

            // Fire the post-restore callback. The SDK-supplied hook
            // drives channel re-bind replay (Stage 3 of the channel
            // re-bind plan): walks the restored daemon's ledger and
            // spawns async `subscribe_channel` calls so publishers
            // start fanning out to the target before the source
            // tears down. Sync callback; the hook itself should
            // `tokio::spawn` the actual work.
            if let Some(cb) = &self.post_restore_callback {
                cb(daemon_origin);
            }
        }

        // Route `RestoreComplete` to the recorded orchestrator. Only the
        // orchestrator holds the migration record; sending to a relay
        // would stall the state machine. `from_node` is used as a
        // fallback when the target-side record has been lost (e.g. a
        // very late chunk after the migration record timed out).
        let reply = MigrationMessage::RestoreComplete {
            daemon_origin,
            restored_seq: seq_through,
        };
        let dest = self
            .target_handler
            .orchestrator_node(daemon_origin)
            .unwrap_or(from_node);
        Ok(Some(vec![OutboundMigrationMessage {
            dest_node: dest,
            payload: wire::encode(&reply)?,
        }]))
    }

    /// Source-side helper: if we have an identity-transport context
    /// and the target's X25519 pubkey is available, seal the
    /// daemon's keypair into the snapshot. On any missing piece
    /// (context, target key, daemon keypair), pass the snapshot
    /// through unchanged — the dispatcher treats a no-envelope path
    /// as "public-identity migration" and expects the target to
    /// have a pre-registered keypair.
    fn maybe_seal_envelope(
        &self,
        snapshot: StateSnapshot,
        daemon_origin: u32,
        target_node: u64,
    ) -> StateSnapshot {
        let Some(ctx) = &self.identity_context else {
            return snapshot;
        };
        // Skip if the snapshot already carries an envelope (e.g. the
        // SDK pre-sealed at `start_migration` time for a local-source
        // case).
        if snapshot.identity_envelope.is_some() {
            return snapshot;
        }
        let Some(target_pub) = (ctx.peer_static_lookup)(target_node) else {
            return snapshot;
        };
        // Find the daemon's keypair in the local registry. The
        // orchestrator + source_handler + target_handler share one
        // registry, so whichever owns this daemon, we see it.
        let kp = match self
            .source_handler_registry_keypair(daemon_origin)
            .or_else(|| self.target_handler_registry_keypair(daemon_origin))
        {
            Some(kp) => kp,
            None => return snapshot,
        };
        // Clone ahead of the consuming `with_identity_envelope` call
        // so we can return the original on seal failure. `StateSnapshot`
        // is `Clone`; the state payload is a `Bytes` (refcount bump),
        // and the remaining fields are small.
        let backup = snapshot.clone();
        match snapshot.with_identity_envelope(&kp, target_pub) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    daemon_origin = format!("{:#x}", daemon_origin),
                    error = %e,
                    "identity envelope seal failed — proceeding without envelope",
                );
                backup
            }
        }
    }

    /// Read-only keypair fetch from the shared daemon registry
    /// reachable via `source_handler`. `source_handler` and
    /// `target_handler` hold `Arc` clones of the same registry in
    /// typical wiring, so checking via source is sufficient; the
    /// `target_handler_registry_keypair` fallback exists for
    /// asymmetric setups where they diverge.
    fn source_handler_registry_keypair(&self, daemon_origin: u32) -> Option<EntityKeypair> {
        let _ = daemon_origin;
        // `MigrationSourceHandler` doesn't expose the registry
        // publicly, so reach through the orchestrator which shares
        // the same `Arc<DaemonRegistry>`.
        self.orchestrator
            .daemon_registry()
            .daemon_keypair(daemon_origin)
    }

    fn target_handler_registry_keypair(&self, daemon_origin: u32) -> Option<EntityKeypair> {
        // Parallel path — the target-side registry may in some
        // configurations be distinct. Today it's the same `Arc`, so
        // this returns the same value as the source path; kept as a
        // seam.
        self.orchestrator
            .daemon_registry()
            .daemon_keypair(daemon_origin)
    }

    /// Target-side helper: pick the keypair to hand to
    /// `restore_snapshot`. Resolution order:
    ///
    /// 1. If the snapshot carries an identity envelope AND we have
    ///    the X25519 private key to unseal it → use the envelope's
    ///    keypair. (Non-envelope cases fall through.)
    /// 2. Otherwise, if `fallback` was provided — the factory was
    ///    registered via `DaemonFactoryRegistry::register` with a
    ///    pre-provisioned keypair — use that.
    /// 3. If neither is available (placeholder registration +
    ///    no envelope in the snapshot), fail: a placeholder factory
    ///    expects the envelope to supply the keypair, and its
    ///    absence means the source skipped identity transport
    ///    without the target being prepared for that.
    ///
    /// Present-but-invalid envelopes are terminal — propagating the
    /// envelope error rather than falling back prevents an attacker
    /// from downgrading identity transport by tampering with the
    /// envelope bytes.
    fn resolve_restore_keypair(
        &self,
        snapshot: &StateSnapshot,
        fallback: Option<&EntityKeypair>,
    ) -> Result<EntityKeypair, String> {
        if let (Some(ctx), Some(_)) = (&self.identity_context, &snapshot.identity_envelope) {
            let priv_secret = x25519_dalek::StaticSecret::from(ctx.local_x25519_priv);
            match snapshot.open_identity_envelope(&priv_secret) {
                Ok(Some(kp)) => return Ok(kp),
                Ok(None) => {
                    // Unreachable under normal conditions — `Some`
                    // envelope on the snapshot paired with
                    // `open_identity_envelope` returning `Ok(None)`
                    // would mean envelope was present but yielded
                    // no keypair. Fall through to fallback.
                }
                Err(e) => return Err(format!("{e}")),
            }
        }
        fallback.cloned().ok_or_else(|| {
            "placeholder factory registered but snapshot has no \
             identity envelope (and no local fallback keypair available)"
                .to_string()
        })
    }

    /// Build a `MigrationFailed` outbound message and clean up local state.
    /// Convenience wrapper that wraps `reason` in
    /// [`MigrationFailureReason::StateFailed`] for generic failures;
    /// callers that need a specific reason code (e.g. `NotReady`,
    /// `FactoryNotFound`) should use
    /// [`Self::fail_migration_with_reason`].
    fn fail_migration(
        &self,
        daemon_origin: u32,
        from_node: u64,
        reason: &str,
    ) -> Result<Vec<OutboundMigrationMessage>, MigrationError> {
        self.fail_migration_with_reason(
            daemon_origin,
            from_node,
            crate::adapter::net::compute::MigrationFailureReason::StateFailed(reason.to_string()),
        )
    }

    /// Build a `MigrationFailed` outbound message with a structured
    /// reason. Clean-up is the same as [`Self::fail_migration`]: the
    /// reassembler entry + target-handler state are dropped so a
    /// retry from the source can start fresh (unless the reason is
    /// `FactoryNotFound` — a retry won't find what isn't there).
    fn fail_migration_with_reason(
        &self,
        daemon_origin: u32,
        from_node: u64,
        reason: crate::adapter::net::compute::MigrationFailureReason,
    ) -> Result<Vec<OutboundMigrationMessage>, MigrationError> {
        tracing::warn!(
            daemon_origin = format!("{:#x}", daemon_origin),
            reason = %reason,
            "migration failed on target",
        );
        self.reassemblers.remove(&daemon_origin);
        let _ = self.target_handler.abort(daemon_origin);
        let msg = MigrationMessage::MigrationFailed {
            daemon_origin,
            reason,
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
            reason: crate::adapter::net::compute::MigrationFailureReason::StateFailed(
                "test failure".into(),
            ),
        };
        let encoded = wire::encode(&msg).unwrap();

        // Should not error — just cleans up
        let outbound = handler.handle_message(&encoded, 0x3333).unwrap();
        assert!(outbound.is_empty());
    }
}
