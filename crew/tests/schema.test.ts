import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { DEFAULT_CREW_SHAPE, DEFAULT_CREW_AGENTS } from "./fixtures/default-crew.js";

describe("CrewShapeSchema", () => {
  it("parses the DEFAULT_CREW example", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    expect(shape.name).toBe("DEFAULT_CREW");
    expect(shape.roles).toHaveLength(4);
    expect(shape.roles.map((r) => r.role)).toEqual([
      "merc",
      "specialist",
      "fixer",
      "caller",
    ]);
  });

  it("rejects shapes without schema_version", () => {
    const { schema_version: _omit, ...rest } = DEFAULT_CREW_SHAPE;
    expect(() => CrewShapeSchema.parse(rest)).toThrow();
  });

  it("rejects shapes whose schema_version is not '1.0'", () => {
    const bad = { ...DEFAULT_CREW_SHAPE, schema_version: "2.0" };
    expect(() => CrewShapeSchema.parse(bad)).toThrow();
  });

  it("applies defaults: capabilities.thinking_allowed defaults to true", () => {
    const shape = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "TINY",
      roles: [
        {
          role: "only",
          permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
          first_input: true,
          final_output: true,
        },
      ],
    });
    expect(shape.roles[0].capabilities.thinking_allowed).toBe(true);
  });

  it("applies defaults: permissions arrays default to []", () => {
    const shape = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "TINY",
      roles: [
        {
          role: "only",
          permissions: {},
          first_input: true,
          final_output: true,
        },
      ],
    });
    expect(shape.roles[0].permissions.talk_to).toEqual([]);
    expect(shape.roles[0].permissions.delegate_to).toEqual([]);
    expect(shape.roles[0].permissions.escalate_to).toEqual([]);
    expect(shape.roles[0].permissions.invite).toEqual([]);
  });

  it("accepts both string and structured invite entries", () => {
    const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
    const merc = shape.roles.find((r) => r.role === "merc")!;
    const fixer = shape.roles.find((r) => r.role === "fixer")!;
    expect(merc.permissions.invite).toHaveLength(1);
    expect(typeof merc.permissions.invite[0]).toBe("object");
    expect(fixer.permissions.invite).toEqual(["merc", "specialist"]);
  });

  it("requires at least one role", () => {
    expect(() =>
      CrewShapeSchema.parse({ schema_version: "1.0", name: "EMPTY", roles: [] }),
    ).toThrow();
  });

  it("requires at least one trigger flag when activation is provided", () => {
    expect(() =>
      CrewShapeSchema.parse({
        schema_version: "1.0",
        name: "BAD",
        roles: [
          {
            role: "dead",
            permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
            activation: {}, // empty — silently dead
            first_input: true,
            final_output: true,
          },
        ],
      }),
    ).toThrow();
  });

  it("activation with explicit false flags is also rejected (no triggers set to true)", () => {
    expect(() =>
      CrewShapeSchema.parse({
        schema_version: "1.0",
        name: "BAD",
        roles: [
          {
            role: "dead",
            permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
            activation: { on_fault: false, on_stall: false },
            first_input: true,
            final_output: true,
          },
        ],
      }),
    ).toThrow();
  });

  it("activation with at least one trigger set to true is accepted", () => {
    const shape = CrewShapeSchema.parse({
      schema_version: "1.0",
      name: "OK",
      roles: [
        {
          role: "alpha",
          permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
          first_input: true,
        },
        {
          role: "beta",
          permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
          final_output: true,
        },
        {
          role: "fixer",
          permissions: { talk_to: [], delegate_to: [], escalate_to: [], invite: [] },
          activation: { on_fault: true },
        },
      ],
    });
    expect(shape.roles).toHaveLength(3);
  });
});

describe("CrewAgentsSchema", () => {
  it("parses the DEFAULT_CREW counts", () => {
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    expect(counts.name).toBe("DEFAULT_CREW");
    expect(counts.agents).toHaveLength(4);
  });

  it("rejects negative amounts", () => {
    const bad = {
      ...DEFAULT_CREW_AGENTS,
      agents: [{ role: "merc", amount: -1 }],
    };
    expect(() => CrewAgentsSchema.parse(bad)).toThrow();
  });

  it("rejects non-integer amounts", () => {
    const bad = {
      ...DEFAULT_CREW_AGENTS,
      agents: [{ role: "merc", amount: 1.5 }],
    };
    expect(() => CrewAgentsSchema.parse(bad)).toThrow();
  });

  it("rejects schema_version other than '1.0'", () => {
    const bad = { ...DEFAULT_CREW_AGENTS, schema_version: "2.0" };
    expect(() => CrewAgentsSchema.parse(bad)).toThrow();
  });
});
