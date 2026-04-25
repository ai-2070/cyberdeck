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

  describe("not implemented modes", () => {
    it("unanimous throws NotImplementedError", () => {
      expect(() => resolveVotes([{ agentId: "a", output: "x" }], "unanimous"))
        .toThrow(NotImplementedError);
    });

    it("weighted_consensus throws NotImplementedError", () => {
      expect(() => resolveVotes([{ agentId: "a", output: "x" }], "weighted_consensus"))
        .toThrow(NotImplementedError);
    });

    it("best_of_n throws NotImplementedError", () => {
      expect(() => resolveVotes([{ agentId: "a", output: "x" }], "best_of_n"))
        .toThrow(NotImplementedError);
    });
  });
});
