import { describe, it, expect } from "vitest";
import { createGraphState, getItems, createMemoryItem } from "@ai2070/memex";
import type { MemoryCommand, MemoryItem } from "@ai2070/memex";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import { createMemexAdapter } from "../src/memex/ai2070.js";
import type { CrewEvent } from "../src/events/types.js";
import { TINY_CREW_SHAPE, TINY_CREW_AGENTS } from "./fixtures/tiny-crew.js";

describe("createMemexAdapter — unit", () => {
  it("apply() stamps memory.create with author / meta / scope", () => {
    const adapter = createMemexAdapter({ crewId: "c1" });
    const item = createMemoryItem({
      kind: "observation",
      content: { key: "x", value: 1 },
      author: "", // empty — adapter should fill
      source_kind: "observed",
      authority: 0.8,
      importance: 0.5,
    });
    adapter.apply(
      { type: "memory.create", item },
      { agentId: "merc-1", crewId: "c1", roleId: "merc" },
    );

    const state = adapter.snapshot();
    const items = getItems(state);
    expect(items).toHaveLength(1);
    const got = items[0];
    expect(got.author).toBe("agent:merc-1");
    expect(got.meta?.agent_id).toBe("merc-1");
    expect(got.meta?.crew_id).toBe("c1");
    expect(got.meta?.role).toBe("merc");
    expect(got.scope).toBe("crew:c1/agent:merc-1");
  });

  it("apply() preserves explicit author / scope when set by the worker", () => {
    const adapter = createMemexAdapter({ crewId: "c1" });
    const item = createMemoryItem({
      kind: "observation",
      content: { key: "x", value: 1 },
      author: "agent:custom",
      scope: "custom:scope",
      source_kind: "observed",
      authority: 0.8,
      importance: 0.5,
    });
    adapter.apply(
      { type: "memory.create", item },
      { agentId: "merc-1", crewId: "c1", roleId: "merc" },
    );

    const got = getItems(adapter.snapshot())[0];
    expect(got.author).toBe("agent:custom");
    expect(got.scope).toBe("custom:scope");
    // meta is still stamped
    expect(got.meta?.agent_id).toBe("merc-1");
  });

  it("handleFor with view='self' filters by meta.agent_id", () => {
    const adapter = createMemexAdapter({ crewId: "c1" });

    // Two agents write items
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "k", value: "alpha-says" },
          author: "",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "alpha-1", crewId: "c1", roleId: "alpha" },
    );
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "k", value: "beta-says" },
          author: "",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "beta-1", crewId: "c1", roleId: "beta" },
    );

    const alphaAgent = { id: "alpha-1", role: "alpha" };
    const alphaRole = { role: "alpha", permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] } } as unknown as Parameters<typeof adapter.handleFor>[1];
    const handle = adapter.handleFor(alphaAgent as Parameters<typeof adapter.handleFor>[0], alphaRole, "self");
    const items = handle.read();
    expect(items).toHaveLength(1);
    expect(items[0].meta?.agent_id).toBe("alpha-1");
  });

  it("handleFor with view='crew' returns items from any agent in the crew", () => {
    const adapter = createMemexAdapter({ crewId: "c1" });
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "k", value: 1 },
          author: "",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "alpha-1", crewId: "c1", roleId: "alpha" },
    );
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "k", value: 2 },
          author: "",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "beta-1", crewId: "c1", roleId: "beta" },
    );

    const alphaAgent = { id: "alpha-1", role: "alpha" } as Parameters<typeof adapter.handleFor>[0];
    const alphaRole = { role: "alpha", permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] } } as Parameters<typeof adapter.handleFor>[1];
    const handle = adapter.handleFor(alphaAgent, alphaRole, "crew");
    const items = handle.read();
    expect(items).toHaveLength(2);
  });
});

describe("Session × MemEX integration", () => {
  it("agent.step.requested carries memex_context when retrieval is configured", () => {
    const adapter = createMemexAdapter({ crewId: "ctx-test" });
    // Pre-load one item authored by alpha-1
    adapter.apply(
      {
        type: "memory.create",
        item: createMemoryItem({
          kind: "observation",
          content: { key: "preload", value: "hello" },
          author: "",
          source_kind: "observed",
          authority: 0.9,
          importance: 0.5,
        }),
      },
      { agentId: "alpha-1", crewId: "ctx-test", roleId: "alpha" },
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
                  view: "self",
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
      crewId: "ctx-test",
      graph,
      clock: frozenClock(0),
      memex: adapter,
    });

    const initial = session.start("ROOT");
    const alphaReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(alphaReq.memex_context).toBeDefined();
    expect(Array.isArray(alphaReq.memex_context)).toBe(true);
  });

  it("no memex_context when memex.enabled is false", () => {
    const adapter = createMemexAdapter({ crewId: "off-test" });
    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "alpha"
          ? { ...r, execution: { memex: { enabled: false } } }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "off-test",
      graph,
      clock: frozenClock(0),
      memex: adapter,
    });
    const initial = session.start("ROOT");
    const alphaReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(alphaReq.memex_context).toBeUndefined();
  });

  it("no memex_context when no adapter is configured", () => {
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
                    budget: 100,
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
      crewId: "no-adapter",
      graph,
      clock: frozenClock(0),
    });
    const initial = session.start("ROOT");
    const alphaReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;
    expect(alphaReq.memex_context).toBeUndefined();
  });

  it("memex_commands in agent.step.completed → memex.command.emitted + adapter applies", () => {
    const adapter = createMemexAdapter({ crewId: "write-test" });
    const shape = CrewShapeSchema.parse(TINY_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "write-test",
      graph,
      clock: frozenClock(0),
      memex: adapter,
    });

    const initial = session.start("ROOT");
    const alphaReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const cmd: MemoryCommand = {
      type: "memory.create",
      item: createMemoryItem({
        kind: "observation",
        content: { key: "from-worker", value: "hi" },
        author: "",
        source_kind: "observed",
        authority: 0.9,
        importance: 0.5,
      }),
    };

    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "ok",
      memex_commands: [cmd],
      ts: 0,
    });

    const emitted = burst.find((e) => e.type === "memex.command.emitted") as
      Extract<CrewEvent, { type: "memex.command.emitted" }>;
    expect(emitted).toBeDefined();
    expect(emitted.agentId).toBe("alpha-1");
    expect(emitted.command).toBe(cmd);

    // Adapter applied the command — alpha's stamped item should be in the graph
    const items = getItems(adapter.snapshot()) as MemoryItem[];
    expect(items).toHaveLength(1);
    expect(items[0].meta?.agent_id).toBe("alpha-1");
    expect(items[0].meta?.crew_id).toBe("write-test");
  });

  it("session works without memex even when memex_commands are sent (no-op)", () => {
    const shape = CrewShapeSchema.parse(TINY_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "no-mem",
      graph,
      clock: frozenClock(0),
    });
    const initial = session.start("ROOT");
    const alphaReq = initial.find((e) => e.type === "agent.step.requested") as
      Extract<CrewEvent, { type: "agent.step.requested" }>;

    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "ok",
      memex_commands: [
        {
          type: "memory.create",
          item: createMemoryItem({
            kind: "observation",
            content: { key: "x", value: 1 },
            author: "agent:alpha-1",
            source_kind: "observed",
            authority: 0.9,
            importance: 0.5,
          }),
        },
      ],
      ts: 0,
    });

    // Event still emitted (replay-friendly), but adapter is undefined so no apply.
    expect(burst.some((e) => e.type === "memex.command.emitted")).toBe(true);
    // Phase still advances normally
    expect(burst.some((e) => e.type === "vote.resolved")).toBe(true);
  });
});
