import { z } from "zod";

export const LifecycleHooksSchema = z.object({
  before_role: z.string().optional(),
  after_role: z.string().optional(),
  before_agent: z.string().optional(),
  after_agent: z.string().optional(),
  on_fault: z.string().optional(),
  on_stall: z.string().optional(),
  on_timeout: z.string().optional(),
  on_permission_denied: z.string().optional(),
  on_checkpoint: z.string().optional(),
});

export type LifecycleHooks = z.infer<typeof LifecycleHooksSchema>;
