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

/// Backing type for per-shard cursor positions. `Arc<str>` makes
/// cursor clones (and internal copies during poll merging) cheap by
/// reference-counting the id bytes rather than copying them.
type CursorPos = Arc<str>;

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
    ///
    /// Stored as `Arc<str>` so internal copies (e.g. `cursor.clone()`
    /// inside the poll merger) bump a refcount instead of duplicating
    /// each id's bytes.
    #[serde(flatten)]
    pub positions: HashMap<u16, CursorPos>,
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

        let positions: HashMap<u16, CursorPos> = serde_json::from_slice(&bytes)
            .map_err(|e| ConsumerError::InvalidCursor(e.to_string()))?;

        Ok(Self { positions })
    }

    /// Get the position for a specific shard.
    pub fn get(&self, shard_id: u16) -> Option<&str> {
        self.positions.get(&shard_id).map(|s| s.as_ref())
    }

    /// Set the position for a specific shard.
    ///
    /// Accepts anything that converts into an `Arc<str>` — notably
    /// `String`, `&str`, and `Arc<str>` itself. This lets adapters
    /// hand us a freshly-allocated `String` (becomes a single boxed
    /// allocation) without forcing a second copy for the cursor.
    pub fn set(&mut self, shard_id: u16, position: impl Into<CursorPos>) {
        self.positions.insert(shard_id, position.into());
    }

    /// Update positions from consumed events.
    pub fn update_from_events(&mut self, events: &[StoredEvent]) {
        for event in events {
            self.positions
                .insert(event.shard_id, Arc::from(event.id.as_str()));
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

/// Match a `StoredEvent` against a filter, surfacing parse failures.
///
/// Returns `true` iff the event parses as JSON AND the filter matches
/// the parsed value. A parse failure is logged at WARN with the event's
/// id and shard so on-disk corruption or framing bugs in upstream
/// adapters are observable from the filtered-poll path; without this,
/// corrupt events were silently dropped from filtered results while the
/// unfiltered path still returned them — a confusing inconsistency.
fn event_matches_filter(event: &StoredEvent, filter: &Filter) -> bool {
    match event.parse() {
        Ok(value) => filter.matches(&value),
        Err(e) => {
            tracing::warn!(
                event_id = %event.id,
                shard_id = event.shard_id,
                error = %e,
                "dropping unparseable event from filtered poll result"
            );
            false
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
        let per_shard_limit = request
            .limit
            .div_ceil(shards.len())
            .max(1)
            .saturating_mul(over_fetch_factor)
            .min(10_000);

        // Poll all shards in parallel. Each future borrows its start
        // position directly from `cursor` (which outlives `join_all` below),
        // avoiding a per-shard `String` allocation on every poll.
        let poll_futures: Vec<_> = shards
            .iter()
            .map(|&shard_id| {
                let adapter = self.adapter.clone();
                let from: Option<&str> = cursor.get(shard_id);
                async move {
                    let result = adapter.poll_shard(shard_id, from, per_shard_limit).await;
                    (shard_id, result)
                }
            })
            .collect();

        let shard_results: Vec<(u16, Result<ShardPollResult, AdapterError>)> =
            futures::future::join_all(poll_futures).await;

        // Collect results, tracking errors. Pre-allocate to the exact total
        // event count so extend() below never reallocates.
        let total_events: usize = shard_results
            .iter()
            .filter_map(|(_, r)| r.as_ref().ok().map(|sr| sr.events.len()))
            .sum();
        let mut all_events = Vec::with_capacity(total_events);
        let mut any_has_more = false;
        // `new_cursor` (fetched-position tracking) is only consulted on the
        // filter path — building it for unfiltered polls wastes a full
        // HashMap clone plus a `set()` per shard every poll.
        let mut new_cursor = if request.filter.is_some() {
            Some(cursor.clone())
        } else {
            None
        };

        for (shard_id, result) in shard_results {
            match result {
                Ok(shard_result) => {
                    // Destructure to move `next_id` out without cloning the
                    // String that the adapter already allocated for us.
                    let ShardPollResult {
                        events,
                        next_id,
                        has_more,
                    } = shard_result;
                    if let (Some(nc), Some(next_id)) = (new_cursor.as_mut(), next_id) {
                        nc.set(shard_id, next_id);
                    }
                    any_has_more |= has_more;
                    all_events.extend(events);
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

        // Apply filter.
        //
        // IMPORTANT: Use `new_cursor` (which tracks fetched positions) as
        // the base cursor so that shards whose events are entirely filtered
        // out still advance past those events. Without this, filtered-out
        // events would be re-fetched on every subsequent poll, causing an
        // infinite loop.
        //
        // Parse failures: a `StoredEvent` whose `raw` bytes don't
        // deserialize as JSON cannot match a filter, so it is dropped
        // from the filtered result. Previously this drop was silent
        // (`unwrap_or(false)`), making on-disk corruption or
        // adapter-side framing bugs invisible to operators — only
        // *unfiltered* polls would surface the bad event.
        //
        // The previous `Ordering::None` path had a `break` once
        // `kept.len() >= limit + 1`, which discarded events from later
        // shards without ever filtering them. Combined with the cursor
        // advancing past every fetched event, that meant matching
        // events on un-inspected shards were silently lost. The fix
        // uses a single full `retain` pass for both ordering modes;
        // the `lazy parse` micro-optimization is gone, but per-event
        // filter-matching is cheap and consistent semantics with the
        // sort path is worth more than parse-skip on over-fetches.
        if let Some(filter) = &request.filter {
            all_events.retain(|e| event_matches_filter(e, filter));
        }

        // Apply ordering
        match request.ordering {
            Ordering::None => {
                // Keep arbitrary order
            }
            Ordering::InsertionTs => {
                // `insertion_ts` is monotonic *per shard*, not
                // globally (see `event.rs:233`), so two events from
                // different shards can carry the same timestamp. With
                // a stable sort on `insertion_ts` alone, ties were
                // broken by the input order — which depends on
                // `futures::future::join_all`'s completion ordering
                // and is non-deterministic across polls. Combined
                // with `truncate(limit)` and the cursor-rollback step,
                // the same logical event could be returned twice or
                // skipped at the limit boundary across consecutive
                // polls.
                //
                // Add `(shard_id, id)` as deterministic
                // tiebreakers. `id` is the storage backend's
                // identifier and is unique within a shard, so the
                // composite is a strict total order.
                all_events.sort_by(|a, b| {
                    a.insertion_ts
                        .cmp(&b.insertion_ts)
                        .then(a.shard_id.cmp(&b.shard_id))
                        .then(a.id.cmp(&b.id))
                });
            }
        }

        // Track per-shard match counts *before* truncate. After
        // truncation, any shard whose match count shrank means matches
        // were dropped — and those matches must be re-fetched on the
        // next poll, otherwise they are silently lost (the cursor
        // would otherwise advance past them via `new_cursor`).
        let mut matched_per_shard: std::collections::HashMap<u16, usize> =
            std::collections::HashMap::new();
        if request.filter.is_some() {
            for e in &all_events {
                *matched_per_shard.entry(e.shard_id).or_insert(0) += 1;
            }
        }

        // Truncate to requested limit
        let had_extra = all_events.len() > request.limit;
        all_events.truncate(request.limit);

        // Build the final cursor.
        //
        // With filtering: start from `new_cursor` (fetched positions) so
        // shards whose events were entirely filtered out advance past
        // them. Then:
        //   1. For shards that had matches truncated (returned <
        //      total_matched), roll the cursor *back* to the original
        //      pre-poll position. The override step then bumps it
        //      forward to the last returned match for that shard, so
        //      the unreturned matches re-appear on the next poll.
        //   2. Override with the last *returned* event id per shard —
        //      this also prevents skipping matching events that were
        //      fetched but truncated by the limit on shards that did
        //      land in the returned set.
        //
        // Without filtering: start from the original `cursor` so shards
        // with no returned events (due to limit truncation) don't skip
        // ahead.
        let mut final_cursor = match new_cursor {
            Some(nc) => nc,
            None => cursor.clone(),
        };

        // Step 1: rollback for shards with truncated matches.
        if request.filter.is_some() && had_extra {
            let mut returned_per_shard: std::collections::HashMap<u16, usize> =
                std::collections::HashMap::new();
            for e in &all_events {
                *returned_per_shard.entry(e.shard_id).or_insert(0) += 1;
            }
            for (shard_id, &total_matched) in &matched_per_shard {
                let returned = returned_per_shard.get(shard_id).copied().unwrap_or(0);
                if returned < total_matched {
                    // Some matches for this shard were truncated. Roll
                    // back to the original cursor so they're re-fetched.
                    // The override below will move us forward to the
                    // last *returned* match (if any), so we still make
                    // progress per poll.
                    match cursor.positions.get(shard_id) {
                        Some(orig) => final_cursor.set(*shard_id, orig.clone()),
                        None => {
                            final_cursor.positions.remove(shard_id);
                        }
                    }
                }
            }
        }

        // Step 2: override to last returned event id per shard.
        // Only the last returned event per shard matters for the
        // cursor, so iterate in reverse and skip shards already seen.
        // This reduces id clones from O(all_events.len()) to
        // O(shards.len()).
        let mut seen_shards: std::collections::HashSet<u16> =
            std::collections::HashSet::with_capacity(shards.len());
        for event in all_events.iter().rev() {
            if seen_shards.insert(event.shard_id) {
                final_cursor.set(event.shard_id, event.id.clone());
            }
        }

        let cursor_advanced = final_cursor.positions != cursor.positions;
        // When filtering removed everything but we did advance past fetched
        // events, signal has_more so the caller keeps polling forward.
        let all_filtered = request.filter.is_some() && all_events.is_empty() && cursor_advanced;
        // Previously `has_more = any_has_more || had_extra ||
        // all_filtered`. If a single adapter returned
        // `ShardPollResult { events: [], next_id: None,
        // has_more: true }` (legal under the trait contract — nothing
        // forbids it), then `any_has_more=true` propagated even
        // though we made *no* progress. The caller observed
        // `(has_more=true, next_id=None)` and re-polled from the
        // same starting cursor indefinitely.
        //
        // Suppress `has_more` when the merger itself made no progress
        // at all (no events returned AND the cursor didn't advance).
        // The caller then sees a clean "nothing to do right now"
        // response and must back off rather than spin.
        let we_made_progress = !all_events.is_empty() || cursor_advanced;
        let has_more = (any_has_more || had_extra || all_filtered) && we_made_progress;
        if any_has_more && !we_made_progress {
            tracing::warn!(
                "PollMerger: an adapter reported has_more=true with no events \
                 and no cursor advance — suppressing to avoid caller infinite-loop"
            );
        }
        // Return the cursor even when all events were filtered out, so the
        // caller advances past the filtered region instead of re-fetching
        // the same events forever. The cursor is None only when nothing was
        // fetched at all (truly empty shards).
        let next_id = if we_made_progress {
            Some(final_cursor.encode())
        } else {
            None
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
            assert!(event.raw_str().unwrap().contains("token"));
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

    #[tokio::test]
    async fn test_poll_merger_small_limit_many_shards() {
        // Regression: limit < shard count caused integer division truncation to 0,
        // making per-shard fetch too small. Now uses ceiling division.
        let adapter = Arc::new(MockAdapter::new());
        let num_shards = 8u16;

        for shard_id in 0..num_shards {
            adapter.add_events(
                shard_id,
                vec![StoredEvent::from_value(
                    format!("{}-1", shard_id),
                    json!({"shard": shard_id}),
                    100,
                    shard_id,
                )],
            );
        }

        let merger = PollMerger::new(adapter, num_shards);

        // Request fewer events than shards — should still work
        let request = ConsumeRequest::new(3);
        let response = merger.poll(request).await.unwrap();

        assert_eq!(response.events.len(), 3);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn test_regression_filtered_shards_cursor_advances() {
        // Bug 3: "Cursor never advances for filtered-out shards"
        //
        // When shard 1's events are entirely filtered out, the cursor for shard 1
        // must still advance past those events. Otherwise, subsequent polls will
        // re-fetch the same filtered-out events forever.
        let adapter = Arc::new(MockAdapter::new());

        // Shard 0: events matching the filter
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "token"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "token"}), 200, 0),
            ],
        );

        // Shard 1: events that will be filtered out
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({"type": "message"}), 150, 1),
                StoredEvent::from_value("1-2".to_string(), json!({"type": "message"}), 250, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);
        let filter = Filter::eq("type", json!("token"));

        // First poll: should return only the "token" events from shard 0
        let response1 = merger
            .poll(ConsumeRequest::new(100).filter(filter.clone()))
            .await
            .unwrap();

        assert_eq!(response1.events.len(), 2, "Should get 2 token events");
        for event in &response1.events {
            assert_eq!(
                event.shard_id, 0,
                "All returned events should be from shard 0"
            );
        }

        let cursor1 = response1
            .next_id
            .expect("Should have a cursor after first poll");

        // Verify the cursor advanced for shard 1 even though its events were filtered out
        let decoded = CompositeCursor::decode(&cursor1).unwrap();
        assert!(
            decoded.get(1).is_some(),
            "Cursor must advance for shard 1 even though all its events were filtered out"
        );
        assert_eq!(
            decoded.get(1),
            Some("1-2"),
            "Shard 1 cursor should point to its last fetched event"
        );

        // Second poll with the cursor: should NOT re-fetch shard 1's events
        let response2 = merger
            .poll(ConsumeRequest::new(100).filter(filter).from(cursor1))
            .await
            .unwrap();

        assert!(
            response2.events.is_empty(),
            "Second poll should return no events (all events already consumed or filtered)"
        );
    }

    #[tokio::test]
    async fn test_regression_poll_merger_filter_does_not_infinite_loop() {
        // Regression: when one shard has events matching the filter and another
        // shard has events that are all filtered out, polling in pages must
        // terminate and return all matching events without looping forever.
        let adapter = Arc::new(MockAdapter::new());

        // Shard 0: 100 events all matching filter
        let shard0_events: Vec<_> = (1..=100)
            .map(|i| {
                StoredEvent::from_value(
                    format!("0-{}", i),
                    json!({"type": "token", "idx": i}),
                    i as u64 * 10,
                    0,
                )
            })
            .collect();
        adapter.add_events(0, shard0_events);

        // Shard 1: 100 events none matching filter
        let shard1_events: Vec<_> = (1..=100)
            .map(|i| {
                StoredEvent::from_value(
                    format!("1-{}", i),
                    json!({"type": "message", "idx": i}),
                    i as u64 * 10 + 5,
                    1,
                )
            })
            .collect();
        adapter.add_events(1, shard1_events);

        let merger = PollMerger::new(adapter, 2);
        let filter = Filter::eq("type", json!("token"));

        let mut all_events = Vec::new();
        let mut cursor: Option<String> = None;
        let max_iterations = 50;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > max_iterations {
                panic!(
                    "Infinite loop detected after {} iterations! Collected {} events so far.",
                    max_iterations,
                    all_events.len()
                );
            }

            let mut request = ConsumeRequest::new(50).filter(filter.clone());
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

        // Should have collected exactly 100 matching events from shard 0
        assert_eq!(
            all_events.len(),
            100,
            "Expected 100 matching events, got {}. Iterations: {}",
            all_events.len(),
            iterations
        );

        // All events should be from shard 0 (the "token" shard)
        for event in &all_events {
            assert_eq!(
                event.shard_id, 0,
                "All matching events should come from shard 0"
            );
        }

        // Verify no duplicates
        let mut ids: Vec<_> = all_events.iter().map(|e| e.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 100, "Should have no duplicate events");
    }

    #[tokio::test]
    async fn test_regression_all_events_filtered_returns_cursor() {
        // Regression: when every fetched event was filtered out, next_id was
        // None, leaving the caller stuck re-fetching the same events forever.
        let adapter = Arc::new(MockAdapter::new());

        // Only non-matching events
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "noise"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "noise"}), 200, 0),
            ],
        );

        let merger = PollMerger::new(adapter, 1);
        let filter = Filter::eq("type", json!("signal"));

        let response = merger
            .poll(ConsumeRequest::new(100).filter(filter))
            .await
            .unwrap();

        // No events match, but cursor must still advance
        assert!(response.events.is_empty());
        assert!(
            response.next_id.is_some(),
            "cursor must advance past filtered events even when none match"
        );
    }

    /// Exercises the non-lazy filter branch: when `Ordering::InsertionTs`
    /// is requested we can't short-circuit at `limit + 1` matches (the
    /// sort needs every event first), so the code falls through to
    /// `retain` → sort → truncate. This test pins:
    /// - Results are globally sorted by `insertion_ts` (not input order).
    /// - Only filter-matching events come through.
    /// - Truncation picks the `limit` *earliest* matches by ts.
    /// - `has_more` is set when matches exceed `limit`.
    #[tokio::test]
    async fn test_poll_merger_filter_insertion_ts_truncates_after_sort() {
        let adapter = Arc::new(MockAdapter::new());

        // Interleave shards with out-of-order timestamps and a mix of
        // matching / non-matching events. Matching timestamps: 120, 200,
        // 260, 400. Non-matching timestamps: 100, 300.
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "token"}), 400, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "noise"}), 100, 0),
                StoredEvent::from_value("0-3".to_string(), json!({"type": "token"}), 200, 0),
            ],
        );
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({"type": "token"}), 260, 1),
                StoredEvent::from_value("1-2".to_string(), json!({"type": "noise"}), 300, 1),
                StoredEvent::from_value("1-3".to_string(), json!({"type": "token"}), 120, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);
        let filter = Filter::eq("type", json!("token"));

        // 4 matches exist; asking for 2 must yield the two earliest
        // after a full sort (120, 200) and signal has_more.
        let response = merger
            .poll(
                ConsumeRequest::new(2)
                    .filter(filter)
                    .ordering(Ordering::InsertionTs),
            )
            .await
            .unwrap();

        assert_eq!(response.events.len(), 2);
        assert_eq!(
            response.events[0].insertion_ts, 120,
            "earliest match must come first"
        );
        assert_eq!(response.events[1].insertion_ts, 200);
        assert!(
            response.has_more,
            "two more matching events remain past the limit"
        );
    }

    #[tokio::test]
    async fn test_regression_corrupt_event_filter_drop_is_consistent_and_logged() {
        // Regression: corrupt events (raw bytes that don't deserialize as
        // JSON) used to be silently dropped from the filtered poll path
        // via `event.parse().map(...).unwrap_or(false)`, while the
        // unfiltered path returned them as-is. That inconsistency hid
        // upstream framing/storage corruption from anyone running with a
        // filter (i.e. most consumers).
        //
        // The fix routes parse failures through `event_matches_filter`,
        // which emits `tracing::warn!` per dropped event. We don't have
        // a tracing-test subscriber wired up so we don't assert on the
        // log line itself; instead we pin the behavioral surface so a
        // future regression that re-silences corruption (e.g., dropping
        // the helper) shows up in code review:
        //   - filtered poll: corrupt event is dropped, valid event kept
        //   - unfiltered poll: corrupt event flows through unchanged
        //
        // If the helper is removed or the warn! is downgraded to debug!,
        // this test still passes — but the helper's doc-comment names
        // the inconsistency and is the artifact that protects the
        // observability requirement.
        let adapter = Arc::new(MockAdapter::new());
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "ok"}), 100, 0),
                // Raw bytes that don't parse as JSON — a torn write or
                // upstream framing bug surface.
                StoredEvent::new(
                    "0-2".to_string(),
                    bytes::Bytes::from_static(b"\xff\xff not json \xff"),
                    200,
                    0,
                ),
            ],
        );

        let merger = PollMerger::new(adapter, 1);

        // Filtered: corrupt event must be dropped, valid event kept.
        let filtered = merger
            .poll(ConsumeRequest::new(100).filter(Filter::eq("type", json!("ok"))))
            .await
            .unwrap();
        assert_eq!(
            filtered.events.len(),
            1,
            "filtered poll must drop the corrupt event"
        );
        assert_eq!(filtered.events[0].id, "0-1");

        // Unfiltered: corrupt event flows through. Documenting that the
        // unfiltered path is the *only* way an operator currently sees
        // the corrupt bytes — without the warn! the filtered path is
        // a black hole.
        let unfiltered = merger.poll(ConsumeRequest::new(100)).await.unwrap();
        assert_eq!(
            unfiltered.events.len(),
            2,
            "unfiltered poll must surface the corrupt event verbatim"
        );
        let ids: Vec<_> = unfiltered.events.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"0-1"));
        assert!(ids.contains(&"0-2"));
    }

    /// Regression: BUG_REPORT.md #2 — `Ordering::None` filter previously
    /// broke out of the drain loop once `kept.len() >= limit + 1`,
    /// which silently discarded events from later shards without
    /// checking the filter. Combined with `new_cursor` advancing for
    /// every polled shard, that meant matching events on un-inspected
    /// shards were lost forever.
    ///
    /// Setup: shard 0 has matches followed by shard 1 with matches.
    /// With `limit=2`, shard 0's first three events (two matches plus
    /// one extra to trigger has_more) used to satisfy the early break,
    /// silently dropping shard 1's matches AND advancing past them.
    /// The fix runs a full `retain` pass over every fetched event,
    /// then rolls back the cursor for shards whose matches were
    /// truncated so they're re-fetched on the next poll.
    #[tokio::test]
    async fn test_regression_ordering_none_filter_does_not_strand_later_shards() {
        let adapter = Arc::new(MockAdapter::new());

        // Shard 0: 3 matching events.
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "token"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "token"}), 110, 0),
                StoredEvent::from_value("0-3".to_string(), json!({"type": "token"}), 120, 0),
            ],
        );
        // Shard 1: 3 matching events.
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({"type": "token"}), 200, 1),
                StoredEvent::from_value("1-2".to_string(), json!({"type": "token"}), 210, 1),
                StoredEvent::from_value("1-3".to_string(), json!({"type": "token"}), 220, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);
        let filter = Filter::eq("type", json!("token"));

        // Page through with a small limit — over many polls every
        // matching event must surface exactly once. Bound iterations
        // to detect either a stall or an explosion.
        let mut all_returned: Vec<String> = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..20 {
            let mut req = ConsumeRequest::new(2).filter(filter.clone());
            if let Some(c) = &cursor {
                req = req.from(c.clone());
            }
            let resp = merger.poll(req).await.unwrap();
            for e in &resp.events {
                all_returned.push(e.id.clone());
            }
            if !resp.has_more {
                break;
            }
            cursor = resp.next_id;
        }

        all_returned.sort();
        all_returned.dedup();
        assert_eq!(
            all_returned,
            vec!["0-1", "0-2", "0-3", "1-1", "1-2", "1-3"],
            "every matching event from every shard must be returned exactly once"
        );
    }

    /// Regression: BUG_REPORT.md #23 — `Ordering::InsertionTs` filter
    /// previously stranded matches on shards whose matching events
    /// all sorted later than `limit` matches from other shards. The
    /// global sort+truncate dropped them, the cursor-override only
    /// fired for shards present in the *returned* set, and so the
    /// cursor for the un-returned shard advanced to its fetched
    /// position via `new_cursor` — silently skipping the matches.
    ///
    /// Setup: shard 0 has 3 early-ts matches and shard 1 has 3
    /// late-ts matches. With `limit=2` and `InsertionTs` ordering,
    /// the first poll returns the two earliest from shard 0;
    /// shard 1's matches must NOT be lost. The fix detects that
    /// shard 1 had matches truncated and rolls its cursor back so
    /// they're re-fetched on the next poll.
    #[tokio::test]
    async fn test_regression_insertion_ts_filter_does_not_strand_late_shard() {
        let adapter = Arc::new(MockAdapter::new());

        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-1".to_string(), json!({"type": "token"}), 100, 0),
                StoredEvent::from_value("0-2".to_string(), json!({"type": "token"}), 110, 0),
                StoredEvent::from_value("0-3".to_string(), json!({"type": "token"}), 120, 0),
            ],
        );
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-1".to_string(), json!({"type": "token"}), 1000, 1),
                StoredEvent::from_value("1-2".to_string(), json!({"type": "token"}), 1010, 1),
                StoredEvent::from_value("1-3".to_string(), json!({"type": "token"}), 1020, 1),
            ],
        );

        let merger = PollMerger::new(adapter, 2);
        let filter = Filter::eq("type", json!("token"));

        let mut all_returned: Vec<String> = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..20 {
            let mut req = ConsumeRequest::new(2)
                .filter(filter.clone())
                .ordering(Ordering::InsertionTs);
            if let Some(c) = &cursor {
                req = req.from(c.clone());
            }
            let resp = merger.poll(req).await.unwrap();
            for e in &resp.events {
                all_returned.push(e.id.clone());
            }
            if !resp.has_more {
                break;
            }
            cursor = resp.next_id;
        }

        all_returned.sort();
        all_returned.dedup();
        assert_eq!(
            all_returned,
            vec!["0-1", "0-2", "0-3", "1-1", "1-2", "1-3"],
            "matches from the late-ts shard must not be lost to truncation"
        );
    }

    /// Regression: BUG_REPORT.md #50 — if any adapter returns
    /// `has_more: true` with no events and no `next_id`, the merger
    /// previously forwarded that as `(has_more=true, next_id=None)`,
    /// causing the caller to re-poll from the same starting cursor
    /// indefinitely. The fix suppresses `has_more` whenever the
    /// merger itself made no observable progress (no events AND no
    /// cursor advance).
    #[tokio::test]
    async fn has_more_is_suppressed_when_no_progress() {
        struct LiarAdapter;

        #[async_trait]
        impl Adapter for LiarAdapter {
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
                _shard_id: u16,
                _from_id: Option<&str>,
                _limit: usize,
            ) -> Result<ShardPollResult, AdapterError> {
                // The pathological case: claim has_more without
                // returning events or advancing the cursor.
                Ok(ShardPollResult {
                    events: Vec::new(),
                    next_id: None,
                    has_more: true,
                })
            }
            fn name(&self) -> &'static str {
                "liar"
            }
        }

        let adapter: Arc<dyn Adapter> = Arc::new(LiarAdapter);
        let merger = PollMerger::new(adapter, 2);
        let response = merger.poll(ConsumeRequest::new(100)).await.unwrap();

        assert!(
            response.events.is_empty(),
            "no events were emitted, but merger returned {}",
            response.events.len()
        );
        // The whole point of this fix: don't let a misbehaving
        // adapter trick the caller into an infinite re-poll.
        assert!(
            !response.has_more,
            "has_more must be suppressed when merger made no progress (#50)"
        );
        assert!(
            response.next_id.is_none(),
            "next_id must remain None when no progress was made (#50)"
        );
    }

    /// Regression: BUG_REPORT.md #52 — `sort_by_key(|e| e.insertion_ts)`
    /// is stable but ties across shards depend on `join_all`'s
    /// completion order, which is non-deterministic. Combined with
    /// `truncate(limit)`, this could drop or duplicate events at the
    /// limit boundary across consecutive polls. The fix breaks ties
    /// deterministically on `(shard_id, id)`.
    #[tokio::test]
    async fn sort_breaks_ties_deterministically_across_shards() {
        // Two shards with events that share `insertion_ts` so the
        // tiebreaker controls the order.
        let adapter = Arc::new(MockAdapter::new());
        adapter.add_events(
            0,
            vec![
                StoredEvent::from_value("0-a".to_string(), json!({}), 100, 0),
                StoredEvent::from_value("0-b".to_string(), json!({}), 100, 0),
            ],
        );
        adapter.add_events(
            1,
            vec![
                StoredEvent::from_value("1-a".to_string(), json!({}), 100, 1),
                StoredEvent::from_value("1-b".to_string(), json!({}), 100, 1),
            ],
        );

        // Poll many times; the order must be stable.
        let merger = PollMerger::new(adapter, 2);
        let mut prior_order: Option<Vec<String>> = None;
        for iter in 0..20 {
            let r = merger
                .poll(
                    ConsumeRequest::new(10).ordering(Ordering::InsertionTs),
                )
                .await
                .unwrap();
            let ids: Vec<String> = r.events.iter().map(|e| e.id.clone()).collect();
            if let Some(prev) = &prior_order {
                assert_eq!(
                    &ids, prev,
                    "iter {iter}: order is non-deterministic — sort tie-break failed (#52)"
                );
            }
            prior_order = Some(ids);
        }

        // And the order must match `(shard_id, id)`.
        let r = merger
            .poll(ConsumeRequest::new(10).ordering(Ordering::InsertionTs))
            .await
            .unwrap();
        let ids: Vec<String> = r.events.iter().map(|e| e.id.clone()).collect();
        assert_eq!(ids, vec!["0-a", "0-b", "1-a", "1-b"]);
    }
}
