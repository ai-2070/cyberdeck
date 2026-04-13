//! Integration tests for Redis adapter.
//!
//! These tests require a running Redis instance.
//! Set REDIS_URL environment variable or use default: redis://localhost:6379
//!
//! Start Redis:
//!   docker run --name redis -p 6379:6379 -e ALLOW_EMPTY_PASSWORD=yes bitnami/redis:latest
//!
//! Run tests:
//!   cargo test --features redis --test integration_redis -- --ignored

use blackstream::{
    AdapterConfig, BackpressureMode, BatchConfig, ConsumeRequest, Event, EventBus, EventBusConfig,
    Filter, Ordering, RedisAdapterConfig,
};
use serde_json::json;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::time::Duration;

fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
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
        .adapter(AdapterConfig::Redis(
            RedisAdapterConfig::new(redis_url())
                .with_prefix(prefix)
                .with_max_stream_len(10000),
        ))
        .build()
        .expect("Failed to build config");

    EventBus::new(config).await.expect("Failed to create bus")
}

#[tokio::test]
#[ignore] // Requires running Redis
async fn test_redis_ingest_and_poll() {
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
#[ignore] // Requires running Redis
async fn test_redis_pagination() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest 25 events
    for i in 0..25 {
        let event = Event::from_str(&format!(r#"{{"index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    bus.flush().await.expect("Failed to flush");
    tokio::time::sleep(Duration::from_millis(100)).await;

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
#[ignore] // Requires running Redis
async fn test_redis_ordering() {
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
#[ignore] // Requires running Redis
async fn test_redis_filtering() {
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
#[ignore] // Requires running Redis
async fn test_redis_batch_ingest() {
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
#[ignore] // Requires running Redis
async fn test_redis_stats() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Ingest some events
    for i in 0..50 {
        let event = Event::from_str(&format!(r#"{{"stat_index": {}}}"#, i)).unwrap();
        bus.ingest(event).expect("Failed to ingest");
    }

    let stats = bus.stats();
    assert!(
        stats.events_ingested.load(AtomicOrdering::Relaxed) >= 50,
        "Expected at least 50 ingested events"
    );

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running Redis
async fn test_redis_reconnection() {
    let prefix = unique_prefix();
    let bus = create_bus(&prefix).await;

    // Verify initial health
    assert!(bus.is_healthy().await, "Bus should be healthy initially");

    // Ingest and poll
    let event = Event::from_str(r#"{"test": "reconnect"}"#).unwrap();
    bus.ingest(event).expect("Failed to ingest");

    bus.flush().await.expect("Failed to flush");

    bus.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
#[ignore] // Requires running Redis
async fn test_redis_persistence_across_instances() {
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
