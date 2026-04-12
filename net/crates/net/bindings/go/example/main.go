// Example usage of the Blackstream Go bindings.
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/ai-2070/blackstream/bindings/go/blackstream"
)

func main() {
	fmt.Printf("Blackstream version: %s\n", blackstream.Version())

	// Create event bus with default configuration
	bus, err := blackstream.New(nil)
	if err != nil {
		log.Fatalf("Failed to create event bus: %v", err)
	}
	defer bus.Shutdown()

	fmt.Printf("Event bus created with %d shards\n", bus.NumShards())

	// Ingest some events using the fast raw path
	events := []string{
		`{"type": "token", "value": "hello", "index": 0}`,
		`{"type": "token", "value": "world", "index": 1}`,
		`{"type": "tool_call", "name": "search", "args": {"query": "AI"}}`,
	}

	for _, e := range events {
		if err := bus.IngestRaw(e); err != nil {
			log.Printf("Failed to ingest event: %v", err)
		}
	}
	fmt.Printf("Ingested %d events\n", len(events))

	// Ingest using Go structs
	type TokenEvent struct {
		Type  string `json:"type"`
		Value string `json:"value"`
		Index int    `json:"index"`
	}

	if err := bus.Ingest(TokenEvent{Type: "token", Value: "!", Index: 2}); err != nil {
		log.Printf("Failed to ingest struct event: %v", err)
	}

	// Batch ingest
	batchEvents := []string{
		`{"type": "token", "value": "batch1"}`,
		`{"type": "token", "value": "batch2"}`,
		`{"type": "token", "value": "batch3"}`,
	}
	ingested := bus.IngestRawBatch(batchEvents)
	fmt.Printf("Batch ingested %d events\n", ingested)

	// Give workers time to process
	time.Sleep(100 * time.Millisecond)

	// Get statistics
	stats, err := bus.Stats()
	if err != nil {
		log.Printf("Failed to get stats: %v", err)
	} else {
		fmt.Printf("Stats: ingested=%d, dropped=%d\n",
			stats.EventsIngested, stats.EventsDropped)
	}

	// Poll events
	response, err := bus.Poll(100, "")
	if err != nil {
		log.Fatalf("Failed to poll: %v", err)
	}

	fmt.Printf("Polled %d events (has_more=%v)\n", response.Count, response.HasMore)
	for i, raw := range response.Events {
		var event map[string]interface{}
		if err := json.Unmarshal(raw, &event); err == nil {
			fmt.Printf("  Event %d: %v\n", i, event)
		}
	}

	// Pagination example
	if response.HasMore {
		nextResponse, err := bus.Poll(100, response.NextID)
		if err != nil {
			log.Printf("Failed to poll next page: %v", err)
		} else {
			fmt.Printf("Next page: %d events\n", nextResponse.Count)
		}
	}

	// Flush and shutdown
	if err := bus.Flush(); err != nil {
		log.Printf("Failed to flush: %v", err)
	}

	fmt.Println("Done!")
}
