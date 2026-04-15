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
