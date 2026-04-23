# Payloads — Size, Encoding, Large Blobs

Short reference for what to put on the bus and what to keep off it.

---

## Wire format

JSON bytes. Always. There is no schema registry, no Protobuf, no Avro, no MessagePack on the wire. The producer serializes to JSON; the consumer parses from JSON.

This is true across all five SDKs, including C (which expects valid JSON in `net_ingest_raw`).

## Practical size guidance

| Payload size | Verdict | Notes |
|---|---|---|
| < 1 KB | Ideal | Single UDP packet on mesh transport, fits any ring buffer slot, lowest latency. |
| 1 KB – 64 KB | Fine | Default per-stream credit window is 64 KB. Above that, mesh transport will backpressure on a single event. |
| 64 KB – 1 MB | Works but slow | Many UDP fragments on mesh, stresses the ring buffer, increases drop probability under load. Tune `window_bytes` upward if you must. |
| > 1 MB | Don't | The bus is a coordination layer, not a file transfer. Use the patterns below. |

These are guidelines. The SDK does not enforce a hard maximum — but the further right you go, the more you fight the system.

## Patterns for large data

### Reference, don't embed

If you have a 50 MB payload (a video frame, a model checkpoint, a CSV), put it where it belongs (S3, MinIO, a local file, a CDN URL) and emit a small event with the reference:

```json
{"frame_id": "abc123", "url": "s3://bucket/frames/abc123.bin", "sha256": "..."}
```

The bus carries the coordination signal at full speed; the bulk data moves through whatever bulk-data path your infrastructure already has.

### Chunk + reassemble

If the data must traverse the mesh (you have no shared storage layer), chunk on the producer and reassemble on the consumer. Each chunk is its own event with `(blob_id, chunk_index, total_chunks, payload)`. Consumer accumulates by `blob_id` and emits the reassembled blob to its own channel when complete. Add a timeout so partial blobs don't accumulate forever.

This is your code, not the SDK's. Keep it simple — Net does not provide a built-in chunking layer.

### Stream-of-events instead of one big event

If the "payload" is naturally a sequence (sensor readings, log lines, token stream), emit each unit as its own event. Subscribers process incrementally. This is the design Net is optimized for.

```python
# Bad: 10 MB JSON array of readings
node.emit({"readings": all_ten_thousand})

# Good: 10,000 small events
for reading in all_ten_thousand:
    node.emit(reading)
```

The "good" version uses more total bytes (more JSON envelope per event) but is far healthier for backpressure, latency, and consumer parallelism.

## Encoding considerations

- **UTF-8.** JSON requires it. Don't put binary in a JSON string field unless you base64-encode it (and at that point, see "Reference, don't embed").
- **Numbers.** JSON numbers are floats by default. For `u64` IDs and timestamps, the SDKs already handle the precision boundary (BigInt in TS, native int in Python, u64 in Rust/Go/C) — but if you're constructing JSON manually, pass IDs as strings to avoid silent precision loss.
- **Reserved field: `_channel`.** TS and Python channel APIs inject this on publish and filter on it during subscribe. Don't put a `_channel` field in your own payload.
- **No null-padding, no fixed-width encoding.** This is JSON, not binary protocol — payloads are variable-length.

## Throughput vs latency trade-off

Smaller events = lower latency, higher per-event overhead.
Bigger events = higher throughput, higher per-event tail latency.

For coordination signals (intents, state changes, decisions), favor small. For bulk telemetry where you want high MB/s, batch a few hundred readings into one event — but stop when one event approaches the ring-buffer slot size.

`emit_batch` / `emitBatch` / `IngestRawBatch` is the right tool for batching: it amortizes the per-call overhead while keeping each event individually addressable on the wire.
