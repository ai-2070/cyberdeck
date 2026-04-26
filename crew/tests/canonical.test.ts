import { describe, it, expect } from "vitest";
import { canonicalize } from "../src/events/canonical.js";
import { correlationId, hashHex } from "../src/runtime/ids.js";

describe("canonicalize", () => {
  it("emits primitives the same way JSON.stringify does", () => {
    expect(canonicalize(null)).toBe("null");
    expect(canonicalize(true)).toBe("true");
    expect(canonicalize(false)).toBe("false");
    expect(canonicalize(0)).toBe("0");
    expect(canonicalize(42)).toBe("42");
    expect(canonicalize(3.14)).toBe("3.14");
    expect(canonicalize("hello")).toBe('"hello"');
    expect(canonicalize('with "quotes"')).toBe('"with \\"quotes\\""');
  });

  it("normalizes -0 to 0", () => {
    expect(canonicalize(-0)).toBe("0");
    expect(canonicalize([0, -0, 0])).toBe("[0,0,0]");
  });

  it("rejects non-finite numbers", () => {
    expect(() => canonicalize(NaN)).toThrow(/non-finite/);
    expect(() => canonicalize(Infinity)).toThrow(/non-finite/);
    expect(() => canonicalize(-Infinity)).toThrow(/non-finite/);
  });

  it("rejects unsupported value types", () => {
    expect(() => canonicalize(undefined)).toThrow(/undefined/);
    expect(() => canonicalize(123n)).toThrow(/bigint/);
    expect(() => canonicalize(() => 0)).toThrow(/function/);
    expect(() => canonicalize(Symbol("x"))).toThrow(/symbol/);
  });

  it("is stable across object key permutations", () => {
    const a = canonicalize({ a: 1, b: 2, c: 3 });
    const b = canonicalize({ c: 3, a: 1, b: 2 });
    const c = canonicalize({ b: 2, c: 3, a: 1 });
    expect(a).toBe(b);
    expect(b).toBe(c);
    expect(a).toBe('{"a":1,"b":2,"c":3}');
  });

  it("is stable across nested object key permutations", () => {
    const a = canonicalize({ outer: { x: 1, y: 2 }, top: "z" });
    const b = canonicalize({ top: "z", outer: { y: 2, x: 1 } });
    expect(a).toBe(b);
  });

  it("preserves array order (arrays are sequences, not sets)", () => {
    expect(canonicalize([3, 1, 2])).toBe("[3,1,2]");
    expect(canonicalize([1, 2, 3])).toBe("[1,2,3]");
  });

  it("preserves length of sparse arrays (holes become null)", () => {
    const sparse = [1, , 3]; // length 3, hole at index 1
    expect(canonicalize(sparse)).toBe("[1,null,3]");

    const arr = new Array(3); // length 3, all holes
    arr[1] = "x";
    expect(canonicalize(arr)).toBe('[null,"x",null]');
  });

  it("skips undefined values in objects but converts undefined in arrays to null", () => {
    expect(canonicalize({ a: 1, b: undefined, c: 3 })).toBe('{"a":1,"c":3}');
    expect(canonicalize([1, undefined, 3])).toBe("[1,null,3]");
  });

  it("detects circular references", () => {
    const obj: Record<string, unknown> = { a: 1 };
    obj.self = obj;
    expect(() => canonicalize(obj)).toThrow(/circular/);

    const arr: unknown[] = [1, 2];
    arr.push(arr);
    expect(() => canonicalize(arr)).toThrow(/circular/);
  });

  it("handles deeply nested structures", () => {
    const value = {
      events: [
        { type: "a", id: 1 },
        { type: "b", id: 2, payload: { nested: { deep: true } } },
      ],
      count: 2,
    };
    const reordered = {
      count: 2,
      events: [
        { id: 1, type: "a" },
        { payload: { nested: { deep: true } }, type: "b", id: 2 },
      ],
    };
    expect(canonicalize(value)).toBe(canonicalize(reordered));
  });

  it("escapes string values via JSON.stringify rules", () => {
    expect(canonicalize("line1\nline2")).toBe('"line1\\nline2"');
    expect(canonicalize("tab\there")).toBe('"tab\\there"');
  });
});

describe("hashHex / correlationId", () => {
  it("hashHex returns a 16-char hex string", () => {
    const h = hashHex("anything");
    expect(h).toMatch(/^[0-9a-f]{16}$/);
  });

  it("hashHex is deterministic for the same input", () => {
    expect(hashHex("foo")).toBe(hashHex("foo"));
    expect(hashHex("")).toBe(hashHex(""));
  });

  it("hashHex differs for different inputs", () => {
    expect(hashHex("foo")).not.toBe(hashHex("bar"));
    expect(hashHex("foo")).not.toBe(hashHex("Foo"));
    expect(hashHex("foo")).not.toBe(hashHex("foo "));
  });

  it("correlationId is deterministic for the same parts", () => {
    const a = correlationId({
      crewId: "c1",
      roleId: "merc",
      agentId: "merc-1",
      attempt: 1,
    });
    const b = correlationId({
      crewId: "c1",
      roleId: "merc",
      agentId: "merc-1",
      attempt: 1,
    });
    expect(a).toBe(b);
  });

  it("correlationId differs across distinct (crew, role, agent, attempt) tuples", () => {
    const base = {
      crewId: "c1",
      roleId: "merc",
      agentId: "merc-1",
      attempt: 1,
    };
    const seen = new Set<string>();
    seen.add(correlationId(base));
    seen.add(correlationId({ ...base, crewId: "c2" }));
    seen.add(correlationId({ ...base, roleId: "specialist" }));
    seen.add(correlationId({ ...base, agentId: "merc-2" }));
    seen.add(correlationId({ ...base, attempt: 2 }));
    expect(seen.size).toBe(5);
  });

  it("correlationId is invariant under part-key permutations (canonicalized)", () => {
    // Different object key order, same logical parts → same id.
    const a = correlationId({
      crewId: "c1",
      roleId: "merc",
      agentId: "merc-1",
      attempt: 1,
    });
    const b = correlationId({
      attempt: 1,
      agentId: "merc-1",
      roleId: "merc",
      crewId: "c1",
    });
    expect(a).toBe(b);
  });
});
