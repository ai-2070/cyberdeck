// @ai2070/crew — public surface
// Phase 1: schemas + graph builder + linter
// Phase 2: event vocabulary + canonical JSON + correlation ids
// Phase 3: state machine session (start/deliver/tick/cancel) + voting + hooks + permissions + fixer + nested
// Phase 4: MemEX adapter (memex_context out, memex_commands in)
// Phase 5: snapshot / resume / checkpoint store

export { CrewShapeSchema } from "./schema/shape.js";
export type {
  CrewShape,
  CrewRoleSpec,
  CrewRolePermissions,
  CrewRoleCapabilities,
  CrewRoleActivation,
  CrewRoleExecution,
  CrewRoleMemex,
  CrewInvitePermission,
  MemexIsolation,
} from "./schema/shape.js";

export { CrewAgentsSchema } from "./schema/agents.js";
export type { CrewAgents, CrewAgentCount } from "./schema/agents.js";

export { ConsensusConfigSchema, VotingModeSchema } from "./schema/consensus.js";
export type { ConsensusConfig, VotingMode } from "./schema/consensus.js";

export { LifecycleHooksSchema } from "./schema/hooks.js";
export type { LifecycleHooks } from "./schema/hooks.js";

export { buildCrewGraph, lintCrewShape } from "./graph/build.js";
export type { LintIssue, LintResult } from "./graph/build.js";

export type {
  CrewGraph,
  CrewRole,
  CrewAgent,
  RoleId,
  AgentId,
} from "./graph/types.js";

export type {
  CrewEvent,
  CrewEventType,
  OutboundEventType,
  InboundEventType,
  TerminalInboundType,
  RoleSnapshot,
  FixerReason,
  GatedAction,
  AbortReason,
  AgentStepRequest,
  MemoryCommand,
  MemoryItemShape,
  EdgeShape,
} from "./events/types.js";
export { isInboundEvent, isTerminalInbound } from "./events/types.js";

export { canonicalize } from "./events/canonical.js";

export { correlationId, hashHex } from "./runtime/ids.js";
export type { CorrelationParts } from "./runtime/ids.js";

export { systemClock, frozenClock } from "./runtime/clock.js";
export type { Clock, MutableClock } from "./runtime/clock.js";

export { createCrewSession } from "./session/machine.js";
export { resumeCrewSession } from "./session/resume.js";
export type { ResumeResult } from "./session/resume.js";
export type {
  CrewSession,
  CrewSnapshot,
  CrewStatus,
  ResumePolicy,
  CreateCrewSessionOpts,
  AgentStepDetail,
  SerializedPhaseState,
  SerializedPendingEntry,
  SerializedNestedHandle,
} from "./session/types.js";

export { createInMemoryCheckpointStore } from "./checkpoint/store.js";
export type { CheckpointStore } from "./checkpoint/types.js";

// MemEX adapter (peer dep — `@ai2070/memex` resolved at runtime).
export type {
  AgentMemexHandle,
  MemexAdapter,
  MemexStampContext,
  MemexView,
} from "./memex/adapter.js";

export { resolveVotes, NotImplementedError } from "./voting/resolve.js";
export type { VoteEntry } from "./voting/resolve.js";

export { createHookRegistry } from "./runtime/hooks.js";
export type {
  HookFn,
  HookRegistry,
  HookContext,
  HookStage,
  CrewControl,
} from "./runtime/hooks.js";

export { checkPermission } from "./session/permissions.js";

export type { ActionRequest, RequestActionResult } from "./session/types.js";
