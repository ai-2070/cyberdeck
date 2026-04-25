import { describe, it } from "vitest";

describe("Checkpoint + resume", () => {
  it.todo("checkpoint at a phase boundary captures session + memex state");
  it.todo("resume + later events produces the same final log as a fresh run");
  it.todo("checkpoint.taken event is emitted when crewControl.checkpoint(id) is called");
});
