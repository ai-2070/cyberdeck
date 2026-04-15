# Layer 1: Trust & Identity — Implementation Plan

## MVP Scope (3 phases, unblocks L2)

Only phases 1-3 are needed before L2 can begin. Phases 4-7 land incrementally after.

---

## Phase 1: Entity Identity

**New files:** `src/adapter/nltp/identity/mod.rs`, `identity/entity.rs`

**New dependency:** `ed25519-dalek = { version = "2", features = ["rand_core"], optional = true }` gated behind `nltp` feature

Core types:
- `EntityId` — 32-byte ed25519 public key. The canonical identity.
- `EntityKeypair` — wraps `SigningKey`/`VerifyingKey`, provides `sign()` and `verify()`
- `EntityId::origin_hash() -> u32` — truncated BLAKE2s, maps directly to the header field
- `EntityId::node_id() -> u64` — truncated BLAKE2s, replaces the current arbitrary u64 node IDs in swarm/routing

**Modifications:** `Cargo.toml`, `config.rs` (add `entity_keypair` field), `mod.rs` (add `pub mod identity`)

**Key design decision:** Entity identity is tied to a keypair, not a network address. An entity can migrate across nodes. The u64 node IDs from L0 become derived values.

---

## Phase 2: Origin Binding

**New file:** `identity/origin.rs`

Makes `origin_hash` non-zero in every packet. Currently all packets send `origin_hash: 0`.

- `OriginStamp` — holds `EntityId`, caches the `u32` origin hash. Computed once at session creation, reused per-packet. Zero per-packet crypto — it's a single `u32` field write.

**Modifications:**
- `pool.rs` — `PacketBuilder::new()` gains `origin_hash: u32`, `build()` calls `header.with_origin(self.origin_hash)` on every packet
- `session.rs` — derives `origin_hash` from entity keypair, passes to pool
- `mod.rs` — wires it through session creation

---

## Phase 3: Permission Tokens

**New file:** `identity/token.rs`

The authorization primitive that L2 channels will enforce.

```rust
PermissionToken {
    issuer: EntityId,        // 32B — who issued this
    subject: EntityId,       // 32B — who this authorizes
    scope: TokenScope,       // Publish, Subscribe, Admin, Delegate + channel_hash
    not_before: u64,         // unix timestamp
    not_after: u64,          // expiry
    delegation_depth: u8,    // re-delegation limit
    nonce: u64,              // for revocation
    signature: [u8; 64],     // ed25519
}
```

- `TokenCache` — `DashMap<(EntityId, u16), PermissionToken>` for per-channel lookup. Sub-microsecond. Entries evicted on expiry.
- Token verification happens at subscription/session time, NOT per-packet.
- Delegation: A can delegate to B, B can delegate to C, with restricted scope and decremented depth.

---

## Deferrable Phases (post-L2)

### Phase 4: Event Provenance

**New file:** `identity/provenance.rs`

Per-event signing at origin. 128-byte provenance header prepended inside EventFrame (not the NLTP header). Opt-in via `subprotocol_id = 0x0001`. Batch signing via Merkle root for high-throughput streams.

### Phase 5: Trust Scoring

**New file:** `identity/trust.rs`

`TrustLedger` tracking good/bad packets per entity. EWMA scoring. Hooks into `FairScheduler` to deprioritize low-trust nodes.

### Phase 6: Revocation

**New file:** `identity/revocation.rs`

Bounded ring buffer of revoked token nonces. Propagated via meta-stream (`subprotocol_id = 0x0002`). Bounded window makes old tokens implicitly expire.

### Phase 7: Node Attestation

**New file:** `identity/attestation.rs`

Software-only stub for MVP, future TPM/SGX/TrustZone integration.

---

## File Modification Summary

| File | Change | Phase |
|------|--------|-------|
| `Cargo.toml` | Add `ed25519-dalek` | 1 |
| `config.rs` | Add `entity_keypair` field | 1 |
| `mod.rs` | Add `pub mod identity`, wire origin_hash | 1-2 |
| `pool.rs` | `PacketBuilder` gains `origin_hash` param | 2 |
| `session.rs` | Store/pass origin_hash | 2 |
| `protocol.rs` | Add `SUBPROTOCOL_PROVENANCE` constant | 4 |
| `router.rs` | Trust-weighted scheduling | 5 |

## Performance Guarantees

- **Per-packet path:** u32 field write (origin_hash) + optional DashMap get (token cache). Sub-microsecond.
- **ed25519 signing:** ~4us. Only at session/token creation, never per-packet.
- **ed25519 verification:** ~70us. Only at token validation time.
- **No header format change.** Provenance data goes inside EventFrame payload.

MVP is ~1,200-1,500 lines across 4 new files.
