//! Retention policy for RedEX v1 (count + size).
//!
//! This module is pure logic: given the current index state and a
//! config, it returns the number of head entries to evict. The actual
//! segment rewrite happens in `RedexFile::sweep_retention`.

use super::config::RedexFileConfig;
use super::entry::{RedexEntry, INLINE_PAYLOAD_SIZE};

/// Compute how many head entries to drop so that the retained tail
/// satisfies `retention_max_events` and `retention_max_bytes`.
///
/// `entries` is the current index in seq order. Returns a count such
/// that `entries[count..]` is the post-sweep tail.
pub(crate) fn compute_eviction_count(entries: &[RedexEntry], cfg: &RedexFileConfig) -> usize {
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
                let size_drop = idx + 1;
                return drop.max(size_drop);
            }
            retained_bytes += size;
        }
        // All entries fit under the size cap — size policy drops none.
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
    use super::super::entry::RedexEntry;
    use super::*;

    fn heap_entries(count: usize, each_size: u32) -> Vec<RedexEntry> {
        (0..count)
            .map(|i| RedexEntry::new_heap(i as u64, (i as u32) * each_size, each_size, 0, 0))
            .collect()
    }

    #[test]
    fn test_no_retention_drops_nothing() {
        let entries = heap_entries(100, 16);
        let cfg = RedexFileConfig::default();
        assert_eq!(compute_eviction_count(&entries, &cfg), 0);
    }

    #[test]
    fn test_count_retention() {
        let entries = heap_entries(100, 16);
        let cfg = RedexFileConfig::default().with_retention_max_events(40);
        assert_eq!(compute_eviction_count(&entries, &cfg), 60);
    }

    #[test]
    fn test_size_retention() {
        // 100 entries × 16 bytes = 1600 bytes. Cap at 480 → keep 30.
        let entries = heap_entries(100, 16);
        let cfg = RedexFileConfig::default().with_retention_max_bytes(480);
        assert_eq!(compute_eviction_count(&entries, &cfg), 70);
    }

    #[test]
    fn test_both_takes_larger_drop() {
        let entries = heap_entries(100, 16);
        // Count says drop 60, size says drop 70 → 70 wins.
        let cfg = RedexFileConfig::default()
            .with_retention_max_events(40)
            .with_retention_max_bytes(480);
        assert_eq!(compute_eviction_count(&entries, &cfg), 70);
    }

    #[test]
    fn test_empty_index() {
        let cfg = RedexFileConfig::default().with_retention_max_events(10);
        assert_eq!(compute_eviction_count(&[], &cfg), 0);
    }
}
