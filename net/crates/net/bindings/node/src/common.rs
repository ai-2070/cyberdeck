//! Shared helpers used by more than one NAPI module.

use napi::bindgen_prelude::*;

/// Pure-logic validation of a BigInt's `(sign_bit, words)`
/// decomposition against "fits losslessly in `u64`." Returns the
/// word on success or a stable error-message string on failure.
/// Extracted from [`bigint_u64`] so it's unit-testable without
/// linking napi FFI — the NAPI binding's test harness can't
/// stand up a Node.js runtime for Drop-impl symbols.
///
/// The rules (one per failure mode):
///
/// - `sign_bit = true` → negative; reject with `"expected
///   non-negative BigInt"`. A raw `BigInt::get_u64()` would
///   bitwise-reinterpret a negative and silently yield a huge
///   u64.
/// - `words.len() != 1` → doesn't fit in a single u64. Reject
///   with `"BigInt value exceeds u64 range"`. Any extra word —
///   even a zero upper word — is treated as oversize because
///   an idiomatic producer wouldn't emit a trailing zero.
pub(crate) fn validate_bigint_u64_parts(
    sign_bit: bool,
    words: &[u64],
) -> std::result::Result<u64, &'static str> {
    if sign_bit {
        return Err("expected non-negative BigInt");
    }
    if words.len() != 1 {
        return Err("BigInt value exceeds u64 range");
    }
    Ok(words[0])
}

/// Convert a JS `BigInt` to `u64`, rejecting negatives and values
/// that exceed `u64::MAX`. The napi `get_u64()` tuple is `(signed,
/// value, lossless)`; silently accepting either flag corrupts ids,
/// timestamps, or sequences since none of them are meaningful as
/// negative or truncated.
///
/// Thin wrapper around [`validate_bigint_u64_parts`] that
/// translates the pure-logic error into a napi `Error`.
#[inline]
pub(crate) fn bigint_u64(b: BigInt) -> Result<u64> {
    validate_bigint_u64_parts(b.sign_bit, &b.words)
        .map_err(|msg| Error::from_reason(msg.to_string()))
}

#[cfg(test)]
mod tests {
    //! Regression tests for [`validate_bigint_u64_parts`] — the
    //! pure-logic half of `bigint_u64`. See [`bigint_u64`] docs
    //! for why the split exists (napi `Error`'s Drop calls
    //! Node-provided FFI symbols that aren't available in a
    //! standalone `cargo test` binary).
    //!
    //! History: cubic flagged that several NAT-surface call
    //! sites (`peer_nat_type`, `probe_reflex`, `connect_direct`)
    //! bypassed this validation by destructuring
    //! `BigInt::get_u64()` directly, silently accepting negatives
    //! and truncations. All call sites were rewritten to go
    //! through `bigint_u64` → this validator; these tests pin
    //! the validator so the NAT surface can't silently accept
    //! out-of-range ids again.
    use super::validate_bigint_u64_parts;

    #[test]
    fn accepts_positive_single_word() {
        assert_eq!(
            validate_bigint_u64_parts(false, &[0xDEAD_BEEF_CAFE_F00D]),
            Ok(0xDEAD_BEEF_CAFE_F00D),
        );
    }

    #[test]
    fn accepts_zero() {
        assert_eq!(validate_bigint_u64_parts(false, &[0]), Ok(0));
    }

    #[test]
    fn accepts_u64_max() {
        assert_eq!(validate_bigint_u64_parts(false, &[u64::MAX]), Ok(u64::MAX));
    }

    #[test]
    fn rejects_negative() {
        // A raw `BigInt::get_u64()` would reinterpret this as a
        // huge u64 and silently target the wrong peer. The
        // validator must reject + surface "non-negative".
        let err = validate_bigint_u64_parts(true, &[5]).unwrap_err();
        assert!(
            err.contains("non-negative"),
            "error should mention non-negative; got {err:?}",
        );
    }

    #[test]
    fn rejects_negative_zero() {
        // Degenerate-but-possible shape. Sign-check fires
        // regardless of magnitude.
        assert!(validate_bigint_u64_parts(true, &[0]).is_err());
    }

    #[test]
    fn rejects_value_exceeding_u64_max() {
        // 2^64 + 1 in two-word form. A raw destructure would
        // silently discard words[1] and return 1.
        let err = validate_bigint_u64_parts(false, &[1, 1]).unwrap_err();
        assert!(
            err.contains("exceeds u64 range"),
            "error should mention range; got {err:?}",
        );
    }

    #[test]
    fn rejects_multi_word_even_when_upper_words_zero() {
        // An idiomatic BigInt producer wouldn't emit trailing
        // zero words, but a misbehaving caller might. Our
        // `words.len() != 1` check is strict — any extra word,
        // zero or not, rejects.
        assert!(validate_bigint_u64_parts(false, &[42, 0, 0]).is_err());
    }
}

