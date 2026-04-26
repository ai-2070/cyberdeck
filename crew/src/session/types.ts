import type { AgentId, CrewGraph } from "../graph/types.js";
import type { Clock } from "../runtime/clock.js";
import type { CrewEvent, AgentStepRequest, GatedAction } from "../events/types.js";
import type { HookRegistry } from "../runtime/hooks.js";

export type CrewStatus = "idle" | "awaiting_responses" | "completed" | "aborted";

export type ResumePolicy = "re-emit-request" | "treat-as-failed" | "abort";

export interface CrewSnapshot {
  schema_version: "1.0";
  crewId: string;
  status: CrewStatus;
  phaseIndex: number;
  // Phase 5 will expand this to capture full machine state for resume.
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
  snapshot(): CrewSnapshot;
  // Workers/external code can ask the session to validate an action.
  // Returns `allowed` plus any events the session emitted (permission.denied,
  // optionally fixer.invoked + agent.step.requested for fixer activation).
  requestAction(req: ActionRequest): RequestActionResult;
}

export interface CreateCrewSessionOpts {
  crewId: string;
  graph: CrewGraph;
  clock: Clock;
  hooks?: HookRegistry;
  defaultTimeoutMs?: number;
  resumePolicy?: ResumePolicy;
}
