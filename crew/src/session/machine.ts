import type { AgentId, CrewGraph, CrewRole, RoleId } from "../graph/types.js";
import type {
  AgentStepRequest,
  CrewEvent,
  RoleSnapshot,
} from "../events/types.js";
import type { Clock } from "../runtime/clock.js";
import type {
  CreateCrewSessionOpts,
  CrewSession,
  CrewSnapshot,
  CrewStatus,
} from "./types.js";
import { correlationId } from "../runtime/ids.js";
import { resolveVotes, type VoteEntry } from "../voting/resolve.js";

export function createCrewSession(opts: CreateCrewSessionOpts): CrewSession {
  return new CrewSessionImpl(opts);
}

interface PhaseState {
  roleId: RoleId;
  pending: Map<string, AgentStepRequest>;
  outputs: Map<string, VoteEntry>;
  terminalSeen: Set<string>;
}

class CrewSessionImpl implements CrewSession {
  private readonly crewId: string;
  private readonly graph: CrewGraph;
  private readonly clock: Clock;
  private readonly defaultTimeoutMs?: number;
  private readonly phaseOrder: RoleId[];

  private status_: CrewStatus = "idle";
  private phaseIndex = -1;
  private currentPhase?: PhaseState;
  private currentInput: unknown = undefined;
  private agentAttempts = new Map<AgentId, number>();

  constructor(opts: CreateCrewSessionOpts) {
    this.crewId = opts.crewId;
    this.graph = opts.graph;
    this.clock = opts.clock;
    this.defaultTimeoutMs = opts.defaultTimeoutMs;
    this.phaseOrder = computePhaseOrder(opts.graph);
  }

  status(): CrewStatus {
    return this.status_;
  }

  pendingRequests(): readonly AgentStepRequest[] {
    if (!this.currentPhase) return [];
    return [...this.currentPhase.pending.values()];
  }

  snapshot(): CrewSnapshot {
    return {
      schema_version: "1.0",
      crewId: this.crewId,
      status: this.status_,
      phaseIndex: this.phaseIndex,
    };
  }

  start(rootInput: unknown): CrewEvent[] {
    if (this.status_ !== "idle") {
      throw new Error(`Cannot start: session is ${this.status_}`);
    }
    this.status_ = "awaiting_responses";
    this.currentInput = rootInput;

    const events: CrewEvent[] = [
      {
        type: "crew.started",
        crewId: this.crewId,
        rootInput,
        ts: this.clock.now(),
      },
    ];
    this.advanceToNextPhase(events);
    return events;
  }

  deliver(event: CrewEvent): CrewEvent[] {
    if (this.status_ !== "awaiting_responses") return [];

    if (event.type === "agent.stream.chunk") {
      // Pass-through: chunks don't advance state, are not stored in outputs,
      // and produce no outbound events. Observers tail the bus separately.
      return [];
    }

    if (
      event.type !== "agent.step.completed" &&
      event.type !== "agent.step.failed" &&
      event.type !== "agent.step.timed_out"
    ) {
      return [];
    }

    const phase = this.currentPhase;
    if (!phase) return [];

    const cid = event.correlationId;

    // Idempotency: first terminal wins, subsequent duplicates are no-ops.
    if (phase.terminalSeen.has(cid)) return [];
    if (!phase.pending.has(cid)) return [];

    phase.terminalSeen.add(cid);
    const req = phase.pending.get(cid)!;
    phase.pending.delete(cid);

    let output: unknown;
    let fault = false;
    let stalled = false;
    if (event.type === "agent.step.completed") {
      output = event.output;
      fault = event.fault === true;
      stalled = event.stalled === true;
    } else {
      // failed | timed_out — model both as faults
      output = undefined;
      fault = true;
    }
    phase.outputs.set(cid, { agentId: req.agentId, output, fault, stalled });

    const events: CrewEvent[] = [];
    if (phase.pending.size === 0) {
      this.completeCurrentPhase(events);
    }
    return events;
  }

  tick(now: number): CrewEvent[] {
    if (this.status_ !== "awaiting_responses") return [];
    const phase = this.currentPhase;
    if (!phase) return [];

    const events: CrewEvent[] = [];
    const expired: AgentStepRequest[] = [];
    for (const req of phase.pending.values()) {
      if (req.timeoutMs !== undefined && req.ts + req.timeoutMs <= now) {
        expired.push(req);
      }
    }
    for (const req of expired) {
      const cid = req.correlationId;
      if (phase.terminalSeen.has(cid)) continue;
      phase.terminalSeen.add(cid);
      phase.pending.delete(cid);
      phase.outputs.set(cid, {
        agentId: req.agentId,
        output: undefined,
        fault: true,
      });
      events.push({
        type: "agent.step.timed_out",
        correlationId: cid,
        ts: now,
      });
    }
    if (phase.pending.size === 0 && expired.length > 0) {
      this.completeCurrentPhase(events);
    }
    return events;
  }

  cancel(reason: "cancelled" | "timeout"): CrewEvent[] {
    if (this.status_ === "completed" || this.status_ === "aborted") return [];
    this.status_ = "aborted";
    this.currentPhase = undefined;
    return [{ type: "crew.aborted", reason, ts: this.clock.now() }];
  }

  // ─────────────────────────────────────────────────────────────────────────

  private advanceToNextPhase(events: CrewEvent[]): void {
    this.phaseIndex++;
    if (this.phaseIndex >= this.phaseOrder.length) {
      this.status_ = "completed";
      this.currentPhase = undefined;
      events.push({
        type: "crew.completed",
        finalOutput: this.currentInput,
        ts: this.clock.now(),
      });
      return;
    }

    const roleId = this.phaseOrder[this.phaseIndex];
    const role = this.graph.roles.get(roleId)!;
    const agents = this.graph.agents.filter((a) => a.role === roleId);

    events.push({ type: "role.entered", roleId, ts: this.clock.now() });

    const phase: PhaseState = {
      roleId,
      pending: new Map(),
      outputs: new Map(),
      terminalSeen: new Set(),
    };

    const timeoutMs = role.execution?.timeout_ms ?? this.defaultTimeoutMs;

    for (const agent of agents) {
      const attempt = (this.agentAttempts.get(agent.id) ?? 0) + 1;
      this.agentAttempts.set(agent.id, attempt);
      const cid = correlationId({
        crewId: this.crewId,
        roleId,
        agentId: agent.id,
        attempt,
      });
      const req: AgentStepRequest = {
        correlationId: cid,
        agentId: agent.id,
        roleId,
        ts: this.clock.now(),
        ...(timeoutMs !== undefined ? { timeoutMs } : {}),
      };
      phase.pending.set(cid, req);
      events.push({
        type: "agent.step.requested",
        correlationId: cid,
        agentId: agent.id,
        roleId,
        input: this.currentInput,
        role: snapshotRole(role),
        ...(timeoutMs !== undefined ? { timeoutMs } : {}),
        ts: this.clock.now(),
      });
    }

    this.currentPhase = phase;

    if (agents.length === 0) {
      // No agents declared for this role — phase auto-completes. Edge case for
      // crews where a middle role has zero count but is in the phase order.
      this.completeCurrentPhase(events);
    }
  }

  private completeCurrentPhase(events: CrewEvent[]): void {
    const phase = this.currentPhase;
    if (!phase) return;

    const role = this.graph.roles.get(phase.roleId)!;
    const mode = role.execution?.voting?.mode ?? "first_valid";

    // Order outputs by graph.agents declaration order — voting is deterministic
    // regardless of inbound delivery order.
    const phaseAgents = this.graph.agents.filter((a) => a.role === phase.roleId);
    const ordered: VoteEntry[] = [];
    for (const a of phaseAgents) {
      for (const out of phase.outputs.values()) {
        if (out.agentId === a.id) {
          ordered.push(out);
          break;
        }
      }
    }

    const resolved = resolveVotes(ordered, mode);

    events.push({
      type: "vote.resolved",
      roleId: phase.roleId,
      mode,
      resolved,
      ts: this.clock.now(),
    });

    this.currentInput = resolved;
    this.advanceToNextPhase(events);
  }
}

function computePhaseOrder(graph: CrewGraph): RoleId[] {
  const roles = [...graph.roles.values()];
  const firstInput = roles.find((r) => r.first_input);
  const finalOutput = roles.find((r) => r.final_output);
  if (!firstInput || !finalOutput) {
    throw new Error(
      "Graph must have first_input and final_output roles (lint should have caught this)",
    );
  }
  // Activation-driven roles (fixer-style) are excluded from the main flow.
  // They run when triggered by faults / stalls / timeouts / permission denials.
  const middle = roles.filter(
    (r) => !r.first_input && !r.final_output && !r.activation,
  );
  return [firstInput.role, ...middle.map((r) => r.role), finalOutput.role];
}

function snapshotRole(role: CrewRole): RoleSnapshot {
  return {
    name: role.role,
    ...(role.description !== undefined ? { description: role.description } : {}),
    capabilities: role.capabilities,
    permissions: {
      talk_to: [...role.permissions.talk_to],
      delegate_to: [...role.permissions.delegate_to],
      escalate_to: [...role.permissions.escalate_to],
    },
  };
}
