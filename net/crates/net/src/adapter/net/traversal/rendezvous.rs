//! Hole-punch rendezvous — synchronize a simultaneous open
//! between two NATed peers via a mutually-connected relay.
//!
//! Three-message dance on [`SUBPROTOCOL_RENDEZVOUS`]:
//!
//! 1. `A → R: PunchRequest { target: B }` — A asks R to mediate.
//! 2. `R → A: PunchIntroduce { peer: B, peer_reflex, fire_at }`
//!    `R → B: PunchIntroduce { peer: A, peer_reflex, fire_at }` —
//!    R tells each side the other's reflexive address and when
//!    to fire.
//! 3. At `fire_at`, A and B each send 3 keep-alives to the other's
//!    reflex at 100 / 250 / 500 ms spacing. Whichever side sees
//!    inbound first `PunchAck`s; on ack, the Noise handshake
//!    continues on the punched path.
//! 4. If no `PunchAck` inside a 5 s window, both sides declare
//!    the punch failed and fall back to routed-handshake. No
//!    internal retry — the single-shot contract is load-bearing
//!    (plan decision 10).
//!
//! This is **stage 3** of `docs/NAT_TRAVERSAL_PLAN.md`. Stubbed
//! here as part of stage 0.
//!
//! [`SUBPROTOCOL_RENDEZVOUS`]: super::SUBPROTOCOL_RENDEZVOUS
