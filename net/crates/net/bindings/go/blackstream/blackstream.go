// Package blackstream provides Go bindings for the Blackstream high-performance event bus.
//
// Blackstream is designed for AI runtime workloads, providing high-throughput event
// ingestion and consumption with support for the BLTP encrypted UDP backend.
//
// # Quick Start
//
//	bus, err := blackstream.New(nil)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer bus.Shutdown()
//
//	// Ingest events
//	err = bus.IngestRaw(`{"token": "hello", "index": 0}`)
//
//	// Poll events
//	events, err := bus.Poll(100, "")
package blackstream

/*
#cgo LDFLAGS: -L${SRCDIR}/../../../target/release -lblackstream
#include "blackstream.h"
#include <stdlib.h>
*/
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
	"runtime"
	"sync"
	"unsafe"
)

// Error codes
var (
	ErrNullPointer     = errors.New("null pointer")
	ErrInvalidUTF8     = errors.New("invalid UTF-8")
	ErrInvalidJSON     = errors.New("invalid JSON")
	ErrInitFailed      = errors.New("initialization failed")
	ErrIngestionFailed = errors.New("ingestion failed")
	ErrPollFailed      = errors.New("poll failed")
	ErrBufferTooSmall  = errors.New("buffer too small")
	ErrShuttingDown    = errors.New("shutting down")
	ErrUnknown         = errors.New("unknown error")
)

func errorFromCode(code C.int) error {
	switch code {
	case 0:
		return nil
	case -1:
		return ErrNullPointer
	case -2:
		return ErrInvalidUTF8
	case -3:
		return ErrInvalidJSON
	case -4:
		return ErrInitFailed
	case -5:
		return ErrIngestionFailed
	case -6:
		return ErrPollFailed
	case -7:
		return ErrBufferTooSmall
	case -8:
		return ErrShuttingDown
	default:
		return ErrUnknown
	}
}

// Config represents the event bus configuration.
type Config struct {
	// NumShards is the number of shards for parallel ingestion.
	// Defaults to the number of CPU cores.
	NumShards int `json:"num_shards,omitempty"`

	// RingBufferCapacity is the capacity of each shard's ring buffer.
	// Must be a power of 2. Defaults to 1048576 (1M events).
	RingBufferCapacity int `json:"ring_buffer_capacity,omitempty"`

	// BackpressureMode determines behavior when buffers are full.
	// Options: "DropNewest", "DropOldest", "FailProducer"
	BackpressureMode string `json:"backpressure_mode,omitempty"`

	// Bltp configuration for BLTP encrypted UDP backend.
	Bltp *BltpConfig `json:"bltp,omitempty"`
}

// BltpConfig represents BLTP encrypted UDP adapter configuration.
type BltpConfig struct {
	// BindAddr is the local bind address (e.g., "127.0.0.1:9000").
	BindAddr string `json:"bind_addr"`

	// PeerAddr is the remote peer address (e.g., "127.0.0.1:9001").
	PeerAddr string `json:"peer_addr"`

	// PSK is the hex-encoded 32-byte pre-shared key.
	PSK string `json:"psk"`

	// Role is the connection role: "initiator" or "responder".
	Role string `json:"role"`

	// PeerPublicKey is the hex-encoded peer's public key (required for initiator).
	PeerPublicKey string `json:"peer_public_key,omitempty"`

	// SecretKey is the hex-encoded secret key (required for responder).
	SecretKey string `json:"secret_key,omitempty"`

	// PublicKey is the hex-encoded public key (required for responder).
	PublicKey string `json:"public_key,omitempty"`

	// Reliability is the reliability mode: "none" (default), "light", or "full".
	Reliability string `json:"reliability,omitempty"`

	// HeartbeatIntervalMs is the heartbeat interval in milliseconds (default: 5000).
	HeartbeatIntervalMs int64 `json:"heartbeat_interval_ms,omitempty"`

	// SessionTimeoutMs is the session timeout in milliseconds (default: 30000).
	SessionTimeoutMs int64 `json:"session_timeout_ms,omitempty"`

	// BatchedIO enables batched I/O for Linux (default: false).
	BatchedIO bool `json:"batched_io,omitempty"`

	// PacketPoolSize is the packet pool size (default: 64).
	PacketPoolSize int `json:"packet_pool_size,omitempty"`
}

// BltpKeypair holds a generated keypair for BLTP.
type BltpKeypair struct {
	// PublicKey is the hex-encoded 32-byte public key.
	PublicKey string `json:"public_key"`

	// SecretKey is the hex-encoded 32-byte secret key.
	SecretKey string `json:"secret_key"`
}

// Event represents a stored event returned from polling.
type Event struct {
	// Raw is the raw JSON payload.
	Raw json.RawMessage `json:"raw"`
}

// PollResponse contains the results of a poll operation.
type PollResponse struct {
	// Events is the list of events.
	Events []json.RawMessage `json:"events"`

	// NextID is the cursor for the next poll.
	NextID string `json:"next_id,omitempty"`

	// HasMore indicates if there are more events available.
	HasMore bool `json:"has_more"`

	// Count is the number of events returned.
	Count int `json:"count"`
}

// Stats contains event bus statistics.
type Stats struct {
	EventsIngested   uint64 `json:"events_ingested"`
	EventsDropped    uint64 `json:"events_dropped"`
	BatchesDispathed uint64 `json:"batches_dispatched"`
}

// Blackstream is a high-performance event bus handle.
//
// All methods are thread-safe and can be called from multiple goroutines.
type Blackstream struct {
	mu     sync.RWMutex
	handle C.blackstream_handle_t
}

// New creates a new Blackstream event bus with the given configuration.
// Pass nil for default configuration.
func New(config *Config) (*Blackstream, error) {
	var configJSON *C.char
	if config != nil {
		data, err := json.Marshal(config)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal config: %w", err)
		}
		configJSON = C.CString(string(data))
		defer C.free(unsafe.Pointer(configJSON))
	}

	handle := C.blackstream_init(configJSON)
	if handle == nil {
		return nil, ErrInitFailed
	}

	bs := &Blackstream{handle: handle}
	runtime.SetFinalizer(bs, (*Blackstream).Shutdown)
	return bs, nil
}

// IngestRaw ingests a raw JSON string (fastest path).
//
// The JSON string is stored directly without parsing.
// This is the recommended method for high-throughput ingestion.
func (bs *Blackstream) IngestRaw(json string) error {
	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return ErrShuttingDown
	}

	cJSON := C.CString(json)
	defer C.free(unsafe.Pointer(cJSON))

	result := C.blackstream_ingest_raw(bs.handle, cJSON, C.size_t(len(json)))
	return errorFromCode(result)
}

// IngestRawBatch ingests multiple raw JSON strings in a batch.
//
// Returns the number of successfully ingested events.
func (bs *Blackstream) IngestRawBatch(jsons []string) int {
	if len(jsons) == 0 {
		return 0
	}

	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return 0
	}

	// Prepare C arrays
	cJSONs := make([]*C.char, len(jsons))
	cLens := make([]C.size_t, len(jsons))

	for i, j := range jsons {
		cJSONs[i] = C.CString(j)
		cLens[i] = C.size_t(len(j))
	}

	// Free all strings after the call
	defer func() {
		for _, cj := range cJSONs {
			C.free(unsafe.Pointer(cj))
		}
	}()

	result := C.blackstream_ingest_raw_batch(
		bs.handle,
		(**C.char)(unsafe.Pointer(&cJSONs[0])),
		(*C.size_t)(unsafe.Pointer(&cLens[0])),
		C.size_t(len(jsons)),
	)

	return int(result)
}

// Ingest ingests an event by marshaling the given value to JSON.
func (bs *Blackstream) Ingest(event interface{}) error {
	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	return bs.IngestRaw(string(data))
}

// IngestBatch ingests multiple events by marshaling them to JSON.
//
// Returns the number of successfully ingested events.
func (bs *Blackstream) IngestBatch(events []interface{}) int {
	jsons := make([]string, 0, len(events))
	for _, e := range events {
		data, err := json.Marshal(e)
		if err != nil {
			continue
		}
		jsons = append(jsons, string(data))
	}
	return bs.IngestRawBatch(jsons)
}

// Poll retrieves events from the bus.
//
// Parameters:
//   - limit: Maximum number of events to return.
//   - cursor: Optional cursor from a previous poll for pagination.
//
// Returns the poll response containing events and pagination info.
func (bs *Blackstream) Poll(limit int, cursor string) (*PollResponse, error) {
	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return nil, ErrShuttingDown
	}

	// Build request JSON
	request := map[string]interface{}{"limit": limit}
	if cursor != "" {
		request["cursor"] = cursor
	}
	requestData, _ := json.Marshal(request)
	cRequest := C.CString(string(requestData))
	defer C.free(unsafe.Pointer(cRequest))

	// Allocate output buffer (start with 64KB, grow if needed)
	bufferSize := 65536
	for {
		buffer := make([]byte, bufferSize)
		result := C.blackstream_poll(
			bs.handle,
			cRequest,
			(*C.char)(unsafe.Pointer(&buffer[0])),
			C.size_t(bufferSize),
		)

		if result == C.int(C.BLACKSTREAM_ERR_BUFFER_TOO_SMALL) {
			bufferSize *= 2
			if bufferSize > 64*1024*1024 { // 64MB max
				return nil, ErrBufferTooSmall
			}
			continue
		}

		if result < 0 {
			return nil, errorFromCode(result)
		}

		// Parse response
		var response PollResponse
		if err := json.Unmarshal(buffer[:result], &response); err != nil {
			return nil, fmt.Errorf("failed to parse response: %w", err)
		}

		return &response, nil
	}
}

// Stats returns event bus statistics.
func (bs *Blackstream) Stats() (*Stats, error) {
	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return nil, ErrShuttingDown
	}

	buffer := make([]byte, 4096)
	result := C.blackstream_stats(
		bs.handle,
		(*C.char)(unsafe.Pointer(&buffer[0])),
		C.size_t(len(buffer)),
	)

	if result < 0 {
		return nil, errorFromCode(result)
	}

	var stats Stats
	if err := json.Unmarshal(buffer[:result], &stats); err != nil {
		return nil, fmt.Errorf("failed to parse stats: %w", err)
	}

	return &stats, nil
}

// NumShards returns the number of shards.
func (bs *Blackstream) NumShards() int {
	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return 0
	}

	return int(C.blackstream_num_shards(bs.handle))
}

// Flush flushes all pending batches to the adapter.
func (bs *Blackstream) Flush() error {
	bs.mu.RLock()
	defer bs.mu.RUnlock()

	if bs.handle == nil {
		return ErrShuttingDown
	}

	result := C.blackstream_flush(bs.handle)
	return errorFromCode(result)
}

// Shutdown gracefully shuts down the event bus and frees resources.
//
// After calling Shutdown, the Blackstream instance is no longer usable.
func (bs *Blackstream) Shutdown() error {
	bs.mu.Lock()
	defer bs.mu.Unlock()

	if bs.handle == nil {
		return nil
	}

	result := C.blackstream_shutdown(bs.handle)
	bs.handle = nil
	runtime.SetFinalizer(bs, nil)
	return errorFromCode(result)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.blackstream_version())
}

// GenerateBltpKeypair generates a new X25519 keypair for BLTP.
//
// Returns a keypair with hex-encoded public and secret keys.
// Use this to generate keys for a responder, then share the public key
// with the initiator.
func GenerateBltpKeypair() (*BltpKeypair, error) {
	result := C.bltp_generate_keypair()
	if result == nil {
		return nil, errors.New("failed to generate keypair (BLTP feature may not be enabled)")
	}
	defer C.blackstream_free_string(result)

	jsonStr := C.GoString(result)
	var keypair BltpKeypair
	if err := json.Unmarshal([]byte(jsonStr), &keypair); err != nil {
		return nil, fmt.Errorf("failed to parse keypair: %w", err)
	}
	return &keypair, nil
}
