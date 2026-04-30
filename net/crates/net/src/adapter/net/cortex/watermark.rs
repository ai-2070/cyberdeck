//! `WatermarkingFold<S, F>` — wraps a user fold and piggybacks
//! `app_seq` discovery onto its event-traversal pass.
//!
//! Background. Both `TasksAdapter` and `MemoriesAdapter` keep a
//! per-origin monotonic counter (`app_seq`) that gets stamped on every
//! `EventMeta::seq_or_ts`. After `open_from_snapshot` the counter must
//! satisfy `app_seq > max(seq_or_ts of any in-log event for our
//! origin)` before the first `ingest_typed`, otherwise the next ingest
//! can stamp a duplicate `seq_or_ts` (data corruption — two distinct
//! events with the same per-origin sequence number).
//!
//! Pre-fix the typed adapters did that discovery via a synchronous
//! `read_range` walk that re-materialized every event after the inner
//! fold task had already done so. For an N-event log on disk this was
//! N redundant payload reads + N redundant checksum verifications + 2N
//! `Bytes` copies (BUG #148 in `BUG_AUDIT_2026_04_30_CORE.md`).
//!
//! Post-fix the typed adapters install this wrapper around the user
//! fold. The fold task reads each event exactly once; on every
//! successful inner-fold `apply` we parse the leading [`EventMeta`]
//! and, if the event matches our `origin_hash`, advance the shared
//! `Arc<AtomicU64>` via `fetch_max(meta.seq_or_ts + 1)`. The typed
//! constructors then `wait_for_seq(replay_end - 1).await` before
//! returning so callers see a fully-ready adapter — `app_seq` is
//! correct synchronously from the caller's perspective even though
//! it was assembled asynchronously by the fold task.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::super::redex::{RedexError, RedexEvent, RedexFold};
use super::meta::{EventMeta, EVENT_META_SIZE};

/// Wraps `inner: F` and updates a shared `Arc<AtomicU64>` watermark
/// from each event's [`EventMeta`] header. Only events whose
/// `origin_hash` matches `self.origin_hash` advance the counter;
/// events from other origins are folded but ignored for the
/// watermark.
///
/// `S` is the user state type passed through to the inner fold.
pub(super) struct WatermarkingFold<S, F> {
    inner: F,
    app_seq: Arc<AtomicU64>,
    origin_hash: u32,
    _state: PhantomData<fn(&mut S)>,
}

impl<S, F> WatermarkingFold<S, F> {
    pub(super) fn new(inner: F, app_seq: Arc<AtomicU64>, origin_hash: u32) -> Self {
        Self {
            inner,
            app_seq,
            origin_hash,
            _state: PhantomData,
        }
    }
}

impl<S, F> RedexFold<S> for WatermarkingFold<S, F>
where
    F: RedexFold<S>,
{
    fn apply(&mut self, ev: &RedexEvent, state: &mut S) -> Result<(), RedexError> {
        // Inner fold owns the user-visible state-update semantics. If
        // it errors we surface that verbatim — the watermark only
        // advances on a successful apply, so a fold-error policy of
        // `Continue` skips this event for both state AND watermark
        // accounting (matching the pre-fix behavior where the
        // synchronous `read_range` loop would have included the
        // event but the fold would have skipped it).
        self.inner.apply(ev, state)?;

        // Defensive payload-length guard — a payload shorter than
        // `EVENT_META_SIZE` cannot have come through `ingest_typed`
        // (which always writes a `EventMeta` prefix). Still possible
        // if a third party appends raw bytes to the same channel
        // file, in which case we silently skip rather than corrupt
        // the watermark with a bogus parse.
        if ev.payload.len() < EVENT_META_SIZE {
            return Ok(());
        }
        let Some(meta) = EventMeta::from_bytes(&ev.payload[..EVENT_META_SIZE]) else {
            return Ok(());
        };
        if meta.origin_hash != self.origin_hash {
            return Ok(());
        }

        // `fetch_max` is the right primitive: events arrive in
        // RedEX-seq order (which is NOT the same as `seq_or_ts` order
        // — two adapters writing to the same channel can interleave
        // their per-origin counters), so we want monotonic-up
        // semantics regardless of arrival order. `saturating_add`
        // pins the watermark at `u64::MAX` if we somehow see
        // `seq_or_ts == u64::MAX` (impossible in practice — would
        // require 2^64 ingests under one origin — but cheap to be
        // safe).
        let next = meta.seq_or_ts.saturating_add(1);
        self.app_seq.fetch_max(next, Ordering::AcqRel);
        Ok(())
    }
}
