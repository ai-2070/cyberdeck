/* Minimal sanity check for the C binding.
 *
 * Build:
 *   gcc hello.c -L /path/to/libnet -lnet -lpthread -ldl -lm -o hello
 * Run:
 *   ./hello
 *
 * What it proves: the library loads, a node initializes, you can ingest a
 * JSON event, poll it back, free the result, and shutdown cleanly.
 *
 * No named-channel API and no async — write the poll yourself.
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "net.h"

int main(void) {
    net_handle_t node = net_init("{\"num_shards\": 1}");
    if (!node) {
        fprintf(stderr, "net_init failed\n");
        return 1;
    }

    const char* json = "{\"msg\":\"hello, mesh\"}";
    if (net_ingest_raw(node, json, strlen(json)) != 0) {
        fprintf(stderr, "net_ingest_raw failed\n");
        net_shutdown(node);
        return 1;
    }

    /* Small wait for the drain worker. */
    usleep(20 * 1000);

    net_poll_result_t result;
    int rc = net_poll_ex(node, 10, NULL, &result);
    if (rc != 0) {
        fprintf(stderr, "net_poll_ex failed (rc=%d)\n", rc);
        net_shutdown(node);
        return 1;
    }
    if (result.count == 0) {
        fprintf(stderr, "no events received\n");
        net_free_poll_result(&result);
        net_shutdown(node);
        return 1;
    }

    printf("received: %.*s\n", (int)result.events[0].raw_len, result.events[0].raw);

    net_free_poll_result(&result);
    net_shutdown(node);
    return 0;
}
