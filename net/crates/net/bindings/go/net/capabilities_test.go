// Tests for the capability announcement + filter surface (Stage G-2).
//
// Each node self-indexes its own announcement, so single-node
// round-trip covers the dict→core conversion plus filter predicate
// end-to-end. Multi-node propagation is covered by the Rust
// integration suite (`tests/capability_broadcast.rs`).

package net

import (
	"slices"
	"testing"
)

func newMeshForCaps(t *testing.T) *MeshNode {
	t.Helper()
	addr := reserveLocalUDPPort(t)
	m, err := NewMeshNode(MeshConfig{BindAddr: addr, PskHex: meshPsk})
	if err != nil {
		t.Fatalf("new mesh: %v", err)
	}
	return m
}

// ---------------------------------------------------------------------------
// Self-match round-trip
// ---------------------------------------------------------------------------

func TestAnnounce_ThenFind_SelfMatchesOnTag(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	if err := m.AnnounceCapabilities(CapabilitySet{Tags: []string{"gpu", "prod"}}); err != nil {
		t.Fatalf("announce: %v", err)
	}
	peers, err := m.FindPeers(CapabilityFilter{RequireTags: []string{"gpu"}})
	if err != nil {
		t.Fatalf("find_peers: %v", err)
	}
	if !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("own node id missing from find_peers result: %v", peers)
	}
}

func TestFindPeers_EmptyWhenFilterMismatches(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	if err := m.AnnounceCapabilities(CapabilitySet{Tags: []string{"cpu"}}); err != nil {
		t.Fatalf("announce: %v", err)
	}
	peers, err := m.FindPeers(CapabilityFilter{RequireTags: []string{"gpu"}})
	if err != nil {
		t.Fatalf("find_peers: %v", err)
	}
	if len(peers) != 0 {
		t.Fatalf("expected empty peer list, got %v", peers)
	}
}

func TestFindPeers_WithoutAnnouncement_IsEmpty(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	peers, err := m.FindPeers(CapabilityFilter{RequireTags: []string{"anything"}})
	if err != nil {
		t.Fatalf("find_peers: %v", err)
	}
	if len(peers) != 0 {
		t.Fatalf("expected empty peer list, got %v", peers)
	}
}

// ---------------------------------------------------------------------------
// Hardware / model / tool filters
// ---------------------------------------------------------------------------

func TestHardwareAndGpuFilter_Matches(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	err := m.AnnounceCapabilities(CapabilitySet{
		Hardware: &HardwareCaps{
			CPUCores: 16,
			MemoryMB: 65536,
			GPU:      &GPUInfo{Vendor: "nvidia", Model: "h100", VRAMMB: 81920},
		},
		Tags: []string{"gpu"},
	})
	if err != nil {
		t.Fatalf("announce: %v", err)
	}

	peers, err := m.FindPeers(CapabilityFilter{
		RequireGPU:  true,
		GPUVendor:   "nvidia",
		MinVRAMMB:   40000,
		MinMemoryMB: 32768,
	})
	if err != nil {
		t.Fatalf("find_peers: %v", err)
	}
	if !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("own id missing: %v", peers)
	}

	strict, err := m.FindPeers(CapabilityFilter{MinVRAMMB: 200_000})
	if err != nil {
		t.Fatalf("find_peers strict: %v", err)
	}
	if len(strict) != 0 {
		t.Fatalf("expected strict filter to miss, got %v", strict)
	}
}

func TestModelAndToolFilter_Matches(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	err := m.AnnounceCapabilities(CapabilitySet{
		Models: []ModelCaps{{
			ModelID:       "llama-3.1-70b",
			Family:        "llama",
			ParametersBx10: 700,
			ContextLength: 128_000,
			Modalities:    []string{"text", "code"},
		}},
		Tools: []ToolCaps{{ToolID: "sql_exec", Name: "SQL Exec"}},
	})
	if err != nil {
		t.Fatalf("announce: %v", err)
	}

	peers, err := m.FindPeers(CapabilityFilter{RequireModels: []string{"llama-3.1-70b"}})
	if err != nil || !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("model filter missed own node: err=%v peers=%v", err, peers)
	}
	peers, err = m.FindPeers(CapabilityFilter{RequireTools: []string{"sql_exec"}})
	if err != nil || !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("tool filter missed own node: err=%v peers=%v", err, peers)
	}
	peers, err = m.FindPeers(CapabilityFilter{
		RequireModalities: []string{"code"},
		MinContextLength:  100_000,
	})
	if err != nil || !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("modality filter missed own node: err=%v peers=%v", err, peers)
	}

	missing, err := m.FindPeers(CapabilityFilter{RequireModels: []string{"missing"}})
	if err != nil {
		t.Fatalf("find_peers missing-model query: %v", err)
	}
	if len(missing) != 0 {
		t.Fatalf("expected no match, got %v", missing)
	}
}

func TestEmptyAnnouncement_StillSelfIndexes(t *testing.T) {
	m := newMeshForCaps(t)
	defer m.Shutdown()

	if err := m.AnnounceCapabilities(CapabilitySet{}); err != nil {
		t.Fatalf("announce: %v", err)
	}
	peers, err := m.FindPeers(CapabilityFilter{})
	if err != nil {
		t.Fatalf("find_peers: %v", err)
	}
	if !slices.Contains(peers, m.NodeID()) {
		t.Fatalf("expected own node indexed, got %v", peers)
	}
}

// ---------------------------------------------------------------------------
// Vendor normalization helper
// ---------------------------------------------------------------------------

func TestNormalizeGPUVendor(t *testing.T) {
	cases := []struct {
		raw, want string
	}{
		{"NVIDIA", "nvidia"},
		{"Nvidia", "nvidia"},
		{"amd", "amd"},
		{"Apple", "apple"},
		{"qualcomm", "qualcomm"},
		{"intel", "intel"},
		{"bogus", "unknown"},
		{"", "unknown"},
	}
	for _, c := range cases {
		got, err := NormalizeGPUVendor(c.raw)
		if err != nil {
			t.Fatalf("normalize %q: %v", c.raw, err)
		}
		if got != c.want {
			t.Fatalf("normalize %q: want %q, got %q", c.raw, c.want, got)
		}
	}
}

// ---------------------------------------------------------------------------
// Constructor kwargs (capability_gc + signed)
// ---------------------------------------------------------------------------

func TestMeshConfig_AcceptsCapabilityKwargs(t *testing.T) {
	addr := reserveLocalUDPPort(t)
	m, err := NewMeshNode(MeshConfig{
		BindAddr:                  addr,
		PskHex:                    meshPsk,
		CapabilityGCIntervalMs:    120_000,
		RequireSignedCapabilities: true,
	})
	if err != nil {
		t.Fatalf("new mesh with cap kwargs: %v", err)
	}
	defer m.Shutdown()
	if m.NodeID() == 0 {
		t.Fatal("node id unset")
	}
}
