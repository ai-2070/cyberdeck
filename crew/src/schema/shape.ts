import { z } from "zod";
import { ConsensusConfigSchema } from "./consensus.js";
import { LifecycleHooksSchema } from "./hooks.js";

const RoleCapabilitiesSchema = z.object({
  thinking_allowed: z.boolean().default(true),
  tools: z.array(z.string()).optional(),
  model: z.string().optional(),
});

const InvitePermissionSchema = z.union([
  z.string(),
  z.object({
    role: z.string(),
    consensus: ConsensusConfigSchema.optional(),
  }),
]);

const RolePermissionsSchema = z.object({
  talk_to: z.array(z.string()).default([]),
  delegate_to: z.array(z.string()).default([]),
  escalate_to: z.array(z.string()).default([]),
  invite: z.array(InvitePermissionSchema).default([]),
  modify_structure: z.boolean().optional(),
});

// `activation` excludes a role from normal phase order. Empty activation =
// silently dead role. Force callers to specify at least one trigger so the
// "fixer-style" intent is explicit.
const RoleActivationSchema = z
  .object({
    on_fault: z.boolean().optional(),
    on_stall: z.boolean().optional(),
    on_timeout: z.boolean().optional(),
    on_permission_denied: z.boolean().optional(),
  })
  .refine(
    (v) => Object.values(v).some((flag) => flag === true),
    "activation must include at least one trigger flag set to true",
  );

const RoleDelegationSchema = z.object({
  max_depth: z.number().int().min(0).default(0),
});

const MemexViewSchema = z.enum(["self", "role", "crew", "all"]);
const MemexIsolationSchema = z.enum(["soft", "hard"]);

const RoleMemexSchema = z.object({
  view: MemexViewSchema.optional(),
  isolation: MemexIsolationSchema.optional(),
  enabled: z.boolean().optional(),
  // Pass-through SmartRetrievalOptions (validated structurally by memex itself).
  retrieval: z.unknown().optional(),
});

const RoleExecutionSchema = z.object({
  voting: ConsensusConfigSchema.optional(),
  checkpoints: z.array(z.string()).optional(),
  timeout_ms: z.number().int().min(0).optional(),
  serial: z.boolean().optional(),
  memex: RoleMemexSchema.optional(),
});

const CrewRoleSchema = z.object({
  role: z.string().min(1),
  description: z.string().optional(),
  capabilities: RoleCapabilitiesSchema.default({ thinking_allowed: true }),
  permissions: RolePermissionsSchema,
  delegation: RoleDelegationSchema.optional(),
  amount: z.number().int().min(0).optional(),
  max_allowed: z.number().int().min(1).optional(),
  activation: RoleActivationSchema.optional(),
  first_input: z.boolean().optional(),
  final_output: z.boolean().optional(),
  nested_crew: z.string().optional(),
  hooks: LifecycleHooksSchema.optional(),
  execution: RoleExecutionSchema.optional(),
});

export const CrewShapeSchema = z.object({
  schema_version: z.literal("1.0"),
  name: z.string().min(1),
  description: z.string().optional(),
  roles: z.array(CrewRoleSchema).min(1),
});

export type CrewShape = z.infer<typeof CrewShapeSchema>;
export type CrewRoleSpec = z.infer<typeof CrewRoleSchema>;
export type CrewRolePermissions = z.infer<typeof RolePermissionsSchema>;
export type CrewRoleCapabilities = z.infer<typeof RoleCapabilitiesSchema>;
export type CrewRoleActivation = z.infer<typeof RoleActivationSchema>;
export type CrewRoleExecution = z.infer<typeof RoleExecutionSchema>;
export type CrewRoleMemex = z.infer<typeof RoleMemexSchema>;
export type MemexView = z.infer<typeof MemexViewSchema>;
export type MemexIsolation = z.infer<typeof MemexIsolationSchema>;
export type CrewInvitePermission = z.infer<typeof InvitePermissionSchema>;
