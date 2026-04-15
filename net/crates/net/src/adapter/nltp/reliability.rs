//! Reliability modes for NLTP streams.
//!
//! NLTP supports two reliability modes:
//! - Fire-and-forget: No acknowledgments, maximum throughput
//! - Reliable: Per-stream reliability with selective NACKs

use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::protocol::NackPayload;

/// Trait for reliability mode implementations
pub trait ReliabilityMode: Send + Sync {
    /// Called when a packet is sent
    fn on_send(&mut self, seq: u64, packet: Bytes);

    /// Called when a packet is received. Returns true if accepted.
    fn on_receive(&mut self, seq: u64) -> bool;

    /// Check if this mode requires acknowledgments
    fn needs_ack(&self) -> bool;

    /// Build a NACK payload if there are missing sequences
    fn build_nack(&self) -> Option<NackPayload>;

    /// Process a received NACK and return packets to retransmit
    fn on_nack(&mut self, nack: &NackPayload) -> Vec<Bytes>;

    /// Get packets that need retransmission due to timeout
    fn get_timed_out(&mut self) -> Vec<Bytes>;

    /// Check if there are unacknowledged packets
    fn has_pending(&self) -> bool;

    /// Get the name of this reliability mode
    fn name(&self) -> &'static str;
}

/// Fire-and-forget reliability mode.
///
/// No acknowledgments, no retransmission, maximum throughput.
/// Suitable for:
/// - LLM token streams
/// - Embeddings
/// - Intermediate activations
/// - Metrics/telemetry
#[derive(Debug, Default)]
pub struct FireAndForget {
    /// Last sequence received (for ordering check)
    last_seq: AtomicU64,
}

impl FireAndForget {
    /// Create a new fire-and-forget mode
    pub fn new() -> Self {
        Self::default()
    }
}

impl ReliabilityMode for FireAndForget {
    #[inline]
    fn on_send(&mut self, _seq: u64, _packet: Bytes) {
        // Nothing to track
    }

    #[inline]
    fn on_receive(&mut self, seq: u64) -> bool {
        // Update last sequence (informational only)
        self.last_seq.fetch_max(seq, Ordering::Relaxed);
        true // Always accept
    }

    #[inline]
    fn needs_ack(&self) -> bool {
        false
    }

    #[inline]
    fn build_nack(&self) -> Option<NackPayload> {
        None
    }

    #[inline]
    fn on_nack(&mut self, _nack: &NackPayload) -> Vec<Bytes> {
        Vec::new()
    }

    #[inline]
    fn get_timed_out(&mut self) -> Vec<Bytes> {
        Vec::new()
    }

    #[inline]
    fn has_pending(&self) -> bool {
        false
    }

    #[inline]
    fn name(&self) -> &'static str {
        "fire-and-forget"
    }
}

/// Unacknowledged packet waiting for ACK/NACK
#[derive(Debug, Clone)]
struct UnackedPacket {
    /// Sequence number
    seq: u64,
    /// Packet data for retransmission
    packet: Bytes,
    /// Time when packet was sent
    sent_at: Instant,
    /// Number of retransmission attempts
    retries: u8,
}

/// Reliable stream mode with selective NACKs.
///
/// Features:
/// - Bounded retransmit window (32 packets)
/// - Selective NACKs (receiver-driven)
/// - Per-stream state
/// - Configurable RTO
///
/// Suitable for:
/// - Tool call results
/// - Guardrail decisions
/// - Session lifecycle events
/// - Error propagation
pub struct ReliableStream {
    /// Highest contiguous sequence received
    ack_seq: u64,
    /// Bitmap of received sequences beyond ack_seq (64-bit sliding window)
    sack_bitmap: u64,
    /// Pending unacknowledged packets (bounded)
    pending: VecDeque<UnackedPacket>,
    /// Retransmit timeout
    rto: Duration,
    /// Maximum pending packets
    max_pending: usize,
    /// Maximum retries per packet
    max_retries: u8,
}

impl ReliableStream {
    /// Default retransmit timeout
    pub const DEFAULT_RTO: Duration = Duration::from_millis(50);

    /// Default max pending packets
    pub const DEFAULT_MAX_PENDING: usize = 32;

    /// Default max retries
    pub const DEFAULT_MAX_RETRIES: u8 = 3;

    /// Create a new reliable stream with default settings
    pub fn new() -> Self {
        Self {
            ack_seq: 0,
            sack_bitmap: 0,
            pending: VecDeque::with_capacity(Self::DEFAULT_MAX_PENDING),
            rto: Self::DEFAULT_RTO,
            max_pending: Self::DEFAULT_MAX_PENDING,
            max_retries: Self::DEFAULT_MAX_RETRIES,
        }
    }

    /// Create with custom settings
    pub fn with_settings(rto: Duration, max_pending: usize, max_retries: u8) -> Self {
        Self {
            ack_seq: 0,
            sack_bitmap: 0,
            pending: VecDeque::with_capacity(max_pending),
            rto,
            max_pending,
            max_retries,
        }
    }

    /// Set the retransmit timeout
    pub fn set_rto(&mut self, rto: Duration) {
        self.rto = rto;
    }

    /// Get the current ack sequence
    pub fn ack_seq(&self) -> u64 {
        self.ack_seq
    }

    /// Process an acknowledgment
    pub fn on_ack(&mut self, ack_seq: u64) {
        // Remove all pending packets up to ack_seq
        while let Some(front) = self.pending.front() {
            if front.seq <= ack_seq {
                self.pending.pop_front();
            } else {
                break;
            }
        }
    }

    /// Check if there are gaps in received sequences
    fn has_gaps(&self) -> bool {
        // If sack_bitmap has any zeros before the first set bit, there are gaps
        self.sack_bitmap != 0 && self.sack_bitmap.trailing_zeros() > 0
    }

    /// Get bitmap of missing sequences
    fn missing_bitmap(&self) -> u64 {
        // Invert sack_bitmap to get missing sequences
        // Only consider bits up to the highest received
        if self.sack_bitmap == 0 {
            return 0;
        }
        let highest_bit = 63 - self.sack_bitmap.leading_zeros();
        let mask = if highest_bit >= 63 {
            u64::MAX
        } else {
            (1u64 << (highest_bit + 1)) - 1
        };
        (!self.sack_bitmap) & mask
    }
}

impl Default for ReliableStream {
    fn default() -> Self {
        Self::new()
    }
}

impl ReliabilityMode for ReliableStream {
    fn on_send(&mut self, seq: u64, packet: Bytes) {
        // Only track if we have room
        if self.pending.len() < self.max_pending {
            self.pending.push_back(UnackedPacket {
                seq,
                packet,
                sent_at: Instant::now(),
                retries: 0,
            });
        }
    }

    fn on_receive(&mut self, seq: u64) -> bool {
        if seq == 0 && self.ack_seq == 0 {
            // First packet (sequence 0 or 1 depending on convention)
            self.ack_seq = seq;
            return true;
        }

        if seq == self.ack_seq + 1 {
            // Next expected sequence
            self.ack_seq = seq;
            // Shift bitmap since we advanced by 1
            self.sack_bitmap >>= 1;

            // Advance through any buffered sequences (now bit 0 represents ack_seq + 1)
            while self.sack_bitmap & 1 != 0 {
                self.ack_seq += 1;
                self.sack_bitmap >>= 1;
            }
            true
        } else if seq > self.ack_seq && seq <= self.ack_seq + 64 {
            // Future sequence within window - mark in SACK bitmap
            let offset = seq - self.ack_seq - 1;
            self.sack_bitmap |= 1 << offset;
            true
        } else if seq <= self.ack_seq {
            // Duplicate (already received)
            false
        } else {
            // Too far ahead - reject
            false
        }
    }

    #[inline]
    fn needs_ack(&self) -> bool {
        true
    }

    fn build_nack(&self) -> Option<NackPayload> {
        if self.has_gaps() {
            Some(NackPayload {
                ack_seq: self.ack_seq,
                missing_bitmap: self.missing_bitmap(),
            })
        } else {
            None
        }
    }

    fn on_nack(&mut self, nack: &NackPayload) -> Vec<Bytes> {
        let mut retransmits = Vec::new();

        // Find packets to retransmit based on NACK
        for missing_seq in nack.missing_sequences() {
            // Find the packet in pending
            for unacked in &mut self.pending {
                if unacked.seq == missing_seq && unacked.retries < self.max_retries {
                    retransmits.push(unacked.packet.clone());
                    unacked.retries += 1;
                    unacked.sent_at = Instant::now();
                    break;
                }
            }
        }

        retransmits
    }

    fn get_timed_out(&mut self) -> Vec<Bytes> {
        let now = Instant::now();
        let mut retransmits = Vec::new();

        for unacked in &mut self.pending {
            if now.duration_since(unacked.sent_at) > self.rto && unacked.retries < self.max_retries
            {
                retransmits.push(unacked.packet.clone());
                unacked.retries += 1;
                unacked.sent_at = now;
            }
        }

        retransmits
    }

    #[inline]
    fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    #[inline]
    fn name(&self) -> &'static str {
        "reliable"
    }
}

impl std::fmt::Debug for ReliableStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReliableStream")
            .field("ack_seq", &self.ack_seq)
            .field("sack_bitmap", &format!("{:064b}", self.sack_bitmap))
            .field("pending_count", &self.pending.len())
            .field("rto_ms", &self.rto.as_millis())
            .finish()
    }
}

/// Create a boxed reliability mode from configuration
pub fn create_reliability_mode(reliable: bool) -> Box<dyn ReliabilityMode> {
    if reliable {
        Box::new(ReliableStream::new())
    } else {
        Box::new(FireAndForget::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_and_forget() {
        let mut mode = FireAndForget::new();

        // Should always accept
        assert!(mode.on_receive(1));
        assert!(mode.on_receive(3)); // Gap is fine
        assert!(mode.on_receive(2)); // Out of order is fine

        // No acks needed
        assert!(!mode.needs_ack());
        assert!(mode.build_nack().is_none());
        assert!(!mode.has_pending());

        // No retransmits
        mode.on_send(1, Bytes::from_static(b"test"));
        assert!(mode.get_timed_out().is_empty());
    }

    #[test]
    fn test_reliable_stream_in_order() {
        let mut mode = ReliableStream::new();

        // Receive in order
        assert!(mode.on_receive(1));
        assert_eq!(mode.ack_seq(), 1);

        assert!(mode.on_receive(2));
        assert_eq!(mode.ack_seq(), 2);

        assert!(mode.on_receive(3));
        assert_eq!(mode.ack_seq(), 3);

        // No NACK needed
        assert!(mode.build_nack().is_none());
    }

    #[test]
    fn test_reliable_stream_gap() {
        let mut mode = ReliableStream::new();

        // Receive with gap
        assert!(mode.on_receive(1));
        assert!(mode.on_receive(3)); // Gap at 2
        assert!(mode.on_receive(5)); // Gap at 4

        assert_eq!(mode.ack_seq(), 1);

        // Should have NACK
        let nack = mode.build_nack().unwrap();
        assert_eq!(nack.ack_seq, 1);

        // Missing: 2, 4 (relative to ack_seq + 1)
        let missing: Vec<_> = nack.missing_sequences().collect();
        assert!(missing.contains(&2));
        assert!(missing.contains(&4));
    }

    #[test]
    fn test_reliable_stream_fill_gap() {
        let mut mode = ReliableStream::new();

        // Receive out of order
        assert!(mode.on_receive(1));
        assert!(mode.on_receive(3));
        assert!(mode.on_receive(4));
        assert_eq!(mode.ack_seq(), 1);

        // Fill gap
        assert!(mode.on_receive(2));

        // Should advance
        assert_eq!(mode.ack_seq(), 4);

        // No NACK needed
        assert!(mode.build_nack().is_none());
    }

    #[test]
    fn test_reliable_stream_duplicate() {
        let mut mode = ReliableStream::new();

        assert!(mode.on_receive(1));
        assert!(mode.on_receive(2));

        // Duplicate should be rejected
        assert!(!mode.on_receive(1));
        assert!(!mode.on_receive(2));

        assert_eq!(mode.ack_seq(), 2);
    }

    #[test]
    fn test_reliable_stream_pending() {
        let mut mode = ReliableStream::new();

        assert!(!mode.has_pending());

        mode.on_send(1, Bytes::from_static(b"packet1"));
        mode.on_send(2, Bytes::from_static(b"packet2"));

        assert!(mode.has_pending());

        // ACK should clear pending
        mode.on_ack(2);
        assert!(!mode.has_pending());
    }

    #[test]
    fn test_reliable_stream_nack_retransmit() {
        let mut mode = ReliableStream::new();

        mode.on_send(1, Bytes::from_static(b"packet1"));
        mode.on_send(2, Bytes::from_static(b"packet2"));
        mode.on_send(3, Bytes::from_static(b"packet3"));

        // NACK for packet 2
        let nack = NackPayload {
            ack_seq: 1,
            missing_bitmap: 0b01, // Missing sequence 2
        };

        let retransmits = mode.on_nack(&nack);
        assert_eq!(retransmits.len(), 1);
        assert_eq!(&retransmits[0][..], b"packet2");
    }

    #[test]
    fn test_reliable_stream_too_far_ahead() {
        let mut mode = ReliableStream::new();

        assert!(mode.on_receive(1));

        // Sequence 100 is too far ahead (beyond 64-bit window)
        assert!(!mode.on_receive(100));

        assert_eq!(mode.ack_seq(), 1);
    }

    #[test]
    fn test_reliable_stream_nack_bitmap_full_window() {
        // Regression: when the highest received bit was 63 (full 64-bit window),
        // 1u64 << 64 overflowed, panicking in debug or producing wrong results
        // in release.
        let mut mode = ReliableStream::new();

        // Receive packet 1, then packet 65 (exactly 64 ahead, at the edge of the window)
        assert!(mode.on_receive(1));
        assert!(mode.on_receive(65));

        // build_nack should not panic and should report missing sequences
        let nack = mode.build_nack();
        assert!(
            nack.is_some(),
            "NACK should be generated for a gap spanning the full window"
        );

        let missing: Vec<_> = nack.unwrap().missing_sequences().collect();
        // Sequences 2..=64 are missing
        assert!(!missing.is_empty());
    }

    #[test]
    fn test_create_reliability_mode() {
        let mode = create_reliability_mode(false);
        assert_eq!(mode.name(), "fire-and-forget");

        let mode = create_reliability_mode(true);
        assert_eq!(mode.name(), "reliable");
    }
}
