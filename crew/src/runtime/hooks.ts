import type { AgentId, CrewAgent, CrewRole } from "../graph/types.js";
import type { GatedAction } from "../events/types.js";

export type HookStage =
  | "before_role"
  | "after_role"
  | "before_agent"
  | "after_agent"
  | "on_fault"
  | "on_stall"
  | "on_timeout"
  | "on_permission_denied"
  | "on_checkpoint";

// crewControl is the in-hook surface for emitting state changes back into the
// session's current outbound event burst. It's synchronous; calls take effect
// immediately within the same start/deliver/tick burst.
export interface CrewControl {
  requestAction(req: {
    from: AgentId;
    to: AgentId;
    action: GatedAction;
  }): boolean;
  checkpoint(id: string): void;
}

export interface HookContext {
  crewId: string;
  stage: HookStage;
  role: CrewRole;
  agent?: CrewAgent;
  input?: unknown;
  output?: unknown;
  fault?: boolean;
  stalled?: boolean;
  control: CrewControl;
}

// Hooks are sync. No Promise<void>, no awaits. If a hook needs async work
// it fires it off out-of-band — see PLAN.md Phase 3 §9.
export type HookFn = (ctx: HookContext) => void;

export interface HookRegistry {
  get(name: string): HookFn | undefined;
}

export function createHookRegistry(hooks: Record<string, HookFn>): HookRegistry {
  return {
    get: (name) => hooks[name],
  };
}
