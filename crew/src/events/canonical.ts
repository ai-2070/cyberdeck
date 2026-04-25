// Stable JSON serialization for hashing, dedup, and replay assertions.
// Stock JSON.stringify is not stable across object key order.

export function canonicalize(value: unknown): string {
  return canonicalizeValue(value, new WeakSet<object>());
}

function canonicalizeValue(value: unknown, seen: WeakSet<object>): string {
  if (value === null) return "null";

  const t = typeof value;

  if (t === "boolean") return value ? "true" : "false";

  if (t === "number") {
    const n = value as number;
    if (!Number.isFinite(n)) {
      throw new Error(`canonicalize: non-finite number ${n} is not representable`);
    }
    return Object.is(n, -0) ? "0" : String(n);
  }

  if (t === "string") return JSON.stringify(value);

  if (t === "bigint") {
    throw new Error("canonicalize: bigint is not supported");
  }

  if (t === "undefined") {
    throw new Error("canonicalize: undefined is not representable at the top level");
  }

  if (t === "function" || t === "symbol") {
    throw new Error(`canonicalize: ${t} is not representable`);
  }

  if (Array.isArray(value)) {
    if (seen.has(value)) throw new Error("canonicalize: circular reference");
    seen.add(value);
    const parts = value.map((v) =>
      v === undefined ? "null" : canonicalizeValue(v, seen),
    );
    seen.delete(value);
    return `[${parts.join(",")}]`;
  }

  if (t === "object") {
    const obj = value as Record<string, unknown>;
    if (seen.has(obj)) throw new Error("canonicalize: circular reference");
    seen.add(obj);
    const keys = Object.keys(obj).sort();
    const parts: string[] = [];
    for (const key of keys) {
      const v = obj[key];
      if (v === undefined) continue;
      parts.push(`${JSON.stringify(key)}:${canonicalizeValue(v, seen)}`);
    }
    seen.delete(obj);
    return `{${parts.join(",")}}`;
  }

  throw new Error(`canonicalize: unsupported value type ${t}`);
}
