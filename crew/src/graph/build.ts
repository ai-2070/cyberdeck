import type { CrewShape } from "../schema/shape.js";
import type { CrewAgents } from "../schema/agents.js";
import type { CrewAgent, CrewGraph, CrewRole, RoleId } from "./types.js";

export interface LintIssue {
  code: string;
  message: string;
  path?: string;
}

export interface LintResult {
  errors: LintIssue[];
  warnings: LintIssue[];
}

export function buildCrewGraph(
  shape: CrewShape,
  agentsCfg: CrewAgents,
  registry?: Record<string, CrewShape>,
  visited: ReadonlySet<string> = new Set(),
): CrewGraph {
  if (shape.name !== agentsCfg.name) {
    throw new Error(
      `Crew name mismatch: shape="${shape.name}" agents="${agentsCfg.name}"`,
    );
  }
  if (visited.has(shape.name)) {
    throw new Error(
      `Cyclic nested crew reference: "${shape.name}" is already being expanded in this chain`,
    );
  }
  const nextVisited = new Set(visited).add(shape.name);

  const lint = lintCrewShape(shape);
  if (lint.errors.length > 0) {
    const summary = lint.errors.map((e) => `[${e.code}] ${e.message}`).join("; ");
    throw new Error(`CrewShape failed lint: ${summary}`);
  }

  const roles = new Map<RoleId, CrewRole>();
  for (const r of shape.roles) {
    roles.set(r.role, r);
  }

  const requested = new Map<RoleId, number>();
  for (const a of agentsCfg.agents) {
    if (!roles.has(a.role)) {
      throw new Error(`Agent count references unknown role: "${a.role}"`);
    }
    requested.set(a.role, a.amount);
  }

  const agents: CrewAgent[] = [];
  for (const r of shape.roles) {
    const count = requested.get(r.role) ?? r.amount ?? 0;
    const max = r.max_allowed ?? Number.POSITIVE_INFINITY;
    if (count > max) {
      throw new Error(
        `Requested ${count} agents for role "${r.role}" exceeds max_allowed=${max}`,
      );
    }
    for (let i = 0; i < count; i++) {
      const id = `${r.role}-${i + 1}`;
      const agent: CrewAgent = { id, role: r.role };
      if (r.nested_crew && registry && registry[r.nested_crew]) {
        const nestedShape = registry[r.nested_crew];
        agent.nestedCrew = buildCrewGraph(
          nestedShape,
          { schema_version: "1.0", name: nestedShape.name, agents: [] },
          registry,
          nextVisited,
        );
      }
      agents.push(agent);
    }
  }

  return { name: shape.name, roles, agents };
}

export function lintCrewShape(shape: CrewShape): LintResult {
  const errors: LintIssue[] = [];
  const warnings: LintIssue[] = [];
  const roleNames = new Set(shape.roles.map((r) => r.role));

  if (roleNames.size !== shape.roles.length) {
    const seen = new Set<string>();
    for (const r of shape.roles) {
      if (seen.has(r.role)) {
        errors.push({
          code: "duplicate_role",
          message: `Duplicate role definition: "${r.role}"`,
          path: r.role,
        });
      }
      seen.add(r.role);
    }
  }

  const firstInputRoles = shape.roles.filter((r) => r.first_input);
  if (firstInputRoles.length === 0) {
    errors.push({
      code: "missing_first_input",
      message: "Crew must have exactly one role with first_input: true",
    });
  } else if (firstInputRoles.length > 1) {
    errors.push({
      code: "multiple_first_input",
      message: `Crew has ${firstInputRoles.length} roles with first_input: true (${firstInputRoles.map((r) => r.role).join(", ")}); expected exactly one`,
    });
  }

  const finalOutputRoles = shape.roles.filter((r) => r.final_output);
  if (finalOutputRoles.length === 0) {
    errors.push({
      code: "missing_final_output",
      message: "Crew must have exactly one role with final_output: true",
    });
  } else if (finalOutputRoles.length > 1) {
    errors.push({
      code: "multiple_final_output",
      message: `Crew has ${finalOutputRoles.length} roles with final_output: true (${finalOutputRoles.map((r) => r.role).join(", ")}); expected exactly one`,
    });
  }

  for (const r of shape.roles) {
    const checkRefs = (refs: string[], action: string) => {
      for (const ref of refs) {
        if (!roleNames.has(ref)) {
          errors.push({
            code: "unknown_role_reference",
            message: `Role "${r.role}" ${action} unknown role "${ref}"`,
            path: `${r.role}.permissions.${action}`,
          });
        }
      }
    };
    checkRefs(r.permissions.talk_to, "talk_to");
    checkRefs(r.permissions.delegate_to, "delegate_to");
    checkRefs(r.permissions.escalate_to, "escalate_to");

    for (const inv of r.permissions.invite) {
      const invRole = typeof inv === "string" ? inv : inv.role;
      if (!roleNames.has(invRole)) {
        errors.push({
          code: "unknown_role_reference",
          message: `Role "${r.role}" invite references unknown role "${invRole}"`,
          path: `${r.role}.permissions.invite`,
        });
      }
    }

    if (r.permissions.escalate_to.includes(r.role)) {
      errors.push({
        code: "self_escalation",
        message: `Role "${r.role}" cannot escalate_to itself`,
        path: `${r.role}.permissions.escalate_to`,
      });
    }

    // Voting modes that the v1 resolver doesn't implement. Catch them at lint
    // time so configs don't crash mid-phase.
    const voting = r.execution?.voting;
    if (voting) {
      if (voting.mode === "best_of_n") {
        errors.push({
          code: "unsupported_voting_mode",
          message: `Role "${r.role}" uses voting mode "best_of_n" which is not implemented in v1`,
          path: `${r.role}.execution.voting.mode`,
        });
      }
      if (
        voting.mode === "weighted_consensus" &&
        voting.weight_function !== "equal"
      ) {
        errors.push({
          code: "unsupported_weight_function",
          message: `Role "${r.role}" uses weight_function "${voting.weight_function}" — v1 supports only "equal"`,
          path: `${r.role}.execution.voting.weight_function`,
        });
      }
    }

    for (const target of r.permissions.delegate_to) {
      if (!r.permissions.talk_to.includes(target)) {
        warnings.push({
          code: "delegate_without_talk",
          message: `Role "${r.role}" can delegate_to "${target}" but cannot talk_to it`,
          path: `${r.role}.permissions`,
        });
      }
    }
  }

  return { errors, warnings };
}
