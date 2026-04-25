# Scoped capability announcements

## Context

Today every `CapabilityAnnouncement` is permissive-global: the
origin's `CapabilitySet` (hardware, software, models, tools, tags,
limits) fans out to every directly-connected peer, then forwards
hop-by-hop up to `MAX_CAPABILITY_HOPS = 16`
(`behavior/capability.rs:787`). There is no way for the origin to
say "this announcement is for my subnet only," even when the
mesh already understands subnet visibility for *channels*
(`channel/config.rs:14` — `Visibility::SubnetLocal /
ParentVisible / Exported / Global`) and the
`SubnetGateway::should_forward` machinery
(`subnet/gateway.rs:103`) is ready to enforce it on the data
path.

The gap matters in three concrete scenarios:

1. **Private-tooling leakage.** A node in subnet `[3,7,2]` runs an
   internal `tool:billing-export` that should never be discoverable
   from siblings or unrelated subnets — but its full `CapabilitySet`
   (with that tool advertised) is gossiped to every peer, and any
   `find_peers(filter)` call from anywhere in the mesh returns it.
2. **Model fingerprinting at the perimeter.** A high-value model
   (`model_id = "internal-finetuned-llm-3"`) advertises through a
   permissive-global announcement; downstream nodes outside the
   parent subnet see it via multi-hop forwarding even though they
   could never get a session-auth token to use it.
3. **Cross-tenant `find_peers` polluting score-and-pick.** The SDK
   `find_best(req)` happily returns a winner in the wrong subnet;
   the publish path then drops the resulting traffic at the gateway,
   wasting the work and surprising the caller.

`SDK_SECURITY_SURFACE_PLAN.md:801` and
`MULTIHOP_CAPABILITY_PLAN.md` both list `AnnouncementScope` as an
explicit *deferred* non-goal — "v1 is permissive-global." This
plan makes it the v2.

## Design invariants

Non-negotiables, carried from the multi-hop work:

1. **Signed by the origin, never re-signed in transit.** Forwarders
   copy the signed payload byte-for-byte. The new scope field is
   inside the signed envelope so a forwarder can't relax it.
2. **Bounded CPU / bandwidth.** Scope filtering must be a *prune*
   of the existing fan-out — never a fan-in (no new subprotocol,
   no second packet per peer).
3. **No new subprotocol id.** `SUBPROTOCOL_CAPABILITY_ANN = 0x0C00`
   stays. The wire change is one additive field on
   `CapabilityAnnouncement`, gated by `#[serde(default,
   skip_serializing_if = "is_scope_global")]` so old-format
   announcements (no field) deserialize as `Global` and continue
   to round-trip with byte-identical signatures during a rolling
   upgrade — same trick `hop_count` and `reflex_addr` already use
   (`capability.rs:746, 772`).
4. **Late joiners still converge inside their scope.** Session-open
   re-push at `mesh.rs:2802` already pushes our latest local
   announcement to a fresh peer — that path applies the same scope
   filter, so a peer in a sibling subnet that opens a direct
   session to us gets a *no-op* on the per-session re-push when
   our local announcement is `SubnetLocal`. (More on the exit
   criteria below.)
5. **Permissive default.** A `CapabilityAnnouncement` constructed
   the old way (no scope set) is `Global`. No silent narrowing.

## Scope (the meta one — what's in / out of *this* plan)

**In scope:**

- New `AnnouncementScope` enum on `CapabilityAnnouncement`,
  mirroring channel `Visibility`: `Global` /
  `ParentVisible` / `Exported(Vec<SubnetId>)` / `SubnetLocal`.
- Origin-side fan-out filter on `MeshNode::announce_capabilities`:
  prune the per-peer broadcast list by checking *the origin's
  scope* against *each peer's known subnet*.
- Forwarder-side fan-out filter in `forward_capability_announcement`
  (`mesh.rs:3887`): same check, but using *the origin's scope* +
  *the origin's subnet derived from the announcement's own
  capabilities* + *the candidate forward target's subnet*. (Origin
  subnet is not on the announcement today; see "Origin subnet
  encoding" below.)
- Gateway integration: `SubnetGateway::should_forward` already
  exists for the *channel* path; we add a sibling
  `should_forward_announcement` that takes scope + origin subnet
  + dest subnet and returns `ForwardDecision`. Reuses the
  existing `DropReason` variants.
- `MeshNodeConfig::with_announcement_scope(AnnouncementScope)`
  builder knob for the local node's scope. Default = `Global`.
- Per-call override on `announce_capabilities`:
  `announce_capabilities_with(caps, scope)` so callers can publish
  a more-restricted announcement than the node-wide default
  without rebuilding the node.
- SDK + NAPI + Python surface for the new builder method and
  the per-call override.
- Five integration tests (see "Tests" below).

**Out of scope:**

- **Per-tag scope.** A scope per `tag` (or per `model` / per `tool`)
  inside one announcement would let a node simultaneously advertise
  `tool:billing-export` as `SubnetLocal` and `model:llama-3.1-70b`
  as `Global` in a single packet. Real, but adds index complexity
  (multiple effective `CapabilitySet` views per origin) and a
  partial-trust verification model. Defer; v2 ships with one scope
  per announcement.
- **Receiver-side scope filtering.** The origin and the forwarders
  do all enforcement. A *receiver* deep inside another subnet that
  somehow gets a `SubnetLocal` announcement (because a gateway is
  misconfigured or a peer is malicious) does not also re-validate
  scope on indexing — that would be a useful defense in depth, but
  it duplicates the path-cut and the only realistic attacker is
  the gateway operator, who has bigger levers. Document as a
  follow-up.
- **Dynamic scope changes via `CapabilityDiff`.** Today
  `CapabilityDiff` only diffs the `CapabilitySet` content, not
  the wrapper `CapabilityAnnouncement` fields. Scope changes
  arrive via a new full announcement (different `version`).
  Adding a `DiffOp::SetScope` is straightforward but mostly cosmetic
  — defer.
- **Scope-aware `CapabilityIndex::query`.** Today
  `CapabilityIndex` stores whatever announcements arrive at a node
  and serves them all to local `find_peers` callers. After
  filtering at the path level there is no useful "scope" for the
  receiver to filter on — by construction, only announcements
  whose origin allowed our subnet to see them reached the index
  in the first place. The index field stays a u64 → caps map.
- **Capability `Exported` export tables** managed via SDK. The
  `Exported(Vec<SubnetId>)` variant carries its own target list
  inside the announcement, so no separate export-table state is
  required (unlike channels). The SDK surface accepts a
  `Vec<SubnetId>` directly; if a node operator wants to manage a
  shared export table across many announcements they can keep their
  own `Vec` and pass it in.

## Design

### Wire format

Add one field at the end of `CapabilityAnnouncement`:

```rust
pub struct CapabilityAnnouncement {
    // … existing fields …
    /// Origin's intent for how widely this announcement should
    /// propagate. `Global` (default) keeps the pre-v2 permissive
    /// behavior. Inside the signed envelope, so a forwarder can't
    /// relax it from `SubnetLocal` to `Global` without invalidating
    /// the signature.
    #[serde(default, skip_serializing_if = "is_scope_global")]
    pub scope: AnnouncementScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnnouncementScope {
    #[default]
    Global,
    ParentVisible,
    Exported(Vec<SubnetId>),
    SubnetLocal,
}

fn is_scope_global(s: &AnnouncementScope) -> bool {
    matches!(s, AnnouncementScope::Global)
}
```

`SubnetId` is `Copy` and already `Serialize + Deserialize`
(`subnet/id.rs`); `Exported(Vec<SubnetId>)` serializes as
`{"Exported": […]}` in the JSON-on-wire form. Total wire-size
overhead for the common-case `Global` path: zero bytes (skipped).
For a typical `Exported` with 2–3 targets: ~40 bytes.

`hop_count` continues to sit *outside* the signed envelope so
forwarders can bump it; `scope` sits *inside*. `signed_payload()`
(`capability.rs:902`) zeros `hop_count` before serializing — no
change needed for `scope`, it just serializes through.

### Origin subnet encoding

Forwarder-side filtering needs to know the origin's subnet to
evaluate `ParentVisible` ("dest is ancestor of source") and
`SubnetLocal` ("source == dest"). Today the announcement does NOT
carry the origin's subnet directly — the receiver derives it via
`local_subnet_policy.assign(&ann.capabilities)`
(`mesh.rs:3829`).

Two options:

1. **Re-derive on every forward.** Each forwarder applies
   *its own* `local_subnet_policy` to `ann.capabilities` and
   treats that as the origin's subnet. Cheap, no wire change. But
   it assumes every node in the mesh runs the *same* subnet
   policy — which is the implicit invariant of Stage D today
   (`SDK_SECURITY_SURFACE_PLAN.md:436`). A node with a divergent
   policy could shift the effective scope on the forwarding path.
2. **Origin attaches its own subnet.** Add `origin_subnet:
   Option<SubnetId>` to the announcement (signed). The origin
   writes `Some(self.local_subnet)` when scope ≠ `Global`, `None`
   otherwise. Forwarders trust the origin's claim because it's
   covered by the signature.

Pick **(2).** It is more robust under heterogeneous-policy meshes,
and the wire cost is one optional 4-byte (or empty-when-`None`)
field. The `Global` fast path serializes nothing and stays byte-
identical to v1.

```rust
/// Origin's claimed subnet, signed alongside `scope`. `None` for
/// `Global` announcements (no enforcement needed) and for legacy
/// announcements (no scope field at all).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub origin_subnet: Option<SubnetId>,
```

Origin-subnet validation on receipt: if `scope != Global` but
`origin_subnet` is `None`, drop the announcement (`tracing::warn!`).
This is a soft invariant — a misbehaving origin could omit the
field deliberately, but doing so removes its own scope enforcement
and doesn't grant it any extra visibility, so it's a self-DoS, not
a privilege escalation.

### Origin-side fan-out filter

Today's `announce_capabilities` (`mesh.rs:5076`):

```rust
let peer_addrs: Vec<SocketAddr> = self.peers.iter().map(|e| e.value().addr).collect();
for addr in peer_addrs {
    self.send_subprotocol(addr, SUBPROTOCOL_CAPABILITY_ANN, &bytes).await?;
}
```

Becomes:

```rust
for entry in self.peers.iter() {
    let peer = entry.value();
    let peer_subnet = self
        .peer_subnets
        .get(&peer.node_id)
        .map(|e| *e.value())
        .unwrap_or(SubnetId::GLOBAL);
    if !announcement_visible(self.local_subnet, peer_subnet, &ann.scope) {
        continue;
    }
    let _ = self.send_subprotocol(peer.addr, SUBPROTOCOL_CAPABILITY_ANN, &bytes).await;
}
```

Where `announcement_visible(source, dest, scope)` is a free
function that mirrors `subnet_visible` (`mesh.rs:4489`) for the
`AnnouncementScope` enum. Same shape:

| scope            | rule                                                                          |
| ---------------- | ----------------------------------------------------------------------------- |
| `Global`         | always true                                                                   |
| `SubnetLocal`    | `source == dest`                                                              |
| `ParentVisible`  | `source == dest \|\| dest.is_ancestor_of(source) \|\| source.is_ancestor_of(dest)` |
| `Exported(targets)` | `targets.iter().any(\|t\| t == dest \|\| t.is_ancestor_of(dest))`             |

Identical predicate logic to `SubnetGateway::should_forward`
(`gateway.rs:130`) so the channel path and the announcement path
drop on the same conditions.

**Edge case: peer subnet unknown.** A peer we've just connected to
hasn't announced yet — `peer_subnets` returns `None`, defaults to
`SubnetId::GLOBAL`. A `SubnetLocal` announcement to a `GLOBAL`
peer is `false` (different subnets), so we skip the send. *Result:*
the new peer doesn't get our `SubnetLocal` caps until they announce
themselves and we learn their subnet. The next session-open
re-push (or our next periodic announce) re-evaluates and pushes
through if the subnet matches. This is a feature: scoped caps stay
private until the receiver proves their subnet.

### Forwarder-side fan-out filter

`forward_capability_announcement` (`mesh.rs:3887`) currently fans
out to every peer except the sender and the split-horizon-excluded
next-hop-to-origin. Add the same `announcement_visible` check, but
using `ann.origin_subnet` (with the `None` → `GLOBAL` fallback)
as `source`:

```rust
let origin_subnet = ann.origin_subnet.unwrap_or(SubnetId::GLOBAL);
for entry in peers.iter() {
    let peer = entry.value();
    if peer.node_id == sender_node_id { continue; }
    if Some(peer.addr) == next_hop_addr { continue; }
    if partition_filter.contains(&peer.addr) { continue; }
    let peer_subnet = peer_subnets.get(&peer.node_id)
        .map(|e| *e.value())
        .unwrap_or(SubnetId::GLOBAL);
    if !announcement_visible(origin_subnet, peer_subnet, &ann.scope) {
        continue;
    }
    // … existing build_subprotocol + send_to …
}
```

`peer_subnets` needs to be cloned into the spawned forwarding task
the same way `partition_filter` and `router` already are
(`mesh.rs:3893`).

### Receiver-side handling

`handle_capability_announcement` (`mesh.rs:3682`) does NOT need
to filter on scope. Any announcement that reached us already
passed the path-level filter at every hop, so by construction we
are an allowed destination. The only receive-time invariants we
add:

1. **Reject `scope != Global` with `origin_subnet == None`** —
   defensive guard against malformed announcements. Drop with
   `tracing::trace!` log line, same as the existing decode-failure
   branch (`mesh.rs:3683`).
2. **Verify origin_subnet vs derived subnet** when both are
   present. If `local_subnet_policy.is_some()` and
   `policy.assign(&ann.capabilities) != ann.origin_subnet`, log
   a warning. Don't drop — the local policy may legitimately
   diverge from the origin's. This is an observability hook for
   debugging policy drift, not an enforcement gate.

Forwarding (`mesh.rs:3867`) inherits the origin-side rule: a
forwarder sees `ann.scope` directly and applies the same path-cut
when re-broadcasting.

### Gateway integration

For nodes that *are* gateways (sit at subnet boundaries with
multiple `peer_subnets` entries spanning subnets), the existing
`SubnetGateway::should_forward` handles channels but not the
capability-announcement subprotocol. Add:

```rust
impl SubnetGateway {
    pub fn should_forward_announcement(
        &self,
        origin_subnet: SubnetId,
        dest_subnet: SubnetId,
        scope: &AnnouncementScope,
        hop_ttl: u8,
        hop_count: u8,
    ) -> ForwardDecision { … }
}
```

Same `DropReason` variants (`SubnetLocal`, `NotAncestor`,
`NotExported`, `TtlExpired`). The gateway is currently consulted
only on the channel path; capability announcements use
subprotocol routing through the regular `MeshNode` peers DashMap.
For v2, mesh-internal forwarding (above) is sufficient; a gateway
node carrying capability announcements through a subprotocol
session uses the same per-peer fan-out filter as a non-gateway
node. The new `should_forward_announcement` is a pure helper
function that gateway implementations and tests can call directly.

### Builder + per-call override

```rust
// MeshNodeConfig
pub fn with_announcement_scope(mut self, scope: AnnouncementScope) -> Self {
    self.announcement_scope = scope;
    self
}

// MeshNode
pub async fn announce_capabilities(&self, caps: CapabilitySet) -> Result<()> {
    self.announce_capabilities_with(caps, self.config.announcement_scope.clone()).await
}

pub async fn announce_capabilities_with(
    &self,
    caps: CapabilitySet,
    scope: AnnouncementScope,
) -> Result<()> {
    // … existing body, but assigns `ann.scope = scope` and
    // `ann.origin_subnet = (scope != Global).then_some(self.local_subnet)`
    // before signing.
}
```

The default-scoped variant calls into the override variant — the
existing call sites in tests and SDK pass through unchanged with
scope = `Global`, preserving permissive-global v1 behavior for
nodes that don't opt in.

## SDK surface

### Rust SDK (`sdk/src/capabilities.rs` + `sdk/src/mesh.rs`)

- Re-export `AnnouncementScope` from `net::adapter::net::behavior::capability`.
- `MeshBuilder::announcement_scope(scope: AnnouncementScope) -> Self`.
- `Mesh::announce_capabilities_scoped(&self, caps: CapabilitySet, scope: AnnouncementScope) -> impl Future<…>`.

### Node.js (`bindings/node/src/capabilities.rs` + `sdk-ts/src/capabilities.ts`)

- TS `type AnnouncementScope = 'global' | 'parentVisible' | 'subnetLocal' | { exported: SubnetId[] }`.
- NAPI converts the union to the Rust enum (string compare on
  the discriminator field).
- `MeshConfig.announcementScope?: AnnouncementScope` plumbed through
  `with_announcement_scope`.
- `Mesh.announceCapabilitiesScoped(caps, scope)` per-call.

### Python (`bindings/python/src/lib.rs` + `sdk-py/`)

- Mirror Node's surface. `announcement_scope` config key, taking
  either a string or a tuple `("Exported", [SubnetId, ...])`.
- `mesh.announce_capabilities_scoped(caps, scope)`.

## Tests

Add `tests/capability_scope.rs` (sibling to `tests/capability_multihop.rs`):

1. **`subnet_local_does_not_cross_boundary`** — three nodes, A and
   B in subnet `[1,1]`, C in `[1,2]`. A announces `SubnetLocal`. B
   indexes A's caps; C does not. Verify via `find_peers` from each.
2. **`parent_visible_reaches_ancestor_not_sibling`** — A in
   `[1,1,1]`, parent gateway in `[1,1]`, sibling C in `[1,2]`. A
   announces `ParentVisible`. Parent indexes A's caps; sibling does
   not.
3. **`exported_targets_are_explicit`** — A in `[1,1]`, B in
   `[2,1]`, C in `[3,1]`. A announces `Exported(vec![[2,1]])`. B
   indexes; C does not.
4. **`scope_signature_invariant_across_forward`** — A announces
   `SubnetLocal` to B in same subnet; B forwards to D (also same
   subnet) with `hop_count = 1`. D verifies the signature
   end-to-end. Same shape as
   `tests/capability_multihop.rs::wire_format_signature_survives_hop_bump`.
5. **`malformed_scope_without_origin_subnet_drops`** — craft an
   announcement (via test-only `unsafe` constructor) with `scope =
   SubnetLocal` and `origin_subnet = None`. Receiver drops; index
   does not contain the origin.

Plus 3 unit tests in `behavior/capability.rs`:

6. **`scope_default_is_global`** — `CapabilityAnnouncement::new`
   produces `scope == Global, origin_subnet == None`.
7. **`scope_serializes_omitted_when_global`** — JSON round-trip of
   a `Global` announcement is byte-identical to a v1 announcement
   without the field. Locks in rolling-upgrade compat.
8. **`scope_in_signed_payload`** — sign with `scope = Global`,
   mutate to `scope = SubnetLocal` post-signing, verify fails.
   Sign with `scope = SubnetLocal`, verify passes. (`signed_payload`
   includes `scope`; mutating it should invalidate.)

Plus 3 unit tests in `subnet/gateway.rs`:

9. **`gateway_announcement_subnet_local_drops`** — direct call to
   `should_forward_announcement` returns
   `Drop(DropReason::SubnetLocal)` for cross-subnet pair.
10. **`gateway_announcement_parent_visible_to_ancestor_forwards`**.
11. **`gateway_announcement_exported_targets_match`**.

## Implementation steps

Each step is independently reviewable; full sequence ~3 days for
Rust core + 2 days for SDK surface.

### Step 1 — Wire format + sign/verify

`src/adapter/net/behavior/capability.rs` (~80 lines):

- Add `AnnouncementScope` enum, `Default for AnnouncementScope`,
  `is_scope_global` predicate.
- Add `scope` and `origin_subnet` fields to
  `CapabilityAnnouncement` with the `#[serde(default,
  skip_serializing_if = …)]` attributes.
- Update `CapabilityAnnouncement::new` to default both to
  `Global` / `None`.
- Add `with_scope(scope)` builder.
- Verify `signed_payload()` covers `scope` + `origin_subnet`
  (no code change needed — it's already
  `to_bytes()` of the full struct minus signature + hop_count).
- Tests 6, 7, 8.

### Step 2 — Origin-side fan-out filter

`src/adapter/net/mesh.rs` (~50 lines):

- Add `announcement_scope: AnnouncementScope` to `MeshNodeConfig`
  and `with_announcement_scope` builder.
- Add `announcement_visible(source, dest, scope)` free function
  next to the existing `subnet_visible`.
- Modify `announce_capabilities` to take an optional scope
  override, build the announcement with `scope` +
  `origin_subnet`, and apply the per-peer filter on fan-out.
- Add `announce_capabilities_with(caps, scope)` public entry
  point.
- Tests 1, 3.

### Step 3 — Forwarder-side filter

`src/adapter/net/mesh.rs` (~30 lines):

- Modify `forward_capability_announcement` signature to take
  `peer_subnets` and the parsed `ann` (or the scope + origin
  subnet pair).
- Apply `announcement_visible` per candidate forward target.
- Add receiver-side validation guards in
  `handle_capability_announcement`: reject
  `scope != Global && origin_subnet == None`; warn on policy
  drift.
- Tests 2, 4, 5.

### Step 4 — Gateway helper

`src/adapter/net/subnet/gateway.rs` (~40 lines):

- Add `should_forward_announcement` mirroring `should_forward`.
- Tests 9, 10, 11.

### Step 5 — SDK surface

`sdk/src/capabilities.rs`, `sdk/src/mesh.rs` (~40 lines).
`bindings/node/src/capabilities.rs`,
`sdk-ts/src/capabilities.ts`, `sdk-ts/src/mesh.ts` (~80 lines).
`bindings/python/src/lib.rs`, `sdk-py/` (~60 lines).

- Re-export `AnnouncementScope` and the builder method.
- Add `announceCapabilitiesScoped` / `announce_capabilities_scoped`
  per language.
- One smoke test per SDK that round-trips a `SubnetLocal`
  announcement through the wire and asserts visibility on a
  same-subnet peer + invisibility on a cross-subnet peer.

### Step 6 — Documentation

- Add a paragraph to `CHANNELS.md` and `BEHAVIOR.md` explaining
  that announcement visibility now mirrors channel visibility.
- Update `CAPABILITY_BROADCAST_PLAN.md` and
  `MULTIHOP_CAPABILITY_PLAN.md` to remove `AnnouncementScope`
  from "Out of scope" / "non-goals" sections.
- Update `SDK_SECURITY_SURFACE_PLAN.md:801` (the explicit
  non-goal block) to reference this plan.

## Open questions

- **Rollout under heterogeneous deployments.** A v1 node that
  doesn't know about `scope` deserializes the field as
  `Global` (default) and re-broadcasts permissively. That means
  during a partial upgrade, a `SubnetLocal` announcement reaching a
  v1 forwarder is effectively widened to that forwarder's reach.
  Mitigation: ship v2 receivers first, then v2 origins. The
  in-cluster upgrade order is already documented for the multi-hop
  rollout — this plan piggybacks.
- **`peer_subnets` warm-up race.** First-contact peer hasn't
  announced → unknown subnet → `SubnetLocal` filter skips the
  send → the peer remains uninformed until they announce. The
  *channel* path has the same warm-up window
  (`mesh.rs:4331`); the symmetry is the right answer. Document
  in `CHANNELS.md`.
- **Should `Global` still carry `origin_subnet`?** Cleaner if it
  did (forwarders never have to fall back to GLOBAL), but it
  breaks the v1-byte-identical signing path. Leave as `None` for
  `Global`, accept the `unwrap_or(GLOBAL)` fallback.
