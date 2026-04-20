# Net Go Bindings

High-performance Go bindings for the Net event bus.

## Prerequisites

1. **Rust toolchain** - Install from https://rustup.rs
2. **Go 1.21+** - Install from https://go.dev

## Building

First, build the Net shared library:

```bash
# From the repository root
cargo build --release

# To include CortEX + RedEX support (required for the cortex.go surface below),
# build with the extended feature set:
cargo build --release --features "netdb redex-disk"

# The library will be at:
# - Linux: target/release/libnet.so
# - macOS: target/release/libnet.dylib
# - Windows: target/release/net.dll
```

## Installation

```bash
go get github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net
```

## Usage

```go
package main

import (
    "fmt"
    "log"

    "github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net"
)

func main() {
    // Create event bus with default configuration
    bus, err := net.New(nil)
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
config := &net.Config{
    NumShards:          8,        // Number of parallel shards
    RingBufferCapacity: 1048576,  // Events per shard (must be power of 2)
    BackpressureMode:   "DropOldest", // or "DropNewest", "FailProducer"
}

bus, err := net.New(config)
```

## Net Encrypted UDP Transport

Net provides encrypted point-to-point UDP transport for high-performance scenarios:

```go
import (
    "crypto/rand"
    "encoding/hex"
    "github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net"
)

// Generate keypair for responder
keypair, err := net.GenerateNetKeypair()
if err != nil {
    log.Fatal(err)
}

// Generate pre-shared key
psk := make([]byte, 32)
rand.Read(psk)
pskHex := hex.EncodeToString(psk)

// Responder side
responder, err := net.New(&net.Config{
    NumShards: 2,
    Net: &net.NetConfig{
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
initiator, err := net.New(&net.Config{
    NumShards: 2,
    Net: &net.NetConfig{
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

- `Net` - Event bus handle
- `Config` - Configuration options
- `NetConfig` - Net encrypted UDP adapter configuration
- `NetKeypair` - Generated keypair for Net
- `PollResponse` - Result of a poll operation
- `Stats` - Event bus statistics

### Functions

- `New(config *Config) (*Net, error)` - Create a new event bus
- `Version() string` - Get the library version
- `GenerateNetKeypair() (*NetKeypair, error)` - Generate a new Net keypair

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

## Mesh transport + channels

Encrypted-UDP mesh handshake, per-peer streams with v2 backpressure,
and named pub/sub channels. Requires building the Rust cdylib with
`--features "net"` (already on when you use the combined
`--features "netdb redex-disk net"` build described above).

```go
package main

import (
    "log"

    "github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net"
)

func main() {
    psk := "42" + strings.Repeat("42", 31)  // 64 hex chars

    // Publisher.
    pub, err := net.NewMeshNode(net.MeshConfig{
        BindAddr: "127.0.0.1:9001",
        PskHex:   psk,
    })
    if err != nil { log.Fatal(err) }
    defer pub.Shutdown()

    // Subscriber.
    sub, err := net.NewMeshNode(net.MeshConfig{
        BindAddr: "127.0.0.1:9000",
        PskHex:   psk,
    })
    if err != nil { log.Fatal(err) }
    defer sub.Shutdown()

    // Handshake: subscriber connects to publisher.
    pubKey, _ := pub.PublicKey()
    go pub.Accept(sub.NodeID())
    if err := sub.Connect("127.0.0.1:9001", pubKey, pub.NodeID()); err != nil {
        log.Fatal(err)
    }
    pub.Start(); sub.Start()

    // Register + subscribe + publish.
    pub.RegisterChannel(net.ChannelConfig{
        Name:       "sensors/temp",
        Visibility: "global",
        Reliable:   true,
    })
    if err := sub.SubscribeChannel(pub.NodeID(), "sensors/temp"); err != nil {
        log.Fatal(err)
    }
    report, err := pub.Publish("sensors/temp", []byte("22.5"), net.PublishConfig{
        Reliability: "reliable",
        OnFailure:   "best_effort",
    })
    if err != nil { log.Fatal(err) }
    log.Printf("%d/%d delivered", report.Delivered, report.Attempted)

    // Subscriber drains the payload via the event bus.
    for shard := uint16(0); shard < 4; shard++ {
        events, _ := sub.RecvShard(shard, 16)
        for _, e := range events {
            log.Printf("recv: %s", e.Payload)
        }
    }
}
```

### Per-peer streams with backpressure

```go
stream, err := node.OpenStream(peerID, 0x1337, net.StreamConfig{
    Reliability: "reliable",
    WindowBytes: 64 * 1024,
})
if err != nil { log.Fatal(err) }
defer stream.Close()

// Three send policies:
// 1. Drop on pressure.
if err := stream.Send(payloads); errors.Is(err, net.ErrBackpressure) {
    metrics.Inc("drops")
}
// 2. Retry with backoff.
stream.SendWithRetry(payloads, 8)
// 3. Block until clear.
stream.SendBlocking(payloads)

// Live stats.
stats, _ := node.StreamStats(peerID, 0x1337)
log.Printf("tx_credit=%d backpressure_events=%d",
    stats.TxCreditRemaining, stats.BackpressureEvents)
```

### Typed errors

- `ErrMeshInit` — bad bind address / PSK / crypto init.
- `ErrMeshHandshake` — `Connect` / `Accept` failed.
- `ErrBackpressure` — stream's in-flight window is full; nothing sent.
- `ErrNotConnected` — peer session is gone.
- `ErrMeshTransport` — other I/O error.
- `ErrChannel` — channel invalid name / visibility / unknown / rate limit / transport.
- `ErrChannelAuth` — publisher rejected the subscribe as unauthorized.

All are sentinels; use `errors.Is`.

### Mesh API reference

| Function / method | Description |
|---|---|
| `NewMeshNode(cfg MeshConfig)` | Open a mesh node |
| `(*MeshNode).PublicKey() / NodeID()` | Identity |
| `(*MeshNode).Connect / Accept / Start / Shutdown` | Handshake + lifecycle |
| `(*MeshNode).OpenStream(peerID, streamID, cfg)` | Open a per-peer stream |
| `(*MeshNode).StreamStats(peerID, streamID)` | Per-stream snapshot |
| `(*MeshNode).RecvShard(shard, limit)` | Drain a shard inbox |
| `(*MeshNode).RegisterChannel(cfg)` | Install a channel config |
| `(*MeshNode).SubscribeChannel(pubID, name)` | Join a channel |
| `(*MeshNode).UnsubscribeChannel(pubID, name)` | Leave a channel |
| `(*MeshNode).Publish(name, payload, cfg)` | Fan one payload to subscribers |
| `(*MeshStream).Send / SendWithRetry / SendBlocking` | Three send policies |
| `(*MeshStream).Close()` | Release stream handle |

## CortEX & NetDb (event-sourced state)

Typed, event-sourced state on top of RedEX — tasks and memories with
filterable queries and Go-channel-based watches. The `SnapshotAndWatch`
primitive preserves the v2 race fix: you get both the initial filter
result and a live delta channel atomically.

Build the cdylib with `--features "netdb redex-disk"` to expose the
cortex surface (see the "Building" section above).

```go
package main

import (
    "context"
    "fmt"
    "log"
    "time"

    "github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net"
)

func main() {
    redex := net.NewRedex("") // heap-only; pass a path for persistence
    defer redex.Free()

    tasks, err := net.OpenTasks(redex, 0xABCDEF01, false)
    if err != nil {
        log.Fatal(err)
    }
    defer tasks.Close()

    // CRUD.
    seq, err := tasks.Create(1, "write docs", 100)
    if err != nil {
        log.Fatal(err)
    }
    if err := tasks.WaitForSeq(seq, 2*time.Second); err != nil {
        log.Fatal(err)
    }

    // Snapshot + watch, atomically.
    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    snapshot, updates, errs, err := tasks.SnapshotAndWatch(ctx, &net.TasksFilter{
        Status: "pending",
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("initial: %d pending\n", len(snapshot))

    go func() {
        _, _ = tasks.Complete(1, 200)
    }()

    select {
    case batch := <-updates:
        fmt.Printf("update: %d pending\n", len(batch))
    case err := <-errs:
        log.Fatal(err)
    case <-time.After(time.Second):
        log.Fatal("timeout")
    }
}
```

### Raw RedEX file

For domain-agnostic persistent logs (no CortEX fold), use the `Redex`
manager directly:

```go
redex := net.NewRedex("/var/lib/net/events")
defer redex.Free()

file, err := redex.OpenFile("analytics/clicks", &net.RedexFileConfig{
    Persistent:      true,
    FsyncIntervalMs: 100,
})
if err != nil {
    log.Fatal(err)
}
defer file.Close()

seq, _ := file.Append([]byte(`{"url": "/home"}`))
fmt.Println("wrote seq", seq)

ctx, cancel := context.WithCancel(context.Background())
defer cancel()
events, errs, _ := file.Tail(ctx, 0)
for {
    select {
    case ev, ok := <-events:
        if !ok {
            return
        }
        fmt.Println(ev.Seq, string(ev.Payload))
    case err := <-errs:
        log.Fatal(err)
    }
}
```

### CortEX API reference

- `NewRedex(persistentDir string) *Redex`
- `(*Redex).OpenFile(name string, config *RedexFileConfig) (*RedexFile, error)`
- `OpenTasks(redex *Redex, originHash uint32, persistent bool) (*TasksAdapter, error)`
- `OpenMemories(redex *Redex, originHash uint32, persistent bool) (*MemoriesAdapter, error)`
- `(*TasksAdapter).Create / Rename / Complete / Delete / WaitForSeq / List / SnapshotAndWatch`
- `(*MemoriesAdapter).Store / Retag / Pin / Unpin / Delete / WaitForSeq / List / SnapshotAndWatch`
- `(*RedexFile).Append / ReadRange / Tail / Len / Sync / Close`

Errors surfaced as typed sentinels:
`ErrCortexClosed`, `ErrCortexFold`, `ErrNetDb`, `ErrRedex`,
`ErrStreamTimeout`, `ErrStreamEnded`.

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

All methods on `Net` are thread-safe and can be called from multiple goroutines concurrently.
