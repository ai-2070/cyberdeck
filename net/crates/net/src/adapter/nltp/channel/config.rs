//! Channel configuration and visibility.
//!
//! Channel policy uses the existing capability system (`CapabilityFilter`)
//! for access rules, combined with L1 permission tokens. This avoids
//! building a separate rule engine.

use super::name::ChannelId;
use crate::adapter::nltp::behavior::capability::{CapabilityFilter, CapabilitySet};
use crate::adapter::nltp::identity::{EntityId, TokenCache, TokenScope};
use dashmap::DashMap;

/// Channel visibility scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Packets never leave the subnet.
    SubnetLocal,
    /// Visible to the parent subnet but not siblings.
    ParentVisible,
    /// Explicitly exported to specific target subnets.
    Exported,
    /// Visible everywhere, no subnet restriction.
    #[default]
    Global,
}

/// Channel configuration with capability-based access control.
///
/// Authorization flow:
/// 1. Node announces capabilities via `CapabilityAd`
/// 2. If `publish_caps` is set, node's `CapabilitySet` must match the filter
/// 3. If `require_token` is true, node must also have a valid `PermissionToken`
/// 4. On success, `(origin_hash, channel_hash)` is inserted into the `AuthGuard`
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Channel identity (name + hash).
    pub channel_id: ChannelId,
    /// Visibility scope for subnet routing.
    pub visibility: Visibility,
    /// Capability requirements for publishing. `None` = any node can publish.
    pub publish_caps: Option<CapabilityFilter>,
    /// Capability requirements for subscribing. `None` = any node can subscribe.
    pub subscribe_caps: Option<CapabilityFilter>,
    /// Whether a valid `PermissionToken` is required (in addition to capabilities).
    pub require_token: bool,
    /// Default priority level for this channel's packets (0 = lowest).
    pub priority: u8,
    /// Default reliability mode for streams on this channel.
    pub reliable: bool,
    /// Optional rate limit in packets per second.
    pub max_rate_pps: Option<u32>,
}

impl ChannelConfig {
    /// Create a new channel config with defaults (open access, global visibility).
    pub fn new(channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            visibility: Visibility::default(),
            publish_caps: None,
            subscribe_caps: None,
            require_token: false,
            priority: 0,
            reliable: false,
            max_rate_pps: None,
        }
    }

    /// Set visibility.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set capability requirements for publishing.
    pub fn with_publish_caps(mut self, filter: CapabilityFilter) -> Self {
        self.publish_caps = Some(filter);
        self
    }

    /// Set capability requirements for subscribing.
    pub fn with_subscribe_caps(mut self, filter: CapabilityFilter) -> Self {
        self.subscribe_caps = Some(filter);
        self
    }

    /// Require a valid permission token.
    pub fn with_require_token(mut self, require: bool) -> Self {
        self.require_token = require;
        self
    }

    /// Set default priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set default reliability.
    pub fn with_reliable(mut self, reliable: bool) -> Self {
        self.reliable = reliable;
        self
    }

    /// Set rate limit.
    pub fn with_rate_limit(mut self, pps: u32) -> Self {
        self.max_rate_pps = Some(pps);
        self
    }

    /// Check if a node is authorized to publish on this channel.
    pub fn can_publish(
        &self,
        node_caps: &CapabilitySet,
        entity_id: &EntityId,
        token_cache: &TokenCache,
    ) -> bool {
        // Check capability requirements
        if let Some(ref filter) = self.publish_caps {
            if !filter.matches(node_caps) {
                return false;
            }
        }
        // Check token requirement
        if self.require_token
            && token_cache
                .check(entity_id, TokenScope::PUBLISH, self.channel_id.hash())
                .is_err()
        {
            return false;
        }
        true
    }

    /// Check if a node is authorized to subscribe to this channel.
    pub fn can_subscribe(
        &self,
        node_caps: &CapabilitySet,
        entity_id: &EntityId,
        token_cache: &TokenCache,
    ) -> bool {
        if let Some(ref filter) = self.subscribe_caps {
            if !filter.matches(node_caps) {
                return false;
            }
        }
        if self.require_token
            && token_cache
                .check(entity_id, TokenScope::SUBSCRIBE, self.channel_id.hash())
                .is_err()
        {
            return false;
        }
        true
    }
}

/// Registry of channel configurations.
///
/// Consulted at subscription/channel-creation time (slow path).
/// The fast path uses the `AuthGuard` bloom filter.
pub struct ChannelConfigRegistry {
    configs: DashMap<u16, ChannelConfig>,
}

impl ChannelConfigRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            configs: DashMap::new(),
        }
    }

    /// Register a channel configuration.
    pub fn insert(&self, config: ChannelConfig) {
        self.configs.insert(config.channel_id.hash(), config);
    }

    /// Look up a channel config by hash.
    pub fn get(
        &self,
        channel_hash: u16,
    ) -> Option<dashmap::mapref::one::Ref<'_, u16, ChannelConfig>> {
        self.configs.get(&channel_hash)
    }

    /// Remove a channel config.
    pub fn remove(&self, channel_hash: u16) -> Option<ChannelConfig> {
        self.configs.remove(&channel_hash).map(|(_, c)| c)
    }

    /// Number of registered channels.
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }

    /// Get the priority for a channel (0 if not configured).
    #[inline]
    pub fn priority(&self, channel_hash: u16) -> u8 {
        self.configs
            .get(&channel_hash)
            .map(|c| c.priority)
            .unwrap_or(0)
    }
}

impl Default for ChannelConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ChannelConfigRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelConfigRegistry")
            .field("channels", &self.configs.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::nltp::behavior::capability::{GpuInfo, GpuVendor, HardwareCapabilities};
    use crate::adapter::nltp::identity::{EntityKeypair, PermissionToken};

    fn make_caps(gpu: bool) -> CapabilitySet {
        if gpu {
            let gpu_info = GpuInfo {
                vendor: GpuVendor::Nvidia,
                model: "test".to_string(),
                vram_mb: 8192,
                compute_units: 0,
                tensor_cores: 0,
                fp16_tflops_x10: 0,
            };
            CapabilitySet::new().with_hardware(HardwareCapabilities::new().with_gpu(gpu_info))
        } else {
            CapabilitySet::new()
        }
    }

    #[test]
    fn test_open_channel() {
        let id = ChannelId::parse("sensors/lidar").unwrap();
        let config = ChannelConfig::new(id);
        let caps = make_caps(false);
        let entity = EntityKeypair::generate();
        let cache = TokenCache::new();

        assert!(config.can_publish(&caps, entity.entity_id(), &cache));
        assert!(config.can_subscribe(&caps, entity.entity_id(), &cache));
    }

    #[test]
    fn test_capability_restricted_channel() {
        let id = ChannelId::parse("compute/gpu-tasks").unwrap();
        let config =
            ChannelConfig::new(id).with_publish_caps(CapabilityFilter::new().require_gpu());

        let entity = EntityKeypair::generate();
        let cache = TokenCache::new();

        let no_gpu = make_caps(false);
        assert!(!config.can_publish(&no_gpu, entity.entity_id(), &cache));

        let with_gpu = make_caps(true);
        assert!(config.can_publish(&with_gpu, entity.entity_id(), &cache));
    }

    #[test]
    fn test_token_required_channel() {
        let id = ChannelId::parse("control/estop").unwrap();
        let config = ChannelConfig::new(id.clone()).with_require_token(true);
        let caps = make_caps(false);
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let cache = TokenCache::new();

        // No token -> denied
        assert!(!config.can_publish(&caps, subject.entity_id(), &cache));

        // Issue a publish token for this channel
        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            id.hash(),
            3600,
            0,
        );
        cache.insert(token);

        // With token -> allowed
        assert!(config.can_publish(&caps, subject.entity_id(), &cache));
    }

    #[test]
    fn test_caps_and_token_combined() {
        let id = ChannelId::parse("compute/secure").unwrap();
        let config = ChannelConfig::new(id.clone())
            .with_publish_caps(CapabilityFilter::new().require_gpu())
            .with_require_token(true);

        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let cache = TokenCache::new();

        // Has GPU but no token -> denied
        let with_gpu = make_caps(true);
        assert!(!config.can_publish(&with_gpu, subject.entity_id(), &cache));

        // Has token but no GPU -> denied
        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            id.hash(),
            3600,
            0,
        );
        cache.insert(token);
        let no_gpu = make_caps(false);
        assert!(!config.can_publish(&no_gpu, subject.entity_id(), &cache));

        // Has both -> allowed
        assert!(config.can_publish(&with_gpu, subject.entity_id(), &cache));
    }

    #[test]
    fn test_config_registry() {
        let reg = ChannelConfigRegistry::new();
        let id = ChannelId::parse("sensors/lidar").unwrap();
        let config = ChannelConfig::new(id.clone()).with_priority(5);

        reg.insert(config);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.priority(id.hash()), 5);

        let retrieved = reg.get(id.hash()).unwrap();
        assert_eq!(retrieved.priority, 5);
    }

    #[test]
    fn test_visibility_default() {
        let id = ChannelId::parse("test").unwrap();
        let config = ChannelConfig::new(id);
        assert_eq!(config.visibility, Visibility::Global);
    }
}
