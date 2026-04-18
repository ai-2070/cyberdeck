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

    /// An encoding helper (e.g. `append_bincode`) failed to serialize.
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
}

impl RedexError {
    /// Construct from any `std::io::Error` with its message preserved.
    pub fn io(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
