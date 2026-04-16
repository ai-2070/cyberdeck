//! Cross-shard poll merge layer.
//!
//! This module handles polling from multiple shards and merging the results
//! into a unified stream with proper cursor management.
//!
//! # Composite Cursor
//!
//! When polling multiple shards, we track position in each shard using a
//! composite cursor encoded as base64 JSON:
//!
//! ```json
//! {"0": "1702123456789-0", "1": "1702123456790-0", ...}
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};

use crate::adapter::{Adapter, ShardPollResult};
use crate::consumer::filter::Filter;
use crate::error::{AdapterError, ConsumerError};
use crate::event::StoredEvent;

/// Ordering mode for consumed events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ordering {
    /// Return events in arbitrary order (fastest).
    #[default]
    None,
    /// Sort events by insertion timestamp (cross-shard ordering).
    InsertionTs,
}

/// Composite cursor tracking position across multiple shards.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompositeCursor {
    /// Per-shard positions (shard_id -> stream_id).
    #[serde(flatten)]
    pub positions: HashMap<u16, String>,
}

impl CompositeCursor {
    /// Create an empty cursor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Encode the cursor as a base64 string.
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(&self.positions).unwrap_or_default();
        BASE64.encode(json.as_bytes())
    }

    /// Decode a cursor from a base64 string.
    pub fn decode(s: &str) -> Result<Self, ConsumerError> {
        let bytes = BASE64
            .decode(s)
            .map_err(|e| ConsumerError::InvalidCursor(e.to_string()))?;

        let positions: HashMap<u16, String> = serde_json::from_slice(&bytes)
            .map_err(|e| ConsumerError::InvalidCursor(e.to_string()))?;

        Ok(Self { positions })
    }

    /// Get the position for a specific shard.
    pub fn get(&self, shard_id: u16) -> Option<&str> {
        self.positions.get(&shard_id).map(|s| s.as_str())
    }

    /// Set the position for a specific shard.
    pub fn set(&mut self, shard_id: u16, position: String) {
        self.positions.insert(shard_id, position);
    }

    /// Update positions from consumed events.
    pub fn update_from_events(&mut self, events: &[StoredEvent]) {
        for event in events {
            self.positions.insert(event.shard_id, event.id.clone());
        }
    }
}

/// Request for consuming events.
#[derive(Debug, Clone, Default)]
pub struct ConsumeRequest {
    /// Start cursor (opaque to caller). None means from the beginning.
    pub from_id: Option<String>,
    /// Maximum number of events to return.
    pub limit: usize,
    /// Optional filter to apply.
    pub filter: Option<Filter>,
    /// Ordering mode.
    pub ordering: Ordering,
    /// Specific shards to poll. None means all shards.
    pub shards: Option<Vec<u16>>,
}

impl ConsumeRequest {
    /// Create a new consume request.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            ..Default::default()
        }
    }

    /// Set the starting cursor.
    pub fn from(mut self, cursor: impl Into<String>) -> Self {
        self.from_id = Some(cursor.into());
        self
    }

    /// Set the filter.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set the ordering mode.
    pub fn ordering(mut self, ordering: Ordering) -> Self {
        self.ordering = ordering;
        self
    }

    /// Set specific shards to poll.
    pub fn shards(mut self, shards: Vec<u16>) -> Self {
        self.shards = Some(shards);
        self
    }
}

/// Response from consuming events.
#[derive(Debug, Clone)]
pub struct ConsumeResponse {
    /// Events matching the request.
    pub events: Vec<StoredEvent>,
    /// Cursor for the next poll. None if no events returned.
    pub next_id: Option<String>,
    /// True if there are more events available.
    pub has_more: bool,
}

impl ConsumeResponse {
    /// Create an empty response.
    pub fn empty() -> Self {
        Self {
            events: Vec::new(),
            next_id: None,
            has_more: false,
        }
    }
}

/// Poll merger for cross-shard aggregation.
pub struct PollMerger {
    /// Adapter for polling shards.
    adapter: Arc<dyn Adapter>,
    /// Total number of shards.
    num_shards: u16,
}

impl PollMerger {
    /// Create a new poll merger.
    pub fn new(adapter: Arc<dyn Adapter>, num_shards: u16) -> Self {
        Self {
            adapter,
            num_shards,
        }
    }

    /// Poll events according to the request.
    pub async fn poll(&self, request: ConsumeRequest) -> Result<ConsumeResponse, ConsumerError> {
        if request.limit == 0 {
            return Ok(ConsumeResponse::empty());
        }

        // Decode cursor
        let cursor = match &request.from_id {
            Some(s) => CompositeCursor::decode(s)?,
            None => CompositeCursor::new(),
        };

        // Determine which shards to poll
        let shards: Vec<u16> = request
            .shards
            .clone()
            .unwrap_or_else(|| (0..self.num_shards).collect());

        if shards.is_empty() {
            return Ok(ConsumeResponse::empty());
        }

        // Calculate per-shard limit (over-fetch to account for filtering)
        // Use ceiling division to avoid truncating to 0 when limit < shard count
        let over_fetch_factor = if request.filter.is_some() { 3 } else { 2 };
        let per_shard_limit = (((request.limit + shards.len() - 1) / shards.len()).max(1)
            * over_fetch_factor)
            .min(10_000);

        // Poll all shards in parallel
        let poll_futures: Vec<_> = shards
            .iter()
            .map(|&shard_id| {
                let adapter = self.adapter.clone();
                let from = cursor.get(shard_id).map(|s| s.to_string());
                async move {
                    let result = adapter
                        .poll_shard(shard_id, from.as_deref(), per_shard_limit)
                        .await;
                    (shard_id, result)
                }
            })
            .collect();

        let shard_results: Vec<(u16, Result<ShardPollResult, AdapterError>)> =
            futures::future::join_all(poll_futures).await;

        // Collect results, tracking errors
        let mut all_events = Vec::new();
        let mut any_has_more = false;
        let mut new_cursor = cursor.clone();

        for (shard_id, result) in shard_results {
            match result {
                Ok(shard_result) => {
                    // Update cursor with last position from this shard
                    if let Some(next_id) = &shard_result.next_id {
                        new_cursor.set(shard_id, next_id.clone());
                    }
                    any_has_more |= shard_result.has_more;
                    all_events.extend(shard_result.events);
                }
                Err(e) => {
                    tracing::warn!(
                        shard_id = shard_id,
                        error = %e,
                        "Failed to poll shard, skipping"
                    );
                    // Continue with other shards
                }
            }
        }

        // Apply filter
        if let Some(filter) = &request.filter {
            all_events.retain(|e| e.parse().map(|v| filter.matches(&v)).unwrap_or(false));
        }

        // Apply ordering
        match request.ordering {
            Ordering::None => {
                // Keep arbitrary order
            }
            Ordering::InsertionTs => {
                all_events.sort_by_key(|e| e.insertion_ts);
            }
        }

        // Truncate to requested limit
        let had_extra = all_events.len() > request.limit;
        all_events.truncate(request.limit);

        // Update cursor based on events actually returned to the user.
        // This ensures the next pagination request starts from where we left off,
        // not from where we fetched up to (which could skip events if we over-fetched).
        let mut final_cursor = cursor.clone();
        for event in &all_events {
            final_cursor.set(event.shard_id, event.id.clone());
        }

        let has_more = any_has_more || had_extra;
        let next_id = if all_events.is_empty() {
            None
        } else {
            Some(final_cursor.encode())
        };

        Ok(ConsumeResponse {
            events: all_events,
            next_id,
            has_more,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cursor_encode_decode() {
        let mut cursor = CompositeCursor::new();
        cursor.set(0, "1702123456789-0".to_string());
        cursor.set(1, "1702123456790-0".to_string());
        cursor.set(5, "1702123456795-0".to_string());

        let encoded = cursor.encode();
        let decoded = CompositeCursor::decode(&encoded).unwrap();

        assert_eq!(decoded.get(0), Some("1702123456789-0"));
        assert_eq!(decoded.get(1), Some("1702123456790-0"));
        assert_eq!(decoded.get(5), Some("1702123456795-0"));
        assert_eq!(decoded.get(2), None);
    }

    #[test]
    fn test_cursor_update_from_events() {
        let mut cursor = CompositeCursor::new();

        let events = vec![
            StoredEvent::from_value("100-0".to_string(), json!({}), 100, 0),
            StoredEvent::from_value("200-0".to_string(), json!({}), 200, 1),
            StoredEvent::from_value("150-0".to_string(), json!({}), 150, 0), // Later event in shard 0
        ];

        cursor.update_from_events(&events);

        // Should have the last position for each shard
        assert_eq!(cursor.get(0), Some("150-0")); // Updated to later event
        assert_eq!(cursor.get(1), Some("200-0"));
    }

    #[test]
    fn test_consume_request_builder() {
        let request = ConsumeRequest::new(100)
            .from("some_cursor")
            .ordering(Ordering::InsertionTs)
            .shards(vec![0, 1, 2])
            .filter(Filter::eq("type", json!("token")));

        assert_eq!(request.limit, 100);
        assert_eq!(request.from_id, Some("some_cursor".to_string()));
        assert_eq!(request.ordering, Ordering::InsertionTs);
        assert_eq!(request.shards, Some(vec![0, 1, 2]));
        assert!(request.filter.is_some());
    }

    #[test]
    fn test_invalid_cursor() {
        let result = CompositeCursor::decode("not_valid_base64!!!");
        assert!(result.is_err());

        // Valid base64 but not valid JSON
        let result = CompositeCursor::decode(&BASE64.encode(b"not json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_composite_cursor_new() {
        let cursor = CompositeCursor::new();
        assert!(cursor.positions.is_empty());
    }

    #[test]
    fn test_composite_cursor_default() {
        let cursor = CompositeCursor::default();
        assert!(cursor.positions.is_empty());
    }

    #[test]
    fn test_composite_cursor_get_nonexistent() {
        let cursor = CompositeCursor::new();
        assert!(cursor.get(0).is_none());
        assert!(cursor.get(100).is_none());
    }

    #[test]
    fn test_composite_cursor_set_overwrites() {
        let mut cursor = CompositeCursor::new();
        cursor.set(0, "first".to_string());
        assert_eq!(cursor.get(0), Some("first"));

        cursor.set(0, "second".to_string());
        assert_eq!(cursor.get(0), Some("second"));
    }

    #[test]
    fn test_composite_cursor_empty_encode() {
        let cursor = CompositeCursor::new();
        let encoded = cursor.encode();
        let decoded = CompositeCursor::decode(&encoded).unwrap();
        assert!(decoded.positions.is_empty());
    }

    #[test]
    fn test_composite_cursor_clone() {
        let mut cursor = CompositeCursor::new();
        cursor.set(0, "pos-0".to_string());
        cursor.set(1, "pos-1".to_string());

        let cloned = cursor.clone();
        assert_eq!(cloned.get(0), Some("pos-0"));
        assert_eq!(cloned.get(1), Some("pos-1"));
    }

    #[test]
    fn test_composite_cursor_debug() {
        let mut cursor = CompositeCursor::new();
        cursor.set(0, "test".to_string());
        let debug = format!("{:?}", cursor);
        assert!(debug.contains("CompositeCursor"));
        assert!(debug.contains("positions"));
    }

    #[test]
    fn test_ordering_default() {
        let ordering = Ordering::default();
        assert_eq!(ordering, Ordering::None);
    }

    #[test]
    fn test_ordering_clone_copy() {
        let ordering = Ordering::InsertionTs;
        let cloned = ordering;
        assert_eq!(cloned, Ordering::InsertionTs);
    }

    #[test]
    fn test_ordering_debug() {
        assert!(format!("{:?}", Ordering::None).contains("None"));
        assert!(format!("{:?}", Ordering::InsertionTs).contains("InsertionTs"));
    }

    #[test]
    fn test_consume_request_new() {
        let request = ConsumeRequest::new(50);
        assert_eq!(request.limit, 50);
        assert!(request.from_id.is_none());
        assert!(request.filter.is_none());
        assert_eq!(request.ordering, Ordering::None);
        assert!(request.shards.is_none());
    }

    #[test]
    fn test_consume_request_default() {
        let request = ConsumeRequest::default();
        assert_eq!(request.limit, 0);
        assert!(request.from_id.is_none());
        assert!(request.filter.is_none());
        assert_eq!(request.ordering, Ordering::None);
        assert!(request.shards.is_none());
    }

    #[test]
    fn test_consume_request_from_string() {
        let request = ConsumeRequest::new(10).from(String::from("cursor123"));
        assert_eq!(request.from_id, Some("cursor123".to_string()));
    }

    #[test]
    fn test_consume_request_clone() {
        let request = ConsumeRequest::new(100)
            .from("cursor")
            .ordering(Ordering::InsertionTs)
            .shards(vec![0, 1]);

        let cloned = request.clone();
        assert_eq!(cloned.limit, 100);
        assert_eq!(cloned.from_id, Some("cursor".to_string()));
        assert_eq!(cloned.ordering, Ordering::InsertionTs);
        assert_eq!(cloned.shards, Some(vec![0, 1]));
    }

    #[test]
    fn test_consume_request_debug() {
        let request = ConsumeRequest::new(10);
        let debug = format!("{:?}", request);
        assert!(debug.contains("ConsumeRequest"));
        assert!(debug.contains("limit"));
    }

    #[test]
    fn test_consume_response_empty() {
        let response = ConsumeResponse::empty();
        assert!(response.events.is_empty());
        assert!(response.next_id.is_none());
        assert!(!response.has_more);
    }

    #[test]
    fn test_consume_response_clone() {
        let mut response = ConsumeResponse::empty();
        response.next_id = Some("cursor".to_string());
        response.has_more = true;

        let cloned = response.clone();
        assert_eq!(cloned.next_id, Some("cursor".to_string()));
        assert!(cloned.has_more);
    }

    #[test]
    fn test_consume_response_debug() {
        let response = ConsumeResponse::empty();
        let debug = format!("{:?}", response);
        assert!(debug.contains("ConsumeResponse"));
        assert!(debug.contains("events"));
    }

    #[test]
    fn test_cursor_update_from_empty_events() {
        let mut cursor = CompositeCursor::new();
        cursor.set(0, "original".to_string());

        let events: Vec<StoredEvent> = vec![];
        cursor.update_from_events(&events);

        // Cursor should be unchanged
        assert_eq!(cursor.get(0), Some("original"));
    }

    #[test]
    fn test_cursor_many_shards() {
        let mut cursor = CompositeCursor::new();
        for i in 0..100u16 {
            cursor.set(i, format!("pos-{}", i));
        }

        let encoded = cursor.encode();
        let decoded = CompositeCursor::decode(&encoded).unwrap();

        for i in 0..100u16 {
            assert_eq!(decoded.get(i), Some(format!("pos-{}", i).as_str()));
        }
    }

    #[test]
    fn test_consume_request_empty_shards() {
        let request = ConsumeRequest::new(100).shards(vec![]);
        assert_eq!(request.shards, Some(vec![]));
    }

    #[test]
    fn test_consume_request_ordering_none() {
        let request = ConsumeRequest::new(100).ordering(Ordering::None);
        assert_eq!(request.ordering, Ordering::None);
    }

    #[test]
    fn test_ordering_equality() {
        assert_eq!(Ordering::None, Ordering::None);
        assert_eq!(Ordering::InsertionTs, Ordering::InsertionTs);
        assert_ne!(Ordering::None, Ordering::InsertionTs);
    }

    // Mock adapter for testing PollMerger
    use crate::adapter::{Adapter, ShardPollResult};
    use crate::error::AdapterError;
    use crate::event::Batch;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::RwLock;

    struct MockAdapter {
        events: RwLock<HashMap<u16, Vec<StoredEvent>>>,
    }

    impl MockAdapter {
        fn new() -> Self {
            Self {
                events: RwLock::new(HashMap::new()),
            }
        }

        fn add_events(&self, shard_id: u16, events: Vec<StoredEvent>) {
            let mut map = self.events.write().unwrap();
            map.entry(shard_id).or_default().extend(events);
        }
    }

    #[async_trait]
    impl Adapter for MockAdapter {
        async fn init(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn on_batch(&self, _batch: Batch) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn flush(&self) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn poll_shard(
            &self,
            shard_id: u16,
            from_id: Option<&str>,
            limit: usize,
        ) -> Result<ShardPollResult, AdapterError> {
            let map = self.events.read().unwrap();
            let events = map.get(&shard_id).cloned().unwrap_or_default();

            // Filter by from_id if provided
            let filtered: Vec<_> = if let Some(from) = from_id {
                events
                    .into_iter()
                    .skip_while(|e| e.id != from)
                    .skip(1) // Skip the from_id itself
                    .collect()
            } else {
                events
            };

            let has_more = filtered.len() > limit;
            let events: Vec<_> = filtered.into_iter().take(limit).collect();
            let next_id = events.last().map(|e| e.id.clone());

            Ok(ShardPollResult {
                events,
                next_id,
                has_more,
            })
        }

        fn name(&self) -> &'static str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_poll_merger_new() {
        let adapter = Arc::new(MockAdapter::new());
        let merger = PollMerger::new(adapter, 4);
        assert_eq!(merger.num_shards, 4);
    }

    #[tokio::test]
    async fn test_poll_merger_empty_limit() {
        let adapter = Arc::new(MockAdapter::new());
        let merger = PollMerger::new(adapter, 4);

        let request = ConsumeRequest::new(0);
        let response = merger.poll(request).await.unwrap();

        assert!(response.events.is_empty());
        assert!(response.next_id.is_none());
        assert!(!response.has_more);
    }

    #[tokio::test]
    async fn test_poll_merger_empty_shards() {
        let adapter = Arc::new(MockAdapter::new());
        let merger = PollMerger::new(adapter, 4);

        let request = ConsumeRequest::new(100).shards(vec![]);
        let response = merger.poll(request).await.unwrap();

        assert!(response.events.is_empty());
        assert!(response.next_id.is_none());
        assert!(!response.has_more);
    }

    #[tokio::test]
    async fn test_poll_merger_with_events() {
        let adapter = Arc::new(MockAdapter::new());

        // Add events to shard 0
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "a"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "b"}), 200, 0),
            ],
        );

        // Add events to shard 1
        adapter.add_events(
            1,
            vec![StoredEvent::from_value(
                "1-1".to_string(),
                json!({"type": "c"}),
                150,
                1,
            )],
        );

        let merger = PollMerger::new(adapter, 2);

        let request = ConsumeRequest::new(100);
        let response = merger.poll(request).await.unwrap();

        assert_eq!(response.events.len(), 3);
        assert!(response.next_id.is_some());
    }

    #[tokio::test]
    async fn test_poll_merger_with_ordering() {
        let adapter = Arc::new(MockAdapter::new());

        // Add events with different timestamps
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({}), 300, 0),
                StoredEvent::from_value("0-2".to_string(), json!({}), 100, 0),
            ],
        );
        adapter.add_events(
            1,
            vec![StoredEvent::from_value(
                "1-1".to_string(),
                json!({}),
                200,
                1,
            )],
        );

        let merger = PollMerger::new(adapter, 2);

        let request = ConsumeRequest::new(100).ordering(Ordering::InsertionTs);
        let response = merger.poll(request).await.unwrap();

        // Events should be sorted by insertion_ts
        assert_eq!(response.events.len(), 3);
        assert_eq!(response.events[0].insertion_ts, 100);
        assert_eq!(response.events[1].insertion_ts, 200);
        assert_eq!(response.events[2].insertion_ts, 300);
    }

    #[tokio::test]
    async fn test_poll_merger_with_filter() {
        let adapter = Arc::new(MockAdapter::new());

        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "token"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "message"}), 200, 0),
                StoredEvent::from_value("0-3".to_string(), json!({"type": "token"}), 300, 0),
            ],
        );

        let merger = PollMerger::new(adapter, 1);

        let request = ConsumeRequest::new(100).filter(Filter::eq("type", json!("token")));
        let response = merger.poll(request).await.unwrap();

        assert_eq!(response.events.len(), 2);
        for event in &response.events {
            assert!(event.raw_str().contains("token"));
        }
    }

    #[tokio::test]
    async fn test_poll_merger_with_limit() {
        let adapter = Arc::new(MockAdapter::new());

        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({}), 200, 0),
                StoredEvent::from_value("0-3".to_string(), json!({}), 300, 0),
            ],
        );

        let merger = PollMerger::new(adapter, 1);

        let request = ConsumeRequest::new(2);
        let response = merger.poll(request).await.unwrap();

        assert_eq!(response.events.len(), 2);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn test_poll_merger_specific_shards() {
        let adapter = Arc::new(MockAdapter::new());

        adapter.add_events(
            0,
            vec![StoredEvent::from_value(
                "0-1".to_string(),
                json!({"shard": 0}),
                100,
                0,
            )],
        );
        adapter.add_events(
            1,
            vec![StoredEvent::from_value(
                "1-1".to_string(),
                json!({"shard": 1}),
                100,
                1,
            )],
        );
        adapter.add_events(
            2,
            vec![StoredEvent::from_value(
                "2-1".to_string(),
                json!({"shard": 2}),
                100,
                2,
            )],
        );

        let merger = PollMerger::new(adapter, 3);

        // Only poll shard 0 and 2
        let request = ConsumeRequest::new(100).shards(vec![0, 2]);
        let response = merger.poll(request).await.unwrap();

        assert_eq!(response.events.len(), 2);
        let shard_ids: Vec<_> = response.events.iter().map(|e| e.shard_id).collect();
        assert!(shard_ids.contains(&0));
        assert!(shard_ids.contains(&2));
        assert!(!shard_ids.contains(&1));
    }

    #[tokio::test]
    async fn test_poll_merger_with_cursor() {
        let adapter = Arc::new(MockAdapter::new());

        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({}), 200, 0),
                StoredEvent::from_value("0-3".to_string(), json!({}), 300, 0),
            ],
        );

        let merger = PollMerger::new(adapter, 1);

        // First poll
        let request = ConsumeRequest::new(2);
        let response1 = merger.poll(request).await.unwrap();
        assert_eq!(response1.events.len(), 2);

        // Second poll with cursor
        let cursor = response1.next_id.unwrap();
        let request2 = ConsumeRequest::new(10).from(cursor);
        let response2 = merger.poll(request2).await.unwrap();

        assert_eq!(response2.events.len(), 1);
        assert_eq!(response2.events[0].id, "0-3");
    }

    #[tokio::test]
    async fn test_poll_merger_pagination_multi_shard() {
        // Test that pagination across multiple shards doesn't skip events
        let adapter = Arc::new(MockAdapter::new());

        // Shard 0: 10 events
        let shard0_events: Vec<_> = (1..=10)
            .map(|i| {
                StoredEvent::from_value(
                    format!("0-{}", i),
                    json!({"shard": 0, "idx": i}),
                    i as u64 * 10,
                    0,
                )
            })
            .collect();
        adapter.add_events(0, shard0_events);

        // Shard 1: 15 events
        let shard1_events: Vec<_> = (1..=15)
            .map(|i| {
                StoredEvent::from_value(
                    format!("1-{}", i),
                    json!({"shard": 1, "idx": i}),
                    i as u64 * 10 + 5,
                    1,
                )
            })
            .collect();
        adapter.add_events(1, shard1_events);

        let merger = PollMerger::new(adapter, 2);

        // Poll in pages of 10 and collect all events
        let mut all_events = Vec::new();
        let mut cursor: Option<String> = None;
        let mut iterations = 0;

        loop {
            iterations += 1;
            let request = match &cursor {
                Some(c) => ConsumeRequest::new(10).from(c.clone()),
                None => ConsumeRequest::new(10),
            };

            let response = merger.poll(request).await.unwrap();
            all_events.extend(response.events);

            if !response.has_more {
                break;
            }
            cursor = response.next_id;

            // Safety: prevent infinite loop
            if iterations > 10 {
                panic!("Too many iterations");
            }
        }

        // Should get all 25 events (10 from shard 0 + 15 from shard 1)
        assert_eq!(
            all_events.len(),
            25,
            "Expected 25 events, got {}. Iterations: {}",
            all_events.len(),
            iterations
        );

        // Verify we got events from both shards
        let shard0_count = all_events.iter().filter(|e| e.shard_id == 0).count();
        let shard1_count = all_events.iter().filter(|e| e.shard_id == 1).count();
        assert_eq!(shard0_count, 10, "Expected 10 events from shard 0");
        assert_eq!(shard1_count, 15, "Expected 15 events from shard 1");
    }

    #[tokio::test]
    async fn test_poll_merger_pagination_no_duplicates() {
        // Test that pagination doesn't return duplicate events
        let adapter = Arc::new(MockAdapter::new());

        // Add events to both shards
        for shard_id in 0..2u16 {
            let events: Vec<_> = (1..=20)
                .map(|i| {
                    StoredEvent::from_value(
                        format!("{}-{}", shard_id, i),
                        json!({"shard": shard_id, "idx": i}),
                        i as u64 * 10,
                        shard_id,
                    )
                })
                .collect();
            adapter.add_events(shard_id, events);
        }

        let merger = PollMerger::new(adapter, 2);

        // Poll in small pages
        let mut all_event_ids = Vec::new();
        let mut cursor: Option<String> = None;

        for _ in 0..20 {
            let request = match &cursor {
                Some(c) => ConsumeRequest::new(5).from(c.clone()),
                None => ConsumeRequest::new(5),
            };

            let response = merger.poll(request).await.unwrap();
            all_event_ids.extend(response.events.iter().map(|e| e.id.clone()));

            if !response.has_more {
                break;
            }
            cursor = response.next_id;
        }

        // Check for duplicates
        let unique_count = {
            let mut ids = all_event_ids.clone();
            ids.sort();
            ids.dedup();
            ids.len()
        };

        assert_eq!(
            unique_count,
            all_event_ids.len(),
            "Found duplicate events! Total: {}, Unique: {}",
            all_event_ids.len(),
            unique_count
        );

        // Should have all 40 events
        assert_eq!(all_event_ids.len(), 40);
    }

    #[tokio::test]
    async fn test_poll_merger_pagination_with_ordering() {
        // Test pagination with timestamp ordering
        let adapter = Arc::new(MockAdapter::new());

        // Add events with interleaved timestamps across shards
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({}), 300, 0),
                StoredEvent::from_value("0-3".to_string(), json!({}), 500, 0),
            ],
        );
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({}), 200, 1),
                StoredEvent::from_value("1-2".to_string(), json!({}), 400, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);

        // Poll with ordering, page size 2
        let mut all_events = Vec::new();
        let mut cursor: Option<String> = None;

        for _ in 0..5 {
            let mut request = ConsumeRequest::new(2).ordering(Ordering::InsertionTs);
            if let Some(c) = &cursor {
                request = request.from(c.clone());
            }

            let response = merger.poll(request).await.unwrap();
            all_events.extend(response.events);

            if !response.has_more {
                break;
            }
            cursor = response.next_id;
        }

        // Should get all 5 events
        assert_eq!(all_events.len(), 5);

        // Verify ordering is maintained
        let timestamps: Vec<_> = all_events.iter().map(|e| e.insertion_ts).collect();
        let mut sorted = timestamps.clone();
        sorted.sort();
        assert_eq!(timestamps, sorted, "Events should be sorted by timestamp");
    }

    #[tokio::test]
    async fn test_poll_merger_cursor_tracks_returned_events_only() {
        // Test that cursor tracks position based on returned events, not fetched events
        let adapter = Arc::new(MockAdapter::new());

        // Shard 0: 3 events
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({}), 200, 0),
                StoredEvent::from_value("0-3".to_string(), json!({}), 300, 0),
            ],
        );

        // Shard 1: 3 events
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({}), 150, 1),
                StoredEvent::from_value("1-2".to_string(), json!({}), 250, 1),
                StoredEvent::from_value("1-3".to_string(), json!({}), 350, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);

        // First poll with limit 2 - should get 2 events and cursor should reflect only those 2
        let response1 = merger.poll(ConsumeRequest::new(2)).await.unwrap();
        assert_eq!(response1.events.len(), 2);
        assert!(response1.has_more);

        // Decode cursor to verify it tracks returned events
        let next_id = response1.next_id.clone().unwrap();
        let cursor = CompositeCursor::decode(&next_id).unwrap();

        // Cursor should only have positions for shards that had events in the returned set
        let returned_shard_ids: std::collections::HashSet<_> =
            response1.events.iter().map(|e| e.shard_id).collect();

        for shard_id in 0..2u16 {
            if returned_shard_ids.contains(&shard_id) {
                // Shard had returned events, cursor should have position
                assert!(
                    cursor.get(shard_id).is_some(),
                    "Cursor should have position for shard {} which had returned events",
                    shard_id
                );
            }
        }

        // Second poll should continue from where we left off
        let response2 = merger
            .poll(ConsumeRequest::new(10).from(next_id))
            .await
            .unwrap();

        // Should get remaining 4 events
        assert_eq!(response2.events.len(), 4, "Should get remaining 4 events");
    }
}
