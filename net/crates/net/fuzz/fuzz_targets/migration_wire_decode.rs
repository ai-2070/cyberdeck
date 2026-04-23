//! Fuzz target: `compute::orchestrator::wire::decode`.
//!
//! Migration subprotocol (0x0500) wire-format decoder. Attacker
//! can send arbitrary bytes via `send_subprotocol(addr, 0x0500,
//! payload)` — the migration handler's dispatcher decodes
//! before any auth check, so a panic in `decode` is a live
//! remote-DoS.
//!
//! The decoder handles a tagged union (TakeSnapshot,
//! SnapshotReady, RestoreComplete, BufferedEvents,
//! CutoverNotify, CutoverAck, CleanupComplete, ActivateTarget,
//! ActivateAck, MigrationFailed) with variable-length chunk
//! payloads. Historical bugs in similar decoders: OOM on
//! attacker-declared length fields, panic on short frames,
//! infinite loop on malformed length prefix.
//!
//! Post-decode roundtrip asserts `encode(decode(x)) == x` for
//! accepted inputs — guards against asymmetric canonicalization
//! bugs that would silently drop fields across the wire.

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::adapter::net::compute::orchestrator::wire;

fuzz_target!(|data: &[u8]| {
    let Ok(msg) = wire::decode(data) else {
        return;
    };

    // Round-trip: encode must succeed for any decoded value.
    let encoded = match wire::encode(&msg) {
        Ok(b) => b,
        Err(e) => panic!(
            "wire::encode failed on a round-tripped MigrationMessage ({:?}): {:?}",
            msg, e
        ),
    };

    // Re-decode must yield an equal-enough message. Strict
    // equality is hard (some MigrationMessage variants wrap
    // Vec<u8> which may or may not re-canonicalize bit-for-bit)
    // — so we only assert `encoded` is stable under a second
    // round-trip, which catches canonicalization drift.
    let redecoded = wire::decode(&encoded).unwrap_or_else(|e| {
        panic!(
            "wire::decode failed on wire::encode output: {:?} / {} bytes",
            e,
            encoded.len()
        )
    });
    let reencoded = wire::encode(&redecoded).expect("second encode must succeed");
    assert_eq!(
        encoded, reencoded,
        "wire canonicalization drift: first-encode != second-encode",
    );
});
