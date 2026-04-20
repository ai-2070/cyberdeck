//! Integration tests for subnet visibility enforcement on the
//! publish + subscribe paths (Stage D-1 of `SDK_SECURITY_SURFACE_PLAN.md`).
//!
//! The original plan sketched a three-node test with A `[3,7,2]`
//! / B `[3,7,3]` / C `[3,8,1]` and claimed `SubnetLocal` delivers
//! A↔B. That interpretation doesn't match the actual `Visibility`
//! enum — `SubnetLocal` is strict same-subnet (see
//! `channel/config.rs`), so A/B with differing level-2 bytes are
//! NOT partitioned together.
//!
//! This test uses the semantics the core actually implements:
//!   - `SubnetLocal` — exact same `SubnetId` only.
//!   - `ParentVisible` — same subnet, or ancestor/descendant pair.
//!   - `Global` — any peer.
//!
//! Run: `cargo test --features net --test subnet_enforcement`

#![cfg(feature = "net")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::CapabilitySet;
use net::adapter::net::{
    ChannelConfig, ChannelConfigRegistry, ChannelId, ChannelName, ChannelPublisher, EntityKeypair,
    MeshNode, MeshNodeConfig, OnFailure, PublishConfig, Reliability, SocketBufferConfig, SubnetId,
    SubnetPolicy, SubnetRule, Visibility,
};
use tokio::net::UdpSocket;

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

async fn find_ports(n: usize) -> Vec<u16> {
    let mut ports = Vec::with_capacity(n);
    let mut sockets = Vec::with_capacity(n);
    for _ in 0..n {
        let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        ports.push(sock.local_addr().unwrap().port());
        sockets.push(sock);
    }
    drop(sockets);
    tokio::time::sleep(Duration::from_millis(10)).await;
    ports
}

/// Build the same `SubnetPolicy` every node uses. Three rules map
/// `region:<x>` / `fleet:<x>` / `unit:<x>` tags onto levels 0–2.
/// Mesh-wide consistency is assumed — mismatched policies lead to
/// asymmetric views of peer subnets (documented in SUBNET_ENFORCEMENT_PLAN).
fn shared_policy() -> Arc<SubnetPolicy> {
    let region = SubnetRule::new("region:", 0).map("us", 3);
    let fleet = SubnetRule::new("fleet:", 1).map("blue", 7).map("green", 8);
    let unit = SubnetRule::new("unit:", 2)
        .map("alpha", 2)
        .map("beta", 3)
        .map("gamma", 1);
    Arc::new(
        SubnetPolicy::new()
            .add_rule(region)
            .add_rule(fleet)
            .add_rule(unit),
    )
}

fn test_config(port: u16, subnet: SubnetId, policy: Arc<SubnetPolicy>) -> MeshNodeConfig {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, PSK)
        .with_heartbeat_interval(Duration::from_millis(200))
        .with_session_timeout(Duration::from_secs(5))
        .with_handshake(3, Duration::from_secs(2))
        .with_capability_gc_interval(Duration::from_millis(250))
        .with_subnet(subnet)
        .with_subnet_policy(policy);
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

/// Build a MeshNode and pre-install a shared channel-config
/// registry so subscribers can be authorized per-channel.
async fn build_node(
    port: u16,
    subnet: SubnetId,
    policy: Arc<SubnetPolicy>,
    registry: Arc<ChannelConfigRegistry>,
) -> Arc<MeshNode> {
    let cfg = test_config(port, subnet, policy);
    let keypair = EntityKeypair::generate();
    let mut node = MeshNode::new(keypair, cfg).await.expect("MeshNode::new");
    node.set_channel_configs(registry);
    Arc::new(node)
}

/// Connect A→B and start both. `start()` is idempotent so calling
/// it again for a second pairing with the same A is fine.
async fn handshake(a: &Arc<MeshNode>, b: &Arc<MeshNode>) {
    let a_id = a.node_id();
    let b_id = b.node_id();
    let b_pub = *b.public_key();
    let b_addr = b.local_addr();
    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });
    a.connect(b_addr, &b_pub, b_id)
        .await
        .expect("connect failed");
    accept
        .await
        .expect("accept task panicked")
        .expect("accept failed");
    a.start();
    b.start();
}

async fn wait_until<F>(mut cond: F) -> bool
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        if cond() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    cond()
}

fn caps_for(region: &str, fleet: &str, unit: &str) -> CapabilitySet {
    CapabilitySet::new()
        .add_tag(format!("region:{region}"))
        .add_tag(format!("fleet:{fleet}"))
        .add_tag(format!("unit:{unit}"))
}

/// End-to-end three-node test. A and B share exact subnet
/// `[3,7,2]`; C is on `[3,8,1]`. A `SubnetLocal` channel on A
/// must deliver to B and reject C.
#[tokio::test]
async fn subnet_local_partitions_a_b_from_c() {
    let policy = shared_policy();
    let ports = find_ports(3).await;

    let a_registry = Arc::new(ChannelConfigRegistry::new());
    let b_registry = Arc::new(ChannelConfigRegistry::new());
    let c_registry = Arc::new(ChannelConfigRegistry::new());

    // A and B both on exact subnet [3,7,2]; C on [3,8,1].
    let shared_subnet = SubnetId::new(&[3, 7, 2]);
    let a = build_node(ports[0], shared_subnet, policy.clone(), a_registry.clone()).await;
    let b = build_node(ports[1], shared_subnet, policy.clone(), b_registry).await;
    let c = build_node(
        ports[2],
        SubnetId::new(&[3, 8, 1]),
        policy.clone(),
        c_registry,
    )
    .await;

    // Hub topology — A connects to B and C.
    handshake(&a, &b).await;
    handshake(&a, &c).await;

    // Each node announces caps that round-trip through `policy` to
    // the subnet it was configured with. A + B share unit:alpha so
    // both derive level 2 = 2; C uses unit:gamma for level 2 = 1.
    a.announce_capabilities(caps_for("us", "blue", "alpha"))
        .await
        .expect("A announce");
    b.announce_capabilities(caps_for("us", "blue", "alpha"))
        .await
        .expect("B announce");
    c.announce_capabilities(caps_for("us", "green", "gamma"))
        .await
        .expect("C announce");

    // Wait until A has indexed both peers' announcements (proxy for
    // `peer_subnets` being populated — they're updated in the same
    // handler).
    let b_id = b.node_id();
    let c_id = c.node_id();
    let learned = wait_until(|| {
        a.capability_index().get(b_id).is_some() && a.capability_index().get(c_id).is_some()
    })
    .await;
    assert!(
        learned,
        "A never indexed both B's and C's capability announcements"
    );

    // Register a SubnetLocal channel on A. `get_by_name` is
    // DashMap-backed, so A's authorize_subscribe will see it
    // synchronously.
    let channel_name = ChannelName::new("lab/metrics").expect("channel name");
    let chan_cfg = ChannelConfig::new(ChannelId::new(channel_name.clone()))
        .with_visibility(Visibility::SubnetLocal);
    a_registry.insert(chan_cfg);

    let a_id = a.node_id();

    // B subscribes — same subnet as A → accepted.
    b.subscribe_channel(a_id, channel_name.clone())
        .await
        .expect("B subscribe should be accepted");

    // C subscribes — different subnet → A's authorize_subscribe
    // rejects with Unauthorized.
    let c_result = c.subscribe_channel(a_id, channel_name.clone()).await;
    assert!(
        c_result.is_err(),
        "C's subscribe should have been rejected under SubnetLocal"
    );

    // A publishes. Only B is on the roster (C was rejected at
    // subscribe), AND the publish filter independently confirms
    // C's subnet would be dropped anyway. attempted == 1.
    let publisher = ChannelPublisher::new(
        channel_name,
        PublishConfig {
            reliability: Reliability::FireAndForget,
            on_failure: OnFailure::BestEffort,
            max_inflight: 16,
        },
    );
    let report = a
        .publish(&publisher, bytes::Bytes::from_static(b"ok"))
        .await
        .expect("publish");
    assert_eq!(
        report.attempted, 1,
        "only B should be attempted (C filtered)"
    );
    assert_eq!(report.delivered, 1, "B should have received the payload");
}

/// `ParentVisible` lets A and a descendant `[3,7,2,1]` see each
/// other but excludes a sibling at a different level-1 prefix.
#[tokio::test]
async fn parent_visible_admits_descendant_rejects_sibling() {
    let policy = shared_policy();
    let ports = find_ports(3).await;
    let a_registry = Arc::new(ChannelConfigRegistry::new());
    let b_registry = Arc::new(ChannelConfigRegistry::new());
    let c_registry = Arc::new(ChannelConfigRegistry::new());

    let a = build_node(
        ports[0],
        SubnetId::new(&[3, 7, 2]),
        policy.clone(),
        a_registry.clone(),
    )
    .await;
    // Descendant: shares A's first three levels, adds one more.
    let descendant = build_node(
        ports[1],
        SubnetId::new(&[3, 7, 2, 5]),
        policy.clone(),
        b_registry,
    )
    .await;
    // Sibling at level 1 — breaks the ancestor chain.
    let sibling = build_node(
        ports[2],
        SubnetId::new(&[3, 9, 1]),
        policy.clone(),
        c_registry,
    )
    .await;

    handshake(&a, &descendant).await;
    handshake(&a, &sibling).await;

    a.announce_capabilities(caps_for("us", "blue", "alpha"))
        .await
        .expect("A announce");
    descendant
        .announce_capabilities(caps_for("us", "blue", "alpha"))
        .await
        .expect("desc announce");
    sibling
        .announce_capabilities(caps_for("us", "green", "gamma"))
        .await
        .expect("sibling announce");

    let desc_id = descendant.node_id();
    let sib_id = sibling.node_id();
    let learned = wait_until(|| {
        a.capability_index().get(desc_id).is_some() && a.capability_index().get(sib_id).is_some()
    })
    .await;
    assert!(learned, "A did not learn both peers' announcements");

    let channel_name = ChannelName::new("lab/parent").expect("channel name");
    a_registry.insert(
        ChannelConfig::new(ChannelId::new(channel_name.clone()))
            .with_visibility(Visibility::ParentVisible),
    );

    let a_id = a.node_id();
    // Descendant: A is an ancestor of `[3,7,2,5]` → accepted.
    descendant
        .subscribe_channel(a_id, channel_name.clone())
        .await
        .expect("descendant subscribe accepted");
    // Sibling at level 1 has no ancestor relationship to A → rejected.
    let sibling_result = sibling.subscribe_channel(a_id, channel_name.clone()).await;
    assert!(
        sibling_result.is_err(),
        "sibling subscribe should have been rejected under ParentVisible"
    );

    let publisher = ChannelPublisher::new(
        channel_name,
        PublishConfig {
            reliability: Reliability::FireAndForget,
            on_failure: OnFailure::BestEffort,
            max_inflight: 16,
        },
    );
    let report = a
        .publish(&publisher, bytes::Bytes::from_static(b"pv"))
        .await
        .expect("publish");
    assert_eq!(report.attempted, 1, "only descendant should be attempted");
    assert_eq!(report.delivered, 1);
}

/// Regression: with no `SubnetPolicy`, the mesh falls back to
/// `SubnetId::GLOBAL` for every peer, so `SubnetLocal` channels
/// still deliver as if there were no partitioning.
#[tokio::test]
async fn without_policy_subnet_local_delivers_everywhere() {
    let ports = find_ports(2).await;
    let a_registry = Arc::new(ChannelConfigRegistry::new());
    let b_registry = Arc::new(ChannelConfigRegistry::new());

    // No policy, default subnet (GLOBAL) on both nodes.
    let a_cfg = test_config_no_policy(ports[0]);
    let b_cfg = test_config_no_policy(ports[1]);
    let mut a_owned = MeshNode::new(EntityKeypair::generate(), a_cfg).await.unwrap();
    a_owned.set_channel_configs(a_registry.clone());
    let a = Arc::new(a_owned);
    let mut b_owned = MeshNode::new(EntityKeypair::generate(), b_cfg).await.unwrap();
    b_owned.set_channel_configs(b_registry);
    let b = Arc::new(b_owned);

    handshake(&a, &b).await;

    let channel_name = ChannelName::new("lab/noisy").expect("channel name");
    a_registry.insert(
        ChannelConfig::new(ChannelId::new(channel_name.clone()))
            .with_visibility(Visibility::SubnetLocal),
    );

    // Both peers are GLOBAL → same_subnet(GLOBAL, GLOBAL) is true →
    // subscribe accepted.
    b.subscribe_channel(a.node_id(), channel_name.clone())
        .await
        .expect("B subscribe accepted");

    let publisher = ChannelPublisher::new(
        channel_name,
        PublishConfig {
            reliability: Reliability::FireAndForget,
            on_failure: OnFailure::BestEffort,
            max_inflight: 16,
        },
    );
    let report = a
        .publish(&publisher, bytes::Bytes::from_static(b"hi"))
        .await
        .expect("publish");
    assert_eq!(report.attempted, 1);
    assert_eq!(report.delivered, 1);
}

fn test_config_no_policy(port: u16) -> MeshNodeConfig {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, PSK)
        .with_heartbeat_interval(Duration::from_millis(200))
        .with_session_timeout(Duration::from_secs(5))
        .with_handshake(3, Duration::from_secs(2));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

/// SubnetId geometry sanity — the invariants the enforcement
/// paths lean on. Co-located here so a refactor of the core
/// helpers surfaces a failure in the same test file as the
/// enforcement that depends on them.
#[tokio::test]
async fn subnet_geometry_invariants() {
    let a = SubnetId::new(&[3, 7, 2]);
    let b_exact = SubnetId::new(&[3, 7, 2]);
    let diff_level2 = SubnetId::new(&[3, 7, 3]);
    let diff_level1 = SubnetId::new(&[3, 8, 1]);
    let descendant = SubnetId::new(&[3, 7, 2, 5]);

    // is_same_subnet is exact equality.
    assert!(a.is_same_subnet(b_exact));
    assert!(!a.is_same_subnet(diff_level2));

    // is_ancestor_of walks the prefix — [3,7,2] is ancestor of
    // [3,7,2,5] but not of [3,7,3] (siblings) or [3,8,1].
    assert!(a.is_ancestor_of(descendant));
    assert!(!a.is_ancestor_of(diff_level2));
    assert!(!a.is_ancestor_of(diff_level1));

    // Global is every subnet's ancestor (zero-mask matches everything).
    assert!(SubnetId::GLOBAL.is_ancestor_of(a));
    assert!(SubnetId::GLOBAL.is_ancestor_of(diff_level1));
}
