import { z } from "zod";

const CrewAgentCountSchema = z.object({
  role: z.string().min(1),
  amount: z.number().int().min(0),
  // Optional per-role system prompt override. When present, it replaces the
  // shape's system_prompt for this role at buildCrewGraph time.
  system_prompt: z.string().optional(),
});

export const CrewAgentsSchema = z.object({
  schema_version: z.literal("1.0"),
  name: z.string().min(1),
  agents: z.array(CrewAgentCountSchema),
});

export type CrewAgents = z.infer<typeof CrewAgentsSchema>;
export type CrewAgentCount = z.infer<typeof CrewAgentCountSchema>;
