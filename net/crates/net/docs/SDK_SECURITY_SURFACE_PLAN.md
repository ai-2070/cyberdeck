# SDK security surface plan — identity, capabilities, subnets

## Context

The `net` crate ships a three-layer security / organization model that no SDK exposes today:

1. **Identity** — ed25519-rooted entities. `EntityKeypair` signs, `PermissionToken` delegates, `TokenCache` caches hot lookups. See [`IDENTITY.md`](IDENTITY.md) for the design.
2. **Capabilities** — what a node can do: hardware, software, models, tools, tags, limits. `CapabilitySet` is the value type; `CapabilityAnnouncement` is the wire primitive; `CapabilityIndex` is the queryable sidecar.
3. **Subnets** — hierarchical 4-level grouping for routing/visibility. `SubnetId` (bit-packed u32), `SubnetPolicy` (tag → level mapping), `SubnetGateway` (forwarding-time enforcement). See [`SUBNETS.md`](SUBNETS.md).

These compose: identity issues tokens, tokens reference capabilities, capabilities derive subnet membership, channels gate access via all three. The [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md) explicitly cuts channel auth from Stages 6–7 for this reason — without an identity + token surface in the SDK, a caller can't construct a `PermissionToken` to pass to `subscribeChannel`.

This plan closes that gap. It is **additive** on top of [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md) and assumes Stages 1–3 of that plan have landed (CortEX re-exports and TS/Python cortex surface).

## Scope

**In scope:**
- Key/identity management: generate, load, save, sign, derive `origin_hash` / `node_id`.
- Token issuance, verification, delegation, caching — the `PermissionToken` lifecycle.
- Capability declaration and filter-based queries.
- Subnet assignment and visibility.
- Wiring these into the channel and Redex auth paths **already built** in the core crate.

**Out of scope:**
- Revocation lists. `PermissionToken` has `not_after`; short TTL + re-issuance is the deliberate v1 answer. A CRL/OCSP-style revocation surface is a v2 design.
- Secure enclaves / hardware-backed keys. `EntityKeypair` is caller-owned bytes; HSM integration is a separate crate.
- New crypto primitives. ed25519 + BLAKE2s is the choice; this plan doesn't revisit it.
- Capability discovery/gossip protocol changes. Today announcements go over `CapabilityAnnouncement`; that wire format stays as-is.

## Coverage today

| Feature | Rust SDK | TS SDK | Python SDK | Go SDK |
|---|---|---|---|---|
| `EntityKeypair` generate / sign | ✗ | ✗ | ✗ | ✗ |
| `PermissionToken` issue / verify | ✗ | ✗ | ✗ | ✗ |
| `TokenCache` | ✗ | ✗ | ✗ | ✗ |
| `CapabilitySet` construction | ✗ | ✗ | ✗ | ✗ |
| `CapabilityAnnouncement` emit | ✗ | ✗ | ✗ | ✗ |
| `CapabilityIndex` query | ✗ | ✗ | ✗ | ✗ |
| `SubnetId` / `SubnetPolicy` | ✗ | ✗ | ✗ | ✗ |
| `SubnetGateway` forwarding | ✗ (core only) | ✗ | ✗ | ✗ |
| Channel auth (token-gated) | blocked on above | blocked | blocked | blocked |

Everything is core-only today. SDK users have literally no way to build a `PermissionToken`.

## Design principles

1. **Security defaults safe, not permissive.** `Net::builder()` today produces an anonymous node. Adding identity should be opt-in (not breaking) but once opted in, defaults should match the production recommendations in [`IDENTITY.md`](IDENTITY.md) (short token TTLs, small delegation depth, token required for global-visibility channels).
2. **Keys are caller-owned.** No SDK method writes to a hardcoded path. Users pass bytes in or a path in; the SDK only operates on what they hand it. This avoids fighting every user's secret-management strategy (vault, k8s secrets, envelope encryption, enclave).
3. **Zero-cost when unused.** A node that doesn't set an identity pays nothing — no signing, no cache, no announcement. Current performance profile must not regress for the "anonymous mesh" use case.
4. **Composition via handles, not globals.** `Identity` is a handle a user creates and hands to `Net::builder().identity(id)` (and `MeshBuilder::identity(id)`). No thread-local, no `static`, no env-var fallback. Explicit beats magic.
5. **SDK hides the primitives that don't round-trip cleanly.** `CapabilityFilter` is a value type with builder chains — clean. `AuthGuard`'s bloom filter is not — it's a runtime construct. SDKs expose `TokenCache` (the thing users reason about) and hide `AuthGuard` (the thing the transport uses internally); the transport wires itself up from the `TokenCache` the user provided.

## Staged rollout

Shippable order:

1. **Stage A — Rust SDK re-exports + `Identity` handle** (2–3 days). All three areas re-exported under feature flags, plus one ergonomic `Identity` wrapper that bundles `EntityKeypair` + `TokenCache` + capability announcement helpers.
2. **Stage B — Token issuance & verification in NAPI + TS SDK** (3–4 days). Pure-compute surface; no network. Highest-leverage TS addition because it unblocks channel auth.
3. **Stage C — Capabilities (declare, announce, query) across Rust/TS** (3–4 days). Grafts onto `Mesh`/`MeshNode`. Network-adjacent but bounded.
4. **Stage D — Subnet configuration across Rust/TS** (2–3 days). Smallest network surface; `SubnetPolicy` + `SubnetId` on `MeshBuilder`.
5. **Stage E — Wire channel auth through the SDK surface** (3–5 days). With identity + capabilities + subnets all addressable, channel auth becomes a config option. Closes the cut from [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md) Stages 6–7.
6. **Stage F — Python surface (identity + capabilities + subnets + auth)** (1 week). Repeat A–E against the PyO3 layer.
7. **Stage G — Go surface (identity + capabilities + subnets + auth)** (1–2 weeks). New C ABI additions; biggest lift because nothing exists today.

**Why this order, not "ship one area end-to-end":** Stage A unblocks Stage B unblocks Stage E. Capabilities (C) and subnets (D) are orthogonal and could reorder. Python (F) and Go (G) come last because they're repeats, not new design — and the Rust/TS design needs to settle before it's duplicated three more times.

---

## Stage A — Rust SDK security surface

Goal: a Rust user can construct an `Identity`, issue tokens, declare capabilities, and bind a subnet without touching the core `net` crate.

### Feature flags

Add to `net/crates/net/sdk/Cargo.toml`:

```toml
[features]
identity = ["net/net"]       # identity types live in adapter::net
capabilities = ["net/net"]   # same
subnets = ["net/net"]        # same
security = ["identity", "capabilities", "subnets"]
```

Everything pulls in the existing `net` feature (`net-sdk`'s gate for the core's `net` feature — which enables the mesh transport). The underlying types live in `adapter::net::identity` / `::behavior::capability` / `::subnet`, so there's no finer-grained gate to reach for. Bundle all three SDK features under `security` for the common case.

### Surface — new `sdk/src/identity.rs`

```rust
//! Identity handle — keypair + token cache + announcement helpers.
//!
//! Built once at node start, handed to `Net::builder()` /
//! `MeshBuilder::identity(...)`. Owns the ed25519 signing key; the
//! transport borrows it for `OriginStamp` derivation and
//! token-gated subscribe checks.

pub use ::net::adapter::net::identity::{
    EntityId, EntityKeypair, OriginStamp, PermissionToken, TokenCache,
    TokenScope, TokenError,
};

/// Caller-owned identity bundle. Cheap to clone (internal Arc).
pub struct Identity {
    keypair: Arc<EntityKeypair>,
    cache: Arc<TokenCache>,
}

impl Identity {
    /// Generate a fresh ed25519 identity. Use once at first-run; persist
    /// the returned bytes via [`Self::to_bytes`] and reload with
    /// [`Self::from_bytes`] on subsequent runs.
    pub fn generate() -> Self;

    /// Load from a 32-byte seed (caller-owned storage — disk, vault, HSM).
    pub fn from_seed(seed: [u8; 32]) -> Self;

    /// Serialize the identity (seed + capability metadata) for persistence.
    /// 32 bytes + small header. Treat as secret material.
    pub fn to_bytes(&self) -> Vec<u8>;

    /// Load a previously-serialized identity.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TokenError>;

    /// Root identifier. Ed25519 public key, 32 bytes.
    pub fn entity_id(&self) -> EntityId;

    /// Derived 32-bit hash used in packet headers.
    pub fn origin_hash(&self) -> u32;

    /// Derived 64-bit node id used for routing/addressing.
    pub fn node_id(&self) -> u64;

    /// Sign arbitrary bytes (typically a capability announcement or a
    /// token issuance). Returns a 64-byte ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> [u8; 64];

    /// Issue a scoped permission token to `subject`. Short TTLs + periodic
    /// re-issuance is the designed v1 answer to revocation.
    pub fn issue_token(
        &self,
        subject: EntityId,
        scope: TokenScope,
        channel: ChannelName,
        ttl: Duration,
        delegation_depth: u8,
    ) -> PermissionToken;

    /// Install a token this node received from another issuer. Used on
    /// subscribers who hold a delegated publish/subscribe grant.
    pub fn install_token(&self, token: PermissionToken) -> Result<(), TokenError>;

    /// Look up a cached token for `(entity, channel)`. Sub-microsecond.
    pub fn lookup_token(&self, entity: EntityId, channel: &ChannelName)
        -> Option<PermissionToken>;
}
```

### Surface — `sdk/src/lib.rs` wiring

```rust
#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "identity")]
pub use identity::Identity;

impl NetBuilder {
    #[cfg(feature = "identity")]
    pub fn identity(mut self, identity: Identity) -> Self { /* ... */ }
}

#[cfg(feature = "net")]
impl MeshBuilder {
    #[cfg(feature = "identity")]
    pub fn identity(mut self, identity: Identity) -> Self { /* ... */ }
}
```

Anonymous mode (no `identity(...)` call) continues to work unchanged — this is a non-breaking addition.

### Surface — new `sdk/src/capabilities.rs`

Mostly re-exports + a short ergonomic chain on `Mesh`:

```rust
pub use ::net::adapter::net::behavior::capability::{
    CapabilitySet, CapabilityAnnouncement, CapabilityFilter,
    CapabilityLimits, CapabilityRequirement, Hardware, Software,
};

impl Mesh {
    /// Declare this node's capabilities. Signs the announcement with
    /// the node's `Identity` (panics if no identity has been set — use
    /// `try_announce_capabilities` for a fallible variant).
    #[cfg(feature = "identity")]
    pub async fn announce_capabilities(&self, caps: CapabilitySet)
        -> Result<(), SdkError>;

    /// Query the local capability index for peers matching `filter`.
    /// Returns (node_id, score) pairs ordered by score descending.
    pub fn find_peers(&self, filter: &CapabilityFilter)
        -> Vec<(u64, f64)>;
}
```

### Surface — new `sdk/src/subnets.rs`

```rust
pub use ::net::adapter::net::subnet::{
    SubnetId, SubnetPolicy, SubnetRule, SubnetGateway, Visibility,
};

impl MeshBuilder {
    /// Pin this node to a specific subnet hierarchy. Overrides the
    /// default, which is `SubnetId::GLOBAL` (no subnet restriction).
    pub fn subnet(mut self, id: SubnetId) -> Self;

    /// Attach a `SubnetPolicy` that re-derives subnet assignment from
    /// capability tags at announcement time. Mutually exclusive with
    /// `subnet(id)` — the builder validates at `.build()`.
    pub fn subnet_policy(mut self, policy: SubnetPolicy) -> Self;
}
```

### Exit criteria

- `net-sdk` with `--features security` re-exports cleanly.
- One doctest: generate identity → issue token → `install_token` on a second identity → verify.
- Anonymous-mode benchmark comparison: no regression in emit/poll throughput.
- README: new "Security" section next to "Mesh Streams."

---

## Stage B — Token surface in NAPI + TS

The pure-compute half of identity is the highest-leverage TS addition because it unblocks channel auth without requiring network wiring first.

### NAPI additions — `bindings/node/src/identity.rs` (new)

```rust
#[napi]
pub struct Identity { inner: Arc<net_sdk::Identity> }

#[napi]
impl Identity {
    #[napi(factory)]
    pub fn generate() -> Self;

    #[napi(factory)]
    pub fn from_seed(seed: Buffer) -> napi::Result<Self>;

    #[napi(factory)]
    pub fn from_bytes(bytes: Buffer) -> napi::Result<Self>;

    #[napi]
    pub fn to_bytes(&self) -> Buffer;

    #[napi(getter)]
    pub fn entity_id(&self) -> Buffer;         // 32 bytes

    #[napi(getter)]
    pub fn origin_hash(&self) -> u32;

    #[napi(getter)]
    pub fn node_id(&self) -> BigInt;

    #[napi]
    pub fn sign(&self, message: Buffer) -> Buffer;

    #[napi]
    pub fn issue_token(
        &self,
        subject: Buffer,              // 32-byte entity_id
        scope: Vec<String>,           // ["publish", "subscribe", ...]
        channel: String,
        ttl_seconds: u32,
        delegation_depth: u8,
    ) -> napi::Result<Buffer>;        // serialized PermissionToken

    #[napi]
    pub fn install_token(&self, token: Buffer) -> napi::Result<()>;

    #[napi]
    pub fn lookup_token(&self, entity: Buffer, channel: String)
        -> Option<Buffer>;
}
```

Tokens cross the NAPI boundary as opaque `Buffer`s (serialized `PermissionToken`). The TS SDK wraps this with a `Token` class for type safety; it never hand-rolls the bytes.

### TS SDK — `sdk-ts/src/identity.ts` (new)

```typescript
export class Identity {
  static generate(): Identity;
  static fromSeed(seed: Buffer): Identity;           // 32 bytes
  static fromBytes(bytes: Buffer): Identity;

  toBytes(): Buffer;                                 // persist this
  readonly entityId: Buffer;                         // 32 bytes
  readonly originHash: number;
  readonly nodeId: bigint;

  sign(message: Buffer): Buffer;

  issueToken(opts: {
    subject: Buffer;                                 // 32-byte entityId
    scope: ('publish' | 'subscribe' | 'admin' | 'delegate')[];
    channel: string;
    ttlSeconds: number;
    delegationDepth?: number;                        // default 0
  }): Token;

  installToken(token: Token): void;
  lookupToken(entity: Buffer, channel: string): Token | null;
}

export class Token {
  readonly bytes: Buffer;                            // opaque serialized form
  readonly issuer: Buffer;
  readonly subject: Buffer;
  readonly scope: ReadonlySet<'publish' | 'subscribe' | 'admin' | 'delegate'>;
  readonly channel: string;
  readonly notBefore: Date;
  readonly notAfter: Date;
  readonly delegationDepth: number;

  static parse(bytes: Buffer): Token;
  verify(issuerEntityId: Buffer): boolean;
  isExpired(atMs?: number): boolean;
}
```

### Errors

Add `IdentityError`, `TokenError` to `sdk-ts/src/errors.ts`. Prefix dispatch: `"identity:"`, `"token:"`. `TokenError` has sub-kinds (`expired`, `invalid_signature`, `wrong_channel`, `delegation_exhausted`) surfaced as a `.kind` discriminator.

### Exit criteria

- Round-trip test: generate identity → `toBytes` → `fromBytes` → verify entity id matches.
- Issue → serialize → parse → verify ✓; tamper bytes → verify ✗.
- Delegation: A issues to B with depth=2 → B re-issues to C with depth=1 → C re-issues to D with depth=0 (D cannot further delegate).

---

## Stage C — Capabilities across Rust + TS

### NAPI — `bindings/node/src/capabilities.rs` (new)

```rust
#[napi(object)]
pub struct CapabilitySetJs {
    pub hardware: Option<HardwareJs>,
    pub software: Option<SoftwareJs>,
    pub models: Vec<ModelJs>,
    pub tools: Vec<ToolJs>,
    pub tags: Vec<String>,
    pub limits: Option<CapabilityLimitsJs>,
}

#[napi]
impl NetMesh {
    #[napi]
    pub async fn announce_capabilities(&self, caps: CapabilitySetJs)
        -> napi::Result<()>;

    #[napi]
    pub fn find_peers(&self, filter: CapabilityFilterJs)
        -> Vec<PeerMatchJs>;
}
```

### TS SDK — `sdk-ts/src/capabilities.ts` (new)

```typescript
export interface CapabilitySet {
  hardware?: { cpus: number; memoryMb: number; gpus?: Gpu[] };
  software?: { os: string; version: string };
  models?: Array<{ name: string; version: string; contextTokens?: number }>;
  tools?: Array<{ name: string; version?: string }>;
  tags?: string[];
  limits?: { maxInflight?: number; maxMessageBytes?: number };
}

export interface CapabilityFilter {
  requireTags?: string[];
  requireModels?: string[];
  requireTools?: string[];
  minMemoryMb?: number;
  minGpuVramMb?: number;
}

// on MeshNode
async announceCapabilities(caps: CapabilitySet): Promise<void>;
findPeers(filter: CapabilityFilter): Array<{ nodeId: bigint; score: number }>;
```

### Exit criteria

- Two-node test: A announces `{tags:["gpu","inference"]}`, B's `findPeers({requireTags:["gpu"]})` returns A.
- Announcement TTL expiry: advance mock clock past TTL → `findPeers` no longer returns A until re-announce.

---

## Stage D — Subnets across Rust + TS

### NAPI — extend `bindings/node/src/mesh.rs`

```rust
#[napi(object)]
pub struct SubnetIdJs {
    /// 1–4 hierarchy levels, each 0–255. Example: `[3, 7, 2]` = region 3,
    /// fleet 7, vehicle 2, subsystem unset.
    pub levels: Vec<u8>,
}

#[napi(object)]
pub struct MeshConfig {
    // existing fields ...
    pub subnet: Option<SubnetIdJs>,
    pub subnet_policy: Option<SubnetPolicyJs>,
}
```

### TS SDK — extend `sdk-ts/src/mesh.ts`

```typescript
export interface MeshConfig {
  // existing fields ...
  subnet?: number[];                                 // 1–4 levels
  subnetPolicy?: {
    rules: Array<{ tagPrefix: string; level: number; value: number }>;
  };
}
```

Visibility on channel config (Stage E) will accept `'subnet-local' | 'parent-visible' | 'exported' | 'global'`.

### Exit criteria

- Three-node test, A `[3,7,2]` / B `[3,7,3]` / C `[3,8,1]`: channel with `visibility:'subnet-local'` at level 2 delivers A↔B only.

---

## Stage E — Wire channel auth through the SDK

With identity + capabilities + subnets addressable, channel auth becomes an `SdkError`-surfaced option on channel config.

### Rust SDK — extend `Mesh::register_channel` / `publish` / `subscribe_channel`

```rust
pub struct ChannelConfig {
    pub name: ChannelName,
    pub visibility: Visibility,
    pub reliable: bool,
    pub require_token: bool,
    pub publish_caps: Option<CapabilityFilter>,
    pub subscribe_caps: Option<CapabilityFilter>,
}

pub struct SubscribeOptions {
    pub token: Option<PermissionToken>,
    pub timeout: Option<Duration>,
}

impl Mesh {
    pub async fn subscribe_channel_with(
        &self,
        channel: &ChannelName,
        opts: SubscribeOptions,
    ) -> Result<()>;
}
```

Existing `subscribe_channel(name)` is kept as a sugar alias for `subscribe_channel_with(name, Default::default())`.

### TS SDK — mirror on `MeshNode`

```typescript
interface ChannelConfig {
  name: string;
  visibility: 'subnet-local' | 'parent-visible' | 'exported' | 'global';
  reliable: boolean;
  requireToken?: boolean;
  publishCaps?: CapabilityFilter;
  subscribeCaps?: CapabilityFilter;
}

async subscribeChannel(name: string, opts?: {
  token?: Token;
  timeoutMs?: number;
}): Promise<void>;
```

Errors: `ChannelAuthError` (from [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md) Stage 6) now actually has something to do — it surfaces when `requireToken=true` and the subscriber doesn't pass a valid `Token`. `TokenError.kind === 'expired'` / `'wrong_channel'` / etc. distinguishes user-recoverable cases (refresh token) from permanent denials.

### Exit criteria

- Subscribe with `requireToken=true` and no token → `ChannelAuthError`.
- Subscribe with expired token → `TokenError { kind: 'expired' }`.
- Subscribe with valid token → succeeds; publish gated similarly.
- Capability-filtered channel: node lacking required tag → rejected with clear error.

---

## Stages F & G — Python and Go

### Python (Stage F)

PyO3 surface mirrors Stage A–E. `PermissionToken` cross boundary as `bytes` (opaque). `Identity` is a `#[pyclass]`. Errors: `IdentityError`, `TokenError(kind)` subclasses of existing `CortexError`-sibling error base.

Files:
- `net/crates/net/bindings/python/src/identity.rs` (new)
- `net/crates/net/bindings/python/src/capabilities.rs` (new)
- `net/crates/net/bindings/python/src/subnets.rs` (new)
- `net/crates/net/bindings/python/python/__init__.py` — add exports

No async/await needed — identity operations are synchronous, announcement is fire-and-forget.

### Go (Stage G)

New C ABI surface. `Identity` as opaque handle; tokens cross as `(*C.char, C.size_t)`. Scope as bitfield `uint8`. Errors extend `errorFromCode`:

```go
const (
    ErrTokenExpired      = C.NET_ERR_TOKEN_EXPIRED
    ErrTokenInvalidSig   = C.NET_ERR_TOKEN_INVALID_SIG
    ErrTokenWrongChannel = C.NET_ERR_TOKEN_WRONG_CHANNEL
    ErrCapabilityDenied  = C.NET_ERR_CAPABILITY_DENIED
    ErrSubnetIsolated    = C.NET_ERR_SUBNET_ISOLATED
)
```

Files:
- `net/crates/net/bindings/go/src/identity.rs` (new) — C ABI
- `net/crates/net/bindings/go/include/net.h` — additions
- `net/crates/net/bindings/go/net/identity.go` (new)
- `net/crates/net/bindings/go/net/capabilities.go` (new)
- `net/crates/net/bindings/go/net/subnets.go` (new)

Go channel-based watch patterns don't apply here — identity is all synchronous compute.

---

## Critical files (Rust + TS, the on-ramp)

### Stage A (Rust)
- `net/crates/net/sdk/Cargo.toml` — add `identity`, `capabilities`, `subnets`, `security` features.
- `net/crates/net/sdk/src/lib.rs` — module wiring, `NetBuilder::identity`, `MeshBuilder::identity` / `subnet` / `subnet_policy`.
- `net/crates/net/sdk/src/identity.rs` (new) — `Identity` handle.
- `net/crates/net/sdk/src/capabilities.rs` (new) — re-exports + `Mesh::announce_capabilities` / `find_peers`.
- `net/crates/net/sdk/src/subnets.rs` (new) — re-exports + builder methods.

### Stage B (TS tokens)
- `net/crates/net/bindings/node/src/identity.rs` (new) — NAPI `Identity` class.
- `net/crates/net/sdk-ts/src/identity.ts` (new) — `Identity`, `Token` wrappers.
- `net/crates/net/sdk-ts/src/errors.ts` — add `IdentityError`, `TokenError`.

### Stage C (TS capabilities)
- `net/crates/net/bindings/node/src/capabilities.rs` (new).
- `net/crates/net/sdk-ts/src/capabilities.ts` (new).
- `net/crates/net/sdk-ts/src/mesh.ts` — add `announceCapabilities`, `findPeers`.

### Stage D (TS subnets)
- `net/crates/net/sdk-ts/src/mesh.ts` — extend `MeshConfig` with `subnet` / `subnetPolicy`.
- `net/crates/net/bindings/node/src/mesh.rs` — extend config object.

### Stage E (channel auth)
- `net/crates/net/sdk/src/mesh.rs` — extend `register_channel`, `publish`, `subscribe_channel_with`.
- `net/crates/net/sdk-ts/src/mesh.ts` — mirror.

---

## Open questions / risks

### Security posture

- **Default TTLs for `issue_token`.** [`IDENTITY.md`](IDENTITY.md) argues for short TTLs. The SDK surface should not default to infinity. Pick `15m` as the SDK default and document.
- **Delegation depth default.** 0 (no re-delegation) is the safe default. Users who want delegation set it explicitly.
- **`scope` as string union vs bitfield.** TS surface uses `('publish' | 'subscribe' | ...)[]` for readability. NAPI converts to `TokenScope` bitfield. Round-trip tests must verify the `admin` + `delegate` combinations survive.
- **Anonymous-mode compatibility.** Confirm the existing anonymous emit/poll/mesh path produces packets with `origin_hash=0` and that channel gate `require_token=false` accepts them. No change needed if already true; document explicitly.

### Key material handling

- **`to_bytes` / `from_bytes` contains a private key.** The SDK doc must say "treat as secret material" on every method. TS Buffer is not zeroed on drop — accept this for v1, note it.
- **No key rotation helper.** If a user wants to rotate, they generate a new `Identity`, re-announce capabilities, and issue new tokens with the new issuer. The SDK does not orchestrate this.
- **HSM integration.** Not in v1. The `Identity` trait is concrete (`struct Identity`), not abstract, to keep the surface tight. A future `trait Signer` exists in IDENTITY.md's design; surface change to a trait would be breaking and should happen before 1.0, not after.

### Cross-SDK consistency

- **Token wire format stability.** `PermissionToken` is 159 bytes today. Once SDKs expose `toBytes()` / `fromBytes()` any change to the layout is a breaking change across every SDK simultaneously. Add a version byte at position 0 before Stage B ships (mirrors the `NetDbBundle` version-byte ask from [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md)).
- **Scope bitfield extension.** The four scopes today (`PUBLISH`, `SUBSCRIBE`, `ADMIN`, `DELEGATE`) consume 4 bits. Explicitly reserve bits 4–7 for future expansion and have SDKs ignore (with warning) any bits they don't recognize.

### Performance

- **`find_peers` cost.** `CapabilityIndex::query` is DashMap lookups; fast. But returning `Vec<(u64, f64)>` for a large filter match on a large mesh could be big. Add an implicit limit (top 100 by score) in the SDK and expose `findPeersLimit` for the rare caller who wants more.
- **Token cache size.** `TokenCache` is unbounded today (expired entries evicted on access). For long-lived nodes with many delegations this could grow. Add a soft cap (e.g., 10_000 entries) in the SDK `Identity` wrapper; evict LRU when over cap. Not a core-crate change.
- **Signing on the hot path.** `announceCapabilities` signs. If a user hammers it (bad idea), they pay ~70µs per call. Document as "call on state change, not in a tight loop."

### Auth turn-on risk

- **Existing anonymous tests.** Test suite must continue to pass with anonymous mode. Gate any new "require token by default" logic behind explicit config — never flip the default on SDK callers who haven't opted in.
- **Channel registration with auth and no identity.** If a user calls `registerChannel({requireToken:true})` on a `MeshNode` that has no `Identity`, fail fast at `registerChannel` with a clear error, not later on a subscribe attempt.

### Scope cuts carried from other plans

- Revocation lists: deferred to v2.
- Hardware key storage (HSM, TPM, Secure Enclave): separate crate, not in this plan.
- Dynamic capability policies (change `SubnetPolicy` at runtime): defer; v1 is set-at-start.

---

## Sizing

| Stage | SDKs touched | Est. effort |
|---|---|---|
| A. Rust SDK re-exports + `Identity` | Rust | 2–3 days |
| B. Token surface NAPI + TS | NAPI + TS | 3–4 days |
| C. Capabilities Rust + TS | NAPI + SDK + TS | 3–4 days |
| D. Subnets Rust + TS | SDK + TS | 2–3 days |
| E. Channel auth wiring | SDK + TS | 3–5 days |
| F. Python surface | PyO3 + Python | ~1 week |
| G. Go surface | C ABI + Go | 1–2 weeks |

Total: ~5–7 weeks. Each stage is an independent PR.

## Dependencies

- Stage E depends on [`SDK_EXPANSION_PLAN.md`](SDK_EXPANSION_PLAN.md) Stage 6 (channels on Mesh) having landed — auth without a channel surface is useless.
- Stages F/G depend on Stages A–E having stabilized.
- No dependency on the CortEX/RedEX SDK work beyond Stage 1 (feature-flag layout).

## Out of scope (for this plan)

- Revocation, HSM, dynamic subnet re-policy — deferred.
- Daemons / MigrationOrchestrator ACLs — handled in whatever plan covers that surface.
- Auditing / access logs — a platform concern, not an SDK one.
