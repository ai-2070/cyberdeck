//! The main SDK handle.

use std::sync::Arc;

use bytes::Bytes;
use serde::Serialize;

use net::event::RawEvent;
use net::{ConsumeRequest, Event, EventBus, StoredEvent};

use crate::config::NetBuilder;
use crate::error::Result;
use crate::stream::{EventStream, SubscribeOpts, TypedEventStream};

/// Receipt from a successful ingestion.
#[derive(Debug, Clone, Copy)]
pub struct Receipt {
    /// The shard the event was assigned to.
    pub shard_id: u16,
    /// Insertion timestamp (nanoseconds).
    pub timestamp: u64,
}

/// Ingestion statistics.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    /// Total events ingested.
    pub events_ingested: u64,
    /// Events dropped due to backpressure.
    pub events_dropped: u64,
    /// Batches dispatched to the adapter.
    pub batches_dispatched: u64,
}

/// Options for a one-shot poll request.
#[derive(Debug, Clone, Default)]
pub struct PollRequest {
    /// Maximum number of events to return.
    pub limit: usize,
    /// Cursor to resume from.
    pub cursor: Option<String>,
    /// Optional event filter.
    pub filter: Option<net::Filter>,
    /// Event ordering.
    pub ordering: Option<net::consumer::Ordering>,
    /// Specific shards to poll.
    pub shards: Option<Vec<u16>>,
}

/// Response from a poll request.
#[derive(Debug, Clone)]
pub struct PollResponse {
    /// Events returned.
    pub events: Vec<StoredEvent>,
    /// Cursor for the next poll.
    pub next_id: Option<String>,
    /// Whether more events are available.
    pub has_more: bool,
}

/// A node on the Net mesh.
///
/// This is the main SDK handle. Every computer, device, and application
/// is a `Net` node — there are no clients, no servers, no coordinators.
///
/// # Example
///
/// ```rust,no_run
/// use net_sdk::{Net, Backpressure};
///
/// # async fn example() -> net_sdk::error::Result<()> {
/// let node = Net::builder()
///     .shards(4)
///     .backpressure(Backpressure::DropOldest)
///     .memory()
///     .build()
///     .await?;
///
/// node.emit(&serde_json::json!({"token": "hello"}))?;
///
/// let response = node.poll(net_sdk::PollRequest { limit: 100, ..Default::default() }).await?;
/// for event in &response.events {
///     println!("{}", event.raw_str().unwrap_or("<non-utf8>"));
/// }
///
/// node.shutdown().await?;
/// # Ok(())
/// # }
/// ```
pub struct Net {
    bus: Arc<EventBus>,
}

impl Net {
    /// Create a builder for configuring a new node.
    pub fn builder() -> NetBuilder {
        NetBuilder::new()
    }

    /// Create a node from an existing `EventBus`.
    pub fn from_bus(bus: EventBus) -> Self {
        Self { bus: Arc::new(bus) }
    }

    /// Build and start a new node from a builder.
    pub(crate) async fn from_builder(builder: NetBuilder) -> Result<Self> {
        let config = builder.build_config()?;
        let bus = EventBus::new(config).await?;
        Ok(Self { bus: Arc::new(bus) })
    }

    // ---- Ingestion ----

    /// Emit a serializable event.
    ///
    /// Serializes `T` to JSON via serde and ingests it.
    pub fn emit<T: Serialize>(&self, event: &T) -> Result<Receipt> {
        let value = serde_json::to_value(event)?;
        let e = Event::new(value);
        let (shard_id, timestamp) = self.bus.ingest(e)?;
        Ok(Receipt {
            shard_id,
            timestamp,
        })
    }

    /// Emit raw bytes (fastest path).
    pub fn emit_raw(&self, bytes: impl Into<Bytes>) -> Result<Receipt> {
        let raw = RawEvent::from_bytes(bytes);
        let (shard_id, timestamp) = self.bus.ingest_raw(raw)?;
        Ok(Receipt {
            shard_id,
            timestamp,
        })
    }

    /// Emit a raw JSON string.
    pub fn emit_str(&self, json: &str) -> Result<Receipt> {
        let raw = RawEvent::from_str(json);
        let (shard_id, timestamp) = self.bus.ingest_raw(raw)?;
        Ok(Receipt {
            shard_id,
            timestamp,
        })
    }

    /// Emit a batch of serializable events. Returns the number ingested.
    pub fn emit_batch<T: Serialize>(&self, events: &[T]) -> Result<usize> {
        let mut raw_events = Vec::with_capacity(events.len());
        for event in events {
            let value = serde_json::to_value(event)?;
            raw_events.push(RawEvent::from_value(value));
        }
        Ok(self.bus.ingest_raw_batch(raw_events))
    }

    /// Emit a batch of raw byte events. Returns the number ingested.
    pub fn emit_raw_batch(&self, events: Vec<Bytes>) -> usize {
        let raw_events: Vec<RawEvent> = events.into_iter().map(RawEvent::from_bytes).collect();
        self.bus.ingest_raw_batch(raw_events)
    }

    // ---- Consumption ----

    /// One-shot poll for events.
    pub async fn poll(&self, request: PollRequest) -> Result<PollResponse> {
        let mut req = ConsumeRequest::new(request.limit);
        if let Some(cursor) = request.cursor {
            req = req.from(cursor);
        }
        if let Some(filter) = request.filter {
            req = req.filter(filter);
        }
        if let Some(ordering) = request.ordering {
            req = req.ordering(ordering);
        }
        if let Some(shards) = request.shards {
            req = req.shards(shards);
        }
        let response = self.bus.poll(req).await?;
        Ok(PollResponse {
            events: response.events,
            next_id: response.next_id,
            has_more: response.has_more,
        })
    }

    /// Subscribe to a stream of events.
    ///
    /// Returns an async `Stream` that yields events with adaptive backoff.
    pub fn subscribe(&self, opts: SubscribeOpts) -> EventStream {
        EventStream::new(self.bus.clone(), opts)
    }

    /// Subscribe to a typed stream of events.
    ///
    /// Each event is automatically deserialized into `T`.
    pub fn subscribe_typed<T: serde::de::DeserializeOwned>(
        &self,
        opts: SubscribeOpts,
    ) -> TypedEventStream<T> {
        TypedEventStream::new(self.bus.clone(), opts)
    }

    // ---- Lifecycle ----

    /// Get ingestion statistics.
    pub fn stats(&self) -> Stats {
        let s = self.bus.stats();
        Stats {
            events_ingested: s.events_ingested.load(std::sync::atomic::Ordering::Relaxed),
            events_dropped: s.events_dropped.load(std::sync::atomic::Ordering::Relaxed),
            batches_dispatched: s
                .batches_dispatched
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Get the number of active shards.
    pub fn shards(&self) -> u16 {
        self.bus.num_shards()
    }

    /// Check if the node is healthy.
    pub async fn health(&self) -> bool {
        self.bus.is_healthy().await
    }

    /// Flush all pending batches to the adapter.
    pub async fn flush(&self) -> Result<()> {
        self.bus.flush().await?;
        Ok(())
    }

    /// Gracefully shut down the node.
    ///
    /// Drains pending events through the adapter and signals all
    /// background tasks to exit. Safe to call even when other
    /// `Arc<EventBus>` clones exist (e.g. an outstanding
    /// `EventStream` from `subscribe`): `EventBus::shutdown_via_ref`
    /// is idempotent and reference-based, so the shutdown work runs
    /// regardless of strong-ref count, and outstanding clones
    /// observe the bus as shut down on their next operation.
    pub async fn shutdown(self) -> Result<()> {
        self.bus.shutdown_via_ref().await?;
        Ok(())
    }

    /// Get a reference to the underlying `EventBus`.
    pub fn bus(&self) -> &EventBus {
        &self.bus
    }
}
