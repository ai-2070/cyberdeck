//! Error types for the Net event bus.

use thiserror::Error;

/// Errors that can occur during event ingestion.
#[derive(Debug, Error)]
pub enum IngestionError {
    /// Ring buffer is full and backpressure policy rejected the event.
    #[error("backpressure: ring buffer full")]
    Backpressure,

    /// Event was dropped due to sampling/decimation policy.
    #[error("event dropped due to sampling")]
    Sampled,

    /// The event bus has been shut down.
    #[error("event bus is shutting down")]
    ShuttingDown,

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Errors that can occur in adapter operations.
#[derive(Debug, Error)]
pub enum AdapterError {
    /// Transient error - operation can be retried.
    #[error("transient error: {0}")]
    Transient(String),

    /// Fatal error - adapter is in an unrecoverable state.
    #[error("fatal error: {0}")]
    Fatal(String),

    /// Backend cannot accept more data - apply backpressure.
    #[error("backend backpressure")]
    Backpressure,

    /// Connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl AdapterError {
    /// Returns true if this error is retryable.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Transient(_) | Self::Backpressure)
    }

    /// Returns true if this error is fatal.
    #[inline]
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }
}

/// Errors that can occur during event consumption/polling.
#[derive(Debug, Error)]
pub enum ConsumerError {
    /// Adapter error during polling.
    #[error("adapter error: {0}")]
    Adapter(#[from] AdapterError),

    /// Invalid cursor format.
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// Invalid filter specification.
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
}

/// Result type alias for ingestion operations.
pub type IngestionResult<T> = Result<T, IngestionError>;

/// Result type alias for adapter operations.
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Result type alias for consumer operations.
pub type ConsumerResult<T> = Result<T, ConsumerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_error_is_retryable() {
        assert!(AdapterError::Transient("temp".into()).is_retryable());
        assert!(AdapterError::Backpressure.is_retryable());
        assert!(!AdapterError::Fatal("dead".into()).is_retryable());
        assert!(!AdapterError::Connection("refused".into()).is_retryable());
        assert!(!AdapterError::Serialization("bad json".into()).is_retryable());
    }

    #[test]
    fn test_adapter_error_is_fatal() {
        assert!(AdapterError::Fatal("dead".into()).is_fatal());
        assert!(!AdapterError::Transient("temp".into()).is_fatal());
        assert!(!AdapterError::Backpressure.is_fatal());
        assert!(!AdapterError::Connection("refused".into()).is_fatal());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            IngestionError::Backpressure.to_string(),
            "backpressure: ring buffer full"
        );
        assert_eq!(
            IngestionError::Sampled.to_string(),
            "event dropped due to sampling"
        );
        assert_eq!(
            IngestionError::ShuttingDown.to_string(),
            "event bus is shutting down"
        );
        assert_eq!(
            AdapterError::Transient("timeout".into()).to_string(),
            "transient error: timeout"
        );
        assert_eq!(
            AdapterError::Fatal("crash".into()).to_string(),
            "fatal error: crash"
        );
        assert_eq!(
            AdapterError::Backpressure.to_string(),
            "backend backpressure"
        );
    }

    #[test]
    fn test_connection_error_not_retryable() {
        // Connection errors cover both transient failures ("send failed") and
        // permanent ones ("adapter not initialized"). Since we can't distinguish
        // them at the type level, Connection is conservatively non-retryable.
        // The batch dispatch path retries all errors regardless of this flag.
        assert!(!AdapterError::Connection("refused".into()).is_retryable());
        assert!(!AdapterError::Connection("adapter not initialized".into()).is_retryable());
    }

    #[test]
    fn test_consumer_error_from_adapter() {
        let adapter_err = AdapterError::Connection("refused".into());
        let consumer_err: ConsumerError = adapter_err.into();
        assert!(matches!(consumer_err, ConsumerError::Adapter(_)));
    }
}
