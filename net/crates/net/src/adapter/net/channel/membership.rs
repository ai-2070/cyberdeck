//! Channel membership subprotocol — Subscribe / Unsubscribe / Ack.
//!
//! Ships over `SUBPROTOCOL_CHANNEL_MEMBERSHIP` on existing encrypted
//! sessions. Carries the channel name (not just the u16 hash) so that
//! the publisher-side `ChannelConfig::can_subscribe` check can look up
//! the authoritative config by name — hash collisions must never cause
//! a subscribe to land on the wrong channel's ACL.

use bytes::{Buf, BufMut};

use super::name::{ChannelError, ChannelName};

/// Subprotocol ID for channel membership (subscribe / unsubscribe / ack).
pub const SUBPROTOCOL_CHANNEL_MEMBERSHIP: u16 = 0x0A00;

const MSG_SUBSCRIBE: u8 = 0;
const MSG_UNSUBSCRIBE: u8 = 1;
const MSG_ACK: u8 = 2;

const ACK_REASON_OK: u8 = 0;
const ACK_REASON_UNAUTHORIZED: u8 = 1;
const ACK_REASON_UNKNOWN_CHANNEL: u8 = 2;
const ACK_REASON_RATE_LIMITED: u8 = 3;
const ACK_REASON_TOO_MANY_CHANNELS: u8 = 4;

/// Why a `Subscribe` or `Unsubscribe` was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckReason {
    /// Capability or token check failed.
    Unauthorized,
    /// Channel not registered on the publisher side.
    UnknownChannel,
    /// Membership churn throttled.
    RateLimited,
    /// Per-peer channel cap exceeded.
    TooManyChannels,
}

/// Channel membership wire message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MembershipMsg {
    /// Ask the publisher to add this node to `channel`'s subscriber set.
    Subscribe {
        /// Channel the sender wants to subscribe to.
        channel: ChannelName,
        /// Request correlation nonce — echoed back in `Ack`.
        nonce: u64,
        /// Serialized [`super::super::identity::PermissionToken`]
        /// presented alongside the subscribe request. `None` / empty
        /// when the sender has no token to offer — the publisher's
        /// `authorize_subscribe` decides whether a token is required.
        token: Option<Vec<u8>>,
    },
    /// Ask the publisher to remove this node from `channel`'s subscriber set.
    Unsubscribe {
        /// Channel the sender wants to unsubscribe from.
        channel: ChannelName,
        /// Request correlation nonce — echoed back in `Ack`.
        nonce: u64,
    },
    /// Acknowledgement for a prior Subscribe / Unsubscribe.
    Ack {
        /// Nonce of the request being acknowledged.
        nonce: u64,
        /// Whether the request was accepted.
        accepted: bool,
        /// If rejected, why.
        reason: Option<AckReason>,
    },
}

/// Error returned by the membership codec.
#[derive(Debug, thiserror::Error)]
pub enum MembershipCodecError {
    /// Unknown or reserved message-type byte.
    #[error("unknown membership message type: {0}")]
    UnknownType(u8),
    /// Buffer ended mid-field.
    #[error("truncated membership message: {0}")]
    Truncated(&'static str),
    /// Channel name failed validation.
    #[error("channel name: {0}")]
    Name(#[from] ChannelError),
    /// Length prefix exceeds the remaining buffer.
    #[error("length {0} exceeds remaining {1}")]
    Overflow(usize, usize),
    /// Length prefix exceeds the declared max.
    #[error("channel name length {0} exceeds limit {1}")]
    NameTooLong(usize, usize),
}

/// Maximum channel-name length accepted by the decoder, in bytes.
/// Matches `name::MAX_NAME_LEN`; duplicated here to keep the wire check local.
const MAX_CHANNEL_NAME_LEN: usize = 255;

/// Encode a membership message to bytes.
pub fn encode(msg: &MembershipMsg) -> Vec<u8> {
    let mut buf = Vec::with_capacity(64);
    match msg {
        MembershipMsg::Subscribe {
            channel,
            nonce,
            token,
        } => {
            buf.put_u8(MSG_SUBSCRIBE);
            buf.put_u64_le(*nonce);
            let name = channel.as_str().as_bytes();
            buf.put_u8(name.len() as u8);
            buf.extend_from_slice(name);
            // Token payload: u16_le length + bytes. Zero length when
            // unset — decoder treats absent trailing bytes as no
            // token, for forward-compat with a potential pre-E-1
            // sender (none exist in practice but the cost is ~nil).
            let token_bytes: &[u8] = token.as_deref().unwrap_or(&[]);
            buf.put_u16_le(token_bytes.len() as u16);
            buf.extend_from_slice(token_bytes);
        }
        MembershipMsg::Unsubscribe { channel, nonce } => {
            buf.put_u8(MSG_UNSUBSCRIBE);
            buf.put_u64_le(*nonce);
            let name = channel.as_str().as_bytes();
            buf.put_u8(name.len() as u8);
            buf.extend_from_slice(name);
        }
        MembershipMsg::Ack {
            nonce,
            accepted,
            reason,
        } => {
            buf.put_u8(MSG_ACK);
            buf.put_u64_le(*nonce);
            buf.put_u8(u8::from(*accepted));
            buf.put_u8(match reason {
                None => ACK_REASON_OK,
                Some(AckReason::Unauthorized) => ACK_REASON_UNAUTHORIZED,
                Some(AckReason::UnknownChannel) => ACK_REASON_UNKNOWN_CHANNEL,
                Some(AckReason::RateLimited) => ACK_REASON_RATE_LIMITED,
                Some(AckReason::TooManyChannels) => ACK_REASON_TOO_MANY_CHANNELS,
            });
        }
    }
    buf
}

/// Decode a membership message from bytes.
pub fn decode(data: &[u8]) -> Result<MembershipMsg, MembershipCodecError> {
    if data.is_empty() {
        return Err(MembershipCodecError::Truncated("empty"));
    }
    let mut cur = std::io::Cursor::new(data);
    let tag = cur.get_u8();
    match tag {
        MSG_SUBSCRIBE | MSG_UNSUBSCRIBE => {
            if cur.remaining() < 9 {
                return Err(MembershipCodecError::Truncated("subscribe header"));
            }
            let nonce = cur.get_u64_le();
            let name_len = cur.get_u8() as usize;
            if name_len == 0 {
                return Err(MembershipCodecError::Truncated("empty channel name"));
            }
            if name_len > MAX_CHANNEL_NAME_LEN {
                return Err(MembershipCodecError::NameTooLong(
                    name_len,
                    MAX_CHANNEL_NAME_LEN,
                ));
            }
            if cur.remaining() < name_len {
                return Err(MembershipCodecError::Overflow(name_len, cur.remaining()));
            }
            let start = cur.position() as usize;
            let end = start + name_len;
            let name_bytes = &data[start..end];
            let name_str = std::str::from_utf8(name_bytes)
                .map_err(|_| MembershipCodecError::Truncated("non-utf8 channel name"))?;
            let channel = ChannelName::new(name_str)?;
            if tag == MSG_SUBSCRIBE {
                // Advance past the name we just read.
                cur.set_position(end as u64);
                // Token: u16_le length + bytes. Zero length ⇒ absent.
                // Legacy pre-E-1 payloads that stop exactly after the
                // name (zero trailing bytes) are treated as "no token"
                // for forward-compat. Exactly one trailing byte is
                // neither — it means a malformed sender wrote half
                // the length prefix, and the older
                // `cur.remaining() < 2` check silently accepted it as
                // "no token," hiding the bug from callers. Reject so
                // truncation surfaces as an error.
                let token = match cur.remaining() {
                    0 => None,
                    1 => {
                        return Err(MembershipCodecError::Truncated(
                            "subscribe token length prefix",
                        ));
                    }
                    _ => {
                        let token_len = cur.get_u16_le() as usize;
                        if token_len == 0 {
                            None
                        } else if cur.remaining() < token_len {
                            return Err(MembershipCodecError::Overflow(token_len, cur.remaining()));
                        } else {
                            let tstart = cur.position() as usize;
                            let tend = tstart + token_len;
                            Some(data[tstart..tend].to_vec())
                        }
                    }
                };
                Ok(MembershipMsg::Subscribe {
                    channel,
                    nonce,
                    token,
                })
            } else {
                Ok(MembershipMsg::Unsubscribe { channel, nonce })
            }
        }
        MSG_ACK => {
            if cur.remaining() < 10 {
                return Err(MembershipCodecError::Truncated("ack"));
            }
            let nonce = cur.get_u64_le();
            // Strict boolean: reject any byte other than 0 or 1 instead of
            // treating every non-zero value as "accepted". Prevents a
            // malformed sender from making an otherwise-unknown reason
            // code silently imply acceptance.
            let accepted = match cur.get_u8() {
                0 => false,
                1 => true,
                other => return Err(MembershipCodecError::UnknownType(other)),
            };
            let reason_byte = cur.get_u8();
            let reason = match reason_byte {
                ACK_REASON_OK => None,
                ACK_REASON_UNAUTHORIZED => Some(AckReason::Unauthorized),
                ACK_REASON_UNKNOWN_CHANNEL => Some(AckReason::UnknownChannel),
                ACK_REASON_RATE_LIMITED => Some(AckReason::RateLimited),
                ACK_REASON_TOO_MANY_CHANNELS => Some(AckReason::TooManyChannels),
                other => return Err(MembershipCodecError::UnknownType(other)),
            };
            Ok(MembershipMsg::Ack {
                nonce,
                accepted,
                reason,
            })
        }
        other => Err(MembershipCodecError::UnknownType(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(name: &str) -> ChannelName {
        ChannelName::new(name).unwrap()
    }

    #[test]
    fn test_roundtrip_subscribe_no_token() {
        let msg = MembershipMsg::Subscribe {
            channel: ch("sensors/lidar"),
            nonce: 0xDEAD_BEEF_CAFE_F00D,
            token: None,
        };
        let bytes = encode(&msg);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_subscribe_with_token() {
        // Arbitrary token bytes — codec doesn't validate internal
        // structure. Validation is the job of `PermissionToken`.
        let token_bytes = vec![0xABu8; 64];
        let msg = MembershipMsg::Subscribe {
            channel: ch("sensors/lidar"),
            nonce: 0xCAFE,
            token: Some(token_bytes),
        };
        let bytes = encode(&msg);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_legacy_subscribe_no_trailing_token_len_decodes_as_none() {
        // Forge a pre-E-1 payload (no u16 token_len trailer).
        use bytes::BufMut;
        let mut buf = Vec::new();
        buf.put_u8(MSG_SUBSCRIBE);
        buf.put_u64_le(42);
        let name = b"lab/x";
        buf.put_u8(name.len() as u8);
        buf.extend_from_slice(name);
        // NO token_len field — stops right after the name.
        let decoded = decode(&buf).unwrap();
        assert_eq!(
            decoded,
            MembershipMsg::Subscribe {
                channel: ch("lab/x"),
                nonce: 42,
                token: None,
            }
        );
    }

    #[test]
    fn test_regression_subscribe_one_byte_token_len_rejected() {
        // Regression for a cubic-flagged P2: a Subscribe payload with
        // exactly one trailing byte after the name used to be silently
        // accepted as "no token" because the decoder guarded on
        // `remaining() < 2`. A half-written `u16_le` length prefix is
        // a truncation, not a legacy payload — it must error.
        use bytes::BufMut;
        let mut buf = Vec::new();
        buf.put_u8(MSG_SUBSCRIBE);
        buf.put_u64_le(42);
        let name = b"lab/x";
        buf.put_u8(name.len() as u8);
        buf.extend_from_slice(name);
        // Exactly ONE trailing byte — half of a u16_le length prefix.
        buf.push(0x05);
        let err = decode(&buf).unwrap_err();
        assert!(
            matches!(err, MembershipCodecError::Truncated(_)),
            "expected Truncated, got {err:?}",
        );
    }

    #[test]
    fn test_roundtrip_unsubscribe() {
        let msg = MembershipMsg::Unsubscribe {
            channel: ch("control/estop"),
            nonce: 42,
        };
        let bytes = encode(&msg);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_ack_accepted() {
        let msg = MembershipMsg::Ack {
            nonce: 7,
            accepted: true,
            reason: None,
        };
        let bytes = encode(&msg);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_ack_rejected() {
        let reasons = [
            AckReason::Unauthorized,
            AckReason::UnknownChannel,
            AckReason::RateLimited,
            AckReason::TooManyChannels,
        ];
        for r in reasons {
            let msg = MembershipMsg::Ack {
                nonce: 99,
                accepted: false,
                reason: Some(r),
            };
            let bytes = encode(&msg);
            let decoded = decode(&bytes).unwrap();
            assert_eq!(decoded, msg);
        }
    }

    #[test]
    fn test_decode_empty_fails() {
        assert!(matches!(
            decode(&[]),
            Err(MembershipCodecError::Truncated(_))
        ));
    }

    #[test]
    fn test_decode_unknown_tag() {
        assert!(matches!(
            decode(&[0xFF]),
            Err(MembershipCodecError::UnknownType(0xFF))
        ));
    }

    #[test]
    fn test_decode_truncated_subscribe() {
        // Tag + partial nonce only.
        assert!(matches!(
            decode(&[MSG_SUBSCRIBE, 0, 0, 0]),
            Err(MembershipCodecError::Truncated(_))
        ));
    }

    #[test]
    fn test_decode_zero_name_len_rejected() {
        let mut buf = vec![MSG_SUBSCRIBE];
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.push(0); // name_len = 0
        assert!(matches!(
            decode(&buf),
            Err(MembershipCodecError::Truncated(_))
        ));
    }

    #[test]
    fn test_decode_overflow_name_len() {
        let mut buf = vec![MSG_SUBSCRIBE];
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.push(10); // claims 10 bytes but we only have 3
        buf.extend_from_slice(b"abc");
        assert!(matches!(
            decode(&buf),
            Err(MembershipCodecError::Overflow(10, 3))
        ));
    }

    #[test]
    fn test_decode_ack_strict_boolean_rejects_non_01() {
        // Valid ack with accepted=true (0x01), reason=OK — sanity check.
        let mut buf = vec![MSG_ACK];
        buf.extend_from_slice(&7u64.to_le_bytes());
        buf.push(1);
        buf.push(ACK_REASON_OK);
        assert!(decode(&buf).is_ok());

        // Same message but accepted=0xFF — must be rejected, not treated
        // as `true`.
        let mut buf = vec![MSG_ACK];
        buf.extend_from_slice(&7u64.to_le_bytes());
        buf.push(0xFF);
        buf.push(ACK_REASON_OK);
        assert!(matches!(
            decode(&buf),
            Err(MembershipCodecError::UnknownType(0xFF))
        ));
    }

    #[test]
    fn test_decode_invalid_channel_name() {
        let mut buf = vec![MSG_SUBSCRIBE];
        buf.extend_from_slice(&0u64.to_le_bytes());
        // name contains '//' which fails validation
        let name = b"a//b";
        buf.push(name.len() as u8);
        buf.extend_from_slice(name);
        assert!(matches!(decode(&buf), Err(MembershipCodecError::Name(_))));
    }
}
