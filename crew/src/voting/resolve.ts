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

// resolveVotes is called with entries already in deterministic order
// (by graph.agents declaration order) so tie-breaking is stable across runs.
export function resolveVotes(votes: VoteEntry[], mode: VotingMode): unknown {
  const valid = votes.filter((v) => !v.fault && !v.stalled);

  switch (mode) {
    case "first_valid":
      return valid.length > 0 ? valid[0].output : undefined;

    case "majority": {
      if (valid.length === 0) return undefined;
      const counts = new Map<string, { output: unknown; count: number }>();
      for (const v of valid) {
        const key = canonicalize(v.output);
        const existing = counts.get(key);
        if (existing) {
          existing.count++;
        } else {
          counts.set(key, { output: v.output, count: 1 });
        }
      }
      // Map preserves insertion order — first-seen wins on ties, which is
      // deterministic given declaration-order input.
      let best: { output: unknown; count: number } | undefined;
      for (const entry of counts.values()) {
        if (!best || entry.count > best.count) best = entry;
      }
      return best?.output;
    }

    case "unanimous":
    case "weighted_consensus":
    case "best_of_n":
      throw new NotImplementedError(`voting mode "${mode}"`);

    default: {
      const _exhaustive: never = mode;
      throw new Error(`unknown voting mode: ${String(_exhaustive)}`);
    }
  }
}
