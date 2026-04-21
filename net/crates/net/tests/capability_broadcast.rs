//! Integration tests for the capability-broadcast subprotocol
//! (`SUBPROTOCOL_CAPABILITY_ANN = 0x0C00`).
//!
//! Covers the four load-bearing guarantees of Stage C-1:
//! - Two-node announce → find round-trip
//! - TTL expiry: post-TTL queries no longer return the peer
//! - Late joiner: session-open push catches new peers up
//! - Version skip: older announcements from the same peer are ignored
//!
//! Run: `cargo test --features net --test capability_broadcast`

#![cfg(feature = "net")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::{
    CapabilityAnnouncement, CapabilityFilter, CapabilitySet,
};
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};
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

fn test_config(port: u16) -> MeshNodeConfig {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
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

/// Build an unstarted MeshNode and return it alongside its node_id.
async fn build_node(port: u16) -> Arc<MeshNode> {
    build_node_with(port, |cfg| cfg).await
}

/// Build a MeshNode with a caller-supplied tweak to the test config.
async fn build_node_with<F>(port: u16, tweak: F) -> Arc<MeshNode>
where
    F: FnOnce(MeshNodeConfig) -> MeshNodeConfig,
{
    let cfg = tweak(test_config(port));
    let keypair = EntityKeypair::generate();
    Arc::new(MeshNode::new(keypair, cfg).await.expect("MeshNode::new"))
}

/// Handshake two nodes (A initiator, B responder) and `start()` both.
async fn handshake(a: &Arc<MeshNode>, b: &Arc<MeshNode>) {
    let a_id = a.node_id();
    let b_id = b.node_id();
    let b_pub = *b.public_key();
    let b_addr = b.local_addr();

    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });
    // No pre-connect sleep — `handshake_initiator` and
    // `handshake_responder` each have internal retry-with-backoff.
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

/// Poll `cond` on `node` every 25ms for up to 2s; returns `true` on
/// match. Callers use this instead of a fixed `sleep` so slow CI
/// boxes don't flake.
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

/// After an NKpsk0 handshake, only the initiator learns the peer's
/// Noise static pubkey — the pattern has no `-> s` leg, so the
/// responder never sees the initiator's static. `peer_static_x25519`
/// reflects exactly what snow exposes: `Some(pub)` on the initiator
/// side, `None` on the responder side. The identity-envelope path
/// uses this to seal to the target when the source was the
/// initiator; the Stage 5 wiring handles the responder-side case
/// by refusing to transport identity when the static is unknown
/// (migration falls back to `public_only` identity).
#[tokio::test]
async fn peer_static_x25519_returns_peer_noise_pubkey_after_handshake() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let a_id = a.node_id();
    let b_id = b.node_id();

    // Initiator side: A learns B's static from the out-of-band
    // pubkey it handed to `connect()`, surfaced post-handshake by
    // snow's `get_remote_static`.
    assert_eq!(
        a.peer_static_x25519(b_id),
        Some(*b.public_key()),
        "initiator (A) should recover B's Noise static pubkey",
    );

    // Responder side: NKpsk0 has no `-> s`, so snow has no remote
    // static to return. Documented limitation of the current
    // pattern; Stage 5 of the identity-migration plan plans around
    // this by requiring the migration source to have initiated the
    // session to the target (or by falling back to public-only
    // identity transport).
    assert!(
        b.peer_static_x25519(a_id).is_none(),
        "responder (B) should see None under NKpsk0 — pattern discloses only -> e",
    );

    // No session with an unknown node_id — return None, not zeros.
    assert!(a.peer_static_x25519(0xDEAD_BEEF_CAFE_F00D).is_none());
}

#[tokio::test]
async fn two_node_announce_is_visible() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let caps = CapabilitySet::new().add_tag("gpu").add_tag("inference");
    a.announce_capabilities(caps)
        .await
        .expect("announce failed");

    let filter = CapabilityFilter::new().require_tag("gpu");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |node| {
        node.find_peers_by_filter(&filter).contains(&a_id)
    })
    .await;
    assert!(arrived, "B did not observe A's capability announcement");
}

#[tokio::test]
async fn announcement_expires_after_ttl() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let caps = CapabilitySet::new().add_tag("ephemeral");
    // TTL = 1s; GC tick from `test_config` is 250ms, so two or three
    // sweeps land before we re-query at 1.5s. Signed — B's default
    // now drops unsigned announcements, and this test is exercising
    // TTL + GC, not the sign-gate.
    a.announce_capabilities_with(caps, Duration::from_secs(1), true)
        .await
        .expect("announce failed");

    let filter = CapabilityFilter::new().require_tag("ephemeral");
    let a_id = a.node_id();
    assert!(
        wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await,
        "B never indexed A's announcement in the first place"
    );

    // Wait beyond TTL; GC should evict.
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let still_present = b.find_peers_by_filter(&filter).contains(&a_id);
    assert!(
        !still_present,
        "B still returns A after TTL expiry (GC not running?)"
    );
}

#[tokio::test]
async fn late_joiner_receives_session_open_push() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;

    // A announces *before* B exists.
    let caps = CapabilitySet::new().add_tag("preannounced");
    a.announce_capabilities(caps)
        .await
        .expect("announce failed");

    // B joins the party.
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let filter = CapabilityFilter::new().require_tag("preannounced");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await;
    assert!(
        arrived,
        "session-open push did not deliver the pre-announcement"
    );
}

#[tokio::test]
async fn require_signed_capabilities_drops_unsigned_announcements() {
    // Post-E-1, `announce_capabilities` signs by default. Test the
    // policy knob by explicitly calling `announce_capabilities_with`
    // with `sign = false` — receiver B's flag must drop those.
    // Receiver B has the flag on; sender A announces unsigned.
    // A self-indexes its own announcement (local path bypasses
    // receive), so a self-query on A still matches — only B's view
    // should be blank.
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node_with(ports[1], |cfg| cfg.with_require_signed_capabilities(true)).await;
    handshake(&a, &b).await;

    a.announce_capabilities_with(
        CapabilitySet::new().add_tag("classified"),
        Duration::from_secs(60),
        false, // unsigned
    )
    .await
    .expect("announce failed");

    // A sees itself (local self-index isn't subject to the flag).
    let filter = CapabilityFilter::new().require_tag("classified");
    assert!(
        a.find_peers_by_filter(&filter).contains(&a.node_id()),
        "sender lost its own self-index"
    );

    // Give the receive path a few ms to process (or drop).
    tokio::time::sleep(Duration::from_millis(100)).await;

    // B must not have indexed A's unsigned announcement.
    assert!(
        !b.find_peers_by_filter(&filter).contains(&a.node_id()),
        "receiver accepted an unsigned announcement despite require_signed_capabilities=true"
    );
}

#[tokio::test]
async fn stale_versions_are_ignored_by_index() {
    // Dodges the mesh entirely — the version-skip invariant is a
    // property of CapabilityIndex itself, not the subprotocol.
    // Keeping the test here so the whole "capability pipeline" suite
    // lives together and this regression catches anyone who alters
    // index semantics.
    use net::adapter::net::behavior::CapabilityIndex;
    use net::adapter::net::EntityId;

    let index = CapabilityIndex::new();
    let caps_v1 = CapabilitySet::new().add_tag("v1");
    let caps_v2 = CapabilitySet::new().add_tag("v2");

    // Direct index test — no mesh, no signature verification.
    // A zero-byte EntityId is a valid data-structure input even
    // though it's not a valid ed25519 public key.
    let eid = EntityId::from_bytes([0u8; 32]);
    let v1 = CapabilityAnnouncement::new(/* node_id */ 0xAA, eid.clone(), 1, caps_v1);
    let v2 = CapabilityAnnouncement::new(0xAA, eid, 2, caps_v2);

    index.index(v2);
    index.index(v1); // older — must be a no-op

    let v2_filter = CapabilityFilter::new().require_tag("v2");
    assert_eq!(index.query(&v2_filter), vec![0xAA]);

    let v1_filter = CapabilityFilter::new().require_tag("v1");
    assert!(
        index.query(&v1_filter).is_empty(),
        "older version overwrote the newer one"
    );
}

/// Regression for a cubic-flagged P1: the announcement handler
/// verified the signature against `entity_id` but never checked
/// that `node_id` matched a derivation from `entity_id`. A signer
/// could therefore produce a valid signature claiming any
/// `node_id`, poisoning the capability index and route learning
/// for an unrelated peer.
///
/// The fix asserts `ann.entity_id.node_id() == ann.node_id`
/// after signature verification. This test constructs a forged
/// announcement — A's real entity_id, A's real signature, but a
/// made-up `node_id` — and ships it via the subprotocol channel.
/// The receiver must NOT index the forged node_id.
#[tokio::test]
async fn forged_node_id_rejected_even_with_valid_signature() {
    use net::adapter::net::behavior::SUBPROTOCOL_CAPABILITY_ANN;

    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    // Craft a forged announcement with a fresh keypair. The
    // signature is valid (signer == entity_id), but we deliberately
    // stamp a `node_id` that does NOT match
    // `entity_id.node_id()` — that's the spoof the fix catches.
    let attacker_kp = EntityKeypair::generate();
    let forged_node_id: u64 = 0x1234_5678_9ABC_DEF0;
    assert_ne!(
        forged_node_id,
        attacker_kp.node_id(),
        "fixture: forged_node_id must differ from the signer's real node_id",
    );

    let caps = CapabilitySet::new().add_tag("forged-node-id-probe");
    let mut ann =
        CapabilityAnnouncement::new(forged_node_id, attacker_kp.entity_id().clone(), 1, caps);
    ann.sign(&attacker_kp);
    assert!(
        ann.verify().is_ok(),
        "forged announcement still carries a valid signature",
    );

    // Ship the raw payload via A's subprotocol channel.
    let payload = ann.to_bytes();
    a.send_subprotocol(b.local_addr(), SUBPROTOCOL_CAPABILITY_ANN, &payload)
        .await
        .expect("send forged announcement");

    // B should NOT admit the forged node_id into its index.
    let filter = CapabilityFilter::new().require_tag("forged-node-id-probe");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        !b.find_peers_by_filter(&filter).contains(&forged_node_id),
        "receiver indexed a forged node_id despite derivation mismatch — \
         node_id must be bound to entity_id cryptographically",
    );
}

/// Regression for a cubic-flagged P2: an attacker that forwards a
/// victim's signed announcement through their own session used to
/// get TOFU-bound to the victim's `entity_id`. The announcement's
/// signature was valid (the victim really signed it), but the
/// `from_node` on the receiver side was the attacker, not the
/// origin — so the binding said "attacker's session belongs to
/// victim," later satisfying `authorize_subscribe`'s entity lookup
/// for the wrong peer. The fix restricts TOFU pinning to
/// announcements that arrived **directly** from the origin
/// (`hop_count == 0`); forwarded announcements still update the
/// capability index + routing but no longer touch
/// `peer_entity_ids`.
#[tokio::test]
async fn forwarded_announcement_does_not_tofu_pin_forwarder_to_victim_entity() {
    use net::adapter::net::behavior::SUBPROTOCOL_CAPABILITY_ANN;

    let ports = find_ports(2).await;
    let attacker = build_node(ports[0]).await;
    let receiver = build_node(ports[1]).await;
    handshake(&attacker, &receiver).await;

    // The victim never joins the network — the attacker harvested
    // the victim's signed announcement bytes (say, from an earlier
    // multi-hop path) and now replays them via their own session.
    let victim_kp = EntityKeypair::generate();
    let victim_entity = victim_kp.entity_id().clone();
    let victim_node_id = victim_kp.node_id();

    let caps = CapabilitySet::new().add_tag("forwarded-tofu-probe");
    let mut ann = CapabilityAnnouncement::new(victim_node_id, victim_entity.clone(), 1, caps);
    ann.sign(&victim_kp);
    assert!(ann.verify().is_ok(), "victim's signature is valid");

    // Bump hop_count so the receiver treats this as a forwarded
    // announcement (skips the `ann.node_id == from_node` check and
    // falls into the relay path). Signature verification still
    // passes because `signed_payload` zeros hop_count.
    ann.hop_count = 1;

    let payload = ann.to_bytes();
    attacker
        .send_subprotocol(receiver.local_addr(), SUBPROTOCOL_CAPABILITY_ANN, &payload)
        .await
        .expect("send forwarded announcement");

    // The announcement may still land in the capability index for
    // the victim's node_id — that's fine, the signature is real.
    let filter = CapabilityFilter::new().require_tag("forwarded-tofu-probe");
    let arrived = wait_until(&receiver, |n| {
        n.find_peers_by_filter(&filter).contains(&victim_node_id)
    })
    .await;
    assert!(
        arrived,
        "receiver should still index the victim by node_id — signature is valid",
    );

    // But the attacker's session must NOT be TOFU-bound to the
    // victim's entity_id. That's the core property: a forwarder
    // cannot impersonate the origin for direct-session auth.
    let attacker_node_id = attacker.node_id();
    assert!(
        receiver.peer_entity_id(attacker_node_id)
            != Some(victim_entity.clone()),
        "attacker's session got TOFU-pinned to the victim's entity_id via a forwarded announcement — \
         forwarder can now impersonate origin for channel auth",
    );
}

/// Regression for a cubic-flagged P1/P2: TOFU used to pin the
/// `(from_node → entity_id)` mapping from the first seen
/// announcement regardless of whether the announcement was
/// authenticated. An unauthenticated peer could therefore poison
/// the binding with a victim's `entity_id`, later satisfying
/// token-based channel checks for that spoofed identity. The fix
/// restricts TOFU pinning to signature-verified announcements;
/// unauthenticated deployments that run without signatures get no
/// pin at all (channel auth fails cleanly at "missing entity").
///
/// Explicit opt-out: the receiver sets
/// `require_signed_capabilities = false` so unsigned announcements
/// still reach the dispatch path (the safer post-fix default
/// drops them up front). This test covers the "discovery without
/// signatures" deployment shape and asserts the defence-in-depth
/// state guards still hold under it.
#[tokio::test]
async fn unsigned_announcement_does_not_tofu_pin_entity() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node_with(ports[1], |cfg| cfg.with_require_signed_capabilities(false)).await;
    handshake(&a, &b).await;

    // A announces UNSIGNED caps. B accepts (explicit opt-out
    // below) but must NOT trust `ann.entity_id` enough to pin it.
    a.announce_capabilities_with(
        CapabilitySet::new().add_tag("unsigned-tofu-probe"),
        Duration::from_secs(60),
        false, // unsigned
    )
    .await
    .expect("announce");

    // Index still admits the announcement under the opt-out.
    let filter = CapabilityFilter::new().require_tag("unsigned-tofu-probe");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await;
    assert!(arrived, "unsigned announcement should still index");

    // But the TOFU map must stay empty for this peer — no pin
    // from an unauthenticated announcement.
    assert!(
        b.peer_entity_id(a_id).is_none(),
        "TOFU pin established from an unsigned announcement — \
         unauthenticated entity_id is attacker-controlled input",
    );
}

/// Regression for a cubic-flagged P1: the subnet-assignment write
/// was gated on `signature_verified` but not on
/// `ann.hop_count == 0`, so a **signed** forwarded announcement
/// still wrote `peer_subnets[from_node]` — where `from_node` is
/// the relay, not the origin. A crafted relay could therefore
/// overwrite its own legitimate subnet binding with whatever
/// subnet the forwarded caps would derive to, or poison a
/// legitimate peer's subnet binding by being the last hop on its
/// path. Matching the TOFU-pin pattern: both writes are now gated
/// on `hop_count == 0`.
#[tokio::test]
async fn forwarded_announcement_does_not_write_relay_peer_subnet() {
    use net::adapter::net::behavior::SUBPROTOCOL_CAPABILITY_ANN;
    use net::adapter::net::{SubnetPolicy, SubnetRule};

    let ports = find_ports(2).await;
    let attacker = build_node(ports[0]).await;

    let rule = SubnetRule::new("region:", 0).map("privileged", 1);
    let policy = SubnetPolicy::new().add_rule(rule);
    let receiver = build_node_with(ports[1], |cfg| cfg.with_subnet_policy(Arc::new(policy))).await;
    handshake(&attacker, &receiver).await;

    // Harvested victim bytes: a real, signature-valid announcement
    // with caps that would classify the origin into the non-GLOBAL
    // subnet. The attacker replays it via its own session with
    // hop_count=1 (forwarded).
    let victim_kp = EntityKeypair::generate();
    let caps = CapabilitySet::new().add_tag("region:privileged");
    let mut ann =
        CapabilityAnnouncement::new(victim_kp.node_id(), victim_kp.entity_id().clone(), 1, caps);
    ann.sign(&victim_kp);
    assert!(ann.verify().is_ok(), "victim's signature is valid");
    ann.hop_count = 1;

    attacker
        .send_subprotocol(
            receiver.local_addr(),
            SUBPROTOCOL_CAPABILITY_ANN,
            &ann.to_bytes(),
        )
        .await
        .expect("send forwarded announcement");

    // The index may admit the victim by node_id (signature is real),
    // but the attacker's own session must NOT have been shifted into
    // the forwarded subnet — that binding belongs to the origin,
    // not the relay.
    let filter = CapabilityFilter::new().require_tag("region:privileged");
    let victim_node_id = victim_kp.node_id();
    let arrived = wait_until(&receiver, |n| {
        n.find_peers_by_filter(&filter).contains(&victim_node_id)
    })
    .await;
    assert!(arrived, "receiver should still index the victim by node_id");

    // Give the dispatch path a beat in case the subnet write lags.
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(
        receiver.peer_subnet(attacker.node_id()).is_none(),
        "forwarded announcement wrote the relay's subnet — a crafted last \
         hop can reshape any legitimate peer's SubnetLocal visibility",
    );
}

/// Regression for a cubic-flagged P1: even with the default
/// (`require_signed_capabilities = true`) dropping unsigned
/// announcements up-front, we want belt-and-braces so an
/// explicit opt-out for discovery can't accidentally re-open the
/// auth surface. `peer_subnets` is populated from
/// `ann.capabilities` via the subnet policy and is later consulted
/// by `subnet_visible` on the publish / subscribe paths — an
/// unsigned announcement must not be allowed to pick the peer's
/// subnet. This test opts out of the signature requirement, sends
/// an unsigned announcement whose caps would land the peer in a
/// non-GLOBAL subnet under any plausible policy, and asserts the
/// subnet binding stays unwritten.
#[tokio::test]
async fn unsigned_announcement_does_not_write_peer_subnet() {
    use net::adapter::net::{SubnetPolicy, SubnetRule};

    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;

    // Receiver opts out of require_signed AND installs a subnet
    // policy whose rule maps `region:privileged` to a non-zero
    // level. `SubnetPolicy::assign` matches on a tag prefix; a
    // peer carrying that tag lands in a non-GLOBAL subnet. If the
    // write path were live, an attacker could drop itself into
    // that subnet just by announcing the matching tag.
    let rule = SubnetRule::new("region:", 0).map("privileged", 1);
    let policy = SubnetPolicy::new().add_rule(rule);
    let b = build_node_with(ports[1], |cfg| {
        cfg.with_require_signed_capabilities(false)
            .with_subnet_policy(Arc::new(policy))
    })
    .await;
    handshake(&a, &b).await;

    a.announce_capabilities_with(
        CapabilitySet::new().add_tag("region:privileged"),
        Duration::from_secs(60),
        false, // unsigned — attacker lying about caps
    )
    .await
    .expect("announce");

    // Capability index is allowed to pick it up (opt-out lets
    // discovery still work).
    let filter = CapabilityFilter::new().require_tag("region:privileged");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await;
    assert!(
        arrived,
        "unsigned announcement should still index under opt-out",
    );

    // Give the dispatch path another beat in case the subnet
    // write lags the index insert.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // `peer_subnets` must stay empty for this peer — the unsigned
    // ann doesn't write here, so `subnet_visible` decisions on
    // `SubnetLocal` channels can't be steered by attacker input.
    assert!(
        b.peer_subnet(a_id).is_none(),
        "unsigned announcement was allowed to pick the peer's subnet — \
         subnet_visible decisions become attacker-controlled",
    );
}
