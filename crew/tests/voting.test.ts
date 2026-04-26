import { describe, it, expect } from "vitest";
import { resolveVotes, NotImplementedError, type VoteEntry } from "../src/voting/resolve.js";

describe("resolveVotes", () => {
  describe("first_valid", () => {
    it("returns the first non-faulted output", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x" },
        { agentId: "a-2", output: "y" },
      ];
      expect(resolveVotes(votes, "first_valid")).toBe("x");
    });

    it("skips faulted entries", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined, fault: true },
        { agentId: "a-2", output: "y" },
      ];
      expect(resolveVotes(votes, "first_valid")).toBe("y");
    });

    it("skips stalled entries", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x", stalled: true },
        { agentId: "a-2", output: "y" },
      ];
      expect(resolveVotes(votes, "first_valid")).toBe("y");
    });

    it("returns undefined when all entries are invalid", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined, fault: true },
        { agentId: "a-2", output: undefined, stalled: true },
      ];
      expect(resolveVotes(votes, "first_valid")).toBeUndefined();
    });

    it("returns undefined for empty input", () => {
      expect(resolveVotes([], "first_valid")).toBeUndefined();
    });
  });

  describe("majority", () => {
    it("picks the most common output", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x" },
        { agentId: "a-2", output: "y" },
        { agentId: "a-3", output: "y" },
      ];
      expect(resolveVotes(votes, "majority")).toBe("y");
    });

    it("uses canonicalize so object key order doesn't fragment votes", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: { a: 1, b: 2 } },
        { agentId: "a-2", output: { b: 2, a: 1 } },
        { agentId: "a-3", output: { a: 1, b: 9 } },
      ];
      const resolved = resolveVotes(votes, "majority") as { a: number; b: number };
      expect(resolved.a).toBe(1);
      expect(resolved.b).toBe(2);
    });

    it("breaks ties via first-occurrence in input order", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x" },
        { agentId: "a-2", output: "y" },
      ];
      // both have count 1, x came first
      expect(resolveVotes(votes, "majority")).toBe("x");
    });

    it("ignores faulted/stalled entries", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x", fault: true },
        { agentId: "a-2", output: "y" },
        { agentId: "a-3", output: "y" },
      ];
      expect(resolveVotes(votes, "majority")).toBe("y");
    });

    it("returns undefined when no valid entries", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined, fault: true },
      ];
      expect(resolveVotes(votes, "majority")).toBeUndefined();
    });
  });

  describe("unanimous", () => {
    it("returns the value when all valid outputs are equal", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "yes" },
        { agentId: "a-2", output: "yes" },
        { agentId: "a-3", output: "yes" },
      ];
      expect(resolveVotes(votes, "unanimous")).toBe("yes");
    });

    it("returns undefined when outputs differ", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "yes" },
        { agentId: "a-2", output: "no" },
      ];
      expect(resolveVotes(votes, "unanimous")).toBeUndefined();
    });

    it("uses canonicalize so key order doesn't break unanimity", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: { a: 1, b: 2 } },
        { agentId: "a-2", output: { b: 2, a: 1 } },
      ];
      expect(resolveVotes(votes, "unanimous")).toEqual({ a: 1, b: 2 });
    });

    it("ignores faulted entries", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined, fault: true },
        { agentId: "a-2", output: "yes" },
        { agentId: "a-3", output: "yes" },
      ];
      expect(resolveVotes(votes, "unanimous")).toBe("yes");
    });
  });

  describe("weighted_consensus (equal weights)", () => {
    it("returns the value that meets the threshold", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "yes" },
        { agentId: "a-2", output: "yes" },
        { agentId: "a-3", output: "no" },
      ];
      // 2/3 = 0.66, threshold 0.66 → "yes" wins
      const result = resolveVotes(votes, {
        mode: "weighted_consensus",
        weight_function: "equal",
        threshold: 0.66,
      });
      expect(result).toBe("yes");
    });

    it("returns undefined when no value meets the threshold", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "x" },
        { agentId: "a-2", output: "y" },
        { agentId: "a-3", output: "z" },
      ];
      const result = resolveVotes(votes, {
        mode: "weighted_consensus",
        threshold: 0.66,
      });
      expect(result).toBeUndefined();
    });

    it("uses canonicalize so key order doesn't fragment buckets", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: { a: 1, b: 2 } },
        { agentId: "a-2", output: { b: 2, a: 1 } },
        { agentId: "a-3", output: { a: 9 } },
      ];
      const result = resolveVotes(votes, {
        mode: "weighted_consensus",
        threshold: 0.5,
      }) as { a: number; b: number };
      expect(result).toEqual({ a: 1, b: 2 });
    });

    it("throws NotImplementedError for non-equal weight functions", () => {
      const votes: VoteEntry[] = [{ agentId: "a", output: "x" }];
      expect(() =>
        resolveVotes(votes, {
          mode: "weighted_consensus",
          weight_function: "by_confidence",
          threshold: 0.5,
        }),
      ).toThrow(NotImplementedError);
    });

    it("defaults to threshold 0.66 and weight_function 'equal' when omitted", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: "yes" },
        { agentId: "a-2", output: "yes" },
        { agentId: "a-3", output: "no" },
      ];
      // Default 0.66 threshold, 2/3 wins
      expect(resolveVotes(votes, "weighted_consensus")).toBe("yes");
    });
  });

  describe("undefined outputs", () => {
    it("majority does not crash on a vote with output: undefined", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined },
        { agentId: "a-2", output: undefined },
        { agentId: "a-3", output: "yes" },
      ];
      // Two undefined votes, one "yes" → undefined wins (majority)
      expect(resolveVotes(votes, "majority")).toBeUndefined();
    });

    it("unanimous treats all-undefined as agreement", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined },
        { agentId: "a-2", output: undefined },
      ];
      expect(resolveVotes(votes, "unanimous")).toBeUndefined();
    });

    it("unanimous distinguishes undefined from defined values", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined },
        { agentId: "a-2", output: "yes" },
      ];
      // Different outputs → no consensus
      expect(resolveVotes(votes, "unanimous")).toBeUndefined();
    });

    it("weighted_consensus does not crash on undefined votes", () => {
      const votes: VoteEntry[] = [
        { agentId: "a-1", output: undefined },
        { agentId: "a-2", output: undefined },
        { agentId: "a-3", output: "yes" },
      ];
      // 2/3 undefined, threshold 0.66 → undefined wins
      expect(resolveVotes(votes, "weighted_consensus")).toBeUndefined();
    });
  });

  describe("not implemented modes", () => {
    it("best_of_n throws NotImplementedError", () => {
      expect(() => resolveVotes([{ agentId: "a", output: "x" }], "best_of_n"))
        .toThrow(NotImplementedError);
    });
  });
});
