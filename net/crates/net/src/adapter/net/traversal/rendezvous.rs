//! Hole-punch rendezvous — synchronize a simultaneous open
//! between two NATed peers via a mutually-connected relay.
//!
//! Three-message dance on [`SUBPROTOCOL_RENDEZVOUS`]:
//!
//! 1. `A → R: PunchRequest { target: B, self_reflex }` — A asks R
//!    to mediate a punch to B and hands R its current best guess
//!    of its own public `SocketAddr`.
//! 2. `R → A: PunchIntroduce { peer: B, peer_reflex, fire_at }`
//!    `R → B: PunchIntroduce { peer: A, peer_reflex, fire_at }` —
//!    R tells each side the other's reflexive address and when to
//!    fire.
//! 3. At `fire_at`, A and B each send keep-alives to the other's
//!    reflex. Whichever side observes inbound from the punched
//!    path first sends `PunchAck` via the routed-handshake path
//!    (not the punched one — we don't yet know the punched path
//!    is reliable) and begins the Noise handshake on the punched
//!    socket.
//! 4. If no `PunchAck` inside a 5 s window, both sides declare
//!    the punch failed and fall back to routed-handshake. No
//!    internal retry — the single-shot contract is load-bearing
//!    (plan decision 10).
//!
//! # Wire format
//!
//! Each message is carried inside the existing event-frame wrapper
//! on [`SUBPROTOCOL_RENDEZVOUS`]:
//!
//! ```text
//! ┌──────────┬─────────────────────────────────────┐
//! │ kind (1) │ body (N)                            │
//! └──────────┴─────────────────────────────────────┘
//! ```
//!
//! - `kind` is the discriminator: `0x01 = PunchRequest`,
//!   `0x02 = PunchIntroduce`, `0x03 = PunchAck`.
//! - Addresses are encoded as `family(1) | addr(16) | port(2)` —
//!   the same 19-byte shape used by the reflex subprotocol, so a
//!   future migration can share the codec without a wire bump.
//! - Multi-byte integers are big-endian.
//!
//! ## PunchRequest body (12 + 19 = 31 bytes)
//!
//! ```text
//! ┌──────────────────┬─────────────────────────────┐
//! │ target_node (8)  │ self_reflex (19)            │
//! └──────────────────┴─────────────────────────────┘
//! ```
//!
//! `target_node` is the `node_id` of the peer the requester wants
//! to punch to. `self_reflex` is the requester's current best
//! guess of its own public `SocketAddr` — R uses this to stamp
//! into B's `PunchIntroduce` if R doesn't have a fresher reflex
//! in its capability cache.
//!
//! ## PunchIntroduce body (8 + 19 + 8 = 35 bytes)
//!
//! ```text
//! ┌────────────┬─────────────────────────┬─────────────────┐
//! │ peer (8)   │ peer_reflex (19)        │ fire_at_ms (8)  │
//! └────────────┴─────────────────────────┴─────────────────┘
//! ```
//!
//! `peer` is the other endpoint's `node_id`. `fire_at_ms` is Unix
//! epoch milliseconds — the synchronized punch-time both sides
//! use to schedule their keep-alives.
//!
//! ## PunchAck body (8 + 4 = 12 bytes)
//!
//! ```text
//! ┌────────────┬───────────────┐
//! │ peer (8)   │ punch_id (4)  │
//! └────────────┴───────────────┘
//! ```
//!
//! `punch_id` correlates an `Ack` back to the originating
//! `PunchRequest`. The plan reserves this as a u32; this module
//! preserves it on the wire even though stage-3a wiring doesn't
//! yet generate punch ids — stages 3b/3c do.
//!
//! # Framing (not correctness)
//!
//! Rendezvous is an optimization, not a connectivity requirement.
//! A failed punch or a rejected `PunchRequest` doesn't prevent two
//! peers from exchanging traffic — they still have the routed-
//! handshake path. Docstrings added here must not imply the
//! rendezvous path is required for NATed peers to communicate.
//!
//! [`SUBPROTOCOL_RENDEZVOUS`]: super::SUBPROTOCOL_RENDEZVOUS

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use bytes::{BufMut, Bytes, BytesMut};

/// Length of the address-encoding block: `family(1) + addr(16) +
/// port(2) = 19 bytes`. Identical shape to
/// [`super::reflex::RESPONSE_LEN`] on purpose so the codecs can
/// share logic in a future refactor without a wire bump.
const ADDR_LEN: usize = 19;

const FAMILY_V4: u8 = 4;
const FAMILY_V6: u8 = 6;

const KIND_PUNCH_REQUEST: u8 = 0x01;
const KIND_PUNCH_INTRODUCE: u8 = 0x02;
const KIND_PUNCH_ACK: u8 = 0x03;

/// Total on-wire size of a `PunchRequest` payload.
/// `kind(1) + target_node(8) + self_reflex(19) = 28`.
pub const PUNCH_REQUEST_LEN: usize = 1 + 8 + ADDR_LEN;

/// Total on-wire size of a `PunchIntroduce` payload.
/// `kind(1) + peer(8) + peer_reflex(19) + fire_at_ms(8) = 36`.
pub const PUNCH_INTRODUCE_LEN: usize = 1 + 8 + ADDR_LEN + 8;

/// Total on-wire size of a `PunchAck` payload.
/// `kind(1) + peer(8) + punch_id(4) = 13`.
pub const PUNCH_ACK_LEN: usize = 1 + 8 + 4;

/// A `PunchRequest` payload — A → R ("please mediate a punch to
/// `target`").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PunchRequest {
    /// The peer `A` wants to punch to. R looks up the peer's
    /// `reflex_addr` from its capability cache; if missing, R
    /// rejects the request with a typed error and A falls back to
    /// routed-handshake (plan §3 coordinator step 1).
    pub target: u64,
    /// A's current best guess of its own public `SocketAddr`.
    /// R forwards this into B's `PunchIntroduce` — it's an
    /// optimization, not load-bearing: R may override with a
    /// fresher reflex observation from its own cache.
    pub self_reflex: SocketAddr,
}

/// A `PunchIntroduce` payload — R → A and R → B ("here's the
/// other endpoint's reflex and when to fire").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PunchIntroduce {
    /// The other endpoint's `node_id`.
    pub peer: u64,
    /// The other endpoint's public `SocketAddr` — the address
    /// this endpoint's keep-alive packets should target.
    pub peer_reflex: SocketAddr,
    /// Unix epoch milliseconds — the synchronized punch-time.
    /// Both endpoints subtract `now()` and schedule keep-alives
    /// relative to the resulting offset. Sub-millisecond drift
    /// between the two sides is fine — the keep-alive train
    /// spans 250 ms and a firewall state-install is faster than
    /// that.
    pub fire_at_ms: u64,
}

/// A `PunchAck` payload — the side that first observed inbound
/// traffic on the punched path tells the other side the punch
/// succeeded. Sent via the routed-handshake path, not the punched
/// one — we don't yet know the punched path is symmetric-reliable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PunchAck {
    /// The other endpoint's `node_id`.
    pub peer: u64,
    /// Correlation token echoed from the originating
    /// `PunchRequest`. Stage-3b wiring generates these; stage 3a
    /// preserves them on the wire.
    pub punch_id: u32,
}

/// Decoded rendezvous subprotocol message. The three variants
/// correspond to the three event-frame payload shapes described in
/// the module docs. Use [`decode`] to obtain one from raw bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendezvousMsg {
    /// A → R: please mediate a punch to `target`.
    PunchRequest(PunchRequest),
    /// R → {A, B}: here's the peer's reflex + the fire time.
    PunchIntroduce(PunchIntroduce),
    /// {A, B} → {A, B}: punch succeeded on my side (sent via
    /// routed-handshake, not the punched path).
    PunchAck(PunchAck),
}

impl RendezvousMsg {
    /// Encode the message as an event-body `Bytes`. Length is
    /// exactly one of [`PUNCH_REQUEST_LEN`], [`PUNCH_INTRODUCE_LEN`],
    /// or [`PUNCH_ACK_LEN`] depending on the variant.
    pub fn encode(&self) -> Bytes {
        match self {
            RendezvousMsg::PunchRequest(req) => encode_punch_request(req),
            RendezvousMsg::PunchIntroduce(intro) => encode_punch_introduce(intro),
            RendezvousMsg::PunchAck(ack) => encode_punch_ack(ack),
        }
    }
}

fn encode_addr(buf: &mut BytesMut, addr: SocketAddr) {
    match addr {
        SocketAddr::V4(v4) => {
            buf.put_u8(FAMILY_V4);
            let mut bytes = [0u8; 16];
            bytes[..4].copy_from_slice(&v4.ip().octets());
            buf.put_slice(&bytes);
            buf.put_u16(v4.port());
        }
        SocketAddr::V6(v6) => {
            buf.put_u8(FAMILY_V6);
            buf.put_slice(&v6.ip().octets());
            buf.put_u16(v6.port());
        }
    }
}

fn decode_addr(bytes: &[u8]) -> Option<SocketAddr> {
    if bytes.len() != ADDR_LEN {
        return None;
    }
    let family = bytes[0];
    let addr_bytes: [u8; 16] = bytes[1..17].try_into().ok()?;
    let port = u16::from_be_bytes([bytes[17], bytes[18]]);
    let ip = match family {
        FAMILY_V4 => IpAddr::V4(Ipv4Addr::new(
            addr_bytes[0],
            addr_bytes[1],
            addr_bytes[2],
            addr_bytes[3],
        )),
        FAMILY_V6 => IpAddr::V6(Ipv6Addr::from(addr_bytes)),
        _ => return None,
    };
    Some(SocketAddr::new(ip, port))
}

fn encode_punch_request(req: &PunchRequest) -> Bytes {
    let mut buf = BytesMut::with_capacity(PUNCH_REQUEST_LEN);
    buf.put_u8(KIND_PUNCH_REQUEST);
    buf.put_u64(req.target);
    encode_addr(&mut buf, req.self_reflex);
    debug_assert_eq!(buf.len(), PUNCH_REQUEST_LEN);
    buf.freeze()
}

fn encode_punch_introduce(intro: &PunchIntroduce) -> Bytes {
    let mut buf = BytesMut::with_capacity(PUNCH_INTRODUCE_LEN);
    buf.put_u8(KIND_PUNCH_INTRODUCE);
    buf.put_u64(intro.peer);
    encode_addr(&mut buf, intro.peer_reflex);
    buf.put_u64(intro.fire_at_ms);
    debug_assert_eq!(buf.len(), PUNCH_INTRODUCE_LEN);
    buf.freeze()
}

fn encode_punch_ack(ack: &PunchAck) -> Bytes {
    let mut buf = BytesMut::with_capacity(PUNCH_ACK_LEN);
    buf.put_u8(KIND_PUNCH_ACK);
    buf.put_u64(ack.peer);
    buf.put_u32(ack.punch_id);
    debug_assert_eq!(buf.len(), PUNCH_ACK_LEN);
    buf.freeze()
}

/// Decode a rendezvous payload. Returns `None` on any malformed
/// input (wrong length for the claimed kind, unknown kind
/// discriminator, unknown address family byte). Callers drop
/// malformed payloads silently — the subprotocol is an
/// optimization, so a bad packet is neither fatal nor worth
/// surfacing.
pub fn decode(payload: &[u8]) -> Option<RendezvousMsg> {
    let &kind = payload.first()?;
    match kind {
        KIND_PUNCH_REQUEST => {
            if payload.len() != PUNCH_REQUEST_LEN {
                return None;
            }
            let target = u64::from_be_bytes(payload[1..9].try_into().ok()?);
            let self_reflex = decode_addr(&payload[9..28])?;
            Some(RendezvousMsg::PunchRequest(PunchRequest {
                target,
                self_reflex,
            }))
        }
        KIND_PUNCH_INTRODUCE => {
            if payload.len() != PUNCH_INTRODUCE_LEN {
                return None;
            }
            let peer = u64::from_be_bytes(payload[1..9].try_into().ok()?);
            let peer_reflex = decode_addr(&payload[9..28])?;
            let fire_at_ms = u64::from_be_bytes(payload[28..36].try_into().ok()?);
            Some(RendezvousMsg::PunchIntroduce(PunchIntroduce {
                peer,
                peer_reflex,
                fire_at_ms,
            }))
        }
        KIND_PUNCH_ACK => {
            if payload.len() != PUNCH_ACK_LEN {
                return None;
            }
            let peer = u64::from_be_bytes(payload[1..9].try_into().ok()?);
            let punch_id = u32::from_be_bytes(payload[9..13].try_into().ok()?);
            Some(RendezvousMsg::PunchAck(PunchAck { peer, punch_id }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sa(addr: &str) -> SocketAddr {
        addr.parse().unwrap()
    }

    #[test]
    fn punch_request_roundtrip_ipv4() {
        let req = PunchRequest {
            target: 0x1122_3344_5566_7788,
            self_reflex: sa("192.0.2.1:9001"),
        };
        let encoded = RendezvousMsg::PunchRequest(req).encode();
        assert_eq!(encoded.len(), PUNCH_REQUEST_LEN);
        match decode(&encoded) {
            Some(RendezvousMsg::PunchRequest(out)) => assert_eq!(out, req),
            other => panic!("expected PunchRequest, got {other:?}"),
        }
    }

    #[test]
    fn punch_request_roundtrip_ipv6() {
        let req = PunchRequest {
            target: 42,
            self_reflex: sa("[2001:db8::1]:443"),
        };
        let encoded = RendezvousMsg::PunchRequest(req).encode();
        match decode(&encoded) {
            Some(RendezvousMsg::PunchRequest(out)) => assert_eq!(out, req),
            other => panic!("expected PunchRequest, got {other:?}"),
        }
    }

    #[test]
    fn punch_introduce_roundtrip() {
        let intro = PunchIntroduce {
            peer: 0xDEAD_BEEF_FEED_CAFE,
            peer_reflex: sa("198.51.100.5:54321"),
            fire_at_ms: 1_700_000_000_500,
        };
        let encoded = RendezvousMsg::PunchIntroduce(intro).encode();
        assert_eq!(encoded.len(), PUNCH_INTRODUCE_LEN);
        match decode(&encoded) {
            Some(RendezvousMsg::PunchIntroduce(out)) => assert_eq!(out, intro),
            other => panic!("expected PunchIntroduce, got {other:?}"),
        }
    }

    #[test]
    fn punch_ack_roundtrip() {
        let ack = PunchAck {
            peer: 7,
            punch_id: 0xCAFEBABE,
        };
        let encoded = RendezvousMsg::PunchAck(ack).encode();
        assert_eq!(encoded.len(), PUNCH_ACK_LEN);
        match decode(&encoded) {
            Some(RendezvousMsg::PunchAck(out)) => assert_eq!(out, ack),
            other => panic!("expected PunchAck, got {other:?}"),
        }
    }

    #[test]
    fn unknown_kind_rejects() {
        // Length matches PunchAck but kind byte is outside the
        // reserved vocabulary. Must decode as `None`, never panic.
        let mut payload = vec![0u8; PUNCH_ACK_LEN];
        payload[0] = 0xFF;
        assert!(decode(&payload).is_none());
    }

    #[test]
    fn empty_payload_rejects() {
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn wrong_length_rejects_per_kind() {
        // A kind byte that claims PunchRequest but carries an
        // incorrect body length is malformed. Tests each kind's
        // length guard — regression protection for a decoder that
        // forgets to check length after reading the kind byte.
        let short_request = vec![KIND_PUNCH_REQUEST; PUNCH_REQUEST_LEN - 1];
        assert!(decode(&short_request).is_none());

        let short_introduce = vec![KIND_PUNCH_INTRODUCE; PUNCH_INTRODUCE_LEN - 1];
        assert!(decode(&short_introduce).is_none());

        let short_ack = vec![KIND_PUNCH_ACK; PUNCH_ACK_LEN - 1];
        assert!(decode(&short_ack).is_none());

        // Too-long is also rejected — extra trailing bytes are
        // never silently ignored.
        let long_ack = vec![KIND_PUNCH_ACK; PUNCH_ACK_LEN + 1];
        assert!(decode(&long_ack).is_none());
    }

    #[test]
    fn unknown_address_family_rejects() {
        // Build an otherwise-valid PunchRequest payload but with
        // an unknown address-family byte (neither 4 nor 6). Must
        // decode as None, not panic or produce a garbage addr.
        let mut payload = vec![0u8; PUNCH_REQUEST_LEN];
        payload[0] = KIND_PUNCH_REQUEST;
        // target = 0 (bytes 1..9 left at 0)
        // address family at byte 9 — set to invalid
        payload[9] = 7;
        assert!(decode(&payload).is_none());
    }

    #[test]
    fn encoded_kind_byte_matches_discriminator() {
        // Explicit layout check — guards against a future refactor
        // that reorders the discriminator byte away from offset 0,
        // which would silently break any peer running the prior
        // version.
        let req = PunchRequest {
            target: 1,
            self_reflex: sa("10.0.0.1:1"),
        };
        let intro = PunchIntroduce {
            peer: 1,
            peer_reflex: sa("10.0.0.1:1"),
            fire_at_ms: 1,
        };
        let ack = PunchAck {
            peer: 1,
            punch_id: 1,
        };
        assert_eq!(RendezvousMsg::PunchRequest(req).encode()[0], KIND_PUNCH_REQUEST);
        assert_eq!(
            RendezvousMsg::PunchIntroduce(intro).encode()[0],
            KIND_PUNCH_INTRODUCE
        );
        assert_eq!(RendezvousMsg::PunchAck(ack).encode()[0], KIND_PUNCH_ACK);
    }
}
