//! Error type for CortEX adapter operations.

use thiserror::Error;

use super::super::redex::RedexError;

/// Errors produced by [`super::CortexAdapter`] operations.
#[derive(Debug, Error)]
pub enum CortexAdapterError {
    /// Underlying RedEX storage error.
    #[error("redex: {0}")]
    Redex(#[from] RedexError),

    /// The adapter has been closed.
    #[error("adapter closed")]
    Closed,

    /// The fold task has stopped (first fold error under
    /// [`super::FoldErrorPolicy::Stop`]). Holds the RedEX sequence
    /// at which the fold stopped.
    #[error("fold stopped at seq {seq}")]
    FoldStopped {
        /// The RedEX sequence of the event whose fold returned an error.
        seq: u64,
    },
}
