import type { AgentId, CrewGraph } from "../graph/types.js";
import type { GatedAction } from "../events/types.js";

// ACL gate. The library validates whether `from` is allowed to perform `action`
// against `to`'s role. It does NOT spawn delegated steps, route messages, or
// create subflows — that's the worker's or hook's responsibility. See
// PLAN.md Phase 3 §5.
export function checkPermission(
  graph: CrewGraph,
  fromAgentId: AgentId,
  toAgentId: AgentId,
  action: GatedAction,
): boolean {
  const fromAgent = graph.agents.find((a) => a.id === fromAgentId);
  const toAgent = graph.agents.find((a) => a.id === toAgentId);
  if (!fromAgent || !toAgent) return false;
  const fromRole = graph.roles.get(fromAgent.role);
  if (!fromRole) return false;
  return fromRole.permissions[action].includes(toAgent.role);
}
