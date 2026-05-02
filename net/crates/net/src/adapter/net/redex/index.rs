//! `RedexIndex<K, V>` — generic tail-driven secondary index.
//!
//! A [`RedexIndex`] spawns a background task that tails a [`RedexFile`],
//! decodes each event as a user-supplied type `T`, and applies the
//! caller's projection to emit [`IndexOp`]s. The projection owns the
//! decision of what to index and how; the index owns the bookkeeping.
//!
//! ## When to use it
//!
//! - You have a RedEX file acting as the source of truth for some
//!   domain.
//! - You want a cheap `(K) → Vec<V>` lookup that reflects the file's
//!   current state, without rolling the bookkeeping yourself.
//! - You can tolerate eventual consistency bounded by "one fold tick"
//!   — the index lags the file by at most one `apply()` call.
//!
//! ## What you give up
//!
//! - **Durability.** The index is **in-memory only**. On restart it
//!   rebuilds from the tail of the RedexFile. If you want durable
//!   indices, persist the snapshot separately and rebuild from seq N.
//! - **Cross-key atomicity.** Each `IndexOp` lands under its own
//!   `DashMap` shard lock; two reads on different keys can observe
//!   the mid-way state.
//! - **Strict read-your-writes.** The index is updated on a background
//!   task, so a thread that appends to the file and immediately reads
//!   from the index may miss its own write by one scheduler hop.
//!
//! ## Wire to snapshot / restore
//!
//! CortEX adapters that own a [`RedexIndex`] rebuild it naturally on
//! restore: the snapshot captures the domain state up to seq N; on
//! `open_from_snapshot(..., last_seq)`, the adapter spawns a new
//! tail at `FromSeq(last_seq + 1)` and the index rebuilds from there.
//! The index never rides in the snapshot itself.

use std::collections::HashSet;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

use dashmap::DashMap;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use tokio::sync::Notify;

use super::error::RedexError;
use super::event::RedexEvent;
use super::file::RedexFile;

/// One mutation a [`RedexIndex`] projection can emit per event.
#[derive(Debug, Clone)]
pub enum IndexOp<K, V> {
    /// Add `value` to the bucket at `key`. No-op if already present.
    Insert(K, V),
    /// Remove `value` from the bucket at `key`. No-op if absent.
    /// When the bucket empties, the key is dropped from the index
    /// entirely so `keys()` and `len()` reflect current contents.
    Remove(K, V),
}

/// Where to start tailing when an index is opened.
///
/// Mirrors the shape CortEX uses (`FromBeginning` / `FromSeq`) —
/// `LiveOnly` is intentionally omitted here because a live-only
/// index is almost never what you want: the bucket would only
/// reflect events received after the index was built, with no
/// history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexStart {
    /// Tail from seq 0; rebuilds the index over the entire file.
    FromBeginning,
    /// Tail from `seq`; skips everything below it. Use after
    /// `open_from_snapshot` to resume with only post-snapshot
    /// events.
    FromSeq(u64),
}

/// Eventually-consistent secondary index driven from a RedEX file tail.
///
/// Lock-free on the read side ([`DashMap`]); a single background task
/// owns the write side. Reads can observe the index at most one fold
/// tick behind the file.
pub struct RedexIndex<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Hash + Eq + Clone + Send + Sync + 'static,
{
    inner: Arc<DashMap<K, HashSet<V>>>,
    /// Fired on Drop to cancel the tail task. The task races this
    /// notify against the tail stream; whichever wins wakes first
    /// and the task exits.
    shutdown: Arc<Notify>,
}

impl<K, V> RedexIndex<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Hash + Eq + Clone + Send + Sync + 'static,
{
    /// Open an index over `file`, decoding each event payload as `T`
    /// (via postcard) and applying the caller's projection to emit
    /// zero or more [`IndexOp`]s per event.
    ///
    /// Spawns a tokio task on the current runtime. Dropping all
    /// clones of the returned handle cancels the task via `Notify`.
    /// Payload-decode errors are logged (`tracing::warn!`) and the
    /// offending event is skipped — a corrupt or wrong-type entry
    /// shouldn't poison the whole index.
    pub fn open<T, F>(file: &RedexFile, start: IndexStart, project: F) -> Self
    where
        T: DeserializeOwned + Send + 'static,
        F: Fn(&T) -> Vec<IndexOp<K, V>> + Send + Sync + 'static,
    {
        let inner: Arc<DashMap<K, HashSet<V>>> = Arc::new(DashMap::new());
        let shutdown = Arc::new(Notify::new());

        let from_seq = match start {
            IndexStart::FromBeginning => 0,
            IndexStart::FromSeq(n) => n,
        };
        let task_inner = inner.clone();
        let task_shutdown = shutdown.clone();
        let task_file = file.clone();

        tokio::spawn(async move {
            // Use Box::pin (not tokio::pin!) so we can swap in a
            // fresh tail stream after a Lagged recovery without
            // restructuring the task.
            let mut tail: Pin<
                Box<dyn Stream<Item = Result<RedexEvent, RedexError>> + Send>,
            > = Box::pin(task_file.tail(from_seq));

            loop {
                tokio::select! {
                    // Drop-on-cancel: the Notify wins and the task
                    // exits, dropping the tail stream.
                    _ = task_shutdown.notified() => return,
                    next = tail.next() => {
                        match next {
                            Some(Ok(event)) => apply_event(&task_inner, &project, &event),
                            Some(Err(RedexError::Lagged)) => {
                                // BUG #3: pre-fix, the loop just
                                // logged at debug! and continued,
                                // but the underlying watcher had
                                // been dropped from the file —
                                // `tail.next()` returned `None` on
                                // the next poll and the task
                                // exited permanently with an
                                // incomplete index. Keys missing
                                // post-`Lagged` stayed missing
                                // forever.
                                //
                                // Recovery: clear the in-memory
                                // index (any prior state is now
                                // ambiguous — we don't know which
                                // ops we missed) and re-tail from
                                // `next_seq()`, i.e. live-only.
                                // We do NOT replay retained
                                // history because the buffer that
                                // produced the original `Lagged`
                                // is the same buffer we'd have to
                                // squeeze the retained range
                                // through, which would just
                                // signal `Lagged` again. The
                                // index user is responsible for
                                // sizing the tail buffer; this
                                // path is the safe-recovery
                                // fallback when that sizing is
                                // wrong.
                                let resume_seq = task_file.next_seq();
                                task_inner.clear();
                                tail = Box::pin(task_file.tail(resume_seq));
                                tracing::warn!(
                                    resume_seq,
                                    "RedexIndex: tail lagged; cleared index, resumed live-only"
                                );
                            }
                            Some(Err(e)) => {
                                tracing::debug!(error = %e, "RedexIndex tail error; continuing");
                            }
                            None => {
                                // BUG #3 corollary: stream ended.
                                // Two distinct causes share this
                                // branch:
                                //   (a) the file was closed —
                                //       the task should exit.
                                //   (b) `notify_watchers` dropped
                                //       our watcher under buffer
                                //       saturation and the
                                //       best-effort `Err(Lagged)`
                                //       send itself failed (the
                                //       channel was already full),
                                //       so the receiver just sees
                                //       a clean end. Pre-fix the
                                //       task always treated this
                                //       as (a) and exited — same
                                //       failure mode as the
                                //       explicit Lagged case.
                                // Distinguish via `is_closed()`.
                                if task_file.is_closed() {
                                    return;
                                }
                                let resume_seq = task_file.next_seq();
                                task_inner.clear();
                                tail = Box::pin(task_file.tail(resume_seq));
                                tracing::warn!(
                                    resume_seq,
                                    "RedexIndex: tail stream ended on a still-open file (saturation-induced \
                                     watcher drop); cleared index, resumed live-only"
                                );
                            }
                        }
                    }
                }
            }
        });

        Self { inner, shutdown }
    }

    /// Snapshot the values at `key`. Returns `None` when the key
    /// doesn't exist. The returned set is a **copy** so the caller
    /// can hold it freely without blocking the tail task.
    pub fn get(&self, key: &K) -> Option<HashSet<V>> {
        self.inner.get(key).map(|e| e.value().clone())
    }

    /// Whether `value` is present in the bucket at `key`.
    pub fn contains(&self, key: &K, value: &V) -> bool {
        self.inner
            .get(key)
            .is_some_and(|e| e.value().contains(value))
    }

    /// Snapshot of all keys currently indexed. Allocates — don't call
    /// in a hot loop. Returned order is unspecified.
    pub fn keys(&self) -> Vec<K> {
        self.inner.iter().map(|e| e.key().clone()).collect()
    }

    /// Number of distinct keys currently indexed.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True if no keys are currently indexed.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V> Drop for RedexIndex<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        // `notify_one` stores a permit if no waiter is currently
        // registered, so a Drop that fires while the tail task is
        // between `tokio::select!` polls still lands — the task
        // consumes the permit on its next `notified()` and exits.
        // `notify_waiters` would be dropped in that window and leak
        // the task.
        self.shutdown.notify_one();
    }
}

impl<K, V> std::fmt::Debug for RedexIndex<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedexIndex")
            .field("keys", &self.inner.len())
            .finish()
    }
}

fn apply_event<T, K, V, F>(inner: &DashMap<K, HashSet<V>>, project: &F, event: &RedexEvent)
where
    T: DeserializeOwned,
    K: Hash + Eq + Clone,
    V: Hash + Eq + Clone,
    F: Fn(&T) -> Vec<IndexOp<K, V>>,
{
    let decoded: T = match postcard::from_bytes(&event.payload) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(
                error = %e,
                seq = event.entry.seq,
                "RedexIndex: failed to decode event; skipping",
            );
            return;
        }
    };
    for op in project(&decoded) {
        match op {
            IndexOp::Insert(k, v) => {
                inner.entry(k).or_default().insert(v);
            }
            IndexOp::Remove(k, v) => {
                // Remove the value; drop the whole bucket if it
                // empties so `len()` and `keys()` reflect current
                // contents.
                let mut drop_key = false;
                if let Some(mut set) = inner.get_mut(&k) {
                    set.remove(&v);
                    drop_key = set.is_empty();
                }
                if drop_key {
                    inner.remove(&k);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::channel::ChannelName;
    use crate::adapter::net::redex::config::RedexFileConfig;
    use crate::adapter::net::redex::manager::Redex;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Tagged {
        id: u64,
        tags: Vec<String>,
    }

    fn open_file(name: &str) -> super::RedexFile {
        let r = Redex::new();
        r.open_file(&ChannelName::new(name).unwrap(), RedexFileConfig::default())
            .unwrap()
    }

    async fn yield_a_few() {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn test_index_populates_from_existing_entries() {
        let f = open_file("idx/basic");
        for id in 0..5u64 {
            let ev = Tagged {
                id,
                tags: vec!["even".into()],
            };
            f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
        }
        // Only the "even" tag should be indexed; values are the ids.
        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });
        yield_a_few().await;

        let even = idx.get(&"even".to_string()).expect("even bucket populated");
        assert_eq!(even.len(), 5);
        for i in 0..5u64 {
            assert!(even.contains(&i));
        }
        assert_eq!(idx.keys().len(), 1);
    }

    #[tokio::test]
    async fn test_index_insert_remove_symmetry() {
        let f = open_file("idx/insert_remove");
        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                // Tag starting with "-" means "remove", otherwise insert.
                t.tags
                    .iter()
                    .map(|tag| {
                        if let Some(stripped) = tag.strip_prefix('-') {
                            IndexOp::Remove(stripped.to_string(), t.id)
                        } else {
                            IndexOp::Insert(tag.clone(), t.id)
                        }
                    })
                    .collect()
            });

        let add = Tagged {
            id: 1,
            tags: vec!["k".into()],
        };
        let remove = Tagged {
            id: 1,
            tags: vec!["-k".into()],
        };
        f.append(&postcard::to_allocvec(&add).unwrap()).unwrap();
        yield_a_few().await;
        assert!(idx.contains(&"k".to_string(), &1u64));

        f.append(&postcard::to_allocvec(&remove).unwrap()).unwrap();
        yield_a_few().await;
        // Key should be GONE once the bucket empties — not an empty
        // set lying around inflating `keys()`.
        assert!(idx.get(&"k".to_string()).is_none());
        assert_eq!(idx.len(), 0);
    }

    #[tokio::test]
    async fn test_index_multiple_ops_per_event() {
        let f = open_file("idx/multiop");
        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });

        let ev = Tagged {
            id: 42,
            tags: vec!["alpha".into(), "beta".into(), "gamma".into()],
        };
        f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
        yield_a_few().await;

        for tag in ["alpha", "beta", "gamma"] {
            assert!(idx.contains(&tag.to_string(), &42u64));
        }
        assert_eq!(idx.len(), 3);
    }

    #[tokio::test]
    async fn test_index_from_seq_skips_earlier_events() {
        let f = open_file("idx/fromseq");
        for id in 0..4u64 {
            let ev = Tagged {
                id,
                tags: vec!["t".into()],
            };
            f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
        }
        // Resume from seq=2 → should index only ids 2 and 3.
        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromSeq(2), |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });
        yield_a_few().await;

        let bucket = idx.get(&"t".to_string()).unwrap();
        assert_eq!(bucket.len(), 2);
        assert!(bucket.contains(&2));
        assert!(bucket.contains(&3));
        assert!(!bucket.contains(&0));
        assert!(!bucket.contains(&1));
    }

    #[tokio::test]
    async fn test_index_decode_error_skips_entry() {
        // Appending garbage bytes that don't deserialize as `Tagged`
        // must not poison the index — the bad event is skipped,
        // subsequent valid events are applied.
        let f = open_file("idx/decode_err");
        f.append(b"\xFF\xFF\xFF\xFF").unwrap();

        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });

        let good = Tagged {
            id: 7,
            tags: vec!["x".into()],
        };
        f.append(&postcard::to_allocvec(&good).unwrap()).unwrap();
        yield_a_few().await;

        assert!(idx.contains(&"x".to_string(), &7u64));
        assert_eq!(idx.len(), 1);
    }

    /// BUG #3: a `Lagged` from the underlying tail must NOT
    /// permanently halt the index task. Pre-fix the loop logged at
    /// debug! and continued, but the file had already dropped the
    /// watcher — the next `tail.next()` returned `None` and the
    /// task exited with the index frozen at whatever state it had
    /// when the lag fired. Keys appended later never appeared.
    ///
    /// Post-fix: on `Lagged` (or on a stream `None` arriving while
    /// the file is still open — saturation-induced watcher drop),
    /// the task clears the in-memory index and re-tails from
    /// `next_seq()` (live-only). Live events from that point
    /// forward populate the index again.
    ///
    /// The test paces the post-lag appends with a yield between
    /// each so the recovered tail's small buffer doesn't itself
    /// saturate; absent that pacing, the buffer-2 watcher would
    /// drop again and the index would clear a second time, which
    /// is also correct but not what we're trying to verify.
    #[tokio::test]
    async fn index_recovers_from_tail_lag_and_continues_indexing() {
        // Tiny tail buffer so the backfill saturates immediately
        // when we open the index against a file with many events.
        let r = Redex::new();
        let f = r
            .open_file(
                &ChannelName::new("idx/lag-recovery").unwrap(),
                RedexFileConfig::default().with_tail_buffer_size(2),
            )
            .unwrap();

        // Pre-load the file with more events than the tail buffer
        // can hold during backfill — this triggers Lagged at
        // open-time.
        for id in 0..10u64 {
            let ev = Tagged {
                id,
                tags: vec!["pre".into()],
            };
            f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
        }

        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });

        // Let the task observe Lagged and re-tail from live.
        yield_a_few().await;

        // Append new events one at a time with a yield between
        // each so the recovered watcher (also buffer=2) drains
        // before the next event arrives.
        for id in 100..105u64 {
            let ev = Tagged {
                id,
                tags: vec!["post".into()],
            };
            f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
            yield_a_few().await;
        }

        // The index must reflect every post-lag event. Pre-fix the
        // task had already exited and `idx.get` would return
        // `None`.
        let post_keys = idx
            .get(&"post".to_string())
            .expect(
                "post-lag bucket missing — pre-fix the index task halted \
                 permanently after Lagged and never observed these events (BUG #3)",
            );
        assert_eq!(
            post_keys.len(),
            5,
            "every post-lag event must be indexed; recovered set was {:?}",
            post_keys
        );
        for id in 100..105u64 {
            assert!(
                post_keys.contains(&id),
                "post-lag id {} missing from recovered index",
                id
            );
        }
    }

    /// BUG #3 corollary: closing the file naturally must still
    /// terminate the index task (no infinite re-tail loop).
    #[tokio::test]
    async fn index_terminates_when_file_closes() {
        let r = Redex::new();
        let f = r
            .open_file(
                &ChannelName::new("idx/close-terminates").unwrap(),
                RedexFileConfig::default(),
            )
            .unwrap();

        let idx: RedexIndex<String, u64> =
            RedexIndex::open::<Tagged, _>(&f, IndexStart::FromBeginning, |t| {
                t.tags
                    .iter()
                    .map(|tag| IndexOp::Insert(tag.clone(), t.id))
                    .collect()
            });

        let ev = Tagged {
            id: 1,
            tags: vec!["a".into()],
        };
        f.append(&postcard::to_allocvec(&ev).unwrap()).unwrap();
        yield_a_few().await;
        assert!(idx.contains(&"a".to_string(), &1));

        // Close should propagate as Err(Closed) on the tail; the
        // task must NOT treat it as a saturation event and re-tail.
        f.close().unwrap();
        yield_a_few().await;
        // Hard to assert "task exited" directly without exposing
        // the JoinHandle, but if the task were re-tailing in a
        // loop on Err(Closed) → re-tail → Err(Closed) → ... the
        // CPU would spin and the test would still pass. The real
        // assertion is that we don't deadlock here — close returned,
        // and the file's drop semantics are unchanged.
        assert!(f.is_closed());
    }
}
