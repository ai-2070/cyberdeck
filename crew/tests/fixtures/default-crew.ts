// DEFAULT_CREW shape + agents — shared test fixture for Phase 1.
// Mirrors the example crew from the project plan.

export const DEFAULT_CREW_SHAPE = {
  schema_version: "1.0",
  name: "DEFAULT_CREW",
  roles: [
    {
      role: "merc",
      capabilities: { thinking_allowed: true },
      permissions: {
        talk_to: ["fixer"],
        delegate_to: [],
        escalate_to: ["fixer"],
        invite: [
          {
            role: "merc",
            consensus: {
              mode: "weighted_consensus",
              weight_function: "equal",
              threshold: 0.66,
            },
          },
        ],
      },
      delegation: { max_depth: 2 },
      amount: 4,
    },
    {
      role: "specialist",
      capabilities: { thinking_allowed: true, tools: ["search", "web"] },
      permissions: {
        talk_to: ["fixer"],
        delegate_to: [],
        escalate_to: ["fixer"],
        invite: [],
      },
    },
    {
      role: "fixer",
      max_allowed: 1,
      capabilities: { thinking_allowed: true },
      activation: { on_fault: true, on_stall: true },
      permissions: {
        talk_to: ["merc", "specialist", "caller"],
        delegate_to: ["merc", "specialist"],
        escalate_to: ["caller"],
        invite: ["merc", "specialist"],
        modify_structure: true,
      },
    },
    {
      role: "caller",
      max_allowed: 1,
      capabilities: { thinking_allowed: false },
      permissions: {
        talk_to: ["fixer"],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      first_input: true,
      final_output: true,
    },
  ],
};

export const DEFAULT_CREW_AGENTS = {
  schema_version: "1.0",
  name: "DEFAULT_CREW",
  agents: [
    { role: "merc", amount: 4 },
    { role: "specialist", amount: 1 },
    { role: "fixer", amount: 1 },
    { role: "caller", amount: 1 },
  ],
};
