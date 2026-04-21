// Package net — subnet configuration types.
//
// Crosses the C boundary as JSON inside `MeshConfig`. Matches the
// PyO3 / NAPI / TS SDK shapes byte-for-byte so the same fixtures
// round-trip across every binding.
//
// Tracks Stage G-3 of `docs/SDK_GO_PARITY_PLAN.md`.

package net

// SubnetRule maps a capability tag to a subnet-level bucket. Used
// alongside `SubnetPolicy` to derive a node's subnet from its
// announcement.
//
// Example: a rule with `TagPrefix: "region:"` + `Level: 0` +
// `Values: {"eu": 1, "us": 2}` collapses a node announcing
// `"region:us"` into subnet byte `2` at level 0.
type SubnetRule struct {
	// TagPrefix matches at any position in the capability-tag list.
	// The first matching tag per rule wins.
	TagPrefix string `json:"tag_prefix"`
	// Level is the subnet hierarchy slot this rule fills (0–3).
	Level uint32 `json:"level"`
	// Values maps `tag_value → subnet_byte`. Keys are the suffix
	// after `TagPrefix`; values are the u8 bucket index at `Level`.
	Values map[string]uint32 `json:"values"`
}

// SubnetPolicy is the set of rules used to assign a subnet from a
// node's announced capability tags.
type SubnetPolicy struct {
	Rules []SubnetRule `json:"rules,omitempty"`
}
