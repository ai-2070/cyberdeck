//! NAT traversal — reflex-address discovery, NAT-type classification,
//! hole-punch rendezvous, and (feature-gated) UPnP / NAT-PMP / PCP
//! port mapping.
//!
//! **Framing.** NAT traversal in this codebase is a
//! **latency / throughput optimization**, not a correctness
//! requirement. Connectivity between two NATed peers already works
//! via routed handshakes + relay forwarding — every message reaches
//! its destination regardless of NAT type. What this module adds is
//! a shorter path for the cases where a direct punch is feasible,
//! reducing the per-packet relay tax and the load concentrated on
//! topological relays.
//!
//! A `NatType::Symmetric` classification or a `PunchFailed` outcome
//! is **not** a connectivity failure — it just means traffic keeps
//! riding the relay. The design doc
//! (`docs/NAT_TRAVERSAL_PLAN.md`) treats this framing as
//! load-bearing; docstrings added here must not imply that any
//! NAT-traversal primitive is required for peers behind NAT to
//! talk to each other.
//!
//! # Module layout
//!
//! - [`reflex`]      — reflex probe subprotocol handler + client.
//! - [`classify`]    — `NatType` classification state machine.
//! - [`rendezvous`]  — hole-punch coordinator subprotocol.
//! - [`config`]      — [`TraversalConfig`] (probe cadence, timeouts, …).
//! - `portmap`       — UPnP / NAT-PMP / PCP client (gated behind
//!   the `port-mapping` cargo feature; lands in stage 4 of the plan).
//!
//! # Staging
//!
//! Implemented incrementally per `docs/NAT_TRAVERSAL_PLAN.md`:
//!
//! | Stage | Surface                                    | Status            |
//! |-------|--------------------------------------------|-------------------|
//! | 0     | Module scaffolding + feature gate          | **done**          |
//! | 1     | Reflex probe subprotocol                   | **done**          |
//! | 2     | NAT type classification + `reflex_addr`    | **done**          |
//! | 3     | Hole-punch rendezvous (coordinator + ack + keep-alive train) | **done** |
//! | 4a    | Reflex override (config + runtime setters) | **done**          |
//! | 4b    | UPnP / NAT-PMP / PCP port-mapping client   | deferred (needs `igd-next` + `rust-natpmp` deps + real-router testing; the `set_reflex_override` runtime setter in stage 4a is the hook point) |
//! | 5     | SDK + NAPI + PyO3 + Go binding surface     | **done**          |
//!
//! Every stage is independently shippable. Earlier stages provide
//! observability (`nat_type`, `reflex_addr`) without the later
//! stages having landed; later stages lift performance without
//! changing the correctness contract.

pub mod classify;
pub mod config;
pub mod reflex;
pub mod rendezvous;

// Re-exports for the stable sub-module surface. Kept narrow on
// purpose — each sub-module owns the bulk of its public types
// and users import them from their origin rather than the root.
pub use config::TraversalConfig;

// =========================================================================
// Traversal stats
// =========================================================================

use std::sync::atomic::{AtomicU64, Ordering};

/// Counters tracking traversal decisions + outcomes. Exposed via
/// [`crate::adapter::net::MeshNode::traversal_stats`]. Every
/// counter is monotonic; resetting isn't supported because the
/// values are only meaningful cumulatively.
///
/// The three counters partition all `connect_direct` outcomes:
///
/// - **`punches_attempted`** — the pair-type matrix decided to
///   attempt a hole-punch. Increments whether the punch
///   eventually succeeds or fails.
/// - **`relay_fallbacks`** — connection ended up on the routed-
///   handshake path: either because the pair-type matrix skipped
///   punching, or because the punch attempt failed and fell back.
///   Every `connect_direct` that doesn't establish a directly-
///   punched session increments this counter.
/// - **`punches_succeeded`** — the punch completed within the
///   deadline and produced a direct session. Always `≤
///   punches_attempted`; the difference is the punch-failure
///   rate.
///
/// Read via [`TraversalStats::snapshot`] for a consistent
/// `(attempted, succeeded, fallbacks)` tuple.
#[derive(Debug, Default)]
pub struct TraversalStats {
    punches_attempted: AtomicU64,
    punches_succeeded: AtomicU64,
    relay_fallbacks: AtomicU64,
}

/// Consistent point-in-time view of the three
/// [`TraversalStats`] counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraversalStatsSnapshot {
    /// Number of punches the pair-type matrix elected to attempt.
    pub punches_attempted: u64,
    /// Number of those attempts that produced a direct session.
    pub punches_succeeded: u64,
    /// Number of `connect_direct` calls that ended on the routed-
    /// handshake path — matrix-skipped + punch-failed.
    pub relay_fallbacks: u64,
}

impl TraversalStats {
    /// Construct a zeroed stats block. Identical to
    /// `Default::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read all three counters. Reads are `Relaxed` — the stats
    /// are observability, not synchronization primitives, and
    /// cross-counter skew at observation time is meaningless
    /// (none of the three counters imply anything about the
    /// others' current value).
    pub fn snapshot(&self) -> TraversalStatsSnapshot {
        TraversalStatsSnapshot {
            punches_attempted: self.punches_attempted.load(Ordering::Relaxed),
            punches_succeeded: self.punches_succeeded.load(Ordering::Relaxed),
            relay_fallbacks: self.relay_fallbacks.load(Ordering::Relaxed),
        }
    }

    /// Bump `punches_attempted`. Called when the pair-type matrix
    /// elects to try a hole-punch, regardless of outcome.
    pub(crate) fn record_punch_attempt(&self) {
        self.punches_attempted.fetch_add(1, Ordering::Relaxed);
    }

    /// Bump `punches_succeeded`. Called when a punch attempt
    /// completes with a direct session.
    pub(crate) fn record_punch_success(&self) {
        self.punches_succeeded.fetch_add(1, Ordering::Relaxed);
    }

    /// Bump `relay_fallbacks`. Called when `connect_direct`
    /// resolves on the routed-handshake path — matrix-skipped or
    /// punch-failed.
    pub(crate) fn record_relay_fallback(&self) {
        self.relay_fallbacks.fetch_add(1, Ordering::Relaxed);
    }
}

// =========================================================================
// Error surface
// =========================================================================

/// Typed failures from the NAT-traversal subsystem. Matches the
/// vocabulary locked in `docs/NAT_TRAVERSAL_PLAN.md` stage 5 — each
/// variant maps to a stable `kind` string the SDK bindings expose
/// to callers.
///
/// **Framing reminder.** Every variant here describes the failure
/// of an *optimization*, not a connectivity failure. A caller that
/// receives `TraversalError` can always proceed via routed-handshake
/// — the traversal path just didn't pan out.
#[derive(Debug, thiserror::Error)]
pub enum TraversalError {
    /// Reflex probe didn't complete in time. The requester gave
    /// up after [`TraversalConfig::reflex_timeout`] without
    /// observing a response.
    #[error("reflex-timeout")]
    ReflexTimeout,

    /// The named peer is not currently reachable from this node
    /// (no session, no cached addr). Rendezvous / reflex paths
    /// need at least a direct or relayed path to the peer; if
    /// none exists, the optimization can't run.
    #[error("peer-not-reachable")]
    PeerNotReachable,

    /// Transport-level failure while sending the probe / punch
    /// traffic (socket error, partition filter, etc.).
    #[error("transport: {0}")]
    Transport(String),

    // Reserved for stages 3–5. Left defined here so downstream
    // stages can add variants without shifting the public enum
    // discriminants.
    /// Rendezvous coordinator couldn't find a mutually-connected
    /// relay-capable peer to introduce the pair.
    #[error("rendezvous-no-relay")]
    RendezvousNoRelay,

    /// Rendezvous coordinator refused to coordinate (rate-limit
    /// / unknown target / policy reject).
    #[error("rendezvous-rejected: {0}")]
    RendezvousRejected(String),

    /// Keep-alive train didn't establish a punched path within
    /// the [`TraversalConfig::punch_deadline`] window.
    #[error("punch-failed")]
    PunchFailed,

    /// UPnP / NAT-PMP / PCP all failed to install a port mapping.
    /// Only surfaces when the `port-mapping` feature is enabled
    /// AND `MeshBuilder::try_port_mapping(true)` opted in.
    #[error("port-map-unavailable")]
    PortMapUnavailable,

    /// Peer doesn't advertise the NAT-traversal capability tag
    /// (compiled without `nat-traversal`, or opted out via
    /// `MeshBuilder::disable_nat_traversal`).
    #[error("unsupported")]
    Unsupported,
}

impl TraversalError {
    /// Stable machine-readable kind string used by the SDK
    /// bindings to expose typed catches. Never localized; never
    /// changed once a variant has shipped.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ReflexTimeout => "reflex-timeout",
            Self::PeerNotReachable => "peer-not-reachable",
            Self::Transport(_) => "transport",
            Self::RendezvousNoRelay => "rendezvous-no-relay",
            Self::RendezvousRejected(_) => "rendezvous-rejected",
            Self::PunchFailed => "punch-failed",
            Self::PortMapUnavailable => "port-map-unavailable",
            Self::Unsupported => "unsupported",
        }
    }
}

// =========================================================================
// Subprotocol ID assignment
// =========================================================================
//
// The `0x0D00` block is the first unused range after the existing
// subprotocol allocations (`0x0400` causal, `0x0500` migration,
// `0x0A00` channel membership, `0x0B00` stream window,
// `0x0C00` capability announcement). Reserved for NAT-traversal
// primitives; ids consumed here:
//
//   0x0D00 — reflex probe (stage 1)
//   0x0D01 — rendezvous (stage 3)
//   0x0D02 — reserved for port-mapping metadata (stage 4, optional)
//
// Future traversal primitives take `0x0D0x` ids sequentially.

/// Subprotocol ID for the reflex-probe request/response exchange.
///
/// Any peer that receives a `SUBPROTOCOL_REFLEX` request replies with
/// the source `ip:port` it observed on the request's UDP envelope.
/// Two or more probes to different peers are sufficient to detect
/// symmetric NAT (the observed source port differs per destination).
///
/// See [`reflex`] for the handler / client implementation.
pub const SUBPROTOCOL_REFLEX: u16 = 0x0D00;

/// Subprotocol ID for hole-punch rendezvous coordination.
///
/// Carries the three-message dance (`PunchRequest` →
/// `PunchIntroduce` × 2 → `PunchAck`) that synchronizes a
/// simultaneous open between two NATed peers, mediated by a
/// mutually-connected relay.
///
/// See [`rendezvous`] for the state machine.
pub const SUBPROTOCOL_RENDEZVOUS: u16 = 0x0D01;
