import { describe, it } from "vitest";

describe("Replay determinism", () => {
  it.todo("identical inbound event sequence produces byte-identical outbound log (after canonicalize)");
  it.todo("ResumePolicy 'abort' refuses to resume if a step is requested but unresolved");
  it.todo("ResumePolicy 're-emit-request' replays the missing request");
});
