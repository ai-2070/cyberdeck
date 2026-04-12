# Blackstream Go Bindings

High-performance Go bindings for the Blackstream event bus.

## Prerequisites

1. **Rust toolchain** - Install from https://rustup.rs
2. **Go 1.21+** - Install from https://go.dev

## Building

First, build the Blackstream shared library:

```bash
# From the repository root
cargo build --release

# The library will be at:
# - Linux: target/release/libblackstream.so
# - macOS: target/release/libblackstream.dylib
# - Windows: target/release/blackstream.dll
```

## Installation

```bash
go get github.com/ai-2070/blackstream/bindings/go/blackstream
```

## Usage

```go
package main

import (
    "fmt"
    "log"

    "github.com/ai-2070/blackstream/bindings/go/blackstream"
)

func main() {
    // Create event bus with default configuration
    bus, err := blackstream.New(nil)
    if err != nil {
        log.Fatal(err)
    }
    defer bus.Shutdown()

    // Ingest events (fast path with raw JSON)
    err = bus.IngestRaw(`{"token": "hello", "index": 0}`)
    if err != nil {
        log.Fatal(err)
    }

    // Ingest using Go structs
    event := map[string]interface{}{
        "type":  "token",
        "value": "world",
    }
    err = bus.Ingest(event)
    if err != nil {
        log.Fatal(err)
    }

    // Batch ingest for higher throughput
    events := []string{
        `{"type": "token", "value": "a"}`,
        `{"type": "token", "value": "b"}`,
        `{"type": "token", "value": "c"}`,
    }
    ingested := bus.IngestRawBatch(events)
    fmt.Printf("Ingested %d events\n", ingested)

    // Poll events
    response, err := bus.Poll(100, "")
    if err != nil {
        log.Fatal(err)
    }

    for _, raw := range response.Events {
        fmt.Printf("Event: %s\n", raw)
    }

    // Pagination
    if response.HasMore {
        nextPage, err := bus.Poll(100, response.NextID)
        if err != nil {
            log.Fatal(err)
        }
        fmt.Printf("Next page has %d events\n", nextPage.Count)
    }

    // Get statistics
    stats, err := bus.Stats()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Total ingested: %d\n", stats.EventsIngested)
}
```

## Configuration

```go
config := &blackstream.Config{
    NumShards:          8,        // Number of parallel shards
    RingBufferCapacity: 1048576,  // Events per shard (must be power of 2)
    BackpressureMode:   "DropOldest", // or "DropNewest", "FailProducer"
}

bus, err := blackstream.New(config)
```

## BLTP Encrypted UDP Transport

BLTP provides encrypted point-to-point UDP transport for high-performance scenarios:

```go
import (
    "crypto/rand"
    "encoding/hex"
    "github.com/ai2070/blackstream/bindings/go/blackstream"
)

// Generate keypair for responder
keypair, err := blackstream.GenerateBltpKeypair()
if err != nil {
    log.Fatal(err)
}

// Generate pre-shared key
psk := make([]byte, 32)
rand.Read(psk)
pskHex := hex.EncodeToString(psk)

// Responder side
responder, err := blackstream.New(&blackstream.Config{
    NumShards: 2,
    Bltp: &blackstream.BltpConfig{
        BindAddr:    "127.0.0.1:9001",
        PeerAddr:    "127.0.0.1:9000",
        PSK:         pskHex,
        Role:        "responder",
        SecretKey:   keypair.SecretKey,
        PublicKey:   keypair.PublicKey,
        Reliability: "light", // "none", "light", or "full"
    },
})

// Initiator side (knows responder's public key)
initiator, err := blackstream.New(&blackstream.Config{
    NumShards: 2,
    Bltp: &blackstream.BltpConfig{
        BindAddr:      "127.0.0.1:9000",
        PeerAddr:      "127.0.0.1:9001",
        PSK:           pskHex,
        Role:          "initiator",
        PeerPublicKey: keypair.PublicKey,
    },
})

// Use as normal
initiator.IngestRaw(`{"event": "data"}`)
```

## API Reference

### Types

- `Blackstream` - Event bus handle
- `Config` - Configuration options
- `BltpConfig` - BLTP encrypted UDP adapter configuration
- `BltpKeypair` - Generated keypair for BLTP
- `PollResponse` - Result of a poll operation
- `Stats` - Event bus statistics

### Functions

- `New(config *Config) (*Blackstream, error)` - Create a new event bus
- `Version() string` - Get the library version
- `GenerateBltpKeypair() (*BltpKeypair, error)` - Generate a new BLTP keypair

### Methods

- `IngestRaw(json string) error` - Ingest raw JSON (fastest)
- `IngestRawBatch(jsons []string) int` - Batch ingest raw JSON
- `Ingest(event interface{}) error` - Ingest Go value as JSON
- `IngestBatch(events []interface{}) int` - Batch ingest Go values
- `Poll(limit int, cursor string) (*PollResponse, error)` - Poll events
- `Stats() (*Stats, error)` - Get statistics
- `NumShards() int` - Get shard count
- `Flush() error` - Flush pending batches
- `Shutdown() error` - Shutdown and free resources

## Running the Example

```bash
cd bindings/go/example

# Set library path (Linux)
export LD_LIBRARY_PATH=../../../target/release:$LD_LIBRARY_PATH

# Set library path (macOS)
export DYLD_LIBRARY_PATH=../../../target/release:$DYLD_LIBRARY_PATH

go run main.go
```

## Performance Tips

1. **Use `IngestRaw`** - Avoid JSON marshaling overhead when possible
2. **Use `IngestRawBatch`** - Batch multiple events for better throughput
3. **Tune `NumShards`** - Match to your CPU core count for parallelism
4. **Increase `RingBufferCapacity`** - Larger buffers handle bursts better

## Thread Safety

All methods on `Blackstream` are thread-safe and can be called from multiple goroutines concurrently.
