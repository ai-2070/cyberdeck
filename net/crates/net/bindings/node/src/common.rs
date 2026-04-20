//! Shared helpers used by more than one NAPI module.

use napi::bindgen_prelude::*;

/// Convert a JS `BigInt` to `u64`, rejecting negatives and values
/// that exceed `u64::MAX`. The napi `get_u64()` tuple is `(signed,
/// value, lossless)`; silently accepting either flag corrupts ids,
/// timestamps, or sequences since none of them are meaningful as
/// negative or truncated.
#[inline]
pub(crate) fn bigint_u64(b: BigInt) -> Result<u64> {
    let (signed, value, lossless) = b.get_u64();
    if signed {
        return Err(Error::from_reason(
            "expected non-negative BigInt".to_string(),
        ));
    }
    if !lossless {
        return Err(Error::from_reason(
            "BigInt value exceeds u64 range".to_string(),
        ));
    }
    Ok(value)
}
