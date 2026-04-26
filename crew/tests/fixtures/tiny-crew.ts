// TINY_CREW — minimal fixture without fixer activation. Useful for tests that
// want to isolate behavior from DEFAULT_CREW's fault-recovery wiring.

export const TINY_CREW_SHAPE = {
  schema_version: "1.0",
  name: "TINY_CREW",
  roles: [
    {
      role: "alpha",
      capabilities: { thinking_allowed: true },
      permissions: {
        talk_to: ["beta"],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      first_input: true,
    },
    {
      role: "beta",
      capabilities: { thinking_allowed: true },
      permissions: {
        talk_to: ["alpha", "gamma"],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      amount: 2,
    },
    {
      role: "gamma",
      capabilities: { thinking_allowed: false },
      permissions: {
        talk_to: [],
        delegate_to: [],
        escalate_to: [],
        invite: [],
      },
      final_output: true,
    },
  ],
};

export const TINY_CREW_AGENTS = {
  schema_version: "1.0",
  name: "TINY_CREW",
  agents: [
    { role: "alpha", amount: 1 },
    { role: "beta", amount: 2 },
    { role: "gamma", amount: 1 },
  ],
};
