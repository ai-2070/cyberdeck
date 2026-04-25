import type { CrewGraph } from "../graph/types.js";
import type { Clock } from "../runtime/clock.js";
import type { CrewEvent, AgentStepRequest } from "../events/types.js";

export type CrewStatus = "idle" | "awaiting_responses" | "completed" | "aborted";

export type ResumePolicy = "re-emit-request" | "treat-as-failed" | "abort";

export interface CrewSnapshot {
  schema_version: "1.0";
  crewId: string;
  status: CrewStatus;
  phaseIndex: number;
  // Phase 5 will expand this to capture full machine state for resume.
}

export interface CrewSession {
  start(rootInput: unknown): CrewEvent[];
  deliver(event: CrewEvent): CrewEvent[];
  tick(now: number): CrewEvent[];
  cancel(reason: "cancelled" | "timeout"): CrewEvent[];
  status(): CrewStatus;
  pendingRequests(): readonly AgentStepRequest[];
  snapshot(): CrewSnapshot;
}

export interface CreateCrewSessionOpts {
  crewId: string;
  graph: CrewGraph;
  clock: Clock;
  defaultTimeoutMs?: number;
  resumePolicy?: ResumePolicy;
}
