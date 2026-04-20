// Package net — encrypted-UDP mesh transport + per-peer streams +
// channels (distributed pub/sub).
//
// Compiled into the Rust cdylib when the core is built with
// `--features net`. Mirrors the Rust SDK's `Mesh` type rather than
// the full core `MeshNode` surface — just the common path needed by
// apps: handshake, per-peer streams with backpressure, channels,
// shard receive.

package net

/*
#include "net.h"
#include <stdlib.h>
#include <string.h>
*/
import "C"

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"runtime"
	"sync"
	"unsafe"
)

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

var (
	ErrMeshInit         = errors.New("mesh init failed")
	ErrMeshHandshake    = errors.New("mesh handshake failed")
	ErrBackpressure     = errors.New("stream backpressure")
	ErrNotConnected     = errors.New("stream not connected")
	ErrMeshTransport    = errors.New("mesh transport error")
	ErrChannel          = errors.New("channel error")
	ErrChannelAuth      = errors.New("channel: unauthorized")
)

func meshErrorFromCode(code C.int) error {
	switch code {
	case 0:
		return nil
	case -1:
		return ErrNullPointer
	case -2:
		return ErrInvalidUTF8
	case -3:
		return ErrInvalidJSON
	case -110:
		return ErrMeshInit
	case -111:
		return ErrMeshHandshake
	case -112:
		return ErrBackpressure
	case -113:
		return ErrNotConnected
	case -114:
		return ErrMeshTransport
	case -115:
		return ErrChannel
	case -116:
		return ErrChannelAuth
	default:
		return fmt.Errorf("mesh unknown error (code %d)", code)
	}
}

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

// MeshConfig configures a new mesh node.
type MeshConfig struct {
	BindAddr         string `json:"bind_addr"`
	// Hex-encoded 32-byte pre-shared key.
	PskHex           string `json:"psk_hex"`
	HeartbeatMs      uint64 `json:"heartbeat_ms,omitempty"`
	SessionTimeoutMs uint64 `json:"session_timeout_ms,omitempty"`
	NumShards        uint16 `json:"num_shards,omitempty"`

	// CapabilityGCIntervalMs controls how often the local capability
	// index evicts stale announcements. Leave zero for the core
	// default.
	CapabilityGCIntervalMs uint64 `json:"capability_gc_interval_ms,omitempty"`
	// RequireSignedCapabilities rejects unsigned announcements when
	// true. Leave nil/false for the core default (accept unsigned in v1).
	RequireSignedCapabilities bool `json:"require_signed_capabilities,omitempty"`

	// Subnet constrains the node to a hierarchical subnet (1–4 bytes
	// each 0–255). Empty / nil means `SubnetId::GLOBAL`.
	Subnet []uint32 `json:"subnet,omitempty"`
	// SubnetPolicy derives a subnet from capability tags at runtime
	// (alternative / complement to `Subnet`). See
	// `docs/SDK_SECURITY_SURFACE_PLAN.md`.
	SubnetPolicy *SubnetPolicy `json:"subnet_policy,omitempty"`

	// IdentitySeedHex reproduces a mesh keypair from a 32-byte seed
	// (64 hex chars). Matches `IdentityFromSeed(sameSeed)` so tokens
	// issued to that identity's `EntityID` work for this mesh.
	IdentitySeedHex string `json:"identity_seed_hex,omitempty"`
}

// StreamConfig configures an opened mesh stream.
type StreamConfig struct {
	// Reliability: "reliable" | "fire_and_forget" (default).
	Reliability string `json:"reliability,omitempty"`
	// WindowBytes sets the initial send-credit window. 0 disables
	// backpressure entirely. Default: 64 KiB.
	WindowBytes    uint32 `json:"window_bytes,omitempty"`
	FairnessWeight uint8  `json:"fairness_weight,omitempty"`
}

// StreamStats is a snapshot of a live stream's stats.
type StreamStats struct {
	TxSeq                uint64 `json:"tx_seq"`
	RxSeq                uint64 `json:"rx_seq"`
	InboundPending       uint64 `json:"inbound_pending"`
	LastActivityNs       uint64 `json:"last_activity_ns"`
	Active               bool   `json:"active"`
	BackpressureEvents   uint64 `json:"backpressure_events"`
	TxCreditRemaining    uint32 `json:"tx_credit_remaining"`
	TxWindow             uint32 `json:"tx_window"`
	CreditGrantsReceived uint64 `json:"credit_grants_received"`
	CreditGrantsSent     uint64 `json:"credit_grants_sent"`
}

// ChannelConfig mirrors the core `ChannelConfig`.
type ChannelConfig struct {
	Name         string `json:"name"`
	Visibility   string `json:"visibility,omitempty"` // "subnet-local" | "parent-visible" | "exported" | "global"
	Reliable     bool   `json:"reliable,omitempty"`
	RequireToken bool   `json:"require_token,omitempty"`
	Priority     uint8  `json:"priority,omitempty"`
	MaxRatePps   uint32 `json:"max_rate_pps,omitempty"`

	// PublishCaps restricts who may publish on this channel. Set
	// when the publisher wants to limit publishing to its own
	// `CapabilitySet` satisfying the filter.
	PublishCaps *CapabilityFilter `json:"publish_caps,omitempty"`
	// SubscribeCaps restricts who may subscribe. Subscribers whose
	// announced caps miss this filter are rejected with
	// `ErrChannelAuth`.
	SubscribeCaps *CapabilityFilter `json:"subscribe_caps,omitempty"`
}

// PublishConfig mirrors the core `PublishConfig`.
type PublishConfig struct {
	Reliability string `json:"reliability,omitempty"` // "reliable" | "fire_and_forget"
	OnFailure   string `json:"on_failure,omitempty"`  // "best_effort" | "fail_fast" | "collect"
	MaxInflight uint32 `json:"max_inflight,omitempty"`
}

// PublishFailure carries one per-peer error from a Publish call.
type PublishFailure struct {
	NodeID  uint64 `json:"node_id"`
	Message string `json:"message"`
}

// PublishReport is returned by Publish.
type PublishReport struct {
	Attempted uint32           `json:"attempted"`
	Delivered uint32           `json:"delivered"`
	Errors    []PublishFailure `json:"errors"`
}

// RecvdEvent is one event drained from a shard inbox.
type RecvdEvent struct {
	ID           string
	Payload      []byte
	InsertionTs  uint64
	ShardID      uint16
}

// ---------------------------------------------------------------------------
// MeshNode
// ---------------------------------------------------------------------------

// MeshNode is a multi-peer encrypted mesh handle.
type MeshNode struct {
	mu     sync.RWMutex
	handle *C.net_meshnode_t
}

// NewMeshNode opens a mesh node. Call Shutdown to cleanly tear down.
func NewMeshNode(cfg MeshConfig) (*MeshNode, error) {
	data, err := json.Marshal(cfg)
	if err != nil {
		return nil, fmt.Errorf("marshal config: %w", err)
	}
	cCfg := C.CString(string(data))
	defer C.free(unsafe.Pointer(cCfg))

	var handle *C.net_meshnode_t
	code := C.net_mesh_new(cCfg, &handle)
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	m := &MeshNode{handle: handle}
	runtime.SetFinalizer(m, (*MeshNode).free)
	return m, nil
}

func (m *MeshNode) free() {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.handle != nil {
		C.net_mesh_free(m.handle)
		m.handle = nil
		runtime.SetFinalizer(m, nil)
	}
}

// Shutdown gracefully tears down the node. Idempotent.
func (m *MeshNode) Shutdown() error {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.handle == nil {
		return nil
	}
	code := C.net_mesh_shutdown(m.handle)
	C.net_mesh_free(m.handle)
	m.handle = nil
	runtime.SetFinalizer(m, nil)
	return meshErrorFromCode(code)
}

// PublicKey returns this node's Noise static public key, hex-encoded.
func (m *MeshNode) PublicKey() (string, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return "", ErrShuttingDown
	}
	var out *C.char
	var outLen C.size_t
	code := C.net_mesh_public_key_hex(m.handle, &out, &outLen)
	if err := meshErrorFromCode(code); err != nil {
		return "", err
	}
	defer C.net_free_string(out)
	return C.GoStringN(out, C.int(outLen)), nil
}

// NodeID returns this node's u64 id.
func (m *MeshNode) NodeID() uint64 {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return 0
	}
	return uint64(C.net_mesh_node_id(m.handle))
}

// EntityID returns this node's 32-byte ed25519 entity id. Matches
// `IdentityFromSeed(seed).EntityID()` when the mesh was constructed
// with `MeshConfig{IdentitySeedHex: hex.EncodeToString(seed), ...}`.
func (m *MeshNode) EntityID() ([]byte, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return nil, ErrShuttingDown
	}
	out := make([]byte, 32)
	code := C.net_mesh_entity_id(m.handle, (*C.uint8_t)(unsafe.Pointer(&out[0])))
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	return out, nil
}

// Connect (initiator). Blocks until the handshake completes.
func (m *MeshNode) Connect(peerAddr, peerPubkeyHex string, peerNodeID uint64) error {
	cAddr := C.CString(peerAddr)
	defer C.free(unsafe.Pointer(cAddr))
	cPk := C.CString(peerPubkeyHex)
	defer C.free(unsafe.Pointer(cPk))
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	code := C.net_mesh_connect(m.handle, cAddr, cPk, C.uint64_t(peerNodeID))
	return meshErrorFromCode(code)
}

// Accept an incoming connection (responder). Returns the peer's wire address.
func (m *MeshNode) Accept(peerNodeID uint64) (string, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return "", ErrShuttingDown
	}
	var out *C.char
	var outLen C.size_t
	code := C.net_mesh_accept(m.handle, C.uint64_t(peerNodeID), &out, &outLen)
	if err := meshErrorFromCode(code); err != nil {
		return "", err
	}
	defer C.net_free_string(out)
	return C.GoStringN(out, C.int(outLen)), nil
}

// Start the receive loop, heartbeats, and router.
func (m *MeshNode) Start() error {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	return meshErrorFromCode(C.net_mesh_start(m.handle))
}

// ---------------------------------------------------------------------------
// Streams
// ---------------------------------------------------------------------------

// MeshStream is an opaque handle to an open per-peer stream.
type MeshStream struct {
	mu     sync.RWMutex
	handle *C.net_mesh_stream_t
	// Parent node — kept alongside the stream so Send calls can
	// reach the owning runtime. The stream's own lifetime is bounded
	// by the node's.
	node *MeshNode
}

// OpenStream opens (or looks up) a stream to a connected peer.
// Repeated calls for the same (peer, streamID) are idempotent;
// first-open wins and later differing configs are logged and ignored.
func (m *MeshNode) OpenStream(peerNodeID, streamID uint64, cfg StreamConfig) (*MeshStream, error) {
	data, err := json.Marshal(cfg)
	if err != nil {
		return nil, fmt.Errorf("marshal stream cfg: %w", err)
	}
	var cCfg *C.char
	if string(data) != "{}" {
		cCfg = C.CString(string(data))
		defer C.free(unsafe.Pointer(cCfg))
	}
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return nil, ErrShuttingDown
	}
	var handle *C.net_mesh_stream_t
	code := C.net_mesh_open_stream(m.handle, C.uint64_t(peerNodeID), C.uint64_t(streamID), cCfg, &handle)
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	s := &MeshStream{handle: handle, node: m}
	runtime.SetFinalizer(s, (*MeshStream).free)
	return s, nil
}

func (s *MeshStream) free() {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.handle != nil {
		C.net_mesh_stream_free(s.handle)
		s.handle = nil
		runtime.SetFinalizer(s, nil)
	}
}

// Close releases the stream handle. Idempotent.
func (s *MeshStream) Close() {
	s.free()
}

// payloadPtrs builds the parallel (pointers, lengths) arrays the C
// ABI expects. cgo forbids passing Go pointers that transitively hold
// Go pointers; the outer arrays must therefore live in C memory. The
// payloads themselves (individual `[]byte`) are Go memory and get
// passed in as single-level pointers, which cgo allows — we pin them
// with `runtime.Pinner` so the GC can't move them during the C call.
//
// Returns a releaser that must be called after the C call returns to
// free the C allocations and unpin the payloads.
func payloadPtrs(payloads [][]byte) (
	pointers **C.uint8_t,
	lens *C.size_t,
	count C.size_t,
	release func(),
) {
	n := len(payloads)
	if n == 0 {
		return nil, nil, 0, func() {}
	}
	ptrBytes := C.size_t(n) * C.size_t(unsafe.Sizeof((*C.uint8_t)(nil)))
	lenBytes := C.size_t(n) * C.size_t(unsafe.Sizeof(C.size_t(0)))
	ptrArr := (*[1 << 28]*C.uint8_t)(C.malloc(ptrBytes))[:n:n]
	lenArr := (*[1 << 28]C.size_t)(C.malloc(lenBytes))[:n:n]
	var pinner runtime.Pinner
	for i, p := range payloads {
		if len(p) == 0 {
			// Any non-nil pointer is fine — C side gates on len.
			ptrArr[i] = (*C.uint8_t)(unsafe.Pointer(&ptrArr[0]))
			lenArr[i] = 0
		} else {
			pinner.Pin(&p[0])
			ptrArr[i] = (*C.uint8_t)(unsafe.Pointer(&p[0]))
			lenArr[i] = C.size_t(len(p))
		}
	}
	pointers = (**C.uint8_t)(unsafe.Pointer(&ptrArr[0]))
	lens = (*C.size_t)(unsafe.Pointer(&lenArr[0]))
	count = C.size_t(n)
	release = func() {
		pinner.Unpin()
		C.free(unsafe.Pointer(pointers))
		C.free(unsafe.Pointer(lens))
	}
	return
}

// Send a batch of payloads on the stream. Returns ErrBackpressure
// when the window is full (nothing sent — caller decides to drop /
// retry / buffer), ErrNotConnected when the peer is gone, or a
// transport error.
//
// Holds the stream AND node read-locks through the C call so a
// concurrent Close/Shutdown can't race the native handles into a
// use-after-free. Concurrent sends run in parallel; Close waits.
func (s *MeshStream) Send(payloads [][]byte) error {
	ptrs, lens, count, release := payloadPtrs(payloads)
	defer release()
	s.mu.RLock()
	defer s.mu.RUnlock()
	n := s.node
	if s.handle == nil || n == nil {
		return ErrShuttingDown
	}
	n.mu.RLock()
	defer n.mu.RUnlock()
	if n.handle == nil {
		return ErrShuttingDown
	}
	code := C.net_mesh_send(s.handle, ptrs, lens, count, n.handle)
	return meshErrorFromCode(code)
}

// SendWithRetry absorbs ErrBackpressure with exponential backoff up
// to `maxRetries`. Other errors propagate immediately.
func (s *MeshStream) SendWithRetry(payloads [][]byte, maxRetries uint32) error {
	ptrs, lens, count, release := payloadPtrs(payloads)
	defer release()
	s.mu.RLock()
	defer s.mu.RUnlock()
	n := s.node
	if s.handle == nil || n == nil {
		return ErrShuttingDown
	}
	n.mu.RLock()
	defer n.mu.RUnlock()
	if n.handle == nil {
		return ErrShuttingDown
	}
	code := C.net_mesh_send_with_retry(s.handle, ptrs, lens, count, C.uint32_t(maxRetries), n.handle)
	return meshErrorFromCode(code)
}

// SendBlocking retries ErrBackpressure up to ~13 min worst case.
func (s *MeshStream) SendBlocking(payloads [][]byte) error {
	ptrs, lens, count, release := payloadPtrs(payloads)
	defer release()
	s.mu.RLock()
	defer s.mu.RUnlock()
	n := s.node
	if s.handle == nil || n == nil {
		return ErrShuttingDown
	}
	n.mu.RLock()
	defer n.mu.RUnlock()
	if n.handle == nil {
		return ErrShuttingDown
	}
	code := C.net_mesh_send_blocking(s.handle, ptrs, lens, count, n.handle)
	return meshErrorFromCode(code)
}

// StreamStats returns a snapshot. `nil` if the stream isn't open.
func (m *MeshNode) StreamStats(peerNodeID, streamID uint64) (*StreamStats, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return nil, ErrShuttingDown
	}
	var out *C.char
	var outLen C.size_t
	code := C.net_mesh_stream_stats(m.handle, C.uint64_t(peerNodeID), C.uint64_t(streamID), &out, &outLen)
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	defer C.net_free_string(out)
	js := C.GoStringN(out, C.int(outLen))
	if js == "null" {
		return nil, nil
	}
	var s StreamStats
	if err := json.Unmarshal([]byte(js), &s); err != nil {
		return nil, fmt.Errorf("decode stream stats: %w", err)
	}
	return &s, nil
}

// ---------------------------------------------------------------------------
// Recv
// ---------------------------------------------------------------------------

type recvEventWire struct {
	ID          string `json:"id"`
	PayloadB64  string `json:"payload_b64"`
	InsertionTs uint64 `json:"insertion_ts"`
	ShardID     uint16 `json:"shard_id"`
}

// RecvShard drains up to `limit` events from a specific inbound shard.
func (m *MeshNode) RecvShard(shardID uint16, limit uint32) ([]RecvdEvent, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return nil, ErrShuttingDown
	}
	var out *C.char
	var outLen C.size_t
	code := C.net_mesh_recv_shard(m.handle, C.uint16_t(shardID), C.uint32_t(limit), &out, &outLen)
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	defer C.net_free_string(out)
	js := C.GoStringN(out, C.int(outLen))
	var wire []recvEventWire
	if err := json.Unmarshal([]byte(js), &wire); err != nil {
		return nil, fmt.Errorf("decode recv_shard: %w", err)
	}
	events := make([]RecvdEvent, 0, len(wire))
	for _, w := range wire {
		payload, err := base64.StdEncoding.DecodeString(w.PayloadB64)
		if err != nil {
			return nil, fmt.Errorf("decode payload: %w", err)
		}
		events = append(events, RecvdEvent{
			ID:          w.ID,
			Payload:     payload,
			InsertionTs: w.InsertionTs,
			ShardID:     w.ShardID,
		})
	}
	return events, nil
}

// ---------------------------------------------------------------------------
// Channels
// ---------------------------------------------------------------------------

// RegisterChannel installs a channel config on this node. Subscribers
// must pass the publisher-side ACL before being added to the roster.
func (m *MeshNode) RegisterChannel(cfg ChannelConfig) error {
	data, err := json.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("marshal channel cfg: %w", err)
	}
	cCfg := C.CString(string(data))
	defer C.free(unsafe.Pointer(cCfg))
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	return meshErrorFromCode(C.net_mesh_register_channel(m.handle, cCfg))
}

// SubscribeChannel joins `channel` on `publisherNodeID`. Blocks until
// the Ack arrives. `ErrChannelAuth` when the publisher rejected as
// unauthorized, `ErrChannel` for other rejections.
func (m *MeshNode) SubscribeChannel(publisherNodeID uint64, channel string) error {
	cCh := C.CString(channel)
	defer C.free(unsafe.Pointer(cCh))
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	return meshErrorFromCode(C.net_mesh_subscribe_channel(m.handle, C.uint64_t(publisherNodeID), cCh))
}

// SubscribeChannelWithToken subscribes to `channel` on
// `publisherNodeID` while presenting a serialized `PermissionToken`
// (typically 159 bytes — whatever `Identity.IssueToken` returned).
// Required when the publisher set `RequireToken=true` or when the
// subscriber's announced caps don't satisfy the publisher's
// `SubscribeCaps` filter.
//
// Malformed / truncated token bytes return `ErrTokenInvalidFormat`
// before any network I/O. Signature-tampered tokens surface as
// `ErrChannelAuth` (the publisher rejects the request).
func (m *MeshNode) SubscribeChannelWithToken(
	publisherNodeID uint64,
	channel string,
	token []byte,
) error {
	if len(token) == 0 {
		return ErrTokenInvalidFormat
	}
	cCh := C.CString(channel)
	defer C.free(unsafe.Pointer(cCh))
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	code := C.net_mesh_subscribe_channel_with_token(
		m.handle,
		C.uint64_t(publisherNodeID),
		cCh,
		(*C.uint8_t)(unsafe.Pointer(&token[0])),
		C.size_t(len(token)),
	)
	// Map token errors first so callers can distinguish them from
	// the channel/auth code range.
	switch code {
	case -121, -122, -123, -124, -125, -126, -127:
		return identityErrorFromCode(code)
	}
	return meshErrorFromCode(code)
}

// UnsubscribeChannel is the idempotent counterpart of SubscribeChannel.
func (m *MeshNode) UnsubscribeChannel(publisherNodeID uint64, channel string) error {
	cCh := C.CString(channel)
	defer C.free(unsafe.Pointer(cCh))
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return ErrShuttingDown
	}
	return meshErrorFromCode(C.net_mesh_unsubscribe_channel(m.handle, C.uint64_t(publisherNodeID), cCh))
}

// Publish fans one payload to every subscriber of `channel`.
func (m *MeshNode) Publish(channel string, payload []byte, cfg PublishConfig) (*PublishReport, error) {
	cCh := C.CString(channel)
	defer C.free(unsafe.Pointer(cCh))
	data, err := json.Marshal(cfg)
	if err != nil {
		return nil, fmt.Errorf("marshal publish cfg: %w", err)
	}
	var cCfg *C.char
	if string(data) != "{}" {
		cCfg = C.CString(string(data))
		defer C.free(unsafe.Pointer(cCfg))
	}
	var ptr *C.uint8_t
	var ln C.size_t
	if len(payload) > 0 {
		ptr = (*C.uint8_t)(unsafe.Pointer(&payload[0]))
		ln = C.size_t(len(payload))
	}
	var out *C.char
	var outLen C.size_t
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.handle == nil {
		return nil, ErrShuttingDown
	}
	code := C.net_mesh_publish(m.handle, cCh, ptr, ln, cCfg, &out, &outLen)
	runtime.KeepAlive(payload)
	if err := meshErrorFromCode(code); err != nil {
		return nil, err
	}
	defer C.net_free_string(out)
	js := C.GoStringN(out, C.int(outLen))
	var report PublishReport
	if err := json.Unmarshal([]byte(js), &report); err != nil {
		return nil, fmt.Errorf("decode publish report: %w", err)
	}
	return &report, nil
}
