//! NAT-type classification — collapse observed reflex addresses
//! into the `Open | Cone | Symmetric | Unknown` wire taxonomy.
//!
//! The richer five-way `NatType` lives on
//! `adapter::net::behavior::metadata::NatType` for internal
//! reasoning; on the wire (capability-announcement tags) we only
//! distinguish the four outcomes that matter for punch decisions:
//!
//! - **Open** — reflexive address equals bind address, or a
//!   port-mapping is installed.
//! - **Cone** — reflexive port is consistent across distinct
//!   destinations (punching is reliable).
//! - **Symmetric** — reflexive port differs per destination
//!   (punching is not reliable; cone × symmetric gets one shot
//!   per decision 8 in the plan).
//! - **Unknown** — fewer than two probes, or classification
//!   hasn't run yet.
//!
//! This module owns the pure-logic FSM. Wiring the FSM to the
//! reflex probe + capability broadcast lives in
//! [`super::super::mesh`]; the split keeps classification
//! testable without spinning up a real mesh.

use std::net::SocketAddr;

/// Wire-form NAT classification. Matches the `nat:*` capability
/// tag vocabulary (`nat:open` | `nat:cone` | `nat:symmetric` |
/// `nat:unknown`) emitted by the capability broadcast after
/// classification.
///
/// Internal code that wants the richer five-way enum
/// (`FullCone / RestrictedCone / PortRestricted / Symmetric / None`)
/// should use [`adapter::net::behavior::metadata::NatType`]
/// directly. This type is the *publishable* summary that fits
/// on one tag and drives the connect-time pair-type matrix in
/// `docs/NAT_TRAVERSAL_PLAN.md` §8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NatClass {
    /// Fewer than two probes completed, or classification hasn't
    /// run yet. Treated as "attempt direct, fall back on first
    /// failure" by the connect-time pair-type matrix — never
    /// treated as "don't attempt."
    ///
    /// Listed first so `NatClass::default()` via `AtomicU8::new(0)`
    /// round-trips to `Unknown` — the pre-classification state.
    Unknown = 0,
    /// Reflex address equals bind address (no NAT) or a
    /// port-mapping is installed (stage 4). Direct connect from
    /// any peer works without punching.
    Open = 1,
    /// Reflex port stable across distinct destinations. Symmetric
    /// about the *address* but not the port — punching succeeds
    /// with high probability against any peer not also symmetric.
    Cone = 2,
    /// Reflex port varies per destination. Cannot reliably
    /// hole-punch; falls back to routed-handshake on any attempt.
    Symmetric = 3,
}

impl NatClass {
    /// The `nat:*` capability tag corresponding to this
    /// classification. Stable string; never localized. The tag is
    /// the source of truth when a peer reads another peer's NAT
    /// type from its capability announcement.
    pub fn tag(&self) -> &'static str {
        match self {
            NatClass::Open => "nat:open",
            NatClass::Cone => "nat:cone",
            NatClass::Symmetric => "nat:symmetric",
            NatClass::Unknown => "nat:unknown",
        }
    }

    /// Parse a `nat:*` tag back into a [`NatClass`]. Returns
    /// `None` for any tag outside the reserved `nat:*` vocabulary.
    /// The capability-filter path uses this to decode peer NAT
    /// classifications without a separate wire field.
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "nat:open" => Some(NatClass::Open),
            "nat:cone" => Some(NatClass::Cone),
            "nat:symmetric" => Some(NatClass::Symmetric),
            "nat:unknown" => Some(NatClass::Unknown),
            _ => None,
        }
    }

    /// Encode as a `u8` suitable for `AtomicU8` storage. `MeshNode`
    /// holds the current classification in an atomic so the
    /// announce-capabilities path can read it without locking.
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::as_u8`]. Unknown values collapse to
    /// `Unknown` rather than panicking — the atomic is `MeshNode`-
    /// internal state, but the defensive fallback lets a future
    /// stage add a variant without silently corrupting stored state.
    #[inline]
    pub fn from_u8(raw: u8) -> Self {
        match raw {
            1 => NatClass::Open,
            2 => NatClass::Cone,
            3 => NatClass::Symmetric,
            _ => NatClass::Unknown,
        }
    }
}

/// Decision returned by the pair-type matrix (plan §3 "Connect-
/// time pair-type matrix"). Drives `connect_direct`'s choice of
/// whether to attempt a punch, route through the relay, or skip
/// the punch entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairAction {
    /// Connect directly to the peer without a hole-punch attempt.
    /// Used when at least one side is Open — NAT traversal is
    /// unnecessary. Stats: no counters bumped until
    /// `connect_direct` resolves; a successful direct connect
    /// isn't a "fallback" because no punch was offered in the
    /// first place.
    Direct,
    /// Fire exactly one rendezvous-coordinated punch (plan
    /// decision 8 — no retry on failure). Stats:
    /// `punches_attempted` bumps when this action is selected;
    /// `punches_succeeded` or `relay_fallbacks` bumps on outcome.
    SinglePunch,
    /// Skip the punch and connect via the routed-handshake path
    /// only. Used for Symmetric × Symmetric (direct punch
    /// infeasible) and Symmetric × Unknown (not worth the
    /// coordinator round-trip when one side can't hole-punch
    /// reliably). Stats: `relay_fallbacks` bumps.
    SkipPunch,
}

/// Decide what `connect_direct` should do given the local and
/// remote NAT classifications. Pure function — no I/O, no state.
///
/// Matrix (plan §3 "Connect-time pair-type matrix"):
///
/// | Local → | Remote → `Open`      | Remote → `Cone`     | Remote → `Symmetric`  | Remote → `Unknown`   |
/// |---------|----------------------|---------------------|-----------------------|----------------------|
/// | `Open`       | `Direct`         | `Direct`            | `SinglePunch`         | `Direct`             |
/// | `Cone`       | `Direct`         | `SinglePunch`       | `SinglePunch`         | `SinglePunch`        |
/// | `Symmetric`  | `SinglePunch`    | `SinglePunch`       | `SkipPunch`           | `SkipPunch`          |
/// | `Unknown`    | `Direct`         | `SinglePunch`       | `SkipPunch`           | `Direct`             |
///
/// `Unknown × Unknown` goes `Direct` (attempt direct, fall back
/// on first failure) — plan decision 8's "never treat Unknown as
/// do-not-attempt" rule. `Symmetric × Unknown` goes `SkipPunch`
/// because the Symmetric side can't reliably punch regardless of
/// the other end's type.
pub fn pair_action(local: NatClass, remote: NatClass) -> PairAction {
    use NatClass::*;
    match (local, remote) {
        // Open never needs a punch — it's directly reachable.
        (Open, _) | (_, Open) => PairAction::Direct,

        // Symmetric × Symmetric: punch is not reliable either way.
        (Symmetric, Symmetric) => PairAction::SkipPunch,

        // Symmetric × Unknown (or vice versa): don't waste the
        // coordinator round-trip; the symmetric side can't punch.
        (Symmetric, Unknown) | (Unknown, Symmetric) => PairAction::SkipPunch,

        // Cone × anything else (Cone, Symmetric, Unknown): worth
        // a single-shot punch. Cone × Open is handled above.
        (Cone, _) | (_, Cone) => PairAction::SinglePunch,

        // Unknown × Unknown: attempt direct, fall back on failure.
        (Unknown, Unknown) => PairAction::Direct,
    }
}

/// NAT classification state machine.
///
/// Accumulates per-peer reflex observations and produces a
/// [`NatClass`] once two or more probes have completed. Pure
/// logic — no I/O, no timing. The caller owns the probe-firing
/// and feeds results in via [`ClassifyFsm::observe`].
///
/// # Classification rule
///
/// 1. If `bind_addr` equals any observed reflex → `Open`. A
///    node whose reflex equals its bind address isn't behind a
///    NAT from the perspective of that peer; port mappings
///    installed via stage 4 produce the same shape.
/// 2. Else if all observed reflex ports match → `Cone`. The
///    symmetric NAT detection test: two observations to different
///    destinations yielding the same source port means the NAT
///    is *not* symmetric-about-port.
/// 3. Else → `Symmetric`. Port varies per destination; direct
///    punching is not reliable.
/// 4. Fewer than two probes → `Unknown`. Never treated as a
///    connectivity failure; the connect-time pair-type matrix
///    defaults to "attempt direct, fall back on first failure."
///
/// # Multiple observations from the same peer
///
/// The FSM keeps the *latest* observation per peer so a
/// mid-session NAT rebind shows up on reclassification. Earlier
/// observations from the same peer are silently replaced.
#[derive(Debug, Clone, Default)]
pub struct ClassifyFsm {
    /// Observations indexed by `(peer_node_id, reflex)`. Kept as
    /// a Vec rather than a HashMap because the expected N is
    /// small (2–4 anchor peers) and linear scan beats hashing at
    /// this size.
    probes: Vec<(u64, SocketAddr)>,
}

impl ClassifyFsm {
    /// Create an empty FSM. Identical to `Default::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a reflex observation from `peer`. If `peer` has
    /// already contributed, the earlier observation is replaced —
    /// only the latest view from each peer counts.
    pub fn observe(&mut self, peer: u64, reflex: SocketAddr) {
        if let Some(slot) = self.probes.iter_mut().find(|(p, _)| *p == peer) {
            slot.1 = reflex;
        } else {
            self.probes.push((peer, reflex));
        }
    }

    /// Number of distinct peers that have contributed an
    /// observation. Useful for tests and for re-classification
    /// triggers that need to check "did we get at least 2 probes?"
    pub fn observation_count(&self) -> usize {
        self.probes.len()
    }

    /// Clear all accumulated observations. Called at the start of
    /// a reclassification sweep so stale entries from a previous
    /// NAT state don't bias the new result.
    pub fn clear(&mut self) {
        self.probes.clear();
    }

    /// Produce the collapsed [`NatClass`] given the current
    /// observations and the node's own bind address.
    ///
    /// `bind_addr` is the address the mesh socket bound to — e.g.
    /// `0.0.0.0:9001` resolved to an interface address. A reflex
    /// observation matching this address means "we're not behind
    /// a NAT" (or a mapping is installed — same observable).
    pub fn classify(&self, bind_addr: SocketAddr) -> NatClass {
        if self.probes.len() < 2 {
            return NatClass::Unknown;
        }

        // Open: any reflex equals bind. A port-mapping installed
        // via stage 4 produces the same shape (bind == external),
        // so this check naturally subsumes that case.
        if self
            .probes
            .iter()
            .any(|(_, reflex)| reflex.port() == bind_addr.port() && reflex.ip() == bind_addr.ip())
        {
            return NatClass::Open;
        }

        // Symmetric vs. Cone: does the reflex port vary per
        // destination? If every observation agrees on port, the
        // NAT is cone-typed (full cone / restricted cone /
        // port-restricted cone all produce stable outbound ports
        // per source). If ports differ, we're symmetric.
        let first_port = self.probes[0].1.port();
        let port_stable = self.probes.iter().all(|(_, reflex)| reflex.port() == first_port);
        if port_stable {
            NatClass::Cone
        } else {
            NatClass::Symmetric
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sa(addr: &str) -> SocketAddr {
        addr.parse().unwrap()
    }

    #[test]
    fn empty_classifies_as_unknown() {
        let fsm = ClassifyFsm::new();
        assert_eq!(fsm.classify(sa("10.0.0.1:9001")), NatClass::Unknown);
    }

    #[test]
    fn one_probe_classifies_as_unknown() {
        // Even if that single probe matches bind — we still need
        // two data points to rule out the "maybe symmetric"
        // hypothesis.
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, sa("10.0.0.1:9001"));
        assert_eq!(fsm.classify(sa("10.0.0.1:9001")), NatClass::Unknown);
    }

    #[test]
    fn reflex_matching_bind_is_open() {
        let bind = sa("192.0.2.1:9001");
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, bind);
        fsm.observe(2, bind);
        assert_eq!(fsm.classify(bind), NatClass::Open);
    }

    #[test]
    fn stable_port_across_peers_is_cone() {
        // Two peers, same external port, different external IPs.
        // A cone NAT keeps outbound port stable per source —
        // this is the defining signature.
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, sa("198.51.100.5:54321"));
        fsm.observe(2, sa("198.51.100.5:54321"));
        assert_eq!(fsm.classify(sa("192.0.2.1:9001")), NatClass::Cone);
    }

    #[test]
    fn varying_port_is_symmetric() {
        // Two peers, different external ports — the symmetric-NAT
        // signature.
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, sa("198.51.100.5:54321"));
        fsm.observe(2, sa("198.51.100.5:54322"));
        assert_eq!(fsm.classify(sa("192.0.2.1:9001")), NatClass::Symmetric);
    }

    #[test]
    fn later_observation_from_same_peer_replaces_earlier() {
        // A reclassification round that re-probes the same peer
        // should see the new observation, not the stale one.
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, sa("198.51.100.5:54321"));
        fsm.observe(2, sa("198.51.100.5:54321"));
        // Peer 1's NAT rebinds to a different port.
        fsm.observe(1, sa("198.51.100.5:54322"));
        assert_eq!(fsm.observation_count(), 2);
        assert_eq!(fsm.classify(sa("192.0.2.1:9001")), NatClass::Symmetric);
    }

    #[test]
    fn clear_resets_to_unknown() {
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, sa("198.51.100.5:54321"));
        fsm.observe(2, sa("198.51.100.5:54321"));
        fsm.clear();
        assert_eq!(fsm.observation_count(), 0);
        assert_eq!(fsm.classify(sa("192.0.2.1:9001")), NatClass::Unknown);
    }

    #[test]
    fn open_beats_cone_when_bind_equals_one_reflex() {
        // Mixed signal: one peer sees bind addr (we're directly
        // reachable from it), another sees a NAT rewrite. The
        // classifier prefers `Open` — being reachable to at
        // least one peer without a NAT is the more useful
        // signal for placement.
        let bind = sa("192.0.2.1:9001");
        let mut fsm = ClassifyFsm::new();
        fsm.observe(1, bind);
        fsm.observe(2, sa("198.51.100.5:54321"));
        assert_eq!(fsm.classify(bind), NatClass::Open);
    }

    #[test]
    fn tag_roundtrip() {
        // Every wire tag round-trips through the NatClass <-> tag
        // boundary. Regressions here would break peer-side NAT
        // discrimination on the capability-broadcast path.
        for variant in [
            NatClass::Open,
            NatClass::Cone,
            NatClass::Symmetric,
            NatClass::Unknown,
        ] {
            let tag = variant.tag();
            assert_eq!(NatClass::from_tag(tag), Some(variant));
        }
    }

    #[test]
    fn unknown_tag_rejects() {
        assert_eq!(NatClass::from_tag("gpu"), None);
        assert_eq!(NatClass::from_tag("nat:"), None);
        assert_eq!(NatClass::from_tag("nat:weird"), None);
        assert_eq!(NatClass::from_tag(""), None);
    }

    #[test]
    fn u8_roundtrip() {
        // Atomic-storage form. `Unknown = 0` so a freshly-zeroed
        // `AtomicU8::new(0)` reads as `Unknown` — the pre-
        // classification state the `MeshNode` starts in.
        assert_eq!(NatClass::Unknown.as_u8(), 0);
        for variant in [
            NatClass::Unknown,
            NatClass::Open,
            NatClass::Cone,
            NatClass::Symmetric,
        ] {
            assert_eq!(NatClass::from_u8(variant.as_u8()), variant);
        }
    }

    #[test]
    fn from_u8_unknown_collapses_to_unknown() {
        // Out-of-range bytes never panic. A future variant shouldn't
        // be able to scribble corrupted state into the atomic and
        // read back garbage elsewhere.
        assert_eq!(NatClass::from_u8(4), NatClass::Unknown);
        assert_eq!(NatClass::from_u8(255), NatClass::Unknown);
    }

    #[test]
    fn pair_action_open_always_direct() {
        // Open on either side = directly reachable; no punch
        // needed. Stable across every remote classification,
        // including Unknown.
        for remote in [
            NatClass::Open,
            NatClass::Cone,
            NatClass::Symmetric,
            NatClass::Unknown,
        ] {
            assert_eq!(
                pair_action(NatClass::Open, remote),
                PairAction::Direct,
                "Open × {remote:?} should be Direct",
            );
            assert_eq!(
                pair_action(remote, NatClass::Open),
                PairAction::Direct,
                "{remote:?} × Open should be Direct",
            );
        }
    }

    #[test]
    fn pair_action_symmetric_symmetric_skips_punch() {
        // Plan decision: neither side can reliably hole-punch,
        // so skip the coordinator round-trip entirely.
        assert_eq!(
            pair_action(NatClass::Symmetric, NatClass::Symmetric),
            PairAction::SkipPunch,
        );
    }

    #[test]
    fn pair_action_cone_cone_single_punch() {
        // The canonical "worth a punch" pair: both sides cone-
        // typed, single-shot attempt.
        assert_eq!(
            pair_action(NatClass::Cone, NatClass::Cone),
            PairAction::SinglePunch,
        );
    }

    #[test]
    fn pair_action_symmetric_cone_attempts_one() {
        // Plan decision 8: symmetric × cone gets one shot.
        assert_eq!(
            pair_action(NatClass::Symmetric, NatClass::Cone),
            PairAction::SinglePunch,
        );
        assert_eq!(
            pair_action(NatClass::Cone, NatClass::Symmetric),
            PairAction::SinglePunch,
        );
    }

    #[test]
    fn pair_action_unknown_unknown_is_direct() {
        // Unknown × Unknown: attempt direct, fall back on first
        // failure. Plan's "never treat Unknown as do-not-attempt"
        // rule.
        assert_eq!(
            pair_action(NatClass::Unknown, NatClass::Unknown),
            PairAction::Direct,
        );
    }

    #[test]
    fn pair_action_symmetric_unknown_skips_punch() {
        // Symmetric side can't punch reliably regardless of the
        // other end — skip the coordinator round-trip.
        assert_eq!(
            pair_action(NatClass::Symmetric, NatClass::Unknown),
            PairAction::SkipPunch,
        );
        assert_eq!(
            pair_action(NatClass::Unknown, NatClass::Symmetric),
            PairAction::SkipPunch,
        );
    }

    #[test]
    fn pair_action_cone_unknown_attempts_one() {
        // Cone × Unknown: worth a punch — cone side can
        // definitely receive if the other side reaches it, and
        // Unknown isn't "can't punch."
        assert_eq!(
            pair_action(NatClass::Cone, NatClass::Unknown),
            PairAction::SinglePunch,
        );
        assert_eq!(
            pair_action(NatClass::Unknown, NatClass::Cone),
            PairAction::SinglePunch,
        );
    }
}
