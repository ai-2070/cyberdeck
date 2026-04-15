"""
Integration tests for Net Python bindings with Net.

Run tests:
  pytest tests/test_integration.py -v

Environment variables:
  RUN_INTEGRATION_TESTS - Set to "1" to run integration tests
"""

import json
import os
import secrets
import time

import pytest
from net import Net

# Check if Net feature is available
try:
    from net import NetKeypair, generate_net_keypair

    Net_AVAILABLE = True
except ImportError:
    Net_AVAILABLE = False

RUN_INTEGRATION_TESTS = os.environ.get("RUN_INTEGRATION_TESTS") == "1"

skip_net = pytest.mark.skipif(
    not RUN_INTEGRATION_TESTS or not Net_AVAILABLE,
    reason="Set RUN_INTEGRATION_TESTS=1 and build with Net feature to run Net tests",
)


class TestNetIntegration:
    """Integration tests for Net adapter (encrypted UDP transport)."""

    @skip_net
    def test_generate_keypair(self):
        """Test Net keypair generation."""
        keypair = generate_net_keypair()

        assert keypair is not None
        assert keypair.public_key is not None
        assert keypair.secret_key is not None

        # Keys should be 32 bytes hex-encoded (64 hex chars)
        assert len(keypair.public_key) == 64
        assert len(keypair.secret_key) == 64

        # Should be valid hex
        int(keypair.public_key, 16)  # Raises ValueError if not valid hex
        int(keypair.secret_key, 16)

        # Each call should generate different keypairs
        keypair2 = generate_net_keypair()
        assert keypair2.public_key != keypair.public_key
        assert keypair2.secret_key != keypair.secret_key

    @skip_net
    def test_exchange_events(self):
        """Test event exchange between initiator and responder."""
        # Generate keypair for responder
        responder_keypair = generate_net_keypair()

        # Generate shared PSK (32 bytes hex)
        psk = secrets.token_hex(32)

        # Create responder (binds first, waits for initiator)
        responder = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19100",
            net_peer_addr="127.0.0.1:19101",
            net_psk=psk,
            net_role="responder",
            net_secret_key=responder_keypair.secret_key,
            net_public_key=responder_keypair.public_key,
            net_reliability="light",
        )

        # Small delay to ensure responder is ready
        time.sleep(0.05)

        # Create initiator
        initiator = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19101",
            net_peer_addr="127.0.0.1:19100",
            net_psk=psk,
            net_role="initiator",
            net_peer_public_key=responder_keypair.public_key,
            net_reliability="light",
        )

        try:
            # Wait for handshake to complete
            time.sleep(0.2)

            # Initiator sends events to responder
            for i in range(5):
                initiator.ingest_raw(json.dumps({"source": "initiator", "index": i}))

            # Responder sends events to initiator
            for i in range(5):
                responder.ingest_raw(json.dumps({"source": "responder", "index": i}))

            # Wait for events to propagate
            time.sleep(0.5)

            # Poll from both sides
            initiator_events = initiator.poll(limit=100)
            responder_events = responder.poll(limit=100)

            # Both should have received events
            assert len(initiator_events) > 0, "Initiator should have received events"
            assert len(responder_events) > 0, "Responder should have received events"
        finally:
            initiator.shutdown()
            responder.shutdown()

    @skip_net
    def test_batch_ingestion(self):
        """Test batch ingestion over Net."""
        responder_keypair = generate_net_keypair()
        psk = secrets.token_hex(32)

        responder = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19102",
            net_peer_addr="127.0.0.1:19103",
            net_psk=psk,
            net_role="responder",
            net_secret_key=responder_keypair.secret_key,
            net_public_key=responder_keypair.public_key,
        )

        time.sleep(0.05)

        initiator = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19103",
            net_peer_addr="127.0.0.1:19102",
            net_psk=psk,
            net_role="initiator",
            net_peer_public_key=responder_keypair.public_key,
        )

        try:
            time.sleep(0.2)

            # Batch ingest
            events = [json.dumps({"batch_index": i}) for i in range(20)]
            count = initiator.ingest_raw_batch(events)
            assert count == 20

            time.sleep(0.5)

            response = responder.poll(limit=100)
            assert len(response) > 0, "Responder should have received batched events"
        finally:
            initiator.shutdown()
            responder.shutdown()

    @skip_net
    def test_full_reliability_mode(self):
        """Test full reliability mode."""
        responder_keypair = generate_net_keypair()
        psk = secrets.token_hex(32)

        responder = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19104",
            net_peer_addr="127.0.0.1:19105",
            net_psk=psk,
            net_role="responder",
            net_secret_key=responder_keypair.secret_key,
            net_public_key=responder_keypair.public_key,
            net_reliability="full",
            net_heartbeat_interval_ms=1000,
            net_session_timeout_ms=10000,
        )

        time.sleep(0.05)

        initiator = Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19105",
            net_peer_addr="127.0.0.1:19104",
            net_psk=psk,
            net_role="initiator",
            net_peer_public_key=responder_keypair.public_key,
            net_reliability="full",
            net_heartbeat_interval_ms=1000,
            net_session_timeout_ms=10000,
        )

        try:
            time.sleep(0.2)

            # Send events with full reliability
            for i in range(10):
                initiator.ingest_raw(json.dumps({"reliable": True, "seq": i}))

            time.sleep(0.5)

            response = responder.poll(limit=100)
            assert len(response) > 0, "Responder should have received reliable events"
        finally:
            initiator.shutdown()
            responder.shutdown()

    @skip_net
    def test_context_manager(self):
        """Test context manager support with Net."""
        responder_keypair = generate_net_keypair()
        psk = secrets.token_hex(32)

        with Net(
            num_shards=1,
            net_bind_addr="127.0.0.1:19106",
            net_peer_addr="127.0.0.1:19107",
            net_psk=psk,
            net_role="responder",
            net_secret_key=responder_keypair.secret_key,
            net_public_key=responder_keypair.public_key,
        ) as responder:
            time.sleep(0.05)

            with Net(
                num_shards=1,
                net_bind_addr="127.0.0.1:19107",
                net_peer_addr="127.0.0.1:19106",
                net_psk=psk,
                net_role="initiator",
                net_peer_public_key=responder_keypair.public_key,
            ) as initiator:
                time.sleep(0.2)

                initiator.ingest_raw(json.dumps({"context_manager": "test"}))
                time.sleep(0.3)

                response = responder.poll(limit=10)
                assert len(response) >= 1
