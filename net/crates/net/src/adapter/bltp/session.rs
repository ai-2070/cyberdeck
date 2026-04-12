//! Session and stream state management for BLTP.
//!
//! This module manages session state after Noise handshake completion,
//! including per-stream state for multiplexing.

use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::event::StoredEvent;

use super::crypto::{FastPacketCipher, PacketCipher, SessionKeys};
use super::pool::{SharedFastPacketPool, SharedPacketPool, SharedThreadLocalPool};
use super::reliability::{create_reliability_mode, ReliabilityMode};

/// Cipher mode for the session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CipherMode {
    /// Fast mode using ChaCha20-Poly1305 with counter-based nonces (default)
    /// ~20-40% faster than legacy mode
    #[default]
    Fast,
    /// Legacy mode using XChaCha20-Poly1305 with random nonces
    /// For backward compatibility with older peers
    Legacy,
}

/// Session ciphers - either fast (counter-based) or legacy (random nonce)
enum SessionCiphers {
    Fast {
        tx: FastPacketCipher,
        rx: FastPacketCipher,
    },
    Legacy {
        tx: PacketCipher,
        rx: PacketCipher,
    },
}

/// Session state after handshake completion.
pub struct BltpSession {
    /// Session ID (derived from handshake)
    session_id: u64,
    /// Remote peer address
    peer_addr: SocketAddr,
    /// Session ciphers (fast or legacy mode)
    ciphers: SessionCiphers,
    /// Per-stream state
    streams: DashMap<u64, StreamState>,
    /// Last activity timestamp (for session timeout)
    last_activity: AtomicU64,
    /// Legacy packet pool for zero-allocation building (used in legacy mode)
    packet_pool: SharedPacketPool,
    /// Fast packet pool for counter-based encryption (used in fast mode)
    fast_packet_pool: Option<SharedFastPacketPool>,
    /// Thread-local fast pool for zero-contention hot path (optional)
    thread_local_pool: Option<SharedThreadLocalPool>,
    /// Default reliability mode for new streams
    default_reliable: bool,
    /// Session is active
    active: AtomicBool,
    /// Cipher mode in use
    cipher_mode: CipherMode,
}

impl BltpSession {
    /// Create a new session from handshake results (uses fast cipher mode by default)
    pub fn new(
        keys: SessionKeys,
        peer_addr: SocketAddr,
        packet_pool: SharedPacketPool,
        default_reliable: bool,
    ) -> Self {
        Self::with_cipher_mode(
            keys,
            peer_addr,
            packet_pool,
            default_reliable,
            CipherMode::Fast,
        )
    }

    /// Create a new session with specified cipher mode
    pub fn with_cipher_mode(
        keys: SessionKeys,
        peer_addr: SocketAddr,
        packet_pool: SharedPacketPool,
        default_reliable: bool,
        cipher_mode: CipherMode,
    ) -> Self {
        let ciphers = match cipher_mode {
            CipherMode::Fast => SessionCiphers::Fast {
                tx: FastPacketCipher::new(&keys.tx_key, keys.session_id),
                rx: FastPacketCipher::new(&keys.rx_key, keys.session_id),
            },
            CipherMode::Legacy => SessionCiphers::Legacy {
                tx: PacketCipher::new(&keys.tx_key),
                rx: PacketCipher::new(&keys.rx_key),
            },
        };

        // Create fast packet pool for fast cipher mode
        let pool_size = packet_pool.capacity();
        let (fast_packet_pool, thread_local_pool) = match cipher_mode {
            CipherMode::Fast => {
                // Create both fast pool and thread-local pool for maximum performance
                let fast_pool = Some(super::pool::shared_fast_pool(
                    pool_size,
                    &keys.tx_key,
                    keys.session_id,
                ));
                let tl_pool = Some(super::pool::shared_thread_local_pool(
                    pool_size,
                    &keys.tx_key,
                    keys.session_id,
                ));
                (fast_pool, tl_pool)
            }
            CipherMode::Legacy => (None, None),
        };

        Self {
            session_id: keys.session_id,
            peer_addr,
            ciphers,
            streams: DashMap::new(),
            last_activity: AtomicU64::new(current_timestamp()),
            packet_pool,
            fast_packet_pool,
            thread_local_pool,
            default_reliable,
            active: AtomicBool::new(true),
            cipher_mode,
        }
    }

    /// Get the session ID
    #[inline]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Get the peer address
    #[inline]
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Get the cipher mode
    #[inline]
    pub fn cipher_mode(&self) -> CipherMode {
        self.cipher_mode
    }

    /// Check if using fast cipher mode
    #[inline]
    pub fn is_fast_mode(&self) -> bool {
        self.cipher_mode == CipherMode::Fast
    }

    /// Get the fast TX cipher (panics if not in fast mode)
    #[inline]
    pub fn fast_tx_cipher(&self) -> &FastPacketCipher {
        match &self.ciphers {
            SessionCiphers::Fast { tx, .. } => tx,
            SessionCiphers::Legacy { .. } => panic!("not in fast cipher mode"),
        }
    }

    /// Get the fast RX cipher (panics if not in fast mode)
    #[inline]
    pub fn fast_rx_cipher(&self) -> &FastPacketCipher {
        match &self.ciphers {
            SessionCiphers::Fast { rx, .. } => rx,
            SessionCiphers::Legacy { .. } => panic!("not in fast cipher mode"),
        }
    }

    /// Get the legacy TX cipher (panics if not in legacy mode)
    #[inline]
    pub fn legacy_tx_cipher(&self) -> &PacketCipher {
        match &self.ciphers {
            SessionCiphers::Legacy { tx, .. } => tx,
            SessionCiphers::Fast { .. } => panic!("not in legacy cipher mode"),
        }
    }

    /// Get the legacy RX cipher (panics if not in legacy mode)
    #[inline]
    pub fn legacy_rx_cipher(&self) -> &PacketCipher {
        match &self.ciphers {
            SessionCiphers::Legacy { rx, .. } => rx,
            SessionCiphers::Fast { .. } => panic!("not in legacy cipher mode"),
        }
    }

    /// Get or create stream state
    pub fn get_or_create_stream(
        &self,
        stream_id: u64,
    ) -> dashmap::mapref::one::RefMut<'_, u64, StreamState> {
        self.streams
            .entry(stream_id)
            .or_insert_with(|| StreamState::new(self.default_reliable))
    }

    /// Get stream state (read-only)
    pub fn get_stream(
        &self,
        stream_id: u64,
    ) -> Option<dashmap::mapref::one::Ref<'_, u64, StreamState>> {
        self.streams.get(&stream_id)
    }

    /// Get the legacy packet pool
    #[inline]
    pub fn packet_pool(&self) -> &SharedPacketPool {
        &self.packet_pool
    }

    /// Get the fast packet pool (panics if not in fast mode)
    #[inline]
    pub fn fast_packet_pool(&self) -> &SharedFastPacketPool {
        self.fast_packet_pool
            .as_ref()
            .expect("fast packet pool not available in legacy mode")
    }

    /// Try to get the fast packet pool (returns None if not in fast mode)
    #[inline]
    pub fn try_fast_packet_pool(&self) -> Option<&SharedFastPacketPool> {
        self.fast_packet_pool.as_ref()
    }

    /// Get the thread-local pool (panics if not in fast mode)
    ///
    /// The thread-local pool provides zero-contention packet building on the hot path.
    #[inline]
    pub fn thread_local_pool(&self) -> &SharedThreadLocalPool {
        self.thread_local_pool
            .as_ref()
            .expect("thread-local pool not available in legacy mode")
    }

    /// Try to get the thread-local pool (returns None if not in fast mode)
    #[inline]
    pub fn try_thread_local_pool(&self) -> Option<&SharedThreadLocalPool> {
        self.thread_local_pool.as_ref()
    }

    /// Update last activity timestamp
    #[inline]
    pub fn touch(&self) {
        self.last_activity
            .store(current_timestamp(), Ordering::Release);
    }

    /// Check if session has timed out
    #[inline]
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        let last = self.last_activity.load(Ordering::Acquire);
        let now = current_timestamp();
        now.saturating_sub(last) > timeout.as_nanos() as u64
    }

    /// Check if session is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Deactivate the session
    #[inline]
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }

    /// Get all stream IDs
    pub fn stream_ids(&self) -> Vec<u64> {
        self.streams.iter().map(|r| *r.key()).collect()
    }

    /// Get the number of streams
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}

impl std::fmt::Debug for BltpSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BltpSession")
            .field("session_id", &format!("{:016x}", self.session_id))
            .field("peer_addr", &self.peer_addr)
            .field("stream_count", &self.streams.len())
            .field("active", &self.active.load(Ordering::Relaxed))
            .finish()
    }
}

/// Per-stream state for multiplexing.
pub struct StreamState {
    /// Next sequence number to send
    tx_seq: AtomicU64,
    /// Last received sequence number
    rx_seq: AtomicU64,
    /// Reliability mode for this stream
    reliability: parking_lot::Mutex<Box<dyn ReliabilityMode>>,
    /// Inbound event queue (for poll_shard)
    inbound: SegQueue<StoredEvent>,
    /// Stream is active
    active: AtomicBool,
}

impl StreamState {
    /// Create a new stream state
    pub fn new(reliable: bool) -> Self {
        Self {
            tx_seq: AtomicU64::new(0),
            rx_seq: AtomicU64::new(0),
            reliability: parking_lot::Mutex::new(create_reliability_mode(reliable)),
            inbound: SegQueue::new(),
            active: AtomicBool::new(true),
        }
    }

    /// Get and increment the TX sequence number
    #[inline]
    pub fn next_tx_seq(&self) -> u64 {
        self.tx_seq.fetch_add(1, Ordering::Relaxed)
    }

    /// Get the current TX sequence number
    #[inline]
    pub fn current_tx_seq(&self) -> u64 {
        self.tx_seq.load(Ordering::Relaxed)
    }

    /// Update the RX sequence number
    #[inline]
    pub fn update_rx_seq(&self, seq: u64) {
        self.rx_seq.fetch_max(seq, Ordering::Relaxed);
    }

    /// Get the current RX sequence number
    #[inline]
    pub fn current_rx_seq(&self) -> u64 {
        self.rx_seq.load(Ordering::Relaxed)
    }

    /// Access the reliability mode
    #[inline]
    pub fn with_reliability<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Box<dyn ReliabilityMode>) -> R,
    {
        let mut guard = self.reliability.lock();
        f(&mut guard)
    }

    /// Push an event to the inbound queue
    #[inline]
    pub fn push_event(&self, event: StoredEvent) {
        self.inbound.push(event);
    }

    /// Pop an event from the inbound queue
    #[inline]
    pub fn pop_event(&self) -> Option<StoredEvent> {
        self.inbound.pop()
    }

    /// Get the number of pending inbound events
    #[inline]
    pub fn inbound_len(&self) -> usize {
        self.inbound.len()
    }

    /// Check if stream is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Deactivate the stream
    #[inline]
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }
}

impl std::fmt::Debug for StreamState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamState")
            .field("tx_seq", &self.tx_seq.load(Ordering::Relaxed))
            .field("rx_seq", &self.rx_seq.load(Ordering::Relaxed))
            .field("inbound_len", &self.inbound.len())
            .field("active", &self.active.load(Ordering::Relaxed))
            .finish()
    }
}

/// Session manager for handling multiple sessions.
///
/// Currently supports single-peer operation, but designed for
/// future multi-peer extension.
pub struct SessionManager {
    /// Current session (single-peer mode)
    session: parking_lot::RwLock<Option<Arc<BltpSession>>>,
    /// Session timeout
    timeout: Duration,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(timeout: Duration) -> Self {
        Self {
            session: parking_lot::RwLock::new(None),
            timeout,
        }
    }

    /// Set the current session
    pub fn set_session(&self, session: BltpSession) {
        let mut guard = self.session.write();
        *guard = Some(Arc::new(session));
    }

    /// Set the current session from an existing Arc
    pub fn set_session_arc(&self, session: Arc<BltpSession>) {
        let mut guard = self.session.write();
        *guard = Some(session);
    }

    /// Get the current session
    pub fn get_session(&self) -> Option<Arc<BltpSession>> {
        self.session.read().clone()
    }

    /// Clear the current session
    pub fn clear_session(&self) {
        let mut guard = self.session.write();
        if let Some(session) = guard.take() {
            session.deactivate();
        }
    }

    /// Check if there's an active session
    pub fn has_session(&self) -> bool {
        self.session.read().is_some()
    }

    /// Check session health and clean up if timed out
    pub fn check_session(&self) -> bool {
        let guard = self.session.read();
        if let Some(session) = guard.as_ref() {
            if session.is_timed_out(self.timeout) {
                drop(guard);
                self.clear_session();
                return false;
            }
            session.is_active()
        } else {
            false
        }
    }
}

impl std::fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("has_session", &self.has_session())
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// Get current timestamp in nanoseconds
#[inline]
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::super::pool::PacketPool;
    use super::*;

    fn test_keys() -> SessionKeys {
        SessionKeys {
            tx_key: [0x42u8; 32],
            rx_key: [0x24u8; 32],
            session_id: 0x1234567890ABCDEF,
        }
    }

    #[test]
    fn test_session_creation() {
        let keys = test_keys();
        let pool = Arc::new(PacketPool::new(4, &keys.tx_key));
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = BltpSession::new(keys.clone(), peer_addr, pool, false);

        assert_eq!(session.session_id(), keys.session_id);
        assert_eq!(session.peer_addr(), peer_addr);
        assert!(session.is_active());
        assert_eq!(session.stream_count(), 0);
    }

    #[test]
    fn test_stream_state() {
        let stream = StreamState::new(false);

        // TX sequence
        assert_eq!(stream.next_tx_seq(), 0);
        assert_eq!(stream.next_tx_seq(), 1);
        assert_eq!(stream.current_tx_seq(), 2);

        // RX sequence
        stream.update_rx_seq(5);
        assert_eq!(stream.current_rx_seq(), 5);
        stream.update_rx_seq(3); // Lower value ignored
        assert_eq!(stream.current_rx_seq(), 5);

        // Inbound queue
        let event = StoredEvent::from_value("1".into(), serde_json::json!({"test": 1}), 100, 0);
        stream.push_event(event);
        assert_eq!(stream.inbound_len(), 1);

        let popped = stream.pop_event().unwrap();
        assert_eq!(popped.id, "1");
        assert_eq!(stream.inbound_len(), 0);
    }

    #[test]
    fn test_session_streams() {
        let keys = test_keys();
        let pool = Arc::new(PacketPool::new(4, &keys.tx_key));
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = BltpSession::new(keys, peer_addr, pool, false);

        // Create streams
        {
            let stream = session.get_or_create_stream(0);
            assert_eq!(stream.next_tx_seq(), 0);
        }

        {
            let stream = session.get_or_create_stream(1);
            assert_eq!(stream.next_tx_seq(), 0);
        }

        assert_eq!(session.stream_count(), 2);

        let ids = session.stream_ids();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_session_timeout() {
        let keys = test_keys();
        let pool = Arc::new(PacketPool::new(4, &keys.tx_key));
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = BltpSession::new(keys, peer_addr, pool, false);

        // Should not be timed out immediately
        assert!(!session.is_timed_out(Duration::from_secs(1)));

        // Touch and verify
        session.touch();
        assert!(!session.is_timed_out(Duration::from_secs(1)));
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new(Duration::from_secs(30));

        assert!(!manager.has_session());

        let keys = test_keys();
        let pool = Arc::new(PacketPool::new(4, &keys.tx_key));
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = BltpSession::new(keys, peer_addr, pool, false);

        manager.set_session(session);
        assert!(manager.has_session());

        let retrieved = manager.get_session().unwrap();
        assert!(retrieved.is_active());

        manager.clear_session();
        assert!(!manager.has_session());
    }

    #[test]
    fn test_session_manager_arc_shares_touch_updates() {
        // Regression: set_session() was given a dummy BltpSession with zeroed
        // keys instead of the real session Arc. The dummy never received touch()
        // updates from packet processing, causing check_session() to falsely
        // report timeouts.
        //
        // The fix uses set_session_arc() so the manager and the packet
        // processing loop share the same Arc<BltpSession>.
        let manager = SessionManager::new(Duration::from_millis(50));

        let keys = test_keys();
        let pool = Arc::new(PacketPool::new(4, &keys.tx_key));
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = Arc::new(BltpSession::new(keys, peer_addr, pool, false));

        // Use set_session_arc so manager holds the same Arc
        manager.set_session_arc(session.clone());

        // Simulate packet processing touching the session
        std::thread::sleep(Duration::from_millis(30));
        session.touch(); // This must be visible to check_session

        // Even though 30ms passed, the touch should keep the session alive
        assert!(
            manager.check_session(),
            "session should be healthy because touch() updated the shared Arc"
        );

        // Now let it truly time out
        std::thread::sleep(Duration::from_millis(60));
        assert!(
            !manager.check_session(),
            "session should have timed out after 60ms with no touch"
        );
    }
}
