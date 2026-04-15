# Transport Layer

The foundational layer of the NLTP mesh. Encrypted UDP with zero-allocation hot paths, multi-hop forwarding, adaptive batching, fair scheduling, failure detection, and swarm discovery.

## Wire Format

Every NLTP packet starts with a 64-byte header aligned to a single CPU cache line. Forwarding nodes read one cache line, make a routing decision, and forward without decrypting the payload.

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         MAGIC (0x424C)        |     VER       |     FLAGS     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   PRIORITY    |    HOP_TTL    |   HOP_COUNT   |  FRAG_FLAGS   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       SUBPROTOCOL_ID          |        CHANNEL_HASH           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         NONCE (12 bytes)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       SESSION_ID (8 bytes)                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       STREAM_ID (8 bytes)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       SEQUENCE (8 bytes)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|      SUBNET_ID (4 bytes)      |     ORIGIN_HASH (4 bytes)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       FRAGMENT_ID             |        FRAGMENT_OFFSET        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       PAYLOAD_LEN             |        EVENT_COUNT            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Constants:**
- Magic: `0x424C` (ASCII "BL")
- Version: 1
- Max packet: 8,192 bytes
- Max payload: 8,096 bytes (packet - header - Poly1305 tag)
- Nonce: 12 bytes (counter-based)
- Tag: 16 bytes (Poly1305)

**Packet flags:** `RELIABLE`, `NACK`, `PRIORITY`, `FIN`, `HANDSHAKE`, `HEARTBEAT`

## Encryption

**Handshake:** Noise NKpsk0 pattern via the `snow` crate. The initiator is anonymous, the responder's static key is known in advance. A pre-shared key adds symmetric authentication.

**Payload encryption:** ChaCha20-Poly1305 AEAD with counter-based nonces. Each session derives separate TX/RX `SessionKeys` from the Noise handshake. The header is never encrypted -- only the payload.

```rust
pub struct SessionKeys {
    pub tx_key: [u8; 32],
    pub rx_key: [u8; 32],
    pub session_id: u64,
}
```

`PacketCipher` wraps the AEAD primitive with a monotonic counter for nonce generation, eliminating nonce-reuse risk without randomness.

## Packet Pools

Zero-allocation on the hot path. `PacketPool` pre-allocates reusable `BytesMut` buffers. `ThreadLocalPool` eliminates contention entirely -- each thread has its own pool.

```rust
PacketPool::new(capacity: usize)        // Shared pool (Arc<ArrayQueue>)
ThreadLocalPool::new(capacity: usize)   // Per-thread, zero contention
```

`PacketBuilder` constructs packets from pre-allocated buffers, batching multiple `EventFrame`s into a single packet. Events are length-prefixed (4-byte LE length + payload).

**Benchmark:** Thread-local pools achieve **23x contention advantage** over shared pools at 32 threads.

## Sessions

`NltpSession` holds post-handshake state: TX/RX ciphers, per-stream sequence numbers, packet pool, and activity timestamps.

```rust
pub struct NltpSession {
    session_id: u64,
    tx_cipher: Mutex<PacketCipher>,
    rx_cipher: Mutex<PacketCipher>,
    streams: DashMap<u64, StreamState>,
    pool: SharedPacketPool,
    origin_hash: u32,
    // ...
}
```

`SessionManager` validates session health and handles timeouts. Sessions are long-lived -- new sessions only form on handshake.

## Stream Routing & Fair Scheduling

`FairScheduler` provides round-robin fairness across streams. Each stream gets a configurable quantum of packets per round. Priority streams can bypass the fairness queue.

```rust
pub struct RouterConfig {
    pub max_queue_depth: usize,   // Per-stream queue limit
    pub fair_quantum: usize,      // Packets per stream per round
}
```

Stream IDs are derived via xxh3 hashing of message keys, providing deterministic stream assignment.

## Multi-Hop Forwarding

`NltpProxy` forwards packets without decrypting payloads. Reads the 64-byte header, decrements TTL, increments hop count, and forwards.

```rust
pub struct RoutingHeader {  // 16 bytes
    pub dest_id: u64,
    pub src_id: u64,
    // TTL, hop_count, flags packed in remaining bytes
}
```

`MultiHopPacketBuilder` constructs routed packets with layered routing headers. Per-hop latency tracking is optional.

**Benchmark:** 30.4 ns per hop (64B payload), 291 ns for a 5-hop chain.

## Reliability

Two modes implementing the `ReliabilityMode` trait:

| Mode | Overhead | Use case |
|------|----------|----------|
| `FireAndForget` | Zero | Sensor streams, telemetry |
| `ReliableStream` | Per-stream tracking | Commands, state updates |

`ReliableStream` uses selective NACKs: the receiver identifies missing sequence numbers and sends a `NackPayload` listing gaps. The sender retransmits only the missing packets. Timeout-driven retransmission handles lost NACKs.

## Adaptive Batching

`AdaptiveBatcher` dynamically sizes packet batches based on observed latency and queue depth.

- Target latency: 100 us (default)
- Batch range: 1 KB - 8 KB
- Burst detection: queue depth > 100 triggers larger batches
- EMA smoothing of batch latency for stable adaptation

**Benchmark:** +15-30% throughput for bursty workloads.

## Failure Detection

`FailureDetector` tracks node health via heartbeats.

```rust
pub enum NodeStatus {
    Healthy,
    Suspected,    // Missed heartbeats but not yet declared failed
    Failed,
    Unknown,
}
```

`RecoveryManager` handles route failover when nodes fail. `CircuitBreaker` prevents cascading failures by temporarily blocking traffic to failing nodes.

**Benchmark:** 32.4 ns per heartbeat processing, 362 ns for a full recovery cycle.

## Swarm Discovery

`Pingwave` is a lightweight neighbor discovery protocol. 24-byte packets flood the mesh with TTL-bounded propagation.

```rust
pub struct Pingwave {
    pub origin_id: u64,
    pub seq: u64,
    pub ttl: u8,
    pub hop_count: u8,
}
```

`CapabilityAd` announces what a node can do (GPU, tools, memory, model slots, tags). `LocalGraph` maintains a k-hop radius view of the mesh topology.

**Benchmark:** Graph construction for 5,000 nodes in 125 us.

## Socket Layer

`NltpSocket` wraps Tokio UDP with optimized buffer sizes:

| Buffer | Default | Testing |
|--------|---------|---------|
| RX | 64 MB | 256 KB |
| TX | 64 MB | 256 KB |

On Linux, `BatchedPacketReceiver` uses `recvmmsg` to read up to 64 packets per syscall.

## Source Files

| File | Purpose |
|------|---------|
| `protocol.rs` | Wire format, header, EventFrame, NackPayload |
| `crypto.rs` | Noise handshake, ChaCha20-Poly1305, SessionKeys |
| `transport.rs` | UDP socket, PacketReceiver/Sender, buffer config |
| `session.rs` | NltpSession, StreamState, SessionManager |
| `pool.rs` | PacketPool, PacketBuilder, ThreadLocalPool |
| `router.rs` | FairScheduler, stream routing, priority bypass |
| `route.rs` | RoutingTable, RoutingHeader, stream stats |
| `proxy.rs` | NltpProxy, zero-copy forwarding, hop tracking |
| `batch.rs` | AdaptiveBatcher, latency-aware sizing |
| `reliability.rs` | FireAndForget, ReliableStream, selective NACKs |
| `failure.rs` | FailureDetector, RecoveryManager, CircuitBreaker |
| `swarm.rs` | Pingwave, CapabilityAd, LocalGraph |
| `linux.rs` | recvmmsg batch reads (Linux-only) |
| `config.rs` | NltpAdapterConfig |
| `mod.rs` | NltpAdapter, routing utilities |
