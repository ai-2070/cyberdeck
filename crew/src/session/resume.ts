import type { CrewEvent } from "../events/types.js";
import type {
  CreateCrewSessionOpts,
  CrewSession,
  CrewSnapshot,
  ResumePolicy,
} from "./types.js";
import { restoreCrewSession } from "./machine.js";

export interface ResumeResult {
  session: CrewSession;
  events: CrewEvent[];
}

// Rebuild a session from a snapshot, replay any inbound events that arrived
// after the snapshot, then apply ResumePolicy to whatever requests remain
// unresolved. Returns the fully-rehydrated session and the events that were
// emitted during replay (so the caller can publish them onto the bus / log).
export function resumeCrewSession(
  snapshot: CrewSnapshot,
  laterEvents: CrewEvent[],
  opts: CreateCrewSessionOpts,
): ResumeResult {
  const session = restoreCrewSession(snapshot, opts);

  const events: CrewEvent[] = [];
  for (const e of laterEvents) {
    events.push(...session.deliver(e));
  }

  applyResumePolicy(session, opts.resumePolicy ?? "abort", opts, events);

  return { session, events };
}

function applyResumePolicy(
  session: CrewSession,
  policy: ResumePolicy,
  opts: CreateCrewSessionOpts,
  events: CrewEvent[],
): void {
  if (session.status() !== "awaiting_responses") return;

  const details = session.pendingDetails();
  if (details.length === 0) return;

  if (policy === "abort") {
    throw new Error(
      `Resume aborted: ${details.length} unresolved pending request(s) (no terminal events seen). ` +
        `Provide ResumePolicy "re-emit-request" or "treat-as-failed" to handle them.`,
    );
  }

  if (policy === "treat-as-failed") {
    for (const d of details) {
      events.push(
        ...session.deliver({
          type: "agent.step.failed",
          correlationId: d.request.correlationId,
          error: {
            name: "ResumeUnresolved",
            message: "Step had no terminal event in laterEvents during resume",
          },
          ts: opts.clock.now(),
        }),
      );
    }
    return;
  }

  if (policy === "re-emit-request") {
    // pendingDetails carries each step's role snapshot and memex context as
    // they were when the request was originally emitted — so we don't have
    // to look up roles in opts.graph (which doesn't contain inner-crew roles)
    // and we don't drop the memex context.
    for (const d of details) {
      events.push({
        type: "agent.step.requested",
        correlationId: d.request.correlationId,
        agentId: d.request.agentId,
        roleId: d.request.roleId,
        input: d.input,
        role: d.roleSnapshot,
        ...(d.memexContext !== undefined ? { memex_context: d.memexContext } : {}),
        ...(d.request.timeoutMs !== undefined ? { timeoutMs: d.request.timeoutMs } : {}),
        ts: opts.clock.now(),
      });
    }
    return;
  }

  const _exhaustive: never = policy;
  throw new Error(`Unknown ResumePolicy: ${String(_exhaustive)}`);
}
