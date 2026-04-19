//! Retention policy for RedEX (count + size + age).
//!
//! This module is pure logic: given the current index state, the
//! per-entry timestamps, and a config, it returns the number of head
//! entries to evict. The actual segment rewrite happens in
//! `RedexFile::sweep_retention`.

use super::config::RedexFileConfig;
use super::entry::{RedexEntry, INLINE_PAYLOAD_SIZE};

/// Compute how many head entries to drop so that the retained tail
/// satisfies every active retention policy (`retention_max_events`,
/// `retention_max_bytes`, `retention_max_age_ns`). When multiple are
/// set, the policies AND together — we take the largest drop count
/// so that ALL constraints are satisfied.
///
/// `entries` is the current index in seq order. `timestamps` is a
/// parallel slice of unix-nanos timestamps captured at append time
/// (same length as `entries`). `now_ns` is the current wall clock in
/// unix nanos. Returns a count such that `entries[count..]` is the
/// post-sweep tail.
pub(crate) fn compute_eviction_count(
    entries: &[RedexEntry],
    timestamps: &[u64],
    now_ns: u64,
    cfg: &RedexFileConfig,
) -> usize {
    debug_assert_eq!(
        entries.len(),
        timestamps.len(),
        "timestamps must parallel entries"
    );

    let mut drop = 0usize;

    // Count-based.
    if let Some(max_events) = cfg.retention_max_events {
        let len = entries.len() as u64;
        if len > max_events {
            drop = (len - max_events) as usize;
        }
    }

    // Size-based: walk from the tail back, accumulating bytes; anything
    // beyond the cap is dropped. `idx` here is the forward index — the
    // entry at `entries[idx]`. When it doesn't fit, we drop [0..idx+1).
    if let Some(max_bytes) = cfg.retention_max_bytes {
        let mut retained_bytes: u64 = 0;
        for (idx, e) in entries.iter().enumerate().rev() {
            let size = entry_payload_size(e);
            if retained_bytes + size > max_bytes {
                drop = drop.max(idx + 1);
                break;
            }
            retained_bytes += size;
        }
        // If we exited the loop without breaking, all entries fit —
        // size policy drops none.
    }

    // Age-based: drop entries whose timestamp is older than the cutoff.
    // Timestamps are ~monotonic (wall clock) within a single process,
    // so the first entry with `ts > cutoff` marks the retained head.
    if let Some(max_age_ns) = cfg.retention_max_age_ns {
        let cutoff = now_ns.saturating_sub(max_age_ns);
        let mut age_drop = 0;
        for &ts in timestamps.iter() {
            if ts > cutoff {
                break;
            }
            age_drop += 1;
        }
        drop = drop.max(age_drop);
    }

    drop
}

#[inline]
fn entry_payload_size(e: &RedexEntry) -> u64 {
    if e.is_inline() {
        INLINE_PAYLOAD_SIZE as u64
    } else {
        e.payload_len as u64
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::entry::RedexEntry;
    use super::*;

    fn heap_entries(count: usize, each_size: u32) -> Vec<RedexEntry> {
        (0..count)
            .map(|i| RedexEntry::new_heap(i as u64, (i as u32) * each_size, each_size, 0, 0))
            .collect()
    }

    /// Parallel timestamps — all "now" for count/size tests that
    /// don't exercise the age policy.
    fn dummy_timestamps(count: usize) -> Vec<u64> {
        vec![0; count]
    }

    #[test]
    fn test_no_retention_drops_nothing() {
        let entries = heap_entries(100, 16);
        let ts = dummy_timestamps(100);
        let cfg = RedexFileConfig::default();
        assert_eq!(compute_eviction_count(&entries, &ts, 0, &cfg), 0);
    }

    #[test]
    fn test_count_retention() {
        let entries = heap_entries(100, 16);
        let ts = dummy_timestamps(100);
        let cfg = RedexFileConfig::default().with_retention_max_events(40);
        assert_eq!(compute_eviction_count(&entries, &ts, 0, &cfg), 60);
    }

    #[test]
    fn test_size_retention() {
        let entries = heap_entries(100, 16);
        let ts = dummy_timestamps(100);
        let cfg = RedexFileConfig::default().with_retention_max_bytes(480);
        assert_eq!(compute_eviction_count(&entries, &ts, 0, &cfg), 70);
    }

    #[test]
    fn test_both_count_and_size_takes_larger_drop() {
        let entries = heap_entries(100, 16);
        let ts = dummy_timestamps(100);
        let cfg = RedexFileConfig::default()
            .with_retention_max_events(40)
            .with_retention_max_bytes(480);
        assert_eq!(compute_eviction_count(&entries, &ts, 0, &cfg), 70);
    }

    #[test]
    fn test_empty_index() {
        let cfg = RedexFileConfig::default().with_retention_max_events(10);
        assert_eq!(compute_eviction_count(&[], &[], 0, &cfg), 0);
    }

    // ---- Age policy ----

    fn ts_seq(count: usize, step_ns: u64) -> Vec<u64> {
        (0..count as u64).map(|i| i * step_ns + 1_000).collect()
    }

    #[test]
    fn test_age_retention_drops_older_than_cutoff() {
        // 10 entries, 1 ns apart at t=1000..1009.
        // max_age = 5 ns, now = 1009 → cutoff = 1004. Drop entries
        // with ts <= 1004: ts 1000..=1004 = 5 entries.
        let entries = heap_entries(10, 16);
        let ts = ts_seq(10, 1);
        let cfg = RedexFileConfig::default().with_retention_max_age(Duration::from_nanos(5));
        assert_eq!(compute_eviction_count(&entries, &ts, 1009, &cfg), 5);
    }

    #[test]
    fn test_age_retention_no_drops_when_all_young() {
        let entries = heap_entries(5, 16);
        // timestamps just below now; max_age large → nothing expires.
        let ts = vec![100, 101, 102, 103, 104];
        let cfg = RedexFileConfig::default().with_retention_max_age(Duration::from_nanos(1000));
        assert_eq!(compute_eviction_count(&entries, &ts, 105, &cfg), 0);
    }

    #[test]
    fn test_age_retention_drops_all_when_all_old() {
        let entries = heap_entries(5, 16);
        let ts = vec![1, 2, 3, 4, 5];
        let cfg = RedexFileConfig::default().with_retention_max_age(Duration::from_nanos(1));
        // cutoff = now - 1 = 99; all ts <= 99 → drop all 5.
        assert_eq!(compute_eviction_count(&entries, &ts, 100, &cfg), 5);
    }

    #[test]
    fn test_age_retention_now_before_any_timestamp_drops_nothing() {
        // Weird edge case: clock skew or tests with small now.
        // All timestamps > now → cutoff = 0 (saturating_sub) → nothing
        // drops.
        let entries = heap_entries(3, 16);
        let ts = vec![1000, 2000, 3000];
        let cfg = RedexFileConfig::default().with_retention_max_age(Duration::from_nanos(100));
        assert_eq!(compute_eviction_count(&entries, &ts, 50, &cfg), 0);
    }

    #[test]
    fn test_combined_count_and_age_takes_larger_drop() {
        // Count says drop 3 (keep newest 2 of 5). Age says drop 4
        // (only newest 1 fits under 2 ns cutoff). Final drop = 4.
        let entries = heap_entries(5, 16);
        let ts = vec![100, 101, 102, 103, 104];
        let cfg = RedexFileConfig::default()
            .with_retention_max_events(2)
            .with_retention_max_age(Duration::from_nanos(2));
        // now=104, cutoff=102 → ts 100,101,102 → 3 old; ts 103 stops.
        // Wait — ts > cutoff is the condition to STOP. So 100,101,102 all <= 102 → 3 drops.
        // Count says drop 3 too. Max is 3.
        assert_eq!(compute_eviction_count(&entries, &ts, 104, &cfg), 3);
    }

    #[test]
    fn test_combined_age_larger_than_count() {
        // 10 entries with small old timestamps. Count says drop 5,
        // age says drop 10. Max = 10 wins.
        let entries = heap_entries(10, 16);
        let ts = vec![1u64; 10];
        let cfg = RedexFileConfig::default()
            .with_retention_max_events(5)
            .with_retention_max_age(Duration::from_nanos(1));
        // now=1000, cutoff=999. All ts = 1 <= 999 → drop 10.
        assert_eq!(compute_eviction_count(&entries, &ts, 1000, &cfg), 10);
    }
}
