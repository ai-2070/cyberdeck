//! Reflex-probe subprotocol — mesh-native STUN analog.
//!
//! A peer receiving a reflex request on [`SUBPROTOCOL_REFLEX`]
//! replies with the source `ip:port` it observed on the UDP
//! envelope of the request. The requester collects one such
//! observation per probe to detect its own public-facing
//! `SocketAddr` without needing a third-party STUN server.
//!
//! Two+ probes to different peers are enough to classify the
//! local NAT as symmetric (reflex port differs per destination)
//! vs. cone (reflex port stable) vs. open (reflex equals bind).
//! Classification itself is stage 2 of `NAT_TRAVERSAL_PLAN.md`;
//! this module only owns the wire format + single-probe flow.
//!
//! # Wire format
//!
//! On-wire payloads are carried inside the existing event-frame
//! wrapper on [`SUBPROTOCOL_REFLEX`]:
//!
//! - **Request** — `[]` (empty event). The source socket address
//!   comes from the UDP envelope; no body needed.
//! - **Response** — 19 bytes: `family: u8 | addr: [u8; 16] | port: u16`.
//!   `family` is `4` for IPv4 (with the address in the low 4 bytes
//!   of `addr` and the upper 12 bytes zeroed) or `6` for IPv6
//!   (address fills the full 16 bytes). Port is big-endian.
//!
//! Length-based dispatch: the handler distinguishes request vs.
//! response by payload length alone. A malformed payload (neither
//! 0 nor 19 bytes) is dropped silently.
//!
//! # Framing (not correctness)
//!
//! Reflex discovery is a latency / throughput optimization, not a
//! connectivity requirement. A reflex-probe timeout or malformed
//! response doesn't prevent two peers from exchanging traffic —
//! they still have the routed-handshake path. Docs that talk about
//! reflex discovery must not imply it's required for NATed peers
//! to communicate.
//!
//! [`SUBPROTOCOL_REFLEX`]: super::SUBPROTOCOL_REFLEX

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use bytes::{BufMut, Bytes, BytesMut};

/// Length of an encoded reflex request payload (event body).
pub const REQUEST_LEN: usize = 0;

/// Length of an encoded reflex response payload (event body).
/// `family(1) + addr(16) + port(2) = 19`. The address field
/// accommodates IPv6; IPv4 addresses occupy the low 4 bytes with
/// the upper 12 bytes zeroed.
pub const RESPONSE_LEN: usize = 19;

const FAMILY_V4: u8 = 4;
const FAMILY_V6: u8 = 6;

/// Decoded reflex subprotocol message. The two variants correspond
/// to the two event-frame payload shapes described in the module
/// docs. Use [`decode`] to obtain one from raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReflexMsg {
    /// Empty-body request. The receiver echoes the UDP-source
    /// address back as a `Response`.
    Request,
    /// 19-byte response carrying the observer's perception of the
    /// requester's public `SocketAddr`.
    Response(SocketAddr),
}

/// Encode a reflex request. The event body is intentionally empty
/// — the receiver reads the source address from the UDP envelope,
/// not the payload.
#[inline]
pub fn encode_request() -> Bytes {
    Bytes::new()
}

/// Encode a reflex response carrying the observer's view of the
/// peer's public `SocketAddr`. Output length is always
/// [`RESPONSE_LEN`].
pub fn encode_response(observed: SocketAddr) -> Bytes {
    let mut buf = BytesMut::with_capacity(RESPONSE_LEN);
    match observed {
        SocketAddr::V4(v4) => {
            buf.put_u8(FAMILY_V4);
            let mut addr = [0u8; 16];
            addr[..4].copy_from_slice(&v4.ip().octets());
            buf.put_slice(&addr);
            buf.put_u16(v4.port());
        }
        SocketAddr::V6(v6) => {
            buf.put_u8(FAMILY_V6);
            buf.put_slice(&v6.ip().octets());
            buf.put_u16(v6.port());
        }
    }
    debug_assert_eq!(buf.len(), RESPONSE_LEN);
    buf.freeze()
}

/// Decode a reflex payload. Returns `None` on any malformed input
/// (wrong length, unknown family byte). Callers drop malformed
/// payloads silently — the subprotocol is an optimization, so a
/// bad packet is neither fatal nor worth surfacing.
pub fn decode(payload: &[u8]) -> Option<ReflexMsg> {
    match payload.len() {
        REQUEST_LEN => Some(ReflexMsg::Request),
        RESPONSE_LEN => {
            let family = payload[0];
            let addr_bytes: [u8; 16] = payload[1..17].try_into().ok()?;
            let port = u16::from_be_bytes([payload[17], payload[18]]);
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
            Some(ReflexMsg::Response(SocketAddr::new(ip, port)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_empty() {
        assert_eq!(encode_request().len(), REQUEST_LEN);
        assert_eq!(REQUEST_LEN, 0);
    }

    #[test]
    fn response_roundtrip_ipv4() {
        let addr: SocketAddr = "192.0.2.45:9001".parse().unwrap();
        let encoded = encode_response(addr);
        assert_eq!(encoded.len(), RESPONSE_LEN);
        match decode(&encoded) {
            Some(ReflexMsg::Response(out)) => assert_eq!(out, addr),
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn response_roundtrip_ipv6() {
        let addr: SocketAddr = "[2001:db8::1]:443".parse().unwrap();
        let encoded = encode_response(addr);
        assert_eq!(encoded.len(), RESPONSE_LEN);
        match decode(&encoded) {
            Some(ReflexMsg::Response(out)) => assert_eq!(out, addr),
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn response_layout_ipv4_zero_padded_upper_bytes() {
        // Explicit byte-level check: IPv4 occupies the low 4 bytes
        // of the 16-byte address field (indices 1..5 after the
        // family byte), with the upper 12 bytes (indices 5..17)
        // zeroed, and the port in big-endian at indices 17..19.
        // Guards against a future optimizer "compressing" the
        // representation or flipping endianness.
        let addr: SocketAddr = "10.0.0.1:80".parse().unwrap();
        let encoded = encode_response(addr);
        assert_eq!(encoded[0], 4, "family byte");
        assert_eq!(&encoded[1..5], &[10, 0, 0, 1], "IPv4 in low 4 bytes");
        assert_eq!(&encoded[5..17], &[0u8; 12], "upper 12 bytes zeroed");
        assert_eq!(&encoded[17..19], &80u16.to_be_bytes(), "port big-endian");
    }

    #[test]
    fn empty_payload_is_request() {
        assert_eq!(decode(&[]), Some(ReflexMsg::Request));
    }

    #[test]
    fn unknown_length_rejects() {
        assert!(decode(&[0xff]).is_none());
        assert!(decode(&[0; 18]).is_none());
        assert!(decode(&[0; 20]).is_none());
    }

    #[test]
    fn unknown_family_rejects() {
        // Length matches a response but family byte is neither 4 nor 6.
        let mut payload = vec![0u8; RESPONSE_LEN];
        payload[0] = 7; // invalid family
        assert!(decode(&payload).is_none());
    }
}
