// Deterministic, non-cryptographic 64-bit hash (FNV-1a) over canonical JSON.
// Used for correlation ids that tie agent.step.requested to its terminal events.
// Same inputs always produce the same id; different inputs produce different ids
// (collision probability is acceptable for in-process correlation, not for security).

import { canonicalize } from "../events/canonical.js";

const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const MASK_64 = 0xffffffffffffffffn;

export function hashHex(input: string): string {
  let hash = FNV_OFFSET;
  for (let i = 0; i < input.length; i++) {
    hash ^= BigInt(input.charCodeAt(i));
    hash = (hash * FNV_PRIME) & MASK_64;
  }
  return hash.toString(16).padStart(16, "0");
}

export interface CorrelationParts {
  crewId: string;
  roleId: string;
  agentId: string;
  attempt: number;
}

export function correlationId(parts: CorrelationParts): string {
  return hashHex(canonicalize(parts));
}
