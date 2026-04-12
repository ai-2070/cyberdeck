//! BLTP wire protocol definitions.
//!
//! This module defines the packet format for the Blackstream L0 Transport Protocol (BLTP).
//! All packets use a fixed 64-byte cache-line aligned header for optimal memory access.

use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Magic bytes: "BL" (0x424C)
pub const MAGIC: u16 = 0x424C;

/// Current protocol version (4-bit, 0–15)
pub const VERSION: u8 = 1;

/// 20-bit dispatch field: `[version:4][namespace_id:8][msg_type:8]`
///
/// Used for routing and classifying frames. Not a flags field.
/// Matches the RedEX 20-bit event type layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DispatchField {
    /// Protocol version (0–15, low 4 bits only)
    pub version: u8,
    /// Namespace identifier (0–255)
    pub namespace_id: u8,
    /// Message type (0–255)
    pub msg_type: u8,
}

impl DispatchField {
    /// Create a new dispatch field. Panics if version > 15.
    #[inline]
    pub fn new(version: u8, namespace_id: u8, msg_type: u8) -> Self {
        assert!(version <= 0x0F, "version must fit in 4 bits (0–15)");
        Self {
            version,
            namespace_id,
            msg_type,
        }
    }

    /// Pack into the canonical 20-bit dispatch value (low 20 bits of u32).
    ///
    /// Layout: `(version << 16) | (namespace_id << 8) | msg_type`
    #[inline]
    pub const fn to_u32(&self) -> u32 {
        let v = (self.version & 0x0F) as u32;
        let ns = self.namespace_id as u32;
        let mt = self.msg_type as u32;
        (v << 16) | (ns << 8) | mt
    }

    /// Unpack from a 20-bit dispatch value (low 20 bits of u32).
    #[inline]
    pub const fn from_u32(raw: u32) -> Self {
        Self {
            version: ((raw >> 16) & 0x0F) as u8,
            namespace_id: ((raw >> 8) & 0xFF) as u8,
            msg_type: (raw & 0xFF) as u8,
        }
    }
}

/// Header size in bytes (cache-line aligned)
pub const HEADER_SIZE: usize = 64;

/// Poly1305 authentication tag size
pub const TAG_SIZE: usize = 16;

/// XChaCha20 nonce size (legacy)
pub const NONCE_SIZE: usize = 24;

/// ChaCha20 nonce size (fast mode - counter-based)
pub const FAST_NONCE_SIZE: usize = 12;

/// Maximum packet size (fits in jumbo frame with headroom)
pub const MAX_PACKET_SIZE: usize = 8192;

/// Maximum payload size (packet - header - tag)
pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_SIZE - TAG_SIZE;

/// Packet flags for protocol control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct PacketFlags(u8);

impl PacketFlags {
    /// No flags set
    pub const NONE: Self = Self(0);
    /// Packet requires acknowledgment
    pub const RELIABLE: Self = Self(0b0000_0001);
    /// This is a NACK/retransmit request
    pub const NACK: Self = Self(0b0000_0010);
    /// High priority (bypass normal queuing)
    pub const PRIORITY: Self = Self(0b0000_0100);
    /// Final packet in batch
    pub const FIN: Self = Self(0b0000_1000);
    /// Handshake packet
    pub const HANDSHAKE: Self = Self(0b0001_0000);
    /// Heartbeat/keepalive
    pub const HEARTBEAT: Self = Self(0b0010_0000);

    /// Create flags from raw bits
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    /// Get raw bits
    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Check if a flag is set
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Set a flag
    #[inline]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Clear a flag
    #[inline]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Check if this is a handshake packet
    #[inline]
    pub const fn is_handshake(self) -> bool {
        self.contains(Self::HANDSHAKE)
    }

    /// Check if this is a heartbeat packet
    #[inline]
    pub const fn is_heartbeat(self) -> bool {
        self.contains(Self::HEARTBEAT)
    }

    /// Check if reliability is requested
    #[inline]
    pub const fn is_reliable(self) -> bool {
        self.contains(Self::RELIABLE)
    }

    /// Check if this is a NACK packet
    #[inline]
    pub const fn is_nack(self) -> bool {
        self.contains(Self::NACK)
    }
}

/// BLTP packet header - 64 bytes, cache-line aligned.
///
/// Wire format:
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |     MAGIC (0x424C)    |  VER  |     FLAGS     | NAMESPACE_ID  |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |    MSG_TYPE   |           Reserved (2 bytes)                  |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                                                               |
/// +                         NONCE (24 bytes)                      +
/// |                                                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       SESSION_ID (8 bytes)                    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       STREAM_ID (8 bytes)                     |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       SEQUENCE (8 bytes)                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |        PAYLOAD_LEN (2 bytes)  |       EVENT_COUNT (2 bytes)   |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// The dispatch triple (`version`, `namespace_id`, `msg_type`) forms a
/// 20-bit dispatch field: `[version:4][namespace_id:8][msg_type:8]`.
/// Use [`dispatch_field()`](BltpHeader::dispatch_field) or
/// [`dispatch_u32()`](BltpHeader::dispatch_u32) to access the packed value.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct BltpHeader {
    /// Magic: "BL" (0x424C)
    pub magic: u16,
    /// Protocol version, low 4 bits only (0–15)
    pub version: u8,
    /// Flags (reliability, priority, etc.)
    pub flags: PacketFlags,
    /// Namespace identifier for dispatch (0–255)
    pub namespace_id: u8,
    /// Message type for dispatch (0–255)
    pub msg_type: u8,
    /// Reserved for future use
    pub reserved: u16,
    /// XChaCha20 nonce (24 bytes) - random per packet
    pub nonce: [u8; NONCE_SIZE],
    /// Session identifier (from handshake)
    pub session_id: u64,
    /// Stream identifier (for multiplexing)
    pub stream_id: u64,
    /// Per-stream sequence number (monotonic)
    pub sequence: u64,
    /// Payload length (after encryption, before tag)
    pub payload_len: u16,
    /// Number of events in payload
    pub event_count: u16,
}

// Verify header size at compile time
const _: () = assert!(std::mem::size_of::<BltpHeader>() == HEADER_SIZE);

impl BltpHeader {
    /// Create a new header with default values.
    ///
    /// Uses the current protocol `VERSION`, namespace 0, msg_type 0.
    /// For custom dispatch values, set `namespace_id` and `msg_type` on
    /// the returned header.
    #[inline]
    pub fn new(
        session_id: u64,
        stream_id: u64,
        sequence: u64,
        nonce: [u8; NONCE_SIZE],
        payload_len: u16,
        event_count: u16,
        flags: PacketFlags,
    ) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            flags,
            namespace_id: 0,
            msg_type: 0,
            reserved: 0,
            nonce,
            session_id,
            stream_id,
            sequence,
            payload_len,
            event_count,
        }
    }

    /// Set the dispatch fields (version, namespace_id, msg_type) from a
    /// [`DispatchField`]. Chainable.
    #[inline]
    pub fn with_dispatch(mut self, dispatch: DispatchField) -> Self {
        self.version = dispatch.version;
        self.namespace_id = dispatch.namespace_id;
        self.msg_type = dispatch.msg_type;
        self
    }

    /// Create a handshake header
    #[inline]
    pub fn handshake(nonce: [u8; NONCE_SIZE], payload_len: u16) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            flags: PacketFlags::HANDSHAKE,
            namespace_id: 0,
            msg_type: 0,
            reserved: 0,
            nonce,
            session_id: 0,
            stream_id: 0,
            sequence: 0,
            payload_len,
            event_count: 0,
        }
    }

    /// Create a heartbeat header
    #[inline]
    pub fn heartbeat(session_id: u64) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            flags: PacketFlags::HEARTBEAT,
            namespace_id: 0,
            msg_type: 0,
            reserved: 0,
            nonce: [0u8; NONCE_SIZE],
            session_id,
            stream_id: 0,
            sequence: 0,
            payload_len: 0,
            event_count: 0,
        }
    }

    /// Get the dispatch field (version + namespace_id + msg_type).
    #[inline]
    pub fn dispatch_field(&self) -> DispatchField {
        DispatchField {
            version: self.version & 0x0F,
            namespace_id: self.namespace_id,
            msg_type: self.msg_type,
        }
    }

    /// Get the packed 20-bit dispatch value.
    ///
    /// Layout: `(version << 16) | (namespace_id << 8) | msg_type`
    #[inline]
    pub fn dispatch_u32(&self) -> u32 {
        self.dispatch_field().to_u32()
    }

    /// Get AAD (Additional Authenticated Data) for AEAD construction.
    ///
    /// Authenticates: magic, dispatch (version + namespace_id + msg_type),
    /// flags, payload_len, event_count, session_id, stream_id, sequence.
    /// This binds the encrypted payload to all header fields, preventing
    /// an attacker from modifying any field without breaking AEAD verification.
    #[inline]
    pub fn aad(&self) -> [u8; 36] {
        let mut aad = [0u8; 36];
        aad[0..2].copy_from_slice(&self.magic.to_le_bytes());
        // Pack dispatch as 4 bytes (low 20 bits meaningful)
        aad[2..6].copy_from_slice(&self.dispatch_u32().to_le_bytes());
        aad[6] = self.flags.bits();
        aad[7] = 0; // padding
        aad[8..10].copy_from_slice(&self.payload_len.to_le_bytes());
        aad[10..12].copy_from_slice(&self.event_count.to_le_bytes());
        aad[12..20].copy_from_slice(&self.session_id.to_le_bytes());
        aad[20..28].copy_from_slice(&self.stream_id.to_le_bytes());
        aad[28..36].copy_from_slice(&self.sequence.to_le_bytes());
        aad
    }

    /// Serialize header to bytes
    #[inline]
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        let mut cursor = &mut buf[..];

        cursor.put_u16_le(self.magic);
        cursor.put_u8(self.version & 0x0F);
        cursor.put_u8(self.flags.bits());
        cursor.put_u8(self.namespace_id);
        cursor.put_u8(self.msg_type);
        cursor.put_u16_le(self.reserved);
        cursor.put_slice(&self.nonce);
        cursor.put_u64_le(self.session_id);
        cursor.put_u64_le(self.stream_id);
        cursor.put_u64_le(self.sequence);
        cursor.put_u16_le(self.payload_len);
        cursor.put_u16_le(self.event_count);

        buf
    }

    /// Parse header from bytes
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < HEADER_SIZE {
            return None;
        }

        let mut cursor = &data[..HEADER_SIZE];

        let magic = cursor.get_u16_le();
        if magic != MAGIC {
            return None;
        }

        let version = cursor.get_u8() & 0x0F;
        let flags = PacketFlags::from_bits(cursor.get_u8());
        let namespace_id = cursor.get_u8();
        let msg_type = cursor.get_u8();
        let reserved = cursor.get_u16_le();

        let mut nonce = [0u8; NONCE_SIZE];
        cursor.copy_to_slice(&mut nonce);

        let session_id = cursor.get_u64_le();
        let stream_id = cursor.get_u64_le();
        let sequence = cursor.get_u64_le();
        let payload_len = cursor.get_u16_le();
        let event_count = cursor.get_u16_le();

        Some(Self {
            magic,
            version,
            flags,
            namespace_id,
            msg_type,
            reserved,
            nonce,
            session_id,
            stream_id,
            sequence,
            payload_len,
            event_count,
        })
    }

    /// Maximum events per packet. Each event needs at least a 4-byte length
    /// prefix, so this is bounded by MAX_PAYLOAD_SIZE / LEN_SIZE.
    pub const MAX_EVENTS_PER_PACKET: u16 = (MAX_PAYLOAD_SIZE / EventFrame::LEN_SIZE) as u16;

    /// Validate the header
    #[inline]
    pub fn validate(&self) -> bool {
        self.magic == MAGIC
            && (self.version & 0x0F) == VERSION
            && self.version <= 0x0F
            && (self.payload_len as usize) <= MAX_PAYLOAD_SIZE
            && self.event_count <= Self::MAX_EVENTS_PER_PACKET
    }
}

/// Event frame format for packing multiple events in a single packet.
///
/// Format: `[len: u32][data: [u8; len]]...`
///
/// Events are concatenated with 4-byte length prefixes. No additional framing.
pub struct EventFrame;

impl EventFrame {
    /// Size of the length prefix
    pub const LEN_SIZE: usize = 4;

    /// Write events to a buffer, returning the total bytes written
    #[inline]
    pub fn write_events(events: &[Bytes], buf: &mut BytesMut) -> usize {
        let start = buf.len();
        for event in events {
            buf.put_u32_le(event.len() as u32);
            buf.put_slice(event);
        }
        buf.len() - start
    }

    /// Read events from a buffer
    #[inline]
    pub fn read_events(mut data: Bytes, count: u16) -> Vec<Bytes> {
        // Cap pre-allocation to what the data can actually hold
        let max_events = data.remaining() / Self::LEN_SIZE;
        let mut events = Vec::with_capacity((count as usize).min(max_events));

        for _ in 0..count {
            if data.remaining() < Self::LEN_SIZE {
                break;
            }

            let len = data.get_u32_le() as usize;
            if data.remaining() < len {
                break;
            }

            events.push(data.split_to(len));
        }

        events
    }

    /// Calculate total size for events
    #[inline]
    pub fn calculate_size(events: &[Bytes]) -> usize {
        events.iter().map(|e| Self::LEN_SIZE + e.len()).sum()
    }
}

/// NACK payload for reliable streams
#[derive(Debug, Clone, Copy)]
pub struct NackPayload {
    /// Highest contiguous sequence received
    pub ack_seq: u64,
    /// Bitmap of missing sequences (relative to ack_seq + 1)
    pub missing_bitmap: u64,
}

impl NackPayload {
    /// Size of NACK payload
    pub const SIZE: usize = 16;

    /// Serialize to bytes
    #[inline]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..8].copy_from_slice(&self.ack_seq.to_le_bytes());
        buf[8..16].copy_from_slice(&self.missing_bitmap.to_le_bytes());
        buf
    }

    /// Parse from bytes
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }

        let ack_seq = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let missing_bitmap = u64::from_le_bytes(data[8..16].try_into().ok()?);

        Some(Self {
            ack_seq,
            missing_bitmap,
        })
    }

    /// Get missing sequence numbers
    #[inline]
    pub fn missing_sequences(&self) -> impl Iterator<Item = u64> + '_ {
        (0..64).filter_map(move |i| {
            if (self.missing_bitmap >> i) & 1 != 0 {
                Some(self.ack_seq + 1 + i)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(std::mem::size_of::<BltpHeader>(), HEADER_SIZE);
    }

    #[test]
    fn test_header_roundtrip() {
        let nonce = [0x42u8; NONCE_SIZE];
        let mut header = BltpHeader::new(
            0x1234567890ABCDEF,
            0xFEDCBA0987654321,
            42,
            nonce,
            1024,
            10,
            PacketFlags::RELIABLE,
        );
        header.namespace_id = 0xAB;
        header.msg_type = 0xCD;

        let bytes = header.to_bytes();
        let parsed = BltpHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, MAGIC);
        assert_eq!(parsed.version, VERSION);
        assert_eq!(parsed.flags, PacketFlags::RELIABLE);
        assert_eq!(parsed.namespace_id, 0xAB);
        assert_eq!(parsed.msg_type, 0xCD);
        assert_eq!(parsed.session_id, 0x1234567890ABCDEF);
        assert_eq!(parsed.stream_id, 0xFEDCBA0987654321);
        assert_eq!(parsed.sequence, 42);
        assert_eq!(parsed.nonce, nonce);
        assert_eq!(parsed.payload_len, 1024);
        assert_eq!(parsed.event_count, 10);
    }

    #[test]
    fn test_header_validation() {
        let nonce = [0u8; NONCE_SIZE];
        let header = BltpHeader::new(0, 0, 0, nonce, 1024, 0, PacketFlags::NONE);
        assert!(header.validate());

        // Invalid magic
        let mut bytes = header.to_bytes();
        bytes[0] = 0xFF;
        let invalid = BltpHeader::from_bytes(&bytes);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_packet_flags() {
        let flags = PacketFlags::NONE
            .with(PacketFlags::RELIABLE)
            .with(PacketFlags::PRIORITY);

        assert!(flags.is_reliable());
        assert!(flags.contains(PacketFlags::PRIORITY));
        assert!(!flags.is_handshake());

        let cleared = flags.without(PacketFlags::RELIABLE);
        assert!(!cleared.is_reliable());
        assert!(cleared.contains(PacketFlags::PRIORITY));
    }

    #[test]
    fn test_aad() {
        let nonce = [0u8; NONCE_SIZE];
        let mut header = BltpHeader::new(
            0x1234567890ABCDEF,
            0xFEDCBA0987654321,
            42,
            nonce,
            1024,
            10,
            PacketFlags::RELIABLE,
        );
        header.namespace_id = 5;
        header.msg_type = 10;

        let aad = header.aad();
        assert_eq!(aad.len(), 36);

        // Verify magic
        assert_eq!(u16::from_le_bytes([aad[0], aad[1]]), MAGIC);
        // Verify dispatch (packed as u32 LE at offset 2)
        let dispatch = u32::from_le_bytes([aad[2], aad[3], aad[4], aad[5]]);
        assert_eq!(dispatch, header.dispatch_u32());
        assert_eq!(dispatch, (1 << 16) | (5 << 8) | 10);
        // Verify flags
        assert_eq!(aad[6], PacketFlags::RELIABLE.bits());
    }

    #[test]
    fn test_dispatch_field() {
        let d = DispatchField::new(1, 2, 3);
        assert_eq!(d.version, 1);
        assert_eq!(d.namespace_id, 2);
        assert_eq!(d.msg_type, 3);
        assert_eq!(d.to_u32(), (1 << 16) | (2 << 8) | 3);

        let unpacked = DispatchField::from_u32(d.to_u32());
        assert_eq!(unpacked, d);

        // Max values
        let max = DispatchField::new(0x0F, 0xFF, 0xFF);
        assert_eq!(max.to_u32(), 0x000F_FFFF);
        assert_eq!(DispatchField::from_u32(max.to_u32()), max);
    }

    #[test]
    fn test_header_dispatch() {
        let dispatch = DispatchField::new(1, 42, 7);
        let nonce = [0u8; NONCE_SIZE];
        let header = BltpHeader::new(0x1234, 0x5678, 1, nonce, 100, 5, PacketFlags::NONE)
            .with_dispatch(dispatch);

        assert_eq!(header.version, 1);
        assert_eq!(header.namespace_id, 42);
        assert_eq!(header.msg_type, 7);
        assert_eq!(header.dispatch_field(), dispatch);
        assert_eq!(header.dispatch_u32(), (1 << 16) | (42 << 8) | 7);

        // Roundtrip through wire
        let bytes = header.to_bytes();
        let parsed = BltpHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.dispatch_field(), dispatch);
    }

    #[test]
    #[should_panic(expected = "version must fit in 4 bits")]
    fn test_dispatch_field_version_overflow() {
        DispatchField::new(16, 0, 0);
    }

    #[test]
    fn test_event_frame_roundtrip() {
        let events = vec![
            Bytes::from_static(b"event1"),
            Bytes::from_static(b"event2"),
            Bytes::from_static(b"event3"),
        ];

        let mut buf = BytesMut::with_capacity(256);
        let size = EventFrame::write_events(&events, &mut buf);

        assert_eq!(size, 3 * 4 + 6 + 6 + 6); // 3 length prefixes + event data

        let parsed = EventFrame::read_events(buf.freeze(), 3);
        assert_eq!(parsed.len(), 3);
        assert_eq!(&parsed[0][..], b"event1");
        assert_eq!(&parsed[1][..], b"event2");
        assert_eq!(&parsed[2][..], b"event3");
    }

    #[test]
    fn test_nack_payload_roundtrip() {
        let nack = NackPayload {
            ack_seq: 100,
            missing_bitmap: 0b1010_0101,
        };

        let bytes = nack.to_bytes();
        let parsed = NackPayload::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.ack_seq, 100);
        assert_eq!(parsed.missing_bitmap, 0b1010_0101);

        let missing: Vec<_> = parsed.missing_sequences().collect();
        assert_eq!(missing, vec![101, 103, 106, 108]);
    }

    #[test]
    fn test_validate_rejects_excessive_event_count() {
        let nonce = [0u8; NONCE_SIZE];
        let header = BltpHeader::new(0, 0, 0, nonce, 100, 10, PacketFlags::NONE);
        assert!(header.validate());

        // event_count exceeding MAX_EVENTS_PER_PACKET must be rejected
        let header = BltpHeader::new(
            0,
            0,
            0,
            nonce,
            100,
            BltpHeader::MAX_EVENTS_PER_PACKET + 1,
            PacketFlags::NONE,
        );
        assert!(!header.validate());
    }

    #[test]
    fn test_read_events_caps_allocation() {
        // Attacker-controlled count=65535 with tiny payload should not
        // allocate 65535 slots
        let data = Bytes::from_static(b"");
        let events = EventFrame::read_events(data, u16::MAX);
        assert!(events.is_empty());
        assert!(events.capacity() <= 1);
    }
}
