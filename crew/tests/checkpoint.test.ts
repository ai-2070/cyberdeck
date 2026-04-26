import { describe, it, expect } from "vitest";
import { CrewShapeSchema } from "../src/schema/shape.js";
import { CrewAgentsSchema } from "../src/schema/agents.js";
import { buildCrewGraph } from "../src/graph/build.js";
import { createCrewSession } from "../src/session/machine.js";
import { resumeCrewSession } from "../src/session/resume.js";
import { frozenClock } from "../src/runtime/clock.js";
import { createInMemoryCheckpointStore } from "../src/checkpoint/store.js";
import { canonicalize } from "../src/events/canonical.js";
import { createHookRegistry } from "../src/runtime/hooks.js";
import type { CrewEvent } from "../src/events/types.js";
import type { CrewSession } from "../src/session/types.js";
import { TINY_CREW_SHAPE, TINY_CREW_AGENTS } from "./fixtures/tiny-crew.js";

function tinySetup() {
  const shape = CrewShapeSchema.parse(TINY_CREW_SHAPE);
  const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
  const graph = buildCrewGraph(shape, counts);
  const clock = frozenClock(0);
  return { shape, counts, graph, clock };
}

function newSession(graph: ReturnType<typeof tinySetup>["graph"]) {
  return createCrewSession({
    crewId: "ckpt-test",
    graph,
    clock: frozenClock(0),
  });
}

// Drive an echo run, optionally pausing after `pauseAfter` requests have been
// answered so we can snapshot mid-flight.
function runEchoUntil(
  session: CrewSession,
  rootInput: unknown,
  predicate: (log: CrewEvent[]) => boolean,
): { log: CrewEvent[]; remaining: CrewEvent[] } {
  const log: CrewEvent[] = [];
  const queue: CrewEvent[] = session.start(rootInput);

  while (queue.length > 0) {
    if (predicate(log)) break;
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
  return { log, remaining: queue };
}

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

describe("CheckpointStore", () => {
  it("put/get round-trips a snapshot", () => {
    const { graph } = tinySetup();
    const session = newSession(graph);
    session.start("ROOT");

    const store = createInMemoryCheckpointStore();
    store.put("phase-end-1", session.snapshot());

    const retrieved = store.get("phase-end-1");
    expect(retrieved).toBeDefined();
    expect(retrieved!.crewId).toBe("ckpt-test");
  });

  it("list returns checkpoint ids; delete removes them", () => {
    const { graph } = tinySetup();
    const session = newSession(graph);
    session.start("ROOT");

    const store = createInMemoryCheckpointStore();
    store.put("a", session.snapshot());
    store.put("b", session.snapshot());
    expect(new Set(store.list())).toEqual(new Set(["a", "b"]));

    store.delete("a");
    expect(store.list()).toEqual(["b"]);
    expect(store.get("a")).toBeUndefined();
  });

  it("autoCheckpoint='phase' emits checkpoint.taken after every vote.resolved", () => {
    const { graph } = tinySetup();
    const session = createCrewSession({
      crewId: "auto-ckpt",
      graph,
      clock: frozenClock(0),
      autoCheckpoint: "phase",
    });
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
            output: "ok",
            ts: 0,
          }),
        );
      }
    }
    const checkpoints = log.filter(
      (e) => e.type === "checkpoint.taken",
    ) as Extract<CrewEvent, { type: "checkpoint.taken" }>[];
    // 3 phases (alpha, beta, gamma) -> 3 phase checkpoints
    expect(checkpoints).toHaveLength(3);
    expect(checkpoints[0].checkpointId).toBe("phase:alpha:0");
    expect(checkpoints[1].checkpointId).toBe("phase:beta:1");
    expect(checkpoints[2].checkpointId).toBe("phase:gamma:2");
  });

  it("declarative role.execution.checkpoints emits one event per declared name", () => {
    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "alpha"
          ? { ...r, execution: { checkpoints: ["sync-1", "sync-2"] } }
          : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "decl-ckpt",
      graph,
      clock: frozenClock(0),
    });

    const initial = session.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst = session.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: "ok",
      ts: 0,
    });

    const checkpoints = burst.filter(
      (e) => e.type === "checkpoint.taken",
    ) as Extract<CrewEvent, { type: "checkpoint.taken" }>[];
    expect(checkpoints.map((c) => c.checkpointId)).toEqual([
      "sync-1",
      "sync-2",
    ]);
  });

  it("checkpoint.taken event is emitted when crewControl.checkpoint(id) is called", () => {
    const hooks = createHookRegistry({
      ckpt: (ctx) => ctx.control.checkpoint("phase-end"),
    });
    const shape = CrewShapeSchema.parse({
      ...TINY_CREW_SHAPE,
      roles: TINY_CREW_SHAPE.roles.map((r) =>
        r.role === "alpha" ? { ...r, hooks: { after_role: "ckpt" } } : r,
      ),
    });
    const counts = CrewAgentsSchema.parse(TINY_CREW_AGENTS);
    const graph = buildCrewGraph(shape, counts);
    const session = createCrewSession({
      crewId: "ckpt",
      graph,
      clock: frozenClock(0),
      hooks,
    });

    const initial = session.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    const burst = session.deliver({
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

describe("Snapshot + resume round-trip", () => {
  it("snapshot at completion + resume + zero events == same final state", () => {
    const { graph } = tinySetup();

    // Fresh run
    const sessA = newSession(graph);
    const logA = runEcho(sessA);
    const snap = sessA.snapshot();
    expect(snap.status).toBe("completed");

    // Resume from snapshot (no later events)
    const { session: sessB, events: laterB } = resumeCrewSession(snap, [], {
      crewId: "ckpt-test",
      graph,
      clock: frozenClock(0),
    });
    expect(sessB.status()).toBe("completed");
    expect(laterB).toEqual([]); // nothing more to emit
    expect(sessB.snapshot().currentInput).toEqual(
      sessA.snapshot().currentInput,
    );
  });

  it("snapshot mid-phase + drive resumed session forward = same final state as fresh run", () => {
    const { graph } = tinySetup();

    const sessA = newSession(graph);
    runEcho(sessA);
    const fresh = sessA.snapshot();

    // Run B: stop after delivering 1 completion (alpha-1 done, betas now pending)
    const sessB = newSession(graph);
    const initial = sessB.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    sessB.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: { from: "alpha-1" },
      ts: 0,
    });
    const snap = sessB.snapshot();

    // Resume with re-emit-request (so resume doesn't throw on the pending betas).
    const { session: sessC, events: resumeEvents } = resumeCrewSession(
      snap,
      [],
      {
        crewId: "ckpt-test",
        graph,
        clock: frozenClock(0),
        resumePolicy: "re-emit-request",
      },
    );

    // The re-emitted requests are the betas — echo them
    const echoQueue: CrewEvent[] = resumeEvents
      .filter((e) => e.type === "agent.step.requested")
      .map((e) => {
        const r = e as Extract<CrewEvent, { type: "agent.step.requested" }>;
        return {
          type: "agent.step.completed",
          correlationId: r.correlationId,
          output: { from: r.agentId },
          ts: 0,
        };
      });

    while (echoQueue.length > 0) {
      const completion = echoQueue.shift()!;
      const next = sessC.deliver(completion);
      for (const ne of next) {
        if (ne.type === "agent.step.requested") {
          const r = ne as Extract<CrewEvent, { type: "agent.step.requested" }>;
          echoQueue.push({
            type: "agent.step.completed",
            correlationId: r.correlationId,
            output: { from: r.agentId },
            ts: 0,
          });
        }
      }
    }

    expect(sessC.status()).toBe("completed");
    expect(canonicalize(sessC.snapshot().currentInput)).toBe(
      canonicalize(fresh.currentInput),
    );
  });

  it("snapshot captures pending requests with their inputs (re-emit-able)", () => {
    const { graph } = tinySetup();
    const session = newSession(graph);
    session.start("ROOT");

    const snap = session.snapshot();
    expect(snap.currentPhase).not.toBeNull();
    expect(snap.currentPhase!.pending.length).toBeGreaterThan(0);
    for (const p of snap.currentPhase!.pending) {
      expect(p.input).toBe("ROOT"); // alpha phase received the rootInput
    }
  });

  it("snapshot and resume preserves canonicalized currentInput across phase advances", () => {
    const { graph } = tinySetup();
    const sessA = newSession(graph);

    const initial = sessA.start("ROOT");
    const alphaReq = initial.find(
      (e) => e.type === "agent.step.requested",
    ) as Extract<CrewEvent, { type: "agent.step.requested" }>;
    sessA.deliver({
      type: "agent.step.completed",
      correlationId: alphaReq.correlationId,
      output: { processed: true },
      ts: 0,
    });

    // Now in beta phase. Snapshot.
    const snap = sessA.snapshot();
    expect(snap.phaseIndex).toBe(1); // beta phase
    expect(canonicalize(snap.currentInput)).toBe(
      canonicalize({ processed: true }),
    );

    // Resume in a parallel universe (use re-emit so the unresolved betas don't abort).
    const { session: sessB } = resumeCrewSession(snap, [], {
      crewId: "ckpt-test",
      graph,
      clock: frozenClock(0),
      resumePolicy: "re-emit-request",
    });
    expect(sessB.status()).toBe("awaiting_responses");
    expect(
      sessB
        .pendingRequests()
        .map((r) => r.agentId)
        .sort(),
    ).toEqual(["beta-1", "beta-2"]);
  });
});
