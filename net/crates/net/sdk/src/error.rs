//! Unified SDK error type.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("node has been shut down")]
    Shutdown,

    #[error("ingestion failed: {0}")]
    Ingestion(String),

    #[error("poll failed: {0}")]
    Poll(String),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("mesh transport not available")]
    NoMesh,

    /// Stream's per-stream in-flight window is full. The caller's events
    /// were NOT sent — daemons decide whether to drop, retry, or buffer
    /// at the app layer. See `Mesh::send_with_retry` / `send_blocking`
    /// for the two built-in policies.
    #[error("stream backpressure: queue full")]
    Backpressure,

    /// Stream's peer session is gone (peer disconnected, never
    /// connected, or the stream was closed).
    #[error("stream not connected")]
    NotConnected,

    /// A publisher's `Ack` rejected a Subscribe / Unsubscribe
    /// request. `None` means the rejection arrived without a
    /// structured reason.
    #[error("channel membership rejected: {0:?}")]
    ChannelRejected(Option<net::adapter::net::AckReason>),
}

#[cfg(feature = "net")]
impl From<net::adapter::net::StreamError> for SdkError {
    fn from(e: net::adapter::net::StreamError) -> Self {
        use net::adapter::net::StreamError;
        match e {
            StreamError::Backpressure => SdkError::Backpressure,
            StreamError::NotConnected => SdkError::NotConnected,
            StreamError::Transport(msg) => SdkError::Adapter(msg),
        }
    }
}

impl From<net::error::IngestionError> for SdkError {
    fn from(e: net::error::IngestionError) -> Self {
        SdkError::Ingestion(e.to_string())
    }
}

impl From<net::error::ConsumerError> for SdkError {
    fn from(e: net::error::ConsumerError) -> Self {
        SdkError::Poll(e.to_string())
    }
}

impl From<net::error::AdapterError> for SdkError {
    fn from(e: net::error::AdapterError) -> Self {
        SdkError::Adapter(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;
