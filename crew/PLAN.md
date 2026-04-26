# Plan — `@ai2070/crew`

A deterministic, replayable, bus-agnostic crew orchestrator. Validates two JSON blobs (shape + counts) with Zod, materializes a virtual graph, and runs it as a **state machine** that emits lifecycle events and consumes responses. The library does no I/O: it does not call models, subscribe to a bus, or persist anything. Plug it onto whatever transport you want — Net later, in-process `EventEmitter` now, anything in between.

## Design invariants

1. **Event-driven state machine.** No `Promise<Output>` from agent calls. The session emits `agent.step.requested` lifecycle events; the caller delivers `agent.step.completed` / `.failed` events back. The library is a coroutine, not an executor.
2. **Bus-agnostic.** No L0 import, no Net import, no preferred transport. The session interface is `start(input) → events`, `deliver(event) → events`, `tick(now) → events`, `cancel() → events`. Wire it however.
3. **Determinism.** Same `(shape, counts, ordered inbound events)` → same outbound event sequence. `Clock` and `Rng` are injected. No hidden time, no hidden randomness, no `Map` iteration leaks.
4. **Event log is the source of truth.** Every state transition is an event. State is `replay(log)`. Snapshots are an optimization, not authoritative.
5. **Replay = re-derivation.** Identical inbound event sequence → identical outbound event sequence.
6. **Cancellation and timeouts are first-class.** `cancel()` is part of the API. Timeouts are scheduled internally; expiry produces an `agent.step.timed_out` event without external prompting.

## External dependencies

- **Zod v4 only.** No v3. MemEX exposes Zod v4 schemas under `@ai2070/memex/schemas` — reuse them, don't redefine memory types.
- **`@ai2070/memex`** (`^0.12`, peer, optional). Used through `MemexAdapter`. Crew core never imports it. The bundled adapter lives at `@ai2070/crew/memex` and wraps `applyCommand` / `getItems` / `smartRetrieve` / `exportSlice` / `importSlice` / `toJSON`.
- **No L0 dependency.** L0 is a typical worker on the other side of the bus, not a dependency. An L0 worker recipe lives in docs.
- **No bus dependency.** Caller wires the session to whatever transport (Net, `EventEmitter`, in-process queue, Kafka, …) they want.
- **Schema version locked at `1.0`.** Breaking changes are fine until 1.0 ships externally.

## Architecture

```
                 ┌────────────────────────┐
                 │     crew library       │
                 │  (state machine only)  │
                 └────────────────────────┘
                       ▲              │
              inbound  │              │ outbound
              events   │              │ lifecycle events
                       │              ▼
                 ┌────────────────────────┐
                 │         bus            │  ← Net, EventEmitter, …
                 │  (caller's choice)     │
                 └────────────────────────┘
                       ▲              │
                       │              ▼
                 ┌────────────────────────┐    ┌──────────────┐
                 │     agent worker       │ →  │      L0      │
                 │   (caller's code)      │    │   (model)    │
                 └────────────────────────┘    └──────────────┘
```

The library is the box at the top. Everything else is the caller's wiring.

## Public API

```ts
import type { CrewGraph, CrewSnapshot, CrewEvent, CrewStatus, HookRegistry, MemexAdapter } from "@ai2070/crew";

export function createCrewSession(
  graph: CrewGraph,
  opts: {
    crewId: string;                 // caller-injected, deterministic
    hooks?: HookRegistry;
    memex?: MemexAdapter;           // optional
    clock: Clock;                   // injected — no Date.now() inside the lib
    rng: Rng;                       // injected
    resumePolicy?: ResumePolicy;
  },
): CrewSession;

export function resumeCrewSession(
  snapshot: CrewSnapshot,
  laterEvents: CrewEvent[],         // inbound events that arrived after the snapshot
  opts: SameAsAbove,
): CrewSession;

export interface CrewSession {
  start(rootInput: unknown): CrewEvent[];
  deliver(event: CrewEvent): CrewEvent[];           // inbound → outbound burst
  tick(now: number): CrewEvent[];                   // drives internal timeouts
  cancel(reason: "cancelled" | "timeout"): CrewEvent[];
  status(): CrewStatus;                             // "idle" | "awaiting_responses" | "completed" | "aborted"
  pendingRequests(): readonly AgentStepRequest[];
  snapshot(): CrewSnapshot;
}
```

`deliver(agent.stream.chunk)` does not advance state — the chunk is appended to the session's internal log for downstream observers but the state machine ignores it for transitions.

## Package layout

```
crew/
├── package.json              # @ai2070/crew, ESM, peerDeps: zod
├── tsconfig.json
├── src/
│   ├── index.ts              # public surface
│   ├── schema/               # Zod v4
│   │   ├── shape.ts
│   │   ├── agents.ts
│   │   ├── consensus.ts
│   │   └── hooks.ts
│   ├── graph/
│   │   ├── types.ts
│   │   └── build.ts          # buildCrewGraph + lintCrewShape
│   ├── events/
│   │   ├── types.ts          # CrewEvent discriminated union
│   │   └── canonical.ts      # canonicalize(value) — stable JSON
│   ├── runtime/
│   │   ├── clock.ts          # Clock interface + system + frozen impls
│   │   ├── rng.ts            # Rng interface + deterministic + system impls
│   │   ├── ids.ts            # deterministic correlation/event ids
│   │   └── hooks.ts          # HookRegistry
│   ├── session/              # the state machine
│   │   ├── types.ts          # CrewSession, CrewStatus, CrewSnapshot, AgentStepRequest
│   │   ├── machine.ts        # start / deliver / tick / cancel
│   │   ├── permissions.ts    # talk_to / delegate_to / escalate_to enforcement
│   │   ├── timeout.ts        # internal timeout scheduling
│   │   └── nested.ts         # nested crew bookkeeping
│   ├── memex/
│   │   ├── adapter.ts        # MemexAdapter interface + AgentMemexHandle
│   │   └── ai2070.ts         # adapter against @ai2070/memex
│   ├── checkpoint/
│   │   ├── types.ts
│   │   └── store.ts          # CheckpointStore interface, in-memory impl
│   └── voting/
│       └── resolve.ts
└── tests/
    ├── schema.test.ts
    ├── build.test.ts
    ├── machine.test.ts
    ├── replay.test.ts
    ├── checkpoint.test.ts
    ├── nested.test.ts
    └── permissions.test.ts
```

## Phases

### Phase 1 — Schemas + graph builder

- Zod v4 (`z.object`, `z.discriminatedUnion`, `.parse`, `.safeParse`).
- `CrewShapeSchema` requires `schema_version: "1.0"`. Reject anything else.
- `CrewAgentsSchema` for counts.
- `buildCrewGraph(shape, counts, registry?)` is pure: deterministic, no I/O, sorts by declaration order so the for-loop is stable.
- `lintCrewShape(shape)` — semantic checks Zod can't express:
  - Exactly one role with `first_input: true`, exactly one with `final_output: true`.
  - Every `talk_to` / `delegate_to` / `escalate_to` / `invite[].role` references a declared role.
  - No role escalates to itself.
  - Fixer's `talk_to` covers everyone in its `delegate_to`.
  - Returns `{ errors, warnings }`. Errors block runtime; warnings surface to the caller.
- Optional per-role `input_schema` / `output_schema` (Zod schemas). When present, the session validates I/O at the role boundary.
- Extend `activation` beyond the boilerplate: `{ on_fault?: boolean; on_stall?: boolean; on_timeout?: boolean; on_permission_denied?: boolean }`. Each flag routes the corresponding event to the fixer when set.
- `permissions.invite` is **accepted** by the schema (so existing shapes parse) but **never honored** in v1 — invitations belong to the v2 dynamic-crew plan.

Deliverable: `parse + lintCrewShape + buildCrewGraph` against the `DEFAULT_CREW` example.

### Phase 2 — Event vocabulary + canonical JSON

Every state transition is an event. Outbound events are emitted by the session; inbound events are delivered by the caller.

```ts
type CrewEvent =
  // ─── outbound (session-emitted) ───
  | { type: "crew.started"; crewId: string; rootInput: unknown; ts: number }
  | { type: "role.entered"; roleId: string; ts: number }
  | {
      type: "agent.step.requested";
      correlationId: string;
      agentId: string;
      roleId: string;
      input: unknown;
      memex_context?: unknown;            // sampled per request — see Phase 3 phase→phase flow
      role: {                              // self-contained snapshot — workers don't load shape out-of-band
        name: string;
        description?: string;
        capabilities?: { model?: string; tools?: string[]; thinking_allowed?: boolean };
        permissions: { talk_to: string[]; delegate_to: string[]; escalate_to: string[] };
      };
      timeoutMs?: number;
      ts: number;
    }
  | { type: "fixer.invoked"; agentId: string; reason: "fault" | "stall" | "timeout" | "permission_denied"; ts: number }
  | { type: "permission.denied"; from: string; to: string; action: "talk_to" | "delegate_to" | "escalate_to"; ts: number }
  | { type: "vote.resolved"; roleId: string; mode: VotingMode; resolved: unknown; ts: number }
  | { type: "checkpoint.taken"; checkpointId: string; ts: number }
  | { type: "memex.command.emitted"; agentId: string; command: MemoryCommand; ts: number }
  | { type: "nested.crew.started"; agentId: string; nestedName: string; ts: number }
  | { type: "nested.crew.completed"; agentId: string; output: unknown; ts: number }
  | { type: "crew.aborted"; reason: "cancelled" | "timeout"; ts: number }
  | { type: "crew.completed"; finalOutput: unknown; ts: number }
  // ─── inbound (caller-delivered) ───
  | { type: "agent.step.completed"; correlationId: string; output: unknown; memex_commands?: MemoryCommand[]; fault?: boolean; stalled?: boolean; ts: number }
  | { type: "agent.step.failed"; correlationId: string; error: { name: string; message: string }; ts: number }
  | { type: "agent.step.timed_out"; correlationId: string; ts: number }     // caller may inject; session also self-emits
  | { type: "agent.stream.chunk"; correlationId: string; chunk: unknown; ts: number }; // pass-through, non-state-advancing
```

Four things this phase pins down:

- **Canonical JSON.** Ship `canonicalize(value)` (sorted keys, normalized number forms). Used wherever events or outputs are hashed or compared. Stock `JSON.stringify` is not stable across object key order.
- **Correlation ids.** Each `agent.step.requested` carries `correlationId = hash(crewId, roleId, agentId, attempt)`. Inbound events match by id. Determinism + parallelism = inbound order doesn't matter.
- **Idempotency on duplicate `deliver`.** Buses retry. The session tracks per-`correlationId` terminal status. The first terminal event (`completed` / `failed` / `timed_out`) wins; subsequent terminal events for the same id are no-ops returning `[]`. Stream chunks always append (duplicates produce duplicate chunks, harmless).
- **Idempotency on resume.** If the log has `agent.step.requested` with no matching terminal event, resume consults `ResumePolicy` (`"re-emit-request" | "treat-as-failed" | "abort"`, caller-supplied). Default `"abort"`.

### Phase 3 — Session state machine

The core. Replaces the async/await for-loop with an event-driven machine.

```
   idle
    │ start(rootInput)
    ▼
   awaiting_caller    (one outstanding agent.step.requested)
    │ agent.step.completed
    ▼
   awaiting_phase     (N outstanding requests for current role)
    │ all completed/failed/timed_out
    ▼
   ... next phase ... → final phase → completed
                         │
                         │ cancel() at any point
                         ▼
                       aborted
```

Mechanics:

1. **`start(rootInput)`** transitions to `awaiting_caller`. Emits `crew.started` + `role.entered` (caller) + `agent.step.requested` for the caller. Schedules a timeout via `Clock`.
2. **`deliver(event)`** matches by `correlationId`. Advances when all in-flight requests for the current phase have resolved (`completed` / `failed` / `timed_out`).
3. **`tick(now)`** is the timeout cranks. For every in-flight request whose `requested.ts + timeoutMs ≤ now`, emit `agent.step.timed_out` and treat as a fault. Caller is responsible for calling `tick()` periodically (or wiring it to a real `setTimeout`); a deterministic test can call `tick(t)` with synthetic times.
4. **`cancel(reason)`** drains the queue, emits `crew.aborted`, transitions to `aborted`. In-flight workers are expected to observe `crew.aborted` on the bus and stop. The library does not cancel them — that's bus territory.
5. **Permission enforcement (ACL gates only).** `talk_to` / `delegate_to` / `escalate_to` are validated, not orchestrated. The library does not spawn delegated steps, route messages, or create subflows. Hooks/workers call `crewControl.requestAction({ from, to, action })`; the machine consults the source role's `permissions`, emits `permission.denied` if disallowed, and returns `false` to the caller. The actual delegation/messaging is the worker's or hook's job — outside the library. (Dynamic auto-delegation is v2.)
6. **Parallelism.** All agents in a role emit `agent.step.requested` simultaneously. Machine waits for all to resolve before voting. Outputs sorted by `agentId` before voting (determinism). Roles can opt into serial with `execution.serial: true`.
7. **Nested crews.** When an agent has a `nestedCrew`, the machine instantiates an inner `CrewSession`, prefixes correlation ids (`{outer}/{inner}`), and proxies events. Inner `crew.completed.finalOutput` becomes the outer agent's step output.
8. **Fixer activation.** `fault` / `stalled` / `timed_out` / `permission_denied` route to fixer if the corresponding `activation.*` flag is set on the fixer role. Fixer is itself an agent step (its own `agent.step.requested` / `completed`), so activation is just another bus round-trip.
9. **Hooks are synchronous.** `HookRegistry` callbacks return `void`, never `Promise<void>`. The library does not `await` hooks. If a hook needs async work, it fires it off out-of-band — it cannot delay or reorder the event burst returned from `start` / `deliver` / `tick`. Preserves the `(events in) → (events out)` contract and replay determinism.
10. **`tick()` is caller-driven.** Timeouts only fire when `session.tick(now)` is called. The library never calls `setTimeout` or `Date.now()` itself. Callers must either schedule `tick(Date.now())` periodically (e.g., `setInterval`) or drive it deterministically in tests. **A session whose worker is stuck will hang silently if `tick` is never called.** Document this in the README and the `Clock` docs.

**Phase → phase data flow.** Phase N produces a single resolved output via `vote.resolved`; that becomes Phase N+1's `input`. Memex context is sampled fresh on every `agent.step.requested` (per role's `execution.memex` config) — memory grows during the run, later roles see updates from earlier ones. Roles can opt out of retrieval with `execution.memex.enabled: false`.

### Phase 4 — MemEX adapter

The agent worker on the other side of the bus does not talk to MemEX directly. Two paths for memory:

- **Memex context goes out with the request.** Before emitting `agent.step.requested`, the session calls `handle.retrieve(...)` (params from `role.execution.memex`) and embeds the result in the event's top-level `memex_context` field (matching the Phase 2 vocabulary). No round-trip from the worker.
- **Memex commands come back with the response.** The worker returns `{ output, memex_commands?: MemoryCommand[] }` in `agent.step.completed`. The session emits one `memex.command.emitted` event per command, then calls `adapter.apply(cmd)`. The event sits between worker and MemEX so replay reconstructs identical state without re-running the model.

Adapter shape:

```ts
import type { MemoryCommand, MemoryItem, MemexExport, GraphState, MemoryFilter, SmartRetrievalOptions } from "@ai2070/memex";

type MemexView = "self" | "role" | "crew" | "all";

interface AgentMemexHandle {
  read(filter?: MemoryFilter): MemoryItem[];                  // wraps getItems
  retrieve(opts: SmartRetrievalOptions): MemoryItem[];        // wraps smartRetrieve
}

interface MemexAdapter {
  handleFor(agent: CrewAgent, role: CrewRole, view: MemexView): AgentMemexHandle;
  apply(cmd: MemoryCommand, ctx: { agentId: string; crewId: string; roleId: string }): void;
  exportSlice(opts: ExportOptions): MemexExport;
  importSlice(slice: MemexExport): ImportReport;
  snapshot(): GraphState;                                     // for checkpointing — toJSON-friendly
}
```

The adapter stamps every applied command with:

- `author = agent:{agentId}`
- `meta.agent_id = {agentId}`
- `meta.crew_id = {crewId}`
- `meta.role = {roleId}`
- `scope = crew:{crewId}/agent:{agentId}` (when not already set)

Reads scope by `view`:

| `view` | Filter |
|--------|--------|
| `"self"` (default) | `meta.agent_id = {agentId}` |
| `"role"` | `meta.role = {roleId}` |
| `"crew"` | `meta.crew_id = {crewId}` |
| `"all"` | no filter |

Maps directly onto `memex/API.md` §"Multi-Agent Memory Segmentation" — no new MemEX features needed.

**Hard isolation for nested crews.** A role can declare `memex.isolation: "soft" | "hard"`. `"hard"` calls `exportSlice` before the nested session starts and `importSlice` on completion (`memex/API.md` §"Hard isolation via transplant"). `"soft"` (default) shares the adapter.

**Adapter is session-scoped.** Don't share one `MemexAdapter` instance across concurrent crew sessions. The caller passes a fresh adapter (or factory) per `createCrewSession`. If shared global memory is desired, that's the adapter's internal wiring — not literal instance reuse. Documented contract: adapters hold mutable session state.

**Hooks don't write MemEX.** Only agent steps emit `MemoryCommand[]`. Hooks are observability: they see events, log/metric/trace, but cannot mutate MemEX through the library. If a hook needs to write, the caller wires it to its own MemEX handle out-of-band. (We can add `crewControl.recordMemex(...)` later if a concrete use case lands — routed through the same `memex.command.emitted` event for replay.)

### Phase 5 — Checkpointing

Snapshot = `{ session machine state, internal log offset, memex GraphState (via toJSON), timestamp, schema_version: "1.0" }`. Triggered:

- Per role's `execution.checkpoints[]` (declarative).
- Programmatically via `crewControl.checkpoint(id)` from a hook.
- Automatically at phase boundaries when `opts.autoCheckpoint = "phase"`.

`resumeCrewSession(snapshot, laterEvents, opts)` re-hydrates state and continues. Resume must produce a log that, concatenated with the pre-checkpoint log, equals what a fresh run would have produced. `checkpoint.test.ts` asserts this property.

### Phase 6 — Voting / resolution (later, but stub now)

Ship `first_valid` and `majority` working in v1 (majority dedups outputs via `canonicalize` from Phase 2). `unanimous`, `weighted_consensus`, `best_of_n` are accepted by the schema; the resolver throws `NotImplementedError`. Wire `weighted_consensus` next since the example shape uses it.

## Decisions

- **Zod v4** only. Locked.
- **Schema version `1.0`**. No backcompat shims pre-1.0 release.
- **`talk_to` / `delegate_to` / `escalate_to`** are ACL gates, not orchestration. Library validates and gates; actual delegation/messaging is the worker's or hook's job. See Phase 3 §5.
- **Streaming agent outputs** are bus events. The library emits `agent.step.requested`, observes `agent.stream.chunk` passthroughs, advances on `agent.step.completed`.
- **`agent.step.requested` is self-contained.** Carries a role snapshot (capabilities + permissions); workers don't need to load the shape out-of-band.
- **Hooks are sync.** No `Promise<void>`, no awaits inside the library. Async hook work happens out-of-band.
- **`tick()` is caller-driven.** Library never schedules its own timers. Without `tick`, timeouts never fire.
- **Duplicate terminal `deliver` is a no-op.** First `completed` / `failed` / `timed_out` per `correlationId` wins. Stream chunks always append.
- **Per-agent scoped memory** is the default (Phase 4).
- **MemEX adapter is session-scoped.** Don't share a single instance across concurrent sessions.
- **Hooks don't write MemEX.** Observability only in v1.
- **Crew id** is caller-injected. The library does not generate it — too easy to leak nondeterminism.
- **Phase order** is declaration order minus `first_input` / `final_output`.
- **Cost tracking is out of scope.** No usage/cost fields, no budget aborts. Wall-clock timeouts only.
- **Dynamic crews and `permissions.invite` are v2.** Schema accepts `invite` for forward-compat; v1 silently ignores it.
- **No L0 dependency.** L0 is the typical worker on the other side of the bus, not a dependency of this library.
- **No bus dependency.** Library accepts/emits events; caller wires transport.

## Out of scope (v1)

- Cost / token-budget tracking.
- Dynamic crew mutation (`fixer.modify_structure`, invitations).
- Built-in workers. The library does not ship a "default" L0 worker — that's a recipe in docs.
- Persistence of the event log. The session keeps a small internal log for snapshots and correlation; durable storage is the caller's job (the bus already moves through the events).
- Cross-process / distributed crew runs. The session is local; multiple processes coordinating one session is out of scope.

## What to build first (smallest useful slice)

1. Phase 1 — schemas + builder + linter. Tests against the `DEFAULT_CREW` example.
2. Phase 2 — event vocabulary + `canonicalize()`.
3. Phase 3 — state machine: `start` / `deliver` / `tick` / `cancel`. Tests use a synchronous "echo worker" stub in the test file that immediately delivers `agent.step.completed` for every `agent.step.requested`. No real bus, no L0.
4. Replay test — feed the same inbound event sequence twice, assert outbound logs are byte-identical (after `canonicalize`).

Everything past that — fixer activation, MemEX, checkpoints, voting — slots onto this skeleton without rewriting it.
