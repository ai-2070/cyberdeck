//! Fuzz target: `CapabilityAnnouncement::from_bytes`.
//!
//! This is the highest-value decoder on the crate's wire surface:
//! a peer-signed JSON envelope that gets cached in the local
//! capability index, consulted on the routing path, checked
//! against subnet policy, and used to authorize subscribes. Every
//! byte is attacker-controlled (or at least can be injected by a
//! malicious peer on the routed-handshake path before signature
//! verification kicks in).
//!
//! Invariants asserted:
//!
//! - No panic on any byte sequence.
//! - If `from_bytes` returns `Some(ann)`, `ann.to_bytes()` yields
//!   a buffer that round-trips back to `Some(ann')` with the same
//!   observable fields. Regression-guards the serde round-trip.
//! - If the round-trip succeeds, calling `verify()` on it does
//!   not panic (it may return `Err` — most fuzz inputs will have
//!   no valid signature, that's fine).

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::adapter::net::behavior::capability::CapabilityAnnouncement;

fuzz_target!(|data: &[u8]| {
    let Some(ann) = CapabilityAnnouncement::from_bytes(data) else {
        return;
    };

    // Round-trip via canonical serialization.
    let bytes = ann.to_bytes();
    let Some(ann2) = CapabilityAnnouncement::from_bytes(&bytes) else {
        // Serde asymmetry — to_bytes → from_bytes must succeed
        // if from_bytes → Some did.
        panic!(
            "CapabilityAnnouncement round-trip failed: original {} bytes, \
             serialized {} bytes",
            data.len(),
            bytes.len()
        );
    };

    // Observable-field equality. Not structural eq because
    // `to_bytes` is canonical and the input may have had extra
    // whitespace or field ordering — but the semantic fields
    // must match.
    assert_eq!(ann.node_id, ann2.node_id);
    assert_eq!(ann.entity_id, ann2.entity_id);
    assert_eq!(ann.version, ann2.version);
    assert_eq!(ann.ttl_secs, ann2.ttl_secs);
    assert_eq!(ann.hop_count, ann2.hop_count);
    assert_eq!(ann.reflex_addr, ann2.reflex_addr);

    // Signature verification must not panic on any input —
    // including announcements with malformed / absent / wrong-
    // length signatures. The result (Ok / Err) doesn't matter
    // here; we only pin panic-freedom.
    let _ = ann.verify();

    // is_expired must not panic either — exercises the
    // timestamp_ns + ttl_secs arithmetic path.
    let _ = ann.is_expired();
});
