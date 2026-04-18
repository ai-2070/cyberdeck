//! Handshake relay via subprotocol.
//!
//! Carries Noise NKpsk0 handshake messages as a normal Net subprotocol so
//! `MeshNode` can establish a session with a peer it has no direct UDP path
//! to, as long as a chain of already-connected peers links them.
//!
//! Wire format for the subprotocol payload:
//!
//! ```text
//! [dest_node_id: u64 LE][src_node_id: u64 LE][noise_bytes: variable]
//! ```
//!
//! The relay reads `dest_node_id` to pick the next hop. Noise NKpsk0 bytes
//! are authenticated with the PSK and the responder's static key, so a relay
//! can see them but can't forge or decrypt them.
//!
//! Completion of the handshake is asymmetric:
//! - Responder completes in one step (reads msg1, writes msg2, derives keys).
//! - Initiator holds `NoiseHandshake` state in `pending_initiator` until
//!   msg2 arrives, then completes and unblocks the waiter via a oneshot.

use bytes::{Buf, Bytes};
use dashmap::DashMap;
use std::net::SocketAddr;
use tokio::sync::oneshot;

use super::crypto::{CryptoError, NoiseHandshake, SessionKeys, StaticKeypair};

/// Subprotocol identifier for relayed Noise handshake messages.
pub const SUBPROTOCOL_HANDSHAKE: u16 = 0x0601;

/// Minimum payload length: two u64 node IDs.
const PAYLOAD_HEADER_LEN: usize = 16;

/// Encoded handshake payload: `[dest: u64][src: u64][noise: ..]`.
pub fn encode_payload(dest_node_id: u64, src_node_id: u64, noise_bytes: &[u8]) -> Bytes {
    let mut out = Vec::with_capacity(PAYLOAD_HEADER_LEN + noise_bytes.len());
    out.extend_from_slice(&dest_node_id.to_le_bytes());
    out.extend_from_slice(&src_node_id.to_le_bytes());
    out.extend_from_slice(noise_bytes);
    Bytes::from(out)
}

/// Noise prologue for a relayed handshake between `initiator` (src) and
/// `responder` (dest). Bound into both sides' Noise state so a relay that
/// rewrites either node ID produces a prologue mismatch, which in turn
/// fails `read_message` at the responder (no session keys derived, no
/// peer registered). The prologue is domain-separated by a short label to
/// avoid accidental collision with other prologue-using contexts.
fn relay_prologue(initiator: u64, responder: u64) -> [u8; 16 + 16] {
    // 16-byte domain tag + 16-byte payload (two u64 LE) = 32 bytes.
    let mut buf = [0u8; 32];
    buf[0..16].copy_from_slice(b"net-relay-v1\0\0\0\0");
    buf[16..24].copy_from_slice(&initiator.to_le_bytes());
    buf[24..32].copy_from_slice(&responder.to_le_bytes());
    buf
}

/// Parsed handshake payload.
pub struct HandshakePayload {
    pub dest_node_id: u64,
    pub src_node_id: u64,
    pub noise_bytes: Bytes,
}

impl HandshakePayload {
    fn decode(bytes: Bytes) -> Option<Self> {
        if bytes.len() < PAYLOAD_HEADER_LEN {
            return None;
        }
        let mut cursor = &bytes[..PAYLOAD_HEADER_LEN];
        let dest_node_id = cursor.get_u64_le();
        let src_node_id = cursor.get_u64_le();
        let noise_bytes = bytes.slice(PAYLOAD_HEADER_LEN..);
        Some(Self {
            dest_node_id,
            src_node_id,
            noise_bytes,
        })
    }
}

/// One-shot waker for a pending initiator handshake.
struct PendingInitiator {
    noise: NoiseHandshake,
    /// Fired with the derived session keys once the responder's msg2 arrives.
    tx: oneshot::Sender<Result<SessionKeys, CryptoError>>,
}

/// Action produced by [`HandshakeHandler::handle_message`] that the receive
/// loop must execute. The handler itself has no I/O or peer-table access.
pub enum HandshakeAction {
    /// Forward the (re-encoded) handshake payload toward `to_node`. Used by
    /// relay nodes in the middle of a chain.
    Forward { to_node: u64, payload: Bytes },
    /// We are the responder — handshake is complete. Register `peer_node_id`
    /// as a peer, reachable via `relay_addr` on the wire, with `keys`. Send
    /// `response_payload` back to the same `relay_addr` as another
    /// `SUBPROTOCOL_HANDSHAKE` message so the initiator can complete.
    RegisterResponderPeer {
        peer_node_id: u64,
        relay_addr: SocketAddr,
        keys: SessionKeys,
        response_payload: Bytes,
    },
}

/// Manages relayed Noise handshakes.
///
/// State is per-`MeshNode`: one handler instance holds pending initiator
/// states for every in-flight outbound handshake this node has started.
pub struct HandshakeHandler {
    local_node_id: u64,
    static_keypair: StaticKeypair,
    psk: [u8; 32],
    /// In-flight initiator handshakes, keyed by responder's `node_id`.
    /// When `[dest=us, src=X]` arrives and `X` is in this map, it's the
    /// msg2 response to our initiator state for peer X.
    pending_initiator: DashMap<u64, PendingInitiator>,
}

impl HandshakeHandler {
    pub fn new(local_node_id: u64, static_keypair: StaticKeypair, psk: [u8; 32]) -> Self {
        Self {
            local_node_id,
            static_keypair,
            psk,
            pending_initiator: DashMap::new(),
        }
    }

    /// Remove a pending initiator entry. Called by the caller on timeout or
    /// other abandonment so the state doesn't leak.
    pub fn cancel_initiator(&self, dest_node_id: u64) {
        self.pending_initiator.remove(&dest_node_id);
    }

    /// Start an initiator handshake: build msg1 and register a waiter keyed
    /// by `dest_node_id`. The caller sends the returned payload as
    /// `SUBPROTOCOL_HANDSHAKE` to a relay, then `await`s on the receiver
    /// to get the session keys once msg2 comes back through the relay.
    ///
    /// Rejects a second concurrent call for the same `dest_node_id`: the
    /// `pending_initiator` map is keyed only by destination, so a silent
    /// overwrite would orphan the first caller's oneshot and leak its
    /// Noise state. Callers that want to retry must wait for the current
    /// attempt to complete (successfully, by timeout, or via
    /// [`Self::cancel_initiator`]) before trying again.
    pub fn begin_initiator(
        &self,
        dest_node_id: u64,
        dest_pubkey: &[u8; 32],
    ) -> Result<(Bytes, oneshot::Receiver<Result<SessionKeys, CryptoError>>), CryptoError> {
        // Check-and-insert under a single DashMap entry lock to avoid a
        // race between two concurrent callers.
        let entry = match self.pending_initiator.entry(dest_node_id) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                return Err(CryptoError::Handshake(format!(
                    "handshake already in flight for peer {:#x}",
                    dest_node_id
                )));
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => entry,
        };

        // Prologue binds (initiator=local, responder=dest) into the Noise
        // transcript. A relay that rewrites either field in the wire
        // envelope will cause the responder to build a different prologue
        // and fail `read_message` on msg1.
        let prologue = relay_prologue(self.local_node_id, dest_node_id);
        let mut noise = NoiseHandshake::initiator_with_prologue(&self.psk, dest_pubkey, &prologue)?;
        let msg1 = noise.write_message(&[])?;
        let (tx, rx) = oneshot::channel();
        entry.insert(PendingInitiator { noise, tx });
        let payload = encode_payload(dest_node_id, self.local_node_id, &msg1);
        Ok((payload, rx))
    }

    /// Process an inbound `SUBPROTOCOL_HANDSHAKE` payload. `from_addr` is the
    /// wire address of the peer that delivered this packet (the previous hop
    /// along the relay chain). Returns zero or more actions for the caller
    /// to execute.
    pub fn handle_message(&self, payload: &[u8], from_addr: SocketAddr) -> Vec<HandshakeAction> {
        let parsed = match HandshakePayload::decode(Bytes::copy_from_slice(payload)) {
            Some(p) => p,
            None => {
                tracing::warn!("handshake relay: payload too short");
                return Vec::new();
            }
        };

        if parsed.dest_node_id != self.local_node_id {
            // Not for us — forward toward dest.
            return vec![HandshakeAction::Forward {
                to_node: parsed.dest_node_id,
                payload: Bytes::copy_from_slice(payload),
            }];
        }

        // Destined for us. If we have a pending initiator state for the
        // sender, this is the msg2 response to our earlier msg1.
        if let Some((_, pending)) = self.pending_initiator.remove(&parsed.src_node_id) {
            let PendingInitiator { mut noise, tx } = pending;
            let keys_result = (|| -> Result<SessionKeys, CryptoError> {
                noise.read_message(&parsed.noise_bytes)?;
                noise.into_session_keys()
            })();
            // Ignore send error — caller may have dropped the receiver if it
            // timed out; there's nothing for us to clean up here.
            let _ = tx.send(keys_result);
            return Vec::new();
        }

        // Fresh responder message (msg1 from a new initiator).
        //
        // Build the prologue from the plaintext envelope's claimed
        // (src, dest) node IDs. If a relay rewrote either field, the
        // prologue here will differ from what the genuine initiator used,
        // and `read_message` below will fail — the tampered handshake is
        // rejected and no peer is registered.
        let prologue = relay_prologue(parsed.src_node_id, parsed.dest_node_id);
        let mut noise = match NoiseHandshake::responder_with_prologue(
            &self.psk,
            &self.static_keypair,
            &prologue,
        ) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(error = %e, "handshake relay: failed to create responder");
                return Vec::new();
            }
        };

        if let Err(e) = noise.read_message(&parsed.noise_bytes) {
            tracing::warn!(error = %e, "handshake relay: responder read_message failed");
            return Vec::new();
        }

        let msg2 = match noise.write_message(&[]) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "handshake relay: responder write_message failed");
                return Vec::new();
            }
        };

        let keys = match noise.into_session_keys() {
            Ok(k) => k,
            Err(e) => {
                tracing::warn!(error = %e, "handshake relay: key extraction failed");
                return Vec::new();
            }
        };

        let response_payload = encode_payload(parsed.src_node_id, self.local_node_id, &msg2);

        vec![HandshakeAction::RegisterResponderPeer {
            peer_node_id: parsed.src_node_id,
            relay_addr: from_addr,
            keys,
            response_payload,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_roundtrip() {
        let noise = b"opaque noise bytes";
        let bytes = encode_payload(0xAAAA, 0xBBBB, noise);
        let parsed = HandshakePayload::decode(bytes).unwrap();
        assert_eq!(parsed.dest_node_id, 0xAAAA);
        assert_eq!(parsed.src_node_id, 0xBBBB);
        assert_eq!(&parsed.noise_bytes[..], noise);
    }

    #[test]
    fn payload_rejects_short_input() {
        assert!(HandshakePayload::decode(Bytes::from_static(b"short")).is_none());
    }

    #[tokio::test]
    async fn relay_initiator_responder_roundtrip() {
        let psk = [0x42u8; 32];
        let responder_kp = StaticKeypair::generate();
        let initiator_kp = StaticKeypair::generate();
        let nid_init: u64 = 0x1111;
        let nid_resp: u64 = 0x2222;

        let initiator = HandshakeHandler::new(nid_init, initiator_kp, psk);
        let responder = HandshakeHandler::new(nid_resp, responder_kp.clone(), psk);

        // 1. Initiator builds msg1.
        let (msg1_payload, keys_rx) = initiator
            .begin_initiator(nid_resp, &responder_kp.public)
            .unwrap();

        // Simulate "sending via relay": the relay just forwards the payload
        // to the responder; here we hand it straight to the responder handler.
        let dummy_relay_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let actions = responder.handle_message(&msg1_payload, dummy_relay_addr);
        assert_eq!(actions.len(), 1);

        let (msg2_payload, resp_keys) = match actions.into_iter().next().unwrap() {
            HandshakeAction::RegisterResponderPeer {
                peer_node_id,
                keys,
                response_payload,
                ..
            } => {
                assert_eq!(peer_node_id, nid_init);
                (response_payload, keys)
            }
            _ => panic!("expected RegisterResponderPeer"),
        };

        // 2. Initiator receives msg2 (also via relay).
        let actions = initiator.handle_message(&msg2_payload, dummy_relay_addr);
        assert!(
            actions.is_empty(),
            "initiator should fire oneshot, not emit actions"
        );

        let init_keys = keys_rx.await.unwrap().unwrap();

        assert_eq!(init_keys.session_id, resp_keys.session_id);
        assert_eq!(init_keys.tx_key, resp_keys.rx_key);
        assert_eq!(init_keys.rx_key, resp_keys.tx_key);
    }

    #[test]
    fn middle_hop_forwards_when_not_addressed_to_us() {
        let psk = [0x77u8; 32];
        let kp = StaticKeypair::generate();
        let middle = HandshakeHandler::new(0x5555, kp, psk);

        // Payload addressed to some other node, not the middle hop.
        let payload = encode_payload(0x9999, 0x1111, b"noise");
        let from: SocketAddr = "127.0.0.1:5000".parse().unwrap();
        let actions = middle.handle_message(&payload, from);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            HandshakeAction::Forward { to_node, .. } => assert_eq!(*to_node, 0x9999),
            _ => panic!("expected Forward"),
        }
    }

    /// Regression: the relay envelope's `src_node_id` field used to flow
    /// directly into peer registration on the responder without any
    /// cryptographic check. A malicious relay could rewrite it and the
    /// responder would bind its session keys to an attacker-chosen peer
    /// ID, misrouting traffic or enabling impersonation.
    ///
    /// The fix binds `(src_node_id, dest_node_id)` into the Noise prologue
    /// on both sides. Rewriting either field on the wire makes the
    /// responder's prologue differ from what the genuine initiator used,
    /// causing `read_message` on msg1 to fail. No `RegisterResponderPeer`
    /// action is emitted and no peer is registered.
    #[tokio::test]
    async fn test_regression_relay_tampering_of_src_node_id_is_rejected() {
        let psk = [0x42u8; 32];
        let responder_kp = StaticKeypair::generate();
        let initiator_kp = StaticKeypair::generate();
        let nid_init: u64 = 0x1111;
        let nid_resp: u64 = 0x2222;
        let nid_attacker: u64 = 0x9999;

        let initiator = HandshakeHandler::new(nid_init, initiator_kp, psk);
        let responder = HandshakeHandler::new(nid_resp, responder_kp.clone(), psk);

        // Genuine initiator builds msg1 addressed to nid_resp with its
        // own nid_init as the source.
        let (genuine_payload, _rx) = initiator
            .begin_initiator(nid_resp, &responder_kp.public)
            .unwrap();

        // Malicious relay rewrites src_node_id from nid_init to
        // nid_attacker. dest and noise_bytes unchanged.
        let parsed = HandshakePayload::decode(genuine_payload.clone()).unwrap();
        assert_eq!(parsed.src_node_id, nid_init);
        let tampered = encode_payload(parsed.dest_node_id, nid_attacker, &parsed.noise_bytes);

        let dummy_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let actions = responder.handle_message(&tampered, dummy_addr);

        assert!(
            actions.is_empty(),
            "tampered src_node_id must cause msg1 to fail Noise auth — \
             no RegisterResponderPeer should be emitted, got {} actions",
            actions.len()
        );
    }

    /// Same attack on the `dest_node_id` field: a relay that rewrites the
    /// envelope's destination (say, to confuse which daemon this handshake
    /// is for) is caught the same way — different prologue, failed MAC.
    #[tokio::test]
    async fn test_regression_relay_tampering_of_dest_node_id_is_rejected() {
        let psk = [0x42u8; 32];
        let responder_kp = StaticKeypair::generate();
        let initiator_kp = StaticKeypair::generate();
        let nid_init: u64 = 0x1111;
        let nid_resp: u64 = 0x2222;

        let initiator = HandshakeHandler::new(nid_init, initiator_kp, psk);
        let responder = HandshakeHandler::new(nid_resp, responder_kp.clone(), psk);

        let (genuine_payload, _rx) = initiator
            .begin_initiator(nid_resp, &responder_kp.public)
            .unwrap();
        // Confirm the genuine envelope was constructed as expected.
        let genuine = HandshakePayload::decode(genuine_payload).unwrap();
        assert_eq!(genuine.dest_node_id, nid_resp);

        // Relay rewrites dest to a different u64 but still addressed at us
        // (so the responder processes it). `local_node_id` of the
        // responder is nid_resp; we'll claim it's nid_resp but set the
        // prologue's dest side to something different by tampering.
        // Because `dest_node_id != self.local_node_id` would trigger the
        // forward branch instead, we re-target to nid_resp but also use
        // a prologue-shifted payload: the easiest way is to fabricate an
        // msg1 that claims dest=nid_resp but was sealed against a
        // different dest in the initiator's prologue.
        //
        // We emulate this by constructing a prologue-mismatched state on
        // the initiator side: start a second initiator handshake against
        // the responder but declare dest=nid_resp+1. The bytes addressed
        // to nid_resp+1 will not open against a responder with
        // `local_node_id = nid_resp` — but we force delivery by rewriting
        // dest=nid_resp in the wire envelope after construction.
        let (other_payload, _rx2) = initiator
            .begin_initiator(nid_resp + 1, &responder_kp.public)
            .unwrap();
        let other_parsed = HandshakePayload::decode(other_payload).unwrap();
        let tampered = encode_payload(nid_resp, nid_init, &other_parsed.noise_bytes);

        // Sanity: the responder *would* decode the envelope (dest matches
        // us), but the prologue it builds won't match what the initiator
        // actually used, so Noise rejects msg1.
        let dummy_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let actions = responder.handle_message(&tampered, dummy_addr);

        assert!(
            actions.is_empty(),
            "tampered dest_node_id must be rejected via Noise prologue \
             mismatch; got {} actions",
            actions.len()
        );
        // Clean up the second pending initiator we created just to get
        // byte material — no test assertion depends on it.
        initiator.cancel_initiator(nid_resp + 1);
        initiator.cancel_initiator(nid_resp);
    }

    /// Regression: `pending_initiator` is keyed only by `dest_node_id`.
    /// A second concurrent `begin_initiator` call for the same dest used
    /// to silently overwrite the first, so the first caller's oneshot was
    /// dropped and its Noise state leaked. On the caller side that
    /// showed up as an opaque channel-closed error, not a real handshake
    /// failure — making parallel retries unreliable.
    ///
    /// The fix rejects the second call with `CryptoError::Handshake` so
    /// the caller can choose to wait, cancel, or back off.
    #[test]
    fn test_regression_concurrent_begin_initiator_is_rejected() {
        let psk = [0x42u8; 32];
        let local_kp = StaticKeypair::generate();
        let responder_kp = StaticKeypair::generate();

        let handler = HandshakeHandler::new(0x1111, local_kp, psk);

        // First call succeeds.
        let first = handler.begin_initiator(0x2222, &responder_kp.public);
        assert!(first.is_ok(), "first concurrent call should succeed");
        let (_payload, _rx) = first.unwrap();

        // Second concurrent call to the same dest must be refused.
        let second = handler.begin_initiator(0x2222, &responder_kp.public);
        match second {
            Err(CryptoError::Handshake(msg)) => {
                assert!(
                    msg.contains("already in flight"),
                    "expected 'already in flight' in error, got: {}",
                    msg
                );
            }
            other => panic!("expected Err(Handshake), got {:?}", other.err()),
        }

        // A different dest is independent and still works.
        let other = handler.begin_initiator(0x3333, &responder_kp.public);
        assert!(other.is_ok(), "different dest must be independent");

        // After cancelling the first, a retry to 0x2222 works.
        handler.cancel_initiator(0x2222);
        let retry = handler.begin_initiator(0x2222, &responder_kp.public);
        assert!(retry.is_ok(), "retry after cancel should succeed");
    }
}
