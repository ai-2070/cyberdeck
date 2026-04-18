# Secure EntityKeypair transfer during migration

## Status

Not implemented. Today the target must have the daemon's `EntityKeypair` pre-registered in its `DaemonFactoryRegistry` before migration can complete. That's a deployment step, not part of the protocol, and it means migrating to a node that wasn't provisioned ahead of time is impossible. This plan adds one new message to the migration subprotocol so the source hands the private key to the target end-to-end encrypted during the migration itself.

## Problem

A restored daemon on the target must keep signing causal-chain events under the original entity's identity. `DaemonHost::from_snapshot` requires an `EntityKeypair` (ed25519 signing key) whose public half matches `snapshot.entity_id`. Today the only path for the target to hold that keypair is out-of-band provisioning via `DaemonFactoryRegistry::register(keypair, config, factory)` before the migration fires.

Consequences:

- Migration targets are constrained to nodes that were pre-provisioned for *every daemon they might ever host*. That's a config-management burden and a liveness footgun — a migration into a "cold" node fails with `MigrationFailed("no daemon factory registered…")`.
- Deployments that want "any capable node can become a target" (common in autoscaling or failure-driven re-placement) can't use migration without either blasting private keys to every node ahead of time (bad: huge blast radius) or out-of-band key-transfer (bad: outside the protocol, per-deployment, easy to get wrong).

The protocol has the pieces to fix this. Source and target can negotiate a direct Noise NKpsk0 session via the routed-handshake machinery that ships today; that session carries end-to-end-confidential payloads neither the orchestrator nor any relay can read. Adding one message type to the migration subprotocol that travels over that session lets the source hand the private key to the target at the moment it's needed and nowhere else.

## Design

### One new wire message

```rust
MigrationMessage::KeyTransfer {
    daemon_origin: u32,
    keypair_bytes: Vec<u8>,  // ed25519 SigningKey bytes (32 bytes)
}
```

- Encoded inside `SUBPROTOCOL_MIGRATION` (0x0500) like every other migration message.
- Sent **source → target direct**, not through the orchestrator. Must ride a routed Noise session between source and target so a relay can't read it. The orchestrator orders the flow but never sees the key.

### Flow

```
Orchestrator (A)        Source (B)              Target (C)
       │                    │                        │
       │ TakeSnapshot ─────►│                        │
       │                    │  (take snapshot)       │
       │◄──SnapshotReady────│                        │
       │ SnapshotReady─────────────────────────────► │
       │                    │                        │
       │                    │ ── KeyTransfer ───────►│  (direct B↔C session)
       │                    │                        │   verifies keypair matches
       │                    │                        │   snapshot.entity_id, stores
       │                    │                        │   in per-origin slot
       │                    │                        │
       │                    │                        │   restore_snapshot proceeds
       │◄──RestoreComplete──────────────────────────│
       │   …buffered events, replay, cutover…        │
       │ ActivateTarget────────────────────────────► │
       │◄──ActivateAck───────────────────────────────│
       │                    │                        │
       │                    │  (zeroize local copy)  │
```

After `ActivateAck`, the source erases its local copy of the keypair. Before that point the key exists on both nodes as a hedge against target-side failure mid-restore.

### Ordering

`KeyTransfer` arrives at the target **before** restore is attempted. Two ways to ensure that:

1. **Source emits it first** — right after `TakeSnapshot` processing, before the first `SnapshotReady` chunk. Simple but makes the source responsible for ordering a message to a non-orchestrator peer in the middle of an orchestrator-driven flow.
2. **Target buffers SnapshotReady chunks until KeyTransfer arrives** — the subprotocol handler sees SnapshotReady without a corresponding key slot, holds the chunks in the existing reassembler, and only triggers restore once both the assembled snapshot *and* the key are present.

(2) is more robust. The reassembler already tolerates chunks arriving in any order. We extend the "ready to restore" check from "all chunks received" to "all chunks received AND key present."

### Keypair slot in `DaemonFactoryRegistry`

Today each `FactoryEntry` is `{factory, keypair, config}`. Split into two forms of registration:

- `register_local(keypair, config, factory)` — today's API. Keypair pre-provisioned; migration to this node doesn't need `KeyTransfer`.
- `register_transient(config, factory)` — keypair absent. Migration to this node **requires** a `KeyTransfer` before restore. The subprotocol handler installs the received keypair into the slot atomically with the restore attempt.

`DaemonFactoryRegistry::construct(origin)` returns a richer enum:

```rust
enum ConstructedInputs {
    /// Keypair already on the target; hand to restore_snapshot directly.
    Ready { daemon, keypair, config },
    /// Target accepts this origin but needs KeyTransfer before restore.
    NeedsKey { daemon, config, factory_handle: FactoryHandle },
}
```

`FactoryHandle` is a small token the handler keeps alive while waiting for the key, so the construct → key-received → restore path is race-free.

### Authentication

The key itself is the authentication. `restore_snapshot` already verifies `snapshot.entity_id.origin_hash() == daemon_origin` and `snapshot.entity_id == keypair.entity_id()`. An attacker would need to forge an ed25519 private key whose public half matches `snapshot.entity_id`; that's an ed25519 preimage attack, i.e., not feasible. So:

- Target receives `KeyTransfer { daemon_origin, keypair_bytes }`.
- Parses 32 bytes → `SigningKey` → derives public key → derives `EntityId` → checks `entity_id == the-buffered-snapshot's entity_id`.
- If match: install into the factory slot, proceed with restore.
- If mismatch: drop the transfer, emit `MigrationFailed`.

No separate signature is needed — the check above is equivalent to "sender had the private key" because producing a matching pair without the private key is computationally infeasible.

### Confidentiality

`KeyTransfer` is sent over a direct source↔target Noise session. That session's transport keys are derived from the NKpsk0 handshake between those two peers specifically; no other node has them. The migration subprotocol payload inside that session is AEAD-encrypted hop-by-hop (which is end-to-end when the hops are just source and target).

If source and target don't already have a direct session:

- Use the routed-handshake plumbing that ships today: `source.connect_via(first_hop_addr, &target_pubkey, target_node_id)` completes an end-to-end Noise session through the existing routing topology.
- Trigger this automatically at the start of migration — the source handler calls `connect_via` (or `connect_routed` once the multi-hop routing discovery plan lands) before it's about to send `KeyTransfer`.

### Source-side zeroization

After `ActivateAck`:

- `MigrationSourceHandler::cleanup(origin)` grows to also `drop_keypair(origin)` — calls `Zeroize::zeroize` on the in-memory keypair bytes, drops the handle, and removes the daemon from the local factory registry if it was there.
- The `EntityKeypair` struct should implement `Zeroize` + `ZeroizeOnDrop`. `ed25519_dalek::SigningKey` is already zeroize-aware; we just need to propagate it through our wrapper.

This means a node that was the source before cutover and survives still has a window where it held the key; after `ActivateAck` that window closes and the key only lives on the target. If the source is compromised *before* `ActivateAck`, the attacker still has the key — but they already had it pre-migration, so this isn't a new exposure.

## Threat model — what this plan defends against

- **Orchestrator compromise.** Orchestrator never sees the key. A compromised orchestrator can route the handshake to a wrong target (see below), but can't read the key material.
- **Relay compromise.** Relays on the source↔target path see Noise ciphertext, nothing else.
- **Pre-provisioned-key leak.** Deployments that don't pre-provision can't leak what they never had.
- **Replay.** Noise transport keys are per-session; an old ciphertext of `KeyTransfer` from a previous session fails AEAD verification under the new session's keys.
- **Keypair forgery.** Target verifies `keypair.entity_id == snapshot.entity_id`; forging a matching private key requires breaking ed25519.

## Out of scope

- **Hostile target chosen by the orchestrator.** If the orchestrator picks a target that is in fact an attacker-controlled node, the source's `KeyTransfer` goes to the attacker. Defenses here are deployment-level (orchestrator trust, capability policies, or an additional "attestation" subprotocol) and not this plan.
- **Forward secrecy of past events.** If a target is compromised later, the attacker holds the current signing key and can impersonate the daemon going forward. They cannot forge past events because past events are pinned by `parent_hash` chaining — a new event with the same sequence number but different content doesn't produce the same chain hash.
- **Per-event authentication of the wire payload beyond Noise AEAD.** Noise gives integrity and confidentiality; we don't add another signature layer.
- **HSM / non-extractable keys.** Nodes that want to keep keys in hardware can't migrate this way by definition; that's a separate deployment pattern (groups, not migration).

## Implementation

Rough ordering so each step is independently reviewable and testable.

### Step 1: `EntityKeypair` zeroization (~15 lines)

- `src/adapter/net/identity/entity.rs`: derive/implement `Zeroize` + `ZeroizeOnDrop` on `EntityKeypair`.
- Add a `zeroize` dep to `Cargo.toml` if not already there.

### Step 2: `KeyTransfer` wire message (~40 lines)

- `src/adapter/net/compute/orchestrator.rs`:
  - `MigrationMessage::KeyTransfer { daemon_origin, keypair_bytes }` variant.
  - Codec tag `MSG_KEY_TRANSFER = 10`.
  - Encode/decode branches. Reject lengths other than exactly 32 bytes for `keypair_bytes` at decode.

### Step 3: `DaemonFactoryRegistry::register_transient` + two-variant `construct` (~50 lines)

- `src/adapter/net/compute/daemon_factory.rs`:
  - Internal entry becomes `{factory, config, keypair: Option<EntityKeypair>}`.
  - Public API: `register_local(keypair, config, factory)` (existing `register` gets renamed) and `register_transient(config, factory)`.
  - `ConstructedInputs` enum with `Ready` / `NeedsKey` variants.
  - `install_keypair(origin, keypair)` to set the keypair after a transient entry receives `KeyTransfer`.

### Step 4: Subprotocol handler — buffer snapshot until key is present (~80 lines)

- `src/adapter/net/subprotocol/migration_handler.rs`:
  - When `SnapshotReady` completes reassembly, check `factories.construct(origin)`.
    - `Ready`: proceed with restore exactly as today.
    - `NeedsKey`: buffer the parsed `StateSnapshot` in a new `pending_key_transfer: DashMap<u32, PendingKeyRestore>`. Don't emit `RestoreComplete` yet.
  - New `MigrationMessage::KeyTransfer` branch:
    - Parse `keypair_bytes` → `EntityKeypair`.
    - If a buffered snapshot for this origin is present: verify `keypair.entity_id == snapshot.entity_id`. If yes, install keypair, run restore, emit `RestoreComplete`. If no, emit `MigrationFailed`.
    - If no buffered snapshot: install keypair into the factory slot and wait for the snapshot to arrive. Same verification on arrival.

### Step 5: Source-side emit + source↔target session (~60 lines)

- `src/adapter/net/compute/migration_source.rs`:
  - `MigrationSourceHandler::emit_key_transfer(origin) -> MigrationMessage::KeyTransfer { … }`.
  - The subprotocol handler, after receiving `TakeSnapshot` and producing the `SnapshotReady` chunks, *also* produces `KeyTransfer` as an additional outbound message addressed to the target. The orchestrator doesn't route `KeyTransfer` — the source sends it direct (the source handler knows the target node_id from the `TakeSnapshot` message, and its `MeshNode` resolves the route).

- `src/adapter/net/mesh.rs`:
  - Add a thin helper on `MeshNode` for "send this migration message direct to this peer," reusing `send_subprotocol` over whichever session is appropriate. If no direct session exists, the handler first calls `connect_routed(target_pubkey, target_node_id)`. (Target pubkey needs to travel with the `TakeSnapshot` message — see Step 6.)

### Step 6: `TakeSnapshot` carries `target_pubkey` (~10 lines)

- `src/adapter/net/compute/orchestrator.rs`:
  - `MigrationMessage::TakeSnapshot { daemon_origin, target_node, target_pubkey: [u8; 32] }`.
  - Orchestrator populates `target_pubkey` from its own peer table (it must know the target's pubkey to have started the migration in the first place).
  - Source needs this to bring up a direct session with the target for `KeyTransfer`.

### Step 7: Source cleanup erases the key (~15 lines)

- `src/adapter/net/compute/migration_source.rs`:
  - `MigrationSourceHandler::cleanup(origin)` → also calls `factories.unregister(origin)` if the source held a `register_local` entry for this daemon, zeroing the in-memory keypair.
  - Called from the subprotocol handler when processing `CutoverNotify` / on path to `CleanupComplete`.

### Step 8: Tests

Unit:
- `KeyTransfer` codec roundtrip.
- `DaemonFactoryRegistry::register_transient` + `install_keypair` cycle; `construct` returns `NeedsKey` before and `Ready` after.
- Keypair / snapshot `entity_id` mismatch on `KeyTransfer` → `MigrationFailed`.

Integration (`tests/migration_integration.rs`):
- `test_migration_transient_target_receives_keypair_from_source`: target starts with `register_transient`, source sends `TakeSnapshot`, full 6-phase chain completes. Target holds the keypair after `ActivateAck`; source's factory entry is gone.
- `test_migration_transient_target_buffers_snapshot_before_key`: `SnapshotReady` arrives before `KeyTransfer`. Target correctly defers restore. Once `KeyTransfer` arrives, restore fires. Assert `RestoreComplete` is emitted exactly once.
- `test_migration_transient_target_buffers_key_before_snapshot`: opposite ordering. Same assertions.
- `test_migration_keypair_mismatch_fails`: hand-craft a `KeyTransfer` whose keypair doesn't match the snapshot's entity_id. Target emits `MigrationFailed`, does not register the daemon.

Integration (`tests/three_node_integration.rs`):
- `test_migration_full_lifecycle_with_transient_target_over_wire`: three nodes over real UDP, target has only `register_transient`. Migration completes via the new flow.

### Step 9: Documentation

- `docs/COMPUTE.md`: update the 6-phase section to describe the `KeyTransfer` step, the `register_transient` option, and the source-side zeroization.
- `docs/MIGRATION_LIFECYCLE_PLAN.md`: add a "follow-up landed" note referencing this plan.

## Scope

~270 lines added across `identity/entity.rs`, `compute/orchestrator.rs`, `compute/daemon_factory.rs`, `compute/migration_source.rs`, and `subprotocol/migration_handler.rs`; ~150 lines of tests; ~30 lines of doc updates. One new wire message (tag `0x0A`) — coordinated rollout required if any deployment is already running migration across versioned nodes.

## Rollout coexistence

Nodes running the old code still work: `register_local` is unchanged, `KeyTransfer` is a no-op on their side (they never emit it and never expect it). Mixed-version deployments are safe as long as any node that wants to *receive* a migration is either (a) pre-provisioned via `register_local` (old path) or (b) running the new code and using `register_transient` (new path). A new-code source migrating to an old-code target works if the target is pre-provisioned.
