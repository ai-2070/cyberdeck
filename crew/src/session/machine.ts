import type { AgentId, CrewAgent, CrewGraph, CrewRole, RoleId } from "../graph/types.js";
import type {
  AgentStepRequest,
  CrewEvent,
  FixerReason,
  RoleSnapshot,
} from "../events/types.js";
import type { Clock } from "../runtime/clock.js";
import type {
  CrewControl,
  HookContext,
  HookRegistry,
  HookStage,
} from "../runtime/hooks.js";
import type {
  ActionRequest,
  CreateCrewSessionOpts,
  CrewSession,
  CrewSnapshot,
  CrewStatus,
  RequestActionResult,
} from "./types.js";
import { correlationId } from "../runtime/ids.js";
import { resolveVotes, type VoteEntry } from "../voting/resolve.js";
import { checkPermission } from "./permissions.js";

export function createCrewSession(opts: CreateCrewSessionOpts): CrewSession {
  return new CrewSessionImpl(opts);
}

interface PhaseState {
  roleId: RoleId;
  pending: Map<string, AgentStepRequest>;     // correlationId -> request
  outputs: Map<AgentId, VoteEntry>;            // agentId -> response (substitution-aware)
  terminalSeen: Set<string>;                   // correlationIds we've processed
  fixerSteps: Map<string, AgentId>;            // fixer cid -> original failed agent
  nestedPending: Set<AgentId>;                 // outer agentIds whose inner session is in flight
}

interface InnerHandle {
  outerAgentId: AgentId;
  outerRoleId: RoleId;
  innerSession: CrewSession;
  innerCids: Set<string>;                      // cids the inner session is currently waiting on
}

type HookCtxPartial = Pick<HookContext, "role"> &
  Partial<Pick<HookContext, "agent" | "input" | "output" | "fault" | "stalled">>;

class CrewSessionImpl implements CrewSession {
  private readonly crewId: string;
  private readonly graph: CrewGraph;
  private readonly clock: Clock;
  private readonly hooks?: HookRegistry;
  private readonly defaultTimeoutMs?: number;
  private readonly phaseOrder: RoleId[];

  private status_: CrewStatus = "idle";
  private phaseIndex = -1;
  private currentPhase?: PhaseState;
  private currentInput: unknown = undefined;
  private agentAttempts = new Map<AgentId, number>();
  private innerByCid = new Map<string, InnerHandle>();
  private innerByOuter = new Map<AgentId, InnerHandle>();

  constructor(opts: CreateCrewSessionOpts) {
    this.crewId = opts.crewId;
    this.graph = opts.graph;
    this.clock = opts.clock;
    this.hooks = opts.hooks;
    this.defaultTimeoutMs = opts.defaultTimeoutMs;
    this.phaseOrder = computePhaseOrder(opts.graph);
  }

  status(): CrewStatus {
    return this.status_;
  }

  pendingRequests(): readonly AgentStepRequest[] {
    if (!this.currentPhase) return [];
    const out: AgentStepRequest[] = [...this.currentPhase.pending.values()];
    for (const handle of this.innerByOuter.values()) {
      out.push(...handle.innerSession.pendingRequests());
    }
    return out;
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
      { type: "crew.started", crewId: this.crewId, rootInput, ts: this.clock.now() },
    ];
    this.advanceToNextPhase(events);
    return events;
  }

  deliver(event: CrewEvent): CrewEvent[] {
    if (this.status_ !== "awaiting_responses") return [];
    if (event.type === "agent.stream.chunk") return [];
    if (
      event.type !== "agent.step.completed" &&
      event.type !== "agent.step.failed" &&
      event.type !== "agent.step.timed_out"
    ) {
      return [];
    }

    // Route to inner session if the cid belongs to a nested crew.
    const innerHandle = this.innerByCid.get(event.correlationId);
    if (innerHandle) {
      return this.deliverToNested(innerHandle, event);
    }

    const phase = this.currentPhase;
    if (!phase) return [];

    const cid = event.correlationId;
    if (phase.terminalSeen.has(cid)) return [];
    if (!phase.pending.has(cid)) return [];

    phase.terminalSeen.add(cid);
    const req = phase.pending.get(cid)!;
    phase.pending.delete(cid);

    let output: unknown;
    let fault = false;
    let stalled = false;
    let timedOut = false;
    if (event.type === "agent.step.completed") {
      output = event.output;
      fault = event.fault === true;
      stalled = event.stalled === true;
    } else if (event.type === "agent.step.failed") {
      output = undefined;
      fault = true;
    } else {
      output = undefined;
      timedOut = true;
    }

    const events: CrewEvent[] = [];
    this.processTerminal(phase, req, cid, { output, fault, stalled, timedOut }, events);

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
      events.push({
        type: "agent.step.timed_out",
        correlationId: cid,
        ts: now,
      });
      this.processTerminal(
        phase,
        req,
        cid,
        { output: undefined, fault: false, stalled: false, timedOut: true },
        events,
      );
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

  requestAction(req: ActionRequest): RequestActionResult {
    const events: CrewEvent[] = [];
    const allowed = this.requestActionInternal(req, events);
    return { allowed, events };
  }

  // ─── private ──────────────────────────────────────────────────────────────

  private processTerminal(
    phase: PhaseState,
    req: AgentStepRequest,
    cid: string,
    res: { output: unknown; fault: boolean; stalled: boolean; timedOut: boolean },
    events: CrewEvent[],
  ): void {
    const isFixerStep = phase.fixerSteps.has(cid);
    const targetAgentId = isFixerStep ? phase.fixerSteps.get(cid)! : req.agentId;

    phase.outputs.set(targetAgentId, {
      agentId: targetAgentId,
      output: res.output,
      fault: res.fault || res.timedOut,
      stalled: res.stalled,
    });

    const role = this.graph.roles.get(req.roleId)!;
    const agent = this.graph.agents.find((a) => a.id === req.agentId);

    // Per-reason hooks fire only for the original (non-fixer) agent.
    if (!isFixerStep) {
      if (res.fault) {
        this.callHook(role.hooks?.on_fault, "on_fault", { role, agent, output: res.output, fault: true }, events);
      } else if (res.stalled) {
        this.callHook(role.hooks?.on_stall, "on_stall", { role, agent, output: res.output, stalled: true }, events);
      } else if (res.timedOut) {
        this.callHook(role.hooks?.on_timeout, "on_timeout", { role, agent }, events);
      }
    }

    // after_agent hook fires for every terminal (including fixer responses).
    this.callHook(
      role.hooks?.after_agent,
      "after_agent",
      { role, agent, output: res.output, fault: res.fault, stalled: res.stalled },
      events,
    );

    // Activation: only for non-fixer normal agents that faulted/stalled/timed out.
    if (!isFixerStep && (res.fault || res.stalled || res.timedOut)) {
      const reason: FixerReason = res.timedOut ? "timeout" : res.stalled ? "stall" : "fault";
      this.maybeInvokeFixer(req.agentId, reason, events);
    }
  }

  private requestActionInternal(req: ActionRequest, events: CrewEvent[]): boolean {
    const allowed = checkPermission(this.graph, req.from, req.to, req.action);
    if (allowed) return true;

    events.push({
      type: "permission.denied",
      from: req.from,
      to: req.to,
      action: req.action,
      ts: this.clock.now(),
    });

    const fromAgent = this.graph.agents.find((a) => a.id === req.from);
    if (fromAgent) {
      const fromRole = this.graph.roles.get(fromAgent.role);
      if (fromRole) {
        this.callHook(
          fromRole.hooks?.on_permission_denied,
          "on_permission_denied",
          { role: fromRole, agent: fromAgent },
          events,
        );
      }
    }
    this.maybeInvokeFixer(req.from, "permission_denied", events);
    return false;
  }

  private maybeInvokeFixer(failedAgentId: AgentId, reason: FixerReason, events: CrewEvent[]): void {
    const phase = this.currentPhase;
    if (!phase) return;

    const fixerRole = [...this.graph.roles.values()].find((r) => {
      const a = r.activation;
      if (!a) return false;
      if (reason === "fault" && a.on_fault) return true;
      if (reason === "stall" && a.on_stall) return true;
      if (reason === "timeout" && a.on_timeout) return true;
      if (reason === "permission_denied" && a.on_permission_denied) return true;
      return false;
    });
    if (!fixerRole) return;

    const fixerAgents = this.graph.agents.filter((a) => a.role === fixerRole.role);
    if (fixerAgents.length === 0) return;
    const fixerAgent = fixerAgents[0];

    events.push({
      type: "fixer.invoked",
      agentId: fixerAgent.id,
      reason,
      ts: this.clock.now(),
    });

    const failedOutput = phase.outputs.get(failedAgentId);
    const fixerInput = {
      reason,
      failedAgentId,
      failedRoleId: phase.roleId,
      lastOutput: failedOutput?.output,
      upstreamInput: this.currentInput,
    };

    this.emitStepRequest(fixerAgent, fixerRole, fixerInput, events, failedAgentId);
  }

  private emitStepRequest(
    agent: CrewAgent,
    role: CrewRole,
    input: unknown,
    events: CrewEvent[],
    fixerForAgent?: AgentId,
  ): void {
    if (agent.nestedCrew) {
      this.spawnNested(agent, role, input, events);
      return;
    }

    const phase = this.currentPhase!;

    this.callHook(role.hooks?.before_agent, "before_agent", { role, agent, input }, events);

    const attempt = (this.agentAttempts.get(agent.id) ?? 0) + 1;
    this.agentAttempts.set(agent.id, attempt);
    const cid = correlationId({
      crewId: this.crewId,
      roleId: role.role,
      agentId: agent.id,
      attempt,
    });
    const timeoutMs = role.execution?.timeout_ms ?? this.defaultTimeoutMs;
    const req: AgentStepRequest = {
      correlationId: cid,
      agentId: agent.id,
      roleId: role.role,
      ts: this.clock.now(),
      ...(timeoutMs !== undefined ? { timeoutMs } : {}),
    };
    phase.pending.set(cid, req);
    if (fixerForAgent !== undefined) {
      phase.fixerSteps.set(cid, fixerForAgent);
    }
    events.push({
      type: "agent.step.requested",
      correlationId: cid,
      agentId: agent.id,
      roleId: role.role,
      input,
      role: snapshotRole(role),
      ...(timeoutMs !== undefined ? { timeoutMs } : {}),
      ts: this.clock.now(),
    });
  }

  private spawnNested(
    agent: CrewAgent,
    role: CrewRole,
    input: unknown,
    events: CrewEvent[],
  ): void {
    const phase = this.currentPhase!;

    this.callHook(role.hooks?.before_agent, "before_agent", { role, agent, input }, events);

    const attempt = (this.agentAttempts.get(agent.id) ?? 0) + 1;
    this.agentAttempts.set(agent.id, attempt);

    events.push({
      type: "nested.crew.started",
      agentId: agent.id,
      nestedName: agent.nestedCrew!.name,
      ts: this.clock.now(),
    });

    const innerSession = createCrewSession({
      crewId: `${this.crewId}/${agent.id}#${attempt}`,
      graph: agent.nestedCrew!,
      clock: this.clock,
      hooks: this.hooks,
      defaultTimeoutMs: this.defaultTimeoutMs,
    });

    const handle: InnerHandle = {
      outerAgentId: agent.id,
      outerRoleId: role.role,
      innerSession,
      innerCids: new Set(),
    };
    this.innerByOuter.set(agent.id, handle);
    phase.nestedPending.add(agent.id);

    const innerEvents = innerSession.start(input);
    this.absorbInner(handle, innerEvents, events);
  }

  private deliverToNested(handle: InnerHandle, event: CrewEvent): CrewEvent[] {
    const innerEvents = handle.innerSession.deliver(event);
    if ("correlationId" in event) {
      handle.innerCids.delete(event.correlationId);
      this.innerByCid.delete(event.correlationId);
    }
    const outerEvents: CrewEvent[] = [];
    this.absorbInner(handle, innerEvents, outerEvents);
    return outerEvents;
  }

  private absorbInner(
    handle: InnerHandle,
    innerEvents: CrewEvent[],
    outerEvents: CrewEvent[],
  ): void {
    for (const e of innerEvents) {
      outerEvents.push(e);
      if (e.type === "agent.step.requested") {
        this.innerByCid.set(e.correlationId, handle);
        handle.innerCids.add(e.correlationId);
      } else if (e.type === "crew.completed") {
        this.finishNested(handle, e.finalOutput, outerEvents, false);
      } else if (e.type === "crew.aborted") {
        this.finishNested(handle, undefined, outerEvents, true);
      }
    }
  }

  private finishNested(
    handle: InnerHandle,
    finalOutput: unknown,
    outerEvents: CrewEvent[],
    aborted: boolean,
  ): void {
    for (const cid of handle.innerCids) {
      this.innerByCid.delete(cid);
    }
    handle.innerCids.clear();
    this.innerByOuter.delete(handle.outerAgentId);

    const phase = this.currentPhase;
    if (!phase) return;
    phase.nestedPending.delete(handle.outerAgentId);

    phase.outputs.set(handle.outerAgentId, {
      agentId: handle.outerAgentId,
      output: finalOutput,
      fault: aborted,
    });

    outerEvents.push({
      type: "nested.crew.completed",
      agentId: handle.outerAgentId,
      output: finalOutput,
      ts: this.clock.now(),
    });

    const role = this.graph.roles.get(handle.outerRoleId);
    const outerAgent = this.graph.agents.find((a) => a.id === handle.outerAgentId);
    if (role && outerAgent) {
      this.callHook(
        role.hooks?.after_agent,
        "after_agent",
        { role, agent: outerAgent, output: finalOutput, fault: aborted },
        outerEvents,
      );
    }

    if (phase.pending.size === 0 && phase.nestedPending.size === 0) {
      this.completeCurrentPhase(outerEvents);
    }
  }

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
      fixerSteps: new Map(),
      nestedPending: new Set(),
    };
    this.currentPhase = phase;

    this.callHook(role.hooks?.before_role, "before_role", { role, input: this.currentInput }, events);

    for (const agent of agents) {
      this.emitStepRequest(agent, role, this.currentInput, events);
    }

    if (agents.length === 0) {
      this.completeCurrentPhase(events);
    }
  }

  private completeCurrentPhase(events: CrewEvent[]): void {
    const phase = this.currentPhase;
    if (!phase) return;

    const role = this.graph.roles.get(phase.roleId)!;
    const mode = role.execution?.voting?.mode ?? "first_valid";

    const phaseAgents = this.graph.agents.filter((a) => a.role === phase.roleId);
    const ordered: VoteEntry[] = [];
    for (const a of phaseAgents) {
      const out = phase.outputs.get(a.id);
      if (out) ordered.push(out);
    }
    const resolved = resolveVotes(ordered, mode);

    events.push({
      type: "vote.resolved",
      roleId: phase.roleId,
      mode,
      resolved,
      ts: this.clock.now(),
    });

    this.callHook(role.hooks?.after_role, "after_role", { role, output: resolved }, events);

    this.currentInput = resolved;
    this.advanceToNextPhase(events);
  }

  private callHook(
    hookName: string | undefined,
    stage: HookStage,
    partial: HookCtxPartial,
    events: CrewEvent[],
  ): void {
    if (!hookName || !this.hooks) return;
    const fn = this.hooks.get(hookName);
    if (!fn) return;
    const control: CrewControl = {
      requestAction: (req) => this.requestActionInternal(req, events),
      checkpoint: (id) => {
        events.push({ type: "checkpoint.taken", checkpointId: id, ts: this.clock.now() });
      },
    };
    fn({
      crewId: this.crewId,
      stage,
      control,
      ...partial,
    });
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
