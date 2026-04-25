import { describe, it } from "vitest";

describe("Nested crews", () => {
  it.todo("inner session correlation ids are prefixed with outer correlation id");
  it.todo("nested.crew.completed.output becomes the outer agent's step output");
  it.todo("hard isolation calls exportSlice before nested run, importSlice after");
  it.todo("soft isolation shares the parent MemexAdapter view");
});
