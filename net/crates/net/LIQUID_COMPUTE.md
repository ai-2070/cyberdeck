# [REDACTED] - Live Migration with State Preservation

Assessment of the protocol's capability to support live migration of a container or sensory unit — transferring internal state, memory, connections, and queues from Node A to Node B (better hardware, load balancing, etc.).

## Existing Capabilities

The protocol has a purpose-built **6-phase migration state machine** (`compute/migration.rs`):

```
Snapshot -> Transfer -> Restore -> Replay -> Cutover -> Complete
```

### Capability Breakdown

| Requirement | Status | How |
|---|---|---|
| **Internal state capture** | Done | `StateSnapshot` with binary serialization (`to_bytes`/`from_bytes`) |
| **State restore on target** | Done | `MeshDaemon::restore(state)` on the target node |
| **Queue/event buffering** | Done | Events buffered in `Vec<CausalEvent>` during transfer, replayed before cutover |
| **Causal ordering** | Done | `CausalChainBuilder` with xxh3-hashed chain links — events stay in order |
| **Reliable delivery** | Done | Per-stream `ReliableStream` mode with NACK-based retransmission |
| **Connection multiplexing** | Done | Thousands of concurrent streams per session via `StreamState` DashMap |
| **Routing switch** | Done | Atomic routing cutover so new events go to the target |
| **Superposition state** | Done | Entity exists on both nodes during migration (`Localized -> Spreading -> Superposed -> Collapsed`) |
| **Failure detection** | Done | Heartbeat-based `FailureDetector` + `CircuitBreaker` for failover |
| **Daemon placement** | Done | Capability-based scheduler can place daemons on nodes with required resources |
| **Continuity proofs** | Done | 36-byte `ContinuityProof` covers sequence ranges to verify nothing was lost |
| **Partition recovery** | Done | Log reconciliation after network splits (`contested/reconcile.rs`) |

## Migration Phases (Detail)

### Phase 1: Snapshot
Take a `StateSnapshot` on the source node. The snapshot captures:
- Entity ID (32 bytes)
- Sequence number (8 bytes)
- Causal chain link (24 bytes)
- Serialized daemon state (opaque bytes)
- Observed horizon (for causal consistency)
- Created timestamp

### Phase 2: Transfer
Send the snapshot to the target via `SUBPROTOCOL_MIGRATION (0x0500)`.

### Phase 3: Restore
Call `daemon.restore(state)` on the target node. Start event buffering so nothing arriving during restoration is lost.

### Phase 4: Replay
Replay all buffered events that arrived during the transfer and restore phases. The `Vec<CausalEvent>` buffer captures everything in causal order.

### Phase 5: Cutover
Atomic routing switch — new events route to the target node. During migration, the entity exists on both nodes simultaneously via `SuperpositionState` with phases: `Localized -> Spreading -> Superposed -> ReadyToCollapse -> Collapsed -> Resolved`.

### Phase 6: Complete
Source node cleanup.

## Supporting Infrastructure

- **Session management** — `SessionManager` maintains sessions with `set_session()`, `clear_session()`, `get_session()`. Sessions track multiple streams with individual sequence numbers and reliability modes.
- **Serialization** — `StateSnapshot::to_bytes()` / `from_bytes()` for state transfer. `CausalEvent` serialization in state log. Binary format with explicit length prefixing.
- **Routing adaptability** — `RoutingTable` with per-entry flags, stats, age tracking. Subnet-aware routing with affinity and proximity considerations. Load balancing across multiple paths.
- **Failure detection** — Heartbeat-based `FailureDetector` with recovery via `RecoveryManager`. `CircuitBreaker` with state tracking (Open/Half-Open/Closed).
- **Partition handling** — Mass failure classification by subnet correlation (`contested/partition.rs`). Divergent log healing after partition (`contested/reconcile.rs`).

## Gaps / What Needs Work

### 1. Connection Resumption
After cutover, clients must do a new Noise handshake to the target. There is no session token transfer or transparent redirect mechanism. Clients need to discover the new node independently.

### 2. In-Flight Packet Rebinding
Packets already in transit to the source cannot be redirected mid-flight. Reliable retransmissions may fail if the source is cleaned up before they are ACK'd.

### 3. Queue Backpressure
The inbound `SegQueue` is unbounded. During replay of many buffered events, memory could spike with no backpressure signal to the source.

### 4. Stream State Preservation
The snapshot captures daemon/entity state but not the stream multiplexing state (sequence numbers, open streams). Those reset on restore.

### 5. Cutover Atomicity
The routing switch is not globally atomic. There is a brief window where some peers route to the source and others to the target.

## Feasibility Summary

**For stateless or loosely-connected daemons:** fully feasible today. The causal chaining, event buffering, snapshot/restore framework, and reliable delivery modes handle this well.

**For daemons with many active client connections:** partially feasible. The hard parts — state serialization, causal consistency, event buffering during transfer, the migration state machine — are all implemented. The remaining work is connection orchestration:

1. **Connection resumption** — Implement token-based session transfer (derive token from snapshot ID)
2. **Client redirection** — Publish target node address in registry after Cutover phase
3. **Graceful drain** — Add explicit drain protocol before source cleanup (flush in-flight ACKs)
4. **Queue backpressure** — Wrap `SegQueue` with capacity limits and slow-down signals

### Estimated Coverage: ~70-80%

The protocol provides the foundation. The remaining 20-30% is application-layer coordination on top of what exists.

## Key File References

### Core Migration
- `src/adapter/net/compute/migration.rs` — Migration state machine
- `src/adapter/net/state/snapshot.rs` — State snapshot capture and serialization
- `src/adapter/net/state/causal.rs` — Causal chain building and verification

### Supporting Infrastructure
- `src/adapter/net/session.rs` — Session and stream state management
- `src/adapter/net/reliability.rs` — Packet reliability and retransmission
- `src/adapter/net/router.rs` — Stream routing
- `src/adapter/net/continuity/superposition.rs` — Migration observability
- `src/adapter/net/failure.rs` — Failure detection and recovery
- `src/adapter/net/contested/partition.rs` — Partition detection
- `src/adapter/net/contested/reconcile.rs` — Log reconciliation
