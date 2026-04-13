//! Configuration for the BLTP adapter.

use std::net::SocketAddr;
use std::time::Duration;

use super::crypto::StaticKeypair;

/// Reliability configuration for BLTP streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReliabilityConfig {
    /// Fire-and-forget: no reliability, maximum throughput
    #[default]
    None,
    /// Lightweight reliability: 32-packet window, selective NACK
    Light,
    /// Full reliability: unbounded retransmit, ordered delivery
    Full,
}

impl ReliabilityConfig {
    /// Check if this mode requires acknowledgments
    #[inline]
    pub fn needs_ack(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Convert to boolean for simple reliable/unreliable distinction
    #[inline]
    pub fn is_reliable(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Role in the BLTP connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionRole {
    /// Initiator: knows responder's static public key
    Initiator,
    /// Responder: provides static keypair for authentication
    Responder,
}

/// Configuration for the BLTP adapter.
#[derive(Debug, Clone)]
pub struct BltpAdapterConfig {
    /// Local bind address
    pub bind_addr: SocketAddr,
    /// Remote peer address
    pub peer_addr: SocketAddr,
    /// Pre-shared key (32 bytes)
    pub psk: [u8; 32],
    /// Connection role (initiator or responder)
    pub role: ConnectionRole,
    /// Our static keypair (required for responder)
    pub static_keypair: Option<StaticKeypair>,
    /// Peer's static public key (required for initiator)
    pub peer_static_pubkey: Option<[u8; 32]>,
    /// Default reliability mode for new streams
    pub default_reliability: ReliabilityConfig,
    /// Packet pool size
    pub packet_pool_size: usize,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Session timeout
    pub session_timeout: Duration,
    /// Enable batched I/O (sendmmsg/recvmmsg on Linux)
    pub batched_io: bool,
    /// Maximum retries for handshake
    pub handshake_retries: usize,
    /// Handshake timeout
    pub handshake_timeout: Duration,
    /// Socket receive buffer size (None = use system default of 64MB)
    pub socket_recv_buffer: Option<usize>,
    /// Socket send buffer size (None = use system default of 64MB)
    pub socket_send_buffer: Option<usize>,
    /// Number of shards (used to map stream IDs to shard IDs on receive)
    pub num_shards: u16,
}

impl BltpAdapterConfig {
    /// Default packet pool size
    pub const DEFAULT_POOL_SIZE: usize = 64;

    /// Default heartbeat interval
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

    /// Default session timeout
    pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default handshake retries
    pub const DEFAULT_HANDSHAKE_RETRIES: usize = 3;

    /// Default handshake timeout
    pub const DEFAULT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

    /// Create a new initiator configuration.
    ///
    /// The initiator must know the responder's static public key.
    pub fn initiator(
        bind_addr: SocketAddr,
        peer_addr: SocketAddr,
        psk: [u8; 32],
        peer_static_pubkey: [u8; 32],
    ) -> Self {
        Self {
            bind_addr,
            peer_addr,
            psk,
            role: ConnectionRole::Initiator,
            static_keypair: None,
            peer_static_pubkey: Some(peer_static_pubkey),
            default_reliability: ReliabilityConfig::None,
            packet_pool_size: Self::DEFAULT_POOL_SIZE,
            heartbeat_interval: Self::DEFAULT_HEARTBEAT_INTERVAL,
            session_timeout: Self::DEFAULT_SESSION_TIMEOUT,
            batched_io: false,
            handshake_retries: Self::DEFAULT_HANDSHAKE_RETRIES,
            handshake_timeout: Self::DEFAULT_HANDSHAKE_TIMEOUT,
            socket_recv_buffer: None,
            socket_send_buffer: None,
            num_shards: 1,
        }
    }

    /// Create a new responder configuration.
    ///
    /// The responder provides its static keypair for authentication.
    pub fn responder(
        bind_addr: SocketAddr,
        peer_addr: SocketAddr,
        psk: [u8; 32],
        static_keypair: StaticKeypair,
    ) -> Self {
        Self {
            bind_addr,
            peer_addr,
            psk,
            role: ConnectionRole::Responder,
            static_keypair: Some(static_keypair),
            peer_static_pubkey: None,
            default_reliability: ReliabilityConfig::None,
            packet_pool_size: Self::DEFAULT_POOL_SIZE,
            heartbeat_interval: Self::DEFAULT_HEARTBEAT_INTERVAL,
            session_timeout: Self::DEFAULT_SESSION_TIMEOUT,
            batched_io: false,
            handshake_retries: Self::DEFAULT_HANDSHAKE_RETRIES,
            handshake_timeout: Self::DEFAULT_HANDSHAKE_TIMEOUT,
            socket_recv_buffer: None,
            socket_send_buffer: None,
            num_shards: 1,
        }
    }

    /// Set the number of shards
    pub fn with_num_shards(mut self, num_shards: u16) -> Self {
        self.num_shards = num_shards;
        self
    }

    /// Set the default reliability mode
    pub fn with_reliability(mut self, reliability: ReliabilityConfig) -> Self {
        self.default_reliability = reliability;
        self
    }

    /// Set the packet pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.packet_pool_size = size;
        self
    }

    /// Set the heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set the session timeout
    pub fn with_session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }

    /// Enable or disable batched I/O
    pub fn with_batched_io(mut self, enabled: bool) -> Self {
        self.batched_io = enabled;
        self
    }

    /// Set handshake configuration
    pub fn with_handshake(mut self, retries: usize, timeout: Duration) -> Self {
        self.handshake_retries = retries;
        self.handshake_timeout = timeout;
        self
    }

    /// Set socket buffer sizes (useful for testing with smaller buffers)
    pub fn with_socket_buffers(mut self, recv_size: usize, send_size: usize) -> Self {
        self.socket_recv_buffer = Some(recv_size);
        self.socket_send_buffer = Some(send_size);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        match self.role {
            ConnectionRole::Initiator => {
                if self.peer_static_pubkey.is_none() {
                    return Err("initiator requires peer_static_pubkey".into());
                }
            }
            ConnectionRole::Responder => {
                if self.static_keypair.is_none() {
                    return Err("responder requires static_keypair".into());
                }
            }
        }

        if self.num_shards == 0 {
            return Err("num_shards must be > 0".into());
        }

        if self.packet_pool_size == 0 {
            return Err("packet_pool_size must be > 0".into());
        }

        if self.heartbeat_interval.is_zero() {
            return Err("heartbeat_interval must be > 0".into());
        }

        if self.session_timeout.is_zero() {
            return Err("session_timeout must be > 0".into());
        }

        if self.session_timeout <= self.heartbeat_interval {
            return Err("session_timeout must be > heartbeat_interval".into());
        }

        Ok(())
    }

    /// Check if this is an initiator configuration
    #[inline]
    pub fn is_initiator(&self) -> bool {
        matches!(self.role, ConnectionRole::Initiator)
    }

    /// Check if this is a responder configuration
    #[inline]
    pub fn is_responder(&self) -> bool {
        matches!(self.role, ConnectionRole::Responder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiator_config() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let config = BltpAdapterConfig::initiator(
            "127.0.0.1:9000".parse().unwrap(),
            "127.0.0.1:9001".parse().unwrap(),
            psk,
            peer_pubkey,
        );

        assert!(config.is_initiator());
        assert!(!config.is_responder());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_responder_config() {
        let psk = [0x42u8; 32];
        let keypair = StaticKeypair::generate();

        let config = BltpAdapterConfig::responder(
            "127.0.0.1:9001".parse().unwrap(),
            "127.0.0.1:9000".parse().unwrap(),
            psk,
            keypair,
        );

        assert!(!config.is_initiator());
        assert!(config.is_responder());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_initiator_missing_pubkey() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let mut config = BltpAdapterConfig::initiator(
            "127.0.0.1:9000".parse().unwrap(),
            "127.0.0.1:9001".parse().unwrap(),
            psk,
            peer_pubkey,
        );
        config.peer_static_pubkey = None;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_responder_missing_keypair() {
        let psk = [0x42u8; 32];
        let keypair = StaticKeypair::generate();

        let mut config = BltpAdapterConfig::responder(
            "127.0.0.1:9001".parse().unwrap(),
            "127.0.0.1:9000".parse().unwrap(),
            psk,
            keypair,
        );
        config.static_keypair = None;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_builder_methods() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let config = BltpAdapterConfig::initiator(
            "127.0.0.1:9000".parse().unwrap(),
            "127.0.0.1:9001".parse().unwrap(),
            psk,
            peer_pubkey,
        )
        .with_reliability(ReliabilityConfig::Light)
        .with_pool_size(128)
        .with_heartbeat_interval(Duration::from_secs(10))
        .with_session_timeout(Duration::from_secs(60))
        .with_batched_io(true);

        assert_eq!(config.default_reliability, ReliabilityConfig::Light);
        assert_eq!(config.packet_pool_size, 128);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
        assert_eq!(config.session_timeout, Duration::from_secs(60));
        assert!(config.batched_io);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_reliability_config() {
        assert!(!ReliabilityConfig::None.needs_ack());
        assert!(!ReliabilityConfig::None.is_reliable());

        assert!(ReliabilityConfig::Light.needs_ack());
        assert!(ReliabilityConfig::Light.is_reliable());

        assert!(ReliabilityConfig::Full.needs_ack());
        assert!(ReliabilityConfig::Full.is_reliable());
    }

    #[test]
    fn test_invalid_timeout_order() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let config = BltpAdapterConfig::initiator(
            "127.0.0.1:9000".parse().unwrap(),
            "127.0.0.1:9001".parse().unwrap(),
            psk,
            peer_pubkey,
        )
        .with_heartbeat_interval(Duration::from_secs(30))
        .with_session_timeout(Duration::from_secs(10)); // Less than heartbeat

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_zero_num_shards_rejected() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let config = BltpAdapterConfig::initiator(
            "127.0.0.1:9000".parse().unwrap(),
            "127.0.0.1:9001".parse().unwrap(),
            psk,
            peer_pubkey,
        )
        .with_num_shards(0);

        assert!(config.validate().is_err());
    }
}
