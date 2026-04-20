"""Tests for the subnet / subnet-policy kwargs on `NetMesh` (Stage F-3).

Single-node smoke tests: construct succeeds with various subnet
shapes, input validation rejects out-of-range level bytes and
malformed policy dicts. Multi-node subnet-local visibility enforcement
is covered by the Rust integration suite (`tests/subnet_*.rs`).
"""

from __future__ import annotations

import pytest

from net import IdentityError, NetMesh


PSK = "42" * 32


def _port(seed: int) -> str:
    return f"127.0.0.1:{29000 + seed}"


# -------------------------------------------------------------------------
# Subnet id kwarg
# -------------------------------------------------------------------------


def test_mesh_accepts_single_level_subnet() -> None:
    m = NetMesh(_port(1), PSK, subnet=[3])
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_mesh_accepts_four_level_subnet() -> None:
    m = NetMesh(_port(2), PSK, subnet=[1, 2, 3, 4])
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_mesh_accepts_no_subnet_kwarg() -> None:
    # Default: SubnetId::GLOBAL — everything should still work.
    m = NetMesh(_port(3), PSK)
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_subnet_empty_levels_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(_port(4), PSK, subnet=[])


def test_subnet_too_many_levels_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(_port(5), PSK, subnet=[1, 2, 3, 4, 5])


def test_subnet_byte_out_of_range_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(_port(6), PSK, subnet=[256])


# -------------------------------------------------------------------------
# Subnet policy kwarg
# -------------------------------------------------------------------------


def test_mesh_accepts_subnet_policy() -> None:
    policy = {
        "rules": [
            {
                "tag_prefix": "region:",
                "level": 0,
                "values": {"eu": 1, "us": 2, "apac": 3},
            },
            {
                "tag_prefix": "zone:",
                "level": 1,
                "values": {"a": 1, "b": 2, "c": 3},
            },
        ]
    }
    m = NetMesh(_port(7), PSK, subnet_policy=policy)
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_subnet_policy_with_no_rules() -> None:
    m = NetMesh(_port(8), PSK, subnet_policy={"rules": []})
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_subnet_policy_missing_rules_key_ok() -> None:
    # Bare `{}` is a policy with no rules — passes silently.
    m = NetMesh(_port(9), PSK, subnet_policy={})
    try:
        assert m.node_id != 0
    finally:
        m.shutdown()


def test_subnet_policy_level_out_of_range_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(
            _port(10),
            PSK,
            subnet_policy={
                "rules": [{"tag_prefix": "region:", "level": 4, "values": {}}]
            },
        )


def test_subnet_policy_rule_missing_tag_prefix_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(
            _port(11),
            PSK,
            subnet_policy={"rules": [{"level": 0, "values": {}}]},
        )


def test_subnet_policy_rules_wrong_type_rejected() -> None:
    with pytest.raises(TypeError):
        NetMesh(_port(12), PSK, subnet_policy={"rules": "not-a-list"})


def test_subnet_policy_value_out_of_range_rejected() -> None:
    with pytest.raises(IdentityError):
        NetMesh(
            _port(13),
            PSK,
            subnet_policy={
                "rules": [
                    {
                        "tag_prefix": "region:",
                        "level": 0,
                        "values": {"eu": 512},
                    }
                ]
            },
        )


# -------------------------------------------------------------------------
# Combined subnet + policy + signed caps flags
# -------------------------------------------------------------------------


def test_mesh_accepts_full_security_config() -> None:
    m = NetMesh(
        _port(14),
        PSK,
        identity_seed=b"\x01" * 32,
        subnet=[7, 3],
        subnet_policy={
            "rules": [
                {
                    "tag_prefix": "tier:",
                    "level": 2,
                    "values": {"prod": 1, "dev": 2},
                }
            ]
        },
        require_signed_capabilities=True,
        capability_gc_interval_ms=120_000,
    )
    try:
        assert len(m.entity_id) == 32
    finally:
        m.shutdown()
