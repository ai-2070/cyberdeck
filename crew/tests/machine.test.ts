import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";
import { DEFAULT_CREW_SHAPE, DEFAULT_CREW_AGENTS } from "./fixtures/default-crew.js";

function setup(opts?: { defaultTimeoutMs?: number }) {
  const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
  const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
  const graph = buildCrewGraph(shape, counts);
  const clock = frozenClock(1000);
  const session = createCrewSession({
    crewId: "crew-test-1",
    graph,
    clock,
    ...opts,
  });
  return { session, clock, graph };
}

// Run the session with a synchronous "echo worker": every agent.step.requested
// gets answered with an agent.step.completed whose output mirrors the input.
function runEcho(session: CrewSession, rootInput: unknown = "ROOT"): CrewEvent[] {
  const log: CrewEvent[] = [];
  const queue: CrewEvent[] = session.start(rootInput);

  while (queue.length > 0) {
    const e = queue.shift()!;
    log.push(e);

    if (e.type === "agent.step.requested") {
      const completion: CrewEvent = {
        type: "agent.step.completed",
        correlationId: e.correlationId,
        output: { from: e.agentId, input: e.input },
        ts: 1000,
      };
      log.push(completion);
      const next = session.deliver(completion);
      queue.push(...next);
    }
  }

  return log;
}

describe("CrewSession.start", () => {
  it("emits crew.started + role.entered + caller's agent.step.requested", () => {
    const { session } = setup();
    const events = session.start("ROOT");
    expect(events[0].type).toBe("crew.started");
    expect(events[1].type).toBe("role.entered");
    expect((events[1] as Extract<CrewEvent, { type: "role.entered" }>).roleId).toBe("caller");
    expect(events[2].type).toBe("agent.step.requested");

    const req = events[2] as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(req.agentId).toBe("caller-1");
    expect(req.input).toBe("ROOT");

    expect(session.status()).toBe("awaiting_responses");
    expect(session.pendingRequests()).toHaveLength(1);
  });

  it("agent.step.requested carries a self-contained role snapshot", () => {
    const { session } = setup();
    const events = session.start("ROOT");
    const req = events[2] as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(req.role).toEqual({
      name: "caller",
      capabilities: { thinking_allowed: false },
      permissions: {
        talk_to: ["fixer"],
        delegate_to: [],
        escalate_to: [],
      },
    });
  });

  it("throws if start() is called twice", () => {
    const { session } = setup();
    session.start("ROOT");
    expect(() => session.start("ROOT")).toThrow();
  });
});

describe("CrewSession.deliver", () => {
  it("runs the full caller→merc→specialist→caller flow with an echo worker", () => {
    const { session } = setup();
    const log = runEcho(session);

    const roleEnters = log
      .filter((e) => e.type === "role.entered")
      .map((e) => (e as Extract<CrewEvent, { type: "role.entered" }>).roleId);
    expect(roleEnters).toEqual(["caller", "merc", "specialist", "caller"]);

    expect(log.filter((e) => e.type === "agent.step.requested")).toHaveLength(7);
    expect(log.filter((e) => e.type === "agent.step.completed")).toHaveLength(7);
    expect(log.filter((e) => e.type === "vote.resolved")).toHaveLength(4);
    expect(log[log.length - 1].type).toBe("crew.completed");

    expect(session.status()).toBe("completed");
  });

  it("parallel mercs all emit requests; machine waits for all to resolve", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const phase2 = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "caller-out",
      ts: 1000,
    });

    const mercReqs = phase2.filter((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>[];
    expect(mercReqs).toHaveLength(4);
    expect(session.pendingRequests()).toHaveLength(4);

    for (let i = 0; i < 3; i++) {
      const next = session.deliver({
        type: "agent.step.completed",
        correlationId: mercReqs[i].correlationId,
        output: `merc-${i + 1}-out`,
        ts: 1000,
      });
      const advanced = next.some(
        (e) => e.type === "vote.resolved" || e.type === "role.entered",
      );
      expect(advanced).toBe(false);
    }
    expect(session.pendingRequests()).toHaveLength(1);

    const after = session.deliver({
      type: "agent.step.completed",
      correlationId: mercReqs[3].correlationId,
      output: "merc-4-out",
      ts: 1000,
    });
    expect(after.some((e) => e.type === "vote.resolved")).toBe(true);
    expect(
      after.some(
        (e) =>
          e.type === "role.entered" &&
          (e as Extract<CrewEvent, { type: "role.entered" }>).roleId === "specialist",
      ),
    ).toBe(true);
  });

  it("phase outputs voted in graph declaration order, not delivery order (determinism)", () => {
    const { session } = setup();
    const log: CrewEvent[] = session.start("ROOT");

    // resolve caller phase
    const callerReq = log.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    log.push(
      ...session.deliver({
        type: "agent.step.completed",
        correlationId: callerReq.correlationId,
        output: "caller-out",
        ts: 1000,
      }),
    );

    const mercReqs = log.filter(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "merc",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>[];

    // Deliver in REVERSE order: 4, 3, 2, 1
    for (const i of [3, 2, 1, 0]) {
      log.push(
        ...session.deliver({
          type: "agent.step.completed",
          correlationId: mercReqs[i].correlationId,
          output: `merc-${i + 1}-out`,
          ts: 1000,
        }),
      );
    }

    const mercVote = log.find(
      (e) =>
        e.type === "vote.resolved" &&
        (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "merc",
    ) as Extract<CrewEvent, { type: "vote.resolved" }>;
    expect(mercVote.resolved).toBe("merc-1-out");
  });

  it("first_valid voting skips faulted outputs", () => {
    const { session } = setup();
    const log: CrewEvent[] = session.start("ROOT");
    const callerReq = log.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    log.push(
      ...session.deliver({
        type: "agent.step.completed",
        correlationId: callerReq.correlationId,
        output: "caller-out",
        ts: 1000,
      }),
    );

    const mercReqs = log.filter(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "merc",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>[];

    log.push(
      ...session.deliver({
        type: "agent.step.completed",
        correlationId: mercReqs[0].correlationId,
        output: "merc-1-out",
        fault: true,
        ts: 1000,
      }),
    );
    for (let i = 1; i < 4; i++) {
      log.push(
        ...session.deliver({
          type: "agent.step.completed",
          correlationId: mercReqs[i].correlationId,
          output: `merc-${i + 1}-out`,
          ts: 1000,
        }),
      );
    }

    const mercVote = log.find(
      (e) =>
        e.type === "vote.resolved" &&
        (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "merc",
    ) as Extract<CrewEvent, { type: "vote.resolved" }>;
    expect(mercVote.resolved).toBe("merc-2-out");
  });

  it("duplicate terminal deliver is a no-op (returns [])", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const first = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "first-out",
      ts: 1000,
    });
    expect(first.length).toBeGreaterThan(0);

    const second = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "second-out",
      ts: 1000,
    });
    expect(second).toEqual([]);
  });

  it("agent.stream.chunk passes through without advancing state", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const result = session.deliver({
      type: "agent.stream.chunk",
      correlationId: callerReq.correlationId,
      chunk: { token: "hello" },
      ts: 1000,
    });
    expect(result).toEqual([]);
    expect(session.pendingRequests()).toHaveLength(1);
    expect(session.status()).toBe("awaiting_responses");
  });

  it("deliver before start is a no-op", () => {
    const { session } = setup();
    expect(
      session.deliver({
        type: "agent.step.completed",
        correlationId: "anything",
        output: "x",
        ts: 1000,
      }),
    ).toEqual([]);
  });

  it("unknown correlationId is a no-op", () => {
    const { session } = setup();
    session.start("ROOT");
    expect(
      session.deliver({
        type: "agent.step.completed",
        correlationId: "bogus",
        output: "x",
        ts: 1000,
      }),
    ).toEqual([]);
  });
});

describe("CrewSession.tick (timeouts)", () => {
  it("emits agent.step.timed_out for expired requests", () => {
    const { session } = setup({ defaultTimeoutMs: 5000 });
    session.start("ROOT");

    expect(session.tick(2000)).toEqual([]);

    const expired = session.tick(6000);
    expect(expired.some((e) => e.type === "agent.step.timed_out")).toBe(true);
    // Single-agent caller phase: timeout should advance to next phase
    expect(expired.some((e) => e.type === "vote.resolved")).toBe(true);
  });

  it("tick before start is a no-op", () => {
    const { session } = setup({ defaultTimeoutMs: 5000 });
    expect(session.tick(99999)).toEqual([]);
  });

  it("tick after completed is a no-op", () => {
    const { session } = setup();
    runEcho(session);
    expect(session.status()).toBe("completed");
    expect(session.tick(99999)).toEqual([]);
  });

  it("requests with no timeoutMs never expire", () => {
    const { session } = setup(); // no defaultTimeoutMs
    session.start("ROOT");
    expect(session.tick(Number.MAX_SAFE_INTEGER)).toEqual([]);
    expect(session.pendingRequests()).toHaveLength(1);
  });
});

describe("CrewSession.cancel", () => {
  it("emits crew.aborted and transitions to aborted", () => {
    const { session } = setup();
    session.start("ROOT");

    const result = session.cancel("cancelled");
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe("crew.aborted");
    expect(
      (result[0] as Extract<CrewEvent, { type: "crew.aborted" }>).reason,
    ).toBe("cancelled");
    expect(session.status()).toBe("aborted");
  });

  it("subsequent cancels are no-ops", () => {
    const { session } = setup();
    session.start("ROOT");
    session.cancel("cancelled");
    expect(session.cancel("cancelled")).toEqual([]);
  });

  it("subsequent deliver after cancel is a no-op", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    session.cancel("cancelled");
    expect(
      session.deliver({
        type: "agent.step.completed",
        correlationId: callerReq.correlationId,
        output: "x",
        ts: 1000,
      }),
    ).toEqual([]);
  });

  it("cancel after completion is a no-op", () => {
    const { session } = setup();
    runEcho(session);
    expect(session.cancel("cancelled")).toEqual([]);
  });
});

describe("CrewSession.snapshot", () => {
  it("returns minimal status info during a run", () => {
    const { session } = setup();
    expect(session.snapshot().status).toBe("idle");
    expect(session.snapshot().phaseIndex).toBe(-1);

    session.start("ROOT");
    expect(session.snapshot().status).toBe("awaiting_responses");
    expect(session.snapshot().phaseIndex).toBe(0);
    expect(session.snapshot().crewId).toBe("crew-test-1");
  });
});
