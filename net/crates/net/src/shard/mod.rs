//! Shard management for parallel event ingestion.
//!
//! The shard module provides:
//! - Lock-free ring buffers for high-throughput event queuing
//! - Per-shard timestamp generation (no cross-shard contention)
//! - Batch assembly with adaptive sizing
//! - Shard manager for coordinating multiple shards
//! - Dynamic shard scaling with weighted producer routing

mod batch;
mod mapper;
mod ring_buffer;

pub use batch::{AdaptiveBatcher, BatchWorker};
pub use mapper::{
    ScalingDecision, ScalingError, ShardMapper, ShardMetrics, ShardMetricsCollector, ShardState,
};
pub use ring_buffer::{BufferFullError, RingBuffer};

// Re-export ScalingPolicy from config for convenience
pub use crate::config::ScalingPolicy;

use bytes::Bytes;

use crate::config::BackpressureMode;
use crate::error::IngestionError;
use crate::event::{InternalEvent, RawEvent};
use crate::timestamp::TimestampGenerator;

use serde_json::Value as JsonValue;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;

/// Identity hasher specialized for `u16` shard ids.
///
/// The default `HashMap` hasher is SipHash, which is HashDoS-resistant
/// — irrelevant here because shard ids are assigned by the bus itself,
/// never derived from user input. For a 1-2 byte key, SipHash adds
/// ~10 ns of setup per probe; an identity-style hasher (just promote
/// the u16 to a u64) is 2-3 ns. With thousands of `resolve_idx` calls
/// per second on the dynamic-scaling ingest path, the difference adds
/// up.
#[derive(Default, Clone, Copy)]
struct U16IdHasher(u64);

impl Hasher for U16IdHasher {
    #[inline]
    fn write_u16(&mut self, n: u16) {
        // Spread the 16-bit key across the 64-bit hash space so the
        // low-order bits the HashMap uses for bucket selection are
        // not always zero. Multiply by a large odd constant — same
        // trick FxHash uses for u64s.
        self.0 = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // The HashMap only ever calls `write_u16` for `u16` keys, but
        // the `Hasher` trait requires `write(&[u8])` to be present.
        // Fall back to a byte-wise mix; not on the hot path.
        for &b in bytes {
            self.0 = self.0.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// `HashMap` keyed by `u16` shard id, using the identity-style hasher
/// above. Construct with `ShardIdMap::default()`.
type ShardIdMap<V> = std::collections::HashMap<u16, V, BuildHasherDefault<U16IdHasher>>;

/// Atomic counters for a single shard. Kept outside `Shard` as `Arc`s
/// so `ShardManager::stats()` can aggregate them without locking each
/// shard's mutex.
#[derive(Debug, Default)]
pub struct ShardCounters {
    /// Total events ingested into this shard.
    pub events_ingested: AtomicU64,
    /// Events dropped due to backpressure.
    pub events_dropped: AtomicU64,
    /// Batches successfully dispatched to the adapter.
    pub batches_dispatched: AtomicU64,
}

/// Statistics for a single shard (snapshot).
#[derive(Debug, Default, Clone, Copy)]
pub struct ShardStats {
    /// Total events ingested.
    pub events_ingested: u64,
    /// Events dropped due to backpressure.
    pub events_dropped: u64,
    /// Batches dispatched to adapter.
    pub batches_dispatched: u64,
}

impl ShardCounters {
    /// Load a consistent snapshot of the counters.
    #[inline]
    pub fn snapshot(&self) -> ShardStats {
        ShardStats {
            events_ingested: self.events_ingested.load(AtomicOrdering::Relaxed),
            events_dropped: self.events_dropped.load(AtomicOrdering::Relaxed),
            batches_dispatched: self.batches_dispatched.load(AtomicOrdering::Relaxed),
        }
    }
}

/// A single shard with its own ring buffer and timestamp generator.
pub struct Shard {
    /// Shard identifier.
    pub id: u16,
    /// Ring buffer for event queuing.
    ring_buffer: RingBuffer<InternalEvent>,
    /// Shard-local timestamp generator (no contention).
    timestamp_gen: TimestampGenerator,
    /// Shared atomic counters (also referenced from `ShardTable` for
    /// lock-free aggregation).
    counters: Arc<ShardCounters>,
    /// Optional metrics collector for dynamic scaling.
    metrics_collector: Option<Arc<ShardMetricsCollector>>,
    /// Wakes the drain worker when an event lands. Producers call
    /// `notify_one()` after a successful push; the drain worker
    /// `await`s `notified()` instead of polling on a fixed timer.
    /// Stored as an `Arc` so the producer can grab a clone, drop the
    /// shard mutex, and notify outside the critical section.
    notify: Arc<tokio::sync::Notify>,
    /// Ring buffer capacity (for metrics).
    capacity: usize,
}

impl Shard {
    /// Create a new shard.
    pub fn new(id: u16, capacity: usize) -> Self {
        Self {
            id,
            ring_buffer: RingBuffer::new(capacity),
            timestamp_gen: TimestampGenerator::new(),
            counters: Arc::new(ShardCounters::default()),
            metrics_collector: None,
            notify: Arc::new(tokio::sync::Notify::new()),
            capacity,
        }
    }

    /// Create a new shard with a metrics collector for dynamic scaling.
    pub fn with_metrics(id: u16, capacity: usize, metrics: Arc<ShardMetricsCollector>) -> Self {
        Self {
            id,
            ring_buffer: RingBuffer::new(capacity),
            timestamp_gen: TimestampGenerator::new(),
            counters: Arc::new(ShardCounters::default()),
            metrics_collector: Some(metrics),
            notify: Arc::new(tokio::sync::Notify::new()),
            capacity,
        }
    }

    /// Clone the atomic counter handle (for lock-free aggregation).
    #[inline]
    pub fn counters(&self) -> Arc<ShardCounters> {
        self.counters.clone()
    }

    /// Clone the wake handle. Producers call `notify_one()` after a
    /// successful push so the drain worker can `await` events instead
    /// of polling.
    #[inline]
    pub fn notify_handle(&self) -> Arc<tokio::sync::Notify> {
        self.notify.clone()
    }

    /// Set the metrics collector.
    pub fn set_metrics_collector(&mut self, metrics: Arc<ShardMetricsCollector>) {
        self.metrics_collector = Some(metrics);
    }

    /// Try to push a raw event (pre-serialized bytes) into the shard's ring buffer.
    /// Returns the assigned insertion timestamp on success.
    ///
    /// This is the fastest ingestion path - no serialization or hashing needed.
    #[inline]
    pub fn try_push_raw(&mut self, raw: Bytes) -> Result<u64, IngestionError> {
        let ts = self.timestamp_gen.next();
        let event = InternalEvent::new(raw, ts, self.id);

        match self.ring_buffer.try_push(event) {
            Ok(()) => {
                self.counters
                    .events_ingested
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Ok(ts)
            }
            Err(_) => {
                self.counters
                    .events_dropped
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Err(IngestionError::Backpressure)
            }
        }
    }

    /// Try to push a JSON value into the shard's ring buffer.
    /// Returns the assigned insertion timestamp on success.
    ///
    /// This serializes the value once before storing.
    #[inline]
    pub fn try_push(&mut self, raw: JsonValue) -> Result<u64, IngestionError> {
        let ts = self.timestamp_gen.next();
        let event = InternalEvent::from_value(raw, ts, self.id);

        match self.ring_buffer.try_push(event) {
            Ok(()) => {
                self.counters
                    .events_ingested
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Ok(ts)
            }
            Err(_) => {
                self.counters
                    .events_dropped
                    .fetch_add(1, AtomicOrdering::Relaxed);
                Err(IngestionError::Backpressure)
            }
        }
    }

    /// Pop a batch of events from the ring buffer.
    ///
    /// Allocates a fresh `Vec`. Prefer [`pop_batch_into`] in the drain
    /// loop where reusing a scratch buffer eliminates one allocation
    /// per cycle.
    ///
    /// [`pop_batch_into`]: Self::pop_batch_into
    #[inline]
    pub fn pop_batch(&mut self, max: usize) -> Vec<InternalEvent> {
        self.ring_buffer.pop_batch(max)
    }

    /// Pop a batch of events into a caller-owned buffer.
    ///
    /// Returns the number of events drained. Appends to `dst` (does not
    /// clear it first), so the drain worker can hold a single scratch
    /// `Vec`, drain into it, hand the contents off, and reuse the
    /// allocation on the next cycle.
    #[inline]
    pub fn pop_batch_into(&mut self, dst: &mut Vec<InternalEvent>, max: usize) -> usize {
        self.ring_buffer.pop_batch_into(dst, max)
    }

    /// Try to pop a single event from the ring buffer.
    #[inline]
    pub fn try_pop(&mut self) -> Option<InternalEvent> {
        self.ring_buffer.try_pop()
    }

    /// Get the current buffer length.
    #[inline]
    pub fn len(&self) -> usize {
        self.ring_buffer.len()
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ring_buffer.is_empty()
    }

    /// Check if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.ring_buffer.is_full()
    }

    /// Get the fill ratio (0.0 - 1.0).
    #[inline]
    pub fn fill_ratio(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.ring_buffer.len() as f64 / self.capacity as f64
        }
    }

    /// Get the ring buffer capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get a snapshot of shard statistics.
    pub fn stats(&self) -> ShardStats {
        self.counters.snapshot()
    }

    /// Record a batch dispatch.
    pub fn record_batch_dispatch(&self) {
        self.counters
            .batches_dispatched
            .fetch_add(1, AtomicOrdering::Relaxed);
    }
}

/// Immutable routing table: shards + index + counter handles.
///
/// Placed behind an `ArcSwap` on `ShardManager` so the common read
/// path (`ingest`, `ingest_raw`, `with_shard`, `stats`) is
/// lock-free. Rebuilt on scale up/down via RCU-style swap.
pub struct ShardTable {
    /// All shards, indexed by position. `Arc<Mutex<Shard>>` lets a new
    /// table share shard handles with the previous table (cheap Arc
    /// clones during rebuild).
    shards: Vec<Arc<parking_lot::Mutex<Shard>>>,
    /// Parallel vector of counter handles. Exposes stats without
    /// locking the shard mutex.
    counters: Vec<Arc<ShardCounters>>,
    /// Map from shard ID to index in `shards`/`counters`.
    ///
    /// Uses an identity-style hasher (see [`U16IdHasher`]) — shard ids
    /// are bus-assigned u16s, not user input, so the HashDoS-resistant
    /// default SipHash is wasted overhead on the dynamic-scaling
    /// ingest path that hits this map per event.
    shard_index: ShardIdMap<usize>,
}

impl ShardTable {
    fn new(shards: Vec<Shard>) -> Self {
        let mut shard_index: ShardIdMap<usize> =
            ShardIdMap::with_capacity_and_hasher(shards.len(), Default::default());
        let mut counters = Vec::with_capacity(shards.len());
        let shards: Vec<_> = shards
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                shard_index.insert(s.id, idx);
                counters.push(s.counters());
                Arc::new(parking_lot::Mutex::new(s))
            })
            .collect();
        Self {
            shards,
            counters,
            shard_index,
        }
    }
}

/// Manager for multiple shards.
///
/// The ShardManager can operate in two modes:
/// 1. Static mode (default): Fixed number of shards, simple hash-based routing
/// 2. Dynamic mode: Shards can be added/removed based on load, weighted routing
pub struct ShardManager {
    /// Routing table. Swapped atomically on scale up/down so readers
    /// never see a partially-updated `(shards, shard_index)` pair.
    table: arc_swap::ArcSwap<ShardTable>,
    /// Current number of active shards.
    num_shards: std::sync::atomic::AtomicU16,
    /// Backpressure mode.
    backpressure_mode: BackpressureMode,
    /// Ring buffer capacity for new shards.
    ring_buffer_capacity: usize,
    /// Optional shard mapper for dynamic scaling.
    mapper: Option<Arc<ShardMapper>>,
    /// Serializes concurrent `add_shard` / `remove_shard` rebuilds.
    /// Not on the ingest path.
    rebuild_lock: parking_lot::Mutex<()>,
}

impl ShardManager {
    /// Create a new shard manager (static mode).
    pub fn new(
        num_shards: u16,
        ring_buffer_capacity: usize,
        backpressure_mode: BackpressureMode,
    ) -> Self {
        let shards: Vec<Shard> = (0..num_shards)
            .map(|id| Shard::new(id, ring_buffer_capacity))
            .collect();

        Self {
            table: arc_swap::ArcSwap::from_pointee(ShardTable::new(shards)),
            num_shards: std::sync::atomic::AtomicU16::new(num_shards),
            backpressure_mode,
            ring_buffer_capacity,
            mapper: None,
            rebuild_lock: parking_lot::Mutex::new(()),
        }
    }

    /// Create a new shard manager with dynamic scaling enabled.
    pub fn with_mapper(
        num_shards: u16,
        ring_buffer_capacity: usize,
        backpressure_mode: BackpressureMode,
        policy: ScalingPolicy,
    ) -> Result<Self, ScalingError> {
        let mapper = Arc::new(ShardMapper::new(num_shards, ring_buffer_capacity, policy)?);

        let shards: Vec<Shard> = (0..num_shards)
            .map(|id| {
                let metrics = mapper.metrics_collector(id).ok_or_else(|| {
                    ScalingError::InvalidPolicy(format!("no metrics collector for shard {}", id))
                })?;
                Ok(Shard::with_metrics(id, ring_buffer_capacity, metrics))
            })
            .collect::<Result<Vec<_>, ScalingError>>()?;

        Ok(Self {
            table: arc_swap::ArcSwap::from_pointee(ShardTable::new(shards)),
            num_shards: std::sync::atomic::AtomicU16::new(num_shards),
            backpressure_mode,
            ring_buffer_capacity,
            mapper: Some(mapper),
            rebuild_lock: parking_lot::Mutex::new(()),
        })
    }

    /// Get the shard mapper (if dynamic scaling is enabled).
    pub fn mapper(&self) -> Option<&Arc<ShardMapper>> {
        self.mapper.as_ref()
    }

    /// Get the number of active shards.
    #[inline]
    pub fn num_shards(&self) -> u16 {
        self.num_shards.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get the backpressure mode.
    #[inline]
    pub fn backpressure_mode(&self) -> BackpressureMode {
        self.backpressure_mode
    }

    /// Select a shard for an event based on its content hash.
    /// Uses weighted selection if dynamic scaling is enabled.
    ///
    /// **Prefer [`select_shard_by_hash`].** This method serializes the
    /// `JsonValue` to bytes just to compute the hash; if you already
    /// have a `RawEvent` (or any pre-computed `xxh3_64` of the
    /// canonical bytes), pass that hash directly. The internal
    /// ingest paths all do — this method exists for ad-hoc external
    /// callers that haven't yet adopted the `RawEvent` pattern.
    ///
    /// [`select_shard_by_hash`]: Self::select_shard_by_hash
    #[inline]
    #[deprecated(
        since = "0.5.1",
        note = "serializes the value just to hash it; prefer `RawEvent::from_value(v).hash()` + `select_shard_by_hash` to avoid the duplicate serialization"
    )]
    pub fn select_shard(&self, event: &JsonValue) -> u16 {
        // Use xxhash for fast, deterministic hashing. `to_vec` avoids the
        // extra UTF-8 validation that `to_string` performs on the serialized
        // buffer, since we only need the bytes for hashing.
        let bytes = serde_json::to_vec(event).expect("Value serialization is infallible");
        let hash = xxhash_rust::xxh3::xxh3_64(&bytes);
        self.select_shard_by_hash(hash)
    }

    /// Select a shard using a pre-computed hash.
    ///
    /// This is faster than `select_shard` when you already have the hash.
    #[inline]
    pub fn select_shard_by_hash(&self, hash: u64) -> u16 {
        if let Some(ref mapper) = self.mapper {
            // Dynamic mode: use weighted selection
            mapper.select_shard(hash)
        } else {
            // Static mode: simple modulo
            let num_shards = self.num_shards.load(std::sync::atomic::Ordering::Acquire);
            (hash % num_shards as u64) as u16
        }
    }

    /// Resolve a shard ID to its table index, using the fast path in
    /// static mode (shard_id == index).
    #[inline]
    fn resolve_idx(&self, table: &ShardTable, shard_id: u16) -> Option<usize> {
        if self.mapper.is_none() {
            Some(shard_id as usize)
        } else {
            table.shard_index.get(&shard_id).copied()
        }
    }

    /// Push `raw` into `shard`, handling backpressure. Only clones the
    /// bytes when `DropOldest` needs them for the retry path.
    #[inline]
    fn push_with_backpressure(
        &self,
        shard: &mut Shard,
        shard_id: u16,
        raw: Bytes,
    ) -> Result<(u16, u64), IngestionError> {
        match self.backpressure_mode {
            BackpressureMode::DropOldest => match shard.try_push_raw(raw.clone()) {
                Ok(ts) => Ok((shard_id, ts)),
                Err(IngestionError::Backpressure) => {
                    // The failed try_push_raw incremented events_dropped for
                    // the *new* event, but the new event isn't actually
                    // dropped — the oldest is. Correct the stats: undo the
                    // spurious drop count, pop the oldest (which is the real
                    // drop), and retry with the same ref-counted bytes.
                    shard
                        .counters
                        .events_dropped
                        .fetch_sub(1, AtomicOrdering::Relaxed);
                    let _ = shard.try_pop();
                    shard
                        .counters
                        .events_dropped
                        .fetch_add(1, AtomicOrdering::Relaxed);
                    shard.try_push_raw(raw).map(|ts| (shard_id, ts))
                }
                Err(e) => Err(e),
            },
            BackpressureMode::Sample { .. } => match shard.try_push_raw(raw) {
                Ok(ts) => Ok((shard_id, ts)),
                Err(IngestionError::Backpressure) => Err(IngestionError::Sampled),
                Err(e) => Err(e),
            },
            BackpressureMode::DropNewest | BackpressureMode::FailProducer => {
                shard.try_push_raw(raw).map(|ts| (shard_id, ts))
            }
        }
    }

    /// Ingest an event into the appropriate shard.
    pub fn ingest(&self, event: JsonValue) -> Result<(u16, u64), IngestionError> {
        // Serialize once upfront - avoids clone on retry
        let raw = Bytes::from(
            serde_json::to_vec(&event).map_err(|e| IngestionError::Serialization(e.to_string()))?,
        );
        let hash = xxhash_rust::xxh3::xxh3_64(&raw);
        let shard_id = self.select_shard_by_hash(hash);

        let table = self.table.load();
        let idx = self
            .resolve_idx(&table, shard_id)
            .ok_or(IngestionError::Backpressure)?;
        let shard_lock = table.shards.get(idx).ok_or(IngestionError::Backpressure)?;

        // Push under the lock; grab the notify handle while still
        // holding it so we can wake the drain worker after release.
        // Calling `notify_one()` outside the lock keeps the critical
        // section minimal — the drain worker, which the wake hands
        // off to, doesn't fight the producer for the same mutex.
        let (result, notify) = {
            let mut shard = shard_lock.lock();
            let r = self.push_with_backpressure(&mut shard, shard_id, raw);
            let n = r.is_ok().then(|| shard.notify_handle());
            (r, n)
        };
        if let Some(n) = notify {
            n.notify_one();
        }
        result
    }

    /// Ingest a raw event (pre-serialized with cached hash).
    ///
    /// This is the fastest ingestion path:
    /// - Uses pre-computed hash for shard selection (no serialization)
    /// - Stores bytes directly (no clone needed, reference-counted)
    #[inline]
    pub fn ingest_raw(&self, event: RawEvent) -> Result<(u16, u64), IngestionError> {
        let shard_id = self.select_shard_by_hash(event.hash());

        let table = self.table.load();
        let idx = self
            .resolve_idx(&table, shard_id)
            .ok_or(IngestionError::Backpressure)?;
        let shard_lock = table.shards.get(idx).ok_or(IngestionError::Backpressure)?;

        let (result, notify) = {
            let mut shard = shard_lock.lock();
            let r = self.push_with_backpressure(&mut shard, shard_id, event.bytes());
            let n = r.is_ok().then(|| shard.notify_handle());
            (r, n)
        };
        if let Some(n) = notify {
            n.notify_one();
        }
        result
    }

    /// Ingest a batch of pre-serialized events, grouped by shard.
    ///
    /// Each destination shard's mutex is acquired once and all of that
    /// shard's events are pushed before releasing. With a uniform hash
    /// distribution this amortizes lock acquisitions from O(events) to
    /// O(shards). Backpressure semantics match per-event `ingest_raw`.
    ///
    /// Accepts any `IntoIterator<Item = RawEvent>` so callers (notably
    /// `EventBus::ingest_batch`, which converts `Event` → `RawEvent`)
    /// can avoid materializing an intermediate `Vec`.
    ///
    /// Returns `(success, total)` — number of successfully ingested
    /// events, and the total number of events the caller produced.
    /// The difference is what backpressure dropped.
    pub fn ingest_raw_batch(
        &self,
        events: impl IntoIterator<Item = RawEvent>,
    ) -> (usize, usize) {
        let table = self.table.load();
        let nshards = table.shards.len();

        if nshards == 0 {
            // No shards to route into — count the input but drop everything.
            let total = events.into_iter().count();
            return (0, total);
        }

        // In static mode, `select_shard_by_hash` already produces a value
        // equal to the table index (`shard_id == idx`). Hoist the mode
        // check out of the per-event loop so the dynamic-mode HashMap
        // lookup is paid only when actually needed.
        let static_mode = self.mapper.is_none();

        // Lazy buckets: only allocate a `Vec<Bytes>` for shards that
        // actually receive events in this batch. The outer Vec is one
        // contiguous allocation; previously we eagerly allocated `nshards`
        // separate empty inner Vecs every call.
        let mut groups: Vec<Option<Vec<Bytes>>> =
            std::iter::repeat_with(|| None).take(nshards).collect();

        let mut total = 0usize;
        for event in events {
            total += 1;
            let shard_id = self.select_shard_by_hash(event.hash());
            let idx = if static_mode {
                shard_id as usize
            } else {
                match table.shard_index.get(&shard_id).copied() {
                    Some(i) => i,
                    None => continue,
                }
            };
            if idx >= nshards {
                continue;
            }
            groups[idx]
                .get_or_insert_with(Vec::new)
                .push(event.bytes());
        }

        let mut success = 0usize;
        for (idx, group_opt) in groups.into_iter().enumerate() {
            let Some(group) = group_opt else { continue };
            let Some(shard_lock) = table.shards.get(idx) else {
                continue;
            };
            // Notify once per shard after the locked push run — single
            // wake covers the whole group, drain worker drains all of
            // it on one trip.
            let notify = {
                let mut shard = shard_lock.lock();
                // Pull `shard_id` from the locked shard rather than
                // carrying a parallel `group_ids` Vec.
                let shard_id = shard.id;
                let mut any_ok = false;
                for bytes in group {
                    if self
                        .push_with_backpressure(&mut shard, shard_id, bytes)
                        .is_ok()
                    {
                        success += 1;
                        any_ok = true;
                    }
                }
                if any_ok {
                    Some(shard.notify_handle())
                } else {
                    None
                }
            };
            if let Some(n) = notify {
                n.notify_one();
            }
        }

        (success, total)
    }

    /// Get a reference to a shard by ID.
    pub fn shard(&self, id: u16) -> Option<ShardRef> {
        let table = self.table.load();
        let idx = self.resolve_idx(&table, id)?;
        let shard = table.shards.get(idx)?.clone();
        Some(ShardRef { shard })
    }

    /// Get the wake handle for a shard by ID.
    ///
    /// The drain worker holds this `Arc<Notify>` and `await`s
    /// `notified()` instead of polling on a fixed timer; producers
    /// fire `notify_one()` after a successful push. Returns `None` if
    /// the shard id isn't currently in the routing table.
    ///
    /// Briefly locks the shard mutex to clone the handle. Called once
    /// at drain-worker startup, not on the hot path.
    pub fn shard_notify(&self, id: u16) -> Option<Arc<tokio::sync::Notify>> {
        let table = self.table.load();
        let idx = self.resolve_idx(&table, id)?;
        let shard_lock = table.shards.get(idx)?;
        let handle = shard_lock.lock().notify_handle();
        Some(handle)
    }

    /// Wake every drain worker by notifying all pending waiters on
    /// every shard. Used at shutdown so workers exit promptly without
    /// waiting for the periodic safety timer.
    pub fn notify_all_drain_workers(&self) {
        let table = self.table.load();
        for shard_lock in &table.shards {
            // Brief lock to grab the Notify handle. Cheap: shutdown
            // is rare and the handle clone is just an Arc bump.
            let n = shard_lock.lock().notify_handle();
            n.notify_waiters();
            // Also leave a permit so a worker that wasn't yet awaiting
            // sees one when it next calls `notified()`.
            n.notify_one();
        }
    }

    /// Execute a function with exclusive access to a shard.
    pub fn with_shard<F, R>(&self, id: u16, f: F) -> Option<R>
    where
        F: FnOnce(&mut Shard) -> R,
    {
        let table = self.table.load();
        let idx = self.resolve_idx(&table, id)?;
        table.shards.get(idx).map(|shard_lock| {
            let mut shard = shard_lock.lock();
            f(&mut shard)
        })
    }

    /// Returns true if every shard's ring buffer is empty.
    ///
    /// Cheaper than `shard_ids()` + repeated `with_shard`: loads the
    /// routing table once and checks each shard behind a brief lock.
    pub fn all_shards_empty(&self) -> bool {
        let table = self.table.load();
        table.shards.iter().all(|s| s.lock().is_empty())
    }

    /// Iterate over all active shard IDs.
    pub fn shard_ids(&self) -> Vec<u16> {
        self.table.load().shard_index.keys().copied().collect()
    }

    /// Get aggregated statistics from all shards.
    ///
    /// Lock-free: reads each shard's atomic counters directly via the
    /// parallel `counters` vector on the routing table, with no per-
    /// shard mutex acquisition.
    pub fn stats(&self) -> ShardStats {
        let table = self.table.load();
        let mut total = ShardStats::default();
        for counters in table.counters.iter() {
            let snap = counters.snapshot();
            total.events_ingested += snap.events_ingested;
            total.events_dropped += snap.events_dropped;
            total.batches_dispatched += snap.batches_dispatched;
        }
        total
    }

    /// Rebuild the routing table with a closure that sees the old
    /// `(shards, counters, shard_index)` and produces the new ones.
    /// Serialized by `rebuild_lock` so concurrent scaling operations
    /// can't race on read-modify-write of the table.
    fn rebuild_table<F>(&self, f: F)
    where
        F: FnOnce(
            &Vec<Arc<parking_lot::Mutex<Shard>>>,
            &Vec<Arc<ShardCounters>>,
            &ShardIdMap<usize>,
        ) -> ShardTable,
    {
        let _guard = self.rebuild_lock.lock();
        let old = self.table.load();
        let new = f(&old.shards, &old.counters, &old.shard_index);
        self.table.store(Arc::new(new));
    }

    /// Add a new shard (for dynamic scaling).
    /// Returns the new shard ID.
    pub fn add_shard(&self) -> Result<u16, ScalingError> {
        let mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        // Scale up through the mapper
        let new_ids = mapper.scale_up(1)?;
        let new_id = new_ids[0];

        // Create the actual shard
        let metrics = mapper.metrics_collector(new_id).ok_or_else(|| {
            ScalingError::InvalidPolicy(format!("no metrics collector for shard {}", new_id))
        })?;
        let new_shard = Shard::with_metrics(new_id, self.ring_buffer_capacity, metrics);
        let new_counters = new_shard.counters();
        let new_shard = Arc::new(parking_lot::Mutex::new(new_shard));

        self.rebuild_table(|shards, counters, shard_index| {
            let mut shards = shards.clone();
            let mut counters = counters.clone();
            let mut shard_index = shard_index.clone();
            let idx = shards.len();
            shards.push(new_shard.clone());
            counters.push(new_counters.clone());
            shard_index.insert(new_id, idx);
            ShardTable {
                shards,
                counters,
                shard_index,
            }
        });

        self.num_shards
            .fetch_add(1, std::sync::atomic::Ordering::Release);

        Ok(new_id)
    }

    /// Start draining a shard (for dynamic scaling).
    pub fn drain_shard(&self, shard_id: u16) -> Result<(), ScalingError> {
        let mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        // Mark as draining in the mapper
        if let Some(collector) = mapper.metrics_collector(shard_id) {
            collector.set_draining(true);
        }

        Ok(())
    }

    /// Remove a stopped shard (for dynamic scaling).
    pub fn remove_shard(&self, shard_id: u16) -> Result<(), ScalingError> {
        // Ensure dynamic scaling is enabled
        let _mapper = self.mapper.as_ref().ok_or(ScalingError::InvalidPolicy(
            "Dynamic scaling not enabled".into(),
        ))?;

        let mut removed = false;
        self.rebuild_table(|shards, counters, shard_index| {
            let mut shards = shards.clone();
            let mut counters = counters.clone();
            let mut shard_index = shard_index.clone();

            if let Some(idx) = shard_index.remove(&shard_id) {
                removed = true;
                shards.swap_remove(idx);
                counters.swap_remove(idx);
                // swap_remove moved the last element into `idx`: update its
                // index mapping.
                if idx < shards.len() {
                    let moved_shard_id = shards[idx].lock().id;
                    shard_index.insert(moved_shard_id, idx);
                }
            }

            ShardTable {
                shards,
                counters,
                shard_index,
            }
        });

        if removed {
            self.num_shards
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        }

        Ok(())
    }

    /// Collect metrics from all shards (for dynamic scaling decisions).
    pub fn collect_metrics(&self) -> Option<Vec<ShardMetrics>> {
        self.mapper.as_ref().map(|m| m.collect_metrics())
    }

    /// Evaluate and optionally execute scaling.
    pub fn evaluate_scaling(&self) -> ScalingDecision {
        self.mapper
            .as_ref()
            .map(|m| m.evaluate_scaling())
            .unwrap_or(ScalingDecision::None)
    }
}

/// An owned handle to a shard. Holding this does not block scaling
/// operations; the shard stays alive via `Arc` refcount even if
/// removed from the table.
pub struct ShardRef {
    shard: Arc<parking_lot::Mutex<Shard>>,
}

impl ShardRef {
    /// Lock the shard for exclusive access.
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, Shard> {
        self.shard.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Sanity-check that `U16IdHasher` actually distinguishes adjacent
    /// shard ids — a buggy hasher that always returned `0` would
    /// degrade `shard_index` to a linked list. Probe a few adjacent
    /// pairs and confirm the produced hashes differ.
    #[test]
    fn test_u16_id_hasher_spreads_adjacent_keys() {
        use std::hash::{Hash, Hasher};
        let probe = |k: u16| {
            let mut h = U16IdHasher::default();
            k.hash(&mut h);
            h.finish()
        };
        for k in 0..16u16 {
            assert_ne!(
                probe(k),
                probe(k + 1),
                "adjacent shard ids must hash differently (k={k})"
            );
        }
        // And: the hash of 0 is not equal to the hash of u16::MAX.
        assert_ne!(probe(0), probe(u16::MAX));
    }

    #[test]
    fn test_shard_push_pop() {
        let mut shard = Shard::new(0, 1024);

        let ts = shard.try_push(json!({"test": 1})).unwrap();
        assert!(ts > 0);
        assert_eq!(shard.len(), 1);

        let event = shard.try_pop().unwrap();
        assert_eq!(event.shard_id, 0);
        assert_eq!(event.insertion_ts, ts);
        assert!(shard.is_empty());
    }

    #[test]
    #[allow(deprecated)] // exercises the deprecated `select_shard` path
    fn test_shard_manager_routing() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Same event should always go to the same shard
        let event = json!({"key": "value"});
        let shard1 = manager.select_shard(&event);
        let shard2 = manager.select_shard(&event);
        assert_eq!(shard1, shard2);

        // Different events may go to different shards
        let events: Vec<_> = (0..100).map(|i| json!({"i": i})).collect();
        let shards: std::collections::HashSet<_> =
            events.iter().map(|e| manager.select_shard(e)).collect();

        // With 100 random events and 4 shards, we should hit multiple shards
        assert!(shards.len() > 1);
    }

    /// Regression: the deprecated `select_shard(&JsonValue)` must produce
    /// the same shard id as `select_shard_by_hash` would for the
    /// equivalent `RawEvent`. They share underlying logic now, but if a
    /// future refactor splits them this test catches the divergence
    /// before consumers do.
    #[test]
    #[allow(deprecated)]
    fn test_select_shard_matches_select_shard_by_hash() {
        let manager = ShardManager::new(8, 1024, BackpressureMode::DropNewest);
        for i in 0..200 {
            let v = json!({"i": i, "tag": format!("user-{i}")});
            let raw = RawEvent::from_value(v.clone());
            assert_eq!(
                manager.select_shard(&v),
                manager.select_shard_by_hash(raw.hash()),
                "select_shard and select_shard_by_hash must agree (i={i})"
            );
        }
    }

    #[test]
    fn test_shard_manager_ingest() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        for i in 0..100 {
            let event = json!({"i": i});
            let result = manager.ingest(event);
            assert!(result.is_ok());
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 100);
        assert_eq!(stats.events_dropped, 0);
    }

    #[test]
    fn test_backpressure_drop_newest() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropNewest);

        // Fill the buffer (capacity 4, usable 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Next insert should fail
        let result = manager.ingest(json!({"i": 999}));
        assert!(matches!(result, Err(IngestionError::Backpressure)));

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 3);
        assert_eq!(stats.events_dropped, 1);
    }

    #[test]
    fn test_backpressure_drop_oldest() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Next insert should succeed by dropping oldest
        let result = manager.ingest(json!({"i": 999}));
        assert!(result.is_ok());

        // Verify the oldest was dropped
        let shard = manager.shard(0).unwrap();
        let events = shard.lock().pop_batch(10);

        // Should have events 1, 2, 999 (0 was dropped)
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].parse().unwrap(), json!({"i": 1}));
        assert_eq!(events[2].parse().unwrap(), json!({"i": 999}));
    }

    #[test]
    fn test_raw_event_ingestion() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        for i in 0..100 {
            let raw = RawEvent::from_str(&format!(r#"{{"i": {}}}"#, i));
            let result = manager.ingest_raw(raw);
            assert!(result.is_ok());
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 100);
        assert_eq!(stats.events_dropped, 0);
    }

    /// `ingest_raw_batch` groups events by destination shard before
    /// pushing — verify the grouping preserves FIFO within a shard,
    /// honors hash-based routing, and that totals match `ingest_raw`.
    #[test]
    fn test_ingest_raw_batch_routes_and_preserves_order() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);
        let events: Vec<RawEvent> = (0..200)
            .map(|i| RawEvent::from_str(&format!(r#"{{"i":{}}}"#, i)))
            .collect();

        // Snapshot the expected destination for each event so we can
        // compare against what actually landed in each shard.
        let expected_dests: Vec<u16> = events
            .iter()
            .map(|e| manager.select_shard_by_hash(e.hash()))
            .collect();

        let (success, total) = manager.ingest_raw_batch(events.clone());
        assert_eq!(success, 200, "all events should land with ample capacity");
        assert_eq!(total, 200, "total count must equal input length");

        // Aggregate totals must match.
        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 200);
        assert_eq!(stats.events_dropped, 0);

        // Per-shard totals must match the expected routing distribution,
        // and the distribution must span more than one shard (otherwise
        // the test wouldn't exercise the grouping path).
        let mut expected_by_shard: std::collections::HashMap<u16, u64> =
            std::collections::HashMap::new();
        for d in &expected_dests {
            *expected_by_shard.entry(*d).or_default() += 1;
        }
        assert!(
            expected_by_shard.len() > 1,
            "hash distribution should span multiple shards"
        );
        for shard_id in 0..4u16 {
            let got = manager
                .with_shard(shard_id, |s| s.stats().events_ingested)
                .unwrap();
            let want = expected_by_shard.get(&shard_id).copied().unwrap_or(0);
            assert_eq!(got, want, "shard {} ingested count mismatch", shard_id);
        }

        // FIFO within a shard: the events a shard received, in the order
        // we batched them, must come out of the ring buffer in the same
        // order.
        for shard_id in 0..4u16 {
            let expected_payloads: Vec<&[u8]> = events
                .iter()
                .zip(expected_dests.iter())
                .filter(|(_, d)| **d == shard_id)
                .map(|(e, _)| e.as_bytes())
                .collect();
            let popped = manager.with_shard(shard_id, |s| s.pop_batch(1024)).unwrap();
            assert_eq!(popped.len(), expected_payloads.len());
            for (i, ev) in popped.iter().enumerate() {
                assert_eq!(
                    ev.as_bytes(),
                    expected_payloads[i],
                    "shard {} position {} out of order",
                    shard_id,
                    i
                );
            }
        }
    }

    /// Batching past a shard's capacity must account every dropped
    /// event under `DropNewest`: `success` + `events_dropped` =
    /// `len(input)`.
    #[test]
    fn test_ingest_raw_batch_drop_accounting() {
        // Single shard, usable capacity 3 (ring buffer reserves one slot).
        let manager = ShardManager::new(1, 4, BackpressureMode::DropNewest);
        let events: Vec<RawEvent> = (0..10)
            .map(|i| RawEvent::from_str(&format!(r#"{{"i":{}}}"#, i)))
            .collect();

        let (success, total) = manager.ingest_raw_batch(events);
        assert_eq!(success, 3, "only 3 should fit under DropNewest");
        assert_eq!(total, 10, "total must reflect every input event");

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 3);
        assert_eq!(stats.events_dropped, 7);
    }

    /// Empty batch is a no-op and must not touch stats.
    #[test]
    fn test_ingest_raw_batch_empty() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);
        assert_eq!(
            manager.ingest_raw_batch(Vec::<RawEvent>::new()),
            (0, 0),
            "empty input yields (success=0, total=0)"
        );
        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 0);
        assert_eq!(stats.events_dropped, 0);
    }

    /// Regression: `ShardManager::ingest_raw_batch` accepts any
    /// `IntoIterator<Item = RawEvent>`, so callers can fuse the
    /// `Event → RawEvent` conversion without materializing an
    /// intermediate `Vec`. Pin the iterator-based call site here so a
    /// future refactor that re-tightens the signature to `Vec<RawEvent>`
    /// fails the test, not the downstream caller.
    #[test]
    fn test_ingest_raw_batch_accepts_iterator() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);
        // Call with a map-iterator (no intermediate Vec).
        let (success, total) = manager.ingest_raw_batch(
            (0..50).map(|i| RawEvent::from_str(&format!(r#"{{"i":{}}}"#, i))),
        );
        assert_eq!(total, 50);
        assert_eq!(success, 50);
    }

    #[test]
    fn test_remove_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.remove_shard(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_add_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.add_shard();
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_drain_shard_requires_dynamic_scaling() {
        // Static mode - no dynamic scaling
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Should fail because dynamic scaling is not enabled
        let result = manager.drain_shard(0);
        assert!(result.is_err());
        assert!(matches!(result, Err(ScalingError::InvalidPolicy(_))));
    }

    #[test]
    fn test_drop_oldest_counts_dropped_events() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer (capacity 4, usable 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // This should succeed by dropping the oldest event
        manager.ingest(json!({"i": 999})).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 4);
        // The initial push fails (counted as dropped), then retry succeeds
        assert_eq!(
            stats.events_dropped, 1,
            "DropOldest cycle should count exactly one drop"
        );
    }

    #[test]
    fn test_drop_oldest_raw_counts_dropped_events() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer
        for i in 0..3 {
            let raw = RawEvent::from_str(&format!(r#"{{"i": {}}}"#, i));
            manager.ingest_raw(raw).unwrap();
        }

        // This should succeed by dropping the oldest event
        let raw = RawEvent::from_str(r#"{"i": 999}"#);
        manager.ingest_raw(raw).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 4);
        assert_eq!(
            stats.events_dropped, 1,
            "DropOldest cycle should count exactly one drop"
        );
    }

    /// Pin the current contract for `BackpressureMode::Sample`:
    /// it returns `IngestionError::Sampled` once the buffer fills,
    /// indistinguishable in shape from a `Backpressure` rejection.
    /// Sampling itself ("keep 1 in N events") is **not implemented**
    /// — the comments in `ingest` / `ingest_raw` defer it to "a
    /// higher level" that does not exist. A consumer setting this
    /// mode today gets a rejection signal, never probabilistic
    /// admission.
    ///
    /// This test pins that contract so it cannot quietly change
    /// without an explicit decision. If sampling is ever wired up,
    /// this test will fail and force an update — at which point
    /// the implementer should also add coverage for the
    /// rate-proportional admission rate.
    #[test]
    fn sample_mode_currently_returns_sampled_after_buffer_fills() {
        // TODO(coverage round 2): `BackpressureMode::Sample` is
        // dead-on-arrival until "higher level" sampling lands;
        // see comments at `ShardManager::ingest` / `ingest_raw`.
        let manager = ShardManager::new(1, 4, BackpressureMode::Sample { rate: 2 });

        // Fill the buffer (capacity 4, usable 3).
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Both ingest paths must report `Sampled` — not `Backpressure`,
        // not `Ok` — so callers can distinguish the (currently
        // unused) sampling rejection from a hard backpressure
        // rejection in case sampling is wired up later.
        let json_result = manager.ingest(json!({"i": 999}));
        assert!(
            matches!(json_result, Err(IngestionError::Sampled)),
            "Sample mode must return Sampled on a full buffer (got {:?})",
            json_result
        );

        let raw_result = manager.ingest_raw(RawEvent::from_str(r#"{"i": 999}"#));
        assert!(
            matches!(raw_result, Err(IngestionError::Sampled)),
            "Sample mode must return Sampled on a full buffer via ingest_raw (got {:?})",
            raw_result
        );
    }

    #[test]
    fn test_drop_oldest_multiple_cycles() {
        let manager = ShardManager::new(1, 4, BackpressureMode::DropOldest);

        // Fill the buffer (usable capacity 3)
        for i in 0..3 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        // Push 5 more events, each triggers a DropOldest cycle
        for i in 3..8 {
            manager.ingest(json!({"i": i})).unwrap();
        }

        let stats = manager.stats();
        assert_eq!(stats.events_ingested, 8);
        assert_eq!(
            stats.events_dropped, 5,
            "each DropOldest cycle should count one drop"
        );
    }

    /// Successful `ingest` must hand off a wake to the per-shard
    /// `Notify`. Without this, the drain worker would have to fall
    /// back on the periodic safety timer (currently 100ms) and the
    /// "first event after idle" latency would regress to that
    /// interval — exactly the bug PERF #4 is meant to fix.
    #[tokio::test]
    async fn test_ingest_wakes_drain_notify() {
        // Single-shard manager so we know exactly which shard gets the
        // event, regardless of hash distribution.
        let manager = ShardManager::new(1, 1024, BackpressureMode::DropNewest);
        let notify = manager.shard_notify(0).expect("shard 0 exists");

        // No producer yet — `notified()` should not be ready.
        let try_immediate = tokio::time::timeout(
            std::time::Duration::from_millis(20),
            notify.notified(),
        )
        .await;
        assert!(
            try_immediate.is_err(),
            "notify must NOT have a permit before any push"
        );

        // After ingest, the next `notified().await` must resolve
        // immediately — Notify's stack-up-to-1 permit semantics are
        // what makes the drain worker's empty-then-await pattern race-free.
        manager.ingest(json!({"k": "v"})).unwrap();
        let woke = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            notify.notified(),
        )
        .await;
        assert!(
            woke.is_ok(),
            "successful ingest must call notify_one(); drain worker would otherwise spin",
        );
    }

    /// `ingest_raw_batch` must notify each destination shard exactly
    /// once (per shard, not per event) so a drain worker that already
    /// got the wake doesn't get a wasteful storm of `notify_one`s for
    /// the rest of the batch.
    #[tokio::test]
    async fn test_ingest_raw_batch_notifies_per_shard() {
        let manager = ShardManager::new(4, 1024, BackpressureMode::DropNewest);

        // Send a small batch of events that should distribute across
        // shards. Just verify each shard with at least one event has a
        // wake permit available.
        let events: Vec<RawEvent> = (0..50)
            .map(|i| RawEvent::from_str(&format!(r#"{{"i":{i}}}"#)))
            .collect();
        let expected_dests: std::collections::HashSet<u16> = events
            .iter()
            .map(|e| manager.select_shard_by_hash(e.hash()))
            .collect();

        let (success, total) = manager.ingest_raw_batch(events);
        assert_eq!(success, 50);
        assert_eq!(total, 50);

        // Every shard that received events must have a wake permit.
        for shard_id in expected_dests {
            let notify = manager.shard_notify(shard_id).unwrap();
            let woke = tokio::time::timeout(
                std::time::Duration::from_millis(50),
                notify.notified(),
            )
            .await;
            assert!(woke.is_ok(), "shard {shard_id} did not receive a wake");
        }
    }

    /// `notify_all_drain_workers` is what `EventBus::shutdown` uses to
    /// pull every drain worker out of its `Notify::notified()` await
    /// without waiting for the periodic safety timer.
    #[tokio::test]
    async fn test_notify_all_drain_workers_wakes_idle_waiters() {
        let manager = ShardManager::new(3, 1024, BackpressureMode::DropNewest);
        let n0 = manager.shard_notify(0).unwrap();
        let n1 = manager.shard_notify(1).unwrap();
        let n2 = manager.shard_notify(2).unwrap();

        // Each shard's notify must initially be "no permit available".
        for n in [&n0, &n1, &n2] {
            assert!(
                tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    n.notified()
                )
                .await
                .is_err()
            );
        }

        manager.notify_all_drain_workers();

        // After the broadcast wake, every shard's notify must have a
        // permit (or wake an existing waiter).
        for (i, n) in [&n0, &n1, &n2].iter().enumerate() {
            let woke = tokio::time::timeout(
                std::time::Duration::from_millis(50),
                n.notified(),
            )
            .await;
            assert!(woke.is_ok(), "shard {i} not woken by broadcast");
        }
    }
}
