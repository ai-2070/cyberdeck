import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import { canonicalize } from "../src/events/canonical.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";
import { DEFAULT_CREW_SHAPE, DEFAULT_CREW_AGENTS } from "./fixtures/default-crew.js";

function freshSession(): CrewSession {
  const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
  const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
  const graph = buildCrewGraph(shape, counts);
  return createCrewSession({
    crewId: "crew-replay",
    graph,
    clock: frozenClock(1000),
  });
}

// Echo worker that returns deterministic outputs derived from the agent id.
function runEcho(session: CrewSession): CrewEvent[] {
  const log: CrewEvent[] = [];
  const queue: CrewEvent[] = session.start("ROOT");
  while (queue.length > 0) {
    const e = queue.shift()!;
    log.push(e);
    if (e.type === "agent.step.requested") {
      const completion: CrewEvent = {
        type: "agent.step.completed",
        correlationId: e.correlationId,
        output: { from: e.agentId },
        ts: 1000,
      };
      log.push(completion);
      const next = session.deliver(completion);
      queue.push(...next);
    }
  }
  return log;
}

describe("Replay determinism", () => {
  it("identical input sequences produce byte-identical event logs (canonicalized)", () => {
    const log1 = runEcho(freshSession());
    const log2 = runEcho(freshSession());

    expect(canonicalize(log1)).toBe(canonicalize(log2));
  });

  it("inbound delivery order does not affect the outbound log for parallel phases", () => {
    // Run once with in-order merc delivery, once with reversed delivery,
    // and assert the outbound logs match (modulo individual delivery order).
    const inOrder = runEcho(freshSession());

    // Build a "reversed mercs" run by hand: capture all requests, deliver mercs reversed.
    const session = freshSession();
    const log: CrewEvent[] = [];

    const initial = session.start("ROOT");
    log.push(...initial);

    // Resolve all phases in declaration order, except mercs delivered reversed.
    const drainSimple = (prev: CrewEvent[]) => {
      let frontier = prev;
      while (true) {
        const reqs = frontier.filter(
          (e): e is Extract<CrewEvent, { type: "agent.step.requested" }> =>
            e.type === "agent.step.requested",
        );
        if (reqs.length === 0) return;

        const isMercPhase = reqs.every((r) => r.roleId === "merc");
        const order = isMercPhase ? [...reqs].reverse() : reqs;

        const collected: CrewEvent[] = [];
        for (const r of order) {
          const completion: CrewEvent = {
            type: "agent.step.completed",
            correlationId: r.correlationId,
            output: { from: r.agentId },
            ts: 1000,
          };
          log.push(completion);
          const next = session.deliver(completion);
          log.push(...next);
          collected.push(...next);
        }
        frontier = collected;
      }
    };
    drainSimple(initial);

    // The two outbound logs differ in inbound delivery interleaving (the
    // agent.step.completed events appear in different orders in the log),
    // but the OUTBOUND events emitted by the session — role.entered,
    // agent.step.requested, vote.resolved, crew.completed — should be identical.
    const outboundOnly = (l: CrewEvent[]) =>
      l.filter((e) => e.type !== "agent.step.completed");
    expect(canonicalize(outboundOnly(log))).toBe(
      canonicalize(outboundOnly(inOrder)),
    );
  });

  // Resume policy tests come with Phase 5 (checkpointing).
  it.todo("ResumePolicy 'abort' refuses to resume if a step is requested but unresolved");
  it.todo("ResumePolicy 're-emit-request' replays the missing request");
});
