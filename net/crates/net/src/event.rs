//! Event types for the Net event bus.
//!
//! Events are opaque JSON values - the event bus performs no schema validation
//! or interpretation of event content.
//!
//! # Performance Optimization
//!
//! Since Net is schema-agnostic, we offer multiple event representations:
//!
//! - `Event`: Standard wrapper around `serde_json::Value` (convenient but slower)
//! - `RawEvent`: Pre-serialized bytes with cached hash (fastest for high-throughput)
//!
//! For maximum performance, use `RawEvent::from_bytes()` when you already have
//! JSON bytes (e.g., from a network buffer or file).

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// An opaque event - any valid JSON value.
///
/// The event bus does not validate, interpret, or enforce any schema.
/// Events are treated as opaque binary blobs internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Event(pub JsonValue);

impl Event {
    /// Create a new event from a JSON value.
    #[inline]
    pub fn new(value: JsonValue) -> Self {
        Self(value)
    }

    /// Create an event from a JSON string.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s).map(Self)
    }

    /// Create an event from raw bytes.
    #[inline]
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes).map(Self)
    }

    /// Get the inner JSON value.
    #[inline]
    pub fn into_inner(self) -> JsonValue {
        self.0
    }

    /// Get a reference to the inner JSON value.
    #[inline]
    pub fn as_value(&self) -> &JsonValue {
        &self.0
    }

    /// Convert to a raw event (serializes once, caches the result).
    #[inline]
    pub fn into_raw(self) -> RawEvent {
        RawEvent::from_value(self.0)
    }
}

impl From<JsonValue> for Event {
    #[inline]
    fn from(value: JsonValue) -> Self {
        Self(value)
    }
}

impl From<Event> for JsonValue {
    #[inline]
    fn from(event: Event) -> Self {
        event.0
    }
}

/// A pre-serialized event with cached hash.
///
/// This is the high-performance event type for schema-agnostic ingestion.
/// By storing the raw bytes and pre-computed hash, we avoid:
/// - Re-serialization on every operation
/// - Repeated hashing for shard selection
///
/// # Example
///
/// ```rust,ignore
/// // From network buffer or file (zero-copy if Bytes is used)
/// let raw = RawEvent::from_bytes(network_buffer);
///
/// // From existing JSON value (serializes once)
/// let raw = RawEvent::from_value(json!({"key": "value"}));
///
/// // Ingestion uses cached hash - no re-serialization
/// bus.ingest_raw(raw)?;
/// ```
#[derive(Clone)]
pub struct RawEvent {
    /// Pre-serialized JSON bytes.
    bytes: Bytes,
    /// Pre-computed hash for shard selection.
    hash: u64,
}

impl RawEvent {
    /// Create a raw event from bytes.
    ///
    /// The bytes must be valid JSON. No validation is performed for performance.
    /// Use `from_bytes_validated` if you need validation.
    #[inline]
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        let bytes = bytes.into();
        let hash = xxhash_rust::xxh3::xxh3_64(&bytes);
        Self { bytes, hash }
    }

    /// Create a raw event from bytes with a pre-computed hash.
    ///
    /// Use this when you've already computed the xxhash (e.g., for reused events).
    /// The caller is responsible for ensuring the hash matches the bytes.
    #[inline]
    pub fn from_bytes_with_hash(bytes: impl Into<Bytes>, hash: u64) -> Self {
        Self {
            bytes: bytes.into(),
            hash,
        }
    }

    /// Create a raw event from bytes with JSON validation.
    #[inline]
    pub fn from_bytes_validated(bytes: impl Into<Bytes>) -> Result<Self, serde_json::Error> {
        let bytes = bytes.into();
        // Validate it's valid JSON by attempting to parse
        let _: JsonValue = serde_json::from_slice(&bytes)?;
        let hash = xxhash_rust::xxh3::xxh3_64(&bytes);
        Ok(Self { bytes, hash })
    }

    /// Create a raw event from a JSON value.
    ///
    /// This serializes the value once and caches the result.
    #[inline]
    pub fn from_value(value: JsonValue) -> Self {
        let bytes =
            Bytes::from(serde_json::to_vec(&value).expect("Value serialization is infallible"));
        let hash = xxhash_rust::xxh3::xxh3_64(&bytes);
        Self { bytes, hash }
    }

    /// Create a raw event from a string.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(s.as_bytes()))
    }

    /// Get the raw bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the bytes (clone is cheap - reference counted).
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }

    /// Get the pre-computed hash.
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Parse the bytes into a JSON value (for when you need to inspect).
    #[inline]
    pub fn parse(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::from_slice(&self.bytes)
    }

    /// Get the byte length.
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl std::fmt::Debug for RawEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawEvent")
            .field("len", &self.bytes.len())
            .field("hash", &self.hash)
            .finish()
    }
}

impl From<Event> for RawEvent {
    #[inline]
    fn from(event: Event) -> Self {
        event.into_raw()
    }
}

impl From<JsonValue> for RawEvent {
    #[inline]
    fn from(value: JsonValue) -> Self {
        RawEvent::from_value(value)
    }
}

/// Internal event representation with metadata assigned at ingestion.
///
/// This is the canonical form of an event within the event bus.
/// The `insertion_ts` provides deterministic ordering within a shard.
///
/// Uses `Bytes` for zero-copy, reference-counted storage.
#[derive(Debug, Clone)]
pub struct InternalEvent {
    /// Pre-serialized JSON payload (reference-counted, zero-copy clone).
    pub raw: Bytes,

    /// Monotonically increasing insertion timestamp (nanoseconds).
    /// Strictly ordered within a shard, not globally.
    pub insertion_ts: u64,

    /// Shard this event was assigned to.
    pub shard_id: u16,
}

impl InternalEvent {
    /// Create a new internal event from raw bytes.
    #[inline]
    pub fn new(raw: Bytes, insertion_ts: u64, shard_id: u16) -> Self {
        Self {
            raw,
            insertion_ts,
            shard_id,
        }
    }

    /// Create from a JSON value (serializes once).
    #[inline]
    pub fn from_value(value: JsonValue, insertion_ts: u64, shard_id: u16) -> Self {
        let raw =
            Bytes::from(serde_json::to_vec(&value).expect("Value serialization is infallible"));
        Self {
            raw,
            insertion_ts,
            shard_id,
        }
    }

    /// Parse the raw bytes into a JSON value.
    #[inline]
    pub fn parse(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::from_slice(&self.raw)
    }

    /// Get the raw bytes as a slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }
}

/// A batch of events for adapter dispatch.
///
/// Batches are formed by shard workers and contain strictly ordered
/// events from a single shard.
#[derive(Debug, Clone)]
pub struct Batch {
    /// The shard this batch belongs to.
    pub shard_id: u16,

    /// Events in insertion order.
    pub events: Vec<InternalEvent>,

    /// Sequence number of the first event in this batch.
    /// Used for idempotent retry handling.
    pub sequence_start: u64,
}

impl Batch {
    /// Create a new batch.
    #[inline]
    pub fn new(shard_id: u16, events: Vec<InternalEvent>, sequence_start: u64) -> Self {
        Self {
            shard_id,
            events,
            sequence_start,
        }
    }

    /// Returns the number of events in this batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if this batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// An event retrieved from storage with its backend-specific ID.
#[derive(Debug, Clone)]
pub struct StoredEvent {
    /// Backend-specific identifier.
    pub id: String,

    /// Raw JSON payload bytes (deferred parsing for performance).
    pub raw: Bytes,

    /// Insertion timestamp from ingestion.
    pub insertion_ts: u64,

    /// Shard this event belongs to.
    pub shard_id: u16,
}

impl StoredEvent {
    /// Create a new stored event from raw bytes.
    #[inline]
    pub fn new(id: String, raw: Bytes, insertion_ts: u64, shard_id: u16) -> Self {
        Self {
            id,
            raw,
            insertion_ts,
            shard_id,
        }
    }

    /// Create a new stored event from a JSON value (serializes once).
    #[inline]
    pub fn from_value(id: String, value: JsonValue, insertion_ts: u64, shard_id: u16) -> Self {
        let raw =
            Bytes::from(serde_json::to_vec(&value).expect("Value serialization is infallible"));
        Self {
            id,
            raw,
            insertion_ts,
            shard_id,
        }
    }

    /// Parse the raw bytes into a JSON value on demand.
    #[inline]
    pub fn parse(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::from_slice(&self.raw)
    }

    /// Get the raw bytes as a string slice (for serialization).
    #[inline]
    pub fn raw_str(&self) -> &str {
        // Safety: Net events are valid UTF-8 JSON
        std::str::from_utf8(&self.raw).unwrap_or("{}")
    }
}

impl Serialize for StoredEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StoredEvent", 4)?;
        state.serialize_field("id", &self.id)?;
        // Serialize raw bytes as a JSON value (parse then embed)
        let value: JsonValue = serde_json::from_slice(&self.raw).unwrap_or(JsonValue::Null);
        state.serialize_field("raw", &value)?;
        state.serialize_field("insertion_ts", &self.insertion_ts)?;
        state.serialize_field("shard_id", &self.shard_id)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_new() {
        let value = json!({"key": "value"});
        let event = Event::new(value.clone());
        assert_eq!(event.as_value(), &value);
    }

    #[test]
    fn test_event_from_str() {
        let event = Event::from_str(r#"{"key": "value"}"#).unwrap();
        assert_eq!(event.as_value()["key"], "value");
    }

    #[test]
    fn test_event_from_str_invalid() {
        let result = Event::from_str("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_event_from_slice() {
        let bytes = br#"{"key": "value"}"#;
        let event = Event::from_slice(bytes).unwrap();
        assert_eq!(event.as_value()["key"], "value");
    }

    #[test]
    fn test_event_into_inner() {
        let value = json!({"key": "value"});
        let event = Event::new(value.clone());
        assert_eq!(event.into_inner(), value);
    }

    #[test]
    fn test_event_into_raw() {
        let event = Event::new(json!({"key": "value"}));
        let raw = event.into_raw();
        assert!(!raw.is_empty());
        assert!(raw.hash() != 0);
    }

    #[test]
    fn test_event_from_json_value() {
        let value = json!({"key": "value"});
        let event: Event = value.clone().into();
        assert_eq!(event.as_value(), &value);
    }

    #[test]
    fn test_event_into_json_value() {
        let value = json!({"key": "value"});
        let event = Event::new(value.clone());
        let result: JsonValue = event.into();
        assert_eq!(result, value);
    }

    #[test]
    fn test_raw_event_from_bytes() {
        let bytes = br#"{"key": "value"}"#;
        let raw = RawEvent::from_bytes(bytes.as_slice());
        assert_eq!(raw.as_bytes(), bytes);
        assert!(!raw.is_empty());
        assert_eq!(raw.len(), bytes.len());
    }

    #[test]
    fn test_raw_event_from_str() {
        let s = r#"{"key": "value"}"#;
        let raw = RawEvent::from_str(s);
        assert_eq!(raw.as_bytes(), s.as_bytes());
    }

    #[test]
    fn test_raw_event_from_value() {
        let value = json!({"key": "value"});
        let raw = RawEvent::from_value(value);
        let parsed = raw.parse().unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_raw_event_from_bytes_validated() {
        let valid = br#"{"key": "value"}"#;
        let result = RawEvent::from_bytes_validated(valid.as_slice());
        assert!(result.is_ok());

        let invalid = b"not valid json";
        let result = RawEvent::from_bytes_validated(invalid.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_raw_event_hash_consistency() {
        let raw1 = RawEvent::from_str(r#"{"key": "value"}"#);
        let raw2 = RawEvent::from_str(r#"{"key": "value"}"#);
        assert_eq!(raw1.hash(), raw2.hash());

        let raw3 = RawEvent::from_str(r#"{"key": "other"}"#);
        assert_ne!(raw1.hash(), raw3.hash());
    }

    #[test]
    fn test_raw_event_bytes_clone() {
        let raw = RawEvent::from_str(r#"{"key": "value"}"#);
        let bytes1 = raw.bytes();
        let bytes2 = raw.bytes();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_raw_event_debug() {
        let raw = RawEvent::from_str(r#"{"key": "value"}"#);
        let debug = format!("{:?}", raw);
        assert!(debug.contains("RawEvent"));
        assert!(debug.contains("len"));
        assert!(debug.contains("hash"));
    }

    #[test]
    fn test_raw_event_from_event() {
        let event = Event::new(json!({"key": "value"}));
        let raw: RawEvent = event.into();
        assert!(!raw.is_empty());
    }

    #[test]
    fn test_raw_event_from_json_value() {
        let value = json!({"key": "value"});
        let raw: RawEvent = value.into();
        assert!(!raw.is_empty());
    }

    #[test]
    fn test_internal_event_new() {
        let raw = Bytes::from(r#"{"key": "value"}"#);
        let event = InternalEvent::new(raw.clone(), 12345, 0);
        assert_eq!(event.raw, raw);
        assert_eq!(event.insertion_ts, 12345);
        assert_eq!(event.shard_id, 0);
    }

    #[test]
    fn test_internal_event_from_value() {
        let event = InternalEvent::from_value(json!({"key": "value"}), 12345, 0);
        assert_eq!(event.insertion_ts, 12345);
        assert_eq!(event.shard_id, 0);
        let parsed = event.parse().unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_internal_event_as_bytes() {
        let raw = Bytes::from(r#"{"key": "value"}"#);
        let event = InternalEvent::new(raw.clone(), 12345, 0);
        assert_eq!(event.as_bytes(), raw.as_ref());
    }

    #[test]
    fn test_batch_new() {
        let events = vec![
            InternalEvent::from_value(json!({"i": 0}), 1, 0),
            InternalEvent::from_value(json!({"i": 1}), 2, 0),
        ];
        let batch = Batch::new(0, events, 100);
        assert_eq!(batch.shard_id, 0);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.sequence_start, 100);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_empty() {
        let batch = Batch::new(0, vec![], 0);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_stored_event_new() {
        let raw = Bytes::from(r#"{"key":"value"}"#);
        let event = StoredEvent::new("stream-123".to_string(), raw, 12345, 0);
        assert_eq!(event.id, "stream-123");
        let parsed = event.parse().unwrap();
        assert_eq!(parsed["key"], "value");
        assert_eq!(event.insertion_ts, 12345);
        assert_eq!(event.shard_id, 0);
    }
}
