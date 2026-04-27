# API Reference — `@ai2070/crew`

Complete reference for the public surface. For a higher-level overview see [README.md](./README.md). For the design document see [PLAN.md](./PLAN.md).

## Contents

- [Schemas](#schemas)
- [Graph](#graph)
- [Events](#events)
- [Canonical JSON](#canonical-json)
- [Correlation IDs](#correlation-ids)
- [Clock](#clock)
- [Hooks](#hooks)
- [Session](#session)
- [Resume](#resume)
- [Voting](#voting)
- [Checkpoints](#checkpoints)
- [Permissions](#permissions)
- [MemEX adapter](#memex-adapter)

---

## Schemas

All schemas are Zod v4. Imports resolve to `zod@^4`.

### `CrewShapeSchema`

Top-level schema for a crew topology — which roles exist, how they connect.

```ts
import { CrewShapeSchema, type CrewShape } from "@ai2070/crew";

const shape: CrewShape = CrewShapeSchema.parse(jsonInput);
```

**Required top-level fields:**

| Field | Type | Description |
|---|---|---|
| `schema_version` | `"1.0"` | Literal — must be `"1.0"` |
| `name` | `string` (≥1 char) | Crew name |
| `roles` | `CrewRoleSpec[]` (≥1) | Role definitions |

**`CrewRoleSpec`** fields:

| Field | Type | Description |
|---|---|---|
| `role` | `string` | Role name (used as id) |
| `description` | `string?` | Free-form description |
| `system_prompt` | `string?` | **Optional, unenforced.** The model's system message for this role. Carried in `RoleSnapshot`. The library does NOT check that LLM-driven roles have one — see [system_prompt is optional](#system_prompt-is-optional). |
| `capabilities` | `RoleCapabilities` | `thinking_allowed` (default `true`), `tools?: string[]`, `model?: string` |
| `permissions` | `RolePermissions` | `talk_to`, `delegate_to`, `escalate_to`, `invite` arrays; optional `modify_structure` |
| `delegation` | `{ max_depth }?` | Optional depth limit |
| `amount` | `number?` | Default agent count for this role |
| `max_allowed` | `number?` | Cap on agent count |
| `activation` | `RoleActivation?` | Fixer-style triggers — at least one of `on_fault` / `on_stall` / `on_timeout` / `on_permission_denied` must be `true` |
| `first_input` | `boolean?` | Marks the entry-point role (exactly one required) |
| `final_output` | `boolean?` | Marks the exit-point role (exactly one required) |
| `nested_crew` | `string?` | Reference to another crew shape in the registry |
| `hooks` | `LifecycleHooks?` | Hook name bindings |
| `execution` | `RoleExecution?` | `voting`, `checkpoints`, `timeout_ms`, `serial`, `memex` |

#### `system_prompt` is optional

Roles can declare a `system_prompt` on the shape; it propagates into the `RoleSnapshot` that workers receive on every `agent.step.requested`. The library does **not** require it — even for roles with `capabilities.thinking_allowed: true`. There's no schema-time error and no lint warning for missing prompts.

This is deliberate:

- Non-LLM glue roles (e.g. a `caller` with `thinking_allowed: false`) legitimately don't need a prompt.
- Workers may construct the system message from other inputs (a registry of personas, the role description, etc.).
- The crew library doesn't know what your worker's prompt assembly looks like.

If you want strict enforcement, do it in your own pre-flight check before calling `buildCrewGraph`:

```ts
const shape = CrewShapeSchema.parse(jsonInput);
for (const r of shape.roles) {
  if (r.capabilities.thinking_allowed !== false && !r.system_prompt) {
    throw new Error(`Role "${r.role}" needs a system_prompt`);
  }
}
const graph = buildCrewGraph(shape, counts);
```

**`permissions.invite`** entries can be either `"role-name"` strings or `{ role: string; consensus?: ConsensusConfig }` objects. Accepted by the schema for forward-compat with v2 dynamic crews; v1 silently ignores them.

**`activation`** must contain at least one trigger flag set to `true`. Empty `activation: {}` is rejected at parse time.

### `CrewAgentsSchema`

Per-role agent counts.

```ts
import { CrewAgentsSchema, type CrewAgents } from "@ai2070/crew";

const counts: CrewAgents = CrewAgentsSchema.parse(jsonInput);
```

| Field | Type | Description |
|---|---|---|
| `schema_version` | `"1.0"` | |
| `name` | `string` | Must match the crew shape's name |
| `agents` | `{ role: string; amount: number }[]` | Per-role overrides |

When `amount` for a role is missing, `buildCrewGraph` falls back to the role's `amount` field on the shape. `max_allowed` caps requested counts; exceeding it throws.

### `ConsensusConfigSchema`, `VotingModeSchema`

```ts
import {
  ConsensusConfigSchema,
  VotingModeSchema,
  type ConsensusConfig,
  type VotingMode,
} from "@ai2070/crew";
```

`VotingMode` is one of `"first_valid"` | `"unanimous"` | `"weighted_consensus"` | `"majority"` | `"best_of_n"`.

`ConsensusConfig`:

| Field | Type | Default |
|---|---|---|
| `mode` | `VotingMode` | `"weighted_consensus"` |
| `weight_function` | `string` | `"equal"` |
| `threshold` | `number` (0..1) | `0.66` |

### `LifecycleHooksSchema`

Optional `hooks` block per role, mapping hook stage names to hook-registry keys (strings).

```ts
{
  hooks: {
    before_role: "log-enter",
    after_agent: "track",
    on_fault: "alert",
    // ...
  }
}
```

Stages: `before_role`, `after_role`, `before_agent`, `after_agent`, `on_fault`, `on_stall`, `on_timeout`, `on_permission_denied`, `on_checkpoint`.

---

## Graph

### `buildCrewGraph(shape, counts, registry?, visited?)`

Materializes a `CrewGraph` from a parsed shape and counts. Pure, deterministic, no I/O.

```ts
import { buildCrewGraph, type CrewGraph } from "@ai2070/crew";

const graph: CrewGraph = buildCrewGraph(shape, counts, { INNER: innerShape });
```

**Parameters:**

| Param | Type | Description |
|---|---|---|
| `shape` | `CrewShape` | Parsed top-level shape |
| `counts` | `CrewAgents` | Parsed agent counts |
| `registry` | `Record<string, CrewShape>?` | Lookup for `nested_crew` references |
| `visited` | `ReadonlySet<string>?` | Internal — used during recursive expansion for cycle detection |

**Throws** when:
- `shape.name !== counts.name`
- A role in `counts.agents` doesn't exist in the shape
- Requested count exceeds a role's `max_allowed`
- `lintCrewShape` reports any errors
- A nested-crew chain forms a cycle (`A → B → A` or self-cycle)

**Returns** `CrewGraph`:

```ts
interface CrewGraph {
  name: string;
  roles: Map<RoleId, CrewRole>;          // declaration order
  agents: CrewAgent[];                    // declaration order, ids "{role}-{i+1}"
}

interface CrewAgent {
  id: AgentId;                            // e.g. "merc-1"
  role: RoleId;
  nestedCrew?: CrewGraph;                 // for roles with nested_crew + registry hit
}
```

### `lintCrewShape(shape)`

Semantic checks Zod can't express.

```ts
import { lintCrewShape, type LintResult } from "@ai2070/crew";

const { errors, warnings }: LintResult = lintCrewShape(shape);
```

**Errors** (block `buildCrewGraph`):

| Code | Cause |
|---|---|
| `duplicate_role` | Two roles share the same name |
| `missing_first_input` / `multiple_first_input` | Need exactly one |
| `missing_final_output` / `multiple_final_output` | Need exactly one |
| `unknown_role_reference` | A `talk_to` / `delegate_to` / `escalate_to` / `invite[].role` references an undeclared role |
| `self_escalation` | Role escalates to itself |
| `unsupported_voting_mode` | Role uses `best_of_n` (not implemented in v1) |
| `unsupported_weight_function` | Role uses `weighted_consensus` with a non-`"equal"` weight function |

**Warnings** (non-blocking):

| Code | Cause |
|---|---|
| `delegate_without_talk` | Role can `delegate_to` a peer it cannot `talk_to` |

---

## Events

### `CrewEvent` discriminated union

```ts
import type { CrewEvent } from "@ai2070/crew";
```

**Outbound** (emitted by the session):

| Type | Notable fields |
|---|---|
| `crew.started` | `crewId`, `rootInput`, `task?` |
| `role.entered` | `roleId` |
| `agent.step.requested` | `correlationId`, `agentId`, `roleId`, `input`, `memex_context?`, `role` (RoleSnapshot), `timeoutMs?` |
| `fixer.invoked` | `agentId`, `reason: "fault" \| "stall" \| "timeout" \| "permission_denied"` |
| `permission.denied` | `from`, `to`, `action: "talk_to" \| "delegate_to" \| "escalate_to"` |
| `vote.resolved` | `roleId`, `mode`, `resolved` |
| `checkpoint.taken` | `checkpointId` |
| `memex.command.emitted` | `agentId`, `command: MemoryCommand` |
| `nested.crew.started` | `agentId`, `nestedName` |
| `nested.crew.completed` | `agentId`, `output` |
| `crew.aborted` | `reason: "cancelled" \| "timeout"` |
| `crew.completed` | `finalOutput` |

**Inbound** (delivered to the session):

| Type | Notable fields |
|---|---|
| `agent.step.completed` | `correlationId`, `output`, `memex_commands?: MemoryCommand[]`, `fault?`, `stalled?` |
| `agent.step.failed` | `correlationId`, `error: { name, message }` |
| `agent.step.timed_out` | `correlationId` (caller-injected; the session also self-emits via `tick`) |
| `agent.stream.chunk` | `correlationId`, `chunk` (passthrough — not state-advancing) |

All events have `ts: number`.

### `RoleSnapshot`

Self-contained role projection embedded in `agent.step.requested`. Workers don't need access to the crew shape — the snapshot has everything they need.

```ts
interface RoleSnapshot {
  name: string;
  description?: string;
  system_prompt?: string;          // see "system_prompt is optional" above
  capabilities?: { model?: string; tools?: string[]; thinking_allowed?: boolean };
  permissions: { talk_to: string[]; delegate_to: string[]; escalate_to: string[] };
}
```

### `Task`

Session-level metadata that describes the crew's purpose. Set at `start()`, emitted once on `crew.started`, captured in the snapshot. **Does not propagate** to per-step `agent.step.requested` events — each phase only sees the upstream phase's resolved output as its `input`. The caller (entry-point role) receives `rootInput` (typically derived from the task) and is responsible for formulating prompts the next phase sees.

```ts
interface Task {
  description: string;
  // Future: priority, deadline, success_criteria, intent_id (memex Intent linkage), parents.
}
```

Use `Task` for observability (logs, traces), replay metadata, and (later) cross-crew coordination via memex Intent. It is NOT a per-agent prompt — that's what `role.system_prompt` plus the cascading `input` are for.

### `MemoryCommand`, `MemoryItemShape`, `EdgeShape`

Structural mirrors of `@ai2070/memex`'s strict types — defined locally so the root export surface stays decoupled. Memex's strict types are structurally assignable, so workers using `@ai2070/memex/createMemoryItem` etc. pass real commands through without casting.

```ts
import type { MemoryCommand, MemoryItemShape } from "@ai2070/crew";

const cmd: MemoryCommand = {
  type: "memory.create",
  item: {
    id: "i1",
    scope: "research/birds",
    kind: "observation",
    content: { species: "owl", finding: "nocturnal" },
    author: "agent:merc-1",
    source_kind: "observed",
    authority: 0.9,
  },
};
```

### `isInboundEvent(event)`, `isTerminalInbound(event)`

Type guards for splitting the union.

```ts
import { isInboundEvent, isTerminalInbound } from "@ai2070/crew";

if (isTerminalInbound(e)) {
  // e is one of completed | failed | timed_out
}
```

---

## Canonical JSON

### `canonicalize(value)`

Stable JSON serialization for hashing, dedup, and replay assertions.

```ts
import { canonicalize } from "@ai2070/crew";

canonicalize({ a: 1, b: 2 }) === canonicalize({ b: 2, a: 1 });   // true
canonicalize(-0) === "0";                                          // -0 normalized
canonicalize([1, , 3]) === "[1,null,3]";                           // sparse → null
```

**Behavior:**

- Object keys sorted alphabetically.
- `-0` → `0`.
- Sparse-array holes serialized as `null` (matches `JSON.stringify`).
- `NaN` / `Infinity` / `-Infinity` throw.
- `undefined`, `bigint`, `function`, `symbol` throw at the top level.
- `undefined` in arrays → `null`. `undefined` in object values → key skipped.
- Circular references throw.

---

## Correlation IDs

### `correlationId({ crewId, roleId, agentId, attempt })`

Deterministic FNV-1a 64-bit hash over `canonicalize` of the parts. Same inputs always produce the same id.

```ts
import { correlationId } from "@ai2070/crew";

const cid = correlationId({
  crewId: "research-1",
  roleId: "merc",
  agentId: "merc-1",
  attempt: 1,
});
// 16-char hex string
```

### `hashHex(input)`

Lower-level FNV-1a helper; takes a string, returns 16-char hex.

```ts
import { hashHex } from "@ai2070/crew";
hashHex("hello"); // deterministic
```

Non-cryptographic. Suitable for in-process correlation, not for security.

---

## Clock

The library never calls `Date.now()`. All timestamps come from an injected `Clock`.

### `systemClock()`

```ts
import { systemClock } from "@ai2070/crew";

const clock = systemClock();
clock.now(); // === Date.now()
```

### `frozenClock(initial?)`

Test-friendly mutable clock.

```ts
import { frozenClock } from "@ai2070/crew";

const clock = frozenClock(0);
clock.now();           // 0
clock.advance(1000);
clock.now();           // 1000
clock.set(5000);
clock.now();           // 5000
```

Returns `MutableClock` (a `Clock` plus `advance` / `set`).

---

## Hooks

### `createHookRegistry(hooks)`

Maps hook-name strings (declared on roles via `role.hooks`) to functions.

```ts
import { createHookRegistry, type HookFn } from "@ai2070/crew";

const trace: HookFn = (ctx) => console.log(ctx.stage, ctx.role.role, ctx.agent?.id);

const hooks = createHookRegistry({
  trace,
  alert: (ctx) => sendAlert(ctx),
});
```

The registry refuses prototype-chain lookups (`__proto__`, `constructor`, etc.) and non-function values.

### `HookFn(ctx)`

Synchronous. No `Promise<void>`. The library never awaits hooks. If a hook needs async work, fire it off out-of-band (e.g. publish an event of your own to a different bus).

```ts
type HookFn = (ctx: HookContext) => void;

interface HookContext {
  crewId: string;
  stage: HookStage;
  role: CrewRole;
  agent?: CrewAgent;
  input?: unknown;
  output?: unknown;
  fault?: boolean;
  stalled?: boolean;
  control: CrewControl;
}
```

### `CrewControl`

Hook-side surface for emitting state into the current event burst.

```ts
interface CrewControl {
  requestAction(req: { from: AgentId; to: AgentId; action: GatedAction }): boolean;
  checkpoint(id: string): void;
}
```

- `requestAction` validates the action against the source role's permissions; if denied, emits `permission.denied` and may activate fixer if `on_permission_denied` is set on a role.
- `checkpoint(id)` emits `checkpoint.taken { checkpointId: id }` into the current burst.

### Hook stages

| Stage | When |
|---|---|
| `before_role` | After `role.entered`, before any agent.step.requested for the phase |
| `after_role` | After `vote.resolved`, before phase advance |
| `before_agent` | Before each `agent.step.requested` |
| `after_agent` | After each terminal event (including fixer responses) |
| `on_fault` | When a non-fixer agent's response carries `fault: true` |
| `on_stall` | When a response carries `stalled: true` |
| `on_timeout` | When `tick` (or a caller-injected timed_out event) expires a request |
| `on_permission_denied` | When `requestAction` is denied (fires on the source role) |
| `on_checkpoint` | When a declarative checkpoint fires (one per name in `execution.checkpoints`) |

---

## Session

### `createCrewSession(opts)`

```ts
import { createCrewSession, type CrewSession } from "@ai2070/crew";

const session: CrewSession = createCrewSession({
  crewId: "research-1",
  graph,
  clock: systemClock(),
  hooks,                       // optional
  memex,                       // optional
  defaultTimeoutMs: 30000,     // optional, applied when role.execution.timeout_ms is unset
  resumePolicy: "abort",       // optional
  autoCheckpoint: "phase",     // optional — emits checkpoint.taken at every phase boundary
});
```

**`CreateCrewSessionOpts`:**

| Field | Type | Default | Notes |
|---|---|---|---|
| `crewId` | `string` | — | Caller-injected for replay determinism |
| `graph` | `CrewGraph` | — | From `buildCrewGraph` |
| `clock` | `Clock` | — | Inject `systemClock()` or `frozenClock()` |
| `hooks` | `HookRegistry?` | none | |
| `memex` | `MemexAdapter?` | none | |
| `defaultTimeoutMs` | `number?` | none | When unset, requests have no timeout unless the role specifies one |
| `resumePolicy` | `ResumePolicy?` | `"abort"` | See [Resume](#resume) |
| `autoCheckpoint` | `"phase"?` | none | Emits `phase:{role}:{phaseIndex}` checkpoint after every `vote.resolved` |

### `CrewSession`

```ts
interface CrewSession {
  start(rootInput: unknown, task?: Task): CrewEvent[];
  deliver(event: CrewEvent): CrewEvent[];
  tick(now: number): CrewEvent[];
  cancel(reason: "cancelled" | "timeout"): CrewEvent[];
  status(): CrewStatus;
  pendingRequests(): readonly AgentStepRequest[];
  pendingDetails(): readonly AgentStepDetail[];
  snapshot(): CrewSnapshot;
  requestAction(req: ActionRequest): RequestActionResult;
}
```

**`start(rootInput, task?)`** — transitions from `"idle"` to `"awaiting_responses"`. Emits `crew.started` (with `task` field if provided) + `role.entered` (caller) + `agent.step.requested` (caller). The `task` is stashed at the session level — present on `crew.started` and in the snapshot, NOT on per-step requests. Throws if called twice.

**`deliver(event)`** — feeds an inbound event back. Returns the burst of outbound events emitted in response. Idempotent on duplicate terminals (first wins). `agent.stream.chunk` is a no-op (chunks pass through the bus, not the session). Returns `[]` after `cancel` or `complete`.

**`tick(now)`** — drives internal timeout scheduling. Pass `Date.now()` (in production) or a synthetic value (in tests). For each in-flight request whose `requested.ts + timeoutMs ≤ now`, the session emits `agent.step.timed_out` and treats it as a fault. **Caller MUST call `tick` periodically — without it, timeouts never fire.**

**`cancel(reason)`** — drains queue, emits `crew.aborted`, transitions to `"aborted"`. Subsequent calls (and subsequent `deliver`s) are no-ops.

**`status()`** — `"idle" | "awaiting_responses" | "completed" | "aborted"`.

**`pendingRequests()`** — outstanding requests across this session and any in-flight nested sessions (recursive).

**`pendingDetails()`** — same set, with `input`, `roleSnapshot`, and `memexContext` for each. Used by re-emit resume.

**`snapshot()`** — full serializable session state. See [Resume](#resume).

**`requestAction(req)`** — `{ allowed: boolean; events: CrewEvent[] }`. ACL gate only — the session does not spawn delegated steps. The actual delegation/messaging is the caller's job.

### Permissions enforcement

```ts
const { allowed, events } = session.requestAction({
  from: "merc-1",
  to: "fixer-1",
  action: "talk_to",
});
if (!allowed) {
  // events contains permission.denied (and possibly fixer.invoked + agent.step.requested
  // if a role has activation.on_permission_denied set)
}
for (const e of events) bus.publish(e);
```

---

## Resume

### `resumeCrewSession(snapshot, laterEvents, opts)`

Rebuild a session from a snapshot, replay any inbound events that arrived after the snapshot, then apply `ResumePolicy` to whatever requests remain unresolved.

```ts
import { resumeCrewSession, type ResumeResult } from "@ai2070/crew";

const { session, events }: ResumeResult = resumeCrewSession(snapshot, laterEvents, {
  crewId: "research-1",
  graph,
  clock: systemClock(),
  memex,
  resumePolicy: "re-emit-request",
});
for (const e of events) bus.publish(e);
```

**`ResumePolicy`** — what to do with requests that were pending at snapshot time and have no terminal in `laterEvents`:

| Value | Behavior |
|---|---|
| `"abort"` *(default)* | Throw. Forces the caller to make an explicit choice. |
| `"re-emit-request"` | Emit a fresh `agent.step.requested` for each pending. Same `correlationId` (idempotency on the bus). Carries the original `input`, `roleSnapshot`, and `memex_context` captured at original emit time. |
| `"treat-as-failed"` | Synthesize an `agent.step.failed` for each pending. The phase advances (and may activate fixer). |

### `CrewSnapshot`

Fully serializable. Captures: machine status, phase index, current input, the session-level `task` (if set at `start()`), per-agent attempt counters, the current phase's pending requests (with input, role snapshot, and memex context for each), processed terminals, fixer-step substitution map, nested handles (recursive snapshots, plus inner adapter state when hard-isolated), and timestamp.

`toJSON(snapshot)` is unnecessary — the snapshot is plain serializable data.

---

## Voting

### `resolveVotes(votes, config)`

```ts
import { resolveVotes, type VoteEntry } from "@ai2070/crew";

const votes: VoteEntry[] = [
  { agentId: "merc-1", output: "yes" },
  { agentId: "merc-2", output: "yes" },
  { agentId: "merc-3", output: "no" },
];
resolveVotes(votes, "weighted_consensus");
// → "yes"  (2/3 = 0.66, default threshold 0.66)
```

**`config`** is either a `VotingMode` string (mode-only with defaults) or a full `ConsensusConfig` (mode + `weight_function` + `threshold`).

**Implemented modes:**

| Mode | Behavior |
|---|---|
| `first_valid` *(default)* | First non-faulted, non-stalled output in declaration order |
| `majority` | Most common output (canonicalized for dedup) |
| `unanimous` | All valid outputs equal — returns the value, else `undefined` |
| `weighted_consensus` | Bucket by canonical output; first bucket whose weight share ≥ threshold wins. Equal weights only in v1. |

**Throws `NotImplementedError`:**

- `best_of_n` — needs per-output scoring not present in `VoteEntry`
- `weighted_consensus` with `weight_function !== "equal"`

`lintCrewShape` catches these at build time.

`undefined` outputs are accepted as a vote — they cluster into a sentinel bucket so canonical-key collisions don't crash the resolver.

---

## Checkpoints

The library does not auto-persist snapshots. The pattern:

1. Hook calls `crewControl.checkpoint(id)` (or set `autoCheckpoint: "phase"` or declare `role.execution.checkpoints[]`).
2. Session emits `checkpoint.taken { checkpointId: id }`.
3. Caller observes the event, calls `session.snapshot()`, and stores the result wherever it wants.

### `createInMemoryCheckpointStore()`

In-memory `CheckpointStore` for tests / single-process apps.

```ts
import { createInMemoryCheckpointStore, type CheckpointStore } from "@ai2070/crew";

const store: CheckpointStore = createInMemoryCheckpointStore();

bus.subscribe("checkpoint.taken", (e) => {
  store.put(e.checkpointId, session.snapshot());
});

// later …
const snap = store.get("phase:merc:1");
if (snap) {
  const { session: resumed } = resumeCrewSession(snap, [], opts);
  // …
}
```

**`CheckpointStore`:**

```ts
interface CheckpointStore {
  put(id: string, snapshot: CrewSnapshot): void;
  get(id: string): CrewSnapshot | undefined;
  list(): string[];
  delete(id: string): void;
}
```

---

## Permissions

### `checkPermission(graph, fromAgentId, toAgentId, action)`

Pure ACL check. Used internally by `requestAction`; exposed for callers who want to consult the rules without emitting events.

```ts
import { checkPermission } from "@ai2070/crew";

checkPermission(graph, "merc-1", "fixer-1", "talk_to"); // boolean
```

Returns `false` for unknown agents, missing roles, or disallowed actions.

---

## MemEX adapter

Subpath module: `@ai2070/crew/memex`. Requires `@ai2070/memex@^0.12` as a peer.

### `createMemexAdapter(opts)`

```ts
import { createMemexAdapter } from "@ai2070/crew/memex";
import { createGraphState } from "@ai2070/memex";

const adapter = createMemexAdapter({
  crewId: "research-1",
  memState: createGraphState(),     // optional starting state
  // intentState, taskState — also optional
});
```

Returns a `MemexAdapter` that:

- **Reads** are scoped per agent by `view`: `"self"` (default), `"role"`, `"crew"`, `"all"`. Caller-supplied filters CANNOT override the view scope (defense against scope-escape).
- **Writes** (`memory.create`) are stamped with `author = agent:{id}`, `meta.{agent_id, crew_id, role}`, and a default `scope = crew:{crewId}/agent:{agentId}` if the worker didn't set one. Other commands pass through.
- **Hard isolation** — `fork(crewId)` deep-clones memory state into a sibling adapter for nested crews with `memex.isolation: "hard"`. `restoreFrom(snapshot, crewId)` rebuilds a forked adapter from a captured snapshot during resume. `exportAll()` returns a slice covering all memories/intents/tasks (the parent calls `importSlice` on it when the nested crew completes).
- **Snapshot** — `snapshot()` returns the underlying `GraphState` (used by snapshot/resume to capture hard-isolated nested adapter state).

### Wiring memex into a session

Per-role config in `execution.memex`:

```ts
{
  role: "merc",
  // ...
  execution: {
    memex: {
      enabled: true,
      view: "self",          // "self" | "role" | "crew" | "all"
      isolation: "soft",     // "soft" (shared adapter) | "hard" (forked)
      retrieval: {           // SmartRetrievalOptions, passed to memex's smartRetrieve
        budget: 4096,
        weights: { authority: 0.5, importance: 0.5 },
        costFn: (item) => JSON.stringify(item.content).length,
      },
    },
  },
}
```

When `enabled` and `retrieval` are set, the session calls `handle.retrieve(retrieval)` before emitting `agent.step.requested` and embeds the result in the event's top-level `memex_context` field.

When the worker returns `agent.step.completed { memex_commands: [...] }`, the session emits one `memex.command.emitted` event per command (so replay reconstructs identical state without re-running the model) and applies each via `adapter.apply(cmd, ctx)`.

### Shape: `MemexAdapter`

```ts
interface MemexAdapter {
  handleFor(agent: CrewAgent, role: CrewRole, view: MemexView): AgentMemexHandle;
  apply(cmd: MemoryCommand, ctx: MemexStampContext): void;
  exportSlice(opts: unknown): unknown;
  importSlice(slice: unknown): unknown;
  snapshot(): unknown;
  fork(crewId: string): MemexAdapter;
  restoreFrom(snapshot: unknown, crewId: string): MemexAdapter;
  exportAll(): unknown;
}
```

The interface uses opaque types (`unknown`) for memex-derived inputs and outputs so the root `@ai2070/crew` surface stays decoupled from `@ai2070/memex`. The `@ai2070/crew/memex` factory provides the strict-typed implementation. Callers who want strict types on read results can cast: `handle.read() as MemoryItem[]`.

---

## See also

- [README.md](./README.md) — overview, install, quick start
- [PLAN.md](./PLAN.md) — design document; the spec the implementation tracks
