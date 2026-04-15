# C SDK Plan

The C SDK is the thinnest layer — a header file and a shared library. It already exists as `src/ffi/mod.rs` and `bindings/go/net/net.h`. The plan is to clean it up, make it a first-class deliverable, and document it properly.

The C SDK unlocks every language that can call C, which is all of them.

## Current State

The FFI layer (`src/ffi/mod.rs`) already exposes:
- `net_init(config_json)` → opaque handle
- `net_ingest(handle, json, len)` → error code
- `net_ingest_raw(handle, json, len)` → error code (fastest)
- `net_ingest_raw_batch(handle, jsons, lens, count)` → count
- `net_ingest_batch(handle, events_json)` → count
- `net_poll(handle, request_json, out_buffer, buffer_len)` → bytes written
- `net_stats(handle, out_buffer, buffer_len)` → bytes written
- `net_flush(handle)` → error code
- `net_shutdown(handle)` → error code
- `net_num_shards(handle)` → u16
- `net_version()` → static string
- `net_generate_keypair()` → JSON string (caller frees)
- `net_free_string(s)` → void

The header exists at `bindings/go/net/net.h` but needs cleanup.

## Phase 1: Fix the Rename

The header still uses `BLACKSTREAM_H` include guard and `BLACKSTREAM_` error prefixes.

```c
// Before
#ifndef BLACKSTREAM_H
#define BLACKSTREAM_H
BLACKSTREAM_SUCCESS = 0,
BLACKSTREAM_ERR_NULL_POINTER = -1,

// After
#ifndef NET_H
#define NET_H
NET_SUCCESS = 0,
NET_ERR_NULL_POINTER = -1,
```

## Phase 2: Standalone Header

Move the header from `bindings/go/net/net.h` to `include/net.h` as a first-class deliverable. The Go binding can reference it from there.

```
include/
  net.h              # The C header — the entire C SDK
```

## Phase 3: Structured Return Types

The current API returns JSON strings for poll and stats. For C consumers who don't want to parse JSON, add structured accessors:

```c
/* Structured ingestion result */
typedef struct {
    uint16_t shard_id;
    uint64_t timestamp;
} net_receipt_t;

/* Ingest raw with receipt */
int net_ingest_raw_ex(net_handle_t handle, const char* json, size_t len, net_receipt_t* out);

/* Structured event */
typedef struct {
    const char* id;
    size_t id_len;
    const char* raw;
    size_t raw_len;
    uint64_t insertion_ts;
    uint16_t shard_id;
} net_event_t;

/* Structured poll result */
typedef struct {
    net_event_t* events;
    size_t count;
    const char* next_id;
    int has_more;
} net_poll_result_t;

/* Poll with structured result (caller frees with net_free_poll_result) */
int net_poll_ex(net_handle_t handle, size_t limit, const char* cursor, net_poll_result_t* out);
void net_free_poll_result(net_poll_result_t* result);

/* Structured stats */
typedef struct {
    uint64_t events_ingested;
    uint64_t events_dropped;
    uint64_t batches_dispatched;
} net_stats_t;

/* Stats without JSON serialization */
int net_stats_ex(net_handle_t handle, net_stats_t* out);
```

The `_ex` variants avoid JSON overhead entirely. The original JSON-based functions remain for convenience.

## Phase 4: Callback-Based Subscription

```c
/* Callback for received events */
typedef void (*net_event_callback_t)(const net_event_t* event, void* user_data);

/* Subscribe to events with a callback. Returns a subscription handle. */
net_handle_t net_subscribe(
    net_handle_t handle,
    size_t batch_limit,
    net_event_callback_t callback,
    void* user_data
);

/* Stop a subscription. */
void net_unsubscribe(net_handle_t subscription);
```

The subscription runs on an internal thread, calling the callback with each event. The user provides a `void* user_data` that gets passed through — standard C pattern.

## Phase 5: Example Program

```c
#include "net.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    /* Create a node */
    net_handle_t node = net_init("{\"num_shards\": 4}");
    if (!node) {
        fprintf(stderr, "Failed to initialize\n");
        return 1;
    }

    /* Ingest events */
    const char* event = "{\"token\": \"hello\"}";
    int rc = net_ingest_raw(node, event, strlen(event));
    if (rc < 0) {
        fprintf(stderr, "Ingest failed: %d\n", rc);
    }

    /* Batch ingest */
    const char* events[] = {
        "{\"token\": \"a\"}",
        "{\"token\": \"b\"}",
        "{\"token\": \"c\"}",
    };
    size_t lens[] = { 15, 15, 15 };
    int count = net_ingest_raw_batch(node, events, lens, 3);
    printf("Ingested %d events\n", count);

    /* Poll */
    char buffer[65536];
    int bytes = net_poll(node, "{\"limit\": 100}", buffer, sizeof(buffer));
    if (bytes > 0) {
        printf("Poll response: %.*s\n", bytes, buffer);
    }

    /* Stats */
    net_stats_t stats;
    net_stats_ex(node, &stats);
    printf("Ingested: %llu, Dropped: %llu\n",
        (unsigned long long)stats.events_ingested,
        (unsigned long long)stats.events_dropped);

    /* Shutdown */
    net_shutdown(node);
    return 0;
}
```

Build:
```bash
# Build the shared library
cargo build --release --features ffi,net

# Compile and link
gcc -o example example.c -L target/release -lnet -lpthread -ldl -lm
```

## Phase 6: pkg-config and CMake Support

```
lib/
  pkgconfig/
    net.pc             # pkg-config file
  cmake/
    NetConfig.cmake    # CMake find module
```

`net.pc`:
```
prefix=/usr/local
libdir=${prefix}/lib
includedir=${prefix}/include

Name: net
Description: Net mesh network C SDK
Version: 0.5.0
Libs: -L${libdir} -lnet -lpthread -ldl -lm
Cflags: -I${includedir}
```

This lets C/C++ projects do:
```bash
gcc $(pkg-config --cflags --libs net) -o app app.c
```

Or in CMake:
```cmake
find_package(Net REQUIRED)
target_link_libraries(myapp Net::Net)
```

## Deliverable

The C SDK ships as:
- `include/net.h` — the header
- `lib/libnet.so` (Linux) / `lib/libnet.dylib` (macOS) / `lib/net.dll` (Windows)
- `lib/pkgconfig/net.pc` — pkg-config
- `lib/cmake/NetConfig.cmake` — CMake support
- `examples/basic.c` — example program

One header, one library. That's the entire SDK.

## What This Unlocks

Every language that can call C gets Net for free:
- **C++** — direct include
- **Zig** — `@cImport`
- **Nim** — `importc`
- **Lua/LuaJIT** — FFI
- **Ruby** — FFI gem
- **Java** — JNI or Panama
- **C#/.NET** — P/Invoke
- **Dart/Flutter** — dart:ffi
- **Swift** — bridging header
- **Kotlin Native** — cinterop
- **Haskell** — FFI
- **Erlang/Elixir** — NIFs
- **PHP** — FFI extension

One header. Every language.
