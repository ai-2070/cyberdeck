/*
 * Net C SDK
 *
 * Network Event Transport — a latency-first encrypted mesh protocol.
 *
 * One header, one shared library. This is the entire C SDK.
 * Links against libnet.so (Linux), libnet.dylib (macOS), or net.dll (Windows).
 *
 * Thread Safety: All functions are thread-safe. Handles can be shared across threads.
 *
 * Memory: Handles from net_init() must be freed with net_shutdown().
 *         Poll results from net_poll_ex() must be freed with net_free_poll_result().
 *         Strings from net_generate_keypair() must be freed with net_free_string().
 */

#ifndef NET_H
#define NET_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/* Types                                                                     */
/* ========================================================================= */

/* Opaque handle to the event bus. */
typedef void* net_handle_t;

/* Error codes. */
typedef enum {
    NET_SUCCESS              =  0,
    NET_ERR_NULL_POINTER     = -1,
    NET_ERR_INVALID_UTF8     = -2,
    NET_ERR_INVALID_JSON     = -3,
    NET_ERR_INIT_FAILED      = -4,
    NET_ERR_INGESTION_FAILED = -5,
    NET_ERR_POLL_FAILED      = -6,
    NET_ERR_BUFFER_TOO_SMALL = -7,
    NET_ERR_SHUTTING_DOWN    = -8,
    NET_ERR_UNKNOWN          = -99
} net_error_t;

/* Ingestion receipt. */
typedef struct {
    uint16_t shard_id;
    uint64_t timestamp;
} net_receipt_t;

/* A single stored event. */
typedef struct {
    const char* id;
    size_t      id_len;
    const char* raw;
    size_t      raw_len;
    uint64_t    insertion_ts;
    uint16_t    shard_id;
} net_event_t;

/* Poll result containing events and cursor. */
typedef struct {
    net_event_t* events;
    size_t       count;
    char*        next_id;
    int          has_more;
} net_poll_result_t;

/* Ingestion statistics. */
typedef struct {
    uint64_t events_ingested;
    uint64_t events_dropped;
    uint64_t batches_dispatched;
} net_stats_t;

/* ========================================================================= */
/* Lifecycle                                                                 */
/* ========================================================================= */

/*
 * Initialize a new node.
 *
 * @param config_json  JSON config string (null-terminated), or NULL for defaults.
 * @return  Handle to the node, or NULL on failure.
 *
 * Example: net_init("{\"num_shards\": 4}")
 * Example: net_init(NULL)  // defaults
 */
net_handle_t net_init(const char* config_json);

/*
 * Shut down the node and free all resources.
 * The handle is invalid after this call.
 *
 * @return  0 on success, negative error code on failure.
 */
int net_shutdown(net_handle_t handle);

/*
 * Get the library version string (static, do not free).
 */
const char* net_version(void);

/*
 * Get the number of shards.
 *
 * @return  Number of shards, or 0 if handle is NULL.
 */
uint16_t net_num_shards(net_handle_t handle);

/* ========================================================================= */
/* Ingestion                                                                 */
/* ========================================================================= */

/*
 * Ingest a raw JSON string (fastest path, no parsing).
 *
 * @param json  JSON string (not null-terminated — length is explicit).
 * @param len   Length of the JSON string in bytes.
 * @return  0 on success, negative error code on failure.
 */
int net_ingest_raw(net_handle_t handle, const char* json, size_t len);

/*
 * Ingest a raw JSON string and get a receipt.
 *
 * @param json  JSON string.
 * @param len   Length of the JSON string in bytes.
 * @param out   Receipt output (shard_id, timestamp). May be NULL.
 * @return  0 on success, negative error code on failure.
 */
int net_ingest_raw_ex(net_handle_t handle, const char* json, size_t len, net_receipt_t* out);

/*
 * Ingest a single event (parses JSON for validation).
 *
 * @param event_json  JSON event string.
 * @param len         Length of the event string in bytes.
 * @return  0 on success, negative error code on failure.
 */
int net_ingest(net_handle_t handle, const char* event_json, size_t len);

/*
 * Ingest multiple raw JSON strings in a batch.
 *
 * @param jsons  Array of pointers to JSON strings.
 * @param lens   Array of lengths for each string.
 * @param count  Number of events.
 * @return  Number of successfully ingested events, or negative error code.
 */
int net_ingest_raw_batch(
    net_handle_t handle,
    const char** jsons,
    const size_t* lens,
    size_t count
);

/*
 * Ingest events from a JSON array string.
 *
 * @param events_json  JSON array (null-terminated).
 * @return  Number of ingested events, or negative error code.
 */
int net_ingest_batch(net_handle_t handle, const char* events_json);

/* ========================================================================= */
/* Consumption                                                               */
/* ========================================================================= */

/*
 * Poll events (JSON interface).
 *
 * @param request_json  JSON request, e.g. {"limit": 100}. NULL for defaults.
 * @param out_buffer    Output buffer for JSON response.
 * @param buffer_len    Size of output buffer.
 * @return  Bytes written on success, negative error code on failure.
 */
int net_poll(
    net_handle_t handle,
    const char* request_json,
    char* out_buffer,
    size_t buffer_len
);

/*
 * Poll events (structured interface, no JSON overhead).
 *
 * @param limit   Maximum number of events.
 * @param cursor  Resume cursor (null-terminated), or NULL for start.
 * @param out     Poll result output. Must be freed with net_free_poll_result().
 * @return  0 on success, negative error code on failure.
 */
int net_poll_ex(
    net_handle_t handle,
    size_t limit,
    const char* cursor,
    net_poll_result_t* out
);

/*
 * Free a poll result returned by net_poll_ex().
 */
void net_free_poll_result(net_poll_result_t* result);

/* ========================================================================= */
/* Statistics                                                                */
/* ========================================================================= */

/*
 * Get statistics (JSON interface).
 *
 * @param out_buffer  Output buffer for JSON.
 * @param buffer_len  Size of output buffer.
 * @return  Bytes written on success, negative error code on failure.
 */
int net_stats(net_handle_t handle, char* out_buffer, size_t buffer_len);

/*
 * Get statistics (structured, no JSON overhead).
 *
 * @param out  Stats output.
 * @return  0 on success, negative error code on failure.
 */
int net_stats_ex(net_handle_t handle, net_stats_t* out);

/* ========================================================================= */
/* Utilities                                                                 */
/* ========================================================================= */

/*
 * Flush pending batches to the adapter.
 *
 * @return  0 on success, negative error code on failure.
 */
int net_flush(net_handle_t handle);

/*
 * Generate a new keypair for encrypted mesh transport.
 *
 * @return  JSON string with hex-encoded public_key and secret_key.
 *          Caller must free with net_free_string(). NULL if not available.
 */
char* net_generate_keypair(void);

/*
 * Free a string returned by net_generate_keypair() or similar.
 */
void net_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* NET_H */
