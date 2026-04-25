import { describe, it } from "vitest";

describe("Permission ACL gates", () => {
  it.todo("requestAction({ from, to, action: 'talk_to' }) returns true when allowed");
  it.todo("requestAction returns false and emits permission.denied when disallowed");
  it.todo("on_permission_denied activation routes to fixer");
  it.todo("the loop never spawns delegated agent.step.requested events on its own");
});
