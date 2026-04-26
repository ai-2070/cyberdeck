import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { frozenClock } from "../src/runtime/clock.js";
import type { CrewEvent } from "../src/events/types.js";
import {
  DEFAULT_CREW_SHAPE,
  DEFAULT_CREW_AGENTS,
} from "./fixtures/default-crew.js";

function setup() {
  const shape = CrewShapeSchema.parse(DEFAULT_CREW_SHAPE);
  const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
  const graph = buildCrewGraph(shape, counts);
  const session = createCrewSession({
    crewId: "perm-test",
    graph,
    clock: frozenClock(0),
  });
  return { session, graph };
}

describe("Permission ACL gates", () => {
  it("requestAction returns true when the source role allows the action", () => {
    const { session } = setup();
    // merc.permissions.talk_to = ["fixer"] — merc-1 → fixer-1 is allowed
    const result = session.requestAction({
      from: "merc-1",
      to: "fixer-1",
      action: "talk_to",
    });
    expect(result.allowed).toBe(true);
    expect(result.events).toEqual([]);
  });

  it("requestAction returns false and emits permission.denied when not allowed", () => {
    const { session } = setup();
    // merc.permissions.talk_to does NOT include caller — denied
    const result = session.requestAction({
      from: "merc-1",
      to: "caller-1",
      action: "talk_to",
    });
    expect(result.allowed).toBe(false);
    const denial = result.events.find(
      (e) => e.type === "permission.denied",
    ) as Extract<CrewEvent, { type: "permission.denied" }>;
    expect(denial).toBeDefined();
    expect(denial.from).toBe("merc-1");
    expect(denial.to).toBe("caller-1");
    expect(denial.action).toBe("talk_to");
  });

  it("requestAction returns false for delegate_to denial", () => {
    const { session } = setup();
    // merc has no delegate_to permissions
    const result = session.requestAction({
      from: "merc-1",
      to: "specialist-1",
      action: "delegate_to",
    });
    expect(result.allowed).toBe(false);
  });

  it("requestAction returns false when source agent doesn't exist", () => {
    const { session } = setup();
    const result = session.requestAction({
      from: "ghost",
      to: "fixer-1",
      action: "talk_to",
    });
    expect(result.allowed).toBe(false);
  });

  it("on_permission_denied does NOT activate fixer when no role flags it", () => {
    // DEFAULT_CREW only flags on_fault and on_stall, not on_permission_denied.
    const { session } = setup();
    const result = session.requestAction({
      from: "merc-1",
      to: "caller-1",
      action: "talk_to",
    });
    expect(result.events.some((e) => e.type === "fixer.invoked")).toBe(false);
  });

  it("on_permission_denied activation routes to fixer when flagged", () => {
    // Custom shape: enable on_permission_denied on the fixer
    const shape = CrewShapeSchema.parse({
      ...DEFAULT_CREW_SHAPE,
      roles: DEFAULT_CREW_SHAPE.roles.map((r) =>
        r.role === "fixer"
          ? {
              ...r,
              activation: { ...r.activation, on_permission_denied: true },
            }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(DEFAULT_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "perm-fixer",
      graph,
      clock: frozenClock(0),
    });
    // The session needs to be in a phase for fixer activation to spawn a step.
    session.start("ROOT");
    const result = session.requestAction({
      from: "merc-1",
      to: "caller-1",
      action: "talk_to",
    });
    expect(result.allowed).toBe(false);
    expect(result.events.some((e) => e.type === "fixer.invoked")).toBe(true);
    expect(
      result.events.some(
        (e) =>
          e.type === "agent.step.requested" &&
          (e as Extract<CrewEvent, { type: "agent.step.requested" }>)
            .agentId === "fixer-1",
      ),
    ).toBe(true);
  });

  it("the loop never spawns delegated agent.step.requested events on its own", () => {
    // Even when permission is allowed, the loop does NOT auto-spawn a step
    // for the target. ACL gate only.
    const { session } = setup();
    const result = session.requestAction({
      from: "merc-1",
      to: "fixer-1",
      action: "talk_to",
    });
    expect(result.allowed).toBe(true);
    expect(result.events.some((e) => e.type === "agent.step.requested")).toBe(
      false,
    );
  });
});
