import type {
  ExportOptions,
  GraphState,
  MemexExport,
  MemoryCommand,
  MemoryFilter,
  MemoryItem,
  ImportReport,
  ScoredItem,
  SmartRetrievalOptions,
} from "@ai2070/memex";
import type { CrewAgent, CrewRole } from "../graph/types.js";

export type MemexView = "self" | "role" | "crew" | "all";

// Per-agent read handle. Reads are scoped by `view`; writes happen via the
// adapter's `apply()` (called by the session machine after a worker returns
// memex_commands in agent.step.completed).
export interface AgentMemexHandle {
  read(filter?: MemoryFilter): MemoryItem[];
  retrieve(opts: SmartRetrievalOptions): ScoredItem[];
}

// Context passed to apply() so the adapter can stamp meta on outbound writes.
export interface MemexStampContext {
  agentId: string;
  crewId: string;
  roleId: string;
}

// Adapter is session-scoped mutable state. Don't share one instance across
// concurrent sessions. See PLAN.md Phase 4.
export interface MemexAdapter {
  handleFor(agent: CrewAgent, role: CrewRole, view: MemexView): AgentMemexHandle;
  apply(cmd: MemoryCommand, ctx: MemexStampContext): void;
  exportSlice(opts: ExportOptions): MemexExport;
  importSlice(slice: MemexExport): ImportReport;
  snapshot(): GraphState;

  // Hard-isolation helpers (used by nested crews with memex.isolation: "hard").
  // `fork` produces a child adapter seeded with a deep copy of this adapter's
  // memory state under a new crewId. `exportAll` returns every memory item in
  // the adapter as a slice (so the parent can importSlice it on completion).
  fork(crewId: string): MemexAdapter;
  exportAll(): MemexExport;
}
