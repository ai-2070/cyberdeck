import { z } from "zod";

const CrewAgentCountSchema = z.object({
  role: z.string().min(1),
  amount: z.number().int().min(0),
});

export const CrewAgentsSchema = z.object({
  schema_version: z.literal("1.0"),
  name: z.string().min(1),
  agents: z.array(CrewAgentCountSchema),
});

export type CrewAgents = z.infer<typeof CrewAgentsSchema>;
export type CrewAgentCount = z.infer<typeof CrewAgentCountSchema>;
