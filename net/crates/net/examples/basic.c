/*
 * Net C SDK — Basic Example
 *
 * Build:
 *   cargo build --release --features ffi,net
 *   gcc -o basic basic.c -L ../target/release -lnet -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./basic    (Linux)
 *   DYLD_LIBRARY_PATH=../target/release ./basic   (macOS)
 */

#include "../include/net.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    printf("Net %s\n\n", net_version());

    /* Create a node with 4 shards */
    net_handle_t node = net_init("{\"num_shards\": 4}");
    if (!node) {
        fprintf(stderr, "Failed to initialize\n");
        return 1;
    }
    printf("Node created with %d shards\n", net_num_shards(node));

    /* Ingest a single event with receipt */
    const char* event = "{\"token\": \"hello\", \"index\": 0}";
    net_receipt_t receipt;
    int rc = net_ingest_raw_ex(node, event, strlen(event), &receipt);
    if (rc == NET_SUCCESS) {
        printf("Ingested to shard %d at ts %llu\n",
            receipt.shard_id, (unsigned long long)receipt.timestamp);
    }

    /* Batch ingest */
    const char* events[] = {
        "{\"token\": \"world\", \"index\": 1}",
        "{\"token\": \"foo\",   \"index\": 2}",
        "{\"token\": \"bar\",   \"index\": 3}",
    };
    size_t lens[] = {
        strlen(events[0]),
        strlen(events[1]),
        strlen(events[2]),
    };
    int count = net_ingest_raw_batch(node, events, lens, 3);
    if (count < 0) {
        fprintf(stderr, "Batch ingest failed: %d\n", count);
    } else {
        printf("Batch ingested %d events\n", count);
    }

    /* Flush to ensure events are available for polling */
    rc = net_flush(node);
    if (rc != NET_SUCCESS) {
        fprintf(stderr, "Flush failed: %d\n", rc);
    }

    /* Poll with structured API (no JSON overhead) */
    net_poll_result_t result;
    rc = net_poll_ex(node, 100, NULL, &result);
    if (rc == NET_SUCCESS) {
        printf("\nPolled %zu events (has_more=%d):\n", result.count, result.has_more);
        for (size_t i = 0; i < result.count; i++) {
            printf("  [shard %d] %.*s\n",
                result.events[i].shard_id,
                (int)result.events[i].raw_len,
                result.events[i].raw);
        }
        net_free_poll_result(&result);
    }

    /* Stats (structured) */
    net_stats_t stats;
    net_stats_ex(node, &stats);
    printf("\nStats: ingested=%llu dropped=%llu batches=%llu\n",
        (unsigned long long)stats.events_ingested,
        (unsigned long long)stats.events_dropped,
        (unsigned long long)stats.batches_dispatched);

    /* Shutdown */
    net_shutdown(node);
    printf("Node shut down.\n");

    return 0;
}
