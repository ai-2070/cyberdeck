//! Integration test for tag-based scoped capability discovery.
//!
//! Reserved `scope:*` tags inside the announcer's `CapabilitySet`
//! resolve to a `CapabilityScope` that callers can filter against
//! via `MeshNode::find_nodes_by_filter_scoped`. Enforcement is
//! purely query-side — the wire format and forwarder logic are
//! untouched (see `docs/SCOPED_CAPABILITIES_PLAN.md`).
//!
//! Three nodes:
//! - A tagged `scope:tenant:oem-123`,
//! - B tagged `scope:tenant:corp-acme`,
//! - C unscoped (resolves to `Global`).
//!
//! Verifies:
//! - `ScopeFilter::Tenant("oem-123")` returns A and C, not B
//!   (Global is permissive, B's tenant doesn't match).
//! - `ScopeFilter::Any` returns all three.
//! - `ScopeFilter::GlobalOnly` returns only C.
//!
//! Run: `cargo test --features net --test capability_scope`

#![cfg(feature = "net")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::{CapabilityFilter, CapabilitySet, ScopeFilter};
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

fn test_config() -> MeshNodeConfig {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, PSK)
        .with_heartbeat_interval(Duration::from_millis(200))
        .with_session_timeout(Duration::from_secs(5))
        .with_handshake(3, Duration::from_secs(2))
        .with_capability_gc_interval(Duration::from_millis(250));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

async fn build_node() -> Arc<MeshNode> {
    let keypair = EntityKeypair::generate();
    Arc::new(
        MeshNode::new(keypair, test_config())
            .await
            .expect("MeshNode::new"),
    )
}

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

async fn wait_until<F>(node: &Arc<MeshNode>, mut cond: F) -> bool
where
    F: FnMut(&MeshNode) -> bool,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        if cond(node) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    cond(node)
}

#[tokio::test]
async fn tenant_scoped_discovery_filters_unrelated_tenants() {
    // Three providers around an observer D. Each provider tags its
    // capability set differently:
    //
    //   A — scope:tenant:oem-123
    //   B — scope:tenant:corp-acme
    //   C — no scope tag (resolves to Global)
    //
    // D handshakes with each so its capability index sees all three.
    let a = build_node().await;
    let b = build_node().await;
    let c = build_node().await;
    let d = build_node().await;

    handshake(&d, &a).await;
    handshake(&d, &b).await;
    handshake(&d, &c).await;

    // Same capability filter on all three — they only differ in the
    // scope tag. Using "model:llama3-70b" as a discriminator that's
    // common to a GPU pool.
    a.announce_capabilities(
        CapabilitySet::new()
            .add_tag("model:llama3-70b")
            .with_tenant_scope("oem-123"),
    )
    .await
    .expect("A announce");
    b.announce_capabilities(
        CapabilitySet::new()
            .add_tag("model:llama3-70b")
            .with_tenant_scope("corp-acme"),
    )
    .await
    .expect("B announce");
    c.announce_capabilities(CapabilitySet::new().add_tag("model:llama3-70b"))
        .await
        .expect("C announce");

    let filter = CapabilityFilter::new().require_tag("model:llama3-70b");
    let a_id = a.node_id();
    let b_id = b.node_id();
    let c_id = c.node_id();

    // First wait for all three announcements to arrive at D under
    // an unfiltered query — the scope filter is a per-call concern,
    // it shouldn't affect propagation.
    let arrived = wait_until(&d, |n| {
        let peers = n.find_nodes_by_filter(&filter);
        peers.contains(&a_id) && peers.contains(&b_id) && peers.contains(&c_id)
    })
    .await;
    assert!(
        arrived,
        "D did not observe all three capability announcements"
    );

    // Tenant("oem-123"): A (matches tenant) + C (Global is
    // permissive). B excluded — its tenant tag doesn't match.
    let oem = d.find_nodes_by_filter_scoped(&filter, &ScopeFilter::Tenant("oem-123"));
    assert!(oem.contains(&a_id), "tenant:oem-123 must include A");
    assert!(
        oem.contains(&c_id),
        "tenant:oem-123 must include unscoped C (Global is permissive)"
    );
    assert!(
        !oem.contains(&b_id),
        "tenant:oem-123 must exclude B (different tenant)"
    );

    // Tenant("corp-acme"): B + C, not A.
    let acme = d.find_nodes_by_filter_scoped(&filter, &ScopeFilter::Tenant("corp-acme"));
    assert!(acme.contains(&b_id), "tenant:corp-acme must include B");
    assert!(
        acme.contains(&c_id),
        "tenant:corp-acme must include unscoped C"
    );
    assert!(
        !acme.contains(&a_id),
        "tenant:corp-acme must exclude A (different tenant)"
    );

    // Any: all three (no SubnetLocal candidates here).
    let any = d.find_nodes_by_filter_scoped(&filter, &ScopeFilter::Any);
    assert!(
        any.contains(&a_id) && any.contains(&b_id) && any.contains(&c_id),
        "ScopeFilter::Any must return all non-SubnetLocal peers; got {:?}",
        any
    );

    // GlobalOnly: just C (the only untagged peer).
    let global = d.find_nodes_by_filter_scoped(&filter, &ScopeFilter::GlobalOnly);
    assert!(global.contains(&c_id), "GlobalOnly must include C");
    assert!(
        !global.contains(&a_id) && !global.contains(&b_id),
        "GlobalOnly must exclude tenant-scoped A and B; got {:?}",
        global
    );
}
