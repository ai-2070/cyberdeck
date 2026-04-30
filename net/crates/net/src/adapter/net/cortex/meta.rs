//! Fixed 20-byte `EventMeta` prefix on every CortEX-adapted payload.
//!
//! Wire layout (little-endian, 20 bytes total):
//!
//! ```text
//! ┌──────────┬───────┬──────┬─────────────┬───────────┬──────────┐
//! │ dispatch │ flags │ _pad │ origin_hash │ seq_or_ts │ checksum │
//! │   u8     │  u8   │  2B  │    u32      │    u64    │   u32    │
//! └──────────┴───────┴──────┴─────────────┴───────────┴──────────┘
//! ```
//!
//! `seq_or_ts` is deliberately NOT interpreted by the adapter. Envelope
//! authors pick per file: per-origin monotonic counter (deterministic
//! fold order) OR unix nanos (wall-clock ordering). Mixing within one
//! file breaks fold ordering assumptions.

/// Size of an `EventMeta` in its wire / on-disk format.
pub const EVENT_META_SIZE: usize = 20;

/// Dispatch value reserved for "raw" payloads — no CortEX-level
/// semantics. Callers that don't need dispatch routing should use
/// this.
pub const DISPATCH_RAW: u8 = 0x00;

/// Flag bit: this event is part of a causal chain.
pub const FLAG_CAUSAL: u8 = 0b0000_0001;
/// Flag bit: this event carries a continuity proof.
pub const FLAG_CONTINUITY_PROOF: u8 = 0b0000_0010;

/// Fixed 20-byte prefix on every payload appended through the
/// CortEX adapter.
///
/// The in-memory layout is whatever the compiler chooses; the wire /
/// on-disk format is produced by [`Self::to_bytes`] and consumed by
/// [`Self::from_bytes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMeta {
    /// Event classifier. `0x00..0x7F` reserved for CortEX-internal
    /// dispatches; `0x80..0xFF` for application / vendor use.
    pub dispatch: u8,
    /// Causal / continuity / proof bits. See `FLAG_*` constants.
    pub flags: u8,
    /// Reserved; must be zero on write, ignored on read.
    pub _pad: [u8; 2],
    /// Producer identity (xxh3 truncation of the origin).
    pub origin_hash: u32,
    /// Per-origin monotonic counter OR unix nanos. Application
    /// identity — orthogonal to the RedEX storage sequence.
    pub seq_or_ts: u64,
    /// xxh3 truncation of the type-specific tail (the bytes after
    /// the 20-byte prefix in the RedEX payload).
    pub checksum: u32,
}

impl EventMeta {
    /// Build an `EventMeta` with zeroed pad bytes.
    pub fn new(dispatch: u8, flags: u8, origin_hash: u32, seq_or_ts: u64, checksum: u32) -> Self {
        Self {
            dispatch,
            flags,
            _pad: [0; 2],
            origin_hash,
            seq_or_ts,
            checksum,
        }
    }

    /// Encode to the 20-byte little-endian wire format. The reserved
    /// `_pad` bytes are always written as zero regardless of what the
    /// caller stuffed into them — the wire contract says "zero on
    /// write, ignored on read."
    pub fn to_bytes(&self) -> [u8; EVENT_META_SIZE] {
        let mut out = [0u8; EVENT_META_SIZE];
        out[0] = self.dispatch;
        out[1] = self.flags;
        // out[2..4] stays [0, 0] (reserved pad).
        out[4..8].copy_from_slice(&self.origin_hash.to_le_bytes());
        out[8..16].copy_from_slice(&self.seq_or_ts.to_le_bytes());
        out[16..20].copy_from_slice(&self.checksum.to_le_bytes());
        out
    }

    /// Decode from a 20-byte slice. Returns `None` if the slice is
    /// shorter than 20 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < EVENT_META_SIZE {
            return None;
        }
        Some(Self {
            dispatch: bytes[0],
            flags: bytes[1],
            _pad: [bytes[2], bytes[3]],
            origin_hash: u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes")),
            seq_or_ts: u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes")),
            checksum: u32::from_le_bytes(bytes[16..20].try_into().expect("4 bytes")),
        })
    }

    /// True if `flags & bits != 0`.
    #[inline]
    pub fn has_flag(&self, bits: u8) -> bool {
        self.flags & bits != 0
    }
}

/// Compute the corruption-detection checksum of a payload tail —
/// the xxh3 hash truncated to the low 32 bits. Stamped into
/// [`EventMeta::checksum`].
///
/// **Scope (BUG #135):** this is an *accidental-corruption*
/// detector, NOT a tamper detector. Two specific limits make it
/// unsuitable for tamper-resistance:
///
/// 1. **32-bit truncation.** A 32-bit unkeyed hash has roughly
///    1-in-2³² collision probability per pair of distinct
///    payloads — fine against random bit-flips on disk, but only
///    ~1-in-2¹⁶ across a long-running file under the birthday
///    bound. Adequate for "did the on-disk record decode
///    correctly?" not "did an attacker craft a payload that
///    matches?".
/// 2. **Unkeyed.** An attacker who can write to the on-disk
///    redex file can recompute the matching checksum trivially
///    by hashing whatever payload they substitute. There is no
///    secret bound to this value.
///
/// Callers that need tamper detection (rather than corruption
/// detection) must layer a keyed MAC at a higher level — e.g.
/// the AEAD-protected mesh packet envelope. The cortex fold
/// paths use this value only to surface obviously-broken on-disk
/// records as `RedexError::Decode` (see BUG #141), not as a
/// security boundary.
///
/// Disk-recovery / external inspection tools can reproduce the
/// value by hashing the bytes after the 20-byte prefix.
#[inline]
pub fn compute_checksum(tail: &[u8]) -> u32 {
    xxhash_rust::xxh3::xxh3_64(tail) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_is_twenty() {
        assert_eq!(EVENT_META_SIZE, 20);
    }

    #[test]
    fn test_roundtrip_all_fields_distinct() {
        let m = EventMeta::new(
            0x42,
            FLAG_CAUSAL | FLAG_CONTINUITY_PROOF,
            0xDEAD_BEEF,
            0x0123_4567_89AB_CDEF,
            0xCAFE_BABE,
        );
        let bytes = m.to_bytes();
        assert_eq!(bytes.len(), 20);
        let decoded = EventMeta::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, m);
        assert_eq!(decoded.dispatch, 0x42);
        assert_eq!(decoded.flags, FLAG_CAUSAL | FLAG_CONTINUITY_PROOF);
        assert_eq!(decoded.origin_hash, 0xDEAD_BEEF);
        assert_eq!(decoded.seq_or_ts, 0x0123_4567_89AB_CDEF);
        assert_eq!(decoded.checksum, 0xCAFE_BABE);
    }

    #[test]
    fn test_regression_pad_is_zeroed_on_write() {
        // Regression: `to_bytes` used to copy `self._pad` verbatim
        // into the output buffer. The struct doc says "reserved;
        // must be zero on write, ignored on read" — but `_pad` is
        // `pub`, so a caller constructing `EventMeta` via struct
        // literal syntax could stamp non-zero pad into the wire
        // format. The fix leaves bytes [2..4] as zero regardless of
        // struct contents.
        let m = EventMeta {
            dispatch: 0x42,
            flags: 0,
            _pad: [0xAA, 0xBB], // non-zero — would leak on write
            origin_hash: 0xDEAD_BEEF,
            seq_or_ts: 1,
            checksum: 0,
        };
        let bytes = m.to_bytes();
        assert_eq!(
            &bytes[2..4],
            &[0u8, 0u8],
            "pad bytes must be zero on write regardless of struct contents"
        );
    }

    #[test]
    fn test_zero_roundtrip() {
        let m = EventMeta::new(0, 0, 0, 0, 0);
        let decoded = EventMeta::from_bytes(&m.to_bytes()).unwrap();
        assert_eq!(decoded, m);
    }

    #[test]
    fn test_unknown_dispatch_decodes_fine() {
        // 0xFE is in application / vendor space — adapter must not
        // reject or special-case it.
        let m = EventMeta::new(0xFE, 0, 1, 2, 3);
        let decoded = EventMeta::from_bytes(&m.to_bytes()).unwrap();
        assert_eq!(decoded.dispatch, 0xFE);
    }

    #[test]
    fn test_short_slice_returns_none() {
        let buf = [0u8; 19];
        assert!(EventMeta::from_bytes(&buf).is_none());
    }

    #[test]
    fn test_nonzero_pad_tolerated_on_read() {
        // Pad bytes must be zero on write, but garbage on read should
        // not corrupt other fields.
        let mut bytes = [0u8; 20];
        bytes[2] = 0xAA;
        bytes[3] = 0xBB;
        bytes[8..16].copy_from_slice(&0x1234_5678_9ABC_DEF0u64.to_le_bytes());
        let decoded = EventMeta::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.seq_or_ts, 0x1234_5678_9ABC_DEF0);
        // Pad is carried through verbatim — surface it for forensic
        // inspection but it has no semantic meaning.
        assert_eq!(decoded._pad, [0xAA, 0xBB]);
    }

    #[test]
    fn test_has_flag() {
        let m = EventMeta::new(0, FLAG_CAUSAL, 0, 0, 0);
        assert!(m.has_flag(FLAG_CAUSAL));
        assert!(!m.has_flag(FLAG_CONTINUITY_PROOF));
    }

    #[test]
    fn test_field_boundaries_isolated() {
        // Write max values in each field; decode must return them all.
        let m = EventMeta::new(u8::MAX, u8::MAX, u32::MAX, u64::MAX, u32::MAX);
        let decoded = EventMeta::from_bytes(&m.to_bytes()).unwrap();
        assert_eq!(decoded, m);
    }
}
