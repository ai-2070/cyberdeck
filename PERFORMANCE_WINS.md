# Performance Wins

Consolidated findings across multiple exploration passes of `net/crates/net` (Rust core, Rust SDK, TS/Python/Go/Node bindings, adapters). Entries marked **[2×]** or **[3×]** were independently surfaced by multiple sessions — higher confidence.

Format: `file:line` — what's slow → fix → why it matters.

---

## Tier 1 — High impact, ship first

### Rust core — serialization & allocations on ingest/ACL hot paths

- **`src/shard/mod.rs:365`** — `select_shard()` calls `serde_json::to_vec(event)` only to feed bytes into xxhash, then the event is serialized *again* downstream for storage.
  - **Fix:** route callers that already hold (or can cheaply produce) serialized bytes through `select_shard_by_hash()` at line 382; compute the hash once during serialization.

- **`src/adapter/net/redex/file.rs:651`** — `tail()` uses `mpsc::unbounded_channel()`. Fast publisher + slow subscriber → unbounded memory growth.
  - **Fix:** bounded `mpsc::channel(N)` with drop-oldest or `try_send` backpressure.

- **`src/adapter/net/redex/file.rs:275, 308`** — `append`/`append_inline` build a fresh `Bytes::copy_from_slice(payload)` on every call even when the caller already owns `Bytes`.
  - **Fix:** accept `Bytes` (or `Cow<[u8]>`) at the API boundary; reuse, don't re-copy.

- **`src/adapter/net/redex/file.rs:379`** — `payload.clone()` per event inside the batch-append loop before the disk write.
  - **Fix:** build `events` with `Bytes` refs; swap payload out of the input vec on the happy path.

- **`src/adapter/net/channel/guard.rs:324, 336, 347`** — `exact.insert/remove/contains_key(&(origin_hash, name.clone()))` on the per-packet control-plane ACL path.
  - **Fix:** key the DashMap by `(u64, u64)` (hash pair) or wrap `ChannelName` in `Arc` so lookups are refcount bumps.

- **`src/adapter/net/behavior/api.rs:456-463`** — `serde_json::to_string(v)` per element for `unique_items: true` validation.
  - **Fix:** implement `Eq`+`Hash` on `serde_json::Value` (or memoize hash), avoid per-element allocation.

- **`src/adapter/net/behavior/api.rs:925-945`** — `matches_path()` allocates two `Vec<&str>` from `split('/')` plus `to_string()` per parameter on every routing call.
  - **Fix:** pre-compile path pattern to regex, or cache parameter positions at construction.

### Node bindings

- **`bindings/node/src/cortex.rs:280-284`** — `append_batch` does `Bytes::copy_from_slice(b.as_ref())` per element, turning a zero-copy path into O(total_bytes) per batch.
  - **Fix:** `Bytes::from_owner(Buffer)` (napi Buffers are refcounted); at minimum pre-reserve the Vec.

### Rust SDK

- **`sdk/src/stream.rs:117, 192`** — `EventStream::poll_next` clones `StoredEvent` (raw payload bytes) twice per yielded event.
  - **Fix:** `VecDeque::pop_front` / `swap_remove` instead of index-clone, or wrap payloads in `Arc<Bytes>`.

### TypeScript SDK

- **`sdk-ts/src/stream.ts:50-75`** — `DEFAULT_POLL_INTERVAL = 1` hot-spins on active streams; backoff only kicks in when idle.
  - **Fix:** raise base interval to 50–100 ms, or switch to event-driven subscription.

- **`sdk-ts/src/channel.ts:49-68`** — `publish()` does `JSON.stringify({...event, _channel})` per event, re-serializing the whole tree. Dominant cost at 10M tokens/sec.
  - **Fix:** pre-compute channel prefix at construction; append payload bytes.

### Python SDK [2×]

- **`sdk-py/src/net_sdk/channel.py:59, 67, 72, 97`** — `json.dumps({"path":"_channel","value":self._name})` and payload re-dumps on every publish/subscribe.
  - **Fix:** cache filter string as `self._filter` in `__init__`; batch-encode payloads.

- **`sdk-py/src/net_sdk/stream.py:78`** — blocking `time.sleep` inside event-polling loop freezes the caller (max backoff 100 ms).
  - **Fix:** expose `async __aiter__` using `asyncio.sleep`.

### Go bindings

- **`bindings/go/net/net.go:235-248`** (`IngestRawBatch`) — `C.CString` allocation + FFI copy per message in batch.
  - **Fix:** use the byte-slice (ptr + `size_t`) pattern already used in `compute.go`.

### Redis adapter [2×]

- **`src/adapter/redis.rs:275, 287`** — `String::from_utf8_lossy(bytes).to_string()` double-allocates per field on every XRANGE entry, just to compare against constant `"d"`/`"id"`.
  - **Fix:** byte-compare directly (`bytes == b"d"`); no allocation.

---

## Tier 2 — Medium impact

### JetStream adapter

- **`src/adapter/jetstream.rs:96`** [2×] — `deserialize_event` does `serde_json::from_slice` then `serde_json::to_vec` just to pluck the `"r"` field; two O(n) passes per delivered event, defeating zero-copy.
  - **Fix:** `serde_json::value::RawValue` for `r`; keep raw `Bytes`, deserialize lazily.

- **`src/adapter/jetstream.rs:309-328`** — N+1 `direct_get(current_seq)` loop. One RTT per event; catastrophic with stream gaps.
  - **Fix:** batch/prefetch a window; fall back to single `direct_get` only after a miss.

- **`src/adapter/jetstream.rs:302`** — `stream.info().await` per poll per shard, just to learn the tail sequence.
  - **Fix:** cache with short TTL, or let `direct_get` signal EOS.

- **`src/adapter/jetstream.rs:229`** — `format!("{}:{}:{}", …)` per event allocates a fresh `String` for the dedup ID.
  - **Fix:** stack-buffer via `itoa`/`smallvec`, or pre-allocate per batch.

- **`src/adapter/jetstream.rs:233`** — `subject.clone()` inside batch-publish loop; subject is constant for the batch.
  - **Fix:** hoist clone above the loop, or pass by reference.

- **`src/adapter/jetstream.rs:116, 160`** — double-clone of `Stream` on cache insert path (read-hit clone + unused insert clone).
  - **Fix:** move the original on insert; clone only on read-hit.

### Redis adapter

- **`src/adapter/redis.rs:298, 320`** — `id.clone()` per event in poll loop + final cursor clone on result.
  - **Fix:** move `id` where possible; `Arc<str>` for cursor reuse.

### Capability registry

- **`src/adapter/net/behavior/capability.rs:1342-1375`** — `by_tag.entry(tag.clone())` etc. clones tag/model strings twice per index insert on every node announcement.
  - **Fix:** use borrow-based entry API; avoid clone on lookup.

- **`src/adapter/net/behavior/capability.rs:1507-1516`** — second `DashMap::get()` per candidate after index intersection (filter-match re-lookup).
  - **Fix:** fold validation into the intersection algorithm; store enough data in the index to short-circuit.

- **`src/adapter/net/behavior/capability.rs:1520-1532`** — `find_best()` may do O(M) scoring per candidate where M = capability count.
  - **Fix:** cache scores in companion index, or short-circuit on filter match.

### Roster & watch fanout

- **`src/adapter/net/channel/roster.rs:100, 123`** — `set.iter().map(|c| c.clone()).collect()` for `remove_peer` / `channels_for`.
  - **Fix:** wrap `ChannelId` in `Arc` so snapshots are refcount bumps.

- **`src/adapter/net/cortex/tasks/watch.rs:152, 169`** and **`…/memories/watch.rs:175, 192`** — `initial.clone()` / `current.clone()` of full `Vec<Task>`/`Vec<Memory>` per watch emission.
  - **Fix:** wrap state in `Arc<Vec<_>>`; each yield becomes a refcount bump.

### Memories filter builder

- **`src/adapter/net/cortex/memories/filter.rs:47, 50, 53, 56, 59`** — `s.clone()` / `tags.clone()` per predicate on every query build.
  - **Fix:** take `&str` / `&[String]` (or intern) in `MemoriesQuery::where_*`; filter values are short-lived.

### Consumer

- **`src/consumer/merge.rs:304, 310`** — `.parse()` inside `retain` re-parses every over-fetched event that will be discarded.
  - **Fix:** parse once, filter on the parsed value.

### Rust SDK

- **`sdk/src/groups/standby.rs:96`** — `event.clone()` in observer closure clones the whole `CausalEvent` (incl. payload) per delivery to a standby observer.
  - **Fix:** pass `&CausalEvent` or wrap in `Arc`.

### Go bindings

- **`bindings/go/net/net.go:299-326`** (`Poll`) — builds a `map[string]interface{}`, marshals it, and `C.CString`s it per poll.
  - **Fix:** reusable byte-slice request buffer; cached encoder.

- **`bindings/go/net/cortex.go:361`** (+568, 613, 642) — per-event `json.Unmarshal` in streaming tail loop; no decoder reuse.
  - **Fix:** `json.Decoder` over the tail stream, or bulk-tail API.

---

## Tier 3 — Polish / marginal wins

### Rust core

- **`src/adapter/net/behavior/capability.rs:1564-1581`** — `gc()` collects all expired IDs into a `Vec` before removal.
  - **Fix:** use DashMap `retain()` / `remove_if()`.

- **`src/adapter/net/behavior/capability.rs:1549-1551`** — `all_nodes()` allocates full `Vec<u64>`.
  - **Fix:** return iterator.

- **`src/adapter/net/behavior/api.rs:1679-1686`** — endpoint prefix via `split('/').take(2).collect::<Vec<_>>().join("/")` allocates Vec on every registration.
  - **Fix:** iterator chain `collect::<String>()` or store split indices.

- **`src/adapter/jetstream.rs:113-161`** — benign TOCTOU on stream cache.
  - **Fix:** use `entry()` for cleanliness.

### Rust SDK

- **`sdk/src/mesh.rs:521, 526, 546`** — redundant `channel.clone()` on subscribe/unsubscribe.
- **`sdk/src/compute.rs:413, 415`** and **`sdk/src/groups/replica.rs:97, 105, 116, 150`** — `kind.to_string()` repeated 2–4× per call; convert once.
- **`sdk/src/stream.rs:139`** — filter cloned per `ConsumeRequest` even when absent.

### TypeScript SDK

- **`sdk-ts/src/channel.ts:78-82`** — subscribe filter `JSON.stringify` per call; filter is static.
  - **Fix:** memoize in constructor.

- **`sdk-ts/src/node.ts:90-93`** — `emitBatch` `events.map(JSON.stringify)` without capacity hint.
  - **Fix:** pre-allocate / streaming encode.

### Python SDK

- **`sdk-py/src/net_sdk/channel.py:15-24`** — `_to_dict` always copies even when input is already a dict.
  - **Fix:** early-return on dict.
- **`sdk-py/src/net_sdk/channel.py:64-67`** — per-event `json.dumps` in `publish_batch`.
  - **Fix:** batch-serialize.

### Go bindings

- **`bindings/go/net/groups.go:237-251`** — unmarshal into temp struct then copy to result slice.
  - **Fix:** unmarshal directly into result slice.

### Build

- **`net/crates/net/Cargo.toml`** — `[profile.bench]` missing `codegen-units = 1`; benches slightly under-optimized vs release.

---

## Coverage gaps

These directories were **not** examined in the passes above and are worth a second sweep: `apps/browser/`, `apps/cli/`, `apps/computer/`, `apps/cron/`, `apps/documents/`, `apps/files/`, `apps/webhooks/`, `chat/`, `stateful-ai/`, `ai-swarm/`, `ai-infernece/`, `cortex/`, `redex/`.

## Suggested attack order

1. **Ingest/ACL hot path Tier 1 items in `net/crates/net`** — these sit on the per-packet path and compound across every publish.
2. **JetStream `deserialize_event` + N+1 `direct_get`** — single largest latency improvement for JetStream-backed deployments.
3. **TS `stream.ts` 1 ms poll + Python `time.sleep`** — client-side CPU/latency wins visible to users.
4. **Node `append_batch` `copy_from_slice`** — throughput cliff for any Node-heavy ingest.
5. Everything else, grouped into a cleanup PR per subsystem.
