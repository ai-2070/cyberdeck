//! Error type for RedEX operations.

use thiserror::Error;

/// Errors produced by RedEX operations.
#[derive(Debug, Error)]
pub enum RedexError {
    /// Payload is larger than the configured segment supports.
    #[error("payload too large: {size} bytes (max {max})")]
    PayloadTooLarge {
        /// Attempted payload size.
        size: usize,
        /// Maximum accepted payload size.
        max: usize,
    },

    /// Segment offset exceeded `u32::MAX`.
    ///
    /// Fires on long-running persistent files whose lifetime heap
    /// bytes have crossed the 4 GB `payload_offset` field width.
    /// Recoverable by reopening the file; disk recovery resets the
    /// base offset. Sweep-time offset renormalization lands in v2.
    #[error("segment offset overflow: offset {offset} exceeds u32::MAX")]
    SegmentOffsetOverflow {
        /// The overflowing absolute offset.
        offset: u64,
    },

    /// Requested sequence is outside the file's retained range.
    #[error("sequence {seq} is outside the retained range [{lo}, {hi})")]
    SeqOutOfRange {
        /// The requested sequence.
        seq: u64,
        /// Lowest retained sequence (inclusive).
        lo: u64,
        /// Next sequence to be assigned (exclusive upper bound).
        hi: u64,
    },

    /// A channel name was rejected (e.g. invalid format, collision on open).
    #[error("channel error: {0}")]
    Channel(String),

    /// An encoding helper (e.g. `append_postcard`) failed to serialize.
    #[error("encode failed: {0}")]
    Encode(String),

    /// Caller is not authorized to append or tail this file.
    #[error("unauthorized")]
    Unauthorized,

    /// Underlying I/O failure (disk segment only).
    #[error("io: {0}")]
    Io(String),

    /// The file has been closed.
    #[error("file closed")]
    Closed,

    /// A tail subscriber fell behind the per-subscription buffer and
    /// was disconnected. Delivery is best-effort: under saturation
    /// the buffer may be too full to accept this signal itself, in
    /// which case the subscriber observes only a plain stream end.
    #[error("tail subscriber lagged; stream disconnected")]
    Lagged,
}

impl RedexError {
    /// Construct from any `std::io::Error` with its message preserved.
    pub fn io(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }

    /// Returns `true` when this error represents a per-event
    /// recoverable failure (a single bad event, NOT an
    /// underlying-storage failure that affects every subsequent
    /// event). The fold-error-policy interpreter treats these as
    /// "always skip-and-continue" even under
    /// [`super::super::cortex::config::FoldErrorPolicy::Stop`]
    /// — otherwise a single corrupt postcard tail (or a 32-bit
    /// checksum collision; see BUG #135) wedges the fold task
    /// permanently and DoSes a multi-tenant cortex instance via
    /// one bad event. (BUG #141.)
    ///
    /// `Encode` covers postcard decode failures and `EventMeta`
    /// shape mismatches — both per-event. `Io` / `Closed` /
    /// `Lagged` are stream-level and properly halt under `Stop`.
    /// `PayloadTooLarge` / `SegmentOffsetOverflow` /
    /// `SeqOutOfRange` / `Channel` / `Unauthorized` are
    /// configuration / authorization issues that benefit from
    /// halting rather than silently skipping; not recoverable.
    pub fn is_recoverable_decode(&self) -> bool {
        matches!(self, Self::Encode(_))
    }
}
