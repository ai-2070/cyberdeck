# Handshake Relay

## Problem

`MeshNode::connect()` requires direct UDP connectivity to the peer for the Noise NKpsk0 handshake. If A wants to establish a session with C, and only B has direct paths to both, A must first establish a direct UDP path to C (which may be impossible due to NAT, firewall, or network topology).

The data path already supports relay — `send_routed` encrypts for C and forwards through B. But the session setup (handshake) can't use this path. A user who builds A↔B↔C topology hits this wall: "I can't connect to C because I can't reach its port directly."

## Design

### Relay envelope

Handshake packets between A and C are wrapped in a relay envelope and sent through B. B doesn't decrypt the handshake (it's encrypted to C's key) — it just forwards based on the envelope's destination.

```
┌──────────────────────────────────────────────┐
│              Relay Envelope                  │
│  dest_node_id: u64  (who this is for)        │
│  src_node_id:  u64  (who sent it)            │
│  msg_type:     u8   (handshake / data)       │
│  payload_len:  u16                           │
│  ┌──────────────────────────────────────┐    │
│  │  Noise Handshake Packet (opaque)     │    │
│  │  (encrypted to dest's public key)    │    │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

The envelope is NOT encrypted between A and B — it's a cleartext routing wrapper around an already-encrypted handshake message. B reads `dest_node_id`, looks up the peer's address, and forwards the entire envelope.

### Wire format

```
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     MAGIC_RELAY (0x4E52)      |    msg_type   |   reserved    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       dest_node_id (8 bytes)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       src_node_id (8 bytes)                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         payload_len           |          reserved             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    payload (variable)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

Total header: 24 bytes. Magic `0x4E52` ("NR" — Net Relay) distinguishes relay envelopes from Net packets (`0x4E45`) and pingwaves (72 bytes, no magic).

`msg_type`:
- `0x01` = Handshake forward (Noise message)
- `0x02` = Handshake response (Noise message)
- `0x03` = Relay data (future — for relaying post-handshake data through intermediary sessions)

### Flow

```
A (initiator)                B (relay)                C (responder)
     │                          │                          │
     │  connect_via(B, C_pubkey, C_node_id)                │
     │                          │                          │
     │  1. Build Noise msg1     │                          │
     │  2. Wrap in relay envelope (dest=C)                 │
     │  3. Send to B            │                          │
     │ ──────────────────────►  │                          │
     │                          │  4. Read envelope         │
     │                          │  5. Lookup C's address    │
     │                          │  6. Forward envelope to C │
     │                          │ ──────────────────────►  │
     │                          │                          │  7. Unwrap envelope
     │                          │                          │  8. Process Noise msg1
     │                          │                          │  9. Build Noise msg2
     │                          │                          │ 10. Wrap in relay envelope (dest=A)
     │                          │  ◄──────────────────────│ 11. Send to B
     │                          │ 12. Forward to A         │
     │  ◄──────────────────────│                          │
     │ 13. Unwrap envelope      │                          │
     │ 14. Process Noise msg2   │                          │
     │ 15. Extract session keys │                          │
     │                          │                          │
     │  Session A↔C established (keys derived from Noise)  │
     │  A stores session with peer_addr=C (via B)          │
     │  Data flows via send_routed (encrypted for C)       │
```

### Key insight: `peer_addr` tracking

After the relayed handshake, A has a session with C but no direct UDP path. A's `peer_addr` for the C session should be B's address (the relay). When A sends data to C, it goes through B's address. B forwards it to C based on the routing table.

But the session's `session_id` is derived from the Noise handshake — it's the same on both A and C. When C receives a packet from B (relayed from A), the session_id lookup in `dispatch_packet` finds the A↔C session. This already works — the relay test proves it.

### `accept_via` on C's side

C needs to know it should process relayed handshake envelopes. Two options:

**Option A:** C's receive loop automatically processes relay envelopes. When a relay envelope arrives with `dest_node_id == C.node_id` and `msg_type == Handshake`, C unwraps it and processes the Noise message. The response is wrapped in a relay envelope addressed to `src_node_id` and sent back through the same source address (B).

**Option B:** C explicitly calls `accept_relayed()` which listens for relay envelopes.

Option A is simpler — the receive loop handles it transparently. No special API needed on the responder side.

## Implementation

### Step 1: Relay envelope (~50 lines)

New file `src/adapter/net/relay_envelope.rs`:
- `RelayEnvelope` struct: `dest_node_id`, `src_node_id`, `msg_type`, `payload`
- `to_bytes()` / `from_bytes()` with `MAGIC_RELAY` (`0x4E52`)
- `RelayMsgType` enum: `Handshake`, `HandshakeResponse`

### Step 2: Receive loop — relay envelope dispatch (~30 lines)

In `MeshNode::dispatch_packet`, add a check before the existing magic-byte discrimination:
- If first 2 bytes == `0x4E52` → relay envelope
- If `dest_node_id == self.node_id` → unwrap and process locally
- If `dest_node_id != self.node_id` → forward to the dest's address via peers map

For handshake processing on the responder side:
- Extract the Noise message from the envelope
- Run the Noise responder flow (same as `try_handshake_responder`)
- Wrap the response in a relay envelope addressed to `src_node_id`
- Send back through the source address (the relay)

### Step 3: `connect_via` on MeshNode (~40 lines)

New method:
```rust
pub async fn connect_via(
    &self,
    relay_addr: SocketAddr,    // B's address
    dest_pubkey: &[u8; 32],    // C's Noise public key
    dest_node_id: u64,         // C's node ID
) -> Result<u64, AdapterError>
```

- Build Noise initiator msg1
- Wrap in relay envelope (dest=`dest_node_id`, src=`self.node_id`)
- Send to `relay_addr`
- Wait for relay envelope response from the relay
- Unwrap, process Noise msg2
- Extract session keys
- Store session with `peer_addr = relay_addr` (data goes through relay)

### Step 4: Tests (~80 lines)

- `test_mesh_node_handshake_relay`: A connects to C through B. A has no direct path to C. The handshake completes via relay envelope through B. After handshake, A sends data to C via `send_routed` through B. C receives it.
- `test_mesh_node_handshake_relay_bidirectional`: After relayed handshake, C can also send data back to A through B.

## What changes, what doesn't

| Component | Changes? | Why |
|-----------|----------|-----|
| `NetSession` | No | Session keys come from Noise, same as before |
| `PacketBuilder` | No | Data packets are built the same way |
| Noise handshake logic | **Reused** | Same `NoiseHandshake::initiator/responder`, just wrapped in relay envelope |
| `dispatch_packet` | **Yes** | New check for relay magic `0x4E52` |
| `MeshNode` | **Yes** | New `connect_via` method, relay envelope handling in receive loop |

## Security

- B cannot read the handshake messages (encrypted to C's Noise key)
- B cannot forge handshake responses (would need C's private key)
- B can drop or delay handshake messages (same as any relay — handled by retry)
- B learns that A is trying to connect to C (topology leak, same as routing headers)
- The relay envelope is cleartext — B sees `src_node_id` and `dest_node_id`

This matches the threat model from the README: "Untrusted relay. Nodes forward packets without decrypting payloads."

## Scope

~120 lines of new code: relay envelope (50), dispatch changes (30), connect_via (40). Two new tests. The relay envelope format is simple and extensible (future `msg_type` values for other relayed operations).
