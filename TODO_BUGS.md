P1 net/crates/net/src/adapter/net/handshake_relay.rs:50-56, 147-154, 188-205 - Relay tamper-binding prologue is never applied
relay_prologue() is documented as binding (initiator, responder) into the Noise transcript so a relay cannot rewrite the node IDs, but neither begin_initiator() nor the responder path actually passes that prologue into NoiseHandshake. As written, a relay can alter the encoded source/destination IDs without triggering the transcript check described in the comments, defeating the intended protection for relayed handshakes.

net/crates/net/src/adapter/net/subprotocol/migration_handler.rs
            daemon_origin,
            restored_seq: seq_through,
        };
        Ok(Some(vec![OutboundMigrationMessage {
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P1: RestoreComplete is sent to source_node, but the RestoreComplete handler requires the orchestrator's migration record. When orchestrator and target are collocated (Some(target) if target == self.local_node_id branch), the source node has no orchestrator record and will fail with DaemonNotFound, stalling the migration. The destination should be the orchestrator node, not the source node.

Prompt for AI agents
@LZL0	Reply...
net/crates/net/src/adapter/net/reroute.rs
            .find(|e| *e.value() == first_alt)
            .unwrap()
            .key();
        policy.on_failure(first_alt_node_id);
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P2: The new regression test can pass without actually validating the bug fix because the second reroute may select B again before recovery.

Prompt for AI agents
Suggested change
        policy.on_failure(first_alt_node_id);
        peers.remove(&0x2222u64);
        policy.on_failure(first_alt_node_id);
        peers.insert(0x2222u64, addr_b);
@LZL0	Reply...
net/crates/net/src/adapter/net/subprotocol/migration_handler.rs
                        // (safe: on_snapshot_ready is idempotent on the
                        // target side because it just re-derives the
                        // forward message we ignore), then reassemble.
                        let _ = self.orchestrator.on_snapshot_ready(
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P2: All errors from on_snapshot_ready are silently discarded via let _ =. While the comment justifies this by idempotency, non-idempotent failures (e.g., unexpected phase transitions) are also swallowed, potentially leaving the orchestrator state machine out of sync. Consider at least logging the error:

if let Err(e) = self.orchestrator.on_snapshot_ready(...) {
    tracing::debug!(?e, "on_snapshot_ready (local target): ignoring");
}
Prompt for AI agents
@LZL0	Reply...
net/crates/net/src/adapter/net/mesh.rs
                        session,
                    },
                );
                ctx.router.add_route(peer_node_id, relay_addr);
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P2: DispatchCtx is missing peer_addrs, so RegisterResponderPeer never registers the new relay-peer in the reroute policy's address map. Every other peer-registration path (connect, accept, connect_via) calls peer_addrs.insert(…). Without it, ReroutePolicy::on_failure cannot save/restore the original route for this peer.

Add peer_addrs: Arc<DashMap<u64, SocketAddr>> to DispatchCtx, pass it in spawn_receive_loop, and insert peer_node_id → relay_addr alongside the existing ctx.peers.insert(…).

Prompt for AI agents
@LZL0	Reply...
net/crates/net/src/adapter/net/handshake_relay.rs
            }
        };

        if parsed.dest_node_id != self.local_node_id {
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P2: Relay forwarding has no TTL/hop-count, so a routing loop between two or more relay nodes will bounce the same packet indefinitely. Consider adding a hop-count byte to the wire header and decrementing it on each forward, dropping the packet when it reaches zero.

Prompt for AI agents
@LZL0	Reply...
net/crates/net/docs/COMPUTE.md
| **Transfer** | Send snapshot to target node via `SUBPROTOCOL_MIGRATION` (0x0500), chunked if needed |
| **Restore** | Reassemble chunks, resolve local `DaemonFactoryRegistry` entry, call `DaemonHost::from_snapshot()` and register the daemon on target. Target starts buffering in-flight events. |
| **Replay** | Target replays buffered events in strict sequence order from source, drains to daemon |
| **Cutover** | Source stops accepting writes and cleans up; daemon is unregistered from source registry |
Contributor
@cubic-dev-ai
cubic-dev-ai bot
10 minutes ago
P3: Move daemon cleanup out of the Cutover phase; code treats cutover as write-stop/routing switch only, and cleanup belongs to the subsequent Complete phase.

P1 net/crates/net/src/adapter/net/mesh.rs:1217 - Relayed handshakes only forward to directly connected peers
HandshakeAction::Forward looks up to_node in ctx.peers and drops the message if there is no direct session. That means a relayed Noise handshake cannot traverse a multi-hop chain, even though the new API and docs describe handshakes as working through a chain of already-connected peers.

P2 bindings/node/src/lib.rs:983 - Invalid event payloads are silently rewritten to empty strings
poll() now converts StoredEvent::raw_str() with unwrap_or(""), which discards any non-UTF-8 payload by turning it into an empty string. That loses event data for binary payloads and makes downstream consumers think the event was empty instead of malformed.

P1 src/adapter/net/subprotocol/migration_handler.rs:456 - Factory entry is removed before the restore completion is actually delivered
remove(daemon_origin) happens immediately after restore_snapshot succeeds, but the RestoreComplete message is only sent later by the receive loop. If the send fails or the node crashes after the restore, the source will retry but the target no longer has the factory entry, turning a transient delivery failure into a permanent migration failure.
P1 src/adapter/net/mesh.rs:1313 - Responder peer is registered only after sending msg2, which can leave the initiator waiting forever on a successful handshake
In RegisterResponderPeer, the code sends the responder's msg2 first and only registers the new peer and route after socket.send_to returns Ok(()). If msg2 is delivered but the task is cancelled/crashes before registration completes, the initiator will derive keys and complete while the responder never records the peer, so subsequent routed traffic to that peer will fail even though the handshake itself succeeded.




P1 src/adapter/net/subprotocol/migration_handler.rs:478 - Migration replies are sent to the immediate hop instead of the orchestrator
RestoreComplete is routed back to from_node, which is only the node that forwarded the packet. In any topology that includes a relay between the target and the orchestrator, this sends the response to the relay, which has no migration record and will drop or error on the message, stalling the migration chain.

P2 bindings/node/src/lib.rs:447 - Binary event payloads are silently converted to empty strings
raw_str().unwrap_or("") turns any non-UTF8 or otherwise undecodable event payload into "" without surfacing an error. That loses data and makes it impossible for consumers to distinguish a genuinely empty payload from a decoding failure.
P1 src/adapter/net/mesh.rs:1180 - Relayed handshake setup can leak a live waiting state on timeout races
connect_via() cancels the pending initiator only after timeout(..., keys_rx) returns. If the relay path delivers the final key result just as the timeout fires, the oneshot may already have been resolved while the initiator entry is still left in pending_initiator until cancellation runs, creating a window where a stale handshake attempt can remain registered and interfere with a retry for the same destination.

net/crates/net/tests/three_node_integration.rs
    a.connect_via(addr_b, &pub_c, nid_c).await.unwrap();

    // Give the spawned msg2-send on C time to resolve before we inspect.
    tokio::time::sleep(Duration::from_millis(300)).await;
Contributor
@cubic-dev-ai
cubic-dev-ai bot
1 minute ago
P3: Avoid a fixed sleep here; wait for c.peer_count() to reach 2 with a timeout so the test doesn't flake on scheduler latency.

Prompt for AI agents
Suggested change
    tokio::time::sleep(Duration::from_millis(300)).await;
    tokio::time::timeout(Duration::from_secs(2), async {
        while c.peer_count() < 2 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("timed out waiting for C to register A");
    
    P2 bindings/node/src/lib.rs:873 - num_shards is silently truncated to 16 bits
MeshOptions.num_shards is accepted as a u32 from JS, but it is cast to u16 without validation. Values above 65,535 will wrap, producing an unexpected shard count and potentially breaking routing/load-balancing behavior in the created MeshNode.
P2 bindings/node/src/lib.rs:900 - node_id() can return a negative value for valid node IDs
node.node_id() is a u64, but the binding converts it to i64 with as i64. Any node ID with the high bit set will be exposed to Node.js as a negative number, which can corrupt identity comparisons and routing logic in consumers that treat the returned value as authoritative.

P1 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:478 - RestoreComplete is routed to the immediate sender instead of the orchestrator
restore_on_target() always sends RestoreComplete back to from_node, but from_node is only the previous hop on the wire. In any relayed or multi-hop deployment, that can be a relay rather than the orchestrator, so the migration record never advances and the full lifecycle can stall waiting for an ack that reached the wrong node.

P1 src/adapter/net/subprotocol/migration_handler.rs:310 - Target migration state is cleared before activation ack is confirmed
ActivateTarget calls target_handler.complete(daemon_origin) before the ActivateAck is sent back to the orchestrator. If that ack is lost or the send fails transiently, a retry will arrive after the target has already discarded its migration state, so the second activation attempt can no longer rely on the original in-flight context.
P2 src/adapter/net/reroute.rs:114 - Reroute policy chooses one alternate for all affected destinations
on_failure picks a single alt_addr once and applies it to every affected route, even though the comment says it should pick the nearest node that can reach each destination. In a heterogeneous topology, that can reroute some destinations through a peer that cannot actually reach them, blackholing traffic until manual correction or recovery.

P2 src/adapter/net/reroute.rs:193 - Multi-hop alternate selection can choose an unreachable second hop
find_graph_alternate() falls back to path[2] when the first hop is the failed node, but that second hop is not guaranteed to be a directly connected peer. The policy then returns that node's address as the next-hop, which can blackhole traffic because the routing table only supports forwarding to peers this node can actually send to directly.
P1 src/adapter/net/subprotocol/migration_handler.rs:309 - Target migration completion is not retry-safe after an ack loss
ActivateTarget calls complete() immediately after activate() and before the ActivateAck is sent back. If that ack packet is dropped or the send fails, the target has already discarded its migration state, so a retried ActivateTarget cannot be handled idempotently and the orchestrator can get stuck waiting for completion.

P2 net/crates/net/src/adapter/net/compute/orchestrator.rs:404 - BufferedEvents decoding can allocate attacker-sized vectors
MSG_BUFFERED_EVENTS reads the event count from the wire and immediately does Vec::with_capacity(count) before validating any upper bound. A malformed packet with a huge count can force a very large allocation attempt, creating a trivial memory-exhaustion DoS on the migration subprotocol.
P1 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:307 and net/crates/net/src/adapter/net/compute/migration_target.rs:225 - Successful migration leaves the daemon factory registered, enabling stale re-restores
After ActivateTarget, the target calls complete() and clears only the in-flight migration state, but the DaemonFactoryRegistry entry is never removed. That means the same daemon_origin can be restored again later from a stale or replayed SnapshotReady, even though the migration is already finished, which can resurrect an authoritative daemon unexpectedly.

P1 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:478-491 - RestoreComplete is sent back to the previous hop instead of the orchestrator
from_node is just the node that forwarded the SnapshotReady packet, not necessarily the migration orchestrator. In any relayed topology, this sends RestoreComplete to the relay, which does not own the migration record, so the lifecycle can stall waiting for an ack that reached the wrong node.
P2 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:307-320 - Activation leaves the factory registry live after completion
complete() removes only the target-side migration state, but the daemon factory entry remains registered indefinitely unless some external caller remembers to remove it. That makes the restore path retryable even after a successful migration, so a stale or replayed SnapshotReady can re-trigger restore logic unexpectedly.
P2 net/crates/net/bindings/node/src/lib.rs:66-74 - num_shards is silently truncated to 16 bits
options.num_shards arrives as a u32, but it is cast to u16 without validation. Values above 65,535 wrap, so JavaScript callers can create a node with a shard count very different from what they requested.
P2 net/crates/net/bindings/node/src/lib.rs:95-100 - node_id() can return negative values for valid IDs
The Rust node ID is a u64, but the binding exposes it as i64 via as. Any ID with the high bit set will appear negative in Node.js, which can break identity comparisons and routing logic in consumers.
P2 net/crates/net/src/adapter/net/reroute.rs:111-146 - One alternate is reused for every failed route
on_failure() computes a single alt_addr and applies it to all affected destinations, even though the topology may require different next hops per destination. In heterogeneous graphs, some of those destinations may not actually be reachable through the chosen alternate, causing traffic to be blackholed until recovery or manual intervention.

P1 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:85-91, 292-304, 478-492 - Migration acknowledgements are sent to the immediate sender instead of the orchestrator
RestoreComplete and the later CleanupComplete/ActivateAck chain use from_node as the destination, but from_node is just the previous hop on the wire. In any topology where the migration messages are relayed, these acknowledgements go to the relay instead of the node that owns the migration record, so the state machine can stall waiting for responses that never reach the orchestrator.
P1 net/crates/net/src/adapter/net/subprotocol/migration_handler.rs:307-320 - Activation is not retry-safe if the final ack is lost
The target calls activate() and immediately complete() before sending ActivateAck. If the ack packet is dropped or the send fails, the target has already discarded its migration state, so a retry cannot be handled idempotently and the orchestrator can remain stuck waiting for completion.
