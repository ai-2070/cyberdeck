"""
Integration tests for Net Python bindings with NLTP.

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

# Check if NLTP feature is available
try:
    from net import NltpKeypair, generate_nltp_keypair

    NLTP_AVAILABLE = True
except ImportError:
    NLTP_AVAILABLE = False

RUN_INTEGRATION_TESTS = os.environ.get("RUN_INTEGRATION_TESTS") == "1"

skip_nltp = pytest.mark.skipif(
    not RUN_INTEGRATION_TESTS or not NLTP_AVAILABLE,
    reason="Set RUN_INTEGRATION_TESTS=1 and build with NLTP feature to run NLTP tests",
)


class TestNltpIntegration:
    """Integration tests for NLTP adapter (encrypted UDP transport)."""

    @skip_nltp
    def test_generate_keypair(self):
        """Test NLTP keypair generation."""
        keypair = generate_nltp_keypair()

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
        keypair2 = generate_nltp_keypair()
        assert keypair2.public_key != keypair.public_key
        assert keypair2.secret_key != keypair.secret_key

    @skip_nltp
    def test_exchange_events(self):
        """Test event exchange between initiator and responder."""
        # Generate keypair for responder
        responder_keypair = generate_nltp_keypair()

        # Generate shared PSK (32 bytes hex)
        psk = secrets.token_hex(32)

        # Create responder (binds first, waits for initiator)
        responder = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19100",
            nltp_peer_addr="127.0.0.1:19101",
            nltp_psk=psk,
            nltp_role="responder",
            nltp_secret_key=responder_keypair.secret_key,
            nltp_public_key=responder_keypair.public_key,
            nltp_reliability="light",
        )

        # Small delay to ensure responder is ready
        time.sleep(0.05)

        # Create initiator
        initiator = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19101",
            nltp_peer_addr="127.0.0.1:19100",
            nltp_psk=psk,
            nltp_role="initiator",
            nltp_peer_public_key=responder_keypair.public_key,
            nltp_reliability="light",
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

    @skip_nltp
    def test_batch_ingestion(self):
        """Test batch ingestion over NLTP."""
        responder_keypair = generate_nltp_keypair()
        psk = secrets.token_hex(32)

        responder = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19102",
            nltp_peer_addr="127.0.0.1:19103",
            nltp_psk=psk,
            nltp_role="responder",
            nltp_secret_key=responder_keypair.secret_key,
            nltp_public_key=responder_keypair.public_key,
        )

        time.sleep(0.05)

        initiator = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19103",
            nltp_peer_addr="127.0.0.1:19102",
            nltp_psk=psk,
            nltp_role="initiator",
            nltp_peer_public_key=responder_keypair.public_key,
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

    @skip_nltp
    def test_full_reliability_mode(self):
        """Test full reliability mode."""
        responder_keypair = generate_nltp_keypair()
        psk = secrets.token_hex(32)

        responder = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19104",
            nltp_peer_addr="127.0.0.1:19105",
            nltp_psk=psk,
            nltp_role="responder",
            nltp_secret_key=responder_keypair.secret_key,
            nltp_public_key=responder_keypair.public_key,
            nltp_reliability="full",
            nltp_heartbeat_interval_ms=1000,
            nltp_session_timeout_ms=10000,
        )

        time.sleep(0.05)

        initiator = Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19105",
            nltp_peer_addr="127.0.0.1:19104",
            nltp_psk=psk,
            nltp_role="initiator",
            nltp_peer_public_key=responder_keypair.public_key,
            nltp_reliability="full",
            nltp_heartbeat_interval_ms=1000,
            nltp_session_timeout_ms=10000,
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

    @skip_nltp
    def test_context_manager(self):
        """Test context manager support with NLTP."""
        responder_keypair = generate_nltp_keypair()
        psk = secrets.token_hex(32)

        with Net(
            num_shards=1,
            nltp_bind_addr="127.0.0.1:19106",
            nltp_peer_addr="127.0.0.1:19107",
            nltp_psk=psk,
            nltp_role="responder",
            nltp_secret_key=responder_keypair.secret_key,
            nltp_public_key=responder_keypair.public_key,
        ) as responder:
            time.sleep(0.05)

            with Net(
                num_shards=1,
                nltp_bind_addr="127.0.0.1:19107",
                nltp_peer_addr="127.0.0.1:19106",
                nltp_psk=psk,
                nltp_role="initiator",
                nltp_peer_public_key=responder_keypair.public_key,
            ) as initiator:
                time.sleep(0.2)

                initiator.ingest_raw(json.dumps({"context_manager": "test"}))
                time.sleep(0.3)

                response = responder.poll(limit=10)
                assert len(response) >= 1
