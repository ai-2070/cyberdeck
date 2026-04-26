import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import { createHookRegistry } from "../src/runtime/hooks.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";
import {
  DEFAULT_CREW_SHAPE,
  DEFAULT_CREW_AGENTS,
} from "./fixtures/default-crew.js";
import { TINY_CREW_SHAPE, TINY_CREW_AGENTS } from "./fixtures/tiny-crew.js";

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
function runEcho(
  session: CrewSession,
  rootInput: unknown = "ROOT",
): CrewEvent[] {
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
    expect(
      (events[1] as Extract<CrewEvent, { type: "role.entered" }>).roleId,
    ).toBe("caller");
    expect(events[2].type).toBe("agent.step.requested");

    const req = events[2] as Extract<
      CrewEvent,
      { type: "agent.step.requested" }
    >;
    expect(req.agentId).toBe("caller-1");
    expect(req.input).toBe("ROOT");

    expect(session.status()).toBe("awaiting_responses");
    expect(session.pendingRequests()).toHaveLength(1);
  });

  it("agent.step.requested carries a self-contained role snapshot", () => {
    const { session } = setup();
    const events = session.start("ROOT");
    const req = events[2] as Extract<
      CrewEvent,
      { type: "agent.step.requested" }
    >;
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

  it("task set at start() appears on crew.started but NOT on agent.step.requested", () => {
    const { session } = setup();
    const task = { description: "Research bird sleep patterns" };

    const log: CrewEvent[] = [];
    const queue: CrewEvent[] = session.start("ROOT", task);

    while (queue.length > 0) {
      const e = queue.shift()!;
      log.push(e);
      if (e.type === "agent.step.requested") {
        queue.push(
          ...session.deliver({
            type: "agent.step.completed",
            correlationId: e.correlationId,
            output: "ok",
            ts: 1000,
          }),
        );
      }
    }

    const started = log.find((e) => e.type === "crew.started") as Extract<
      CrewEvent,
      { type: "crew.started" }
    >;
    expect(started.task).toEqual(task);

    // Per-step requests must NOT carry the task — each phase only sees the
    // upstream phase's resolved output as its input.
    const requests = log.filter(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>[];
    expect(requests.length).toBeGreaterThan(0);
    for (const r of requests) {
      expect((r as { task?: unknown }).task).toBeUndefined();
    }
  });

  it("crew.started has no task field when start() was called without one", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const started = initial.find((e) => e.type === "crew.started") as Extract<
      CrewEvent,
      { type: "crew.started" }
    >;
    expect(started.task).toBeUndefined();
  });

  it("RoleSnapshot carries the role's system_prompt (with agents-config override)", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "caller"
          ? { ...r, system_prompt: "default-caller-instructions" }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "DEFAULT_CREW",
      agents: [
        { role: "merc", amount: 4 },
        { role: "specialist", amount: 1 },
        { role: "fixer", amount: 1 },
        { role: "caller", amount: 1, system_prompt: "research bird sleep" },
      ],
    });
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "sp-test",
      graph,
      clock: frozenClock(0),
    });
    const events = session.start("ROOT");
    const req = events.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(req.role.system_prompt).toBe("research bird sleep");
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

    expect(log.filter((e) => e.type === "agent.step.requested")).toHaveLength(
      7,
    );
    expect(log.filter((e) => e.type === "agent.step.completed")).toHaveLength(
      7,
    );
    expect(log.filter((e) => e.type === "vote.resolved")).toHaveLength(4);
    expect(log[log.length - 1].type).toBe("crew.completed");

    expect(session.status()).toBe("completed");
  });

  it("parallel mercs all emit requests; machine waits for all to resolve", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

    const phase2 = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "caller-out",
      ts: 1000,
    });

    const mercReqs = phase2.filter(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>[];
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
          (e as Extract<CrewEvent, { type: "role.entered" }>).roleId ===
            "specialist",
      ),
    ).toBe(true);
  });

  it("phase outputs voted in graph declaration order, not delivery order (determinism)", () => {
    const { session } = setup();
    const log: CrewEvent[] = session.start("ROOT");

    // resolve caller phase
    const callerReq = log.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
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
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId ===
          "merc",
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

  it("fault triggers fixer, fixer's response substitutes in voting", () => {
    // DEFAULT_CREW has fixer.activation.on_fault: true, so a fault must spawn
    // a fixer step. The fixer's response substitutes for the failed agent's
    // output, and first_valid picks merc-1's slot (now fixer's output).
    const { session } = setup();
    const log: CrewEvent[] = session.start("ROOT");

    const callerReq = log.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
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
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId ===
          "merc",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>[];

    // merc-1 faults — should emit fixer.invoked + agent.step.requested for fixer-1
    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: mercReqs[0].correlationId,
      output: "merc-1-attempted",
      fault: true,
      ts: 1000,
    });
    log.push(...burst);
    expect(burst.some((e) => e.type === "fixer.invoked")).toBe(true);
    const fixerReq = burst.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).agentId ===
          "fixer-1",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(fixerReq).toBeDefined();

    // mercs 2..4 succeed
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

    // Fixer responds — phase should now complete and merc-1's slot is the fixer output
    log.push(
      ...session.deliver({
        type: "agent.step.completed",
        correlationId: fixerReq.correlationId,
        output: "fixer-rescued",
        ts: 1000,
      }),
    );

    const mercVote = log.find(
      (e) =>
        e.type === "vote.resolved" &&
        (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "merc",
    ) as Extract<CrewEvent, { type: "vote.resolved" }>;
    expect(mercVote.resolved).toBe("fixer-rescued");
  });

  it("duplicate terminal deliver is a no-op (returns [])", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

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
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

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
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

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

describe("Fixer activation", () => {
  it("fault triggers fixer when activation.on_fault is set", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

    // caller faults — DEFAULT_CREW fixer has on_fault: true
    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "boom",
      fault: true,
      ts: 1000,
    });

    const fixerInvoked = burst.find(
      (e) => e.type === "fixer.invoked",
    ) as Extract<CrewEvent, { type: "fixer.invoked" }>;
    expect(fixerInvoked).toBeDefined();
    expect(fixerInvoked.reason).toBe("fault");

    const fixerReq = burst.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).agentId ===
          "fixer-1",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(fixerReq).toBeDefined();
    // Fixer's input describes the failure
    const fixerInput = fixerReq.input as {
      reason: string;
      failedAgentId: string;
    };
    expect(fixerInput.reason).toBe("fault");
    expect(fixerInput.failedAgentId).toBe("caller-1");
  });

  it("no activation flag = no fixer; fault stays as a faulted output", () => {
    const tinyShape = CrewShapeSchema.parse(TINY_CREW_SHAPE);
    const tinyAgents = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const tinyGraph = buildCrewGraph(tinyShape, tinyAgents);
    const tinySession = createCrewSession({
      crewId: "tiny-1",
      graph: tinyGraph,
      clock: frozenClock(0),
    });

    const initial = tinySession.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst = tinySession.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "alpha-fail",
      fault: true,
      ts: 0,
    });

    expect(burst.some((e) => e.type === "fixer.invoked")).toBe(false);
    // Phase advances since alpha was the only agent in its phase — fault is
    // recorded as the resolved output (first_valid skipped, returns undefined).
    const alphaVote = burst.find(
      (e) =>
        e.type === "vote.resolved" &&
        (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "alpha",
    ) as Extract<CrewEvent, { type: "vote.resolved" }>;
    expect(alphaVote.resolved).toBeUndefined();
  });

  it("stall triggers fixer when activation.on_stall is set", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "stalled-output",
      stalled: true,
      ts: 1000,
    });
    const fixerInvoked = burst.find(
      (e) => e.type === "fixer.invoked",
    ) as Extract<CrewEvent, { type: "fixer.invoked" }>;
    expect(fixerInvoked).toBeDefined();
    expect(fixerInvoked.reason).toBe("stall");
  });

  it("timeout does NOT trigger fixer when activation.on_timeout is not set", () => {
    // DEFAULT_CREW has only on_fault and on_stall.
    const { session } = setup({ defaultTimeoutMs: 5000 });
    session.start("ROOT");
    const burst = session.tick(6000);
    expect(burst.some((e) => e.type === "agent.step.timed_out")).toBe(true);
    expect(burst.some((e) => e.type === "fixer.invoked")).toBe(false);
  });

  it("does not recursively invoke fixer when fixer itself faults", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const callerReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst1 = session.deliver({
      type: "agent.step.completed",
      correlationId: callerReq.correlationId,
      output: "boom",
      fault: true,
      ts: 1000,
    });
    const fixerReq = burst1.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).agentId ===
          "fixer-1",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;

    // Fixer also faults
    const burst2 = session.deliver({
      type: "agent.step.completed",
      correlationId: fixerReq.correlationId,
      output: "fixer-bombed",
      fault: true,
      ts: 1000,
    });
    // No additional fixer.invoked event
    expect(burst2.some((e) => e.type === "fixer.invoked")).toBe(false);
    // Phase completes (single-agent caller phase, fixer's faulted output substitutes)
    expect(burst2.some((e) => e.type === "vote.resolved")).toBe(true);
  });
});

describe("Hooks", () => {
  it("before_role and after_role fire around a phase", () => {
    const stages: string[] = [];
    const hooks = createHookRegistry({
      log: (ctx) => stages.push(`${ctx.stage}:${ctx.role.role}`),
    });

    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) => ({
        ...r,
        hooks: { before_role: "log", after_role: "log" },
      })),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const sess = createCrewSession({
      crewId: "hooked",
      graph,
      clock: frozenClock(0),
      hooks,
    });

    const queue = sess.start("ROOT");
    while (queue.length > 0) {
      const e = queue.shift()!;
      if (e.type === "agent.step.requested") {
        queue.push(
          ...sess.deliver({
            type: "agent.step.completed",
            correlationId: e.correlationId,
            output: "ok",
            ts: 0,
          }),
        );
      }
    }

    expect(stages).toEqual([
      "before_role:alpha",
      "after_role:alpha",
      "before_role:beta",
      "after_role:beta",
      "before_role:gamma",
      "after_role:gamma",
    ]);
  });

  it("before_agent and after_agent fire per agent", () => {
    const events: string[] = [];
    const hooks = createHookRegistry({
      track: (ctx) => events.push(`${ctx.stage}:${ctx.agent?.id ?? "?"}`),
    });

    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "beta"
          ? { ...r, hooks: { before_agent: "track", after_agent: "track" } }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const sess = createCrewSession({
      crewId: "hooked",
      graph,
      clock: frozenClock(0),
      hooks,
    });

    const queue = sess.start("ROOT");
    while (queue.length > 0) {
      const e = queue.shift()!;
      if (e.type === "agent.step.requested") {
        queue.push(
          ...sess.deliver({
            type: "agent.step.completed",
            correlationId: e.correlationId,
            output: "ok",
            ts: 0,
          }),
        );
      }
    }

    expect(events).toContain("before_agent:beta-1");
    expect(events).toContain("before_agent:beta-2");
    expect(events).toContain("after_agent:beta-1");
    expect(events).toContain("after_agent:beta-2");
  });

  it("registry refuses prototype lookups (e.g. '__proto__')", () => {
    const reg = createHookRegistry({ legit: () => {} });
    expect(reg.get("legit")).toBeTypeOf("function");
    expect(reg.get("__proto__")).toBeUndefined();
    expect(reg.get("constructor")).toBeUndefined();
    expect(reg.get("toString")).toBeUndefined();
  });

  it("control.checkpoint emits checkpoint.taken into the same burst", () => {
    const hooks = createHookRegistry({
      checkpoint: (ctx) => ctx.control.checkpoint("phase-end"),
    });

    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "alpha" ? { ...r, hooks: { after_role: "checkpoint" } } : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const sess = createCrewSession({
      crewId: "ckpt",
      graph,
      clock: frozenClock(0),
      hooks,
    });

    const initial = sess.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst = sess.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "ok",
      ts: 0,
    });

    const ckpt = burst.find((e) => e.type === "checkpoint.taken") as Extract<
      CrewEvent,
      { type: "checkpoint.taken" }
    >;
    expect(ckpt).toBeDefined();
    expect(ckpt.checkpointId).toBe("phase-end");
  });
});
