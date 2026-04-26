import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph, lintCrewShape } from "../src/graph/build.js";
import {
  DEFAULT_CREW_SHAPE,
  DEFAULT_CREW_AGENTS,
} from "./fixtures/default-crew.js";

describe("buildCrewGraph", () => {
  it("builds a graph with the right agent counts from DEFAULT_CREW", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);

    expect(graph.name).toBe("DEFAULT_CREW");
    expect(graph.roles.size).toBe(4);
    expect(graph.agents).toHaveLength(7);
    expect(graph.agents.filter((a) => a.role === "merc")).toHaveLength(4);
    expect(graph.agents.filter((a) => a.role === "specialist")).toHaveLength(1);
    expect(graph.agents.filter((a) => a.role === "fixer")).toHaveLength(1);
    expect(graph.agents.filter((a) => a.role === "caller")).toHaveLength(1);
  });

  it("agent ids are deterministic: {role}-{i+1}", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);

    const ids = graph.agents.map((a) => a.id);
    expect(ids).toEqual([
      "merc-1",
      "merc-2",
      "merc-3",
      "merc-4",
      "specialist-1",
      "fixer-1",
      "caller-1",
    ]);
  });

  it("preserves declaration order in agent generation", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);

    const order = graph.agents.map((a) => a.role);
    expect(order).toEqual([
      "merc",
      "merc",
      "merc",
      "merc",
      "specialist",
      "fixer",
      "caller",
    ]);
  });

  it("respects max_allowed caps (throws on excess)", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse({
      ...DEFAULT_CREW_AGENTS,
      agents: [
        { role: "merc", amount: 4 },
        { role: "specialist", amount: 1 },
        { role: "fixer", amount: 5 },
        { role: "caller", amount: 1 },
      ],
    });
    expect(() => buildCrewGraph(shape, counts)).toThrow(/exceeds max_allowed/);
  });

  it("falls back to role.amount when count is missing", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "DEFAULT_CREW",
      agents: [
        { role: "specialist", amount: 1 },
        { role: "fixer", amount: 1 },
        { role: "caller", amount: 1 },
      ],
    });
    const graph = buildCrewGraph(shape, counts);
    // merc.amount = 4 from shape default, no override in counts
    expect(graph.agents.filter((a) => a.role === "merc")).toHaveLength(4);
  });

  it("rejects counts that reference an unknown role", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "DEFAULT_CREW",
      agents: [{ role: "ghost", amount: 1 }],
    });
    expect(() => buildCrewGraph(shape, counts)).toThrow(/unknown role/);
  });

  it("agents-config system_prompt overrides the role's shape default", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "merc"
          ? { ...r, system_prompt: "default-merc-prompt" }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "DEFAULT_CREW",
      agents: [
        { role: "merc", amount: 4, system_prompt: "research-bird-sleep" },
        { role: "specialist", amount: 1 },
        { role: "fixer", amount: 1 },
        { role: "caller", amount: 1 },
      ],
    });
    const graph = buildCrewGraph(shape, counts);
    expect(graph.roles.get("merc")!.system_prompt).toBe("research-bird-sleep");
    // Role without override keeps its shape default (or undefined)
    expect(graph.roles.get("specialist")!.system_prompt).toBeUndefined();
  });

  it("agents-config without system_prompt leaves the shape default in place", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "merc"
          ? { ...r, system_prompt: "default-merc-prompt" }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    expect(graph.roles.get("merc")!.system_prompt).toBe("default-merc-prompt");
  });

  it("throws on name mismatch between shape and counts", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const counts = CrewAgentsSchema.parse({
      ...DEFAULT_CREW_AGENTS,
      name: "WRONG",
    });
    expect(() => buildCrewGraph(shape, counts)).toThrow(/name mismatch/i);
  });

  it("refuses to build when lint reports errors", () => {
    // Shape with two final_output roles — lint error
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer" ? { ...r, final_output: true } : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    expect(() => buildCrewGraph(shape, counts)).toThrow(/failed lint/i);
  });
});

describe("lintCrewShape", () => {
  it("passes the DEFAULT_CREW with no errors", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const lint = lintCrewShape(shape);
    expect(lint.errors).toEqual([]);
  });

  it("errors when no role has first_input", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "caller" ? { ...r, first_input: false } : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "missing_first_input")).toBe(
      true,
    );
  });

  it("errors when more than one role has first_input", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer" ? { ...r, first_input: true } : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "multiple_first_input")).toBe(
      true,
    );
  });

  it("errors when no role has final_output", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "caller" ? { ...r, final_output: false } : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "missing_final_output")).toBe(
      true,
    );
  });

  it("errors when more than one role has final_output", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer" ? { ...r, final_output: true } : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "multiple_final_output")).toBe(
      true,
    );
  });

  it("errors when a permission references an unknown role", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "specialist"
          ? { ...r, permissions: { ...r.permissions, talk_to: ["ghost"] } }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "unknown_role_reference")).toBe(
      true,
    );
  });

  it("errors when invite references an unknown role (string form)", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer"
          ? { ...r, permissions: { ...r.permissions, invite: ["ghost"] } }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "unknown_role_reference")).toBe(
      true,
    );
  });

  it("errors when invite references an unknown role (object form)", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "merc"
          ? {
              ...r,
              permissions: {
                ...r.permissions,
                invite: [{ role: "ghost" }],
              },
            }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "unknown_role_reference")).toBe(
      true,
    );
  });

  it("errors when a role escalates to itself", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "specialist"
          ? {
              ...r,
              permissions: { ...r.permissions, escalate_to: ["specialist"] },
            }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "self_escalation")).toBe(true);
  });

  it("warns when a role can delegate_to a peer it cannot talk_to", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer"
          ? {
              ...r,
              permissions: {
                ...r.permissions,
                talk_to: ["merc", "caller"], // dropped specialist
              },
            }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.warnings.some((w) => w.code === "delegate_without_talk")).toBe(
      true,
    );
  });

  it("DEFAULT_CREW produces no warnings either", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const lint = lintCrewShape(shape);
    expect(lint.warnings).toEqual([]);
  });

  it("errors on unsupported voting mode (best_of_n)", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "merc"
          ? {
              ...r,
              execution: {
                voting: {
                  mode: "best_of_n",
                  weight_function: "equal",
                  threshold: 0.5,
                },
              },
            }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(lint.errors.some((e) => e.code === "unsupported_voting_mode")).toBe(
      true,
    );
  });

  it("errors on unsupported weight_function for weighted_consensus", () => {
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "merc"
          ? {
              ...r,
              execution: {
                voting: {
                  mode: "weighted_consensus",
                  weight_function: "by_confidence",
                  threshold: 0.66,
                },
              },
            }
          : r,
      ),
    });
    const lint = lintCrewShape(shape);
    expect(
      lint.errors.some((e) => e.code === "unsupported_weight_function"),
    ).toBe(true);
  });
});

describe("buildCrewGraph — cycle detection", () => {
  it("throws on direct self-cycle", () => {
    const shape = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "SELF",
      roles: [
        {
          role: "only",
          permissions: {
            talk_to: [],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          first_input: true,
          final_output: true,
          nested_crew: "SELF",
          amount: 1,
        },
      ],
    });
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "SELF",
      agents: [{ role: "only", amount: 1 }],
    });
    expect(() => buildCrewGraph(shape, counts, { SELF: shape })).toThrow(
      /Cyclic nested crew/i,
    );
  });

  it("throws on multi-step cycle (A → B → A)", () => {
    const A = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "A",
      roles: [
        {
          role: "x",
          permissions: {
            talk_to: [],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          first_input: true,
          final_output: true,
          nested_crew: "B",
          amount: 1,
        },
      ],
    });
    const B = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "B",
      roles: [
        {
          role: "y",
          permissions: {
            talk_to: [],
            delegate_to: [],
            escalate_to: [],
            invite: [],
          },
          first_input: true,
          final_output: true,
          nested_crew: "A",
          amount: 1,
        },
      ],
    });
    const counts = CrewAgentsSchema.parse({
      schema_version: "1.0",
      name: "A",
      agents: [{ role: "x", amount: 1 }],
    });
    expect(() => buildCrewGraph(A, counts, { A, B })).toThrow(
      /Cyclic nested crew/i,
    );
  });
});
