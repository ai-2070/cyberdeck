// Throughput benchmarks for Net Go SDK.
//
// Measures pure SDK throughput using the in-memory noop adapter.
//
// Run with: go test -bench=. -benchmem ./net/
package net

import (
	"encoding/json"
	"fmt"
	"runtime"
	"testing"
)

// Sample events of different sizes
var (
	smallEvent  = `{"t":"x"}`
	mediumEvent = mustMarshal(map[string]interface{}{
		"token":      "hello_world",
		"index":      42,
		"session_id": "abc123def456",
		"timestamp":  1234567890,
		"metadata":   map[string]interface{}{"key": "value"},
	})
	largeEvent = mustMarshal(map[string]interface{}{
		"token":      "hello_world_this_is_a_longer_token",
		"index":      42,
		"session_id": "abc123def456ghi789jkl012mno345",
		"timestamp":  1234567890,
		"user_id":    "user_12345678901234567890",
		"request_id": "req_abcdefghijklmnopqrstuvwxyz",
		"metadata": map[string]interface{}{
			"key1":   "value1_with_some_extra_data",
			"key2":   "value2_with_more_extra_data",
			"key3":   "value3_with_even_more_data",
			"nested": map[string]interface{}{"a": 1, "b": 2, "c": 3, "d": 4, "e": 5},
		},
		"tags":    []string{"tag1", "tag2", "tag3", "tag4", "tag5"},
		"content": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
	})
)

func mustMarshal(v interface{}) string {
	data, err := json.Marshal(v)
	if err != nil {
		panic(err)
	}
	return string(data)
}

// createBenchBus creates a bus with noop in-memory adapter for benchmarking
func createBenchBus(numShards int) (*Net, error) {
	return New(&Config{
		NumShards:          numShards,
		RingBufferCapacity: 1 << 20, // 1M events per shard
		BackpressureMode:   "DropOldest",
		// No backend config = uses noop adapter
	})
}

// BenchmarkIngestRaw_Small benchmarks raw ingestion with small events
func BenchmarkIngestRaw_Small(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if err := bus.IngestRaw(smallEvent); err != nil {
			b.Fatalf("Ingestion failed: %v", err)
		}
	}
}

// BenchmarkIngestRaw_Medium benchmarks raw ingestion with medium events
func BenchmarkIngestRaw_Medium(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if err := bus.IngestRaw(mediumEvent); err != nil {
			b.Fatalf("Ingestion failed: %v", err)
		}
	}
}

// BenchmarkIngestRaw_Large benchmarks raw ingestion with large events
func BenchmarkIngestRaw_Large(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if err := bus.IngestRaw(largeEvent); err != nil {
			b.Fatalf("Ingestion failed: %v", err)
		}
	}
}

// BenchmarkIngest_Medium benchmarks struct ingestion (includes JSON marshaling)
func BenchmarkIngest_Medium(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	event := map[string]interface{}{
		"token":      "hello_world",
		"index":      42,
		"session_id": "abc123def456",
		"timestamp":  1234567890,
		"metadata":   map[string]interface{}{"key": "value"},
	}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		if err := bus.Ingest(event); err != nil {
			b.Fatalf("Ingestion failed: %v", err)
		}
	}
}

// BenchmarkIngestRawBatch benchmarks batch ingestion with different batch sizes
func BenchmarkIngestRawBatch(b *testing.B) {
	batchSizes := []int{10, 100, 500, 1000}

	for _, batchSize := range batchSizes {
		b.Run(fmt.Sprintf("batch_%d", batchSize), func(b *testing.B) {
			bus, err := createBenchBus(4)
			if err != nil {
				b.Fatalf("Failed to create bus: %v", err)
			}
			defer bus.Shutdown()

			batch := make([]string, batchSize)
			for i := range batch {
				batch[i] = mediumEvent
			}

			b.ResetTimer()
			b.ReportAllocs()

			for i := 0; i < b.N; i++ {
				if count := bus.IngestRawBatch(batch); count == 0 {
					b.Fatalf("Batch ingestion returned 0")
				}
			}
		})
	}
}

// BenchmarkShardCount benchmarks with different shard counts
func BenchmarkShardCount(b *testing.B) {
	shardCounts := []int{1, 2, 4, 8, 16}

	for _, numShards := range shardCounts {
		b.Run(fmt.Sprintf("shards_%d", numShards), func(b *testing.B) {
			bus, err := createBenchBus(numShards)
			if err != nil {
				b.Fatalf("Failed to create bus: %v", err)
			}
			defer bus.Shutdown()

			b.ResetTimer()
			b.ReportAllocs()

			for i := 0; i < b.N; i++ {
				if err := bus.IngestRaw(mediumEvent); err != nil {
					b.Fatalf("Ingestion failed: %v", err)
				}
			}
		})
	}
}

// BenchmarkParallelIngestion benchmarks parallel ingestion from multiple goroutines.
// Uses default GOMAXPROCS goroutines via RunParallel.
func BenchmarkParallelIngestion(b *testing.B) {
	bus, err := createBenchBus(runtime.NumCPU())
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	b.ResetTimer()
	b.ReportAllocs()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if err := bus.IngestRaw(mediumEvent); err != nil {
				b.Errorf("Ingestion failed: %v", err)
			}
		}
	})
}

// BenchmarkHighThroughput simulates high-throughput batch scenario using RunParallel.
func BenchmarkHighThroughput(b *testing.B) {
	bus, err := createBenchBus(runtime.NumCPU())
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	// Pre-create batch
	batchSize := 1000
	batch := make([]string, batchSize)
	for i := range batch {
		batch[i] = mediumEvent
	}

	b.ResetTimer()
	b.ReportAllocs()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_ = bus.IngestRawBatch(batch)
		}
	})
}

// BenchmarkRawVsStruct compares raw string vs struct ingestion
func BenchmarkRawVsStruct(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	event := map[string]interface{}{
		"token":      "hello_world",
		"index":      42,
		"session_id": "abc123def456",
		"timestamp":  1234567890,
		"metadata":   map[string]interface{}{"key": "value"},
	}

	b.Run("IngestRaw", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if err := bus.IngestRaw(mediumEvent); err != nil {
				b.Fatalf("Ingestion failed: %v", err)
			}
		}
	})

	b.Run("Ingest", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if err := bus.Ingest(event); err != nil {
				b.Fatalf("Ingestion failed: %v", err)
			}
		}
	})
}

// BenchmarkEventSizes compares performance across event sizes
func BenchmarkEventSizes(b *testing.B) {
	bus, err := createBenchBus(4)
	if err != nil {
		b.Fatalf("Failed to create bus: %v", err)
	}
	defer bus.Shutdown()

	events := map[string]string{
		"small":  smallEvent,
		"medium": mediumEvent,
		"large":  largeEvent,
	}

	for name, event := range events {
		b.Run(fmt.Sprintf("size_%s_%dB", name, len(event)), func(b *testing.B) {
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				if err := bus.IngestRaw(event); err != nil {
					b.Fatalf("Ingestion failed: %v", err)
				}
			}
		})
	}
}
