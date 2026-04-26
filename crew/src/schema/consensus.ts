import { z } from "zod";

export const VotingModeSchema = z.enum([
  "first_valid",
  "unanimous",
  "weighted_consensus",
  "majority",
  "best_of_n",
]);

export type VotingMode = z.infer<typeof VotingModeSchema>;

export const ConsensusConfigSchema = z.object({
  mode: VotingModeSchema.default("weighted_consensus"),
  weight_function: z.string().default("equal"),
  threshold: z.number().min(0).max(1).default(0.66),
});

export type ConsensusConfig = z.infer<typeof ConsensusConfigSchema>;
