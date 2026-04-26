import {
  applyCommand,
  cloneGraphState,
  createGraphState,
  createIntentState,
  createTaskState,
  exportSlice as memexExportSlice,
  getIntents,
  getItems,
  getTasks,
  importSlice as memexImportSlice,
  smartRetrieve,
} from "@ai2070/memex";
import type {
  ExportOptions,
  GraphState,
  IntentState,
  MemexExport,
  MemoryCommand as MemexMemoryCommand,
  MemoryFilter,
  MemoryItem,
  SmartRetrievalOptions,
  TaskState,
} from "@ai2070/memex";
import type {
  AgentMemexHandle,
  MemexAdapter,
  MemexStampContext,
  MemexView,
} from "./adapter.js";

// Re-export the adapter types so the `@ai2070/crew/memex` subpath is self-contained.
export type {
  AgentMemexHandle,
  MemexAdapter,
  MemexStampContext,
  MemexView,
} from "./adapter.js";

export interface CreateMemexAdapterOpts {
  crewId: string;
  memState?: GraphState;
  intentState?: IntentState;
  taskState?: TaskState;
}

// Concrete adapter against @ai2070/memex. Uses memex's strict types
// internally; the returned adapter conforms to the opaque MemexAdapter
// interface (from ./adapter.ts) so the root `@ai2070/crew` surface stays
// memex-free for type checking.
export function createMemexAdapter(opts: CreateMemexAdapterOpts): MemexAdapter {
  const crewId = opts.crewId;
  let mem: GraphState = opts.memState ?? createGraphState();
  let intent: IntentState = opts.intentState ?? createIntentState();
  let task: TaskState = opts.taskState ?? createTaskState();

  return {
    handleFor(agent, role, view): AgentMemexHandle {
      const baseFilter = filterForView(view, agent.id, role.role, crewId);
      return {
        read(filter) {
          return getItems(mem, mergeFilters(baseFilter, filter as MemoryFilter | undefined));
        },
        retrieve(retrievalOpts) {
          return smartRetrieve(
            mem,
            mergeFilterIntoSmart(retrievalOpts as SmartRetrievalOptions, baseFilter),
          );
        },
      };
    },

    apply(cmd, ctx) {
      const stamped = stampCommand(cmd as MemexMemoryCommand, ctx);
      const result = applyCommand(mem, stamped);
      mem = result.state;
    },

    exportSlice(o): MemexExport {
      return memexExportSlice(mem, intent, task, o as ExportOptions);
    },

    importSlice(slice) {
      const r = memexImportSlice(mem, intent, task, slice as MemexExport);
      mem = r.memState;
      intent = r.intentState;
      task = r.taskState;
      return r.report;
    },

    snapshot() {
      return mem;
    },

    fork(newCrewId: string): MemexAdapter {
      // Memory state is deep-cloned via memex's cloneGraphState. Intent and
      // task states start empty in the fork — memex doesn't expose clone
      // helpers for them yet, and the common case for hard-isolated nested
      // crews is "I want my own scratch space for memories".
      return createMemexAdapter({
        crewId: newCrewId,
        memState: cloneGraphState(mem),
      });
    },

    restoreFrom(snapshot: unknown, newCrewId: string): MemexAdapter {
      // Used by resumeCrewSession to rebuild a hard-isolated inner adapter
      // from a captured GraphState. Same internal shape as fork, but
      // pre-loaded with the caller's state instead of cloning ours.
      return createMemexAdapter({
        crewId: newCrewId,
        memState: snapshot as GraphState,
      });
    },

    exportAll(): MemexExport {
      const memItems = getItems(mem);
      const intents = getIntents(intent);
      const tasks = getTasks(task);
      return memexExportSlice(mem, intent, task, {
        memory_ids: memItems.map((i) => i.id),
        intent_ids: intents.map((i) => i.id),
        task_ids: tasks.map((t) => t.id),
      });
    },
  };
}

function filterForView(
  view: MemexView,
  agentId: string,
  roleId: string,
  crewId: string,
): MemoryFilter | undefined {
  switch (view) {
    case "all":
      return undefined;
    case "self":
      return { meta: { agent_id: agentId } };
    case "role":
      return { meta: { role: roleId, crew_id: crewId } };
    case "crew":
      return { meta: { crew_id: crewId } };
  }
}

// Scope filters from the view (self/role/crew) MUST win over caller-supplied
// filters — otherwise an agent could pass `meta: { agent_id: "other" }` and read
// outside its scope. `base` is always the view scope here; caller filters can
// only refine within it.
function mergeFilters(
  base: MemoryFilter | undefined,
  override: MemoryFilter | undefined,
): MemoryFilter | undefined {
  if (!base) return override;
  if (!override) return base;
  return {
    ...override,
    ...base,
    meta: { ...(override.meta ?? {}), ...(base.meta ?? {}) },
  };
}

function mergeFilterIntoSmart(
  opts: SmartRetrievalOptions,
  baseFilter: MemoryFilter | undefined,
): SmartRetrievalOptions {
  if (!baseFilter) return opts;
  return {
    ...opts,
    filter: mergeFilters(baseFilter, opts.filter),
  };
}

// Stamp memory.create with author/meta/scope so cross-agent visibility filters
// work without the worker remembering to set them. Other commands pass through —
// the worker is responsible for setting `author` on update/retract.
function stampCommand(cmd: MemexMemoryCommand, ctx: MemexStampContext): MemexMemoryCommand {
  if (cmd.type !== "memory.create") return cmd;

  const item: MemoryItem = { ...cmd.item };
  if (!item.author) item.author = `agent:${ctx.agentId}`;
  item.meta = {
    ...(item.meta ?? {}),
    agent_id: ctx.agentId,
    crew_id: ctx.crewId,
    role: ctx.roleId,
  };
  if (!item.scope) item.scope = `crew:${ctx.crewId}/agent:${ctx.agentId}`;
  return { type: "memory.create", item };
}
