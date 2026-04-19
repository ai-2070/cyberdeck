//! Stream window subprotocol — receiver → sender credit grants.
//!
//! Ships over `SUBPROTOCOL_STREAM_WINDOW` on existing encrypted
//! sessions. Each grant adds `credit_bytes` to the sender's
//! `tx_credit_remaining` on the named stream. Wire layout is fixed:
//! 12 bytes per message (`u64 stream_id LE` + `u32 credit_bytes LE`).

use bytes::{Buf, BufMut};

/// Subprotocol ID for stream-window credit grants.
pub const SUBPROTOCOL_STREAM_WINDOW: u16 = 0x0B00;

/// Fixed wire size in bytes.
pub const STREAM_WINDOW_SIZE: usize = 12;

/// Receiver → sender credit grant. Each arrival bumps the sender's
/// `tx_credit_remaining` on `stream_id` by `credit_bytes` (saturating
/// at `u32::MAX`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamWindow {
    /// Stream the grant applies to.
    pub stream_id: u64,
    /// Bytes of credit being extended.
    pub credit_bytes: u32,
}

/// Errors produced by the codec.
#[derive(Debug, thiserror::Error)]
pub enum StreamWindowCodecError {
    /// Buffer shorter than the fixed 12 bytes.
    #[error("truncated stream-window message: {0} bytes (need 12)")]
    Truncated(usize),
    /// Buffer longer than the fixed 12 bytes. Rejects garbage trailers
    /// rather than silently ignoring them.
    #[error("oversize stream-window message: {0} bytes (need 12)")]
    Oversize(usize),
}

impl StreamWindow {
    /// Encode to a fixed 12-byte buffer.
    #[inline]
    pub fn encode(&self) -> [u8; STREAM_WINDOW_SIZE] {
        let mut buf = [0u8; STREAM_WINDOW_SIZE];
        (&mut buf[..8]).put_u64_le(self.stream_id);
        (&mut buf[8..]).put_u32_le(self.credit_bytes);
        buf
    }

    /// Decode a 12-byte message. Returns an error on truncated or
    /// oversize input.
    pub fn decode(data: &[u8]) -> Result<Self, StreamWindowCodecError> {
        match data.len() {
            n if n < STREAM_WINDOW_SIZE => Err(StreamWindowCodecError::Truncated(n)),
            n if n > STREAM_WINDOW_SIZE => Err(StreamWindowCodecError::Oversize(n)),
            _ => {
                let mut cur = std::io::Cursor::new(data);
                let stream_id = cur.get_u64_le();
                let credit_bytes = cur.get_u32_le();
                Ok(Self {
                    stream_id,
                    credit_bytes,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let msg = StreamWindow {
            stream_id: 0xDEAD_BEEF_CAFE_F00D,
            credit_bytes: 65_536,
        };
        let bytes = msg.encode();
        assert_eq!(bytes.len(), STREAM_WINDOW_SIZE);
        let parsed = StreamWindow::decode(&bytes).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_decode_truncated_rejected() {
        let err = StreamWindow::decode(&[0u8; 11]).unwrap_err();
        assert!(matches!(err, StreamWindowCodecError::Truncated(11)));
    }

    #[test]
    fn test_decode_oversize_rejected() {
        let err = StreamWindow::decode(&[0u8; 13]).unwrap_err();
        assert!(matches!(err, StreamWindowCodecError::Oversize(13)));
    }

    #[test]
    fn test_decode_empty_rejected() {
        let err = StreamWindow::decode(&[]).unwrap_err();
        assert!(matches!(err, StreamWindowCodecError::Truncated(0)));
    }

    #[test]
    fn test_endianness_is_little_endian() {
        // Explicit LE check — stream_id=1 must produce `01 00 ... 00`.
        let msg = StreamWindow {
            stream_id: 1,
            credit_bytes: 1,
        };
        let bytes = msg.encode();
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x00);
        assert_eq!(bytes[8], 0x01);
        assert_eq!(bytes[9], 0x00);
    }
}
