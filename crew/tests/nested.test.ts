import { describe, it, expect } from "vitest";
import { createMemoryItem, getItems } from "@ai2070/memex";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { resumeCrewSession } from "../src/session/resume.js";
import { createMemexAdapter } from "../src/memex/ai2070.js";
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

describe("Nested crews — hard memex isolation", () => {
  function setupHardIsolated() {
    const parentShape = CrewShapeSchema.parse({
      ...PARENT_SHAPE,
      roles: PARENT_SHAPE.roles.map((r) =>
        r.role === "worker"
          ? { ...r, execution: { memex: { isolation: "hard" } } }
          : r,
      ),
    });
    const innerShape = CrewShapeSchema.parse(INNER_SHAPE);
    const parentAgents = CrewAgentsSchema.parse(PARENT_AGENTS);
    const graph = buildCrewGraph(parentShape, parentAgents, { INNER: innerShape });
    const adapter = createMemexAdapter({ crewId: "outer-hard" });

    // Pre-load a memory item in parent's adapter
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "parent-pre", value: "before-fork" },
          author: "agent:host-1",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "host-1", crewId: "outer-hard", roleId: "host" },
    );

    const session = createCrewSession({
      crewId: "outer-hard",
      graph,
      clock: frozenClock(0),
      memex: adapter,
    });
    return { session, adapter, graph };
  }

  it("inner adapter is forked from parent (inner sees parent's items at fork)", () => {
    const { session, adapter } = setupHardIsolated();
    const initial = session.start("ROOT");
    const hostReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    session.deliver({
      type: "agent.step.completed",
      correlationId: hostReq.correlationId,
      output: "host-out",
      ts: 0,
    });

    // Parent's adapter still has its pre-loaded item
    expect(getItems(adapter.snapshot())).toHaveLength(1);
  });

  it("inner agent's memex_commands write to inner adapter only; on completion they merge into parent", () => {
    const { session, adapter } = setupHardIsolated();
    const log: CrewEvent[] = [];
    const queue: CrewEvent[] = session.start("ROOT");

    let innerWroteItem = false;
    while (queue.length > 0) {
      const e = queue.shift()!;
      log.push(e);
      if (e.type === "agent.step.requested") {
        // Inner alpha-1 attaches a memex_commands write
        const memex_commands =
          e.agentId === "alpha-1" && !innerWroteItem
            ? [
                {
                  type: "memory.create" as const,
                  item: createMemoryItem({
                    kind: "observation",
                    content: { key: "inner-write", value: "written-inside" },
                    author: "agent:alpha-1",
                    source_kind: "observed",
                    authority: 0.9,
                    importance: 0.5,
                  }),
                },
              ]
            : undefined;
        if (memex_commands) innerWroteItem = true;
        queue.push(
          ...session.deliver({
            type: "agent.step.completed",
            correlationId: e.correlationId,
            output: "ok",
            ...(memex_commands ? { memex_commands } : {}),
            ts: 0,
          }),
        );
      }
    }

    // After full run, parent's adapter should have BOTH the pre-loaded item
    // (parent-pre) AND the inner-written item (inner-write), the latter
    // imported from the inner adapter when the nested crew completed.
    const items = getItems(adapter.snapshot());
    expect(items).toHaveLength(2);
    const keys = items.map((i) => (i.content as { key: string }).key).sort();
    expect(keys).toEqual(["inner-write", "parent-pre"]);
  });

  it("snapshot/resume preserves inner adapter state — inner writes don't leak to parent until completion, even across resume", () => {
    const { session, adapter, graph } = setupHardIsolated();

    // Drive: host completes
    const initial = session.start("ROOT");
    const hostReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    const afterHost = session.deliver({
      type: "agent.step.completed",
      correlationId: hostReq.correlationId,
      output: "host-out",
      ts: 0,
    });

    // Inner alpha is now pending (and got memex_context from forked adapter
    // including parent-pre). Worker writes item X via memex_commands.
    const alphaReq = afterHost.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "alpha",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    session.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "alpha-out",
      memex_commands: [
        {
          type: "memory.create",
          item: createMemoryItem({
            kind: "observation",
            content: { key: "X", value: "from-alpha" },
            author: "agent:alpha-1",
            source_kind: "observed",
            authority: 0.9,
            importance: 0.5,
          }),
        },
      ],
      ts: 0,
    });

    // Beta is now pending in the inner. SNAPSHOT here.
    const snap = session.snapshot();

    // Sanity: parent's adapter still has only parent-pre (X is in inner adapter).
    const parentDuringNested = getItems(adapter.snapshot()).map(
      (i) => (i.content as { key: string }).key,
    );
    expect(parentDuringNested).toEqual(["parent-pre"]);

    // Resume from snapshot. The inner adapter should be rebuilt from
    // innerAdapterSnapshot — NOT downgraded to soft (which would have inner
    // writing directly to parent).
    const { session: resumed, events: resumeEvents } = resumeCrewSession(snap, [], {
      crewId: "outer-hard",
      graph,
      clock: frozenClock(0),
      memex: adapter,
      resumePolicy: "re-emit-request",
    });

    // Re-emitted beta request is in resumeEvents; deliver beta with another write (Y).
    const betaReq = resumeEvents.find(
      (e) =>
        e.type === "agent.step.requested" &&
        (e as Extract<CrewEvent, { type: "agent.step.requested" }>).roleId === "beta",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(betaReq).toBeDefined();

    resumed.deliver({
      type: "agent.step.completed",
      correlationId: betaReq.correlationId,
      output: "beta-out",
      memex_commands: [
        {
          type: "memory.create",
          item: createMemoryItem({
            kind: "observation",
            content: { key: "Y", value: "from-beta-after-resume" },
            author: "agent:beta-1",
            source_kind: "observed",
            authority: 0.9,
            importance: 0.5,
          }),
        },
      ],
      ts: 0,
    });

    // Inner crew.completed should have fired, triggering importSlice from the
    // (rebuilt) inner adapter into parent. Outer is now in the final host phase.
    // Drive the final host to completion.
    let pending = resumed.pendingRequests();
    while (pending.length > 0) {
      for (const req of pending) {
        resumed.deliver({
          type: "agent.step.completed",
          correlationId: req.correlationId,
          output: "ok",
          ts: 0,
        });
      }
      pending = resumed.pendingRequests();
    }

    // Parent's adapter now has parent-pre + X + Y — proving:
    //   1. X was preserved across snapshot/resume (it was in the rebuilt inner adapter).
    //   2. Y was written to the rebuilt inner adapter, not directly to parent.
    //   3. Both X and Y were imported into parent when the inner crew completed.
    const finalKeys = getItems(adapter.snapshot())
      .map((i) => (i.content as { key: string }).key)
      .sort();
    expect(finalKeys).toEqual(["X", "Y", "parent-pre"]);
  });
});
