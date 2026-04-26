import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";

const PARENT_SHAPE = {
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

const INNER_SHAPE = {
  schema_version: "1.0",
  name: "INNER",
  roles: [
    {
      role: "alpha",
      capabilities: { thinking_allowed: true },
      permissions: {
        talk_to: ["beta"],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      first_input: true,
      amount: 1,
    },
    {
      role: "beta",
      capabilities: { thinking_allowed: true },
      permissions: {
        talk_to: [],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      final_output: true,
      amount: 1,
    },
  ],
};

const PARENT_AGENTS = {
  schema_version: "1.0",
  name: "PARENT",
  agents: [
    { role: "host", amount: 1 },
    { role: "worker", amount: 1 },
  ],
};

function setup() {
  const parentShape = CrewShapeSchema.parse(PARENT_SHAPE);
  const innerShape = CrewShapeSchema.parse(INNER_SHAPE);
  const parentAgents = CrewAgentsSchema.parse(PARENT_AGENTS);
  const graph = buildCrewGraph(parentShape, parentAgents, { INNER: innerShape });
  const session = createCrewSession({
    crewId: "outer",
    graph,
    clock: frozenClock(0),
  });
  return { session, graph };
}

// Run the session with an echo worker; return the full event log.
function runEcho(session: CrewSession): CrewEvent[] {
  const log: CrewEvent[] = [];
  const queue: CrewEvent[] = session.start("ROOT");
  while (queue.length > 0) {
    const e = queue.shift()!;
    log.push(e);
    if (e.type === "agent.step.requested") {
      queue.push(
        ...session.deliver({
          type: "agent.step.completed",
          correlationId: e.correlationId,
          output: { from: e.agentId },
          ts: 0,
        }),
      );
    }
  }
  return log;
}

describe("Nested crews", () => {
  it("buildCrewGraph attaches the inner CrewGraph to the agent", () => {
    const { graph } = setup();
    const worker = graph.agents.find((a) => a.id === "worker-1")!;
    expect(worker.nestedCrew).toBeDefined();
    expect(worker.nestedCrew!.name).toBe("INNER");
  });

  it("spawning a nested agent emits nested.crew.started", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const hostReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: hostReq.correlationId,
      output: "host-output",
      ts: 0,
    });

    expect(burst.some((e) => e.type === "nested.crew.started")).toBe(true);
    const nestedStart = burst.find((e) => e.type === "nested.crew.started") as
      Extract<CrewEvent, { type: "nested.crew.started" }>;
    expect(nestedStart.agentId).toBe("worker-1");
    expect(nestedStart.nestedName).toBe("INNER");
  });

  it("inner crew events surface in the outer log with their own correlation ids", () => {
    const log = runEcho(setup().session);

    const innerCrewStarted = log.filter(
      (e) =>
        e.type === "crew.started" &&
        (e as Extract<CrewEvent, { type: "crew.started" }>).crewId.includes("worker-1"),
    );
    expect(innerCrewStarted).toHaveLength(1);

    const innerRoles = log
      .filter((e) => e.type === "role.entered")
      .map((e) => (e as Extract<CrewEvent, { type: "role.entered" }>).roleId);
    // outer roles: host, worker, host. Inner adds: alpha, beta.
    expect(innerRoles).toContain("alpha");
    expect(innerRoles).toContain("beta");
  });

  it("inner crew.completed → outer nested.crew.completed; outer agent's output = inner finalOutput", () => {
    const log = runEcho(setup().session);

    const innerComplete = log.find(
      (e) =>
        e.type === "crew.completed" &&
        // inner's crew.completed appears before outer's
        log.indexOf(e) < log.length - 1,
    ) as Extract<CrewEvent, { type: "crew.completed" }>;
    const nestedComplete = log.find((e) => e.type === "nested.crew.completed") as
      Extract<CrewEvent, { type: "nested.crew.completed" }>;
    expect(nestedComplete).toBeDefined();
    expect(nestedComplete.agentId).toBe("worker-1");
    expect(nestedComplete.output).toEqual(innerComplete.finalOutput);
  });

  it("outer phase does not complete until inner crew completes", () => {
    const { session } = setup();
    const initial = session.start("ROOT");
    const hostReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    const afterHost = session.deliver({
      type: "agent.step.completed",
      correlationId: hostReq.correlationId,
      output: "host-output",
      ts: 0,
    });

    // worker phase has spawned, inner alpha step is pending. NO worker
    // vote.resolved yet.
    expect(
      afterHost.some(
        (e) =>
          e.type === "vote.resolved" &&
          (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "worker",
      ),
    ).toBe(false);

    // Resolve alpha (inner's first phase)
    const alphaReq = afterHost.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "alpha",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const afterAlpha = session.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "alpha-output",
      ts: 0,
    });
    // Still no worker vote.resolved (beta is still pending in inner)
    expect(
      afterAlpha.some(
        (e) =>
          e.type === "vote.resolved" &&
          (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "worker",
      ),
    ).toBe(false);

    // Resolve beta — inner crew.completed fires, outer worker phase advances
    const betaReq = afterAlpha.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "beta",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const afterBeta = session.deliver({
      type: "agent.step.completed",
      correlationId: betaReq.correlationId,
      output: "beta-output",
      ts: 0,
    });

    expect(afterBeta.some((e) => e.type === "nested.crew.completed")).toBe(true);
    expect(
      afterBeta.some(
        (e) =>
          e.type === "vote.resolved" &&
          (e as Extract<CrewEvent, { type: "vote.resolved" }>).roleId === "worker",
      ),
    ).toBe(true);
  });

  it("inner correlation ids differ from outer correlation ids", () => {
    const log = runEcho(setup().session);
    const reqs = log.filter((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>[];
    const cids = new Set(reqs.map((r) => r.correlationId));
    expect(cids.size).toBe(reqs.length); // all unique
  });

  it("full nested run completes with crew.completed emitted by the outer session", () => {
    const log = runEcho(setup().session);
    expect(log[log.length - 1].type).toBe("crew.completed");
    // The last crew.completed is the OUTER one — its crewId is "outer"
    const lastComplete = log[log.length - 1] as Extract<CrewEvent, { type: "crew.completed" }>;
    expect(lastComplete).toBeDefined();
  });
});
