/*
 * Net C API Header
 * 
 * High-performance event bus for AI runtime workloads.
 * This header provides C-compatible bindings for use with CGO.
 */

#ifndef NET_H
#define NET_H

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
    NET_ERR_UNKNOWN = -99
} net_error_t;

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

#ifdef __cplusplus
}
#endif

#endif /* NET_H */
