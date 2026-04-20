/*
 * Net C API Header
 *
 * High-performance event bus for AI runtime workloads.
 * This header provides C-compatible bindings for use with CGO.
 */

#ifndef NET_SDK_H
#define NET_SDK_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to the event bus */
typedef void* net_handle_t;

/* Error codes */
typedef enum {
    NET_SUCCESS = 0,
    NET_ERR_NULL_POINTER = -1,
    NET_ERR_INVALID_UTF8 = -2,
    NET_ERR_INVALID_JSON = -3,
    NET_ERR_INIT_FAILED = -4,
    NET_ERR_INGESTION_FAILED = -5,
    NET_ERR_POLL_FAILED = -6,
    NET_ERR_BUFFER_TOO_SMALL = -7,
    NET_ERR_SHUTTING_DOWN = -8,
    NET_ERR_UNKNOWN = -99,
    /* CortEX / RedEX surface (compiled when the Rust cdylib has
     * `netdb` + `redex-disk` features on). Codes below -99 so they
     * don't collide with the event-bus surface above. */
    NET_ERR_CORTEX_CLOSED = -100,
    NET_ERR_CORTEX_FOLD = -101,
    NET_ERR_NETDB = -102,
    NET_ERR_REDEX = -103
} net_error_t;

/* Watch / tail cursor status codes. Returned from net_*_next functions
 * instead of the negative error scheme; positive to distinguish
 * "no event available" from an actual failure. */
#define NET_STREAM_TIMEOUT 1
#define NET_STREAM_ENDED   2

/*
 * Initialize a new event bus with optional JSON configuration.
 *
 * @param config_json JSON configuration string (UTF-8, null-terminated), or NULL for defaults.
 * @return Handle to the event bus, or NULL on failure.
 *
 * Example config:
 * {
 *   "num_shards": 8,
 *   "ring_buffer_capacity": 1048576,
 *   "backpressure_mode": "DropOldest"
 * }
 */
net_handle_t net_init(const char* config_json);

/*
 * Ingest a single event (parses JSON).
 *
 * @param handle Event bus handle.
 * @param event_json JSON event string.
 * @param len Length of the event string.
 * @return 0 on success, negative error code on failure.
 */
int net_ingest(net_handle_t handle, const char* event_json, size_t len);

/*
 * Ingest a raw JSON string (fastest path, no parsing).
 *
 * @param handle Event bus handle.
 * @param json JSON string.
 * @param len Length of the JSON string.
 * @return 0 on success, negative error code on failure.
 */
int net_ingest_raw(net_handle_t handle, const char* json, size_t len);

/*
 * Ingest multiple raw JSON strings in a batch.
 *
 * @param handle Event bus handle.
 * @param jsons Array of pointers to JSON strings.
 * @param lens Array of lengths for each JSON string.
 * @param count Number of events.
 * @return Number of successfully ingested events.
 */
int net_ingest_raw_batch(
    net_handle_t handle,
    const char** jsons,
    const size_t* lens,
    size_t count
);

/*
 * Ingest multiple events from a JSON array.
 *
 * @param handle Event bus handle.
 * @param events_json JSON array of events.
 * @return Number of ingested events, or negative error code.
 */
int net_ingest_batch(net_handle_t handle, const char* events_json);

/*
 * Poll events from the bus.
 *
 * @param handle Event bus handle.
 * @param request_json JSON request (e.g., {"limit": 100}).
 * @param out_buffer Output buffer for JSON response.
 * @param buffer_len Size of output buffer.
 * @return Bytes written on success, negative error code on failure.
 */
int net_poll(
    net_handle_t handle,
    const char* request_json,
    char* out_buffer,
    size_t buffer_len
);

/*
 * Get event bus statistics.
 *
 * @param handle Event bus handle.
 * @param out_buffer Output buffer for JSON statistics.
 * @param buffer_len Size of output buffer.
 * @return Bytes written on success, negative error code on failure.
 */
int net_stats(net_handle_t handle, char* out_buffer, size_t buffer_len);

/*
 * Flush pending batches to the adapter.
 *
 * @param handle Event bus handle.
 * @return 0 on success, negative error code on failure.
 */
int net_flush(net_handle_t handle);

/*
 * Get the number of shards.
 *
 * @param handle Event bus handle.
 * @return Number of shards, or 0 if handle is null.
 */
uint16_t net_num_shards(net_handle_t handle);

/*
 * Shut down the event bus and free resources.
 *
 * @param handle Event bus handle (invalid after this call).
 * @return 0 on success, negative error code on failure.
 */
int net_shutdown(net_handle_t handle);

/*
 * Get the library version.
 *
 * @return Version string (static, do not free).
 */
const char* net_version(void);

/*
 * Generate a new Net keypair (requires Net feature).
 *
 * @return JSON string with hex-encoded public_key and secret_key,
 *         or NULL if Net is not enabled. Caller must free with net_free_string.
 */
char* net_generate_keypair(void);

/*
 * Free a string returned by Net functions.
 *
 * @param s String to free (may be NULL).
 */
void net_free_string(char* s);

/* =========================================================================
 * CortEX + RedEX surface.
 *
 * Compiled when the Rust cdylib is built with `--features "netdb redex-disk"`.
 * Symbols remain unresolved when the cdylib lacks those features — Go code
 * must gate usage accordingly (the Go wrapper exposes a compile-time
 * check via a build tag).
 *
 * Watch / tail cursors:
 *   * `next(cursor, timeout_ms, &out_json, &out_len)` returns:
 *       `0`                 — event delivered; *out_json owned by caller
 *       `NET_STREAM_TIMEOUT`— no event within timeout_ms
 *       `NET_STREAM_ENDED`  — cursor reached end-of-stream
 *       negative            — net_error_t
 *     Caller frees *out_json via `net_free_string` when `0` is returned.
 * ========================================================================= */

/* Opaque handle types */
typedef struct net_redex_s           net_redex_t;
typedef struct net_redex_file_s      net_redex_file_t;
typedef struct net_redex_tail_s      net_redex_tail_t;
typedef struct net_tasks_adapter_s   net_tasks_adapter_t;
typedef struct net_tasks_watch_s     net_tasks_watch_t;
typedef struct net_memories_adapter_s net_memories_adapter_t;
typedef struct net_memories_watch_s  net_memories_watch_t;

/* ---- Redex manager ---- */
net_redex_t* net_redex_new(const char* persistent_dir);
void         net_redex_free(net_redex_t* handle);

/* ---- RedexFile ---- */
int  net_redex_open_file(net_redex_t* redex, const char* name,
                         const char* config_json,
                         net_redex_file_t** out_handle);
void net_redex_file_free(net_redex_file_t* handle);
int  net_redex_file_append(net_redex_file_t* handle, const uint8_t* payload,
                           size_t len, uint64_t* out_seq);
uint64_t net_redex_file_len(net_redex_file_t* handle);
int  net_redex_file_read_range(net_redex_file_t* handle,
                               uint64_t start, uint64_t end,
                               char** out_json, size_t* out_len);
int  net_redex_file_sync(net_redex_file_t* handle);
int  net_redex_file_close(net_redex_file_t* handle);

int  net_redex_file_tail(net_redex_file_t* handle, uint64_t from_seq,
                         net_redex_tail_t** out_cursor);
int  net_redex_tail_next(net_redex_tail_t* cursor, uint32_t timeout_ms,
                         char** out_json, size_t* out_len);
void net_redex_tail_free(net_redex_tail_t* cursor);

/* ---- Tasks adapter ---- */
int  net_tasks_adapter_open(net_redex_t* redex, uint32_t origin_hash,
                            int persistent, net_tasks_adapter_t** out_handle);
int  net_tasks_adapter_close(net_tasks_adapter_t* handle);
void net_tasks_adapter_free(net_tasks_adapter_t* handle);

int  net_tasks_create(net_tasks_adapter_t* handle, uint64_t id,
                      const char* title, uint64_t now_ns, uint64_t* out_seq);
int  net_tasks_rename(net_tasks_adapter_t* handle, uint64_t id,
                      const char* new_title, uint64_t now_ns, uint64_t* out_seq);
int  net_tasks_complete(net_tasks_adapter_t* handle, uint64_t id,
                        uint64_t now_ns, uint64_t* out_seq);
int  net_tasks_delete(net_tasks_adapter_t* handle, uint64_t id,
                      uint64_t* out_seq);
int  net_tasks_wait_for_seq(net_tasks_adapter_t* handle, uint64_t seq,
                            uint32_t timeout_ms);
int  net_tasks_list(net_tasks_adapter_t* handle, const char* filter_json,
                    char** out_json, size_t* out_len);
int  net_tasks_snapshot_and_watch(net_tasks_adapter_t* handle,
                                  const char* filter_json,
                                  char** out_snapshot, size_t* out_snapshot_len,
                                  net_tasks_watch_t** out_cursor);
int  net_tasks_watch_next(net_tasks_watch_t* cursor, uint32_t timeout_ms,
                          char** out_json, size_t* out_len);
void net_tasks_watch_free(net_tasks_watch_t* cursor);

/* ---- Memories adapter ---- */
int  net_memories_adapter_open(net_redex_t* redex, uint32_t origin_hash,
                               int persistent, net_memories_adapter_t** out_handle);
int  net_memories_adapter_close(net_memories_adapter_t* handle);
void net_memories_adapter_free(net_memories_adapter_t* handle);

/* `input_json` carries all store/retag parameters because Go strings
 * and tag lists are awkward to marshal one-field-at-a-time across cgo.
 * Shape: {"id": <u64>, "content": <str>, "tags": [<str>...],
 *         "source": <str>, "now_ns": <u64>}.
 * Retag shape drops `content` / `source`. */
int  net_memories_store(net_memories_adapter_t* handle,
                        const char* input_json, uint64_t* out_seq);
int  net_memories_retag(net_memories_adapter_t* handle,
                        const char* input_json, uint64_t* out_seq);
int  net_memories_pin(net_memories_adapter_t* handle, uint64_t id,
                      uint64_t now_ns, uint64_t* out_seq);
int  net_memories_unpin(net_memories_adapter_t* handle, uint64_t id,
                        uint64_t now_ns, uint64_t* out_seq);
int  net_memories_delete(net_memories_adapter_t* handle, uint64_t id,
                         uint64_t* out_seq);
int  net_memories_wait_for_seq(net_memories_adapter_t* handle, uint64_t seq,
                               uint32_t timeout_ms);
int  net_memories_list(net_memories_adapter_t* handle, const char* filter_json,
                       char** out_json, size_t* out_len);
int  net_memories_snapshot_and_watch(net_memories_adapter_t* handle,
                                     const char* filter_json,
                                     char** out_snapshot, size_t* out_snapshot_len,
                                     net_memories_watch_t** out_cursor);
int  net_memories_watch_next(net_memories_watch_t* cursor, uint32_t timeout_ms,
                             char** out_json, size_t* out_len);
void net_memories_watch_free(net_memories_watch_t* cursor);

#ifdef __cplusplus
}
#endif

#endif /* NET_SDK_H */
