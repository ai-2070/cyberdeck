# Net C SDK

One header, one shared library. This is the entire C SDK.

Unlocks every language that can call C: C++, Zig, Nim, Lua, Ruby, Java, C#, Dart, Swift, Kotlin, Haskell, Erlang, PHP.

## Files

- `net.h` — the header
- `libnet.so` (Linux) / `libnet.dylib` (macOS) / `net.dll` (Windows) — the library

## Build

```bash
# Build the shared library
cargo build --release --features ffi,net

# The library is at:
# Linux:  target/release/libnet.so
# macOS:  target/release/libnet.dylib
# Windows: target/release/net.dll
```

## Quick Start

```c
#include "net.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    // Create a node
    net_handle_t node = net_init("{\"num_shards\": 4}");
    if (!node) return 1;

    // Ingest
    const char* event = "{\"token\": \"hello\"}";
    net_receipt_t receipt;
    net_ingest_raw_ex(node, event, strlen(event), &receipt);
    printf("shard=%d ts=%llu\n", receipt.shard_id, (unsigned long long)receipt.timestamp);

    // Flush
    net_flush(node);

    // Poll (structured, no JSON parsing needed)
    net_poll_result_t result;
    net_poll_ex(node, 100, NULL, &result);
    for (size_t i = 0; i < result.count; i++) {
        printf("%.*s\n", (int)result.events[i].raw_len, result.events[i].raw);
    }
    net_free_poll_result(&result);

    // Stats (structured)
    net_stats_t stats;
    net_stats_ex(node, &stats);
    printf("ingested=%llu dropped=%llu\n",
        (unsigned long long)stats.events_ingested,
        (unsigned long long)stats.events_dropped);

    // Shutdown
    net_shutdown(node);
    return 0;
}
```

## Compile and Link

```bash
# GCC
gcc -o app app.c -L target/release -lnet -lpthread -ldl -lm

# Run
LD_LIBRARY_PATH=target/release ./app       # Linux
DYLD_LIBRARY_PATH=target/release ./app     # macOS
```

## API

### Lifecycle

| Function | Description |
|----------|-------------|
| `net_init(config_json)` | Create a node. NULL config for defaults. Returns handle. |
| `net_shutdown(handle)` | Shut down and free resources. |
| `net_version()` | Library version string (static, do not free). |
| `net_num_shards(handle)` | Number of active shards. |

### Ingestion

| Function | Description |
|----------|-------------|
| `net_ingest_raw(handle, json, len)` | Ingest raw JSON (fastest). |
| `net_ingest_raw_ex(handle, json, len, &receipt)` | Ingest with receipt (shard_id, timestamp). |
| `net_ingest(handle, json, len)` | Ingest with JSON validation. |
| `net_ingest_raw_batch(handle, jsons, lens, count)` | Batch ingest. Returns count. |
| `net_ingest_batch(handle, json_array)` | Ingest from JSON array string. |

### Consumption

| Function | Description |
|----------|-------------|
| `net_poll(handle, request_json, out_buffer, buffer_len)` | Poll (JSON interface). |
| `net_poll_ex(handle, limit, cursor, &result)` | Poll (structured, no JSON). Free with `net_free_poll_result`. |
| `net_free_poll_result(&result)` | Free a structured poll result. |

### Statistics

| Function | Description |
|----------|-------------|
| `net_stats(handle, out_buffer, buffer_len)` | Stats (JSON interface). |
| `net_stats_ex(handle, &stats)` | Stats (structured, no JSON). |

### Utilities

| Function | Description |
|----------|-------------|
| `net_flush(handle)` | Flush pending batches. |
| `net_generate_keypair()` | Generate mesh keypair. Free with `net_free_string`. |
| `net_free_string(s)` | Free a string from `net_generate_keypair`. |

## Types

```c
net_handle_t        // Opaque node handle (void*)
net_receipt_t       // { shard_id, timestamp }
net_event_t         // { id, id_len, raw, raw_len, insertion_ts, shard_id }
net_poll_result_t   // { events, count, next_id, has_more }
net_stats_t         // { events_ingested, events_dropped, batches_dispatched }
net_error_t         // NET_SUCCESS (0), NET_ERR_* (negative)
```

## Error Codes

| Code | Name | Value |
|------|------|-------|
| `NET_SUCCESS` | Success | 0 |
| `NET_ERR_NULL_POINTER` | Null pointer | -1 |
| `NET_ERR_INVALID_UTF8` | Invalid UTF-8 | -2 |
| `NET_ERR_INVALID_JSON` | Invalid JSON | -3 |
| `NET_ERR_INIT_FAILED` | Init failed | -4 |
| `NET_ERR_INGESTION_FAILED` | Ingestion failed | -5 |
| `NET_ERR_POLL_FAILED` | Poll failed | -6 |
| `NET_ERR_BUFFER_TOO_SMALL` | Buffer too small | -7 |
| `NET_ERR_SHUTTING_DOWN` | Shutting down | -8 |
| `NET_ERR_UNKNOWN` | Unknown error | -99 |

## Thread Safety

All functions are thread-safe. Handles can be shared across threads.

## Subscription Pattern

The C SDK does not manage threads. Use `net_poll_ex` in your own loop:

```c
char* cursor = NULL;
while (running) {
    net_poll_result_t result;
    int rc = net_poll_ex(node, 100, cursor, &result);
    if (rc < 0) break;

    for (size_t i = 0; i < result.count; i++) {
        process(&result.events[i]);
    }

    // Copy cursor before freeing the result.
    free(cursor);
    cursor = result.next_id ? strdup(result.next_id) : NULL;
    net_free_poll_result(&result);
}
free(cursor);
```

## Mesh transport

The header in this directory (`include/net.h`) is intentionally a
**narrow, public, event-bus-only** surface — every symbol declared
here is a stability commitment.

The mesh transport (encrypted peer sessions, channels, NAT
traversal, capability discovery) is implemented in the same
shared library but lives behind a **separate, broader header**:
[`bindings/go/net/net.h`](../bindings/go/net/net.h). That header is
written for the Go cgo bindings and is the de-facto reference for
C consumers who want the mesh API. Symbols are stable in practice
but not committed in the same way as `include/net.h`.

**Pick one header per translation unit.** Both files use the same
`#ifndef NET_SDK_H` include guard, so including both in the same
`.c` file silently drops one — the second include becomes a no-op
and any of its symbols you reference will fail to link or
compile. The split is by concern, not by additivity:

- **`include/net.h`** — bus-only consumers. Stable surface.
- **`bindings/go/net/net.h`** — mesh-using consumers. Includes the
  bus surface as well, plus the mesh, channels, capabilities, NAT,
  etc. (because the symbols are in the same dylib, the broader
  header is a *superset* — it just isn't the public/stable one).

If your project needs both surfaces, include the broader header —
it's a strict superset. If you need only the bus surface,
including the narrow header keeps you on the stable contract and
won't pull in mesh declarations you don't use.

A mesh node is its own handle (`net_meshnode_t*`), created via
`net_mesh_new` and torn down via `net_mesh_shutdown` — independent
of the bus handle (`net_handle_t`). A single process can hold both
simultaneously regardless of how the headers are included.

The Go bindings (`bindings/go/net/`) wrap this surface; their
README has runnable examples for every function family. The
section below is a function inventory — for usage prose, see
[`bindings/go/README.md`](../bindings/go/README.md).

### Quick start (mesh)

```c
#include "../bindings/go/net/net.h"   /* broader header */

net_meshnode_t* mesh = NULL;
const char* cfg =
    "{\"bind_addr\":\"127.0.0.1:9000\",\"psk_hex\":\"42424242...\"}";
if (net_mesh_new(cfg, &mesh) != 0) return 1;
net_mesh_start(mesh);

/* Announce hardware/software/tag fingerprints. */
net_mesh_announce_capabilities(mesh, "{\"tags\":[\"gpu\",\"prod\"]}");

/* Query the local capability index. Result is a JSON array of
 * node ids; free with net_free_string. */
char* result = NULL;
size_t result_len = 0;
net_mesh_find_nodes(mesh, "{\"require_tags\":[\"gpu\"]}",
                    &result, &result_len);
printf("matches: %.*s\n", (int)result_len, result);
net_free_string(result);

net_mesh_shutdown(mesh);
```

### Mesh function families

| Family | Functions | Purpose |
|--------|-----------|---------|
| Lifecycle | `net_mesh_new`, `net_mesh_shutdown`, `net_mesh_start`, `net_mesh_public_key_hex`, `net_mesh_entity_id` | Create / start / tear down a mesh node. |
| Connections | `net_mesh_connect`, `net_mesh_accept`, `net_mesh_connect_direct` | Establish encrypted peer sessions. |
| Streams | `net_mesh_open_stream`, `net_mesh_send`, `net_mesh_send_with_retry`, `net_mesh_send_blocking`, `net_mesh_stream_stats`, `net_mesh_recv_shard` | Per-peer ordered byte streams. |
| Channels | `net_mesh_register_channel`, `net_mesh_subscribe_channel`, `net_mesh_subscribe_channel_with_token`, `net_mesh_unsubscribe_channel`, `net_mesh_publish` | Topic-based pub/sub over the mesh. |
| Capabilities | `net_mesh_announce_capabilities`, `net_mesh_find_nodes`, `net_mesh_find_nodes_scoped`, `net_mesh_find_best_node`, `net_mesh_find_best_node_scoped` | Capability discovery + scored placement. |
| NAT traversal | `net_mesh_nat_type`, `net_mesh_reflex_addr`, `net_mesh_peer_nat_type`, `net_mesh_probe_reflex`, `net_mesh_reclassify_nat`, `net_mesh_traversal_stats`, `net_mesh_set_reflex_override`, `net_mesh_clear_reflex_override` | Optional optimization — routed-handshake fallback always works. |

### Scoped capability discovery

`scope:*` reserved tags on a `CapabilitySet` narrow *who finds whom*
at query time. The wire format and forwarders are unchanged —
enforcement is purely query-side.

| Tag form               | Effect                                                          |
|------------------------|-----------------------------------------------------------------|
| _(none)_               | `Global` (default) — visible to every query that doesn't opt out. |
| `scope:subnet-local`   | Visible only under `{"kind":"same_subnet"}` queries.            |
| `scope:tenant:<id>`    | Visible to `{"kind":"tenant","tenant":"<id>"}` queries (and to permissive global queries). |
| `scope:region:<name>`  | Visible to `{"kind":"region","region":"<name>"}` queries.       |

```c
// GPU pool advertised to one tenant only.
net_mesh_announce_capabilities(mesh,
    "{\"tags\":[\"model:llama3-70b\",\"scope:tenant:oem-123\"]}");

// Tenant-scoped query.
char* result = NULL; size_t result_len = 0;
net_mesh_find_nodes_scoped(mesh,
    "{\"require_tags\":[\"model:llama3-70b\"]}",
    "{\"kind\":\"tenant\",\"tenant\":\"oem-123\"}",
    &result, &result_len);
net_free_string(result);

// Scored placement — pick the highest-scoring node within a scope.
uint64_t winner = 0;
int has_match = 0;
net_mesh_find_best_node_scoped(mesh,
    "{\"filter\":{\"require_gpu\":true},\"prefer_more_vram\":1.0}",
    "{\"kind\":\"tenant\",\"tenant\":\"oem-123\"}",
    &winner, &has_match);
if (has_match) printf("placement -> %llu\n", (unsigned long long)winner);
```

`scope.kind` accepts `any` (default) | `global_only` | `same_subnet`
| `tenant` (with `tenant`) | `tenants` (with `tenants`) | `region`
(with `region`) | `regions` (with `regions`). Both snake_case
(`global_only`) and camelCase (`globalOnly`) are accepted so
fixtures round-trip across SDKs. Strictest scope wins —
`scope:subnet-local` dominates tenant/region tags on the same set.

`net_mesh_find_best_node[_scoped]` use an out-param contract: the
return code is 0 on both hit and miss; `*out_has_match` is `1` on
hit (with `*out_node_id` populated) or `0` on miss. The boolean
disambiguates from `node_id == 0`, which is a valid id.

Full design + cross-SDK rationale:
[`docs/SCOPED_CAPABILITIES_PLAN.md`](../docs/SCOPED_CAPABILITIES_PLAN.md).

### Mesh types

```c
net_meshnode_t      // Opaque mesh-node handle (separate from net_handle_t).
net_mesh_stream_t   // Opaque per-peer stream handle.
```

### Where to look for full prose

- [`net.h`](../bindings/go/net/net.h) — every function has a doc-comment
  with input shapes, error codes, and ownership rules.
- [`bindings/go/README.md`](../bindings/go/README.md) — runnable
  examples for the full mesh surface (the Go bindings are a thin
  wrapper over `net.h`, so the example translation back to C is
  near-1:1).
- [`net/README.md`](../README.md) — architectural overview, NAT
  traversal design, channel visibility model.

## License

Apache-2.0
