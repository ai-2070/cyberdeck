//! Simplified configuration builder for the SDK.

use std::time::Duration;

use net::config::{
    AdapterConfig, BackpressureMode, BatchConfig, EventBusConfig, EventBusConfigBuilder,
    ScalingPolicy,
};

/// Backpressure strategy.
#[derive(Debug, Clone, Copy)]
pub enum Backpressure {
    /// Drop newest events when buffer is full.
    DropNewest,
    /// Drop oldest events when buffer is full (default).
    DropOldest,
    /// Fail the producer when buffer is full.
    FailProducer,
    /// Sample events at the given rate (1 in N). Requires runtime support.
    Sample(u32),
}

impl From<Backpressure> for BackpressureMode {
    fn from(bp: Backpressure) -> Self {
        match bp {
            Backpressure::DropNewest => BackpressureMode::DropNewest,
            Backpressure::DropOldest => BackpressureMode::DropOldest,
            Backpressure::FailProducer => BackpressureMode::FailProducer,
            Backpressure::Sample(rate) => BackpressureMode::Sample { rate },
        }
    }
}

/// Builder for constructing a [`Net`](crate::Net) node.
pub struct NetBuilder {
    pub(crate) inner: EventBusConfigBuilder,
    pub(crate) adapter: Option<AdapterConfig>,
    /// Caller-owned identity. Stored here so `Net::from_builder` can
    /// plumb it into whichever adapter consumes keypairs; the
    /// event-bus itself is adapter-agnostic and doesn't use it.
    #[cfg(feature = "net")]
    pub(crate) identity: Option<crate::identity::Identity>,
}

impl NetBuilder {
    pub(crate) fn new() -> Self {
        Self {
            inner: EventBusConfig::builder(),
            adapter: None,
            #[cfg(feature = "net")]
            identity: None,
        }
    }

    /// Pin this node to a caller-owned [`Identity`](crate::Identity).
    ///
    /// Stored on the builder and handed to whichever adapter consumes
    /// keypairs (today: the mesh adapter when `net` is enabled). For
    /// an event-bus-only node without a mesh adapter, the identity is
    /// retained but unused — it becomes load-bearing once you wire in
    /// a `NetAdapterConfig`.
    #[cfg(feature = "net")]
    pub fn identity(mut self, identity: crate::identity::Identity) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Set the number of shards.
    pub fn shards(mut self, n: u16) -> Self {
        self.inner = self.inner.num_shards(n);
        self
    }

    /// Set the ring buffer capacity per shard (must be power of 2).
    pub fn buffer_capacity(mut self, capacity: usize) -> Self {
        self.inner = self.inner.ring_buffer_capacity(capacity);
        self
    }

    /// Set the backpressure strategy.
    pub fn backpressure(mut self, bp: Backpressure) -> Self {
        self.inner = self.inner.backpressure_mode(bp.into());
        self
    }

    /// Use the high-throughput batch preset.
    pub fn high_throughput(mut self) -> Self {
        self.inner = self.inner.batch(BatchConfig::high_throughput());
        self
    }

    /// Use the low-latency batch preset.
    pub fn low_latency(mut self) -> Self {
        self.inner = self.inner.batch(BatchConfig::low_latency());
        self
    }

    /// Set a custom batch configuration.
    pub fn batch(mut self, batch: BatchConfig) -> Self {
        self.inner = self.inner.batch(batch);
        self
    }

    /// Enable dynamic scaling.
    pub fn scaling(mut self, policy: ScalingPolicy) -> Self {
        self.inner = self.inner.scaling(policy);
        self
    }

    /// Set the adapter timeout.
    pub fn adapter_timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.adapter_timeout(timeout);
        self
    }

    /// Use in-memory transport (no persistence, single process).
    pub fn memory(mut self) -> Self {
        self.adapter = Some(AdapterConfig::Noop);
        self
    }

    /// Use Redis Streams transport.
    #[cfg(feature = "redis")]
    pub fn redis(mut self, config: net::config::RedisAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::Redis(config));
        self
    }

    /// Use NATS JetStream transport.
    #[cfg(feature = "jetstream")]
    pub fn jetstream(mut self, config: net::config::JetStreamAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::JetStream(config));
        self
    }

    /// Use Net encrypted UDP mesh transport.
    #[cfg(feature = "net")]
    pub fn mesh(mut self, config: net::adapter::net::NetAdapterConfig) -> Self {
        self.adapter = Some(AdapterConfig::Net(Box::new(config)));
        self
    }

    /// Build the configuration, consuming the builder.
    pub(crate) fn build_config(mut self) -> crate::error::Result<EventBusConfig> {
        // The `mut adapter` binding is mutated only inside the
        // `#[cfg(feature = "net")]` block below; when the `net`
        // feature is off it's unused-mut. Suppress the warning
        // narrowly so `-D warnings` builds without `net` stay
        // clean.
        #[cfg_attr(not(feature = "net"), allow(unused_mut))]
        if let Some(mut adapter) = self.adapter {
            // Plumb the caller-supplied identity into the adapter if
            // it consumes keypairs. Today only the `net` adapter
            // does; other adapters drop the identity silently (the
            // `identity()` docstring flags this). Without this
            // injection the builder's `identity(...)` would be a
            // no-op — the mesh would generate a fresh keypair every
            // run and `node_id` / `entity_id` would drift across
            // restarts.
            //
            // Pre-fix this unconditionally overwrote
            // `net_cfg.entity_keypair`. A caller who configured the
            // adapter via `.mesh(NetAdapterConfig::initiator(...).with_entity_keypair(kp_a))`
            // and ALSO called `.identity(id_b)` got `id_b` silently
            // — both inputs were caller-supplied keypairs and one
            // was discarded with no error or precedence note. Both
            // are load-bearing for `node_id` / `entity_id`, so the
            // wrong one routes the node under the wrong identity.
            // Fix: error on conflicting keypairs (different
            // entity_ids); allow when both are set to the same
            // keypair (idempotent); inject when only `identity()`
            // was set (the original support path).
            #[cfg(feature = "net")]
            if let (Some(id), AdapterConfig::Net(ref mut net_cfg)) =
                (self.identity.as_ref(), &mut adapter)
            {
                let builder_kp = id.keypair();
                if let Some(adapter_kp) = net_cfg.entity_keypair.as_ref() {
                    if adapter_kp.entity_id() != builder_kp.entity_id() {
                        return Err(crate::error::SdkError::Config(format!(
                            "conflicting identities: NetAdapterConfig::with_entity_keypair \
                             pinned entity_id {} but NetBuilder::identity pinned {}. \
                             Set the keypair on exactly one of them.",
                            adapter_kp.entity_id(),
                            builder_kp.entity_id(),
                        )));
                    }
                    // Same entity_id: identical key material, no-op.
                } else {
                    net_cfg.entity_keypair = Some((**builder_kp).clone());
                }
            }
            self.inner = self.inner.adapter(adapter);
        }
        self.inner
            .build()
            .map_err(|e| crate::error::SdkError::Config(e.to_string()))
    }
}

#[cfg(all(test, feature = "net"))]
mod tests {
    use super::*;
    use net::adapter::net::NetAdapterConfig;
    use std::net::SocketAddr;

    fn sample_net_adapter() -> NetAdapterConfig {
        let bind: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let peer: SocketAddr = "127.0.0.1:1".parse().unwrap();
        NetAdapterConfig::initiator(bind, peer, [0x00; 32], [0x00; 32])
    }

    /// Regression for a cubic-flagged P1: `NetBuilder::identity(...)`
    /// stored the identity on the builder but `build_config` never
    /// plumbed it into the adapter. Callers pinning a seed expected
    /// a stable `node_id` / `entity_id` across restarts and silently
    /// got a fresh keypair every build.
    ///
    /// Test constructs a builder with a seeded identity + a `Net`
    /// adapter via `.mesh(...)`, calls `build_config`, and asserts
    /// the resulting `NetAdapterConfig.entity_keypair` contains the
    /// caller's key — not `None` (pre-fix behaviour) and not a
    /// freshly generated one.
    #[test]
    fn net_builder_identity_plumbs_into_adapter() {
        let seed = [0x42u8; 32];
        let identity = crate::identity::Identity::from_seed(seed);
        let expected_entity_id = identity.entity_id().clone();

        let config = NetBuilder::new()
            .mesh(sample_net_adapter())
            .identity(identity)
            .build_config()
            .expect("build_config");

        let AdapterConfig::Net(net_cfg) = config.adapter else {
            panic!("adapter lost its Net variant during build_config");
        };
        let kp = net_cfg
            .entity_keypair
            .as_ref()
            .expect("entity_keypair should have been plumbed from NetBuilder::identity");
        assert_eq!(
            kp.entity_id(),
            &expected_entity_id,
            "plumbed entity_id must match the pinned identity's",
        );
    }

    /// Regression for the other half of the same contract: a
    /// builder with no identity must leave `entity_keypair` unset
    /// so the mesh's internal default (generate a fresh keypair)
    /// is honoured. The fix mustn't accidentally inject an empty /
    /// default keypair when the caller didn't pin one.
    #[test]
    fn net_builder_without_identity_leaves_adapter_keypair_unset() {
        let config = NetBuilder::new()
            .mesh(sample_net_adapter())
            .build_config()
            .expect("build_config");

        let AdapterConfig::Net(net_cfg) = config.adapter else {
            panic!("adapter lost its Net variant");
        };
        assert!(
            net_cfg.entity_keypair.is_none(),
            "entity_keypair must stay unset when the builder has no identity",
        );
    }

    /// When both `NetAdapterConfig::with_entity_keypair`
    /// and `NetBuilder::identity` are set with DIFFERENT entity
    /// IDs, build_config must error rather than silently picking
    /// one. Both inputs are caller-supplied; both are load-bearing
    /// for `node_id` / `entity_id`; silently overriding routes the
    /// node under the wrong identity.
    #[test]
    fn build_config_errors_on_conflicting_identities() {
        let kp_a = net::adapter::net::identity::EntityKeypair::generate();
        let kp_b_seed = [0xAAu8; 32];
        let identity_b = crate::identity::Identity::from_seed(kp_b_seed);
        // Sanity — distinct entity_ids.
        assert_ne!(kp_a.entity_id(), identity_b.entity_id());

        let mesh_cfg = sample_net_adapter().with_entity_keypair(kp_a);

        let err = NetBuilder::new()
            .mesh(mesh_cfg)
            .identity(identity_b)
            .build_config()
            .expect_err(
                "build_config must reject conflicting keypairs; \
                 pre-fix it silently let identity() win",
            );

        let msg = format!("{}", err);
        assert!(
            msg.contains("conflicting identities"),
            "expected 'conflicting identities' in error, got: {}",
            msg
        );
    }

    /// Setting the SAME keypair on both sides is
    /// idempotent and must build cleanly. Documents the
    /// "identical key material, no-op" path.
    #[test]
    fn build_config_accepts_matching_identities_on_both_sides() {
        let seed = [0x11u8; 32];
        let identity = crate::identity::Identity::from_seed(seed);
        let same_kp = (**identity.keypair()).clone();
        let expected_id = identity.entity_id().clone();

        let mesh_cfg = sample_net_adapter().with_entity_keypair(same_kp);

        let config = NetBuilder::new()
            .mesh(mesh_cfg)
            .identity(identity)
            .build_config()
            .expect("matching keypairs on both sides must build");

        let AdapterConfig::Net(net_cfg) = config.adapter else {
            panic!("adapter lost its Net variant");
        };
        assert_eq!(
            net_cfg.entity_keypair.as_ref().unwrap().entity_id(),
            &expected_id,
        );
    }

    /// Setting the keypair via the adapter only
    /// (no `.identity()` call) preserves the adapter's choice —
    /// the build path must NOT overwrite it with a default.
    #[test]
    fn build_config_preserves_adapter_keypair_when_no_builder_identity() {
        let kp = net::adapter::net::identity::EntityKeypair::generate();
        let expected_id = kp.entity_id().clone();
        let mesh_cfg = sample_net_adapter().with_entity_keypair(kp);

        let config = NetBuilder::new()
            .mesh(mesh_cfg)
            .build_config()
            .expect("build_config");

        let AdapterConfig::Net(net_cfg) = config.adapter else {
            panic!("adapter lost its Net variant");
        };
        assert_eq!(
            net_cfg.entity_keypair.as_ref().unwrap().entity_id(),
            &expected_id,
            "adapter-side entity_keypair must survive when builder has no identity",
        );
    }
}
