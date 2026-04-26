import type { CrewRoleSpec } from "../schema/shape.js";

export type RoleId = string;
export type AgentId = string;

export type CrewRole = CrewRoleSpec;

export interface CrewAgent {
  id: AgentId;
  role: RoleId;
  nestedCrew?: CrewGraph;
}

export interface CrewGraph {
  name: string;
  roles: Map<RoleId, CrewRole>;
  agents: CrewAgent[];
}
