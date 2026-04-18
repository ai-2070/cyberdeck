# Handshake Relay (via Subprotocol)

## Problem

`MeshNode::connect()` requires direct UDP connectivity for the Noise NKpsk0 handshake. If A wants a session with C and only B has direct paths to both, A can't establish the session.

## Design

Handshake messages travel as a normal Net subprotocol (`SUBPROTOCOL_HANDSHAKE = 0x0601`). No separate envelope or magic — the existing Net header, routing, and forwarding handle everything.

### Flow

```
A (initiator)                B (relay)                C (responder)
     │                          │                          │
     │  A has session with B    │  B has session with C    │
     │                          │                          │
     │  1. Build Noise msg1 for C                          │
     │  2. Send as SUBPROTOCOL_HANDSHAKE to B              │
     │     (payload = [dest_node_id:8][noise_msg])         │
     │ ──────(A↔B encrypted)──►│                          │
     │                          │  3. Handler sees HANDSHAKE│
     │                          │  4. Read dest_node_id = C │
     │                          │  5. Forward payload to C  │
     │                          │ ──(B↔C encrypted)──────►│
     │                          │                          │  6. Handler processes
     │                          │                          │  7. Noise msg1 → msg2
     │                          │                          │  8. Send response to B
     │                          │  ◄──(B↔C encrypted)─────│  (HANDSHAKE, dest=A)
     │                          │  9. Forward to A          │
     │  ◄──(A↔B encrypted)─────│                          │
     │ 10. Process Noise msg2   │                          │
     │ 11. Extract session keys │                          │
     │                          │                          │
     │  Session A↔C established                            │
     │  Data via send_routed (A↔C keys, forwarded by B)    │
```

### Why not a separate relay envelope

The initial plan used a custom relay envelope (magic `0x4E52`, separate 24-byte header). Kyra correctly identified this as redundant:

- The Net header already carries dest_id, src_id, subprotocol_id
- The forwarding logic already handles multi-hop routing
- A second magic/header duplicates information and splits the dispatch path
- It breaks the "one 64-byte header everywhere" story

### Payload format

The handshake subprotocol payload carries:

```
[dest_node_id: u64][src_node_id: u64][noise_bytes: variable]
```

- `dest_node_id`: who the handshake is for (C)
- `src_node_id`: who initiated (A)
- `noise_bytes`: opaque Noise NKpsk0 handshake message

B reads `dest_node_id` to know where to forward. B can see the Noise bytes but can't forge them (NKpsk0 authentication) or derive session keys (needs C's private key).

### Security

B sees handshake bytes at the application layer (it decrypts A↔B to read the subprotocol, then re-encrypts B↔C to forward). This is acceptable:

- Noise NKpsk0 bytes are authenticated — B can't forge them
- The PSK prevents B from impersonating A or C
- After handshake completes, A↔C data uses `send_routed` with keys B doesn't have
- This matches the threat model: B is a trusted relay for routing metadata but can't read post-handshake payloads

## Implementation

### Step 1: Constants and payload format (~20 lines)

```rust
pub const SUBPROTOCOL_HANDSHAKE: u16 = 0x0601;

struct HandshakePayload {
    dest_node_id: u64,
    src_node_id: u64,
    noise_bytes: Bytes,
}
```

### Step 2: HandshakeHandler (~80 lines)

Manages per-peer Noise states for in-flight handshakes:

```rust
pub struct HandshakeHandler {
    /// In-flight handshake states: src_node_id → Noise responder state
    pending: DashMap<u64, NoiseResponderState>,
    /// This node's Noise static keypair
    static_keypair: StaticKeypair,
    /// Pre-shared key
    psk: [u8; 32],
}
```

Methods:
- `handle_initiator_msg(src_node_id, noise_bytes)` → creates responder state, returns response bytes
- `handle_responder_msg(dest_node_id, noise_bytes)` → feeds into existing initiator state, returns SessionKeys

### Step 3: Wire into MeshNode receive loop (~20 lines)

In `process_local_packet`, after the migration handler check:

```rust
if parsed.header.subprotocol_id == SUBPROTOCOL_HANDSHAKE {
    if let Some(ref handler) = ctx.handshake_handler {
        // extract payload, process, send response
    }
}
```

### Step 4: `connect_via` on MeshNode (~30 lines)

```rust
pub async fn connect_via(
    &self,
    relay_addr: SocketAddr,
    dest_pubkey: &[u8; 32],
    dest_node_id: u64,
) -> Result<u64, AdapterError>
```

- Creates Noise initiator state
- Sends msg1 as `SUBPROTOCOL_HANDSHAKE` via `send_subprotocol` to the relay
- Waits for response (relay forwards it back)
- Completes handshake, creates NetSession

### Step 5: Tests

- `test_mesh_handshake_via_relay`: A↔B↔C, handshake via subprotocol, data flows after
- `test_mesh_handshake_relay_bidirectional`: C sends data back to A after relayed handshake

## Prerequisite: re-key peers by node_id

The current `MeshNode` stores sessions in `peers: DashMap<SocketAddr, PeerInfo>`. When A connects to C via relay B, both the B session and the C-via-B session share B's wire address. Inserting the C session overwrites B's, breaking the relay.

**Required refactor before handshake relay:**

1. Change `peers: DashMap<SocketAddr, PeerInfo>` → `peers: DashMap<u64, PeerInfo>` (keyed by node_id)
2. Add `addr: SocketAddr` field to `PeerInfo` (where to send packets for this peer — for relay-reached peers, this is the relay's addr)
3. Add `addr_to_node: DashMap<SocketAddr, u64>` as a **fast-path hint** for dispatch — not a sole dispatch key (see below)
4. Update send methods to resolve node_id → addr via `PeerInfo.addr`

### Dispatch key: `session_id`, not source addr

A SocketAddr-keyed map (or a one-to-one `addr → node_id` map) cannot distinguish a direct B session from a relayed C session when both share B's wire address. The authoritative logical key is the inner header's `session_id` (derived from the Noise handshake hash, distinct per session).

`dispatch_packet` uses `session_id` in both branches:

- **Routed packets** (have a routing header, detected by leading bytes ≠ Net magic): look up session by `session_id` directly — source addr is ignored.
- **Direct packets** (start with Net magic): try `addr_to_node` as a fast path, then **validate** the resolved peer's `session.session_id()` matches the packet's `session_id`. On mismatch, fall back to a `session_id` scan of `peers`. `addr_to_node` never entries for relay-reached peers (the addr already points to the relay's own direct session).

This keeps the common case O(1) while correctly handling the collision case — a packet for the relayed C session that happens to arrive from B's wire addr will still dispatch to C via the fallback scan.

This is ~50 lines of refactoring, no new features. After it, handshake relay is straightforward — C stores the A session and B session under different node_ids even though they share B's wire address.

## Scope

With the prerequisite refactor: ~200 lines total (50 re-key + 100 handshake handler + 50 connect_via). 2 tests.
