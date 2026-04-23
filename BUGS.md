# Bug findings

Verified against the tree at commit `c97c33f` (master). Line numbers are the
authoritative citation — quoted snippets may abbreviate.

---

## HIGH — reliable streams can silently lose the first packet

**File:** `net/crates/net/src/adapter/net/reliability.rs`
**Lines:** `268-273` (`on_receive`, the `seq == ack_seq + 1` branch)

Senders always start at `seq = 0` (`session.rs:832`: `tx_seq: AtomicU64::new(0)`),
so the very first on-wire packet in a reliable stream is always seq 0. The
receiver, however, treats the first received packet with `seq == 1` as
"next expected" and advances `ack_seq` to 1 without recording seq 0 as
missing:

```rust
if seq == self.ack_seq + 1 {
    self.ack_seq = seq;
    self.received_first = true;
    self.sack_bitmap >>= 1;
    ...
}
```

Consequences:

- `missing_bitmap()` stays zero, so `build_nack()` returns `None` — seq 0 is
  never NACK'd.
- The sender's timeout-driven retransmit of seq 0 is rejected by the
  `seq == 0 && received_first` branch (line 264-265) as a duplicate.
- Any cumulative ACK the receiver emits with `ack_seq >= 1` causes the sender
  to evict seq 0 from `pending` in `on_ack` (lines 201-207), so the sender
  stops retransmitting.

Net effect: whenever the very first packet of a reliable stream drops, its
payload is lost to the application — violating the reliability contract.

The regression test `test_regression_seq_zero_rejected_when_stream_starts_at_one`
(line 814) encodes this behavior as "expected," but its premise
("seq 0 never sent") contradicts the sender's own behavior — `next_tx_seq`
always starts at 0.

**Fix:** when the first received packet has `seq > 0`, mark `[0, seq)` as
missing in the SACK bitmap (or advance `ack_seq` only after seq 0 has been
observed) so a NACK goes out. Update the regression test to match.

---

## MEDIUM — integer overflow in token rate limiter

**File:** `net/crates/net/src/adapter/net/behavior/safety.rs`
**Lines:** `509`, `512` (`check_tokens`)

```rust
let current = self.global_tokens.load(Ordering::Relaxed);
let effective_limit = (limit as f64 * burst as f64) as u64;
if current + tokens > effective_limit {
    return Err(SafetyViolation::RateLimitExceeded {
        limit_type: RateLimitType::TokensPerMinute,
        current: current + tokens,
        limit: effective_limit,
    });
}
```

Both `current` (from `global_tokens`) and `tokens` (caller-supplied via
`req.estimated_tokens as u64`, line 1058) are `u64`. `current + tokens` has
no overflow guard:

- Debug builds: panic — turns the rate limiter into a DoS vector.
- Release builds: wraps — a caller able to push `global_tokens` near
  `u64::MAX` could bypass the check entirely.

Practical exploitability is bounded by how large `estimated_tokens` can get
and the `reset_interval` cadence; still a real arithmetic bug.

**Fix:**

```rust
let would_be = match current.checked_add(tokens) {
    Some(v) => v,
    None => return Err(SafetyViolation::RateLimitExceeded { … }),
};
if would_be > effective_limit { … }
```

---

## LOW — weighted-LC scorer collapses small weights

**File:** `net/crates/net/src/adapter/net/behavior/loadbalance.rs`
**Lines:** `811-813` (`select_weighted_least_connections`)

```rust
let score_a = a.connections.load(Ordering::Relaxed) as f64
    / a.effective_weight().max(1.0);
```

`.max(1.0)` was presumably meant as a divide-by-zero guard, but it also
collapses every weight in the open interval `(0, 1]` onto `1.0`. An endpoint
with `weight = 0.1` is scored identically to one with `weight = 1.0`, and
the weighted selection silently degrades into a plain least-connections
selection whenever operators tune weights below 1.

**Fix:** guard against zero only — e.g. `.max(f64::MIN_POSITIVE)`, or a small
configured epsilon — so fractional weights retain their relative ordering.

---

## LOW — weighted round-robin loses precision past 2⁵³

**File:** `net/crates/net/src/adapter/net/behavior/loadbalance.rs`
**Lines:** `765-766` (`select_weighted_round_robin`)

```rust
let counter = self.rr_counter.fetch_add(1, Ordering::Relaxed);
let target = counter as f64 % total_weight;
```

`counter` is `u64`, cast to `f64` before the modulo. Past roughly `2^53`
selections the cast drops the low bits and the rotation stalls on a small
set of indices, skewing distribution for long-running processes.

**Fix:** do the modulo in integer space — scale weights to an integer
representation, or periodically reset `rr_counter` — so rotation stays exact.

---

## LOW (theoretical) — replay window saturates at `u64::MAX`

**File:** `net/crates/net/src/adapter/net/crypto.rs`
**Lines:** `427-450` (`ReplayWindow::commit`)

```rust
let shift = (received - self.rx_counter).saturating_add(1);
self.shift_bitmap_up(shift);
self.rx_counter = received.saturating_add(1);
self.bitmap[0] |= 1u64;
true
```

Both `shift` and `rx_counter` use `saturating_add(1)`. Once `rx_counter`
saturates at `u64::MAX`, any subsequent `commit(u64::MAX)` call takes the
`received >= rx_counter` branch, re-shifts the bitmap by 1 and re-sets
bit 0 — returning `true` and re-accepting the same nonce. The function's
stated contract ("replay detected at commit time") is technically violated
at the ceiling.

**Practical reachability:** effectively zero — 2⁶⁴ packets per session is
far beyond any realistic lifetime. Worth mentioning only because the other
saturation paths in this file (`commit` / `shift_bitmap_up`) are otherwise
carefully written to avoid UB.

**Fix:** if `rx_counter == u64::MAX`, treat further commits as replay (return
`false`) or require a re-keying before continuing.

---

# Second-pass findings

Targeted sweeps through NAT traversal, capability index, FFI boundary,
stream-window accounting, circuit breakers, and wire codecs. The code is
generally careful — the bulk of what used to bite here is already fixed
and pinned by regression tests. These are the remaining items I can stand
behind.

## MEDIUM — `channel_hash == 0` is silently a wildcard grant

**File:** `net/crates/net/src/adapter/net/identity/token.rs`
**Lines:** `218-224` (`PermissionToken::authorizes`)

```rust
pub fn authorizes(&self, action: TokenScope, channel: u16) -> bool {
    if !self.scope.contains(action) { return false; }
    // channel_hash 0 = wildcard (all channels)
    self.channel_hash == 0 || self.channel_hash == channel
}
```

`channel_hash` is a 16-bit truncation of `xxh3_64(name)`
(`channel/name.rs:114-116`), so one channel name in ~65 536 hashes to zero.
A `PermissionToken` scoped to such a channel is indistinguishable on the
wire from one issued with "wildcard all channels" — the verifier grants
access to every channel. Channel namespaces are caller-controlled in a
mesh, so an attacker who can register names can brute-force a
hash-to-zero name cheaply (`xxh3_64` is non-cryptographic) and mint what
looks like a legitimately-scoped token but is actually universal.

The bloom / exact-name ACL on the publisher side (`channel/guard.rs`) is
the real backstop, but any code path that checks `PermissionToken::authorizes`
directly is bypassable.

**Fix:** carry wildcard as an explicit bit in `TokenScope` (or a
dedicated `wildcard: bool` field covered by the signature) and drop the
`channel_hash == 0` special case. Ban `channel_hash == 0` at issue time
so no hash-collision token can accidentally occupy the wildcard slot.

## LOW — empty inverted-index sets leak on revoke

**File:** `net/crates/net/src/adapter/net/behavior/capability.rs`
**Lines:** `1363-1396` (`CapabilityIndex::remove_from_indexes`)

```rust
if let Some(mut set) = self.by_tag.get_mut(tag) {
    set.remove(&node_id);
}
```

Each of `by_tag`, `by_model`, `by_tool`, `by_gpu_vendor`, `gpu_nodes`
removes `node_id` from the inner `HashSet` but never removes the
DashMap entry when the set empties. Over a long-lived deployment with
high churn of short-lived tags / model ids (every node version-bump
produces a new tag variant, and a peer rejoining under a fresh
`node_id` deposits a new entry in the old tag's set on GC), the outer
maps grow unboundedly even though the actual population is stable.
`gc()` at line 1535 only removes expired *nodes* — the stale tag keys
it leaves behind are never collected.

**Fix:** in each `set.remove(&node_id)` path, check `set.is_empty()` and
drop the outer entry too. DashMap supports this via
`remove_if(|_, set| set.is_empty())` after releasing the `get_mut`
guard — or just use `entry` with `Occupied.remove_entry` when emptied.

## LOW — `fp16_tflops_x10` silently saturates on realistic hardware

**File:** `net/crates/net/src/adapter/net/behavior/capability.rs`
**Line:** `113` (`GpuInfo::with_fp16_tflops`)

```rust
self.fp16_tflops_x10 = (tflops * 10.0) as u16;
```

`u16::MAX / 10 = 6553.5` TFLOPS. Individual GPUs are still comfortably
below that (H100 is ~4000 FP16 TFLOPS dense), but an aggregated
"total GPU throughput" view — the obvious callers of a capability
announcement — saturates at ~6.5 PFLOPS, which a mid-sized cluster
already exceeds. Saturation is silent (no error, no warning).

**Fix:** widen to `u32` (or drop the `_x10` encoding and just use `f32`
TFLOPS). The wire format already spends 2 bytes on this field; 4 bytes
is a negligible upgrade and removes the ceiling.

## INFO — rendezvous wire-format doc claims LE, code is BE

**File:** `net/crates/net/src/adapter/net/traversal/rendezvous.rs`
**Lines:** `194-202` (comment), `360-381` (encoders), `399-425` (decoder)

The docstring on `encode_keepalive` asserts that the migration to LE was
driven by "divergence from … every other wire field in the traversal
path." But `encode_punch_request` / `encode_punch_introduce` /
`encode_punch_ack` use bare `put_u64` / `put_u16` — which `bytes`
defines as **big-endian** — and the paired `decode` uses
`u64::from_be_bytes` / `u32::from_be_bytes` / `u16::from_be_bytes`. So
the rendezvous body is BE while the keepalive body and the Net header
are LE.

Not a functional bug (encode/decode round-trips), but the comment is
load-bearing: anyone reading it in the future will assume rendezvous
matches and write new code accordingly. Either unify on one endianness
(LE to match the rest of the crate) or correct the comment to say
"the keepalive should match the Net header, not the other traversal
primitives."

---

# Third-pass findings

Additional sweep through state/causal, identity/envelope, subprotocol codecs,
the subscriber roster, and the FFI redex path. Most of these areas are
carefully written; the items below are the ones I can defend.

## MEDIUM — subscriber add/remove race can silently lose a subscription

**File:** `net/crates/net/src/adapter/net/channel/roster.rs`
**Lines:** `42-56` (`add`), `59-82` (`remove`)

`add` builds and clones the inner `Arc<DashSet>` under the entry guard, then
releases the guard before calling `subs.insert(node_id)`:

```rust
let subs = self.subs.entry(channel.clone())
    .or_insert_with(|| Arc::new(DashSet::new()))
    .clone();                    // shard lock released here
let peers = self.by_peer.entry(node_id)...            // separate DashMap
let inserted = subs.insert(node_id);                  // ← Arc may already be orphaned
```

Between the two lines, a concurrent `remove(channel, other_node)` can land,
observe the still-empty inner set (line 70), and run
`self.subs.remove_if(channel, |_, v| v.is_empty())` (line 72), evicting the
outer entry. The `Arc<DashSet>` that `add` holds is now orphaned: the
insert at line 53 succeeds locally, but it is not reachable from
`members(channel)` — the roster has lost track of this subscription even
though the publisher's own `by_peer` index still thinks the peer is
subscribed.

The window is narrow (between the two entries) and requires a concurrent
`remove` on the same channel for some other member, but the outcome is
publishes silently skipping a subscriber.

**Fix:** keep the entry guard alive across the member insert, e.g.:

```rust
self.subs
    .entry(channel.clone())
    .or_insert_with(|| Arc::new(DashSet::new()))
    .insert(node_id);
```

or re-check `self.subs.get(channel)` after insert and retry if the Arc
is no longer the mapped one.

## LOW — ed25519 token verification uses lax `verify`

**File:** `net/crates/net/src/adapter/net/identity/entity.rs`
**Lines:** `58-62` (`EntityId::verify`), used by `token.rs:169-175`

```rust
pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), EntityError> {
    let vk = self.verifying_key()?;
    vk.verify(message, signature)
        .map_err(|_| EntityError::InvalidSignature)
}
```

`ed25519_dalek::VerifyingKey::verify` is the lax variant; it accepts any
point-compressed `S` in `[0, 2·L)`, so a valid signature `(R, S)` can be
malleated to `(R, S+L)` and will still verify. The envelope path already
uses `verify_strict` (`envelope.rs:275`), but `PermissionToken::verify`
flows through this helper.

A malleated token has different bytes but signs the same payload, so a
cache or dedup table keyed on the 159-byte wire form can be fooled into
treating one logical token as two. Functionally `nonce` is the
revocation key and is part of the signed payload, so revocation still
works; this is mostly a wire-hygiene concern.

**Fix:** switch `EntityId::verify` to `vk.verify_strict(...)` and keep
`verify` for anything that genuinely needs lax semantics (nothing in
this crate seems to).

## LOW — `write_causal_events` panics on > 4 GiB payloads

**File:** `net/crates/net/src/adapter/net/state/causal.rs`
**Lines:** `234-244` (`write_causal_events`)

```rust
let total_len = CAUSAL_LINK_SIZE + event.payload.len();
let total_len_u32 = u32::try_from(total_len).expect("causal event exceeds 4 GiB");
```

Any caller able to feed an oversized payload (e.g. via an FFI path that
forwards arbitrary `Bytes`) can crash the whole process. This is already
a limit the protocol imposes — the fix is to return an error, not to
unwrap, so a malformed producer doesn't take down the node.

The paired `read_causal_events` path is already defensive (it silently
drops malformed entries, line 287-294); the writer should be symmetric.

**Fix:** return a `Result` (`WriteError::PayloadTooLarge`) and let the
caller decide whether to split or reject.

## INFO — `ObservedHorizon::merge` overflows `logical_time` at u64::MAX

**File:** `net/crates/net/src/adapter/net/state/horizon.rs`
**Line:** `71`

```rust
self.logical_time = self.logical_time.max(other.logical_time) + 1;
```

Classical `+ 1` on two `u64::MAX` values panics in debug / wraps in
release. Unreachable in practice (2⁶⁴ merges), noted only because the
neighboring arithmetic (`observe`) has the same shape at line 32.

**Fix:** use `saturating_add(1)` for consistency with other monotonic
counters in the crate.

---

# Fourth-pass findings

Sweep through mesh.rs session lifecycle, SDK surface (compute/groups/stream),
SPSC ring buffer, consumer filter/merge, protocol AAD, and the
reflex/proxy/router forwarding path. One concrete mesh-level bug; the rest
of what I checked is solid.

## MEDIUM — failed peers are never evicted from the `peers` map

**File:** `net/crates/net/src/adapter/net/mesh.rs`
**Lines:** `1199-1218` (failure detector wiring), `3046-3098` (heartbeat loop)

The failure detector's `on_failure` callback cleans up several auxiliary
structures when a peer is declared dead:

```rust
.on_failure(move |node_id| {
    rp_failure.on_failure(node_id);
    let removed = roster_failure.remove_peer(node_id);
    ...
    peer_subnets_failure.remove(&node_id);
    peer_entity_ids_failure.remove(&node_id);
})
```

but it does **not** touch `self.peers` or `self.addr_to_node`. The heartbeat
loop at `mesh.rs:3046` evicts idle *streams* (`session.evict_idle_streams`,
line 3086) and stale routing entries (`routing_table().sweep_stale(max_route_age)`,
line 3071), but again leaves `peers` untouched. The only code path that
removes an entry from `self.peers` is the handshake send-failure fallback
at line 2471.

Consequences:

- `PeerInfo` entries (holding the full `NetSession` with keys, streams,
  and atomic counters) accumulate in `self.peers` for every peer that
  ever went silent without a handshake failure on our side.
- `send_to_peer(addr)` at line 3102+ looks the peer up via
  `addr_to_node` → `peers` and sends through the dead session. UDP
  `send_to` succeeds quietly (packets vanish), so the caller's send
  returns `Ok` until the application-layer timeout fires.
- A peer that went away and then reconnected from a **new** address
  gets a second entry in `addr_to_node` pointing at the fresh session,
  but the stale `addr_to_node[old_addr] → node_id` pointer remains,
  so any code that maps the old address back to a peer still finds it.

**Fix:** inside the `on_failure` closure, also

```rust
if let Some((_, peer_info)) = self.peers.remove(&node_id) {
    self.addr_to_node.remove(&peer_info.addr);
    self.peer_addrs.remove(&node_id);
}
```

and guard against the reconnect race by matching on `peer_info.addr == known_addr`
before removal (the same `remove_if` pattern already used at line 2471).

## LOW — empty `Filter::And` matches every event

**File:** `net/crates/net/src/consumer/filter.rs`
**Lines:** `89-99` (`Filter::matches`)

```rust
Self::And { filters } => filters.iter().all(|f| f.matches(event)),
```

Rust's `.all()` on an empty iterator returns `true`, so a `Filter::And {
filters: vec![] }` is logically "match everything." For externally-supplied
filters (SDK caller, JSON over the wire), this means a malformed or
pathological payload silently broadens the filter to unrestricted access
— not how AND semantics usually behave in query-engine APIs.

The mirror case (empty `Or` matches nothing via `.any()`) is closer to
what a caller expects, but both deserve explicit handling.

**Fix:** reject empty `filters` vectors at parse time, or treat empty
`And`/`Or` as an error at `matches` call-sites. Cheapest patch is a
validator in `Filter::from_json` that rejects empty child arrays.

## INFO — `EventStream::poll_next` wakes itself unnecessarily after creating a sleep

**File:** `net/crates/net/sdk/src/stream.rs`
**Lines:** `156-161`

```rust
this.current_interval = (this.current_interval * 2).min(this.opts.max_backoff);
this.sleep = Some(Box::pin(tokio::time::sleep(this.current_interval)));
cx.waker().wake_by_ref();
Poll::Pending
```

The `wake_by_ref()` immediately re-schedules the task, which then re-enters
`poll_next`, reaches line 123, polls the freshly-created sleep for the
first time, and only *then* parks. The extra round-trip through the
executor is pure overhead: the correct pattern is to poll the sleep
once in this same call and return whatever it returned. Not a bug, just
wasted cycles on every idle-poll backoff tick (every 1–100 ms by default).

**Fix:** drop the `wake_by_ref` and fall through to `Pin::new(&mut sleep).poll(cx)`
by just calling `poll_next` recursively, or inline the sleep poll.

---

# Fifth-pass findings

Sweep through `SnapshotReassembler`, `ShardMapper` scaling arithmetic,
`reconcile_entity`, `EntityLog::append`, proximity graph edge EWMA, and
the event bus scaling monitor. The bulk of this is defensively written;
two items worth flagging.

## LOW — `ShardMapper::scale_up` overflow-prone `u16` arithmetic

**File:** `net/crates/net/src/shard/mapper.rs`
**Lines:** `515`, `533`, `565`

```rust
if current + count > self.policy.max_shards { ... }
...
self.active_count.fetch_add(count, AtomicOrdering::Release);
```

`current`, `count`, and `max_shards` are all `u16`. When `current + count`
would exceed `u16::MAX` (65 535) the addition wraps, producing a small
number that's almost certainly ≤ `max_shards`, so both guards at
lines 515 and 533 silently pass. Execution continues to line 544 where
the `max_id` overflow check catches it via `checked_add`, so in practice
the mapper returns `AtMaxShards` at a different gate rather than running
unbounded. But `fetch_add(count)` at line 565 on `active_count: AtomicU16`
also wraps silently in the unlikely path that somehow reaches it past a
bug in the earlier gate.

Shard counts in the tens of thousands are unreachable in practice
(policy.max_shards is typically <256), so this is style/hygiene, not a
live exploit — but the `max_id` check should be the *only* overflow
gate, not a fallback.

**Fix:** replace both additions with `checked_add` and bubble up
`AtMaxShards` on overflow; use `checked_add` on `active_count` increment
or switch to `AtomicU32` if any caller legitimately uses >65k shards.

## LOW — reconcile accepts a remote chain with arbitrary starting sequence

**File:** `net/crates/net/src/adapter/net/contested/reconcile.rs`
**Lines:** `65-74` (entry), `199-212` (`verify_remote_chain`)

`reconcile_entity` validates that the remote chain's first event has the
expected `origin_hash` and that consecutive remote events link correctly
(`validate_chain_link` in a loop). It does **not** check that the remote
chain's first event's `parent_hash` chains back to anything in our local
log — in particular, there's no constraint that `their_events[0].sequence`
equals `split_seq + 1`.

So a remote peer can submit a well-formed chain with an arbitrary starting
sequence (e.g. 100 when `split_seq` is 10 and our local side has events
at 11, 12, 13). The divergence-detection loop at line 104 zips the two
slices index-by-index, so it compares our seq-11 event to their seq-100
event as if they were siblings, yielding a "conflict" at `diverge_seq = 11`
and a longest-chain / payload-hash tiebreak.

Since `reconcile_entity` returns only an outcome description (not
applied events), the worst case is a bogus `ForkRecord` being emitted
and potentially propagated — the actual log state isn't corrupted, and
any subsequent replay of the "winning" chain would fail
`EntityLog::append`'s chain validation. Still, a better contract would
reject the remote chain up front.

**Fix:** in `verify_remote_chain`, require `events[0].sequence == split_seq + 1`
(or, if genesis-at-split is allowed, `<= split_seq + 1`) and reject
otherwise. This closes the "arbitrary-start-sequence" loophole without
changing any honest-path behavior.

## Notes on what this pass covered without finding anything

- `SnapshotReassembler::feed` (orchestrator.rs:743-826): total_chunks and
  chunk_index bounds-checked, stale `seq_through` rejected, per-daemon
  pending state evicted on newer `seq_through`, single-chunk fast path.
  Same-`chunk_index` re-insert does overwrite, but only the authenticated
  AEAD source can inject into an in-flight reassembly, so the threat
  doesn't apply.
- `EntityLog::append` (state/log.rs:66-94): duplicate check has a
  `current_seq > 0` carve-out for the first genesis event, but the
  chain-linkage check at line 87 rejects subsequent seq-0 events via
  the `SequenceGap` path — no double-acceptance window.
- `ProximityGraph::on_pingwave_from` (behavior/proximity.rs:536-604):
  self-loop guard, dedup cache, split DashMap lock ordering
  (`nodes` → `edges`) — no deadlocks, no unbounded growth on the
  hot path.
- `MigrationState` phase machine (compute/migration.rs:162-233): every
  transition gates on current phase; `force_phase` is pub(crate) and
  documented. No skip-phase loopholes.
- `EventBus::run_scaling_monitor` and `shutdown` (bus.rs:200-267,
  514-543): handles are taken out of locks before `.await`, shutdown
  flag is checked on either side of every sleep, `Drop` sets the flag
  even when `shutdown()` is not awaited.

---

# Sixth-pass findings

Final sweep covering `redex/disk` crash recovery, `continuity/discontinuity`,
`contested/partition`, `compute/orchestrator` chunking, the mesh.rs direct/
routed dispatch discriminator, `behavior/api` path-parameter parsing,
`subnet/id`, and the `subprotocol/registry`. One cross-cutting hygiene
pattern, one packet-classification quirk, and a positive-coverage summary.

## LOW — `from_bytes` codecs accept trailing bytes instead of rejecting them

**Files / lines (selected):**

- `net/crates/net/src/adapter/net/continuity/discontinuity.rs:184-207` (`ForkRecord::from_bytes`)
- `net/crates/net/src/adapter/net/continuity/chain.rs:151-162` (`ContinuityProof::from_bytes`)
- `net/crates/net/src/adapter/net/state/causal.rs:82-93` (`CausalLink::from_bytes`)
- `net/crates/net/src/adapter/net/behavior/capability.rs:679`, `924`
- `net/crates/net/src/adapter/net/behavior/diff.rs:215`
- `net/crates/net/src/adapter/net/behavior/proximity.rs:172`
- `net/crates/net/src/adapter/net/swarm.rs:70, 226, 320`
- `net/crates/net/src/adapter/net/route.rs:138`
- `net/crates/net/src/adapter/net/subprotocol/negotiation.rs:73`
- `net/crates/net/src/adapter/net/protocol.rs:382, 531`

All of the above guard their `from_bytes` with

```rust
if data.len() < Self::WIRE_SIZE {
    return None;
}
```

and then slice the leading bytes. Input longer than `WIRE_SIZE` is
silently accepted and the trailer is ignored. Three codecs in the crate
already rejected the longer-input case explicitly —
`PermissionToken::from_bytes` (token.rs:323-326), `IdentityEnvelope::from_bytes`
(envelope.rs:352-355), and `StreamWindow::decode` (stream_window.rs:69-83),
with `token.rs` carrying a docstring explaining exactly why:

> Rejects buffers whose length is anything other than exactly `Self::WIRE_SIZE`.
> Previously this method only guarded the lower bound, silently accepting
> concatenated or trailing-garbage payloads — which weakened the wire-format
> contract and let malformed blobs parse as valid tokens.

The fix was applied to those three but not to the dozen-plus others. In
practice most of these types are framed by the upstream length prefix
(event-frame, subprotocol payload, routing header), so trailing bytes
are filtered out before `from_bytes` sees them. The risk surface is the
places that aren't framed — e.g. `ForkRecord` carried inside a
reconciliation payload, or `ContinuityProof` appended to a snapshot —
where concatenating multiple records or smuggling a trailer past the
parser could matter.

**Fix:** change each `data.len() < Self::WIRE_SIZE` to `!= Self::WIRE_SIZE`
(or the callsite-appropriate inequality) and update the small number of
call sites that intentionally slice out a prefix to pre-trim first.

## LOW — routing-vs-direct packet classification breaks for ~1-in-17M node_ids

**File:** `net/crates/net/src/adapter/net/mesh.rs`
**Lines:** `2170-2173`

```rust
let is_routed = data.len() >= ROUTING_HEADER_SIZE + protocol::HEADER_SIZE
    && u16::from_le_bytes([data[0], data[1]]) != MAGIC;
```

The discriminator peeks at the first two bytes of the packet and treats
it as routed iff they aren't `0x4E45` (`MAGIC`). Routed packets begin
with `RoutingHeader`, whose first field is `dest_id: u64` in LE — so
the first two bytes are the low 16 bits of the recipient's own
`node_id`. `node_id` is a truncated BLAKE2s hash of the entity keypair
(`identity/entity.rs:47-50`), effectively uniform. When a node's
low-16-bits happen to equal `MAGIC` **and** bit 16–23 happen to equal
`VERSION` (`1`), routed packets **to that node** pass the fake
Net-header `validate()` at `protocol.rs:447-452` and fall into the
direct-packet path.

AEAD fails and the packet is dropped, so no security or state
corruption — but that node silently fails to receive any routed packet
from the ~1-in-17M node_id ambiguity. The owner has no way to diagnose
it because the mesh looks healthy otherwise.

**Fix:** discriminate on a cheap tag byte in the routing header instead
of "anything that doesn't look like a Net magic." Set the top bit of
`RouteFlags` to `1` on all routing headers and use that bit as the
discriminator, or move `dest_id` out of byte-0 in the routing header so
the collision window shrinks to 1-in-2^{48+} and becomes effectively
zero.

---

## Notes on what this pass covered without finding anything

- `DiskSegment::append_entry` / `append_entries` (redex/disk.rs:258-307):
  dat-before-idx write order, proper torn-tail + torn-dat recovery at
  reopen (`open` / `read_index`), `set_len` truncation for either
  direction of torn state, and the external state lock in
  `RedexFile::append` serializes the two per-file locks here —
  concurrent appends can't interleave dat and idx writes.
- `SnapshotReassembler` wire-framing (`orchestrator.rs:302-346`):
  `total_chunks == 0`, overflow, oversize-chunk, stale `seq_through`,
  and `total_chunks_mismatch` are all rejected at the decoder
  boundary, and the in-memory feed is additionally re-validated.
- `PartitionDetector` (partition.rs): `healing_progress`
  divide-by-zero is guarded at line 87, duplicate recoveries are
  filtered (line 198), and `take_healed` drains atomically.
- `CausalLink::validate_chain_link` and `assess_continuity`
  (state/causal.rs, continuity/chain.rs): `sequence.checked_add(1)` used
  throughout, payload hash covers both ancestor and payload so payload
  tampering is detected as a parent-hash mismatch.
- `StandbyGroup::promote` (compute/standby_group.rs:241-281): old active
  marked unhealthy *before* searching for a replacement; `max_by_key`
  on `synced_through` is the longest-chain rule; NoHealthyMember is
  returned when no standby is eligible.
- `SubnetId` (subnet/id.rs:24-147): bit-packing is correct for all
  level indices 0..4, `mask_for_depth(d) == 0xFFFFFFFF` for
  out-of-range `d` (defensive saturation), `is_ancestor_of` handles
  global-as-ancestor correctly.
- `SubprotocolRegistry` (subprotocol/registry.rs): DashMap-based, no
  locks held across calls, `with_defaults` pre-populates the five
  core subprotocols without side effects.

---

## Notes on findings the initial sweep got wrong

An earlier exploratory pass flagged several issues that don't hold up under
inspection — leaving them here so they don't get re-reported:

- **`reliability.rs:281` allegedly off-by-one**: the check
  `seq <= self.ack_seq + 64` together with `offset = seq - self.ack_seq - 1`
  produces `offset ∈ [0, 63]`, which fits a `u64` bitmap exactly. Not a bug.
- **`channel/guard.rs` bloom OOB**: the analysis confused bit indices with
  byte indices. `h1 ∈ [0, 2^15)` is a bit index; `h1 >> 3` is the byte
  index into a `4096`-entry array — in range by construction.
- **`identity/token.rs` `is_valid` vs `is_expired` disagreement**: both use
  `now >= not_after`. The comment explicitly aligns them on strict
  `< not_after`. Not a bug.
