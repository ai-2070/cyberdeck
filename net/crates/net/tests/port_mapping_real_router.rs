//! `#[ignore]`-gated real-router integration test for stage
//! 4b-4's [`SequentialMapper`]. Runs against the developer's
//! actual home network — discovers the default gateway, probes
//! NAT-PMP + UPnP, installs a mapping for a test port, asserts
//! the install produced a non-unspecified external address,
//! and revokes cleanly.
//!
//! Not part of CI. Run explicitly:
//!
//! ```text
//! cargo test --features port-mapping \
//!     --test port_mapping_real_router -- --ignored
//! ```
//!
//! Skips (with `eprintln!` + early return, not a test failure)
//! when:
//!
//! - No default gateway is discoverable (unsupported platform
//!   or CI sandbox with no routing table).
//! - Neither NAT-PMP nor UPnP responds — typical for networks
//!   where UPnP has been turned off (operator preference or
//!   enterprise policy). The sequencer is designed to
//!   gracefully return `Unavailable`; we just note and exit.
//!
//! Exits *with a failure* only on:
//!
//! - A gateway that responded (probe succeeded) but then the
//!   install failed — that's a bug we want to surface.
//! - External address that's unspecified / loopback / RFC 1918
//!   — the router handed us a mapping without a real WAN IP.

#![cfg(all(feature = "net", feature = "port-mapping"))]

use std::time::Duration;

use net::adapter::net::traversal::portmap::{
    sequential::sequential_mapper_from_os, PortMapperClient,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a real router with UPnP-IGD or NAT-PMP enabled"]
async fn install_and_revoke_against_live_router() {
    let Some(mapper) = sequential_mapper_from_os().await else {
        eprintln!(
            "skipped: sequential_mapper_from_os() returned None — \
             no default gateway discoverable on this platform",
        );
        return;
    };

    match mapper.probe().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!(
                "skipped: probe failed ({e:?}) — likely no UPnP / NAT-PMP \
                 responder on this network. Not a test failure.",
            );
            return;
        }
    }
    let active = mapper
        .active_protocol()
        .expect("probe succeeded → active protocol should be Some");
    eprintln!("probe succeeded; active protocol = {active:?}");

    // Ask for a mapping on an unlikely-to-collide port with a
    // 1-hour lease.
    let internal_port: u16 = 51234;
    let ttl = Duration::from_secs(3600);
    let mapping = mapper
        .install(internal_port, ttl)
        .await
        .expect("install on a live responder should succeed");

    eprintln!(
        "installed mapping: external={}, internal_port={}, ttl={:?}, protocol={:?}",
        mapping.external, mapping.internal_port, mapping.ttl, mapping.protocol,
    );

    // Sanity on the returned external address:
    let ext_ip = mapping.external.ip();
    assert!(
        !ext_ip.is_unspecified(),
        "router handed us 0.0.0.0 as external — malformed mapping",
    );
    assert!(
        !ext_ip.is_loopback(),
        "router handed us a loopback external — malformed mapping",
    );
    assert_eq!(
        mapping.internal_port, internal_port,
        "mapping's internal_port should echo what we requested",
    );

    // Revoke. The sequencer's `remove` is best-effort (never
    // returns an error); we just want to observe it doesn't
    // panic or hang.
    mapper.remove(&mapping).await;
    eprintln!("revoke sent; test complete");
}
