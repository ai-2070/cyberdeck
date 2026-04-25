import { describe, it } from "vitest";

describe("CrewSession state machine", () => {
  it.todo("start() emits crew.started + caller's agent.step.requested");
  it.todo("deliver(agent.step.completed) advances to the next phase");
  it.todo("parallel mercs all emit requests, machine waits for all to resolve");
  it.todo("phase outputs sorted by agentId before voting (determinism)");
  it.todo("duplicate terminal deliver is a no-op (returns [])");
  it.todo("agent.stream.chunk passes through without advancing state");
  it.todo("tick(now) emits agent.step.timed_out for expired requests");
  it.todo("cancel() emits crew.aborted and transitions to aborted");
  it.todo("permission.denied emitted when requestAction is disallowed");
  it.todo("agent.step.requested carries a self-contained role snapshot");
});
