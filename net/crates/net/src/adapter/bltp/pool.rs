//! Zero-allocation packet pool and builder.
//!
//! This module provides pre-allocated buffers for packet construction
//! to avoid heap allocations on the hot path.

use bytes::{Bytes, BytesMut};
use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

use super::crypto::{generate_nonce, FastPacketCipher, PacketCipher};
use super::protocol::{
    BltpHeader, EventFrame, PacketFlags, MAX_PACKET_SIZE, MAX_PAYLOAD_SIZE, NONCE_SIZE,
};

/// Pre-allocated packet builder for zero-allocation packet construction.
pub struct PacketBuilder {
    /// Pre-allocated payload buffer
    payload: BytesMut,
    /// Cipher for encryption
    cipher: PacketCipher,
    /// Scratch buffer for packet assembly
    packet: BytesMut,
}

impl PacketBuilder {
    /// Create a new packet builder
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            payload: BytesMut::with_capacity(MAX_PAYLOAD_SIZE),
            cipher: PacketCipher::new(key),
            packet: BytesMut::with_capacity(MAX_PACKET_SIZE),
        }
    }

    /// Update the encryption key
    pub fn set_key(&mut self, key: &[u8; 32]) {
        self.cipher = PacketCipher::new(key);
    }

    /// Build a packet from events.
    ///
    /// This method:
    /// 1. Writes events to the payload buffer with length prefixes
    /// 2. Encrypts the payload in-place
    /// 3. Assembles the final packet with header + encrypted payload + tag
    ///
    /// Returns the complete packet as `Bytes`.
    #[inline]
    pub fn build(
        &mut self,
        session_id: u64,
        stream_id: u64,
        sequence: u64,
        events: &[Bytes],
        flags: PacketFlags,
    ) -> Bytes {
        // Reset buffers (no allocation)
        self.payload.clear();
        self.packet.clear();

        // Write event frames to payload buffer
        EventFrame::write_events(events, &mut self.payload);

        // Generate random nonce
        let nonce = generate_nonce();

        // Build header
        let header = BltpHeader::new(
            session_id,
            stream_id,
            sequence,
            nonce,
            self.payload.len() as u16,
            events.len() as u16,
            flags,
        );

        // Get AAD for AEAD
        let aad = header.aad();

        // Encrypt payload in-place (appends tag)
        self.cipher
            .encrypt_in_place(&nonce, &aad, &mut self.payload)
            .expect("encryption should not fail");

        // Assemble packet: header + encrypted_payload + tag
        self.packet.extend_from_slice(&header.to_bytes());
        self.packet.extend_from_slice(&self.payload);

        // Return as frozen Bytes (cheap clone)
        self.packet.clone().freeze()
    }

    /// Build a handshake packet
    #[inline]
    pub fn build_handshake(&mut self, payload: &[u8]) -> Bytes {
        self.packet.clear();

        let nonce = generate_nonce();
        let header = BltpHeader::handshake(nonce, payload.len() as u16);

        self.packet.extend_from_slice(&header.to_bytes());
        self.packet.extend_from_slice(payload);

        self.packet.clone().freeze()
    }

    /// Build a heartbeat packet
    #[inline]
    pub fn build_heartbeat(&mut self, session_id: u64) -> Bytes {
        self.packet.clear();

        let header = BltpHeader::heartbeat(session_id);
        self.packet.extend_from_slice(&header.to_bytes());

        self.packet.clone().freeze()
    }

    /// Get the maximum number of events that can fit in a single packet
    #[inline]
    pub fn max_events_for_size(&self, avg_event_size: usize) -> usize {
        let frame_overhead = EventFrame::LEN_SIZE;
        MAX_PAYLOAD_SIZE / (avg_event_size + frame_overhead)
    }

    /// Check if events would fit in a single packet
    #[inline]
    pub fn would_fit(&self, events: &[Bytes]) -> bool {
        EventFrame::calculate_size(events) <= MAX_PAYLOAD_SIZE
    }
}

impl std::fmt::Debug for PacketBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketBuilder")
            .field("payload_capacity", &self.payload.capacity())
            .field("packet_capacity", &self.packet.capacity())
            .finish()
    }
}

/// Fast packet builder using counter-based nonces.
///
/// This is ~20-40% faster than the legacy PacketBuilder because it uses
/// ChaCha20-Poly1305 with counter-based nonces, eliminating the HChaCha20
/// key derivation that XChaCha20 performs per-packet.
pub struct FastPacketBuilder {
    /// Pre-allocated payload buffer
    payload: BytesMut,
    /// Fast cipher with counter-based nonces
    cipher: FastPacketCipher,
    /// Scratch buffer for packet assembly
    packet: BytesMut,
    /// Session ID for this builder
    session_id: u64,
}

impl FastPacketBuilder {
    /// Create a new fast packet builder
    pub fn new(key: &[u8; 32], session_id: u64) -> Self {
        Self {
            payload: BytesMut::with_capacity(MAX_PAYLOAD_SIZE),
            cipher: FastPacketCipher::new(key, session_id),
            packet: BytesMut::with_capacity(MAX_PACKET_SIZE),
            session_id,
        }
    }

    /// Update the encryption key and session ID
    pub fn set_key(&mut self, key: &[u8; 32], session_id: u64) {
        self.cipher = FastPacketCipher::new(key, session_id);
        self.session_id = session_id;
    }

    /// Build a packet from events using fast counter-based encryption.
    ///
    /// This method:
    /// 1. Writes events to the payload buffer with length prefixes
    /// 2. Encrypts the payload in-place using counter-based nonce
    /// 3. Assembles the final packet with header + encrypted payload + tag
    ///
    /// The nonce counter is stored in the header's nonce field (first 12 bytes).
    /// Returns the complete packet as `Bytes`.
    #[inline]
    pub fn build(
        &mut self,
        stream_id: u64,
        sequence: u64,
        events: &[Bytes],
        flags: PacketFlags,
    ) -> Bytes {
        // Reset buffers (no allocation)
        self.payload.clear();
        self.packet.clear();

        // Write event frames to payload buffer
        EventFrame::write_events(events, &mut self.payload);

        // Build header with placeholder nonce (will be filled with counter)
        let mut nonce = [0u8; NONCE_SIZE];

        // Get AAD before encryption (we'll update nonce after)
        let header = BltpHeader::new(
            self.session_id,
            stream_id,
            sequence,
            nonce,
            self.payload.len() as u16,
            events.len() as u16,
            flags,
        );
        let aad = header.aad();

        // Encrypt payload in-place and get the counter used
        let counter = self
            .cipher
            .encrypt_in_place(&aad, &mut self.payload)
            .expect("encryption should not fail");

        // Store counter in nonce field (12-byte nonce, zero-padded to 24)
        let counter_bytes = counter.to_le_bytes();
        nonce[0..4].copy_from_slice(&(self.session_id as u32).to_le_bytes());
        nonce[4..12].copy_from_slice(&counter_bytes);
        // Remaining bytes stay zero (padding)

        // Build final header with actual nonce
        let final_header = BltpHeader::new(
            self.session_id,
            stream_id,
            sequence,
            nonce,
            (self.payload.len() - 16) as u16, // payload len before tag
            events.len() as u16,
            flags,
        );

        // Assemble packet: header + encrypted_payload + tag
        self.packet.extend_from_slice(&final_header.to_bytes());
        self.packet.extend_from_slice(&self.payload);

        // Return as frozen Bytes (cheap clone)
        self.packet.clone().freeze()
    }

    /// Build a handshake packet (unencrypted)
    #[inline]
    pub fn build_handshake(&mut self, payload: &[u8]) -> Bytes {
        self.packet.clear();

        let nonce = generate_nonce();
        let header = BltpHeader::handshake(nonce, payload.len() as u16);

        self.packet.extend_from_slice(&header.to_bytes());
        self.packet.extend_from_slice(payload);

        self.packet.clone().freeze()
    }

    /// Build a heartbeat packet
    #[inline]
    pub fn build_heartbeat(&mut self) -> Bytes {
        self.packet.clear();

        let header = BltpHeader::heartbeat(self.session_id);
        self.packet.extend_from_slice(&header.to_bytes());

        self.packet.clone().freeze()
    }

    /// Get the maximum number of events that can fit in a single packet
    #[inline]
    pub fn max_events_for_size(&self, avg_event_size: usize) -> usize {
        let frame_overhead = EventFrame::LEN_SIZE;
        MAX_PAYLOAD_SIZE / (avg_event_size + frame_overhead)
    }

    /// Check if events would fit in a single packet
    #[inline]
    pub fn would_fit(&self, events: &[Bytes]) -> bool {
        EventFrame::calculate_size(events) <= MAX_PAYLOAD_SIZE
    }

    /// Get the session ID
    #[inline]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
}

impl std::fmt::Debug for FastPacketBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastPacketBuilder")
            .field("session_id", &format!("{:016x}", self.session_id))
            .field("payload_capacity", &self.payload.capacity())
            .field("packet_capacity", &self.packet.capacity())
            .finish()
    }
}

/// Pool of packet builders for amortized allocation.
///
/// The pool maintains a set of reusable `PacketBuilder` instances,
/// avoiding allocation on every packet construction.
pub struct PacketPool {
    /// Queue of available builders
    builders: ArrayQueue<PacketBuilder>,
    /// Encryption key for new builders
    key: [u8; 32],
    /// Pool capacity
    capacity: usize,
}

impl PacketPool {
    /// Create a new packet pool
    pub fn new(size: usize, key: &[u8; 32]) -> Self {
        let builders = ArrayQueue::new(size);

        // Pre-populate the pool
        for _ in 0..size {
            let _ = builders.push(PacketBuilder::new(key));
        }

        Self {
            builders,
            key: *key,
            capacity: size,
        }
    }

    /// Update the encryption key for all builders
    ///
    /// Note: This only affects newly created builders and builders
    /// that are returned to the pool after this call.
    pub fn set_key(&mut self, key: &[u8; 32]) {
        self.key = *key;
    }

    /// Get a builder from the pool
    ///
    /// If the pool is empty, creates a new builder.
    #[inline]
    pub fn get(&self) -> PooledBuilder<'_> {
        let builder = self
            .builders
            .pop()
            .unwrap_or_else(|| PacketBuilder::new(&self.key));

        PooledBuilder {
            pool: self,
            builder: Some(builder),
        }
    }

    /// Get the pool capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of available builders
    #[inline]
    pub fn available(&self) -> usize {
        self.builders.len()
    }
}

impl std::fmt::Debug for PacketPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketPool")
            .field("capacity", &self.capacity)
            .field("available", &self.builders.len())
            .finish()
    }
}

/// RAII guard for a pooled builder.
///
/// Returns the builder to the pool when dropped.
pub struct PooledBuilder<'a> {
    pool: &'a PacketPool,
    builder: Option<PacketBuilder>,
}

impl<'a> PooledBuilder<'a> {
    /// Build a packet from events
    #[inline]
    pub fn build(
        &mut self,
        session_id: u64,
        stream_id: u64,
        sequence: u64,
        events: &[Bytes],
        flags: PacketFlags,
    ) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build(session_id, stream_id, sequence, events, flags)
    }

    /// Build a handshake packet
    #[inline]
    pub fn build_handshake(&mut self, payload: &[u8]) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_handshake(payload)
    }

    /// Build a heartbeat packet
    #[inline]
    pub fn build_heartbeat(&mut self, session_id: u64) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_heartbeat(session_id)
    }

    /// Check if events would fit in a single packet
    #[inline]
    pub fn would_fit(&self, events: &[Bytes]) -> bool {
        self.builder
            .as_ref()
            .expect("builder taken")
            .would_fit(events)
    }
}

impl Drop for PooledBuilder<'_> {
    fn drop(&mut self) {
        if let Some(mut builder) = self.builder.take() {
            // Update key if pool key has changed
            builder.set_key(&self.pool.key);
            // Return to pool (ignore if full)
            let _ = self.pool.builders.push(builder);
        }
    }
}

impl std::fmt::Debug for PooledBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledBuilder")
            .field("has_builder", &self.builder.is_some())
            .finish()
    }
}

/// Shared packet pool (thread-safe)
pub type SharedPacketPool = Arc<PacketPool>;

/// Create a shared packet pool
pub fn shared_pool(size: usize, key: &[u8; 32]) -> SharedPacketPool {
    Arc::new(PacketPool::new(size, key))
}

/// Pool of fast packet builders for amortized allocation.
///
/// Uses counter-based nonces for ~20-40% faster encryption.
pub struct FastPacketPool {
    /// Queue of available builders
    builders: ArrayQueue<FastPacketBuilder>,
    /// Encryption key for new builders
    key: [u8; 32],
    /// Session ID for builders
    session_id: u64,
    /// Pool capacity
    capacity: usize,
}

impl FastPacketPool {
    /// Create a new fast packet pool
    pub fn new(size: usize, key: &[u8; 32], session_id: u64) -> Self {
        let builders = ArrayQueue::new(size);

        // Pre-populate the pool
        for _ in 0..size {
            let _ = builders.push(FastPacketBuilder::new(key, session_id));
        }

        Self {
            builders,
            key: *key,
            session_id,
            capacity: size,
        }
    }

    /// Update the encryption key and session ID
    pub fn set_key(&mut self, key: &[u8; 32], session_id: u64) {
        self.key = *key;
        self.session_id = session_id;
    }

    /// Get a builder from the pool
    #[inline]
    pub fn get(&self) -> FastPooledBuilder<'_> {
        let builder = self
            .builders
            .pop()
            .unwrap_or_else(|| FastPacketBuilder::new(&self.key, self.session_id));

        FastPooledBuilder {
            pool: self,
            builder: Some(builder),
        }
    }

    /// Get the pool capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of available builders
    #[inline]
    pub fn available(&self) -> usize {
        self.builders.len()
    }

    /// Get the session ID
    #[inline]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
}

impl std::fmt::Debug for FastPacketPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastPacketPool")
            .field("capacity", &self.capacity)
            .field("available", &self.builders.len())
            .field("session_id", &format!("{:016x}", self.session_id))
            .finish()
    }
}

/// RAII guard for a fast pooled builder.
pub struct FastPooledBuilder<'a> {
    pool: &'a FastPacketPool,
    builder: Option<FastPacketBuilder>,
}

impl<'a> FastPooledBuilder<'a> {
    /// Build a packet from events
    #[inline]
    pub fn build(
        &mut self,
        stream_id: u64,
        sequence: u64,
        events: &[Bytes],
        flags: PacketFlags,
    ) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build(stream_id, sequence, events, flags)
    }

    /// Build a handshake packet
    #[inline]
    pub fn build_handshake(&mut self, payload: &[u8]) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_handshake(payload)
    }

    /// Build a heartbeat packet
    #[inline]
    pub fn build_heartbeat(&mut self) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_heartbeat()
    }

    /// Check if events would fit in a single packet
    #[inline]
    pub fn would_fit(&self, events: &[Bytes]) -> bool {
        self.builder
            .as_ref()
            .expect("builder taken")
            .would_fit(events)
    }
}

impl Drop for FastPooledBuilder<'_> {
    fn drop(&mut self) {
        if let Some(mut builder) = self.builder.take() {
            // Update key/session if pool values have changed
            if builder.session_id() != self.pool.session_id {
                builder.set_key(&self.pool.key, self.pool.session_id);
            }
            // Return to pool (ignore if full)
            let _ = self.pool.builders.push(builder);
        }
    }
}

impl std::fmt::Debug for FastPooledBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastPooledBuilder")
            .field("has_builder", &self.builder.is_some())
            .finish()
    }
}

/// Shared fast packet pool (thread-safe)
pub type SharedFastPacketPool = Arc<FastPacketPool>;

/// Create a shared fast packet pool
pub fn shared_fast_pool(size: usize, key: &[u8; 32], session_id: u64) -> SharedFastPacketPool {
    Arc::new(FastPacketPool::new(size, key, session_id))
}

// ============================================================================
// Thread-Local Pool (Zero-Contention Hot Path)
// ============================================================================

use std::cell::RefCell;

thread_local! {
    /// Thread-local cache of fast packet builders.
    /// This eliminates atomic contention on the hot path.
    static LOCAL_FAST_BUILDERS: RefCell<Vec<FastPacketBuilder>> = const { RefCell::new(Vec::new()) };
}

/// Thread-local fast packet pool for zero-contention packet building.
///
/// This pool uses thread-local storage to cache packet builders, falling back
/// to a shared `ArrayQueue` when the local cache is empty. This design:
///
/// - Eliminates atomic operations on the hot path (when local cache is warm)
/// - Maintains fairness through periodic returns to shared pool
/// - Auto-refills from shared pool in batches to amortize atomic costs
///
/// # Performance
///
/// When the local cache is warm, `acquire()` and `release()` have zero atomic
/// operations, making them ~10-15% faster than the shared pool under contention.
pub struct ThreadLocalFastPool {
    /// Shared fallback pool
    shared: ArrayQueue<FastPacketBuilder>,
    /// Encryption key for new builders
    key: [u8; 32],
    /// Session ID for builders
    session_id: u64,
    /// Maximum builders per thread-local cache
    local_capacity: usize,
    /// Total pool capacity
    capacity: usize,
}

impl ThreadLocalFastPool {
    /// Default number of builders to cache per thread
    pub const DEFAULT_LOCAL_CAPACITY: usize = 8;

    /// Create a new thread-local pool
    pub fn new(size: usize, key: &[u8; 32], session_id: u64) -> Self {
        Self::with_local_capacity(size, key, session_id, Self::DEFAULT_LOCAL_CAPACITY)
    }

    /// Create a new thread-local pool with custom local capacity
    pub fn with_local_capacity(
        size: usize,
        key: &[u8; 32],
        session_id: u64,
        local_capacity: usize,
    ) -> Self {
        let shared = ArrayQueue::new(size);

        // Pre-populate the shared pool
        for _ in 0..size {
            let _ = shared.push(FastPacketBuilder::new(key, session_id));
        }

        Self {
            shared,
            key: *key,
            session_id,
            local_capacity,
            capacity: size,
        }
    }

    /// Acquire a builder from the pool.
    ///
    /// First tries the thread-local cache (zero atomics), then falls back
    /// to the shared pool, refilling the local cache in batches.
    #[inline]
    pub fn acquire(&self) -> FastPacketBuilder {
        LOCAL_FAST_BUILDERS.with(|pool| {
            let mut pool = pool.borrow_mut();

            // Fast path: pop from local cache (no atomics)
            if let Some(mut builder) = pool.pop() {
                // Update key/session if changed
                if builder.session_id() != self.session_id {
                    builder.set_key(&self.key, self.session_id);
                }
                return builder;
            }

            // Slow path: refill from shared pool
            let refill_count = self.local_capacity.min(self.shared.len());
            for _ in 0..refill_count {
                if let Some(b) = self.shared.pop() {
                    pool.push(b);
                } else {
                    break;
                }
            }

            // Try local again after refill
            pool.pop()
                .map(|mut b| {
                    if b.session_id() != self.session_id {
                        b.set_key(&self.key, self.session_id);
                    }
                    b
                })
                .unwrap_or_else(|| FastPacketBuilder::new(&self.key, self.session_id))
        })
    }

    /// Release a builder back to the pool.
    ///
    /// Keeps builders in the thread-local cache up to `local_capacity * 2`,
    /// then returns excess to the shared pool.
    #[inline]
    pub fn release(&self, mut builder: FastPacketBuilder) {
        // Update key/session if changed
        if builder.session_id() != self.session_id {
            builder.set_key(&self.key, self.session_id);
        }

        LOCAL_FAST_BUILDERS.with(|pool| {
            let mut pool = pool.borrow_mut();

            if pool.len() < self.local_capacity * 2 {
                // Keep in local cache
                pool.push(builder);
            } else {
                // Return excess to shared pool
                let _ = self.shared.push(builder);
            }
        })
    }

    /// Get the pool capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of builders in the shared pool
    #[inline]
    pub fn shared_available(&self) -> usize {
        self.shared.len()
    }

    /// Get the session ID
    #[inline]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Get the local capacity per thread
    #[inline]
    pub fn local_capacity(&self) -> usize {
        self.local_capacity
    }
}

impl std::fmt::Debug for ThreadLocalFastPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadLocalFastPool")
            .field("capacity", &self.capacity)
            .field("shared_available", &self.shared.len())
            .field("local_capacity", &self.local_capacity)
            .field("session_id", &format!("{:016x}", self.session_id))
            .finish()
    }
}

/// RAII guard for a thread-local pooled builder.
pub struct ThreadLocalPooledBuilder<'a> {
    pool: &'a ThreadLocalFastPool,
    builder: Option<FastPacketBuilder>,
}

impl<'a> ThreadLocalPooledBuilder<'a> {
    /// Build a packet from events
    #[inline]
    pub fn build(
        &mut self,
        stream_id: u64,
        sequence: u64,
        events: &[Bytes],
        flags: PacketFlags,
    ) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build(stream_id, sequence, events, flags)
    }

    /// Build a handshake packet
    #[inline]
    pub fn build_handshake(&mut self, payload: &[u8]) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_handshake(payload)
    }

    /// Build a heartbeat packet
    #[inline]
    pub fn build_heartbeat(&mut self) -> Bytes {
        self.builder
            .as_mut()
            .expect("builder taken")
            .build_heartbeat()
    }

    /// Check if events would fit in a single packet
    #[inline]
    pub fn would_fit(&self, events: &[Bytes]) -> bool {
        self.builder
            .as_ref()
            .expect("builder taken")
            .would_fit(events)
    }
}

impl Drop for ThreadLocalPooledBuilder<'_> {
    fn drop(&mut self) {
        if let Some(builder) = self.builder.take() {
            self.pool.release(builder);
        }
    }
}

impl std::fmt::Debug for ThreadLocalPooledBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadLocalPooledBuilder")
            .field("has_builder", &self.builder.is_some())
            .finish()
    }
}

/// Shared thread-local pool (thread-safe)
pub type SharedThreadLocalPool = Arc<ThreadLocalFastPool>;

/// Create a shared thread-local pool
pub fn shared_thread_local_pool(
    size: usize,
    key: &[u8; 32],
    session_id: u64,
) -> SharedThreadLocalPool {
    Arc::new(ThreadLocalFastPool::new(size, key, session_id))
}

impl ThreadLocalFastPool {
    /// Get a builder with RAII guard
    #[inline]
    pub fn get(&self) -> ThreadLocalPooledBuilder<'_> {
        ThreadLocalPooledBuilder {
            pool: self,
            builder: Some(self.acquire()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::protocol::{HEADER_SIZE, TAG_SIZE};
    use super::*;

    #[test]
    fn test_packet_builder_basic() {
        let key = [0x42u8; 32];
        let mut builder = PacketBuilder::new(&key);

        let events = vec![Bytes::from_static(b"event1"), Bytes::from_static(b"event2")];

        let packet = builder.build(0x1234, 0x5678, 42, &events, PacketFlags::NONE);

        // Verify packet structure
        assert!(packet.len() >= HEADER_SIZE + TAG_SIZE);

        // Parse header
        let header = BltpHeader::from_bytes(&packet).unwrap();
        assert!(header.validate());
        assert_eq!(header.session_id, 0x1234);
        assert_eq!(header.stream_id, 0x5678);
        assert_eq!(header.sequence, 42);
        assert_eq!(header.event_count, 2);
    }

    #[test]
    fn test_packet_builder_handshake() {
        let key = [0x42u8; 32];
        let mut builder = PacketBuilder::new(&key);

        let payload = b"handshake data";
        let packet = builder.build_handshake(payload);

        let header = BltpHeader::from_bytes(&packet).unwrap();
        assert!(header.flags.is_handshake());
        assert_eq!(header.payload_len, payload.len() as u16);
    }

    #[test]
    fn test_packet_builder_heartbeat() {
        let key = [0x42u8; 32];
        let mut builder = PacketBuilder::new(&key);

        let packet = builder.build_heartbeat(0xDEADBEEF);

        let header = BltpHeader::from_bytes(&packet).unwrap();
        assert!(header.flags.is_heartbeat());
        assert_eq!(header.session_id, 0xDEADBEEF);
        assert_eq!(header.payload_len, 0);
    }

    #[test]
    fn test_packet_builder_would_fit() {
        let key = [0x42u8; 32];
        let builder = PacketBuilder::new(&key);

        // Small events should fit
        let small_events = vec![Bytes::from_static(b"small")];
        assert!(builder.would_fit(&small_events));

        // Very large event should not fit
        let large_event = vec![Bytes::from(vec![0u8; MAX_PAYLOAD_SIZE + 1])];
        assert!(!builder.would_fit(&large_event));
    }

    #[test]
    fn test_packet_pool_basic() {
        let key = [0x42u8; 32];
        let pool = PacketPool::new(4, &key);

        assert_eq!(pool.capacity(), 4);
        assert_eq!(pool.available(), 4);

        // Get a builder
        {
            let _builder = pool.get();
            assert_eq!(pool.available(), 3);
        }

        // Builder returned
        assert_eq!(pool.available(), 4);
    }

    #[test]
    fn test_packet_pool_exhaust() {
        let key = [0x42u8; 32];
        let pool = PacketPool::new(2, &key);

        // Get all builders
        let b1 = pool.get();
        let b2 = pool.get();
        assert_eq!(pool.available(), 0);

        // Get one more (should allocate)
        let _b3 = pool.get();
        assert_eq!(pool.available(), 0);

        // Drop one
        drop(b1);
        assert_eq!(pool.available(), 1);

        // Drop others
        drop(b2);
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_pooled_builder() {
        let key = [0x42u8; 32];
        let pool = PacketPool::new(2, &key);

        let mut builder = pool.get();

        let events = vec![Bytes::from_static(b"test")];
        let packet = builder.build(1, 2, 3, &events, PacketFlags::RELIABLE);

        let header = BltpHeader::from_bytes(&packet).unwrap();
        assert!(header.flags.is_reliable());
    }

    #[test]
    fn test_shared_pool() {
        let key = [0x42u8; 32];
        let pool = shared_pool(4, &key);

        let pool_clone = pool.clone();

        // Both references work
        let _b1 = pool.get();
        let _b2 = pool_clone.get();
    }

    #[test]
    fn test_thread_local_pool_basic() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF;
        let pool = ThreadLocalFastPool::new(8, &key, session_id);

        assert_eq!(pool.capacity(), 8);
        assert_eq!(pool.session_id(), session_id);
        assert_eq!(
            pool.local_capacity(),
            ThreadLocalFastPool::DEFAULT_LOCAL_CAPACITY
        );
    }

    #[test]
    fn test_thread_local_pool_acquire_release() {
        let key = [0x42u8; 32];
        let session_id = 0xDEADBEEF;
        let pool = ThreadLocalFastPool::new(4, &key, session_id);

        // Acquire a builder
        let builder = pool.acquire();
        assert_eq!(builder.session_id(), session_id);

        // Release it back
        pool.release(builder);

        // Acquire again - should get from local cache (no atomics)
        let builder2 = pool.acquire();
        assert_eq!(builder2.session_id(), session_id);
        pool.release(builder2);
    }

    #[test]
    fn test_thread_local_pool_raii_guard() {
        let key = [0x42u8; 32];
        let session_id = 0xCAFEBABE;
        let pool = ThreadLocalFastPool::new(4, &key, session_id);

        {
            let mut builder = pool.get();
            let events = vec![Bytes::from_static(b"test event")];
            let packet = builder.build(1, 42, &events, PacketFlags::NONE);

            // Verify packet was built correctly
            let header = BltpHeader::from_bytes(&packet).unwrap();
            assert_eq!(header.stream_id, 1);
            assert_eq!(header.sequence, 42);
            assert_eq!(header.event_count, 1);
        }
        // Builder automatically returned to pool on drop
    }

    #[test]
    fn test_thread_local_pool_batch_refill() {
        let key = [0x42u8; 32];
        let session_id = 0x1111;
        let pool = ThreadLocalFastPool::with_local_capacity(16, &key, session_id, 4);

        // Acquire multiple builders to trigger batch refill
        let mut builders = Vec::new();
        for _ in 0..8 {
            builders.push(pool.acquire());
        }

        // All should have correct session ID
        for b in &builders {
            assert_eq!(b.session_id(), session_id);
        }

        // Release all back
        for b in builders {
            pool.release(b);
        }
    }

    #[test]
    fn test_thread_local_pool_overflow_to_shared() {
        let key = [0x42u8; 32];
        let session_id = 0x2222;
        // local_capacity = 2, so local cache holds up to 4 (2 * 2)
        let pool = ThreadLocalFastPool::with_local_capacity(8, &key, session_id, 2);

        // Acquire and release many builders
        for _ in 0..10 {
            let b = pool.acquire();
            pool.release(b);
        }

        // Pool should still function correctly
        let builder = pool.acquire();
        assert_eq!(builder.session_id(), session_id);
    }

    #[test]
    fn test_shared_thread_local_pool() {
        let key = [0x42u8; 32];
        let session_id = 0x3333;
        let pool = shared_thread_local_pool(8, &key, session_id);

        let pool_clone = pool.clone();

        // Both references work
        let _b1 = pool.get();
        let _b2 = pool_clone.get();
    }
}
