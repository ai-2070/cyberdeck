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
| `net_poll(handle, request_json, buf, buf_len)` | Poll (JSON interface). |
| `net_poll_ex(handle, limit, cursor, &result)` | Poll (structured, no JSON). Free with `net_free_poll_result`. |
| `net_free_poll_result(&result)` | Free a structured poll result. |

### Statistics

| Function | Description |
|----------|-------------|
| `net_stats(handle, buf, buf_len)` | Stats (JSON interface). |
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

## License

Apache-2.0
