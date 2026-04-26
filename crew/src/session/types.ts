import type { AgentId, CrewGraph, RoleId } from "../graph/types.js";
import type { Clock } from "../runtime/clock.js";
import type {
  CrewEvent,
  AgentStepRequest,
  GatedAction,
  RoleSnapshot,
} from "../events/types.js";
import type { HookRegistry } from "../runtime/hooks.js";
import type { MemexAdapter } from "../memex/adapter.js";
import type { VoteEntry } from "../voting/resolve.js";

export type CrewStatus = "idle" | "awaiting_responses" | "completed" | "aborted";

export type ResumePolicy = "re-emit-request" | "treat-as-failed" | "abort";

// Public projection of an in-flight step. Carries the input, the role
// snapshot the worker received, and the memex context that was sampled when
// the request was emitted. Resume's "re-emit-request" path uses these so it
// never needs to look up the role in some outer graph (which doesn't have
// inner-crew roles).
export interface AgentStepDetail {
  request: AgentStepRequest;
  input: unknown;
  roleSnapshot: RoleSnapshot;
  memexContext?: unknown;
}

export interface SerializedPendingEntry {
  request: AgentStepRequest;
  input: unknown;
  roleSnapshot: RoleSnapshot;
  memexContext?: unknown;
}

export interface SerializedPhaseState {
  roleId: RoleId;
  pending: SerializedPendingEntry[];
  outputs: Array<[AgentId, VoteEntry]>;
  terminalSeen: string[];
  fixerSteps: Array<[string, AgentId]>;
  nestedPending: AgentId[];
}

export interface SerializedNestedHandle {
  outerAgentId: AgentId;
  outerRoleId: RoleId;
  innerCids: string[];
  innerSnapshot: CrewSnapshot;
}

export interface CrewSnapshot {
  schema_version: "1.0";
  crewId: string;
  status: CrewStatus;
  phaseIndex: number;
  currentInput: unknown;
  agentAttempts: Array<[AgentId, number]>;
  currentPhase: SerializedPhaseState | null;
  nested: SerializedNestedHandle[];
  ts: number;
}

export interface ActionRequest {
  from: AgentId;
  to: AgentId;
  action: GatedAction;
}

export interface RequestActionResult {
  allowed: boolean;
  events: CrewEvent[];
}

export interface CrewSession {
  start(rootInput: unknown): CrewEvent[];
  deliver(event: CrewEvent): CrewEvent[];
  tick(now: number): CrewEvent[];
  cancel(reason: "cancelled" | "timeout"): CrewEvent[];
  status(): CrewStatus;
  pendingRequests(): readonly AgentStepRequest[];
  pendingDetails(): readonly AgentStepDetail[];
  snapshot(): CrewSnapshot;
  requestAction(req: ActionRequest): RequestActionResult;
}

export interface CreateCrewSessionOpts {
  crewId: string;
  graph: CrewGraph;
  clock: Clock;
  hooks?: HookRegistry;
  memex?: MemexAdapter;
  defaultTimeoutMs?: number;
  resumePolicy?: ResumePolicy;
  // When set to "phase", the session emits a checkpoint.taken event after every
  // vote.resolved with id `phase:{roleId}:{phaseIndex}`. The caller is expected
  // to observe and persist via session.snapshot() if they want durable resume.
  autoCheckpoint?: "phase";
}
