"""Tests for the capability-announcement + filter surface (Stage F-2).

Each node self-indexes its own announcement, so the single-node
roundtrip is a full contract test for the dict→core conversion plus
the filter predicate. Multi-node propagation is covered by the Rust
integration suite (`tests/capability_broadcast.rs`).
"""

from __future__ import annotations

import pytest

from net import NetMesh, normalize_gpu_vendor


PSK = "42" * 32


def _port(seed: int) -> str:
    return f"127.0.0.1:{28000 + seed}"


# -------------------------------------------------------------------------
# Self-match round-trip
# -------------------------------------------------------------------------


def test_announce_then_find_self_matches_on_tag() -> None:
    m = NetMesh(_port(1), PSK)
    try:
        m.announce_capabilities({"tags": ["gpu", "prod"]})
        peers = m.find_peers({"require_tags": ["gpu"]})
        assert m.node_id in peers
    finally:
        m.shutdown()


def test_find_peers_empty_when_filter_mismatches() -> None:
    m = NetMesh(_port(2), PSK)
    try:
        m.announce_capabilities({"tags": ["cpu"]})
        peers = m.find_peers({"require_tags": ["gpu"]})
        assert peers == []
    finally:
        m.shutdown()


def test_find_peers_without_announcement_is_empty() -> None:
    m = NetMesh(_port(3), PSK)
    try:
        peers = m.find_peers({"require_tags": ["anything"]})
        assert peers == []
    finally:
        m.shutdown()


# -------------------------------------------------------------------------
# Hardware filter round-trip
# -------------------------------------------------------------------------


def test_hardware_and_gpu_filter_matches() -> None:
    m = NetMesh(_port(4), PSK)
    try:
        m.announce_capabilities(
            {
                "hardware": {
                    "cpu_cores": 16,
                    "memory_mb": 65536,
                    "gpu": {
                        "vendor": "nvidia",
                        "model": "h100",
                        "vram_mb": 81920,
                    },
                },
                "tags": ["gpu"],
            }
        )
        peers = m.find_peers(
            {
                "require_gpu": True,
                "gpu_vendor": "nvidia",
                "min_vram_mb": 40000,
                "min_memory_mb": 32768,
            }
        )
        assert m.node_id in peers

        # Too-strict VRAM requirement should reject.
        peers_strict = m.find_peers({"min_vram_mb": 200_000})
        assert peers_strict == []
    finally:
        m.shutdown()


def test_model_and_tool_filter_matches() -> None:
    m = NetMesh(_port(5), PSK)
    try:
        m.announce_capabilities(
            {
                "models": [
                    {
                        "model_id": "llama-3.1-70b",
                        "family": "llama",
                        "parameters_b_x10": 700,
                        "context_length": 128_000,
                        "modalities": ["text", "code"],
                    }
                ],
                "tools": [{"tool_id": "sql_exec", "name": "SQL Exec"}],
            }
        )
        assert m.node_id in m.find_peers(
            {"require_models": ["llama-3.1-70b"]}
        )
        assert m.node_id in m.find_peers({"require_tools": ["sql_exec"]})
        assert m.node_id in m.find_peers(
            {"require_modalities": ["code"], "min_context_length": 100_000}
        )
        assert m.find_peers({"require_models": ["missing"]}) == []
    finally:
        m.shutdown()


def test_empty_announcement_still_self_indexes() -> None:
    m = NetMesh(_port(6), PSK)
    try:
        m.announce_capabilities({})
        # Empty filter matches any announcer in the index.
        peers = m.find_peers({})
        assert m.node_id in peers
    finally:
        m.shutdown()


# -------------------------------------------------------------------------
# Vendor normalization helper
# -------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        ("NVIDIA", "nvidia"),
        ("Nvidia", "nvidia"),
        ("amd", "amd"),
        ("Apple", "apple"),
        ("qualcomm", "qualcomm"),
        ("intel", "intel"),
        ("bogus", "unknown"),
        ("", "unknown"),
    ],
)
def test_normalize_gpu_vendor(raw: str, expected: str) -> None:
    assert normalize_gpu_vendor(raw) == expected


# -------------------------------------------------------------------------
# Input validation
# -------------------------------------------------------------------------


def test_announce_rejects_wrong_type_for_hardware() -> None:
    m = NetMesh(_port(7), PSK)
    try:
        with pytest.raises(TypeError):
            m.announce_capabilities({"hardware": "not-a-dict"})
    finally:
        m.shutdown()


def test_find_peers_rejects_wrong_type_for_require_tags() -> None:
    m = NetMesh(_port(8), PSK)
    try:
        with pytest.raises(TypeError):
            m.find_peers({"require_tags": "gpu"})  # must be list
    finally:
        m.shutdown()
