//! Error type for NetDB operations.

use thiserror::Error;

use super::super::cortex::CortexAdapterError;

/// Errors produced by [`super::NetDb`] / [`super::NetDbBuilder`]
/// / [`super::NetDbSnapshot`].
#[derive(Debug, Error)]
pub enum NetDbError {
    /// An underlying CortEX adapter operation failed.
    #[error("cortex: {0}")]
    Cortex(#[from] CortexAdapterError),

    /// A model was accessed that wasn't included at build time.
    /// (Only raised by the `try_*` accessors; the panicking
    /// accessors never return this — they panic instead.)
    #[error("model '{0}' was not included in this NetDb")]
    ModelNotIncluded(&'static str),

    /// Snapshot encode / decode failure.
    #[error("snapshot: {0}")]
    Snapshot(String),
}
