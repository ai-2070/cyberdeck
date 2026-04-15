# MemEX — Structured Memory for AI Agents

Multi-session continuity for AI systems.

MemEX stores beliefs, evidence, conflicts, and updates -- not just retrieved text. It gives agents a continuous belief state across sessions instead of fragmented chat logs.

## The Problem

Every chat session starts from scratch. Memory systems try to fix this by appending text and summarizing when it gets long. But that loses:

- **Why** something is believed (provenance)
- **How much** to trust it (authority, conviction)
- **What conflicts** with it (contradictions)
- **Whether** it's still relevant (decay)
- **Where** it came from (source attribution)

Most systems conflate "I can retrieve it" with "I know it." Retrieval is not memory. MemEX separates recall (a tool problem) from belief state (a knowledge problem).

## What MemEX Does

MemEX is a typed, scored, provenance-tracked graph. Each memory item carries:

- A **kind** -- what it is (observation, assertion, hypothesis, derivation, simulation, policy, trait)
- A **source_kind** -- how it got here (user-stated, observed, inferred, imported)
- Three **scores** -- authority (trust), conviction (author confidence), importance (attention priority)
- **Parents** -- what items it was derived from, forming provenance chains
- **Edges** -- typed relationships to other items (supports, contradicts, supersedes, alias)

This means the system can:

- Carry forward beliefs across sessions, not just text
- Track what was observed vs inferred vs assumed
- Surface contradictions instead of silently overwriting
- Explain *why* it believes something (provenance tree)
- Decay stale context while preserving stable knowledge
- Recognize that two observations refer to the same entity

## Where MemEX Fits

MemEX is the structured memory layer in a larger stack. It doesn't replace your other tools -- it gives them something better to read from and write to.

```
┌─────────────────────────────────────────────────┐
│                  Agent / App                     │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │  Chat     │  │ Working  │  │   Cognition   │  │
│  │  Window   │  │ Memory   │  │   Layer       │  │
│  │ (sliding) │  │(scratch) │  │  (thinking)   │  │
│  └────┬─────┘  └────┬─────┘  └──────┬────────┘  │
│       │              │               │            │
│       └──────────────┼───────────────┘            │
│                      │                            │
│              ┌───────▼────────┐                   │
│              │     MemEX      │                   │
│              │  (this library) │                   │
│              └───────┬────────┘                   │
│                      │                            │
│         ┌────────────┼────────────┐               │
│         │            │            │               │
│   ┌─────▼─────┐ ┌───▼───┐ ┌─────▼─────┐         │
│   │  Vector   │ │ Text  │ │ Event     │         │
│   │  Search   │ │ Search│ │ Store     │         │
│   └───────────┘ └───────┘ └───────────┘         │
└─────────────────────────────────────────────────┘
```

### How the pieces connect

**Chat window (sliding context)** -- the current conversation. As messages flow, the agent extracts observations, assertions, and preferences and writes them to MemEX. The chat window is ephemeral; MemEX is where things persist.

**Working memory (scratchpad)** -- short-lived, high-importance items the agent is actively reasoning about. These live in MemEX with `kind: "hypothesis"` or `kind: "assumption"` and high `importance`. After processing, their importance decays and they settle into long-term memory.

**Vector / text search** -- MemEX stores structured items, not embeddings. Search tools subscribe to MemEX lifecycle events and maintain their own indexes. Search indexes are derived from MemEX, not the other way around.

**Cognition layer** -- uses `getScoredItems` and `smartRetrieve` to build its thinking queue. Writes back inferred items, resolved contradictions, and updated scores. The agent prioritizes thinking using authority, conviction, and importance.

**Event store** -- the append-only command log. MemEX emits lifecycle events that get persisted. On restart, `replayFromEnvelopes` rebuilds the graph from the log.

MemEX is the system of record. It does not replace retrieval systems -- it governs them. Vector search and keyword search are recall tools; MemEX is the epistemic coordination layer that decides what matters, what conflicts, and what to include in context. The library itself is pure TypeScript with a single runtime dependency (`uuidv7`). Storage, search, and bus integration belong in the service layer above.

### What changes in agent behavior

Without MemEX, an agent:
- Forgets between sessions, or retrieves flat text with no trust signal
- Can't tell if something was observed, inferred, or assumed
- Silently overwrites old beliefs with new ones
- Can't explain why it believes something
- Treats everything as equally important

With MemEX, an agent:
- Carries forward a structured belief state across sessions
- Knows the difference between an observation and a hypothesis
- Surfaces contradictions instead of hiding them
- Can trace any belief back to its evidence chain
- Prioritizes what to think about based on importance and uncertainty
- Lets stale context fade while stable knowledge persists

## Install

```bash
npm install @ai2070/memex
```

## Quick Start

```ts
import {
  createGraphState,
  createMemoryItem,
  applyCommand,
  getItems,
  getScoredItems,
  smartRetrieve,
} from "@ai2070/memex";

// create an empty graph
let state = createGraphState();

// add an observation
const obs = createMemoryItem({
  scope: "user:laz/general",
  kind: "observation",
  content: { key: "login_count", value: 42 },
  author: "agent:monitor",
  source_kind: "observed",
  authority: 0.9,
  importance: 0.7,
});

const result = applyCommand(state, { type: "memory.create", item: obs });
state = result.state;

// add a hypothesis derived from the observation
const hyp = createMemoryItem({
  scope: "user:laz/general",
  kind: "hypothesis",
  content: { key: "is_power_user", value: true },
  author: "agent:reasoner",
  source_kind: "agent_inferred",
  parents: [obs.id],
  authority: 0.4,
  conviction: 0.7,
  importance: 0.8,
});

state = applyCommand(state, { type: "memory.create", item: hyp }).state;

// query with filters
const recent = getItems(state, {
  or: [{ kind: "observation" }, { kind: "assertion" }],
  range: { authority: { min: 0.5 } },
  created: { after: Date.now() - 86400000 },
});

// scored retrieval with time decay
const ranked = getScoredItems(
  state,
  {
    authority: 0.5,
    conviction: 0.3,
    importance: 0.2,
    decay: { rate: 0.1, interval: "day", type: "exponential" },
  },
  { pre: { scope: "user:laz/general" }, limit: 10 },
);

// smart retrieval: decay + contradiction surfacing + diversity + budget
const context = smartRetrieve(state, {
  budget: 4096,
  costFn: (item) => JSON.stringify(item.content).length,
  weights: {
    authority: 0.5,
    importance: 0.5,
    decay: { rate: 0.1, interval: "day", type: "exponential" },
  },
  filter: { scope: "user:laz/general" },
  contradictions: "surface",
  diversity: { author_penalty: 0.3 },
});
```

## Core Concepts

### Memory Items

Not everything is a "fact." A `MemoryItem` can be an observation, an assertion, an assumption, a hypothesis, a derivation, a simulation, a policy, or a trait. The `kind` field says what it *is*; the `source_kind` field says how it *got here*.

### Three Scores

| Score | Question | Range |
|-------|----------|-------|
| `authority` | How much should the system trust this? | 0..1 |
| `conviction` | How sure was the author? | 0..1 |
| `importance` | How much attention does this need right now? (salience) | 0..1 |

These are orthogonal. A hypothesis can be high-importance (matters a lot) but low-authority (not yet verified).

### Time Decay

Scores decay over time at query time -- the stored values are not mutated. Configure decay per query:

```ts
{ rate: 0.1, interval: "day", type: "exponential" }
```

Three types: **exponential** (smooth curve, never zero), **linear** (straight to zero), **step** (drops at interval boundaries). You can also filter out items that have decayed below a threshold.

### Provenance

Items can declare **parents** -- the items they were derived or inferred from. This creates provenance chains that let the system explain *why* it believes something:

```ts
getSupportSet(state, claimId)
// -> [claim, parent1, parent2, grandparent1] -- everything that justifies this claim
```

If a parent is retracted, `getStaleItems` finds orphaned children. `cascadeRetract` removes the entire dependency chain.

### Contradictions

When two items conflict, they can be linked with a `CONTRADICTS` edge. At retrieval time:

- `contradictions: "filter"` -- keep the higher-scoring side (clean context)
- `contradictions: "surface"` -- keep both, flagged with `contradicted_by` (agent reasoning)

Contradictions can be resolved: `resolveContradiction` creates a `SUPERSEDES` edge and lowers the loser's authority.

### Identity

Two observations of the same entity can be aliased: `markAlias` creates bidirectional `ALIAS` edges. `getAliasGroup` returns the full identity group via transitive closure.

### Edges

Typed relationships between items:

| Edge | Meaning |
|------|---------|
| `DERIVED_FROM` | Relationship discovered after creation |
| `CONTRADICTS` | Two items assert conflicting things |
| `SUPPORTS` | Evidence for another item |
| `ABOUT` | References another item |
| `SUPERSEDES` | Replaces another item (conflict resolution) |
| `ALIAS` | Same entity, different observations |

### Events

Three categories, all under `namespace: "memory"`:

- **Commands** (imperative): `memory.create`, `memory.update`, `memory.retract`, `edge.create`, `edge.update`, `edge.retract`
- **Lifecycle** (past tense): `memory.created`, `memory.updated`, `memory.retracted`, `edge.created`, `edge.updated`, `edge.retracted`
- **State**: `state.memory`, `state.edge`

Commands go in, lifecycle events come out of the reducer, state events are full snapshots for downstream consumers.

### Immutability

`applyCommand` never mutates input state. It returns a new `GraphState` and an array of lifecycle events. History is in the append-only event log; `GraphState` is always the latest snapshot.

## Design Philosophy

Every system encodes assumptions about truth, knowledge, and time -- whether it acknowledges them or not. MemEX makes those assumptions explicit.

| Question | Typical system | MemEX |
|----------|---------------|-------|
| What is knowledge? | Similar text (vectors) or structured facts (SQL) | Beliefs with provenance, confidence, and conflict |
| What exists? | Documents, rows | Observations, hypotheses, derivations, policies, traits |
| Is truth binary? | Yes (stored or not) | No -- graded by authority, conviction, and importance |
| Does knowledge decay? | No (or manually pruned) | Yes -- query-time decay, configurable per retrieval |
| What about contradictions? | Overwrite or ignore | Represent, carry, and optionally resolve |

Most memory systems compress and resolve -- they produce a single clean narrative. MemEX preserves and represents -- it maintains a field of competing claims that a reasoning layer can interpret. The graph is the pre-answer belief state, not the final answer.

This is a deliberate architectural choice. MemEX is not a thinking system. It is a substrate that makes thinking systems possible. Storage, search, and cognition belong above the library. MemEX provides the structured epistemic state they operate on.

Vector search tells you what is similar. MemEX tells you what you believe.

## Three Graphs

MemEX contains three logical graphs in one package. Use what you need:

| Graph | Purpose | Core type | Namespace |
|-------|---------|-----------|-----------|
| **Memory** | Epistemic state -- beliefs, evidence, contradictions | `MemoryItem` | `"memory"` |
| **Intent** | Goals and objectives | `Intent` | `"intent"` |
| **Task** | Units of work tied to intents | `Task` | `"task"` |

All three follow the same pattern: commands → reducer → lifecycle events. They cross-reference by ID:

```ts
// intent links to memory items that motivated it
const intent = createIntent({ label: "find_kati", root_memory_ids: [obs.id], ... });

// sub-intent decomposes a parent goal
const sub = createIntent({ label: "check_financials", parent_id: intent.id, ... });

// task links to its parent intent and memory items it consumes/produces
const task = createTask({ intent_id: intent.id, input_memory_ids: [obs.id], ... });

// subtask breaks a task into steps
const step = createTask({ intent_id: intent.id, parent_id: task.id, action: "parse_profile", ... });

// after task completes, memory items link back
createMemoryItem({ ..., intent_id: intent.id, task_id: task.id });
```

## The Loop

The three graphs form a continuous cycle:

```
    ┌─────────────────────────────────────────┐
    │                                         │
    ▼                                         │
 Memory ──────► Intent ──────► Task ──────────┘
 (belief)       (direction)    (execution)
    │               │              │
    │  something    │  spawns      │  produces
    │  important    │  actionable  │  new memory
    │  or uncertain │  steps       │  (results,
    │  appears      │              │   failures,
    │               │              │   observations)
    └───────────────┘              │
         updates belief            │
         state with new ◄──────────┘
         evidence
```

1. **Memory produces intents** — an important or uncertain item surfaces, triggering a goal
2. **Intents spawn tasks** — the goal breaks into actionable steps
3. **Tasks produce new memory** — results, observations, and failures write back as memory items
4. **Memory updates belief state** — new evidence resolves contradictions, reinforces or decays existing beliefs

Most AI systems mix these together: goals hidden in prompts, tasks implicit in code, memory as text blobs. MemEX separates them:

| Layer | Responsibility |
|-------|---------------|
| Memory | What is believed |
| Intent | What is wanted |
| Task | What is done |

Each layer has its own types, commands, reducer, and query — but they reference each other by ID and share the same event envelope pattern. The separation is what makes the loop auditable: you can trace any belief back to the task that produced it, the intent that motivated it, and the evidence it was based on.

## Cognitive Transfer

The three graphs together form a complete cognitive state that can be serialized, transferred, and resumed by another agent.

```
Agent A → Agent B:

  Memory export  (what I know)
+ Intent export  (what I want)
+ Task export    (what I've tried, what worked, what failed)
= Complete cognitive state transferred
```

This isn't just data migration. The receiving agent inherits:

- **Context** — the belief state (observations, hypotheses, contradictions)
- **Direction** — active goals and their priorities
- **Progress** — which approaches were tried, which failed, which are still running

The agent picks up where the other left off. It doesn't re-derive context from scratch. It doesn't retry failed approaches. It continues.

### The vector model

Think of the three graphs as a cognitive vector:

| Component | Role | Analogy |
|-----------|------|---------|
| **Memory** | Origin | Starting point in state space — what is known |
| **Intent** | Magnitude | How much energy is allocated — priority and importance |
| **Task** | Direction | Which approaches have been tried — path through solution space |

Transferring cognition between agents is transferring this vector. The receiving agent starts from the same origin (memory), pursues the same goals with the same energy (intent), and avoids the same dead ends (task history).

This is what `exportSlice` / `importSlice` enables at the library level. The transport layer (network, bus, file) is outside the library; MemEX provides the serializable structure.

### What transplant enables

| Pattern | How it works |
|---------|-------------|
| **Safe delegation** | Export a slice to a sub-agent. It operates on its own copy. Merge results back append-only -- no risk of corrupting the main graph. |
| **Parallel reasoning** | Fork belief state into multiple slices. Run different reasoning paths independently. Compare outcomes before merging. |
| **Reproducibility** | Event logs + deterministic slices mean any state can be replayed, audited, or debugged after the fact. |
| **State mobility** | Memory is not tied to one runtime. Export, serialize, move between agents or machines, rehydrate anywhere. |

Memory is no longer a local resource. It is portable belief.

## Features

**Memory graph:**
- Full query algebra: `and`, `or`, `not`, `range`, `ids`, `scope_prefix`, `parents` (includes/count), `intent_id`, `task_id`, `meta` (dot-path), `meta_has`, `created` (time range), `decay` (freshness filter)
- Multi-sort with tiebreakers (authority, conviction, importance, recency)
- Configurable time decay: exponential, linear, or step -- applied at query time, not stored
- Scored retrieval with pre/post filters, min_score threshold, and decay
- Smart retrieval: contradiction-aware packing + diversity penalties + budget limits
- Budget-aware retrieval (greedy knapsack by score/cost)
- Provenance trees and minimal support sets (`getSupportTree`, `getSupportSet`)
- Temporal sort and time-based importance decay
- Bulk transforms with conditional update/retract (`applyMany`)
- Conflict detection and resolution (`CONTRADICTS` / `SUPERSEDES`)
- Staleness detection and cascade retraction
- Identity resolution (transitive `ALIAS` groups)
- Serialization (`toJSON` / `fromJSON` / `stringify` / `parse`)
- Graph stats (counts by kind, author, scope, edge kind)
- Event envelope wrapping for bus integration
- Command log replay for state reconstruction

**Intent graph:**
- Status machine: active ↔ paused → completed / cancelled
- Sub-intent hierarchies via `parent_id`
- Query by owner, status, priority, parent, linked memory items
- Invalid transitions throw typed errors

**Task graph:**
- Status machine: pending → running → completed / failed, with retry support (failed → running)
- Subtask hierarchies via `parent_id`
- Links to parent intent, input/output memory items, agent assignment
- Query by intent, action, status, agent, parent, linked memory items

**Transplant (export / import):**
- Export a self-contained slice by walking provenance chains, aliases, related intents/tasks
- Import into another graph instance — default: skip existing ids, append-only
- Optional shallow compare to detect conflicts, optional re-id to mint new ids on conflict
- JSON-serializable slices for migration, sub-agent isolation, cloning, and backup

## Multi-Agent & Crew Orchestration

MemEX supports multi-agent systems where each agent works on a segment of the graph. No separate memory stores per agent — one graph, segmented by conventions.

### Soft isolation (shared graph, scoped views)

Each agent reads and writes to the shared graph, filtered by `meta.agent_id` and `scope`:

```ts
// agent:researcher only sees its own observations
const myMemories = getItems(state, {
  meta: { agent_id: "agent:researcher" },
});

// agent:analyst sees everything in a project scope
const projectMemories = getItems(state, {
  scope_prefix: "project:cyberdeck/",
});

// orchestrator sees all agents' work, ranked by importance
const ranked = getScoredItems(state,
  { authority: 0.5, importance: 0.5 },
  { pre: { scope_prefix: "project:cyberdeck/" } },
);
```

Agents write with their own `author` and `meta.agent_id`. The orchestrator can query across all agents, compare their findings, and resolve contradictions.

### Hard isolation (exported slices)

For risky operations or external sandboxes, export a slice for the sub-agent to work on independently:

```ts
// give the sub-agent a slice of the graph
const slice = exportSlice(memState, intentState, taskState, {
  memory_ids: relevantIds,
  include_parents: true,
  include_related_tasks: true,
});

// sub-agent works on its own copy...
// ...then merge results back
const { memState: updated, report } = importSlice(
  memState, intentState, taskState,
  subAgentSlice,
);
// report.created -> what the sub-agent added
// report.updated -> what was merged into existing items
// existing items untouched by default (append-only)
```

### Crew patterns

| Pattern | How |
|---------|-----|
| Shared workspace | All agents write to the same scope, filter by `meta.agent_id` to see own work |
| Pipeline | Agent A's `output_memory_ids` on a task become agent B's `input_memory_ids` |
| Review | Agent B reads agent A's items, creates `SUPPORTS` / `CONTRADICTS` edges |
| Delegation | Orchestrator creates an intent, assigns tasks to specific agents via `task.agent_id` |
| Sandbox | Export slice → sub-agent mutates copy → import results back |

### What the `author` and `meta` fields enable

```ts
// who wrote this?
item.author                     // "agent:researcher"

// which agent instance?
item.meta.agent_id              // "agent:researcher-v2"

// which session?
item.meta.session_id            // "session-abc"

// which crew run?
item.meta.crew_id               // "crew:investigation-42"

// which intent spawned this?
item.intent_id                  // "i1"

// which task produced this?
item.task_id                    // "t1"
```

All of these are queryable via `meta` and `meta_has` filters. The graph is one shared structure; segmentation is just queries.

## Dynamic Resolution

MemEX supports different levels of detail at every stage of the memory lifecycle:

| Stage | Low resolution | High resolution |
|-------|---------------|----------------|
| **Retrieval** | High-authority items only, no inferred, fast | Include hypotheses, simulations, full provenance chains |
| **Thinking** | Direct facts + deterministic derivations | Multi-hop reasoning, contradiction surfacing, support tree traversal |
| **Insertion** | Store summaries, mark details as low-importance | Store atomic events with full `DERIVED_FROM` chains |

Resolution is controlled through the same primitives -- filters, score weights, and decay:

```ts
// low resolution: only trusted, recent items
getItems(state, {
  range: { authority: { min: 0.7 } },
  not: { or: [{ kind: "hypothesis" }, { kind: "simulation" }] },
  decay: { config: { rate: 0.3, interval: "day", type: "exponential" }, min: 0.5 },
});

// high resolution: everything, scored and ranked
smartRetrieve(state, {
  budget: 8192,
  costFn: (item) => JSON.stringify(item.content).length,
  weights: { authority: 0.3, conviction: 0.3, importance: 0.4 },
  contradictions: "surface",
});
```

The agent decides resolution based on the task. A routine action uses low resolution. A decision with consequences uses high resolution. The same graph serves both -- no separate "fast" and "deep" memory stores.

### Thinking Budget from Scores

The three scores can drive the thinking budget itself. Items that are important but uncertain deserve more processing. Items that have been processed should have their importance reduced.

```text
thinking_priority = importance * (1 - authority)
```

An item with `importance: 0.9` and `authority: 0.3` gets priority `0.63` -- high attention, uncertain, worth reasoning about. An item with `importance: 0.9` and `authority: 0.95` gets priority `0.045` -- important but already trusted, just use it.

After the agent processes an item, reduce its importance:

```ts
applyCommand(state, {
  type: "memory.update",
  item_id: processedItem.id,
  partial: { importance: processedItem.importance * 0.3 },
  author: "system:thinker",
  reason: "processed",
});
```

This creates a natural attention cycle: new items arrive with high importance, get processed, importance drops, and they fade into long-term memory unless re-activated. Items that are never processed accumulate and eventually surface through importance-weighted queries.

The cognition layer above can use `getScoredItems` with importance-heavy weights to build its thinking queue, and `decayImportance` to age out items that were never worth processing.

## Choosing Parameters

The library provides knobs. Here's how to think about turning them.

### Decay

| Scenario | Recommendation |
|----------|---------------|
| Chat context, ephemeral state | Fast decay: `{ rate: 0.3, interval: "hour", type: "linear" }` |
| Project knowledge, working memory | Moderate decay: `{ rate: 0.1, interval: "day", type: "exponential" }` |
| Policies, traits, identity | No decay — these don't become less true over time |
| Mixed graph | Use the `decay` filter to exclude stale items, but don't decay items with `kind: "policy"` or `kind: "trait"` — filter them in separately with `or` |

### Diversity penalties

| Scenario | Recommendation |
|----------|---------------|
| Exploration ("what do we know?") | High `author_penalty` (0.3-0.5) — spread across sources |
| Verification ("is this true?") | Low or zero `author_penalty` — you *want* correlated evidence |
| Summarization | Moderate `parent_penalty` (0.2-0.3) — avoid redundant derivations |
| Debugging / audit | Zero penalties — show everything |

### Score weights

| Scenario | Weights |
|----------|---------|
| High-trust retrieval | `{ authority: 0.8, importance: 0.2 }` |
| Attention-driven (what needs processing?) | `{ importance: 0.8, authority: 0.2 }` |
| Agent self-evaluation | `{ conviction: 0.5, authority: 0.5 }` |
| Balanced | `{ authority: 0.4, conviction: 0.3, importance: 0.3 }` |

### Contradiction handling

| Scenario | Mode |
|----------|------|
| User-facing context (clean, no confusion) | `contradictions: "filter"` |
| Agent reasoning (needs to see disagreement) | `contradictions: "surface"` |
| Audit / debugging | Neither — use `getContradictions()` directly |

These are starting points, not prescriptions. Calibrate based on your use case.

See [API.md](./API.md) for the full API reference.

## License

Apache 2.0 -- see [LICENSE](./LICENSE).
