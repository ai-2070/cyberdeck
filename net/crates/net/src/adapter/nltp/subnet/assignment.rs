//! Label-based subnet assignment.
//!
//! Nodes belong to subnets by their capability tags, not static configuration.
//! A `SubnetPolicy` maps tag patterns to hierarchy levels, deriving a `SubnetId`
//! from a node's `CapabilitySet`.

use std::collections::HashMap;

use super::id::SubnetId;
use crate::adapter::nltp::behavior::capability::CapabilitySet;

/// Policy for assigning nodes to subnets based on capability tags.
///
/// Rules are evaluated in order. Each rule maps a tag prefix to a hierarchy
/// level and provides a value map for the tag's value.
///
/// Example: a node with tags `["region:us-west", "fleet:alpha"]` and rules:
/// - `SubnetRule { tag_prefix: "region:", level: 0, values: {"us-west": 1} }`
/// - `SubnetRule { tag_prefix: "fleet:", level: 1, values: {"alpha": 2} }`
///
/// Would get `SubnetId::new(&[1, 2])`.
#[derive(Debug, Clone)]
pub struct SubnetPolicy {
    rules: Vec<SubnetRule>,
}

/// A single rule mapping a tag pattern to a hierarchy level.
#[derive(Debug, Clone)]
pub struct SubnetRule {
    /// Tag prefix to match (e.g., "region:").
    pub tag_prefix: String,
    /// Which hierarchy level this tag fills (0-3).
    pub level: u8,
    /// Map from tag value to level value (e.g., "us-west" -> 1).
    pub values: HashMap<String, u8>,
}

impl SubnetPolicy {
    /// Create an empty policy (all nodes get SubnetId::GLOBAL).
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the policy.
    ///
    /// # Panics
    /// Panics if the rule's level is >= 4.
    pub fn add_rule(mut self, rule: SubnetRule) -> Self {
        assert!(
            rule.level < 4,
            "subnet rule level must be 0-3, got {}",
            rule.level
        );
        self.rules.push(rule);
        self
    }

    /// Assign a subnet ID to a node based on its capability tags.
    ///
    /// Evaluates all rules against the node's tags. Unmatched levels
    /// remain zero (meaning "no restriction at that level").
    pub fn assign(&self, caps: &CapabilitySet) -> SubnetId {
        let mut levels = [0u8; 4];

        for rule in &self.rules {
            for tag in &caps.tags {
                if let Some(value) = tag.strip_prefix(&rule.tag_prefix) {
                    if let Some(&level_value) = rule.values.get(value) {
                        levels[rule.level as usize] = level_value;
                        break; // first match wins for this rule
                    }
                }
            }
        }

        SubnetId::new(&levels)
    }
}

impl Default for SubnetPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl SubnetRule {
    /// Create a new rule.
    pub fn new(tag_prefix: impl Into<String>, level: u8) -> Self {
        Self {
            tag_prefix: tag_prefix.into(),
            level,
            values: HashMap::new(),
        }
    }

    /// Map a tag value to a level value.
    ///
    /// # Panics
    /// Panics if `level_value` is 0 (reserved for "unmatched / no restriction").
    pub fn map(mut self, tag_value: impl Into<String>, level_value: u8) -> Self {
        assert!(
            level_value != 0,
            "level_value 0 is reserved for unmatched levels"
        );
        self.values.insert(tag_value.into(), level_value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::nltp::behavior::capability::CapabilitySet;

    fn caps_with_tags(tags: &[&str]) -> CapabilitySet {
        let mut caps = CapabilitySet::new();
        for tag in tags {
            caps = caps.add_tag(*tag);
        }
        caps
    }

    #[test]
    fn test_empty_policy() {
        let policy = SubnetPolicy::new();
        let caps = caps_with_tags(&["region:us-west"]);
        assert_eq!(policy.assign(&caps), SubnetId::GLOBAL);
    }

    #[test]
    fn test_single_level() {
        let policy = SubnetPolicy::new().add_rule(
            SubnetRule::new("region:", 0)
                .map("us-west", 1)
                .map("eu-central", 2),
        );

        let caps = caps_with_tags(&["region:us-west"]);
        assert_eq!(policy.assign(&caps), SubnetId::new(&[1]));

        let caps = caps_with_tags(&["region:eu-central"]);
        assert_eq!(policy.assign(&caps), SubnetId::new(&[2]));
    }

    #[test]
    fn test_multi_level() {
        let policy = SubnetPolicy::new()
            .add_rule(
                SubnetRule::new("region:", 0)
                    .map("us-west", 1)
                    .map("eu-central", 2),
            )
            .add_rule(SubnetRule::new("fleet:", 1).map("alpha", 1).map("beta", 2));

        let caps = caps_with_tags(&["region:us-west", "fleet:beta"]);
        assert_eq!(policy.assign(&caps), SubnetId::new(&[1, 2]));
    }

    #[test]
    fn test_unmatched_tag() {
        let policy = SubnetPolicy::new().add_rule(SubnetRule::new("region:", 0).map("us-west", 1));

        // Tag value not in the map
        let caps = caps_with_tags(&["region:unknown"]);
        assert_eq!(policy.assign(&caps), SubnetId::GLOBAL);

        // No matching tag prefix
        let caps = caps_with_tags(&["fleet:alpha"]);
        assert_eq!(policy.assign(&caps), SubnetId::GLOBAL);
    }

    #[test]
    fn test_partial_match() {
        let policy = SubnetPolicy::new()
            .add_rule(SubnetRule::new("region:", 0).map("us-west", 3))
            .add_rule(SubnetRule::new("fleet:", 1).map("alpha", 7));

        // Only region tag, no fleet
        let caps = caps_with_tags(&["region:us-west"]);
        assert_eq!(policy.assign(&caps), SubnetId::new(&[3]));
    }

    #[test]
    fn test_four_levels() {
        let policy = SubnetPolicy::new()
            .add_rule(SubnetRule::new("region:", 0).map("us", 1))
            .add_rule(SubnetRule::new("fleet:", 1).map("f1", 2))
            .add_rule(SubnetRule::new("vehicle:", 2).map("v42", 3))
            .add_rule(SubnetRule::new("subsystem:", 3).map("lidar", 4));

        let caps = caps_with_tags(&["region:us", "fleet:f1", "vehicle:v42", "subsystem:lidar"]);
        assert_eq!(policy.assign(&caps), SubnetId::new(&[1, 2, 3, 4]));
    }
}
