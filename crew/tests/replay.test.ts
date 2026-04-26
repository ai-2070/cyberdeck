import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { resumeCrewSession } from "../src/session/resume.js";
import { frozenClock } from "../src/runtime/clock.js";
import { canonicalize } from "../src/events/canonical.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";
import {
  DEFAULT_CREW_SHAPE,
  DEFAULT_CREW_AGENTS,
} from "./fixtures/default-crew.js";
import { TINY_CREW_SHAPE, TINY_CREW_AGENTS } from "./fixtures/tiny-crew.js";

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
});

describe("ResumePolicy", () => {
  function tinyGraph() {
    const shape = CrewShapeSchema.parse(TINY_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    return buildCrewGraph(shape, counts);
  }

  it("'abort' (default) throws when there are unresolved pending requests", () => {
    const graph = tinyGraph();
    const session = createCrewSession({
      crewId: "abort-test",
      graph,
      clock: frozenClock(0),
    });
    session.start("ROOT");
    const snap = session.snapshot();

    expect(() =>
      resumeCrewSession(snap, [], {
        crewId: "abort-test",
        graph,
        clock: frozenClock(0),
        resumePolicy: "abort",
      }),
    ).toThrow(/unresolved/i);
  });

  it("'re-emit-request' produces a fresh agent.step.requested for each unresolved pending", () => {
    const graph = tinyGraph();
    const session = createCrewSession({
      crewId: "reemit-test",
      graph,
      clock: frozenClock(0),
    });
    const initial = session.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const snap = session.snapshot();

    const { events } = resumeCrewSession(snap, [], {
      crewId: "reemit-test",
      graph,
      clock: frozenClock(100),
      resumePolicy: "re-emit-request",
    });

    const reEmitted = events.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(reEmitted).toBeDefined();
    expect(reEmitted.correlationId).toBe(alphaReq.correlationId);
    expect(reEmitted.input).toBe("ROOT");
    expect(reEmitted.role.name).toBe("alpha");
  });

  it("'re-emit-request' preserves memex_context that was sampled at original emit time", async () => {
    const { createMemexAdapter } = await import("../src/memex/ai2070.js");
    const { createMemoryItem } = await import("@ai2070/memex");

    const adapter = createMemexAdapter({ crewId: "ctx-resume" });
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "k", value: "preload" },
          author: "agent:alpha-1",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "alpha-1", crewId: "ctx-resume", roleId: "alpha" },
    );

    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "alpha"
          ? {
              ...r,
              execution: {
                memex: {
                  enabled: true,
                  retrieval: {
                    budget: 1024,
                    weights: { authority: 1, conviction: 0, importance: 0 },
                    costFn: () => 1,
                  },
                },
              },
            }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "ctx-resume",
      graph,
      clock: frozenClock(0),
      memex: adapter,
    });
    const initial = session.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(alphaReq.memex_context).toBeDefined();

    const snap = session.snapshot();
    const { events } = resumeCrewSession(snap, [], {
      crewId: "ctx-resume",
      graph,
      clock: frozenClock(100),
      memex: adapter,
      resumePolicy: "re-emit-request",
    });
    const reEmitted = events.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(reEmitted.memex_context).toEqual(alphaReq.memex_context);
  });

  it("'re-emit-request' works for nested-crew pendings (inner role not in outer graph)", async () => {
    const { createCrewSession: createCrewSessionFn } =
      await import("../src/session/machine.js");
    const PARENT = {
      schema_version: "1.0",
      name: "PARENT",
      roles: [
        {
          role: "host",
          capabilities: { thinking_allowed: true },
          permissions: {
            talk_to: ["worker"],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          first_input: true,
          final_output: true,
        },
        {
          role: "worker",
          capabilities: { thinking_allowed: true },
          permissions: {
            talk_to: ["host"],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          nested_crew: "INNER",
          amount: 1,
        },
      ],
    };
    const INNER = {
      schema_version: "1.0",
      name: "INNER",
      roles: [
        {
          role: "alpha",
          capabilities: { thinking_allowed: true },
          permissions: {
            talk_to: [],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          first_input: true,
          final_output: true,
          amount: 1,
        },
      ],
    };
    const parentShape = CrewShapeSchema.parse(PARENT);
    const innerShape = CrewShapeSchema.parse(INNER);
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "PARENT",
      agents: [
        { role: "host", amount: 1 },
        { role: "worker", amount: 1 },
      ],
    });
    const graph = buildCrewGraph(parentShape, counts, { INNER: innerShape });

    const session = createCrewSessionFn({
      crewId: "nested-resume",
      graph,
      clock: frozenClock(0),
    });
    const initial = session.start("ROOT");
    const hostReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    session.deliver({
      type: "agent.step.completed",
      correlationId: hostReq.correlationId,
      output: "host-out",
      ts: 0,
    });
    // Now in worker phase; inner alpha is pending. Snapshot.
    const snap = session.snapshot();

    const { events } = resumeCrewSession(snap, [], {
      crewId: "nested-resume",
      graph,
      clock: frozenClock(0),
      resumePolicy: "re-emit-request",
    });

    // The re-emitted request must be alpha (inner role), with the inner role
    // snapshot, even though "alpha" isn't in the outer graph's roles.
    const reEmitted = events.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId ===
          "alpha",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(reEmitted).toBeDefined();
    expect(reEmitted.role.name).toBe("alpha");
    expect(reEmitted.agentId).toMatch(/alpha/);
  });

  it("'treat-as-failed' synthesizes failed terminals so the phase can advance", () => {
    const graph = tinyGraph();
    const session = createCrewSession({
      crewId: "fail-test",
      graph,
      clock: frozenClock(0),
    });
    session.start("ROOT");
    const snap = session.snapshot();

    const { session: resumed, events } = resumeCrewSession(snap, [], {
      crewId: "fail-test",
      graph,
      clock: frozenClock(0),
      resumePolicy: "treat-as-failed",
    });
    expect(events.some((e) => e.type === "vote.resolved")).toBe(true);
    expect(events.some((e) => e.type === "role.entered")).toBe(true);
    expect(resumed.status()).toBe("awaiting_responses");
    expect(resumed.pendingRequests().every((r) => r.roleId === "beta")).toBe(
      true,
    );
  });
});
