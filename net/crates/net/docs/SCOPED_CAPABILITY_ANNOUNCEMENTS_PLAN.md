# Scoped capability announcements — PLAN

**Status.** Proposed. Builds directly on shipped multi-hop capability
propagation (`MULTIHOP_CAPABILITY_PLAN.md`). The follow-up that plan
explicitly named (`ann.scope: AnnouncementScope` defaulting to
`Global`, enforced at the forwarder before re-broadcast) is this
plan's `Stage S-2`.

## Context

`CapabilityAnnouncement` is unconditionally global today. An origin
signs a `CapabilitySet` and the mesh fan-outs it to every directly-
connected peer (`mesh.rs:2802` session-open push +
`announce_capabilities` periodic broadcast); multi-hop forwarders
re-broadcast up to `MAX_CAPABILITY_HOPS = 16` per
`MULTIHOP_CAPABILITY_PLAN.md`. The receiver-side filter is signature
+ TTL + size; nothing about *who* should see the announcement enters
the picture.

That works for an open mesh where every node is a candidate for
every workload. It breaks down for three real cases observable
today:

1. **Subnet-local workloads.** A node tagged `region:eu-west` may
   be unable to legally serve traffic from `region:us-east`; the
   capability announcement still propagates there, gets indexed by
   `find_best`, and shows up in scoring even though routing /
   compliance gates make it unusable. Today the operator has to
   filter at query time, which is post-hoc and brittle.
2. **Private capabilities.** A node may want to advertise GPU
   inventory only to a known orchestrator entity (or set of
   entities), not to every peer in a 1000-node mesh. There's no
   way today to express "this announcement is for entity X
   exclusively"; the only knob is "don't announce."
3. **Cross-subnet exposes.** The opposite of (1): a gateway-class
   node *intentionally* exposes itself across subnets to act as a
   bridge. There's no way to declare that intent in the
   announcement itself — it has to be inferred from tags.

The channel layer already has [`Visibility`](../src/adapter/net/channel/config.rs)
for the analogous problem on data packets (`SubnetLocal`,
`ParentVisible`, `Exported`, `Global`). Capability announcements
should grow a parallel concept so the mesh doesn't need a
capability layer that's permissively global riding on top of a
data layer that isn't.

## Goals

**In scope.**

- A signed `AnnouncementScope` field on `CapabilityAnnouncement` that
  the origin chooses and forwarders / receivers enforce.
- Three first-class scope shapes:
  - `Global` (default; current behavior; on-the-wire identical to
    pre-Stage announcements via `skip_serializing_if`).
  - `SubnetLocal` — visible only within the origin's own subnet
    (computed via `SubnetPolicy::assign` against the origin's
    `CapabilitySet`).
  - `Audience(Vec<EntityId>)` — visible only to listed entities.
- Forwarder-side enforcement: drop before re-broadcast when the
  next-hop peer is not in scope.
- Receiver-side enforcement: drop before indexing when *self* is not
  in scope. Defense-in-depth — forwarders are best-effort; the
  receiver gate is authoritative for inclusion in `CapabilityIndex`.
- Wire-format compatibility with pre-Stage nodes via
  `#[serde(default, skip_serializing_if = "is_global")]` so an
  origin emitting `Scope::Global` produces the same bytes (and
  therefore the same signature) as today.
- SDK surface (Rust / TS / Py / Go) plumbed through to the binding
  layer — `announce_capabilities` gains an optional scope arg.

**Non-goals.**

- A query-time scope filter on `CapabilityIndex::query()` /
  `find_best()`. Enforcement at index-insert time means the index
  is *already* scoped from the receiver's perspective; a query that
  walks `self.nodes` returns only entries the receiver was supposed
  to see. No additional `viewer:` argument.
- Channel-scoped announcements (announcement valid only in context
  of channel C). Channel auth and capability auth are orthogonal
  today; tying them is a v2 conversation. Flag in follow-ups.
- Filter-scoped announcements (visible only to peers whose own
  caps match a `CapabilityFilter`). The chicken-and-egg problem —
  the receiver may not have announced yet — pushes this to v2.
- Revocation primitive. The existing TTL eviction + re-announce
  with tighter scope is sufficient and matches how the rest of
  the system handles state changes.

## Design invariants

Same non-negotiables as the multi-hop plan, plus two scope-specific
ones:

1. **Scope is signed.** Embedding `AnnouncementScope` in the
   signature payload (alongside the rest of the fields) is the only
   way to prevent a forwarder from widening the audience. A
   tampering forwarder could in principle *narrow* the audience by
   refusing to forward, but that's already the steady-state
   behavior of an honest forwarder enforcing scope.
2. **Forwarders read scope without verifying signature.** The
   forwarder's job is "decide whether to re-broadcast to peer
   `next_hop`." It doesn't need to verify the signature to do that
   — the receiver re-checks. Forwarders that *can* verify (because
   they have the origin's `EntityId` in their entity index) MAY do
   so, but it's not required. This matches the existing
   "forward-then-verify-on-terminal-receive" pingwave pattern.
3. **Audience size is bounded.** `Audience(Vec<EntityId>)` is
   capped at `MAX_AUDIENCE_ENTITIES = 32`. Anything larger
   indicates a use-case mismatch (you want `SubnetLocal` or
   `Global`, not a hand-rolled allowlist). Origin-side reject;
   receiver-side reject so a forged-larger payload doesn't survive.

## Key decisions

### §1 — single enum vs. parallel fields

Single enum (`AnnouncementScope`). A separate
`subnet_local: bool` + `audience: Vec<EntityId>` pair makes
nonsensical combinations representable (`subnet_local=true,
audience=[X]` — does the receiver need to be both in the subnet
*and* in the audience, or either?). The enum forces a single,
unambiguous answer per announcement and keeps the wire format
flat — the variant tag is one byte, the payload is what it needs
to be.

### §2 — `Visibility` reuse vs. parallel type

Channel `Visibility` already has `SubnetLocal | ParentVisible |
Exported | Global`. Tempting to reuse. **Don't.** Channels and
announcements have different scoping needs:

- Channels carry data; visibility decides whether *packets* leave
  a subnet. Subnet *gateways* enforce.
- Announcements carry metadata; scope decides whether *peers* are
  told the origin exists. Forwarders enforce, but the granularity
  also includes per-entity audience lists, which channels don't
  need.

Define `AnnouncementScope` as its own type. The naming convention
makes the parallel obvious without coupling the implementations —
if channels later want an `Audience` variant, they can borrow the
shape.

### §3 — `SubnetLocal` semantics

`SubnetLocal` means "visible only to peers that, when their own
caps are run through `SubnetPolicy::assign`, produce the same
`SubnetId` as the origin." The forwarder computes this:

```rust
fn peer_in_scope(&self, peer_caps: &CapabilitySet, ann_scope: &AnnouncementScope) -> bool {
    match ann_scope {
        AnnouncementScope::Global => true,
        AnnouncementScope::SubnetLocal => {
            let origin_subnet = self.subnet_policy.assign(/* origin caps */);
            let peer_subnet = self.subnet_policy.assign(peer_caps);
            origin_subnet == peer_subnet
        }
        AnnouncementScope::Audience(entities) => {
            entities.iter().any(|e| e == peer_entity_id)
        }
    }
}
```

For `SubnetLocal`, the origin's caps are inside the announcement,
so the forwarder can run `SubnetPolicy::assign` without external
state. The peer's caps come from `CapabilityIndex::get(peer_node)`
— if we don't yet have an indexed entry for the peer, we **err on
the side of forwarding** (treat scope as Global for that hop). The
receiver will still drop it if scope mismatches at terminal
ingestion. This avoids the bootstrapping problem where every node
in a fresh mesh withholds announcements from every other node
because nobody has indexed anybody yet.

### §4 — `Audience` requires session-bound entity_id

For `Audience`, the forwarder needs to know the next-hop peer's
`EntityId` to compare against the allowlist. `EntityId` is bound
to a session at handshake time (Noise NKpsk0 establishes the
peer's static key, which the binding layer turns into an
`EntityId`). The `MeshNode::peer_entity_ids: DashMap<u64,
EntityId>` index already exists for channel auth (see `mesh.rs`
search for `peer_entity_ids`); reuse it.

If the index doesn't have an entry for the next-hop peer (rare —
indicates a session that established without entity binding),
default to *not* forwarding. `Audience` semantics demand stricter
than-default behavior; bootstrapping that has to wait one full
handshake cycle is acceptable.

### §5 — receiver-side enforcement

`handle_capability_announcement` (`mesh.rs`, the dispatch hook for
`SUBPROTOCOL_CAPABILITY_ANN`) gains a scope check before calling
`ctx.capability_index.index(ann)`:

```rust
match &ann.scope {
    AnnouncementScope::Global => { /* accept */ }
    AnnouncementScope::SubnetLocal => {
        let my_subnet = self.subnet_policy.assign(&self.local_caps);
        let origin_subnet = self.subnet_policy.assign(&ann.capabilities);
        if my_subnet != origin_subnet {
            // Drop: origin scoped to its own subnet, we're in a different one.
            return Ok(());
        }
    }
    AnnouncementScope::Audience(entities) => {
        if !entities.iter().any(|e| e == &self.entity_id()) {
            // Drop: we're not in the audience.
            return Ok(());
        }
    }
}
```

Drops are silent — no error metric increment, no log at warn level.
A scope drop is the *expected* outcome for out-of-scope peers, not
an anomaly. Bump a `capability_scope_dropped` counter at trace level
for diagnostics.

### §6 — wire format & signature

Add to `CapabilityAnnouncement`:

```rust
pub struct CapabilityAnnouncement {
    // … existing fields …

    /// Visibility scope for this announcement. Defaults to `Global`,
    /// in which case the field is omitted from the serialized form
    /// so pre-Stage-S nodes verify identical signed bytes (the same
    /// `#[serde(default, skip_serializing_if)]` trick that
    /// `hop_count` and `reflex_addr` use).
    #[serde(default, skip_serializing_if = "AnnouncementScope::is_global")]
    pub scope: AnnouncementScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnnouncementScope {
    #[default]
    Global,
    SubnetLocal,
    Audience(Vec<EntityId>),
}

impl AnnouncementScope {
    pub fn is_global(&self) -> bool {
        matches!(self, AnnouncementScope::Global)
    }
}
```

`signed_payload()` includes `scope` *unless* it is `Global`, in
which case it serializes the same bytes as today. Old nodes that
omit the field deserialize it as `Global` via `#[serde(default)]`.
New origins emitting `Global` produce identical canonical bytes
for verification on old verifiers — preserves the rolling-upgrade
property documented in `MULTIHOP_CAPABILITY_PLAN.md` for
`hop_count`.

### §7 — query-time semantics

No change to `CapabilityIndex::query()` /
`CapabilityIndex::find_best()`. The index is populated at receive
time, scope is enforced before insertion, so `self.nodes` already
reflects only the announcements the receiver was supposed to see.
A subsequent query that walks `self.nodes` is correctly scoped by
construction.

This collapses what would otherwise be a thorny question ("how
does a query express the viewer's identity for scope filtering?")
into the receive-time gate. It also means scope changes at the
origin propagate via the existing TTL / version mechanism: a re-
announce with a tighter scope evicts out-of-scope receivers'
entries via TTL within `ttl_secs` (default 300s), then the next
re-announce simply isn't accepted by them.

### §8 — origin self-validation

The origin SHOULD validate its own scope choice before broadcast:

- `SubnetLocal` requires the origin's caps to produce a non-default
  `SubnetId` via `SubnetPolicy::assign`. If they don't, log at warn
  and downgrade to `Global` (announcing `SubnetLocal` from a node
  with no subnet tags is a bug, not a feature).
- `Audience` requires `entities.len() <= MAX_AUDIENCE_ENTITIES` and
  `entities.len() > 0`. Empty audience is a footgun (announcement
  goes nowhere, including self-index — deliberate origin self-mute
  should use the existing "don't call announce" path). Reject at
  the API surface, not at the wire.

### §9 — split horizon + scope interaction

Multi-hop today already has split-horizon (don't re-broadcast back
to the route's next-hop toward origin). Scope adds a new drop axis
that runs *after* split-horizon: a peer that survives split-horizon
may still be dropped because it's out of scope. The order matters
for accounting — split-horizon drops shouldn't increment the
scope counter, and vice versa.

### §10 — bootstrapping & late joiners

Late-joiner convergence today relies on session-open re-push
(`mesh.rs:2802`). Scope doesn't change this — the late joiner
still receives the announcement, evaluates scope at receipt time,
and either indexes it or drops it. No special-cased path.

For `SubnetLocal` specifically, a late joiner whose
`CapabilityAnnouncement` hasn't yet propagated has no
`peer_entity_ids` entry on the forwarder side; per §3, the
forwarder forwards anyway and the receiver decides. This is the
correct behavior — defaulting to drop would leave subnet-local
late joiners blind until the next periodic re-announce cycle.

## Wire format change

`CapabilityAnnouncement` gains one field: `scope: AnnouncementScope`.

- **Serialization** in `to_bytes()` / `signed_payload()` — append
  scope after existing fields, omitted via `skip_serializing_if`
  when `Global`.
- **Variant tag** for the enum — postcard's default discriminant
  encoding is one byte. `Global=0, SubnetLocal=1, Audience=2`.
- **`Audience` payload** — length-prefixed list of 32-byte
  `EntityId`s. Length capped at `MAX_AUDIENCE_ENTITIES = 32`,
  enforced both at origin (API-level) and receiver (deserialization
  rejects oversized).

Old nodes that don't know about scope deserialize the absent field
as `Global` (via `serde(default)`); this is the same forward-compat
trick `hop_count` and `reflex_addr` already use.

## Forwarder behavior

`forward_capability_announcement` (the existing multi-hop
forwarder) gains a per-peer scope check immediately after the
existing split-horizon and hop-count checks:

```
for peer in self.connected_peers() {
    if peer.addr == sender_addr { continue; }                  // origin loopback
    if hop_count >= MAX_CAPABILITY_HOPS { return; }            // hop cap
    if router.next_hop(ann.node_id) == peer.addr { continue; } // split horizon
    if !peer_in_scope(peer, &ann.scope) { continue; }          // NEW: scope drop
    self.send_subprotocol(peer.addr, SUBPROTOCOL_CAPABILITY_ANN, bytes);
}
```

`peer_in_scope` is the function defined in §3. The
`capability_scope_dropped_forward` counter tags drops at this
point; the receiver-side `capability_scope_dropped_receive`
counter tags drops at terminal ingestion. Both are at trace
level; aggregation surfaces them in the existing
`CapabilityIndexStats`.

## Receiver behavior

`handle_capability_announcement` gates ingestion on scope per §5,
*before* the existing `version`-skip check (no point evaluating
version dedup on an announcement we'll drop anyway). The order is:

1. Size check (≤16KB).
2. Self-loopback check (`ann.node_id == self.node_id()`).
3. **Scope check (NEW).** Drop silently if out-of-scope.
4. Hop cap (`hop_count >= MAX`).
5. Signature check (when `require_signed_capabilities=true`).
6. Version-skip check.
7. `CapabilityIndex::index(ann)`.

Scope check before signature check is intentional: scope drops are
common, signature verification is expensive (ed25519 verify is
~50µs on commodity hardware), and a scope-violating announcement
is dropped regardless of signature validity.

## API surface

### Rust core

```rust
// adapter/net/behavior/capability.rs
impl CapabilityAnnouncement {
    pub fn new(node_id: u64, entity_id: EntityId, version: u64, caps: CapabilitySet) -> Self {
        // existing — defaults scope to Global
    }

    pub fn with_scope(mut self, scope: AnnouncementScope) -> Self {
        self.scope = scope;
        self
    }
}

// adapter/net/mesh.rs
impl MeshNode {
    pub fn announce_capabilities_scoped(
        &self,
        caps: CapabilitySet,
        scope: AnnouncementScope,
    ) -> Result<(), MeshError> { /* ... */ }

    // existing API stays — defaults to Global so callers that don't care don't break.
    pub fn announce_capabilities(&self, caps: CapabilitySet) -> Result<(), MeshError> {
        self.announce_capabilities_scoped(caps, AnnouncementScope::Global)
    }
}
```

### Rust SDK

`net-sdk` re-exports `AnnouncementScope` and adds
`MeshNode::announce_capabilities_scoped` 1:1. The default
`announce_capabilities` is a thin wrapper that passes `Global`.

### NAPI (Node.js binding)

```typescript
// index.d.ts
export type AnnouncementScopeJs =
    | { kind: 'global' }
    | { kind: 'subnetLocal' }
    | { kind: 'audience'; entities: Buffer[] };

export class MeshNode {
    announceCapabilities(caps: CapabilitySetJs, scope?: AnnouncementScopeJs): Promise<void>;
}
```

POJO discriminator on `kind` matches existing NAPI conventions
(see `RecoveryAction` in `bindings/node`). Unknown kinds error
synchronously at the FFI boundary, not asynchronously inside the
mesh.

### PyO3 (Python binding)

```python
class AnnouncementScope:
    @staticmethod
    def global_() -> "AnnouncementScope": ...
    @staticmethod
    def subnet_local() -> "AnnouncementScope": ...
    @staticmethod
    def audience(entities: list[bytes]) -> "AnnouncementScope": ...

mesh.announce_capabilities(caps, scope=AnnouncementScope.subnet_local())
```

`global_` not `global` because `global` is a Python keyword.

### Go (cgo binding)

```go
type AnnouncementScopeKind int
const (
    ScopeGlobal AnnouncementScopeKind = iota
    ScopeSubnetLocal
    ScopeAudience
)

type AnnouncementScope struct {
    Kind     AnnouncementScopeKind
    Entities [][]byte // populated only when Kind==ScopeAudience
}

func (m *MeshNode) AnnounceCapabilities(caps *CapabilitySet, scope *AnnouncementScope) error
```

Pin the audience byte slices via `runtime.Pinner` before crossing
the cgo boundary, mirroring the fix in
`bindings/go/net/net.go::IngestRawBatch`.

## Stages

| Stage | What | Days | Cumulative |
|-------|------|-----:|-----------:|
| S-1   | `AnnouncementScope` type + wire format + signed-payload coverage + serde compat tests | 2 | 2 |
| S-2   | Forwarder enforcement (`peer_in_scope` + counter wiring + split-horizon ordering) | 2 | 4 |
| S-3   | Receiver enforcement (`handle_capability_announcement` gate + counter wiring) | 1 | 5 |
| S-4   | Origin-side validation (`MAX_AUDIENCE_ENTITIES`, `SubnetLocal`-on-untagged-node downgrade) | 1 | 6 |
| S-5   | Rust SDK `announce_capabilities_scoped` + integration tests | 1 | 7 |
| S-6   | NAPI POJO + TS types + vitest coverage | 1.5 | 8.5 |
| S-7   | PyO3 surface + pytest coverage | 1 | 9.5 |
| S-8   | Go binding (with `runtime.Pinner` for audience entities) + `go test` coverage | 1 | 10.5 |
| S-9   | Docs sweep — README mention, plan finalization, MULTIHOP_CAPABILITY_PLAN follow-up checkbox | 0.5 | 11 |

Total estimate: **~11 engineer-days**, single contributor, no
parallelism.

## Test plan

### Unit — `capability.rs`

- `scope_global_serializes_identically_to_pre_stage` — serde
  round-trip with `Scope::Global` produces byte-identical output to
  pre-Stage `to_bytes()` (signature compatibility).
- `scope_audience_caps_at_max_entities` — origin API rejects
  `Audience(vec_of_33_entities)`; receiver deserialization rejects
  the same payload on the wire.
- `scope_subnet_local_on_untagged_origin_logs_and_downgrades` —
  warn-level log + emitted scope is `Global` not `SubnetLocal`.
- `signed_payload_includes_scope` — tampering scope from
  `Audience([X])` to `Global` invalidates the signature.

### Unit — `mesh.rs` (forwarder + receiver)

- `forwarder_drops_subnet_local_for_other_subnet_peer` — direct
  unit test of `peer_in_scope` with two contrived `CapabilitySet`s
  that resolve to different `SubnetId`s.
- `forwarder_forwards_subnet_local_when_peer_caps_unknown` — §3
  bootstrapping rule.
- `forwarder_drops_audience_for_unlisted_peer` — peer's
  `EntityId` not in the audience.
- `forwarder_drops_audience_when_peer_entity_unknown` — strict
  default per §4.
- `receiver_drops_subnet_local_for_self_outside_subnet`.
- `receiver_drops_audience_for_self_not_in_list`.
- `receiver_indexes_global_unconditionally` — sanity.

### Integration — `tests/capability_scope.rs` (new file)

- `three_node_subnet_local_announce` — A and B in
  `region:eu-west`, C in `region:us-east`. A announces
  `SubnetLocal`. After convergence: B's index has A, C's index does
  not.
- `three_node_audience_announce` — A names B in audience. After
  convergence: B's index has A, C's index does not, and C's
  forwarder (if it sat between A and B) successfully forwarded the
  announcement to B without indexing it locally.
- `scope_changes_evict_via_ttl` — A announces `Global`, B and C
  index. A re-announces `SubnetLocal` with a higher version. Within
  `ttl_secs`, C's entry for A expires and is not re-installed; B's
  re-installs.
- `scope_versioning_does_not_skip_when_scope_widens` — A
  announces `SubnetLocal` v=10, then `Global` v=11. C accepts v=11
  even though it dropped v=10 (no version-skip on a previously-
  unseen-by-this-node origin).

### TS / Py / Go SDK mirrors

Each binding suite gets one happy-path test (announce with
non-Global scope → query from in-scope node sees it, query from
out-of-scope node doesn't) plus the audience-cap rejection test.
Mirror the names from the Rust integration suite so a reviewer
can grep across surfaces.

## Risks

- **Subnet identity drift.** `SubnetPolicy::assign` is
  configuration-derived (`SubnetRule`s in node config). If two
  nodes disagree on the rule set, they'll compute different
  `SubnetId`s for the same caps and `SubnetLocal` will partition
  the mesh in unintuitive ways. Mitigation: document that
  `SubnetPolicy` MUST be uniform across the mesh, same as channel
  visibility today. Long-term: gossip the policy alongside
  capability announcements (out of scope here).
- **Audience explosion.** A naive caller building
  `Audience(known_peers)` with `known_peers.len() = 1000` tries to
  shove 32KB of `EntityId`s into a 16KB-capped announcement. The
  origin-side `MAX_AUDIENCE_ENTITIES = 32` cap is the hard stop;
  any larger use case is a `SubnetLocal` mis-fit and the API
  rejects. Mitigation: clear error message at the SDK layer
  pointing the caller at `SubnetLocal`.
- **Forwarder verify cost regression.** Per §6, scope is
  signature-covered. A forwarder that today does *not* verify
  signatures (it just re-broadcasts) will continue to not verify
  — scope enforcement is independent of signature verification.
  Sneaky scope downgrade by a malicious forwarder *is* possible
  if the forwarder strips `scope` and rewrites the signature
  (which it can't, lacking the origin's key). Net: no new
  attack surface; existing signature trust model carries through.
- **`Audience` exposes recipient identities.** An eavesdropper
  observing a `SUBPROTOCOL_CAPABILITY_ANN` packet sees the audience
  list in plaintext (the announcement is encrypted on the wire by
  the existing transport, but a session peer that's *not* in the
  audience still sees the bytes flow through them as a forwarder).
  This is a privacy regression for callers who treat the audience
  list itself as sensitive. Documented; mitigation is "use
  `SubnetLocal` if the membership shape is sensitive."

## Files touched

| File | Change |
|------|--------|
| `src/adapter/net/behavior/capability.rs` | `AnnouncementScope` type, `scope` field, signed-payload coverage, `is_global` helper, origin-side validation. |
| `src/adapter/net/mesh.rs` | `peer_in_scope`, forwarder gate, receiver gate, `peer_entity_ids` reuse, scope-drop counters. |
| `src/adapter/net/subnet/assignment.rs` | (read-only) reused via `SubnetPolicy::assign`. |
| `src/adapter/net/identity/entity.rs` | (read-only) `EntityId` as the audience element. |
| `sdk/src/mesh.rs` | `announce_capabilities_scoped` re-export. |
| `bindings/node/src/lib.rs` | NAPI POJO + `announceCapabilities(scope)` plumbing. |
| `bindings/python/src/lib.rs` (or new module) | `AnnouncementScope` PyO3 class + `announce_capabilities` kwarg. |
| `bindings/go/net/net.go` | `AnnouncementScope` Go struct, `runtime.Pinner` for audience. |
| `tests/capability_scope.rs` | New integration suite. |
| `bindings/node/test/capability_scope.test.ts` | TS mirror. |
| `bindings/python/tests/test_capability_scope.py` | Py mirror. |
| `bindings/go/net/capability_scope_test.go` | Go mirror. |
| `docs/MULTIHOP_CAPABILITY_PLAN.md` | Tick the deferred-scope follow-up. |

## Exit criteria

- `tests/capability_scope.rs::three_node_subnet_local_announce`
  passes consistently.
- `tests/capability_scope.rs::three_node_audience_announce` passes
  consistently.
- `tests/capability_scope.rs::scope_changes_evict_via_ttl` passes
  consistently.
- The on-the-wire compat test
  (`scope_global_serializes_identically_to_pre_stage`) passes — a
  pre-Stage node verifies a post-Stage node's `Global`-scoped
  announcement.
- All four binding suites have at least the happy-path scoped
  announce test and the audience-cap rejection test.
- `MULTIHOP_CAPABILITY_PLAN.md`'s "Subnet scope — deferred"
  paragraph is updated to point at this plan as shipped.

## Follow-ups (not in this plan)

- **Channel-scoped announcements.** `AnnouncementScope::Channel(channel_hash)`
  for "this announcement is meaningful only in the context of channel
  C." Requires deciding how query-time enforcement composes with the
  existing receive-time enforcement, and whether a single
  announcement can carry multiple channel scopes.
- **Filter-scoped announcements.** `AnnouncementScope::Filter(CapabilityFilter)`
  for "visible only to peers whose own caps match F." Solves the
  GPU-only-to-GPU-consumers use case; needs the receiver to have
  *its own* announcement settled first to evaluate the filter, which
  is a bootstrapping wrinkle that v1 deliberately avoids.
- **Subnet policy gossip.** The risk noted above ("subnet identity
  drift") would be solved by making `SubnetPolicy` itself a
  capability-like artifact that propagates through the mesh.
  Substantial — separate plan.
- **Per-recipient encryption of `Audience` lists.** If audience
  membership turns out to be sensitive in practice, encrypt the list
  to a key derived from the announcement's signed payload. Out of
  scope; flag if a user reports the privacy regression in §Risks
  above.
