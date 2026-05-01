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
    /// structured reason. Gated behind `net` because
    /// `AckReason` lives in the network-transport surface;
    /// non-`net` SDK builds (e.g. redis-only consumer helpers)
    /// don't compile this variant.
    #[cfg(feature = "net")]
    #[error("channel membership rejected: {0:?}")]
    ChannelRejected(Option<net::adapter::net::AckReason>),

    /// NAT-traversal failure — a reflex probe, hole-punch, or
    /// port-mapping path couldn't complete. Always represents a
    /// missed *optimization*, never a connectivity failure:
    /// every NATed peer remains reachable through the routed-
    /// handshake path. `kind` is a stable discriminator
    /// matching the `nat-traversal` crate's error vocabulary
    /// (`reflex-timeout` / `peer-not-reachable` / `transport` /
    /// `rendezvous-no-relay` / `rendezvous-rejected` /
    /// `punch-failed` / `port-map-unavailable` / `unsupported`).
    #[cfg(feature = "nat-traversal")]
    #[error("traversal {kind}: {message}")]
    Traversal {
        /// Stable machine-readable discriminator — same value as
        /// `TraversalError::kind()`. Never localized; never
        /// changed once a variant has shipped.
        kind: &'static str,
        /// Human-readable detail (e.g. the underlying socket
        /// error on a `transport` failure). Empty for kinds
        /// that have no variable detail.
        message: String,
    },
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

#[cfg(feature = "nat-traversal")]
impl From<net::adapter::net::traversal::TraversalError> for SdkError {
    fn from(e: net::adapter::net::traversal::TraversalError) -> Self {
        SdkError::Traversal {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;
