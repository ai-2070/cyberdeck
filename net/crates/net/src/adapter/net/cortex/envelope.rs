//! `EventEnvelope` + `IntoRedexPayload` — the handoff from caller code
//! to the CortEX adapter's ingest path.
//!
//! The adapter is agnostic to what's inside the payload tail. It only
//! requires that a caller type can project itself into
//! `(EventMeta, Bytes)`. Implementations define how their domain
//! objects map onto the 20-byte header + type-specific bytes.
//!
//! A default [`EventEnvelope`] is provided for callers that already
//! have the `(meta, payload)` pair in hand. CortEX's own richer
//! envelope type (defined in CortEX's plan, not here) will implement
//! [`IntoRedexPayload`] directly.

use bytes::Bytes;

use super::meta::EventMeta;

/// Project a caller type into a RedEX payload.
///
/// Implementations return `(meta, tail)`. The adapter concatenates the
/// 20-byte meta with the tail and appends the result to RedEX.
pub trait IntoRedexPayload {
    /// Consume `self` and return the `(meta, tail)` pair.
    fn into_redex_payload(self) -> (EventMeta, Bytes);
}

/// A straightforward envelope pairing an [`EventMeta`] with its
/// payload tail. Use this when CortEX's richer envelope type is not
/// needed (tests, simple callers, the v1 adapter wire-up).
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    /// The 20-byte header that will land at the start of the RedEX
    /// payload.
    pub meta: EventMeta,
    /// Type-specific bytes following the header.
    pub payload: Bytes,
}

impl EventEnvelope {
    /// Build an envelope from a header and any byte-like payload.
    pub fn new(meta: EventMeta, payload: impl Into<Bytes>) -> Self {
        Self {
            meta,
            payload: payload.into(),
        }
    }
}

impl IntoRedexPayload for EventEnvelope {
    fn into_redex_payload(self) -> (EventMeta, Bytes) {
        (self.meta, self.payload)
    }
}

/// Convenience impl: a `(meta, tail)` tuple projects trivially.
impl IntoRedexPayload for (EventMeta, Bytes) {
    fn into_redex_payload(self) -> (EventMeta, Bytes) {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_into_payload_roundtrip() {
        let meta = EventMeta::new(0x10, 0, 0xAA, 1, 0xBB);
        let tail = Bytes::from_static(b"hello");
        let env = EventEnvelope::new(meta, tail.clone());
        let (meta_out, tail_out) = env.into_redex_payload();
        assert_eq!(meta_out, meta);
        assert_eq!(tail_out, tail);
    }

    #[test]
    fn test_tuple_impl() {
        let meta = EventMeta::new(0, 0, 0, 0, 0);
        let tail = Bytes::from_static(b"x");
        let (m, t) = (meta, tail.clone()).into_redex_payload();
        assert_eq!(m, meta);
        assert_eq!(t, tail);
    }
}
