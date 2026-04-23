# Failure-Path Hardening Plan

Move the crate from "happy paths well-tested, recovery paths find real bugs when probed" (current state after the P1/P2/P3 test-coverage sweep) to "recovery paths are systematically exercised and the cross-node invariants hold under adversarial conditions." Five stages, ordered by ROI per hour; not every stage has to land — Stage 4 is the big lift and a team on a tight budget can stop at Stage 1-3 and still capture ~70% of the value.

> **Framing.** The `TEST_COVERAGE_PLAN.md` sweep closed individual invariant gaps: one test per documented contract. This plan is the next layer up — it asks "what classes of bug do we expect to exist that hand-authored tests won't find?" and builds infrastructure to surface those. The concrete motivation is the bug yield rate during the P1/P2/P3 sweep: writing happy-path-adjacent tests surfaced 3 real production bugs (P1-5 `on_failure` not clearing capability index, P1-7 `SequentialMapper` stuck on losing protocol, the NAT-PMP fallback publishing gateway LAN IP). That rate implies more bugs exist in recovery paths we haven't written tests for; rather than keep hand-writing them, we invest in infrastructure that generates test load faster than humans can.

## Goals

- Every `&[u8]` wire-boundary decode path is fuzzed.
- Every atomics-heavy core has a `loom` model-check harness covering its documented memory-ordering contract.
- Every (subprotocol × phase × failure-mode) triple has at least one failure-injection integration test.
- A deterministic simulator runs the cross-node protocols under random schedule + random failure injection, with a witness loop asserting cross-subsystem invariants.
- A nightly soak run proves no unbounded state growth under realistic mixed load.

## Non-goals

- **Line / branch coverage targets.** Same reasoning as `TEST_COVERAGE_PLAN.md` — reward testing trivial getters, miss the real bugs.
- **Rewriting existing tests into new frameworks.** The 1,539 passing tests stay as they are. This plan adds layers, doesn't replace.
- **General distributed-systems research.** If `madsim` or similar Rust-native DST infra fits, adopt it; don't build a new simulator from scratch.
- **Cryptographic formal verification.** Out of scope — the ed25519 / x25519 / noise crates we depend on are already audited upstream.

---

## Pre-flight: .unwrap_or audit (half day)

Before any Stage-1 infrastructure lands, sweep every `.unwrap_or(default)` and `.unwrap_or_else(|| default)` in recovery paths and ask: "when the main value is missing, is the default silently wrong, or is it a safe fallback?"

**Motivation.** `NatPmpMapper::install` does `cached_external.unwrap_or(self.gateway)`, which silently publishes the router's private LAN IP as the mapping's external address when `probe()` wasn't called first. Cubic caught this as a P1 one layer up (the `SequentialMapper` fallback), but the underlying sloppiness at `natpmp.rs:485` still exists — it's a land mine if any future caller forgets to probe. This class of bug is invisible to fuzzing (it doesn't panic) and invisible to happy-path tests (the main path populates the cache).

**Output.** A list of sites where the `.unwrap_or` fallback is a silent wrongness rather than a safe default. Fix each by either making the function return `Result<_, _>` when the precondition isn't met, or by documenting the precondition on the function signature and audit-pinning with a debug assertion.

**Why before Stage 1.** Stage 1 fuzz won't find these; Stage 4 DST will, but slowly and with noisy repro. A half-day eyeball sweep is the highest-ROI move available.

---

## Stage 1 — Wire-boundary fuzzing

**Status:** infrastructure landed; 5 of 7 targets written. Smoke runs (~10-15 s each) completed clean across ~1.9M inputs total, zero crashes, zero panics.

**Cost:** 1-2 weeks.
**Setup:** `cargo-fuzz v0.13.1` installed; `fuzz/` crate scaffolded at `net/crates/net/fuzz/` with `libfuzzer-sys` targets. Rust nightly is the only runtime prerequisite (libfuzzer-sys constraint).

**Targets landed:**

| Target | Entry point | Status |
|--------|-------------|--------|
| `capability_announcement_from_bytes` | `CapabilityAnnouncement::from_bytes` + round-trip + `verify()` + `is_expired()` | landed, smoke-run clean |
| `snapshot_reassembler_feed` | `SnapshotReassembler::feed` over sequence of ops with attacker-chosen `(origin, seq, index, total, payload)` | landed, smoke-run clean |
| `nat_pmp_decode_response` | `natpmp::decode_response` | landed, smoke-run clean |
| `migration_wire_decode` | `compute::orchestrator::wire::decode` + `encode` canonicalization round-trip | landed, smoke-run clean |
| `routing_header_from_bytes` | `RoutingHeader::from_bytes` + round-trip | landed, smoke-run clean |

**Targets pending:**

- `mesh_frame_decode` — every subprotocol ID × random payload through the top-level subprotocol dispatch. Requires a bit more setup (needs a live MeshNode or a reachable dispatcher helper).
- `channel_membership_envelope` — postcard-encoded roster / subscribe / unsubscribe messages.

These are deferred to a follow-up pass; they need harness work to expose the dispatcher without spinning up a full mesh.

**Invariants asserted by each target:**

1. No panic on any byte sequence.
2. No unbounded allocation (watch for `Vec::with_capacity(attacker_u32)`).
3. No slow-parse pathological cases (libfuzzer's per-input timeout catches these by default at 1200 s).
4. On `Ok(x)`, `encode(x)` round-trips to a value equal to `x` under `decode` (reassembler-feed target additionally asserts `pending_count() < 1024` to surface state-leak bugs).

**Runbook.**

```sh
# One-off smoke run, 60 s per target:
cd net/crates/net/fuzz
cargo +nightly fuzz run capability_announcement_from_bytes -- -max_total_time=60
cargo +nightly fuzz run snapshot_reassembler_feed         -- -max_total_time=60
cargo +nightly fuzz run nat_pmp_decode_response           -- -max_total_time=60
cargo +nightly fuzz run migration_wire_decode             -- -max_total_time=60
cargo +nightly fuzz run routing_header_from_bytes         -- -max_total_time=60

# Extended CI run, 1 hour per target (nightly job):
for t in capability_announcement_from_bytes snapshot_reassembler_feed \
         nat_pmp_decode_response migration_wire_decode \
         routing_header_from_bytes; do
  cargo +nightly fuzz run "$t" -- -max_total_time=3600
done

# Reproduce a past crash (artifact file written by libfuzzer):
cargo +nightly fuzz run <target> artifacts/<target>/crash-<hash>

# List all targets:
cargo +nightly fuzz list
```

**Corpus hygiene.**

- `fuzz/corpus/<target>/` is committed. libfuzzer writes newly-discovered coverage-expanding inputs here automatically; checking them in means fresh clones + CI re-reach the same branches without re-exploring from scratch.
- `fuzz/artifacts/<target>/` is git-ignored. Crashes land as `crash-<sha1>` files; reproduce by passing the path to `cargo fuzz run`.
- `fuzz/target/` is git-ignored (standard `cargo` build dir).

**ROI evidence from initial landing.** The 5-target × ~15-second smoke run explored 1.9M total inputs (migration_wire_decode alone did 485k runs at ~30k inputs/sec). That's a higher bug-imagination rate than any human author can match; the fact that nothing panicked is encouraging signal that the happy-path + hand-crafted-malformed coverage (P1/P2 sweep) left few low-hanging panics. A 24-hour CI run is the next step to surface the less-accessible state-space corners.

**Exit criterion.** Every target runs for 24 hours (cumulative, not simultaneous) in CI without finding new panics. Crashes that do surface reduce to committed regression tests via the `artifacts/<target>/crash-*` path. The two deferred targets (`mesh_frame_decode`, `channel_membership_envelope`) land in a follow-up before this stage is fully closed.

---

## Stage 2 — Concurrency model checking with loom

**Cost:** 1-2 weeks.
**Setup:** Add `loom` as a dev-dependency gated on `#[cfg(loom)]`. Wrap atomics / `Mutex` / `RwLock` uses with `loom::sync::*` aliases.

**Cores to model-check:**

- `AuthGuard` — bloom-bit writes + verified-map updates + exact-match ACL. The authorize/revoke race (P2-8) is barrier-stress-tested; loom would verify the memory-ordering annotations are correct, not just probabilistically-not-broken.
- `TokenCache` — insert / check / evict under concurrency (P2-9 barrier-stressed).
- `RoutingTable` — metric precedence under concurrent `add_route_with_metric` (P2-10).
- `CapabilityIndex` — versions↔nodes lock-ordering (P1-2 barrier-stressed).
- `FailureDetector` — `check_all` vs `on_failure` callback race.

**What loom catches that stress tests don't:**

- Acquire/Release vs SeqCst confusion (loom exhaustively explores the allowed reorderings; stress tests only hit observed hardware).
- Missing publication barriers (loom surfaces torn reads that hardware caches happened to hide).
- Lock-ordering deadlocks that only occur under specific interleavings.

**ROI rationale.** The barrier-retrofit pass this session found that 7 concurrency tests weren't actually racing under default scheduler behavior — a silent false-green. Loom makes the "are we actually exploring interleavings?" question moot: it explores all of them. Atomics-heavy code that passes loom has a much stronger correctness claim than code that passes `cargo test --release`.

**Exit criterion.** One loom harness per core, covering both main-path and recovery-path paths. Runs in < 10 minutes total so it can be part of the default test suite, not just nightly.

---

## Stage 3 — Failure-injection integration-test matrix

**Cost:** 2-3 weeks.
**Setup:** Extend the existing `punch_topology` / `build_node` helpers with a `chaos` module that supports: `crash_task(handle)`, `panic_in_callback(hook)`, `drop_next_packet(filter)`, `delay_next_packet(filter, duration)`.

**Matrix.** For each (subprotocol, phase, failure-mode) triple, one test asserting the documented invariant survives the failure.

Subprotocols: `pingwave`, `handshake`, `capability`, `rendezvous`, `migration`, `channel`, `partition`, `failure-detector`, `reflex-probe`, `port-mapping`.

Phases (per subprotocol): roughly `init → negotiate → establish → steady → tear-down`. Migration has its documented 6 phases; rendezvous has probe → punch → ack; etc.

Failure modes: `peer-crash-mid-phase`, `wire-packet-drop`, `wire-packet-duplicate`, `wire-packet-reorder`, `wire-packet-delay`, `clock-jump-forward`, `clock-jump-backward`, `partition-split`, `partition-heal-mid-phase`, `resource-exhaustion`.

**Existing tests this generalizes.** `tests/peer_death_clears_capability_index.rs`, `tests/migration_target_failure_mid_chunking.rs`, and `tests/rendezvous_coordinator.rs`'s staleness case are hand-crafted instances. The matrix systematizes what's currently ad-hoc.

**Invariants the matrix asserts (examples):**

- Capability index membership stays ⊆ live peers under every failure-mode.
- No migration is pending > `N × session_timeout` after any failure.
- Routing table has no dangling next-hops referencing dead peers.
- Channel rosters converge within `N × heartbeat_interval` of partition heal.
- No duplicate event delivery across partition heal.

**ROI rationale.** This is where the hand-authored sweep hit diminishing returns: each new test is ~100 lines and covers one cell of the matrix. Mechanizing the matrix means 50 tests generated from a 200-line harness instead of 50 × 100 = 5,000 lines of hand-written code. More importantly, it forces explicit enumeration: when a new subprotocol lands, it gets N rows in the matrix by default rather than being silently untested against partition heal.

**Exit criterion.** Every subprotocol has a row for every failure-mode, and every cell is either a passing test or a documented "this combination isn't meaningful because X." No cell is silently missing.

---

## Stage 4 — Deterministic simulation testing

**Cost:** 6-12 weeks. This is the big lift.
**Setup:** Adopt `madsim` (Rust-native deterministic async runtime + virtual network) or fork something similar. Don't write from scratch — every distributed team that writes their own simulator regrets it.

**What it looks like.** `MeshSim::new(n_nodes)` returns N `MeshNode` instances whose tokio runtime, UDP socket, and `Instant::now()` are all simulator-controlled. The simulator:

1. Advances time in virtual steps.
2. Delivers, drops, reorders, duplicates, delays packets according to a seeded random schedule.
3. Injects node crashes at phase boundaries.
4. Injects clock jumps (forward + backward) on individual nodes.
5. Injects partitions (A cannot reach B; B can still reach A; heal after T).

**Witness loop.** On every simulated tick, a witness thread queries every node's `.health_snapshot()` and asserts cross-node invariants:

- Global capability index membership stays consistent modulo GC lag.
- Routing tables have no dangling next-hops.
- No migration is pending beyond its timeout budget.
- Channel publishers observe the same roster membership that subscribers see (eventually).
- `sum(delivered_events) == sum(ingested_events)` across partitions once healed.

**Why it matters.** This is where FoundationDB found its bugs, how TigerBeetle proves its recovery semantics, how Antithesis hunts heisenbugs. The bug rate at DST maturity is roughly "one new protocol bug per N hours of simulated wall-clock." Without DST, those bugs become production incidents; with it, they become failing seeds committed to CI.

**Prerequisite: determinism.** Every source of nondeterminism in the crate has to go through the simulator's clock + RNG:

- `Instant::now()` → simulator time.
- `SystemTime::now()` → simulator time.
- `tokio::time::sleep` → simulator sleep.
- `rand` → simulator RNG with caller-supplied seed.
- `UdpSocket` → simulator-backed channel.

Some of this is already clean (the crate mostly threads `Instant` through); some needs refactoring (any `std::time::SystemTime` in TTL code needs to be swappable for a test clock).

**ROI rationale.** Expensive to build, but the bug yield at maturity is higher than Stages 1-3 combined. Stages 1-3 are necessary — they find the bugs that don't need cross-node state to manifest. Stage 4 finds the bugs that do: three-way agreement drift, partition-heal divergence, migration that only deadlocks when the target's X25519 rekey races the source's cutover.

**Exit criterion.** A nightly DST run with 100 seeds × 30-minute simulated-time horizons completes with zero witness-loop failures. Any failing seed reduces to a committed regression test with the failing schedule.

---

## Stage 5 — Nightly soak + chaos

**Cost:** 1 week setup, ongoing cost is CI minutes.
**Setup:** Dedicated nightly CI job, 5-node topology, realistic mixed workload generator, Linux `tc`-based network chaos layer.

**Workload.** Over 24 hours:

- Continuous pub/sub fanout at 1k events/sec across 20 channels.
- Continuous capability reclassification (simulate NAT rebinds every N minutes).
- Continuous migration traffic (move a daemon between nodes every M minutes).
- Random `block_peer` / `unblock_peer` toggles.

**Chaos.** Linux `tc` on loopback injects:

- Packet loss (0%, 1%, 5%, 20% bands; rotate every hour).
- Bandwidth caps (unlimited, 10 Mbps, 1 Mbps; rotate every hour).
- Latency jitter (0 ms, 10 ms ± 5 ms, 100 ms ± 50 ms; rotate every hour).

**Assertions.** After the 24-hour window:

- RSS growth < X% per subsystem (no unbounded state).
- `capability_index.len()`, `routing_table.len()`, `session_count` all bounded.
- `migrations_pending == 0` at quiescence.
- `panics_observed == 0`.
- Every injected `block_peer` leaves no orphaned entries after the session timeout.

**ROI rationale.** Only pays off after Stages 1-3 land. Soak on a codebase that still has simple races just reproduces the simple races expensively. Once the simple races are gone, soak finds the slow leaks + the bugs whose arrival rate is "once per billion events."

**Exit criterion.** Nightly job is green for 30 consecutive nights.

---

## Cross-cutting prerequisites

Half of what makes Stages 2-4 tractable is already in the crate; the rest is small additions:

- **`.health_snapshot()` on every subsystem.** Several (`RoutingTable`, `CapabilityIndex`, `AuthGuard`, `FailureDetector`, `MigrationOrchestrator`, `SessionTable`) already have `stats()`-style accessors. Standardize the name + return shape so a witness loop can poll uniformly.
- **Structured event log.** Minimal counters + transition records (who moved to which phase when) written to an append-only buffer. Stage 4 failures need post-mortem material; without it, DST repros are forensically useless.
- **Panic hook.** Captures full mesh state (every subsystem's snapshot) on any `panic!`. Stage 1 fuzz + Stage 4 DST are the consumers.
- **Deterministic-clock injection point.** Thread a `Clock` trait through the TTL / timeout / GC machinery so `MeshSim` can drive time. Most of this already uses `Instant`, which threads cleanly; the `SystemTime::now()` call sites (primarily in `is_expired` paths) are the blockers.

---

## Ordering + tradeoffs

**Recommended ordering:** pre-flight `.unwrap_or` audit → Stage 1 (fuzz) → Stage 2 (loom) → Stage 3 (failure-injection matrix) → Stage 4 (DST) → Stage 5 (soak).

**If resources force cutting:**

- Cut Stage 5 first — it only pays off after Stages 1-4.
- Cut Stage 4 next — expensive, highest yield, but Stages 1-3 + the existing unit/integration tests cover 70% of the total value for 30% of the cost.
- Stages 1-3 are the floor: a crate shipping distributed-systems primitives without wire-boundary fuzzing and concurrency model checking is not hardened, full stop.

**What NOT to do:**

- Don't try to enumerate every possible failure with human-written tests. Unbounded work; you'll never cover the bugs you didn't imagine. Let fuzz + DST imagine for you.
- Don't rewrite existing tests into any new framework. Additive layers only.
- Don't build a custom simulator if `madsim` fits — the maintenance cost of a bespoke sim across years is catastrophic.
- Don't skip the pre-flight `.unwrap_or` audit. The `NatPmpMapper` silent-wrong-IP is the kind of bug no amount of testing infrastructure catches cheaply — it only shows up as a capability announcement publishing a 192.168.x.x external address, which DST would take many seeds to reproduce. A human sweep costs half a day.

## Exit criteria for the whole program

- Pre-flight audit: zero sites where `.unwrap_or(default)` is silently wrong.
- Stage 1: every wire decoder fuzzed for 24 cumulative CI hours without new panics.
- Stage 2: every atomics-heavy core has a loom harness covering its documented memory-ordering contract.
- Stage 3: the (subprotocol × phase × failure-mode) matrix has no silently missing cells.
- Stage 4: 100-seed × 30-minute DST run completes clean nightly.
- Stage 5: 30 consecutive green nightly soak runs.

At that point the crate has left "hardening" and entered "maintenance" for failure paths — new bugs come from new features, not from the existing surface.
