import type { CrewAgent, CrewRole } from "../graph/types.js";
import type { MemoryCommand } from "../events/types.js";

// adapter.ts is the contract surface reachable from `@ai2070/crew`'s root
// (CreateCrewSessionOpts.memex references MemexAdapter). To keep memex an
// optional peer, this file deliberately uses opaque types (`unknown`) for
// memex-derived inputs and outputs. The concrete factory at
// `@ai2070/crew/memex` (memex/ai2070.ts) provides the strict-typed
// implementation. Consumers who use memex can cast results from
// `handle.read()` etc. to memex's `MemoryItem[]` / `ScoredItem[]` themselves.

export type MemexView = "self" | "role" | "crew" | "all";

export interface MemexStampContext {
  agentId: string;
  crewId: string;
  roleId: string;
}

export interface AgentMemexHandle {
  read(filter?: unknown): unknown[];
  retrieve(opts: unknown): unknown[];
}

export interface MemexAdapter {
  handleFor(agent: CrewAgent, role: CrewRole, view: MemexView): AgentMemexHandle;
  apply(cmd: MemoryCommand, ctx: MemexStampContext): void;
  exportSlice(opts: unknown): unknown;
  importSlice(slice: unknown): unknown;
  snapshot(): unknown;
  fork(crewId: string): MemexAdapter;
  exportAll(): unknown;
}
