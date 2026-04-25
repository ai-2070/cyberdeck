# Plan — `@ai2070/crew`

A deterministic, replayable, for-loop crew orchestrator. Validates two JSON blobs (shape + counts) with Zod, materializes a virtual graph, and steps through it. Everything runtime-side (model calls, tool dispatch) is behind interfaces — the library itself is pure orchestration.

## Design invariants

These are the load-bearing constraints. Everything else flexes around them.

1. **Determinism.** Same `(shape, counts, inputs, hook outputs, runtime outputs)` → same event log. No `Date.now()`, no `Math.random()`, no `Map` iteration order leaks. All time/randomness comes from an injected `Clock` + `Rng`.
2. **Event log is the source of truth.** The crew loop emits an append-only stream of typed events (`crew.started`, `role.entered`, `agent.step`, `vote.resolved`, `checkpoint.taken`, `memex.delta_emitted`, `crew.completed`). State is derivable from the log.
3. **Replay = re-derivation.** Given the log + the same `AgentRuntime` outputs (or a recorded transcript of them), the loop produces the same final state. This is what makes checkpoints atomic.
4. **No I/O in core.** MemEX, model calls, tools, persistence — all behind interfaces. Core has zero side-effects beyond emitting events to a sink you provide.
5. **Cancellation, timeouts, and budgets are first-class.** `AbortSignal` threads through every async edge; per-step and per-crew budgets are part of the schema, not best-effort wrappers.

## External dependencies

- **Zod v4 only.** No v3 support. Lock the major in `peerDependencies`.
- **`@ai2070/memex`** (peer, optional). Used through `MemexAdapter`. Crew library does not import it directly.
- **L0 inference substrate** (peer). Streaming, token transport, and runtime concurrency live there. The crew loop sees one-shot `runAgentStep` returns; whether the substrate streams underneath is invisible to the loop.
- **Schema version locked at `1.0`.** No backcompat shims this round — breaking changes are fine until 1.0 ships externally.

## Package layout

```
crew/
├── package.json              # @ai2070/crew, ESM, peerDeps: zod, @ai2070/memex (optional)
├── tsconfig.json             # strict, NodeNext
├── src/
│   ├── index.ts              # public surface
│   ├── schema/
│   │   ├── shape.ts          # CrewShapeSchema
│   │   ├── agents.ts         # CrewAgentsSchema
│   │   ├── consensus.ts      # ConsensusConfigSchema, VotingMode
│   │   └── hooks.ts          # LifecycleHooksSchema
│   ├── graph/
│   │   ├── types.ts          # CrewGraph, CrewRole, CrewAgent, RoleId, AgentId
│   │   └── build.ts          # buildCrewGraph(shape, counts, registry?)
│   ├── runtime/
│   │   ├── types.ts          # AgentRuntime, HookRegistry, Clock, Rng
│   │   ├── hooks.ts          # InMemoryHookRegistry
│   │   └── ids.ts            # deterministic agent/event id generation
│   ├── events/
│   │   ├── types.ts          # CrewEvent discriminated union
│   │   ├── log.ts            # EventSink interface, InMemoryEventLog
│   │   └── replay.ts         # replay(log, runtime, hooks) -> state
│   ├── memex/
│   │   ├── adapter.ts        # MemexAdapter interface (read view + delta sink)
│   │   └── ai2070.ts         # optional adapter against @ai2070/memex
│   ├── checkpoint/
│   │   ├── types.ts          # Checkpoint shape (graph snapshot + log offset + memex cursor)
│   │   └── store.ts          # CheckpointStore interface, InMemoryCheckpointStore
│   ├── voting/
│   │   └── resolve.ts        # resolveVotes(...) — first_valid | unanimous | majority | weighted
│   └── loop/
│       ├── run.ts            # runCrewOnce(graph, runtime, hooks, opts)
│       └── nested.ts         # runAgentOrNested(...)
└── test/
    ├── schema.test.ts
    ├── build.test.ts
    ├── loop.test.ts
    ├── replay.test.ts        # determinism: log -> replay -> identical final state
    ├── checkpoint.test.ts    # checkpoint -> resume -> identical outcome
    └── nested.test.ts
```

## Phases

### Phase 1 — Schemas + graph builder (the boilerplate, hardened)

Take the second snippet from the draft verbatim as the starting point, plus:

- Reject the input early if a role with `first_input: true` does not appear, or if more than one role has `first_input` / `final_output`.
- Enforce `permissions.talk_to`, `delegate_to`, `escalate_to`, `invite[].role` reference declared roles. Mismatch → schema-level error, not runtime.
- Enforce `max_allowed` ≥ requested count (warn vs throw is a config flag — default throw, since silent capping is a footgun).
- Sort `roles.entries()` deterministically by declaration order so the for-loop is stable.
- `buildCrewGraph` is pure: takes `(shape, counts, registry?)`, returns `CrewGraph`. No I/O.
- Schemas authored against **Zod v4** APIs (`z.object`, `z.discriminatedUnion`, `.parse` / `.safeParse`).
- Top-level shape requires `schema_version: "1.0"`. Reject anything else.
- Add a separate `lintCrewShape(shape)` pass for semantic checks Zod can't express: no role escalates to itself, every `delegate_to` / `talk_to` / `escalate_to` / `invite[].role` references a declared role, exactly one `first_input` and exactly one `final_output`, fixer's `talk_to` covers everyone in its `delegate_to`. Returns warnings + errors; runtime refuses errors, surfaces warnings.
- Optional per-role `input_schema` / `output_schema` (Zod schemas). When present, the loop validates I/O at the role boundary.

Deliverable: `CrewShapeSchema.parse(...)` + `buildCrewGraph(...)` + `lintCrewShape(...)` + tests against the `DEFAULT_CREW` example.

### Phase 2 — Event log + atomic/replayable runner

The thing that turns the loop from "a script" into a real orchestrator.

```ts
type Usage = { tokens_in?: number; tokens_out?: number; cost?: number; latency_ms?: number };

type CrewEvent =
  | { type: "crew.started"; crewId: string; rootInput: unknown; ts: number }
  | { type: "role.entered"; roleId: string; ts: number }
  | { type: "agent.step.requested"; agentId: string; roleId: string; input: unknown; ts: number }
  | { type: "agent.step.completed"; agentId: string; output: unknown; usage?: Usage; fault?: boolean; stalled?: boolean; ts: number }
  | { type: "agent.step.failed"; agentId: string; error: { name: string; message: string }; ts: number }
  | { type: "agent.step.timed_out"; agentId: string; budget_ms: number; ts: number }
  | { type: "fixer.invoked"; agentId: string; reason: "fault" | "stall" | "timeout" | "permission_denied"; ts: number }
  | { type: "permission.denied"; from: string; to: string; action: "talk_to" | "delegate_to" | "escalate_to" | "invite"; ts: number }
  | { type: "vote.resolved"; roleId: string; mode: VotingMode; resolved: unknown; ts: number }
  | { type: "checkpoint.taken"; checkpointId: string; ts: number }
  | { type: "memex.delta.emitted"; agentId: string; delta: MemexDelta; ts: number }
  | { type: "nested.crew.started"; agentId: string; nestedName: string; ts: number }
  | { type: "nested.crew.completed"; agentId: string; output: unknown; ts: number }
  | { type: "crew.aborted"; reason: "cancelled" | "budget_exceeded" | "timeout"; ts: number }
  | { type: "crew.completed"; finalOutput: unknown; usage?: Usage; ts: number };
```

`runCrewOnce` writes to an `EventSink`. A second function, `replay(log, runtimeStub)`, walks events and reconstructs the final state (used in tests + checkpoint resume). Deterministic IDs: `agent.step` event id = hash of `(crewId, roleId, agentId, parentEventId)`.

This is also where the **atomicity** guarantee lives: the loop never updates external state directly. It emits a `memex.delta.emitted` event; the adapter applies it. If the loop crashes mid-phase, replaying the log up to the last checkpoint reconstructs identical state.

Three things this phase also pins down:

- **Canonical JSON.** Ship `canonicalize(value)` (sorted keys, normalized number forms). Used wherever events or vote outputs are hashed/compared. Stock `JSON.stringify` is not stable across object key order.
- **Exception model.** Throws inside `runAgentStep` are caught and turned into `agent.step.failed` events (distinct from modeled `{ fault: true }` outputs). Failed steps are eligible for fixer intervention if the role's `activation.on_fault` is set.
- **Idempotency on resume.** If the log has `agent.step.requested` with no matching `completed` / `failed` / `timed_out`, resume consults a `ResumePolicy` (`"retry" | "skip" | "abort"`, caller-supplied). Default `"abort"` — the caller has to make the call explicitly.

### Phase 3 — Loop with hooks, fixer, nested crews, parallelism

This is the second snippet from the draft, lightly cleaned. Concrete additions:

1. **`runAgentOrNested`** passes through the same `HookRegistry` and `EventSink` (the draft passes `{ get: () => undefined }` which silently swallows hooks in nested crews — change to inherited registry).
2. **Nested crew memex flow:** parent passes its `MemexAdapter` view down by default, but a role can declare `memex: { isolation: "soft" | "hard" }`. `"hard"` means the adapter exports a slice (`exportSlice` from MemEX) before the nested run and `importSlice`s the result on completion — this maps onto the patterns in `memex/README.md` §"Hard isolation (exported slices)". `"soft"` is the default and just shares the adapter.
3. **Parallelism within a phase.** All agents in the same role run via `Promise.all`. After settle, results are sorted by `agentId` before voting/aggregation. Determinism preserved regardless of completion order. Roles can opt out with `execution.serial: true`.
4. **AbortSignal everywhere.** `runCrewOnce(graph, runtime, hooks, { signal })`. Cancellation aborts in-flight steps via the runtime, drains pending phases, emits `crew.aborted`. Nested crews receive a child signal.
5. **Timeouts.** Per-role `execution.timeout_ms` (and a per-crew default in `opts`). Step exceeds budget → `agent.step.timed_out` → fixer if `activation.on_fault` (timeout counts as fault for activation purposes; the event type stays distinct so traces can tell them apart).
6. **Permission enforcement.** `talk_to` / `delegate_to` / `escalate_to` are enforced by the loop, not informational. Hooks/runtime request an action via `crew.requestAction({ from, to, action })`; if the source role's permission set doesn't allow it, the loop emits `permission.denied` and the action is rejected. Fixer can be activated on denial via `activation.on_permission_denied`.
7. **Per-step budgets.** Optional `execution.max_tokens` / `execution.max_cost` on a role. Runtime is expected to surface `usage` on each completion; the loop accumulates into a per-crew total and emits `crew.aborted { reason: "budget_exceeded" }` if exceeded.

### Phase 4 — MemEX integration

Define `MemexAdapter` minimally:

```ts
interface MemexAdapter {
  view(scope?: string): MemexGraph;          // read-only handle for runtime
  applyDelta(delta: MemexDelta, ctx: { agentId: string; crewId: string }): void;
  exportSlice?(memory_ids: string[]): MemexSlice;
  importSlice?(slice: MemexSlice): { created: number; updated: number };
}
```

Ship one concrete adapter, `createMemexAdapter(state)`, that wraps `@ai2070/memex`'s `applyCommand` / `getItems` / `exportSlice` / `importSlice`. The crew library does **not** depend on MemEX at runtime — peer-dependency, behind a feature import: `import { createMemexAdapter } from "@ai2070/crew/memex"`.

Each agent step that produces a `memexOut` flows: agent → event log (`memex.delta.emitted`) → adapter (`applyDelta`). The event sits between them so replay works without re-running the model.

**Per-agent scoped memory.** The adapter auto-populates `meta.agent_id`, `meta.crew_id`, and a `scope` of `crew:{crewId}/agent:{agentId}` on every write. The runtime never has to set these — they come from the adapter wrapper around `applyCommand`. Reads default to the agent's own scope; cross-agent visibility is opt-in via `view({ scope_prefix: "crew:{crewId}/" })` (orchestrator pattern from `memex/README.md` §"Soft isolation"). Roles declare visibility on the shape: `memex.view: "self" | "role" | "crew" | "all"` (default `"self"`).

### Phase 5 — Checkpointing

A checkpoint = `{ graph snapshot, event log offset, memex cursor (or slice digest), timestamp }`. Triggered:

- Per role's `execution.checkpoints[]` (declarative, from the shape).
- Programmatically via `crew.checkpoint(id)` from inside a hook.
- Automatically at phase boundaries if `opts.autoCheckpoint = "phase"`.

`resumeCrew(checkpoint, runtime, hooks)` reads the checkpoint, re-hydrates state, and continues the loop. Resume must produce a log that, concatenated with the pre-checkpoint log, equals what a fresh run would have produced — this is the property tested in `checkpoint.test.ts`.

### Phase 6 — Voting / resolution (later, but stub now)

Take `resolveVotes` from the draft as Phase-6 scaffolding. Ship `first_valid` working, the rest as named-but-stubbed strategies that throw "not implemented". This way the schema accepts them and roles can declare `execution.voting`, but we don't pretend to support semantics we haven't built. Wire `weighted_consensus` first since it's what the example shape uses.

### Phase 7 — Dynamic crews (later)

`fixer.permissions.modify_structure` is the hook. Add a `CrewMutation` event type (`role.added`, `agent.added`, `agent.removed`, `permission.changed`). The fixer hook returns a list of mutations; the loop applies them between phases (never mid-phase — that's where determinism dies). All mutations land in the event log so replay still works.

`permissions.invite` is part of this phase. In v1 the schema accepts it (so existing crew shapes parse) but the loop rejects invitations at runtime with `permission.denied { action: "invite" }`. Don't pretend to honor an invite when there's no machinery behind it.

Defer until Phase 1–5 are stable and tested.

## Decisions

- **Zod v4** only. Locked.
- **Schema version `1.0`**. No backcompat shims pre-1.0 release.
- **`talk_to` / `delegate_to` / `escalate_to`** are enforced by the loop. See Phase 3 §6.
- **Streaming** is L0's responsibility. The crew loop sees one-shot `runAgentStep` returns.
- **Per-agent scoped memory** is the default. See Phase 4.
- **Crew id** is caller-injected. The library does not generate it — too easy to leak nondeterminism.
- **Phase order** is declaration order minus `first_input` / `final_output`. Explicit `phase: number` revisits if/when this is too coarse.

## What to build first (smallest useful slice)

1. Phase 1 schemas + builder.
2. Phase 2 event log + a no-op `AgentRuntime` that echoes input.
3. Phase 3 loop (caller → mercs → specialist → caller) running against the example, producing a deterministic event log.
4. Replay test: run twice, assert event logs are byte-identical.

Everything past that — fixer, MemEX, checkpoints, voting, dynamic — slots onto this skeleton without rewriting it.
