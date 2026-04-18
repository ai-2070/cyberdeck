//! Session and stream state management for Net.
//!
//! This module manages session state after Noise handshake completion,
//! including per-stream state for multiplexing.

use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::event::StoredEvent;

use super::crypto::{PacketCipher, SessionKeys};
use super::pool::{SharedLocalPool, SharedPacketPool};
use super::reliability::{create_reliability_mode, ReliabilityMode};

/// Session state after handshake completion.
pub struct NetSession {
    /// Session ID (derived from handshake)
    session_id: u64,
    /// Remote peer address
    peer_addr: SocketAddr,
    /// TX cipher (ChaCha20-Poly1305 with counter-based nonces)
    tx_cipher: PacketCipher,
    /// RX cipher (ChaCha20-Poly1305 with counter-based nonces)
    rx_cipher: PacketCipher,
    /// Per-stream state
    streams: DashMap<u64, StreamState>,
    /// Last activity timestamp (for session timeout)
    last_activity: AtomicU64,
    /// Packet pool for zero-allocation building
    packet_pool: SharedPacketPool,
    /// Thread-local pool for zero-contention hot path
    thread_local_pool: SharedLocalPool,
    /// Default reliability mode for new streams
    default_reliable: bool,
    /// Session is active
    active: AtomicBool,
}

impl NetSession {
    /// Create a new session from handshake results
    pub fn new(
        keys: SessionKeys,
        peer_addr: SocketAddr,
        pool_size: usize,
        default_reliable: bool,
    ) -> Self {
        let tx_cipher = PacketCipher::new(&keys.tx_key, keys.session_id);
        let rx_cipher = PacketCipher::new(&keys.rx_key, keys.session_id);

        let packet_pool = super::pool::shared_pool(pool_size, &keys.tx_key, keys.session_id);
        let thread_local_pool =
            super::pool::shared_local_pool(pool_size, &keys.tx_key, keys.session_id);

        Self {
            session_id: keys.session_id,
            peer_addr,
            tx_cipher,
            rx_cipher,
            streams: DashMap::new(),
            last_activity: AtomicU64::new(current_timestamp()),
            packet_pool,
            thread_local_pool,
            default_reliable,
            active: AtomicBool::new(true),
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

    /// Get the TX cipher
    #[inline]
    pub fn tx_cipher(&self) -> &PacketCipher {
        &self.tx_cipher
    }

    /// Get the RX cipher
    #[inline]
    pub fn rx_cipher(&self) -> &PacketCipher {
        &self.rx_cipher
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

    /// Look up stream state without creating it. Returns `None` if the
    /// stream was never opened or has been closed.
    pub fn try_stream(
        &self,
        stream_id: u64,
    ) -> Option<dashmap::mapref::one::Ref<'_, u64, StreamState>> {
        self.streams.get(&stream_id)
    }

    /// Open a stream with an explicit reliability mode and fair-scheduler
    /// weight.
    ///
    /// Idempotent: if the stream already exists, this is a no-op and the
    /// caller's config is **ignored with a warning log** — the first open
    /// wins. Callers that want to change a stream's config must close +
    /// re-open it.
    pub fn open_stream_with(&self, stream_id: u64, reliable: bool, fairness_weight: u8) {
        self.open_stream_full(stream_id, reliable, fairness_weight, 0);
    }

    /// Extended open that also sets the per-stream TX window for
    /// backpressure. `tx_window == 0` keeps the pre-backpressure
    /// behavior (unbounded local queue).
    pub fn open_stream_full(
        &self,
        stream_id: u64,
        reliable: bool,
        fairness_weight: u8,
        tx_window: u32,
    ) {
        use dashmap::mapref::entry::Entry;
        match self.streams.entry(stream_id) {
            Entry::Occupied(existing) => {
                let existing = existing.get();
                if existing.reliable_mode() != reliable
                    || existing.fairness_weight() != fairness_weight.max(1)
                    || existing.tx_window() != tx_window
                {
                    tracing::warn!(
                        stream_id = format!("{:#x}", stream_id),
                        existing_reliable = existing.reliable_mode(),
                        new_reliable = reliable,
                        existing_weight = existing.fairness_weight(),
                        new_weight = fairness_weight,
                        existing_tx_window = existing.tx_window(),
                        new_tx_window = tx_window,
                        "open_stream: ignoring conflicting config; first open wins"
                    );
                }
            }
            Entry::Vacant(v) => {
                v.insert(StreamState::new_full(reliable, fairness_weight, tx_window));
            }
        }
    }

    /// Close a stream: mark it inactive and remove its state.
    ///
    /// Idempotent — closing a non-existent stream is a no-op. After
    /// close, a subsequent `open_stream_with` creates a fresh stream.
    pub fn close_stream(&self, stream_id: u64) {
        if let Some((_, state)) = self.streams.remove(&stream_id) {
            state.deactivate();
        }
    }

    /// Remove streams whose `last_activity` is older than `max_idle`,
    /// keeping the active count at or below `max_streams` by LRU-evicting
    /// the oldest if still over cap. Returns the number of streams
    /// evicted. Called from the session owner's heartbeat loop.
    pub fn evict_idle_streams(
        &self,
        max_idle: Duration,
        max_streams: usize,
        reason_tag: &'static str,
    ) -> usize {
        let mut evicted = 0;
        let now = current_timestamp();
        let max_idle_ns = u64::try_from(max_idle.as_nanos()).unwrap_or(u64::MAX);

        // Pass 1: drop idle streams.
        let idle: Vec<u64> = self
            .streams
            .iter()
            .filter(|e| now.saturating_sub(e.value().last_activity_ns()) > max_idle_ns)
            .map(|e| *e.key())
            .collect();
        for sid in idle {
            if let Some((_, state)) = self.streams.remove(&sid) {
                state.deactivate();
                evicted += 1;
                tracing::debug!(
                    stream_id = format!("{:#x}", sid),
                    reason = reason_tag,
                    "stream evicted: idle timeout"
                );
            }
        }

        // Pass 2: if still over the cap, LRU-evict the oldest.
        while self.streams.len() > max_streams {
            let oldest = self
                .streams
                .iter()
                .min_by_key(|e| e.value().last_activity_ns())
                .map(|e| *e.key());
            match oldest {
                Some(sid) => {
                    if let Some((_, state)) = self.streams.remove(&sid) {
                        state.deactivate();
                        evicted += 1;
                        tracing::warn!(
                            stream_id = format!("{:#x}", sid),
                            reason = "cap_exceeded",
                            total_streams = self.streams.len(),
                            max_streams = max_streams,
                            "stream evicted: max_streams cap"
                        );
                    }
                }
                None => break,
            }
        }

        evicted
    }

    /// Get stream state (read-only)
    pub fn get_stream(
        &self,
        stream_id: u64,
    ) -> Option<dashmap::mapref::one::Ref<'_, u64, StreamState>> {
        self.streams.get(&stream_id)
    }

    /// Get the packet pool
    #[inline]
    pub fn packet_pool(&self) -> &SharedPacketPool {
        &self.packet_pool
    }

    /// Get the thread-local pool for zero-contention packet building
    #[inline]
    pub fn thread_local_pool(&self) -> &SharedLocalPool {
        &self.thread_local_pool
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
        let timeout_ns = u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX);
        now.saturating_sub(last) > timeout_ns
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

impl std::fmt::Debug for NetSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetSession")
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
    /// Nanoseconds since epoch of the last activity (send or receive).
    /// Used by the session's idle-eviction sweep.
    last_activity: AtomicU64,
    /// Reliability mode this stream was created with. Stored so
    /// `open_stream` can warn when a caller re-opens with a different
    /// config (config is immutable for the stream's lifetime).
    reliable_mode: bool,
    /// Fair-scheduler quantum multiplier (1 = equal share).
    fairness_weight: u8,
    /// Maximum concurrent in-flight packets for this stream's send
    /// path before `send_on_stream` returns `StreamError::Backpressure`.
    /// v1 semantics: counts **packets**, not bytes. Named for forward
    /// compatibility with a future byte-accounted credit-window swap.
    /// `0` means "no limit" — the pre-backpressure behavior.
    tx_window: u32,
    /// Current in-flight packets. Incremented before a socket send,
    /// decremented after (success or failure). Compared against
    /// `tx_window` to decide whether to admit the next send.
    tx_inflight: AtomicU32,
    /// Number of `send_on_stream` calls that have returned
    /// `StreamError::Backpressure` since this stream was opened.
    backpressure_events: AtomicU64,
}

impl StreamState {
    /// Create a new stream state
    pub fn new(reliable: bool) -> Self {
        Self::new_with_weight(reliable, 1)
    }

    /// Create a new stream state with a fair-scheduler weight.
    pub fn new_with_weight(reliable: bool, fairness_weight: u8) -> Self {
        Self::new_full(reliable, fairness_weight, 0)
    }

    /// Create a new stream state with full config (weight + tx window).
    pub fn new_full(reliable: bool, fairness_weight: u8, tx_window: u32) -> Self {
        Self {
            tx_seq: AtomicU64::new(0),
            rx_seq: AtomicU64::new(0),
            reliability: parking_lot::Mutex::new(create_reliability_mode(reliable)),
            inbound: SegQueue::new(),
            active: AtomicBool::new(true),
            last_activity: AtomicU64::new(current_timestamp()),
            reliable_mode: reliable,
            fairness_weight: fairness_weight.max(1),
            tx_window,
            tx_inflight: AtomicU32::new(0),
            backpressure_events: AtomicU64::new(0),
        }
    }

    /// Refresh last-activity timestamp. Called on every send and on
    /// every receive that lands packets/events into the stream.
    #[inline]
    pub fn touch(&self) {
        self.last_activity
            .store(current_timestamp(), Ordering::Release);
    }

    /// Nanoseconds since epoch of the last activity.
    #[inline]
    pub fn last_activity_ns(&self) -> u64 {
        self.last_activity.load(Ordering::Acquire)
    }

    /// Reliability mode this stream was created with.
    #[inline]
    pub fn reliable_mode(&self) -> bool {
        self.reliable_mode
    }

    /// Fair-scheduler weight for this stream.
    #[inline]
    pub fn fairness_weight(&self) -> u8 {
        self.fairness_weight
    }

    /// Configured max in-flight packets before Backpressure. `0` = no limit.
    #[inline]
    pub fn tx_window(&self) -> u32 {
        self.tx_window
    }

    /// Current in-flight packet count.
    #[inline]
    pub fn tx_inflight(&self) -> u32 {
        self.tx_inflight.load(Ordering::Acquire)
    }

    /// Cumulative number of Backpressure rejections since the stream opened.
    #[inline]
    pub fn backpressure_events(&self) -> u64 {
        self.backpressure_events.load(Ordering::Relaxed)
    }

    /// Try to acquire a TX slot. Returns `true` on success (and increments
    /// the in-flight counter); returns `false` when the window is full —
    /// caller is expected to return `StreamError::Backpressure`.
    ///
    /// A `tx_window` of 0 is "unbounded" and always admits.
    pub fn try_acquire_tx_slot(&self) -> bool {
        if self.tx_window == 0 {
            self.tx_inflight.fetch_add(1, Ordering::AcqRel);
            return true;
        }
        // CAS loop: admit iff current < window.
        loop {
            let cur = self.tx_inflight.load(Ordering::Acquire);
            if cur >= self.tx_window {
                self.backpressure_events.fetch_add(1, Ordering::Relaxed);
                return false;
            }
            if self
                .tx_inflight
                .compare_exchange_weak(cur, cur + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Release a TX slot after the socket send completes. Safe against
    /// under-flow via saturating decrement.
    pub fn release_tx_slot(&self) {
        // Saturating: if something ever decrements without a paired
        // acquire (shouldn't happen, but defensive), we don't wrap.
        let prev = self
            .tx_inflight
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| {
                Some(v.saturating_sub(1))
            });
        debug_assert!(prev.is_ok());
    }

    /// Get and increment the TX sequence number. Refreshes `last_activity`.
    #[inline]
    pub fn next_tx_seq(&self) -> u64 {
        self.touch();
        self.tx_seq.fetch_add(1, Ordering::Relaxed)
    }

    /// Get the current TX sequence number
    #[inline]
    pub fn current_tx_seq(&self) -> u64 {
        self.tx_seq.load(Ordering::Relaxed)
    }

    /// Update the RX sequence number. Refreshes `last_activity`.
    #[inline]
    pub fn update_rx_seq(&self, seq: u64) {
        self.touch();
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
    session: parking_lot::RwLock<Option<Arc<NetSession>>>,
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
    pub fn set_session(&self, session: NetSession) {
        let mut guard = self.session.write();
        *guard = Some(Arc::new(session));
    }

    /// Set the current session from an existing Arc
    pub fn set_session_arc(&self, session: Arc<NetSession>) {
        let mut guard = self.session.write();
        *guard = Some(session);
    }

    /// Get the current session
    pub fn get_session(&self) -> Option<Arc<NetSession>> {
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

use super::current_timestamp;

#[cfg(test)]
mod tests {
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
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = NetSession::new(keys.clone(), peer_addr, 4, false);

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
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = NetSession::new(keys, peer_addr, 4, false);

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
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        let session = NetSession::new(keys, peer_addr, 4, false);

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
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = NetSession::new(keys, peer_addr, 4, false);

        manager.set_session(session);
        assert!(manager.has_session());

        let retrieved = manager.get_session().unwrap();
        assert!(retrieved.is_active());

        manager.clear_session();
        assert!(!manager.has_session());
    }

    #[test]
    fn test_open_stream_with_idempotent() {
        let keys = test_keys();
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = NetSession::new(keys, peer_addr, 4, false);

        // First open creates state.
        session.open_stream_with(42, true, 3);
        assert_eq!(session.stream_count(), 1);
        let state = session.get_stream(42).unwrap();
        assert!(state.reliable_mode());
        assert_eq!(state.fairness_weight(), 3);
        drop(state);

        // Second open with matching config is a no-op.
        session.open_stream_with(42, true, 3);
        assert_eq!(session.stream_count(), 1);

        // Second open with DIFFERENT config is also a no-op
        // (first open wins). We log a warning but don't mutate.
        session.open_stream_with(42, false, 7);
        let state = session.get_stream(42).unwrap();
        assert!(
            state.reliable_mode(),
            "first open wins — reliable still true"
        );
        assert_eq!(
            state.fairness_weight(),
            3,
            "first open wins — weight still 3"
        );
    }

    #[test]
    fn test_close_stream_removes_state() {
        let keys = test_keys();
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = NetSession::new(keys, peer_addr, 4, false);

        session.open_stream_with(1, false, 1);
        session.open_stream_with(2, true, 2);
        assert_eq!(session.stream_count(), 2);

        session.close_stream(1);
        assert_eq!(session.stream_count(), 1);
        assert!(session.get_stream(1).is_none());
        assert!(session.get_stream(2).is_some());

        // Closing a non-existent stream is a no-op.
        session.close_stream(99);
        assert_eq!(session.stream_count(), 1);

        // Re-open after close creates fresh state with new config.
        session.close_stream(2);
        session.open_stream_with(2, false, 5);
        let state = session.get_stream(2).unwrap();
        assert!(!state.reliable_mode());
        assert_eq!(state.fairness_weight(), 5);
    }

    #[test]
    fn test_evict_idle_streams_timeout_and_cap() {
        let keys = test_keys();
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = NetSession::new(keys, peer_addr, 4, false);

        // Open three streams; touch only one so the other two look idle.
        session.open_stream_with(1, false, 1);
        session.open_stream_with(2, false, 1);
        session.open_stream_with(3, false, 1);
        std::thread::sleep(Duration::from_millis(10));
        session.get_or_create_stream(2).touch();

        // With a tight idle timeout, streams 1 and 3 should be evicted;
        // stream 2 was just touched so it survives.
        let evicted = session.evict_idle_streams(Duration::from_millis(5), usize::MAX, "test");
        assert_eq!(evicted, 2);
        assert_eq!(session.stream_count(), 1);
        assert!(session.get_stream(2).is_some());

        // Cap eviction: open two more streams so we have 3, then cap at 1.
        session.open_stream_with(4, false, 1);
        session.open_stream_with(5, false, 1);
        assert_eq!(session.stream_count(), 3);
        let evicted = session.evict_idle_streams(Duration::from_nanos(u64::MAX), 1, "test");
        assert_eq!(evicted, 2);
        assert_eq!(session.stream_count(), 1);
    }

    #[test]
    fn test_session_manager_arc_shares_touch_updates() {
        let manager = SessionManager::new(Duration::from_millis(50));

        let keys = test_keys();
        let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let session = Arc::new(NetSession::new(keys, peer_addr, 4, false));

        manager.set_session_arc(session.clone());

        std::thread::sleep(Duration::from_millis(30));
        session.touch();

        assert!(
            manager.check_session(),
            "session should be healthy because touch() updated the shared Arc"
        );

        std::thread::sleep(Duration::from_millis(60));
        assert!(
            !manager.check_session(),
            "session should have timed out after 60ms with no touch"
        );
    }

    #[test]
    fn test_stream_state_tx_window_trips_backpressure() {
        let state = StreamState::new_full(false, 1, 2);
        assert!(state.try_acquire_tx_slot(), "first acquire under window");
        assert!(state.try_acquire_tx_slot(), "second acquire at window edge");
        assert!(
            !state.try_acquire_tx_slot(),
            "third acquire must be refused — window full"
        );
        assert_eq!(state.backpressure_events(), 1);
        assert_eq!(state.tx_inflight(), 2);
    }

    #[test]
    fn test_stream_state_tx_window_releases_on_send_completion() {
        let state = StreamState::new_full(false, 1, 1);
        assert!(state.try_acquire_tx_slot());
        assert!(!state.try_acquire_tx_slot());

        state.release_tx_slot();
        assert_eq!(state.tx_inflight(), 0);
        assert!(
            state.try_acquire_tx_slot(),
            "acquire succeeds after the prior slot is released"
        );
    }

    #[test]
    fn test_stream_state_tx_window_zero_is_unbounded() {
        let state = StreamState::new_full(false, 1, 0);
        // Burst a large number of acquires; none should refuse when
        // the window is 0 (pre-backpressure behavior).
        for _ in 0..10_000 {
            assert!(state.try_acquire_tx_slot());
        }
        assert_eq!(state.backpressure_events(), 0);
    }

    #[test]
    fn test_stream_state_release_saturates_at_zero() {
        // Defensive: a stray release without a paired acquire must not
        // underflow. A subsequent acquire stays within the window.
        let state = StreamState::new_full(false, 1, 1);
        state.release_tx_slot();
        state.release_tx_slot();
        assert_eq!(state.tx_inflight(), 0);
        assert!(state.try_acquire_tx_slot());
    }
}
