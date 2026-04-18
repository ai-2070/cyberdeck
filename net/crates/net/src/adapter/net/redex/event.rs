//! `RedexEvent` — the materialized event yielded by tail / read_range.

use bytes::Bytes;

use super::entry::RedexEntry;

/// A materialized RedEX event: the 20-byte index record plus the
/// payload bytes.
///
/// For `INLINE` entries, `payload` is the 8 inline bytes (held in a
/// short `Bytes`). For heap entries, `payload` is a slice over the
/// file's payload segment.
#[derive(Debug, Clone)]
pub struct RedexEvent {
    /// The 20-byte on-disk record, verbatim.
    pub entry: RedexEntry,
    /// The materialized payload.
    pub payload: Bytes,
}
