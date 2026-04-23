//! Fuzz target: `natpmp::decode_response`.
//!
//! NAT-PMP response packets are received over UDP from the
//! router; while RFC 6886 §3.1 requires clients to filter by
//! source IP (and we do), a peer on the LAN that the gateway
//! happens to route through can still inject responses. The
//! decoder is the last line of defense before a decoded
//! `NatPmpResponse` feeds into the mapper's external-IP cache.
//!
//! Hand-rolled malformed-input coverage exists
//! (`decode_response_never_panics_on_malformed_input` in
//! `natpmp.rs` tests). This fuzzer generalizes that to arbitrary
//! byte sequences — in particular, finds partial/short packets
//! that the fixed-offset layout would trip over if any offset
//! check regresses.

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::adapter::net::traversal::portmap::natpmp::decode_response;

fuzz_target!(|data: &[u8]| {
    let _ = decode_response(data);
});
