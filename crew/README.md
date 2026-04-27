# @ai2070/crew

Deterministic, replayable, bus-agnostic crew orchestration for AI agents.

A crew is a graph of typed roles and agents that takes an input through a sequence of phases and produces an output. This library does the orchestration — phase ordering, parallelism, voting, fault recovery, nested crews, scoped memory — without making any decisions about how agents actually run. Plug it onto whatever transport you want.

## What it is

A **state machine**, not an executor. The library:

- Validates a crew shape and agent counts (Zod v4) and materializes a virtual graph.
- Emits lifecycle events (`agent.step.requested`, `vote.resolved`, `fixer.invoked`, `nested.crew.completed`, …) that callers publish onto a bus.
- Consumes responses (`agent.step.completed`, `agent.step.failed`, `agent.stream.chunk`) and advances accordingly.
- Tracks parallel agents, dedups duplicate deliveries, manages timeouts, routes faults to fixer agents, and runs nested crews with optional memory isolation.

It is **not**:

- A model runner. The library never calls L0 / OpenAI / Anthropic / anything.
- A bus. There's no preferred transport (Net, EventEmitter, Kafka, in-process queue, anything works).
- A persistence layer. Snapshots are values; storage is the caller's job.
- An async framework. Hooks are sync; the loop is sync; everything is `(events in) → (events out)`.

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

## Install

```bash
npm install @ai2070/crew zod
```

Optional peers:

```bash
npm install @ai2070/memex     # for scoped per-agent memory
```

## Quick Start

```ts
import {
  CrewShapeSchema,
  CrewAgentsSchema,
  buildCrewGraph,
  createCrewSession,
  systemClock,
} from "@ai2070/crew";

// 1. A crew is two JSON blobs: the shape (roles, permissions, capabilities)
//    and the counts (how many of each role).
const shape = CrewShapeSchema.parse({
  schema_version: "1.0",
  name: "RESEARCH_CREW",
  roles: [
    {
      role: "caller",
      capabilities: { thinking_allowed: false },
      permissions: { talk_to: ["fixer"], delegate_to: [], escalate_to: [], invite: [] },
      first_input: true,
      final_output: true,
    },
    {
      role: "merc",
      capabilities: { thinking_allowed: true, model: "claude-sonnet-4-6" },
      system_prompt: "You are a research operative. Gather facts efficiently.",
      permissions: { talk_to: ["caller"], delegate_to: [], escalate_to: ["fixer"], invite: [] },
      amount: 4,
    },
    {
      role: "fixer",
      max_allowed: 1,
      capabilities: { thinking_allowed: true },
      activation: { on_fault: true, on_stall: true },
      permissions: {
        talk_to: ["merc", "caller"],
        delegate_to: ["merc"],
        escalate_to: ["caller"],
        invite: [],
      },
    },
  ],
});

const counts = CrewAgentsSchema.parse({
  schema_version: "1.0",
  name: "RESEARCH_CREW",
  agents: [
    { role: "caller", amount: 1 },
    { role: "merc", amount: 4 },
    { role: "fixer", amount: 1 },
  ],
});

// 2. Build the graph (validated, lint-checked, deterministic).
const graph = buildCrewGraph(shape, counts);

// 3. Create a session.
const session = createCrewSession({
  crewId: "research-1",
  graph,
  clock: systemClock(),
});

// 4. Drive the session against your bus. The optional `task` argument is
//    session-level metadata — emitted once on `crew.started`, captured in
//    snapshots, NOT propagated to per-step requests.
const initial = session.start("What sleep patterns do birds have?", {
  description: "Research bird sleep patterns",
});
for (const e of initial) bus.publish(e);

bus.subscribe("agent.step.completed", (e) => {
  for (const out of session.deliver(e)) bus.publish(out);
});

// Periodically tick to fire timeouts.
setInterval(() => {
  for (const out of session.tick(Date.now())) bus.publish(out);
}, 1000);
```

The bus consumer (somewhere else in your stack) listens for `agent.step.requested`, runs whatever model/prompt logic it wants (e.g. calls L0), and publishes `agent.step.completed` back. The crew session never knows.

## Mental Model

- **The shape declares the topology** — which roles exist, how they relate, who can talk/delegate/escalate to whom, what activates fault recovery.
- **The session emits requests** — when a phase starts, the session emits `agent.step.requested` for every agent in that role with a self-contained role snapshot, the input, and the optional memex context. Workers handle them however.
- **The session waits for responses** — every request has a deterministic `correlationId`. Responses match by id. Phases wait for all agents to resolve before voting.
- **Voting picks one output per phase** — `first_valid` (default), `majority`, `unanimous`, or `weighted_consensus`. The resolved value becomes the input to the next phase.
- **The cascade is one-way** — phase N+1 only sees phase N's resolved output as its `input`. The original `rootInput` only reaches the caller (entry-point role). Each role brings its own memex view and own `system_prompt`. Workers compose the model prompt from those three pieces.
- **Faults route to fixer** — if any role has `activation.on_fault`, an agent's failure spawns a fixer step whose response substitutes for the failed agent's slot in voting.
- **Nested crews are recursive** — a role can declare `nested_crew`, which causes its agents to spawn inner sessions instead of going to the bus. Memory can be soft-isolated (shared adapter) or hard-isolated (forked, merged on completion).
- **Task is session-level metadata** — `start(rootInput, task?)` accepts an optional `Task = { description: string }`. It rides on `crew.started` once and lives in the snapshot for replay/observability. It does **not** propagate onto per-step requests; it's not a substitute for the input cascade.
- **Snapshot/resume preserves it all** — `session.snapshot()` returns a serializable record of every pending step's input, role snapshot, memex context, the session-level task, and inner-session state. `resumeCrewSession(snap, laterEvents, opts)` rebuilds it.

### `system_prompt` is optional and unenforced

Each role can declare a `system_prompt: string` on the shape. It's carried in the `RoleSnapshot` embedded in `agent.step.requested` so workers can use it as the model's system message. The library does **not** validate that LLM-driven roles (`capabilities.thinking_allowed: true`) actually have one — that's a worker-side / caller-side concern. If you want strict checks, walk the parsed shape yourself before `buildCrewGraph` and reject roles that need a prompt and don't have one.

## Determinism

Everything is deterministic given the inputs:

- Same `(shape, counts, ordered inbound events)` → byte-identical outbound event log (after `canonicalize()`).
- Inbound delivery order doesn't affect outbound — votes are tallied in graph declaration order.
- `Clock` and ids are injected; the library never calls `Date.now()` or generates random data.
- `tick()` is caller-driven — timeouts only fire when the caller calls it.

This makes replay testing trivial and snapshot/resume correct by construction.

## What's in the box

| Concept | Public surface | Source |
|---|---|---|
| Schema validation | `CrewShapeSchema`, `CrewAgentsSchema` | `src/schema/` |
| Graph builder + linter | `buildCrewGraph`, `lintCrewShape` | `src/graph/` |
| Event vocabulary | `CrewEvent` discriminated union | `src/events/types.ts` |
| Canonical JSON | `canonicalize` | `src/events/canonical.ts` |
| Correlation ids | `correlationId`, `hashHex` | `src/runtime/ids.ts` |
| Clock injection | `systemClock`, `frozenClock` | `src/runtime/clock.ts` |
| Hooks | `createHookRegistry`, `HookContext` | `src/runtime/hooks.ts` |
| Session machine | `createCrewSession`, `resumeCrewSession` | `src/session/` |
| Voting | `resolveVotes` | `src/voting/resolve.ts` |
| Checkpoints | `createInMemoryCheckpointStore` | `src/checkpoint/` |
| MemEX adapter (peer) | `createMemexAdapter` | `src/memex/ai2070.ts` (subpath `@ai2070/crew/memex`) |

See **[API.md](./API.md)** for the full reference.

## Status

Every section of [PLAN.md](./PLAN.md) is implemented:

- ✅ Schemas + graph builder + linter
- ✅ Event vocabulary + canonical JSON + deterministic correlation ids
- ✅ State machine: `start` / `deliver` / `tick` / `cancel`
- ✅ Hooks (sync), permissions (ACL gates), fixer activation, nested crews
- ✅ MemEX adapter (memex_context out, memex_commands in, hard isolation)
- ✅ Snapshot / resume (with `ResumePolicy`) + `CheckpointStore`
- ✅ Voting: `first_valid`, `majority`, `unanimous`, `weighted_consensus` (equal weights)

Deferred to v2 (per plan): dynamic crews / `permissions.invite`, custom voting weight functions, `best_of_n` voting (needs scoring source), cost tracking.

## License

Apache-2.0
