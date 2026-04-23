//! Fuzz target: `RoutingHeader::from_bytes`.
//!
//! 18-byte fixed routing header at the front of every multi-hop
//! routed packet. Tight parser — offsets are hardcoded — which
//! makes it a low-complexity target, but it's also the first
//! thing the receive loop reaches on any inbound packet with
//! the `0x5452` magic, so a panic here is a remote-DoS on a
//! byte-pattern an attacker can fabricate trivially.
//!
//! Round-trip invariant: if `from_bytes` returns Some, then
//! serializing that value and re-parsing yields the same value
//! byte-for-byte. Pins the parser↔serializer symmetry that
//! tests like `test_regression_proximity_graph_local_id_matches_peer_encoding`
//! lean on.

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::adapter::net::RoutingHeader;

fuzz_target!(|data: &[u8]| {
    let Some(hdr) = RoutingHeader::from_bytes(data) else {
        return;
    };

    let serialized = hdr.to_bytes();
    let reparsed = RoutingHeader::from_bytes(&serialized).unwrap_or_else(|| {
        panic!(
            "RoutingHeader round-trip failed: hdr={:?}, serialized={:?}",
            hdr, serialized
        )
    });
    assert_eq!(hdr, reparsed, "routing header round-trip lost a field");
});
