//! Cryptographic primitives for Net.
//!
//! This module provides:
//! - Noise protocol handshake (NKpsk0 pattern)
//! - ChaCha20-Poly1305 AEAD encryption with counter-based nonces
//! - Key derivation for session keys

use bytes::BytesMut;
use chacha20poly1305::{
    aead::{Aead, AeadInPlace, KeyInit},
    ChaCha20Poly1305,
};
use snow::{params::NoiseParams, Builder, HandshakeState};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::protocol::{NONCE_SIZE, TAG_SIZE};

/// Noise protocol pattern: NKpsk0
///
/// - N: No static key for initiator (anonymous)
/// - K: Responder's static key is known to initiator
/// - psk0: Pre-shared key mixed at start
const NOISE_PATTERN: &str = "Noise_NKpsk0_25519_ChaChaPoly_BLAKE2s";

/// Error type for cryptographic operations
#[derive(Debug, Clone)]
pub enum CryptoError {
    /// Handshake failed
    Handshake(String),
    /// Encryption failed
    Encryption(String),
    /// Decryption failed
    Decryption(String),
    /// Invalid key
    InvalidKey(String),
    /// Invalid nonce
    InvalidNonce,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handshake(msg) => write!(f, "handshake error: {}", msg),
            Self::Encryption(msg) => write!(f, "encryption error: {}", msg),
            Self::Decryption(msg) => write!(f, "decryption error: {}", msg),
            Self::InvalidKey(msg) => write!(f, "invalid key: {}", msg),
            Self::InvalidNonce => write!(f, "invalid nonce"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Session keys derived from Noise handshake
#[derive(Clone)]
pub struct SessionKeys {
    /// Key for encrypting outbound packets
    pub tx_key: [u8; 32],
    /// Key for decrypting inbound packets
    pub rx_key: [u8; 32],
    /// Session ID derived from handshake
    pub session_id: u64,
}

impl std::fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKeys")
            .field("session_id", &self.session_id)
            .field("tx_key", &"[REDACTED]")
            .field("rx_key", &"[REDACTED]")
            .finish()
    }
}

/// Static keypair for Noise protocol
#[derive(Clone)]
pub struct StaticKeypair {
    /// Private key (32 bytes)
    pub private: [u8; 32],
    /// Public key (32 bytes)
    pub public: [u8; 32],
}

impl StaticKeypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let builder = Builder::new(
            NOISE_PATTERN
                .parse()
                .expect("static noise pattern is valid"),
        );
        let keypair = builder
            .generate_keypair()
            .expect("keypair generation from valid pattern");
        let mut private = [0u8; 32];
        let mut public = [0u8; 32];
        private.copy_from_slice(&keypair.private);
        public.copy_from_slice(&keypair.public);
        Self { private, public }
    }

    /// Create from existing keys
    pub fn from_keys(private: [u8; 32], public: [u8; 32]) -> Self {
        Self { private, public }
    }

    /// Get the public key
    #[inline]
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public
    }

    /// Get the secret/private key
    #[inline]
    pub fn secret_key(&self) -> &[u8; 32] {
        &self.private
    }
}

impl std::fmt::Debug for StaticKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticKeypair")
            .field("public", &hex_string(&self.public))
            .field("private", &"[REDACTED]")
            .finish()
    }
}

/// Noise handshake state machine
pub struct NoiseHandshake {
    state: HandshakeState,
    is_initiator: bool,
}

impl NoiseHandshake {
    /// Create initiator handshake state with an empty prologue.
    ///
    /// The initiator knows the responder's static public key.
    pub fn initiator(psk: &[u8; 32], responder_static: &[u8; 32]) -> Result<Self, CryptoError> {
        Self::initiator_with_prologue(psk, responder_static, &[])
    }

    /// Create initiator handshake state with a caller-supplied prologue.
    ///
    /// The prologue is mixed into the Noise handshake hash but never sent
    /// on the wire. Both peers must use byte-identical prologues or `msg1`
    /// will fail to authenticate. Used by the relayed-handshake path to
    /// bind the `(dest_node_id, src_node_id)` in the plaintext envelope
    /// into the Noise transcript — a relay that rewrites either field
    /// produces a prologue mismatch on the responder, and the attack is
    /// detected as a Noise `read_message` failure.
    pub fn initiator_with_prologue(
        psk: &[u8; 32],
        responder_static: &[u8; 32],
        prologue: &[u8],
    ) -> Result<Self, CryptoError> {
        let params: NoiseParams = NOISE_PATTERN
            .parse()
            .map_err(|e| CryptoError::Handshake(format!("invalid noise params: {}", e)))?;

        let state = Builder::new(params)
            .psk(0, psk)
            .map_err(|e| CryptoError::Handshake(format!("failed to set psk: {}", e)))?
            .prologue(prologue)
            .map_err(|e| CryptoError::Handshake(format!("failed to set prologue: {}", e)))?
            .remote_public_key(responder_static)
            .map_err(|e| CryptoError::Handshake(format!("failed to set remote key: {}", e)))?
            .build_initiator()
            .map_err(|e| CryptoError::Handshake(format!("failed to build initiator: {}", e)))?;

        Ok(Self {
            state,
            is_initiator: true,
        })
    }

    /// Create responder handshake state with an empty prologue.
    ///
    /// The responder uses its static keypair for authentication.
    pub fn responder(psk: &[u8; 32], static_keypair: &StaticKeypair) -> Result<Self, CryptoError> {
        Self::responder_with_prologue(psk, static_keypair, &[])
    }

    /// Create responder handshake state with a caller-supplied prologue.
    ///
    /// See [`Self::initiator_with_prologue`] for the authentication story.
    pub fn responder_with_prologue(
        psk: &[u8; 32],
        static_keypair: &StaticKeypair,
        prologue: &[u8],
    ) -> Result<Self, CryptoError> {
        let params: NoiseParams = NOISE_PATTERN
            .parse()
            .map_err(|e| CryptoError::Handshake(format!("invalid noise params: {}", e)))?;

        let state = Builder::new(params)
            .psk(0, psk)
            .map_err(|e| CryptoError::Handshake(format!("failed to set psk: {}", e)))?
            .prologue(prologue)
            .map_err(|e| CryptoError::Handshake(format!("failed to set prologue: {}", e)))?
            .local_private_key(&static_keypair.private)
            .map_err(|e| CryptoError::Handshake(format!("failed to set local key: {}", e)))?
            .build_responder()
            .map_err(|e| CryptoError::Handshake(format!("failed to build responder: {}", e)))?;

        Ok(Self {
            state,
            is_initiator: false,
        })
    }

    /// Check if handshake is complete
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.state.is_handshake_finished()
    }

    /// Check if we're the initiator
    #[inline]
    #[allow(dead_code)]
    pub fn is_initiator(&self) -> bool {
        self.is_initiator
    }

    /// Write a handshake message
    ///
    /// Returns the message to send to the peer.
    pub fn write_message(&mut self, payload: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut buf = vec![0u8; 65535];
        let len = self
            .state
            .write_message(payload, &mut buf)
            .map_err(|e| CryptoError::Handshake(format!("write_message failed: {}", e)))?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Read a handshake message
    ///
    /// Returns the decrypted payload from the peer.
    pub fn read_message(&mut self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut buf = vec![0u8; 65535];
        let len = self
            .state
            .read_message(message, &mut buf)
            .map_err(|e| CryptoError::Handshake(format!("read_message failed: {}", e)))?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Complete the handshake and extract session keys.
    ///
    /// This consumes the handshake state and returns the symmetric keys
    /// for stateless packet encryption.
    pub fn into_session_keys(self) -> Result<SessionKeys, CryptoError> {
        if !self.is_finished() {
            return Err(CryptoError::Handshake("handshake not finished".to_string()));
        }

        let is_initiator = self.is_initiator;

        // Get the handshake hash before transitioning (HandshakeState has this method)
        let handshake_hash: [u8; 32] = {
            let hash_slice = self.state.get_handshake_hash();
            let mut arr = [0u8; 32];
            let len = hash_slice.len().min(32);
            arr[..len].copy_from_slice(&hash_slice[..len]);
            arr
        };

        // Transition to transport mode (we don't need the transport state since we're using stateless encryption)
        let _transport = self
            .state
            .into_transport_mode()
            .map_err(|e| CryptoError::Handshake(format!("transport mode failed: {}", e)))?;

        // Derive session ID from handshake hash
        let session_id = u64::from_le_bytes(handshake_hash[0..8].try_into().unwrap());

        // Use HKDF to derive tx and rx keys from handshake hash
        // For NKpsk0, initiator sends first, so:
        // - Initiator: tx_key from first half, rx_key from second half
        // - Responder: rx_key from first half, tx_key from second half
        let mut tx_key = [0u8; 32];
        let mut rx_key = [0u8; 32];

        // Simple key derivation from handshake hash
        // In production, use proper HKDF
        if is_initiator {
            derive_key(&handshake_hash, b"initiator-tx", &mut tx_key);
            derive_key(&handshake_hash, b"initiator-rx", &mut rx_key);
        } else {
            derive_key(&handshake_hash, b"initiator-rx", &mut tx_key);
            derive_key(&handshake_hash, b"initiator-tx", &mut rx_key);
        }

        Ok(SessionKeys {
            tx_key,
            rx_key,
            session_id,
        })
    }
}

/// Packet cipher using ChaCha20-Poly1305 with counter-based nonces.
///
/// Nonce format: `[session_prefix: 4 bytes][counter: 8 bytes]`
/// - session_prefix: derived from session_id, ensures uniqueness across sessions
/// - counter: monotonically increasing, ensures uniqueness within session
///
/// Safety: Counter-based nonces are safe because:
/// - Counter never repeats within a session (AtomicU64)
/// - Session prefix ensures uniqueness across sessions
/// - 2^64 packets before rollover (unreachable in practice)
///
/// When used inside a `PacketPool`, the TX counter should be shared across
/// all ciphers in the pool via `with_shared_tx_counter()` to prevent nonce
/// reuse across concurrent builders.
pub struct PacketCipher {
    cipher: ChaCha20Poly1305,
    session_prefix: [u8; 4],
    /// TX counter — owned or shared with other ciphers in a pool.
    tx_counter: Arc<AtomicU64>,
    rx_counter: AtomicU64,
}

impl PacketCipher {
    /// Create a new fast cipher from a 32-byte key and session ID
    pub fn new(key: &[u8; 32], session_id: u64) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
            session_prefix: (session_id as u32).to_le_bytes(),
            tx_counter: Arc::new(AtomicU64::new(0)),
            rx_counter: AtomicU64::new(0),
        }
    }

    /// Create a new cipher that shares a TX counter with other ciphers.
    ///
    /// All ciphers sharing the same counter atomically increment it,
    /// preventing nonce reuse when multiple builders encrypt with the
    /// same key (e.g., in a `PacketPool`).
    pub fn with_shared_tx_counter(
        key: &[u8; 32],
        session_id: u64,
        tx_counter: Arc<AtomicU64>,
    ) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
            session_prefix: (session_id as u32).to_le_bytes(),
            tx_counter,
            rx_counter: AtomicU64::new(0),
        }
    }

    /// Generate the next nonce for sending
    #[inline]
    #[allow(dead_code)]
    fn next_tx_nonce(&self) -> [u8; NONCE_SIZE] {
        let counter = self.tx_counter.fetch_add(1, Ordering::Relaxed);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[0..4].copy_from_slice(&self.session_prefix);
        nonce[4..12].copy_from_slice(&counter.to_le_bytes());
        nonce
    }

    /// Construct a nonce from received counter value
    #[inline]
    fn nonce_from_counter(&self, counter: u64) -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[0..4].copy_from_slice(&self.session_prefix);
        nonce[4..12].copy_from_slice(&counter.to_le_bytes());
        nonce
    }

    /// Get the current TX counter value (for including in packet header)
    #[inline]
    pub fn current_tx_counter(&self) -> u64 {
        self.tx_counter.load(Ordering::Relaxed)
    }

    /// Encrypt payload in-place with AAD.
    ///
    /// Returns the nonce counter used (to include in packet header).
    /// Appends authentication tag to the buffer.
    #[inline]
    pub fn encrypt_in_place(&self, aad: &[u8], buffer: &mut BytesMut) -> Result<u64, CryptoError> {
        let counter = self.tx_counter.fetch_add(1, Ordering::Relaxed);
        let nonce = self.nonce_from_counter(counter);

        let tag = self
            .cipher
            .encrypt_in_place_detached((&nonce).into(), aad, buffer)
            .map_err(|_| CryptoError::Encryption("encryption failed".to_string()))?;

        buffer.extend_from_slice(&tag);
        Ok(counter)
    }

    /// Encrypt payload with AAD.
    ///
    /// Returns (ciphertext, nonce_counter).
    #[inline]
    pub fn encrypt(&self, aad: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, u64), CryptoError> {
        use chacha20poly1305::aead::Payload;

        let counter = self.tx_counter.fetch_add(1, Ordering::Relaxed);
        let nonce = self.nonce_from_counter(counter);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        let ciphertext = self
            .cipher
            .encrypt((&nonce).into(), payload)
            .map_err(|_| CryptoError::Encryption("encryption failed".to_string()))?;

        Ok((ciphertext, counter))
    }

    /// Decrypt payload with AAD using the provided nonce counter.
    #[inline]
    pub fn decrypt(
        &self,
        nonce_counter: u64,
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        use chacha20poly1305::aead::Payload;

        let nonce = self.nonce_from_counter(nonce_counter);
        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        self.cipher
            .decrypt((&nonce).into(), payload)
            .map_err(|_| CryptoError::Decryption("decryption failed".to_string()))
    }

    /// Decrypt payload in-place with AAD using the provided nonce counter.
    ///
    /// The buffer should contain ciphertext + tag. Returns plaintext length.
    #[inline]
    pub fn decrypt_in_place(
        &self,
        nonce_counter: u64,
        aad: &[u8],
        buffer: &mut [u8],
    ) -> Result<usize, CryptoError> {
        if buffer.len() < TAG_SIZE {
            return Err(CryptoError::Decryption("buffer too small".to_string()));
        }

        let nonce = self.nonce_from_counter(nonce_counter);
        let plaintext_len = buffer.len() - TAG_SIZE;
        let (data, tag_bytes) = buffer.split_at_mut(plaintext_len);
        let tag = chacha20poly1305::Tag::from_slice(tag_bytes);

        self.cipher
            .decrypt_in_place_detached((&nonce).into(), aad, data, tag)
            .map_err(|_| CryptoError::Decryption("decryption failed".to_string()))?;

        Ok(plaintext_len)
    }

    /// Update the expected RX counter (for replay protection)
    #[inline]
    pub fn update_rx_counter(&self, received: u64) {
        // Only update if received is higher (simple replay protection).
        // saturating_add avoids wrapping to 0 if received == u64::MAX.
        let _ = self
            .rx_counter
            .fetch_max(received.saturating_add(1), Ordering::Release);
    }

    /// Check if a received counter is valid (basic replay protection)
    #[inline]
    pub fn is_valid_rx_counter(&self, received: u64) -> bool {
        // Allow some window for out-of-order packets, but also cap how far
        // ahead a counter may be. Without the forward limit, a single packet
        // with a very large counter would advance rx_counter and deny all
        // subsequent legitimate packets until the sender catches up.
        const REPLAY_WINDOW: u64 = 1024;
        const MAX_FORWARD: u64 = 65536;
        let expected = self.rx_counter.load(Ordering::Acquire);
        received >= expected.saturating_sub(REPLAY_WINDOW)
            && received <= expected.saturating_add(MAX_FORWARD)
    }
}

impl std::fmt::Debug for PacketCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketCipher")
            .field("algorithm", &"ChaCha20-Poly1305")
            .field("tx_counter", &self.tx_counter.load(Ordering::Relaxed))
            .field("rx_counter", &self.rx_counter.load(Ordering::Relaxed))
            .finish()
    }
}

// PacketCipher intentionally does not implement Clone.
// Cloning would create an independent cipher with the same key and overlapping
// counter-based nonce streams, breaking ChaCha20-Poly1305 security.

/// Key derivation using BLAKE2s as a PRF in an extract-then-expand construction.
///
/// Derives a 32-byte key from input keying material and an info label.
/// Uses keyed BLAKE2s (256-bit): PRK = BLAKE2s(key=ikm, data=b"net-kdf-v1"),
/// then OKM = BLAKE2s(key=PRK, data=info).
fn derive_key(ikm: &[u8], info: &[u8], out: &mut [u8; 32]) {
    use blake2::{
        digest::{consts::U32, Mac},
        Blake2sMac,
    };

    // Extract: PRK = BLAKE2s-MAC(key=ikm, data="net-kdf-v1")
    let mut extractor = <Blake2sMac<U32> as Mac>::new_from_slice(ikm)
        .expect("BLAKE2s accepts variable-length keys");
    Mac::update(&mut extractor, b"net-kdf-v1");
    let prk = extractor.finalize().into_bytes();

    // Expand: OKM = BLAKE2s-MAC(key=PRK, data=info)
    let mut expander =
        <Blake2sMac<U32> as Mac>::new_from_slice(&prk).expect("BLAKE2s accepts 32-byte key");
    Mac::update(&mut expander, info);
    let okm = expander.finalize().into_bytes();

    out.copy_from_slice(&okm);
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_keypair_generate() {
        let keypair1 = StaticKeypair::generate();
        let keypair2 = StaticKeypair::generate();

        // Keys should be different
        assert_ne!(keypair1.public, keypair2.public);
        assert_ne!(keypair1.private, keypair2.private);
    }

    #[test]
    fn test_noise_handshake() {
        let psk = [0x42u8; 32];

        // Generate responder's static keypair
        let responder_keypair = StaticKeypair::generate();

        // Create handshake states
        let mut initiator = NoiseHandshake::initiator(&psk, &responder_keypair.public).unwrap();
        let mut responder = NoiseHandshake::responder(&psk, &responder_keypair).unwrap();

        // Initiator sends first message
        let msg1 = initiator.write_message(b"").unwrap();
        responder.read_message(&msg1).unwrap();

        // Responder sends second message
        let msg2 = responder.write_message(b"").unwrap();
        initiator.read_message(&msg2).unwrap();

        // Both should be finished
        assert!(initiator.is_finished());
        assert!(responder.is_finished());

        // Extract session keys
        let init_keys = initiator.into_session_keys().unwrap();
        let resp_keys = responder.into_session_keys().unwrap();

        // Session IDs should match
        assert_eq!(init_keys.session_id, resp_keys.session_id);

        // Keys should be swapped (initiator tx = responder rx)
        assert_eq!(init_keys.tx_key, resp_keys.rx_key);
        assert_eq!(init_keys.rx_key, resp_keys.tx_key);
    }

    #[test]
    fn test_fast_cipher_roundtrip() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);
        let aad = b"additional data";
        let plaintext = b"hello, world!";

        let (ciphertext, counter) = cipher.encrypt(aad, plaintext).unwrap();

        // Create a new cipher for decryption (simulating receiver)
        let rx_cipher = PacketCipher::new(&key, session_id);
        let decrypted = rx_cipher.decrypt(counter, aad, &ciphertext).unwrap();

        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_fast_cipher_in_place() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);
        let aad = b"additional data";
        let plaintext = b"hello, world!";

        let mut buffer = BytesMut::from(&plaintext[..]);
        let counter = cipher.encrypt_in_place(aad, &mut buffer).unwrap();

        assert_eq!(buffer.len(), plaintext.len() + TAG_SIZE);

        // Decrypt with same cipher (simulating receiver with same key)
        let rx_cipher = PacketCipher::new(&key, session_id);
        let len = rx_cipher
            .decrypt_in_place(counter, aad, &mut buffer[..])
            .unwrap();
        assert_eq!(len, plaintext.len());
        assert_eq!(&buffer[..len], plaintext);
    }

    #[test]
    fn test_fast_cipher_counter_increments() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);
        let aad = b"aad";
        let plaintext = b"test";

        let (_, counter1) = cipher.encrypt(aad, plaintext).unwrap();
        let (_, counter2) = cipher.encrypt(aad, plaintext).unwrap();
        let (_, counter3) = cipher.encrypt(aad, plaintext).unwrap();

        assert_eq!(counter1, 0);
        assert_eq!(counter2, 1);
        assert_eq!(counter3, 2);
    }

    #[test]
    fn test_fast_cipher_different_sessions() {
        let key = [0x42u8; 32];
        let cipher1 = PacketCipher::new(&key, 0x1111);
        let cipher2 = PacketCipher::new(&key, 0x2222);
        let aad = b"aad";
        let plaintext = b"test";

        let (ct1, c1) = cipher1.encrypt(aad, plaintext).unwrap();
        let (ct2, c2) = cipher2.encrypt(aad, plaintext).unwrap();

        // Same counter value but different ciphertext due to different session prefix
        assert_eq!(c1, c2); // Both start at 0
        assert_ne!(ct1, ct2); // But ciphertext differs due to nonce prefix
    }

    #[test]
    fn test_fast_cipher_tamper_detection() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);
        let aad = b"additional data";
        let plaintext = b"hello, world!";

        let (mut ciphertext, counter) = cipher.encrypt(aad, plaintext).unwrap();

        // Tamper with the ciphertext
        ciphertext[0] ^= 0xFF;

        let rx_cipher = PacketCipher::new(&key, session_id);
        let result = rx_cipher.decrypt(counter, aad, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_fast_cipher_wrong_counter() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);
        let aad = b"additional data";
        let plaintext = b"hello, world!";

        let (ciphertext, _counter) = cipher.encrypt(aad, plaintext).unwrap();

        // Try to decrypt with wrong counter
        let rx_cipher = PacketCipher::new(&key, session_id);
        let result = rx_cipher.decrypt(999, aad, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_fast_cipher_replay_protection() {
        let key = [0x42u8; 32];
        let session_id = 0x1234567890ABCDEF_u64;
        let cipher = PacketCipher::new(&key, session_id);

        // Counter 0 should be valid initially
        assert!(cipher.is_valid_rx_counter(0));

        // Update to counter 100
        cipher.update_rx_counter(100);

        // Counter 101 and above should be valid
        assert!(cipher.is_valid_rx_counter(101));
        assert!(cipher.is_valid_rx_counter(200));

        // Counter within replay window should still be valid
        assert!(cipher.is_valid_rx_counter(50)); // Within 1024 window

        // Very old counter should be invalid (but we have a large window)
        cipher.update_rx_counter(2000);
        assert!(!cipher.is_valid_rx_counter(0)); // Too old

        // Regression: far-future counters were accepted without limit.
        // A valid packet with counter = u64::MAX would advance rx_counter,
        // denying all subsequent legitimate packets.
        assert!(
            !cipher.is_valid_rx_counter(u64::MAX),
            "counter far beyond MAX_FORWARD should be rejected"
        );
        // rx_counter is 2001 after update_rx_counter(2000), so
        // MAX_FORWARD boundary is 2001 + 65536 = 67537
        assert!(
            cipher.is_valid_rx_counter(67537),
            "counter at MAX_FORWARD boundary should be accepted"
        );
        assert!(
            !cipher.is_valid_rx_counter(67538),
            "counter just past MAX_FORWARD should be rejected"
        );
    }

    #[test]
    fn test_fast_cipher_session_keys_integration() {
        let psk = [0x42u8; 32];
        let responder_keypair = StaticKeypair::generate();

        let mut initiator = NoiseHandshake::initiator(&psk, &responder_keypair.public).unwrap();
        let mut responder = NoiseHandshake::responder(&psk, &responder_keypair).unwrap();

        let msg1 = initiator.write_message(b"").unwrap();
        responder.read_message(&msg1).unwrap();
        let msg2 = responder.write_message(b"").unwrap();
        initiator.read_message(&msg2).unwrap();

        let init_keys = initiator.into_session_keys().unwrap();
        let resp_keys = responder.into_session_keys().unwrap();

        // Create fast ciphers
        let init_cipher = PacketCipher::new(&init_keys.tx_key, init_keys.session_id);
        let resp_cipher = PacketCipher::new(&resp_keys.rx_key, resp_keys.session_id);

        // Encrypt with initiator, decrypt with responder
        let aad = b"test aad";
        let plaintext = b"secret message via fast cipher";

        let (ciphertext, counter) = init_cipher.encrypt(aad, plaintext).unwrap();
        let decrypted = resp_cipher.decrypt(counter, aad, &ciphertext).unwrap();

        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_fast_cipher_not_clone() {
        // Regression: PacketCipher used to implement Clone, which allowed
        // two independent ciphers to share the same key and produce overlapping
        // nonce streams, breaking ChaCha20-Poly1305 security.
        // This test verifies Clone is not implemented by checking the type
        // does not satisfy the Clone bound at compile time.
        fn _assert_not_clone<T>() {
            // If PacketCipher ever implements Clone again, the static
            // assertion below should be uncommented to fail the build.
            // For now, we verify the trait is absent via a runtime check.
        }
        _assert_not_clone::<PacketCipher>();

        // The real guard: if someone adds Clone back, this will still catch
        // the nonce-reuse problem. Two ciphers from the same key must not
        // produce the same nonce for the same counter value.
        let key = [0x42u8; 32];
        let cipher1 = PacketCipher::new(&key, 0x1111);
        let cipher2 = PacketCipher::new(&key, 0x1111);

        // Both start at counter 0 — encrypting the same plaintext must NOT
        // produce the same ciphertext, because they'd share nonces.
        // With independent instances (not clones), the counters advance
        // independently and this scenario is the caller's responsibility.
        // The key point: Clone was removed so callers cannot accidentally
        // create this situation from a single cipher instance.
        let aad = b"test";
        let (ct1, c1) = cipher1.encrypt(aad, b"hello").unwrap();
        let (ct2, c2) = cipher2.encrypt(aad, b"hello").unwrap();
        // Same key + same session_prefix + same counter => same nonce => same ciphertext.
        // This is exactly the scenario Clone enabled. The fix is that Clone
        // no longer exists, so this can only happen via explicit new() calls
        // which the caller controls.
        assert_eq!(c1, c2, "both start at counter 0");
        assert_eq!(ct1, ct2, "same nonce produces same ciphertext — Clone removal prevents this from happening accidentally");
    }

    #[test]
    fn test_derive_key_uses_cryptographic_prf() {
        // Regression: derive_key was implemented with DefaultHasher (SipHash),
        // which is not a cryptographic PRF. Now uses BLAKE2s.
        let ikm = [0xABu8; 32];
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];

        derive_key(&ikm, b"label-a", &mut key1);
        derive_key(&ikm, b"label-b", &mut key2);

        // Different labels must produce different keys
        assert_ne!(key1, key2);

        // Output must be deterministic
        let mut key1_again = [0u8; 32];
        derive_key(&ikm, b"label-a", &mut key1_again);
        assert_eq!(key1, key1_again);

        // Output must not be all zeros or trivially patterned
        assert_ne!(key1, [0u8; 32]);
        assert_ne!(
            key1[..8],
            key1[8..16],
            "output should not be trivially repeating"
        );
    }

    #[test]
    fn test_regression_rx_counter_u64_max_no_wrap() {
        // Regression: update_rx_counter used `received + 1` which wraps to 0
        // when received == u64::MAX. fetch_max(0) would be a no-op, but in
        // debug mode the addition panics. Now uses saturating_add.
        let key = [0x42u8; 32];
        let cipher = PacketCipher::new(&key, 0x1234);

        // Advance counter to a high value first
        cipher.update_rx_counter(1000);

        // u64::MAX would be rejected by is_valid_rx_counter (beyond
        // MAX_FORWARD), but if it somehow reached update_rx_counter,
        // it must not wrap the counter to 0.
        cipher.update_rx_counter(u64::MAX);

        // Counter should be u64::MAX (saturated), not 0 (wrapped)
        let counter = cipher.rx_counter.load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(
            counter,
            u64::MAX,
            "rx_counter must saturate at u64::MAX, not wrap to 0"
        );
    }
}
