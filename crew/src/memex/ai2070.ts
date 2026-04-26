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
  MemoryCommand,
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

// Concrete adapter against @ai2070/memex. Wraps mutable graph/intent/task
// state and stamps meta on outbound writes. Created per crew session.
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
          return getItems(mem, mergeFilters(baseFilter, filter));
        },
        retrieve(retrievalOpts: SmartRetrievalOptions) {
          return smartRetrieve(mem, mergeFilterIntoSmart(retrievalOpts, baseFilter));
        },
      };
    },

    apply(cmd, ctx) {
      const stamped = stampCommand(cmd, ctx);
      const result = applyCommand(mem, stamped);
      mem = result.state;
    },

    exportSlice(o: ExportOptions): MemexExport {
      return memexExportSlice(mem, intent, task, o);
    },

    importSlice(slice: MemexExport) {
      const r = memexImportSlice(mem, intent, task, slice);
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

function mergeFilters(
  base: MemoryFilter | undefined,
  override: MemoryFilter | undefined,
): MemoryFilter | undefined {
  if (!base) return override;
  if (!override) return base;
  return {
    ...base,
    ...override,
    meta: { ...(base.meta ?? {}), ...(override.meta ?? {}) },
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
function stampCommand(cmd: MemoryCommand, ctx: MemexStampContext): MemoryCommand {
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
