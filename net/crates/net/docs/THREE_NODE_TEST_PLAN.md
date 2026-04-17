# Three-Node Integration Test Plan

## Why three nodes

The existing test suite covers two-node scenarios (initiator ↔ responder) and in-process unit tests. Two nodes prove encryption, handshake, and point-to-point data flow. But Net is a mesh — the properties that matter only emerge at three nodes:

- **Forwarding.** A packet from A reaches C through B without B decrypting the payload. Two nodes can't test this because there's no relay.
- **Rerouting.** When B dies, A's traffic to C finds another path. With two nodes, a dead peer is just a dead connection — there's no alternative.
- **Mesh topology.** Proximity graphs, capability routing, and subnet gateways require at least three participants to exercise non-trivial decisions.
- **Migration.** Mikoshi's orchestrator, source, and target are three distinct roles on three distinct nodes. The existing migration tests simulate this in-process with shared registries — they don't test actual packet flow between separate adapter instances.
- **Partition.** A network split with two nodes is binary (connected or not). Three nodes can form two partitions (A|BC or AB|C) with asymmetric visibility.

## Architecture

```
         Node A (0x1111)
        /              \
  encrypted           encrypted
  UDP link            UDP link
      /                    \
Node B (0x2222) ——————— Node C (0x3333)
              encrypted
              UDP link
```

Each node is a `NetAdapter` instance with its own keypair, bound to a distinct localhost port. All three perform Noise NKpsk0 handshakes with each other (3 sessions total). The test harness runs all three in a single tokio runtime using separate tasks.

### Test file

`tests/three_node_integration.rs` with `#![cfg(feature = "net")]`.

### Shared helpers

```rust
struct TestNode {
    adapter: NetAdapter,
    node_id: u64,
    port: u16,
    keypair: StaticKeypair,
}

/// Spin up 3 nodes, complete all pairwise handshakes, return them ready to use.
async fn setup_triangle() -> (TestNode, TestNode, TestNode);

/// Find 3 available UDP ports.
async fn find_three_ports() -> (u16, u16, u16);
```

Each test calls `setup_triangle()`, runs its scenario, then shuts down all three nodes. Tests must be independent (no shared state between tests).

---

## Test categories

### 1. Mesh formation

| Test | What it proves |
|------|---------------|
| `test_three_node_handshake` | All 3 pairwise Noise handshakes complete within timeout. Verifies key exchange when each node must manage 2 concurrent sessions. |
| `test_three_node_health` | After handshake, all 3 nodes report `is_healthy() == true`. After shutting down one, the other two still report healthy to each other. |

### 2. Forwarding (relay without decryption)

| Test | What it proves |
|------|---------------|
| `test_relay_through_middle_node` | A sends to C's stream via B. B forwards without decrypting (header-only routing). C receives and decrypts the payload. Verifies the core "untrusted relay" property from the README. |
| `test_relay_preserves_payload_integrity` | A sends 100 events through B to C. C verifies all payloads match what A sent (no corruption from relay). Validates zero-copy forwarding. |
| `test_relay_tamper_detected` | Same as above, but B modifies one byte in the forwarded payload before sending to C. C rejects the packet because the ChaCha20-Poly1305 AEAD tag fails verification. Validates the "compromise of a relay leaks nothing" claim — a malicious relay can't tamper without detection. This is a one-line variant of the integrity test: inject a bit flip between B's receive and forward. |
| `test_relay_hop_count_incremented` | A sends a packet to C via B. C checks that `hop_count` in the received header is 2 (A→B = 1, B→C = 2). Verifies TTL/hop mechanics. |
| `test_relay_ttl_expiry` | A sends a packet with TTL=1 to C via B. B receives it, decrements TTL to 0, and drops it. C receives nothing. Verifies that relay nodes enforce TTL. |

### 3. Rerouting on failure

| Test | What it proves |
|------|---------------|
| `test_reroute_on_node_failure` | A streams events to B. B is shut down mid-stream. A detects failure (heartbeat timeout) and reroutes to C. C receives subsequent events without A explicitly switching targets. Verifies the "silence propagates, mesh reroutes" property. |
| `test_reroute_preserves_causal_chain` | Same as above but with causal ordering enabled. After reroute from B to C, the causal chain is intact — every event's `parent_hash` chains correctly across the reroute boundary. |
| `test_circuit_breaker_trips_on_failure` | A sends to B. B is shut down. A's circuit breaker for the A→B path transitions to Open. After the reset timeout, it transitions to HalfOpen. Verifies the circuit breaker state machine over actual network failure. |
| `test_failure_detection_timing` | A and B are connected. B is shut down. Measure the time from B's last heartbeat to A marking B as Failed. Assert it's within the configured heartbeat interval + timeout threshold. **Important:** on localhost, kernel scheduling adds microseconds of jitter — don't assert nanosecond-scale bounds or the test will be flaky on CI. Use generous bounds (e.g., 2x the configured timeout). The value of this test is proving the detection mechanism works over real sockets, not benchmarking its latency. This is the most important test in the rerouting category. |

### 4. Bidirectional and multi-stream

| Test | What it proves |
|------|---------------|
| `test_bidirectional_simultaneous` | A sends to B while B sends to A simultaneously. Both receive each other's events. Verifies full-duplex operation and that TX/RX key derivation is correct in both directions. |
| `test_three_way_broadcast` | A sends events that should reach both B and C. B and C each receive all events. Verifies one-to-many delivery. |
| `test_independent_streams_no_interference` | A sends stream S1 to B and stream S2 to C concurrently. B only receives S1 events, C only receives S2 events. Verifies stream isolation. |

### 5. Fair scheduling under contention

| Test | What it proves |
|------|---------------|
| `test_fair_scheduling_across_streams` | A opens 10 streams to B, each sending events at different rates. B verifies no stream is starved — every stream's events are received within a bounded time. Verifies the FairScheduler quantum mechanism. |
| `test_priority_bypass` | A sends priority-flagged events and normal events to B. B verifies priority events are received first (bypassing fair scheduling queue). |

### 6. Capability-driven routing

| Test | What it proves |
|------|---------------|
| `test_capability_based_routing` | A announces GPU capability, B announces CPU-only, C needs GPU. C's request routes to A (not B). Verifies the capability index drives routing decisions. |
| `test_capability_diff_propagation` | A announces capabilities. A then publishes a CapabilityDiff removing a capability. B and C observe the diff and update their indexes. Subsequent requests requiring the removed capability no longer route to A. |

### 7. Migration (Mikoshi) over the mesh

| Test | What it proves |
|------|---------------|
| `test_migration_three_node_full_lifecycle` | A (orchestrator) migrates a daemon from B (source) to C (target). Full 6-phase lifecycle over actual encrypted UDP — snapshot, transfer, restore, replay, cutover, complete. Verifies Mikoshi works over the real transport, not just in-process simulations. |
| `test_migration_with_in_flight_events` | B hosts a daemon processing events. A initiates migration to C. Events continue arriving at B during transfer. After cutover, C processes subsequent events. No events are lost (verified by sequence numbers). **This is the test most likely to find bugs the unit tests missed** — the existing migration tests use shared in-process registries and never exercise real packet ordering, timing, or reassembler behavior over UDP. |
| `test_migration_snapshot_chunking_over_wire` | B's daemon has large state (>7KB, spanning multiple chunks). Migration to C reassembles chunks correctly over the wire. Verifies `MAX_SNAPSHOT_CHUNK_SIZE` framing survives real packet boundaries. |
| `test_migration_abort_cleanup` | A starts migrating a daemon from B to C. Migration is aborted mid-transfer. B retains the daemon, C has no stale state. Both B and C are in clean state for future migrations. |

### 8. Partition and healing

**Implementation note:** These are the hardest tests to implement on localhost. You can't cause a real partition between processes on the same machine. Instead, each node needs a configurable filter layer — an `AtomicBool` per peer that the test harness can toggle to make the transport silently drop all packets to/from that peer. This is simulating partition, not causing one. The tests validate the detection and healing logic, not the network event that triggers the detection. The filter should be injected at the transport level (after encryption, before `send_to`) so it's invisible to the protocol layers above.

```rust
/// Per-peer packet filter for partition simulation.
struct PartitionFilter {
    /// Peers whose packets should be silently dropped.
    blocked: DashMap<u64, AtomicBool>,
}
```

| Test | What it proves |
|------|---------------|
| `test_network_partition_detection` | Activate the filter: block all packets between A and {B,C}. B and C detect A as failed via heartbeat timeout. A detects B and C as failed. Both sides enter partition mode independently (no coordination needed). Verifies CONTESTED.md's correlated failure detection logic. |
| `test_partition_healing` | After the partition from above, deactivate the filter. Both sides detect recovery (nodes reappear via resumed heartbeats). Verify the mesh reconverges — all 3 nodes can communicate again. |
| `test_partition_log_reconciliation` | During a simulated partition, A and B both append events to the same entity's causal chain (divergent chains). On healing, conflict resolution runs: longest chain wins, losing chain becomes a ForkRecord. Verify no events are lost. |

### 9. Subnet gateway enforcement

| Test | What it proves |
|------|---------------|
| `test_subnet_gateway_blocks_local_traffic` | A and B are in subnet 1, C is in subnet 2. B is the gateway. A sends a `SubnetLocal` channel event. B does not forward it to C. Verifies visibility enforcement. |
| `test_subnet_gateway_forwards_global` | Same topology. A sends a `Global` channel event. B forwards it to C. Verifies global visibility passes through gateways. |

### 10. Backpressure and load shedding

| Test | What it proves |
|------|---------------|
| `test_backpressure_ring_buffer_full` | A floods B with events faster than B can process. B's ring buffer fills. New events are dropped (DropOldest or DropNewest depending on config). B doesn't crash, doesn't OOM, and recovers when the flood stops. Verifies the "ring buffer is a speed buffer, not a waiting room" property. |
| `test_overloaded_node_goes_silent` | A floods B. B's ring buffer fills and B stops acknowledging heartbeats (overloaded). A's failure detector marks B as degraded. A reroutes to C. Verifies the backpressure → silence → reroute chain. |

---

## Implementation priorities

**Phase 1 — Foundation (do first):**
- `setup_triangle()` helper
- Tests 1.1-1.2 (mesh formation)
- Tests 2.1-2.2 (forwarding)
- Tests 4.1 (bidirectional)

**Phase 2 — Failure and resilience:**
- Tests 3.1-3.4 (rerouting)
- Tests 10.1-10.2 (backpressure)

**Phase 3 — Migration over wire:**
- Tests 7.1-7.4 (Mikoshi)

**Phase 4 — Advanced mesh behavior:**
- Tests 6.1-6.2 (capability routing)
- Tests 8.1-8.3 (partition)
- Tests 5.1-5.2 (fair scheduling)
- Tests 9.1-9.2 (subnet gateway)

## Constraints

- All tests run on `localhost` (no real network required)
- Each test must complete within 30 seconds
- Tests must not depend on execution order
- Port allocation uses `UdpSocket::bind("127.0.0.1:0")` to avoid conflicts
- Timeouts must be generous enough for CI but tight enough to catch real hangs
- Tests use `#[tokio::test]` with multi-threaded runtime (3 nodes need concurrent tasks)

## What this doesn't test

- **Wire latency.** Localhost has ~0 latency. The benchmarks cover timing. Integration tests cover correctness.
- **Real multi-machine deployment.** That's a separate operational test suite, not a cargo test.
- **NAT traversal.** Requires real network topology.
- **Scale beyond 3 nodes.** Correctness properties proven at 3 should hold at N. Performance at scale is a benchmark concern.
