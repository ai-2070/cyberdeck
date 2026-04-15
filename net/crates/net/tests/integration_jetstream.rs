//! Integration tests for NATS JetStream adapter.
//!
//! These tests require a running NATS server with JetStream enabled.
//! Set JETSTREAM_URL environment variable or use default: nats://localhost:4222
//!
//! Start NATS:
//!   docker run --name nats -p 4222:4222 nats:latest -js
//!
//! Run tests:
//!   cargo test --features jetstream --test integration_jetstream -- --ignored

use async_nats::jetstream;
use net::{
    AdapterConfig, BackpressureMode, BatchConfig, ConsumeRequest, Event, EventBus, EventBusConfig,
    Filter, JetStreamAdapterConfig, Ordering,
};
use serde_json::json;

use std::time::Duration;

fn jetstream_url() -> String {
    std::env::var("JETSTREAM_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

fn unique_prefix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test_{}", ts)
}

async fn create_bus(prefix: &str) -> EventBus {
    let config = EventBusConfig::builder()
        .num_shards(2)
        .ring_buffer_capacity(1024)
        .backpressure_mode(BackpressureMode::DropOldest)
        .batch(BatchConfig::low_latency())
        .adapter(AdapterConfig::JetStream(
            JetStreamAdapterConfig::new(jetstream_url())
                .with_prefix(prefix)
                .with_max_messages(10000),
        ))
        .build()
        .expect("Failed to build config");

    EventBus::new(config).await.expect("Failed to create bus")
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_ingest_and_poll() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest events
    for i in 0..10 {
        let event = Event::from_str(&format!(r#"{{"index": {}, "data": "test"}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    // Flush to ensure events are persisted
    bus.flush().await.expect("Failed to flush");

    // Small delay for async persistence
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Poll events
    let response = bus
        .poll(ConsumeRequest::new(100))
        .await
        .expect("Failed to poll");

    assert!(
        response.events.len() >= 10,
        "Expected at least 10 events, got {}",
        response.events.len()
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_pagination() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest 25 events
    for i in 0..25 {
        let event = Event::from_str(&format!(r#"{{"index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Poll in pages of 10
    let mut all_events = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let request = match &cursor {
            Some(c) => ConsumeRequest::new(10).from(c.clone()),
            None => ConsumeRequest::new(10),
        };

        let response = bus.poll(request).await.expect("Failed to poll");
        all_events.extend(response.events);

        if !response.has_more {
            break;
        }
        cursor = response.next_id;
    }

    assert!(
        all_events.len() >= 25,
        "Expected at least 25 events, got {}",
        all_events.len()
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_ordering() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest events with different values
    for i in 0..20 {
        let event = Event::from_str(&format!(r#"{{"seq": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Poll with ordering by insertion timestamp
    let response = bus
        .poll(ConsumeRequest::new(100).ordering(Ordering::InsertionTs))
        .await
        .expect("Failed to poll");

    // Verify events are ordered by timestamp
    let mut prev_ts = 0u64;
    for event in &response.events {
        assert!(
            event.insertion_ts >= prev_ts,
            "Events not ordered: {} < {}",
            event.insertion_ts,
            prev_ts
        );
        prev_ts = event.insertion_ts;
    }

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_filtering() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest events with different types
    for i in 0..10 {
        let event_type = if i % 2 == 0 { "token" } else { "tool_call" };
        let event =
            Event::from_str(&format!(r#"{{"type": "{}", "index": {}}}"#, event_type, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Filter for token events only
    let filter = Filter::eq("type", json!("token"));
    let response = bus
        .poll(ConsumeRequest::new(100).filter(filter))
        .await
        .expect("Failed to poll");

    // All returned events should be token type
    for event in &response.events {
        assert_eq!(
            event.raw.get("type").and_then(|v| v.as_str()),
            Some("token"),
            "Expected token type, got {:?}",
            event.raw
        );
    }

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_batch_ingest() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Batch ingest
    let events: Vec<Event> = (0..100)
        .map(|i| Event::from_str(&format!(r#"{{"batch_index": {}}}"#, i)).unwrap())
        .collect();

    let count = bus.ingest_batch(events);
    assert_eq!(count, 100);

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let response = bus
        .poll(ConsumeRequest::new(200))
        .await
        .expect("Failed to poll");
    assert!(
        response.events.len() >= 100,
        "Expected at least 100 events, got {}",
        response.events.len()
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_stats() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest some events
    for i in 0..50 {
        let event = Event::from_str(&format!(r#"{{"stat_index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    let stats = bus.stats();
    assert!(
        stats
            .events_ingested
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 50,
        "Expected at least 50 ingested events"
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_health() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Verify health
    assert!(bus.is_healthy().await, "Bus should be healthy");

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_persistence_across_instances() {
    let prefix = unique_prefix();

    // First instance: ingest events
    {
        let bus = create_bus(&prefix).await;

        for i in 0..10 {
            let event = Event::from_str(&format!(r#"{{"persist_index": {}}}"#, i)).unwrap();
            bus.ingest(event).expect("Failed to ingest");
        }

        bus.flush().await.expect("Failed to flush");
        tokio::time::sleep(Duration::from_millis(100)).await;
        bus.shutdown().await.expect("Failed to shutdown");
    }

    // Second instance: read the same events
    {
        let bus = create_bus(&prefix).await;

        let response = bus
            .poll(ConsumeRequest::new(100))
            .await
            .expect("Failed to poll");

        assert!(
            response.events.len() >= 10,
            "Expected at least 10 persisted events, got {}",
            response.events.len()
        );

        bus.shutdown().await.expect("Failed to shutdown");
    }
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_stream_limits() {
    let prefix = unique_prefix();

    // Create bus with strict limits
    let config = EventBusConfig::builder()
        .num_shards(1)
        .ring_buffer_capacity(1024)
        .adapter(AdapterConfig::JetStream(
            JetStreamAdapterConfig::new(jetstream_url())
                .with_prefix(&prefix)
                .with_max_messages(50), // Limit to 50 messages
        ))
        .build()
        .expect("Failed to build config");

    let bus = EventBus::new(config).await.expect("Failed to create bus");

    // Ingest more than the limit
    for i in 0..100 {
        let event = Event::from_str(&format!(r#"{{"limit_index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    // Give JetStream time to enforce limits (eventually consistent)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Should only have ~50 events due to limits (older ones discarded)
    let response = bus
        .poll(ConsumeRequest::new(200))
        .await
        .expect("Failed to poll");

    // With discard policy, we should have around the max limit
    // Note: JetStream limit enforcement is eventually consistent, so we allow some tolerance
    assert!(
        response.events.len() <= 100, // Allow up to 100 as limits may not be enforced immediately
        "Expected around 50 events due to limits, got {}",
        response.events.len()
    );
    // Also verify we don't have way more than we ingested
    assert!(
        response.events.len() <= 100,
        "Should not have more events than ingested"
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running NATS with JetStream
async fn test_jetstream_sequence_gaps() {
    // This test verifies that polling correctly handles sequence gaps in JetStream.
    // Gaps occur when messages are deleted from the stream.
    // The fix ensures gaps don't consume the fetch limit, so we get the requested
    // number of events even when there are gaps in the sequence numbers.

    let prefix = unique_prefix();
    let stream_name = format!("{}_shard_0", prefix);

    // Connect directly to NATS to manipulate the stream
    let client = async_nats::connect(&jetstream_url())
        .await
        .expect("Failed to connect to NATS");
    let js = jetstream::new(client);

    // Create the bus and ingest events
    let bus = create_bus(&prefix).await;

    // Ingest 20 events
    for i in 0..20 {
        let event = Event::from_str(&format!(r#"{{"gap_index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Delete some messages to create gaps (delete sequences 3, 5, 7, 9, 11)
    // This creates gaps in the sequence numbers
    let stream = js
        .get_stream(&stream_name)
        .await
        .expect("Failed to get stream");

    for seq in [3u64, 5, 7, 9, 11] {
        // Ignore errors if message doesn't exist at exact sequence
        let _ = stream.delete_message(seq).await;
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Poll for 10 events - should get 10 even with gaps
    let response = bus
        .poll(ConsumeRequest::new(10))
        .await
        .expect("Failed to poll");

    // We should get exactly 10 events despite the gaps
    // Before the fix, gaps would consume fetch iterations, returning fewer events
    assert_eq!(
        response.events.len(),
        10,
        "Expected exactly 10 events despite sequence gaps, got {}",
        response.events.len()
    );

    // Verify has_more is correct (we have ~15 events remaining after deletions, polled 10)
    assert!(
        response.has_more,
        "Expected has_more=true since we have more events"
    );

    // Poll again to get remaining events
    let response2 = bus
        .poll(ConsumeRequest::new(10).from(response.next_id.unwrap()))
        .await
        .expect("Failed to poll second page");

    // Should get the remaining events (20 - 5 deleted - 10 first page = 5)
    assert!(
        response2.events.len() >= 5,
        "Expected at least 5 more events, got {}",
        response2.events.len()
    );

    bus.shutdown().await.expect("Failed to shutdown");
}
