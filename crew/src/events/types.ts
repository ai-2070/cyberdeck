import type { VotingMode } from "../schema/consensus.js";
import type { AgentId, RoleId } from "../graph/types.js";

// Structural mirror of @ai2070/memex's MemoryCommand. Defined locally to keep
// the root public surface free of a hard transitive type-import on
// @ai2070/memex (which is an optional peer). Memex's strict MemoryCommand is
// structurally assignable to this type, so consumers that DO use memex pass
// real commands through without casting. Consumers who don't use memex can
// build minimal command objects against this shape.
export type MemoryCommand =
  | { type: "memory.create"; item: Record<string, unknown> }
  | {
      type: "memory.update";
      item_id: string;
      partial: Record<string, unknown>;
      author: string;
      reason?: string;
      basis?: Record<string, unknown>;
    }
  | { type: "memory.retract"; item_id: string; author: string; reason?: string }
  | { type: "edge.create"; edge: Record<string, unknown> }
  | {
      type: "edge.update";
      edge_id: string;
      partial?: Record<string, unknown>;
      author: string;
      reason?: string;
    }
  | { type: "edge.retract"; edge_id: string; author: string; reason?: string };

// Self-contained role snapshot embedded in agent.step.requested.
// Workers receive this and don't need to load the crew shape out-of-band.
export interface RoleSnapshot {
  name: string;
  description?: string;
  capabilities?: {
    model?: string;
    tools?: string[];
    thinking_allowed?: boolean;
  };
  permissions: {
    talk_to: string[];
    delegate_to: string[];
    escalate_to: string[];
  };
}

export type FixerReason = "fault" | "stall" | "timeout" | "permission_denied";
export type GatedAction = "talk_to" | "delegate_to" | "escalate_to";
export type AbortReason = "cancelled" | "timeout";

export type CrewEvent =
  // ─── outbound (session-emitted) ───
  | { type: "crew.started"; crewId: string; rootInput: unknown; ts: number }
  | { type: "role.entered"; roleId: RoleId; ts: number }
  | {
      type: "agent.step.requested";
      correlationId: string;
      agentId: AgentId;
      roleId: RoleId;
      input: unknown;
      memex_context?: unknown;
      role: RoleSnapshot;
      timeoutMs?: number;
      ts: number;
    }
  | { type: "fixer.invoked"; agentId: AgentId; reason: FixerReason; ts: number }
  | {
      type: "permission.denied";
      from: AgentId;
      to: AgentId;
      action: GatedAction;
      ts: number;
    }
  | {
      type: "vote.resolved";
      roleId: RoleId;
      mode: VotingMode;
      resolved: unknown;
      ts: number;
    }
  | { type: "checkpoint.taken"; checkpointId: string; ts: number }
  | {
      type: "memex.command.emitted";
      agentId: AgentId;
      command: MemoryCommand;
      ts: number;
    }
  | {
      type: "nested.crew.started";
      agentId: AgentId;
      nestedName: string;
      ts: number;
    }
  | {
      type: "nested.crew.completed";
      agentId: AgentId;
      output: unknown;
      ts: number;
    }
  | { type: "crew.aborted"; reason: AbortReason; ts: number }
  | { type: "crew.completed"; finalOutput: unknown; ts: number }
  // ─── inbound (caller-delivered via session.deliver) ───
  | {
      type: "agent.step.completed";
      correlationId: string;
      output: unknown;
      memex_commands?: MemoryCommand[];   // Phase 4: worker-emitted memex writes
      fault?: boolean;
      stalled?: boolean;
      ts: number;
    }
  | {
      type: "agent.step.failed";
      correlationId: string;
      error: { name: string; message: string };
      ts: number;
    }
  | { type: "agent.step.timed_out"; correlationId: string; ts: number }
  | {
      type: "agent.stream.chunk";
      correlationId: string;
      chunk: unknown;
      ts: number;
    };

export type CrewEventType = CrewEvent["type"];

export type OutboundEventType =
  | "crew.started"
  | "role.entered"
  | "agent.step.requested"
  | "fixer.invoked"
  | "permission.denied"
  | "vote.resolved"
  | "checkpoint.taken"
  | "memex.command.emitted"
  | "nested.crew.started"
  | "nested.crew.completed"
  | "crew.aborted"
  | "crew.completed";

export type InboundEventType =
  | "agent.step.completed"
  | "agent.step.failed"
  | "agent.step.timed_out"
  | "agent.stream.chunk";

export type TerminalInboundType =
  | "agent.step.completed"
  | "agent.step.failed"
  | "agent.step.timed_out";

const TERMINAL_INBOUND: ReadonlySet<string> = new Set<TerminalInboundType>([
  "agent.step.completed",
  "agent.step.failed",
  "agent.step.timed_out",
]);

const INBOUND: ReadonlySet<string> = new Set<InboundEventType>([
  "agent.step.completed",
  "agent.step.failed",
  "agent.step.timed_out",
  "agent.stream.chunk",
]);

export function isInboundEvent(e: CrewEvent): boolean {
  return INBOUND.has(e.type);
}

export function isTerminalInbound(e: CrewEvent): boolean {
  return TERMINAL_INBOUND.has(e.type);
}

// Public projection used by CrewSession.pendingRequests().
export interface AgentStepRequest {
  correlationId: string;
  agentId: AgentId;
  roleId: RoleId;
  ts: number;
  timeoutMs?: number;
}
