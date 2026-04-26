import type { VotingMode } from "../schema/consensus.js";
import { canonicalize } from "../events/canonical.js";

export class NotImplementedError extends Error {
  constructor(feature: string) {
    super(`Not implemented in v1: ${feature}`);
    this.name = "NotImplementedError";
  }
}

export interface VoteEntry {
  agentId: string;
  output: unknown;
  fault?: boolean;
  stalled?: boolean;
}

// Either a bare VotingMode (= mode-only) or a ConsensusConfig-shaped object.
export type VotingConfig =
  | VotingMode
  | { mode: VotingMode; weight_function?: string; threshold?: number };

// resolveVotes is called with entries already in deterministic order
// (by graph.agents declaration order) so tie-breaking is stable across runs.
export function resolveVotes(votes: VoteEntry[], config: VotingConfig): unknown {
  const cfg =
    typeof config === "string"
      ? { mode: config, weight_function: "equal", threshold: 0.66 }
      : {
          mode: config.mode,
          weight_function: config.weight_function ?? "equal",
          threshold: config.threshold ?? 0.66,
        };

  const valid = votes.filter((v) => !v.fault && !v.stalled);

  switch (cfg.mode) {
    case "first_valid":
      return valid.length > 0 ? valid[0].output : undefined;

    case "majority": {
      if (valid.length === 0) return undefined;
      const counts = bucketByCanonical(valid);
      let best: { output: unknown; weight: number } | undefined;
      for (const entry of counts.values()) {
        if (!best || entry.weight > best.weight) best = entry;
      }
      return best?.output;
    }

    case "unanimous": {
      if (valid.length === 0) return undefined;
      const first = canonicalize(valid[0].output);
      const allSame = valid.every((v) => canonicalize(v.output) === first);
      return allSame ? valid[0].output : undefined;
    }

    case "weighted_consensus": {
      if (valid.length === 0) return undefined;
      if (cfg.weight_function !== "equal") {
        throw new NotImplementedError(
          `weight_function "${cfg.weight_function}" (v1 supports "equal" only)`,
        );
      }
      const totalWeight = valid.length; // equal weights = 1 each
      const buckets = bucketByCanonical(valid);
      // Map preserves insertion order (declaration order) so first-bucket wins on ties.
      for (const entry of buckets.values()) {
        if (entry.weight / totalWeight >= cfg.threshold) {
          return entry.output;
        }
      }
      return undefined;
    }

    case "best_of_n":
      // best_of_n requires a per-output score that VoteEntry doesn't carry.
      // Callers can supply their own resolver before calling resolveVotes.
      throw new NotImplementedError(
        `voting mode "${cfg.mode}" requires per-output scoring (provide your own resolver in v1)`,
      );

    default: {
      const _exhaustive: never = cfg.mode;
      throw new Error(`unknown voting mode: ${String(_exhaustive)}`);
    }
  }
}

function bucketByCanonical(
  entries: VoteEntry[],
): Map<string, { output: unknown; weight: number }> {
  const out = new Map<string, { output: unknown; weight: number }>();
  for (const v of entries) {
    const key = canonicalize(v.output);
    const existing = out.get(key);
    if (existing) {
      existing.weight += 1;
    } else {
      out.set(key, { output: v.output, weight: 1 });
    }
  }
  return out;
}
