//! Error surface for the `groups` module.
//!
//! Wraps the core `GroupError` and adds SDK-level variants
//! (`NotReady`, `FactoryNotFound`) that the wrappers surface
//! before touching the core. TS / Python / Go bindings dispatch
//! on these via the same `daemon: group: <kind>[: detail]` prefix
//! pattern used for migration errors.

use thiserror::Error;

use ::net::adapter::net::compute::GroupError as CoreGroupError;

use crate::compute::DaemonError;

/// Errors returned by the SDK's group wrappers.
#[derive(Debug, Error)]
pub enum GroupError {
    /// The caller's `DaemonRuntime` has not yet reached `Ready`.
    /// Start the runtime before constructing any group.
    #[error("daemon runtime is not ready — call DaemonRuntime::start() first")]
    NotReady,

    /// The caller referenced a kind that was never registered
    /// via `DaemonRuntime::register_factory`. Register the kind
    /// before spawning a group of that kind.
    #[error("no factory registered for kind '{0}'")]
    FactoryNotFound(String),

    /// Pass-through for errors surfaced by the core group layer
    /// (`NoHealthyMember`, `PlacementFailed`, `RegistryFailed`,
    /// `InvalidConfig`).
    #[error(transparent)]
    Core(#[from] CoreGroupError),

    /// Pass-through for daemon-level errors surfaced during
    /// member spawn (registry collision, host construction).
    #[error("daemon error: {0}")]
    Daemon(#[from] DaemonError),
}
