//! Disk-backed durability segment for persistent `RedexFile`s.
//!
//! Feature-gated behind `redex-disk`. When `RedexFileConfig::persistent`
//! is set and the owning [`super::Redex`] manager was given a
//! persistent directory, appends are mirrored to two append-only files
//! per channel:
//!
//! - `<base>/<channel_path>/idx` — 20-byte [`RedexEntry`] records.
//! - `<base>/<channel_path>/dat` — payload bytes (offsets match the
//!   in-memory [`super::segment::HeapSegment`]).
//!
//! The heap segment remains authoritative during normal operation; the
//! disk files exist for crash recovery. On reopen the full `dat` file
//! is replayed into the heap, so retention is in-memory-only in v1
//! (the disk files grow unbounded; operators delete old files manually
//! between runs when that matters). v2 will reconcile this.
//!
//! Durability policy:
//!
//! - Append-path fsync is governed by [`super::FsyncPolicy`], threaded
//!   in as `fsync_every_n` at open time. `0` disables append-side
//!   syncing; a positive value triggers a fsync every `n`th append.
//! - `close()` always fsyncs both files, regardless of policy.
//! - Explicit `super::RedexFile::sync()` always fsyncs both files.
//! - Order matters inside [`DiskSegment::sync`]: dat fsyncs before
//!   idx so a crash can only leave the index shorter than dat,
//!   which the reopen-time truncation handles.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;

use super::super::channel::ChannelName;
use super::entry::{RedexEntry, REDEX_ENTRY_SIZE};
use super::error::RedexError;

/// Result of opening a persistent segment. Carries both file handles
/// and the index state recovered from disk.
pub(super) struct RecoveredSegment {
    pub disk: DiskSegment,
    pub index: Vec<RedexEntry>,
    pub payload_bytes: Vec<u8>,
}

/// Disk-backed durability segment: append-only idx + dat files.
pub(super) struct DiskSegment {
    /// Full path to the per-channel directory. Kept for diagnostics.
    #[allow(dead_code)]
    dir: PathBuf,
    idx_file: Mutex<File>,
    dat_file: Mutex<File>,
    /// Append-path fsync interval: after `fsync_every_n` successful
    /// appends, the segment invokes `sync()` itself. `0` disables
    /// append-side syncing (Never or Interval policies — the latter
    /// is driven externally by a per-file background task).
    fsync_every_n: u64,
    /// Appends since the last append-driven sync (successful or not).
    /// Only meaningful when `fsync_every_n > 0`.
    appends_since_sync: AtomicU64,
    /// Test-only injection: when set, the next `append_entry` /
    /// `append_entries` call returns `RedexError::Io` before touching
    /// either file. Exercises the caller's rollback paths without
    /// needing a real I/O failure (disk full, permission denied).
    #[cfg(test)]
    fail_next_append: AtomicBool,
    /// Test-only counter: cumulative successful `sync()` calls —
    /// close-time, append-driven (EveryN), or external
    /// (Interval / explicit). Lets policy tests assert the observed
    /// fsync cadence without racing real I/O.
    #[cfg(test)]
    sync_count: AtomicU64,
}

impl DiskSegment {
    /// Open (or create) the idx + dat files for `name` under `base_dir`
    /// and recover the index from disk.
    ///
    /// `fsync_every_n` is derived from [`super::FsyncPolicy`] at the
    /// `RedexFile` layer: `EveryN(n)` forwards `n`; `Never` and
    /// `Interval` (handled by the per-file background task) both
    /// forward `0`, disabling append-side syncs on this segment.
    pub(super) fn open(
        base_dir: &Path,
        name: &ChannelName,
        fsync_every_n: u64,
    ) -> Result<RecoveredSegment, RedexError> {
        let dir = channel_dir(base_dir, name);
        std::fs::create_dir_all(&dir).map_err(RedexError::io)?;
        let idx_path = dir.join("idx");
        let dat_path = dir.join("dat");

        // Recover existing index.
        let (mut index, idx_len_truncated) = read_index(&idx_path)?;
        let mut payload_bytes = read_payload(&dat_path)?;

        // Torn-idx tail: the last 20-byte write was partial (crash
        // mid-append). Truncate idx to a whole multiple of 20 bytes.
        if idx_len_truncated {
            let file = OpenOptions::new()
                .write(true)
                .open(&idx_path)
                .map_err(RedexError::io)?;
            file.set_len((index.len() * REDEX_ENTRY_SIZE) as u64)
                .map_err(RedexError::io)?;
        }

        // Torn-dat tail: our write ordering is dat-before-idx, so a
        // crash between the two writes leaves dat shorter than the
        // last idx entry thinks it should be. Separately, external
        // truncation (disk corruption, filesystem bug, admin action)
        // can shrink dat past ANY heap entry, not just the tail.
        //
        // Walk the index backward, skipping inline entries (their
        // payload rides inside the 20-byte idx record and doesn't
        // reference dat). Track the earliest heap entry whose
        // `(offset + len)` runs past the actual dat size — because
        // dat is append-only, heap offsets are monotonic, so if an
        // entry at position `i` is torn then every heap entry at
        // positions `>= i` is either torn or a later append that
        // never got its dat write. Drop everything from that point
        // onward.
        let dat_len = payload_bytes.len() as u64;
        let mut truncate_at: Option<usize> = None;
        for (i, e) in index.iter().enumerate().rev() {
            if e.is_inline() {
                // Inline entries are always valid regardless of dat
                // state. Keep walking back to check earlier heap
                // entries.
                continue;
            }
            let end = (e.payload_offset as u64).saturating_add(e.payload_len as u64);
            if end > dat_len {
                // Torn. Everything from here to the end of the index
                // must go. Record this position and keep walking —
                // an even earlier heap entry might also be torn
                // (external truncation scenarios).
                truncate_at = Some(i);
            } else {
                // First heap entry that fits. By dat's append-only
                // monotonicity, every earlier heap entry also fits.
                break;
            }
        }
        let idx_trimmed = truncate_at.is_some();
        if let Some(cut) = truncate_at {
            index.truncate(cut);
        }
        if idx_trimmed {
            let file = OpenOptions::new()
                .write(true)
                .open(&idx_path)
                .map_err(RedexError::io)?;
            file.set_len((index.len() * REDEX_ENTRY_SIZE) as u64)
                .map_err(RedexError::io)?;
        }
        // Trim any trailing dat bytes that no idx entry references.
        // Finds the highest `(offset + len)` among retained heap
        // entries and truncates dat to that.
        let retained_dat_end = index
            .iter()
            .filter(|e| !e.is_inline())
            .map(|e| (e.payload_offset as u64).saturating_add(e.payload_len as u64))
            .max()
            .unwrap_or(0);
        if retained_dat_end < dat_len {
            let file = OpenOptions::new()
                .write(true)
                .open(&dat_path)
                .map_err(RedexError::io)?;
            file.set_len(retained_dat_end).map_err(RedexError::io)?;
            payload_bytes.truncate(retained_dat_end as usize);
        }

        // Verify per-entry checksums during recovery. Without
        // this check, on-disk corruption (torn writes, bit-rot,
        // FS bug, or external tampering) is silently accepted
        // and becomes part of the recovered state. Drop entries
        // whose checksum doesn't match. Inline entries are
        // self-contained 8-byte payloads carried inside the
        // index record; we still verify them in case the index
        // file itself was corrupted.
        let mut bad_entries = 0usize;
        index.retain(|e| {
            let payload: &[u8] = if e.is_inline() {
                let Some(inline) = e.inline_payload() else {
                    bad_entries += 1;
                    return false;
                };
                let computed = super::entry::payload_checksum(&inline);
                if e.checksum() != computed {
                    bad_entries += 1;
                    return false;
                }
                return true;
            } else {
                let off = e.payload_offset as usize;
                let len = e.payload_len as usize;
                let end = off.saturating_add(len);
                if end > payload_bytes.len() {
                    // Should be impossible after the truncation
                    // pass above, but stay defensive.
                    bad_entries += 1;
                    return false;
                }
                &payload_bytes[off..end]
            };
            let computed = super::entry::payload_checksum(payload);
            if e.checksum() != computed {
                bad_entries += 1;
                false
            } else {
                true
            }
        });
        if bad_entries > 0 {
            tracing::error!(
                bad_entries,
                surviving = index.len(),
                "DiskSegment::open: dropped {} entries with bad checksums during recovery; \
                 on-disk dat may have torn writes or be corrupt",
                bad_entries
            );
        }

        let idx_file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&idx_path)
            .map_err(RedexError::io)?;
        let dat_file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&dat_path)
            .map_err(RedexError::io)?;

        Ok(RecoveredSegment {
            disk: DiskSegment {
                dir,
                idx_file: Mutex::new(idx_file),
                dat_file: Mutex::new(dat_file),
                fsync_every_n,
                appends_since_sync: AtomicU64::new(0),
                #[cfg(test)]
                fail_next_append: AtomicBool::new(false),
                #[cfg(test)]
                sync_count: AtomicU64::new(0),
            },
            index,
            payload_bytes,
        })
    }

    /// Test-only: cumulative successful `sync()` count.
    #[cfg(test)]
    pub(super) fn sync_count(&self) -> u64 {
        self.sync_count.load(Ordering::Acquire)
    }

    /// Bump the per-append counter and, if `fsync_every_n` is set,
    /// call `sync()` when the counter reaches it. Counter resets after
    /// each sync so the cadence stays periodic rather than
    /// exponentially triggered. An fsync error at this boundary is
    /// logged but **not** propagated — the caller's append already
    /// succeeded in the page cache, and a blocked-fsync error here
    /// would surface as a spurious append failure. Explicit
    /// `sync()` / `close()` still surface errors.
    fn maybe_sync_after_append(&self, applied: u64) {
        if self.fsync_every_n == 0 || applied == 0 {
            return;
        }
        let prev = self.appends_since_sync.fetch_add(applied, Ordering::AcqRel);
        let now = prev.saturating_add(applied);
        if now < self.fsync_every_n {
            return;
        }
        // Reset unconditionally before syncing so a concurrent appender
        // sees a clean counter even if sync is slow.
        self.appends_since_sync.store(0, Ordering::Release);
        if let Err(e) = self.sync() {
            tracing::warn!(error = %e, "EveryN fsync failed; tail may be unsynced");
        }
    }

    /// Test-only: arm a one-shot failure on the next
    /// `append_entry` / `append_entries` call. Returns `Io` before
    /// touching either file. Used to exercise the caller's rollback
    /// paths without needing a real I/O failure.
    #[cfg(test)]
    pub(super) fn arm_next_append_failure(&self) {
        self.fail_next_append.store(true, Ordering::Release);
    }

    /// Append an entry and (for heap entries) its payload to disk.
    /// Inline entries skip the dat write — their payload rides in the
    /// 20-byte idx record.
    ///
    /// Writes go through the OS page cache. Append-time fsync is
    /// governed by [`super::FsyncPolicy`] via `fsync_every_n`:
    /// `Never` / `Interval` skip it here entirely; `EveryN(n)`
    /// triggers a sync every `n`th successful append. Explicit
    /// `sync()` / `close()` still fsync regardless of policy.
    pub(super) fn append_entry(
        &self,
        entry: &RedexEntry,
        payload: &[u8],
    ) -> Result<(), RedexError> {
        #[cfg(test)]
        if self.fail_next_append.swap(false, Ordering::AcqRel) {
            return Err(RedexError::Io("test-injected append failure".into()));
        }
        if !entry.is_inline() {
            let mut dat = self.dat_file.lock();
            dat.write_all(payload).map_err(RedexError::io)?;
        }
        let mut idx = self.idx_file.lock();
        idx.write_all(&entry.to_bytes()).map_err(RedexError::io)?;
        drop(idx);
        self.maybe_sync_after_append(1);
        Ok(())
    }

    /// Append several entries and their payloads atomically (per-file:
    /// each file's buffered write is contiguous). Inline entries only
    /// touch the idx file.
    ///
    /// Same fsync semantics as [`Self::append_entry`] — a batch of N
    /// counts as N applied appends against the `EveryN` cadence.
    pub(super) fn append_entries(
        &self,
        entries_and_payloads: &[(RedexEntry, &[u8])],
    ) -> Result<(), RedexError> {
        #[cfg(test)]
        if self.fail_next_append.swap(false, Ordering::AcqRel) {
            return Err(RedexError::Io("test-injected append failure".into()));
        }
        let mut dat = self.dat_file.lock();
        for (entry, payload) in entries_and_payloads {
            if !entry.is_inline() {
                dat.write_all(payload).map_err(RedexError::io)?;
            }
        }
        drop(dat);

        let mut idx = self.idx_file.lock();
        for (entry, _) in entries_and_payloads {
            idx.write_all(&entry.to_bytes()).map_err(RedexError::io)?;
        }
        drop(idx);
        self.maybe_sync_after_append(entries_and_payloads.len() as u64);
        Ok(())
    }

    /// Flush both files to durable storage. Order matters for crash
    /// consistency: the payload (`dat`) must be durable before the
    /// index entry (`idx`) that references it. A crash between the
    /// two syncs with the old order could leave an index entry
    /// pointing at bytes that were never flushed — on recovery the
    /// index would reference torn payload data. With dat-first the
    /// worst case is an index that's one or more entries shorter
    /// than the dat, which the torn-tail truncation logic on reopen
    /// already handles correctly.
    pub(super) fn sync(&self) -> Result<(), RedexError> {
        self.dat_file.lock().sync_all().map_err(RedexError::io)?;
        self.idx_file.lock().sync_all().map_err(RedexError::io)?;
        #[cfg(test)]
        self.sync_count.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
}

fn channel_dir(base_dir: &Path, name: &ChannelName) -> PathBuf {
    let mut p = base_dir.to_path_buf();
    for seg in name.as_str().split('/') {
        p.push(seg);
    }
    p
}

/// Read the full idx file into a `Vec<RedexEntry>`.
///
/// Returns `(entries, truncated)` where `truncated` is true if the
/// tail of the file was a partial record (torn write from a crash).
/// Callers should `set_len` the file to `entries.len() * 20` if so.
fn read_index(path: &Path) -> Result<(Vec<RedexEntry>, bool), RedexError> {
    if !path.exists() {
        return Ok((Vec::new(), false));
    }
    let mut f = File::open(path).map_err(RedexError::io)?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).map_err(RedexError::io)?;
    let full_records = bytes.len() / REDEX_ENTRY_SIZE;
    let truncated = bytes.len() % REDEX_ENTRY_SIZE != 0;
    let mut entries = Vec::with_capacity(full_records);
    for i in 0..full_records {
        let start = i * REDEX_ENTRY_SIZE;
        let chunk: [u8; REDEX_ENTRY_SIZE] = bytes[start..start + REDEX_ENTRY_SIZE]
            .try_into()
            .expect("20-byte chunk");
        entries.push(RedexEntry::from_bytes(&chunk));
    }
    Ok((entries, truncated))
}

/// Read the full dat file into a byte vector.
fn read_payload(path: &Path) -> Result<Vec<u8>, RedexError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut f = File::open(path).map_err(RedexError::io)?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).map_err(RedexError::io)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::super::entry::payload_checksum;
    use super::*;

    fn tmpdir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "redex_disk_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn cleanup(p: &Path) {
        let _ = std::fs::remove_dir_all(p);
    }

    #[test]
    fn test_disk_append_and_recover() {
        let base = tmpdir();
        let name = ChannelName::new("t/disk1").unwrap();

        {
            let recovered = DiskSegment::open(&base, &name, 0).unwrap();
            assert!(recovered.index.is_empty());
            assert!(recovered.payload_bytes.is_empty());

            // Simulate writing two heap entries.
            let p1 = b"alpha";
            let e1 = RedexEntry::new_heap(0, 0, p1.len() as u32, 0, payload_checksum(p1));
            recovered.disk.append_entry(&e1, p1).unwrap();

            let p2 = b"beta";
            let e2 = RedexEntry::new_heap(1, 5, p2.len() as u32, 0, payload_checksum(p2));
            recovered.disk.append_entry(&e2, p2).unwrap();

            recovered.disk.sync().unwrap();
        }

        // Reopen; recover both entries and their payload.
        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        assert_eq!(recovered.index.len(), 2);
        assert_eq!(recovered.index[0].seq, 0);
        assert_eq!(recovered.index[1].seq, 1);
        assert_eq!(&recovered.payload_bytes[..5], b"alpha");
        assert_eq!(&recovered.payload_bytes[5..9], b"beta");

        cleanup(&base);
    }

    #[test]
    fn test_disk_inline_entries_skip_dat_file() {
        let base = tmpdir();
        let name = ChannelName::new("t/inline").unwrap();

        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        let payload = *b"abcdefgh";
        let entry = RedexEntry::new_inline(0, &payload, payload_checksum(&payload));
        recovered.disk.append_entry(&entry, &payload).unwrap();
        recovered.disk.sync().unwrap();
        drop(recovered);

        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        assert_eq!(recovered.index.len(), 1);
        assert!(recovered.index[0].is_inline());
        // Dat file should be empty — inline payload lives in idx.
        assert!(recovered.payload_bytes.is_empty());

        cleanup(&base);
    }

    #[test]
    fn test_torn_idx_tail_is_truncated_on_reopen() {
        let base = tmpdir();
        let name = ChannelName::new("t/torn").unwrap();

        // Write one good entry.
        {
            let recovered = DiskSegment::open(&base, &name, 0).unwrap();
            let p = b"ok";
            let e = RedexEntry::new_heap(0, 0, p.len() as u32, 0, payload_checksum(p));
            recovered.disk.append_entry(&e, p).unwrap();
            recovered.disk.sync().unwrap();
        }

        // Manually append 7 garbage bytes to simulate a torn write.
        let idx_path = channel_dir(&base, &name).join("idx");
        let mut f = OpenOptions::new().append(true).open(&idx_path).unwrap();
        f.write_all(&[0xFF; 7]).unwrap();
        f.sync_all().unwrap();
        drop(f);

        // Reopen: partial tail must be truncated; one entry recovered.
        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        assert_eq!(recovered.index.len(), 1);
        assert_eq!(recovered.index[0].seq, 0);

        // Verify the file was actually truncated back to 20 bytes.
        let after_len = std::fs::metadata(&idx_path).unwrap().len();
        assert_eq!(after_len, 20);

        cleanup(&base);
    }

    #[test]
    fn test_channel_dir_handles_nested_names() {
        let base = PathBuf::from("/tmp/base");
        let name = ChannelName::new("sensors/lidar/front").unwrap();
        let dir = channel_dir(&base, &name);
        assert_eq!(dir, PathBuf::from("/tmp/base/sensors/lidar/front"));
    }

    /// Externally truncating the dat file (admin action, FS bug,
    /// crash mid-rename) past a heap entry's tail must drop only
    /// the torn entries — every preceding heap entry AND every
    /// inline entry between them must survive. The recovery walk
    /// at lines 127-160 documents this scenario; this test pins
    /// the documented behavior so a refactor can't quietly trim
    /// too much (data loss) or too little (stale offset → garbage
    /// reads). Lay-out under test:
    ///
    ///     idx: [heap1, inline1, heap2, inline2, heap3]
    ///     dat: [<h1 bytes>, <h2 bytes>, <h3 bytes>]
    ///
    /// Truncate dat to keep h1 and h2 but kill h3. After reopen,
    /// the surviving index must be `[heap1, inline1, heap2,
    /// inline2]`.
    #[test]
    fn test_external_dat_truncation_drops_torn_heap_after_inlines() {
        let base = tmpdir();
        let name = ChannelName::new("t/external-trunc").unwrap();

        // Inline payloads must be exactly INLINE_PAYLOAD_SIZE bytes.
        let inline_a = *b"in_a____";
        let inline_b = *b"in_b____";
        let h1_payload = b"heap1";
        let h2_payload = b"heap2_longer";
        let h3_payload = b"heap3_data";

        let h1_off = 0u32;
        let h2_off = h1_off + h1_payload.len() as u32;
        let h3_off = h2_off + h2_payload.len() as u32;
        let dat_keep_len = (h2_off + h2_payload.len() as u32) as u64;

        // Phase 1 — write the layout.
        {
            let recovered = DiskSegment::open(&base, &name, 0).unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(
                        0,
                        h1_off,
                        h1_payload.len() as u32,
                        0,
                        payload_checksum(h1_payload),
                    ),
                    h1_payload,
                )
                .unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_inline(1, &inline_a, payload_checksum(&inline_a)),
                    &inline_a,
                )
                .unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(
                        2,
                        h2_off,
                        h2_payload.len() as u32,
                        0,
                        payload_checksum(h2_payload),
                    ),
                    h2_payload,
                )
                .unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_inline(3, &inline_b, payload_checksum(&inline_b)),
                    &inline_b,
                )
                .unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(
                        4,
                        h3_off,
                        h3_payload.len() as u32,
                        0,
                        payload_checksum(h3_payload),
                    ),
                    h3_payload,
                )
                .unwrap();

            recovered.disk.sync().unwrap();
        }

        // Phase 2 — externally truncate dat to kill heap3 only.
        let dat_path = channel_dir(&base, &name).join("dat");
        OpenOptions::new()
            .write(true)
            .open(&dat_path)
            .unwrap()
            .set_len(dat_keep_len)
            .unwrap();

        // Phase 3 — reopen and assert.
        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        let seqs: Vec<u64> = recovered.index.iter().map(|e| e.seq).collect();
        assert_eq!(
            seqs,
            vec![0, 1, 2, 3],
            "expected heap1,inline1,heap2,inline2 to survive (got seqs {:?})",
            seqs
        );

        // Both inlines must still report inline.
        assert!(!recovered.index[0].is_inline(), "seq 0 should be heap");
        assert!(recovered.index[1].is_inline(), "seq 1 should be inline");
        assert!(!recovered.index[2].is_inline(), "seq 2 should be heap");
        assert!(recovered.index[3].is_inline(), "seq 3 should be inline");

        // Dat must have been re-trimmed to exactly the surviving
        // heap entries' end. (Lines 161-177 do a final sweep that
        // truncates dat to the highest retained `(offset + len)`.)
        assert_eq!(
            recovered.payload_bytes.len() as u64,
            dat_keep_len,
            "dat should be exactly heap1+heap2 bytes after recovery"
        );
        assert_eq!(
            std::fs::metadata(&dat_path).unwrap().len(),
            dat_keep_len,
            "dat file size mismatch after recovery"
        );
        // Index must have been re-trimmed to drop heap3's record
        // (one 20-byte slot removed from the tail).
        let idx_path = channel_dir(&base, &name).join("idx");
        assert_eq!(
            std::fs::metadata(&idx_path).unwrap().len(),
            (4 * REDEX_ENTRY_SIZE) as u64,
            "idx file should have exactly 4 records after recovery"
        );

        cleanup(&base);
    }

    /// Truncating dat all the way back to BEFORE heap2 must drop
    /// heap2 and heap3, but keep heap1 and any inlines between
    /// them. Layout:
    ///
    ///     idx: [heap1, inline1, heap2, inline2, heap3]
    ///     dat truncated to: <h1 bytes>
    ///
    /// Surviving index: `[heap1, inline1]`. heap2's tail is
    /// torn → it and everything after it must be dropped. inline2
    /// sits *after* heap2 in the index, so even though it does
    /// not depend on dat, it is dropped by the backward-walk
    /// because heap2's tear marks position 2 as truncated and
    /// later positions are torn-or-later.
    #[test]
    fn test_external_dat_truncation_to_first_heap_drops_everything_after() {
        let base = tmpdir();
        let name = ChannelName::new("t/external-trunc-deep").unwrap();

        let inline_a = *b"in_a____";
        let inline_b = *b"in_b____";
        let h1_payload = b"heap1";
        let h2_payload = b"heap2";
        let h3_payload = b"heap3";

        {
            let recovered = DiskSegment::open(&base, &name, 0).unwrap();

            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(0, 0, 5, 0, payload_checksum(h1_payload)),
                    h1_payload,
                )
                .unwrap();
            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_inline(1, &inline_a, payload_checksum(&inline_a)),
                    &inline_a,
                )
                .unwrap();
            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(2, 5, 5, 0, payload_checksum(h2_payload)),
                    h2_payload,
                )
                .unwrap();
            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_inline(3, &inline_b, payload_checksum(&inline_b)),
                    &inline_b,
                )
                .unwrap();
            recovered
                .disk
                .append_entry(
                    &RedexEntry::new_heap(4, 10, 5, 0, payload_checksum(h3_payload)),
                    h3_payload,
                )
                .unwrap();
            recovered.disk.sync().unwrap();
        }

        // Truncate dat to keep heap1 only.
        let dat_path = channel_dir(&base, &name).join("dat");
        OpenOptions::new()
            .write(true)
            .open(&dat_path)
            .unwrap()
            .set_len(5)
            .unwrap();

        let recovered = DiskSegment::open(&base, &name, 0).unwrap();
        let seqs: Vec<u64> = recovered.index.iter().map(|e| e.seq).collect();
        assert_eq!(
            seqs,
            vec![0, 1],
            "deep dat truncation must keep only entries up to (but not past) the earliest torn heap (got seqs {:?})",
            seqs
        );
        assert!(!recovered.index[0].is_inline());
        assert!(recovered.index[1].is_inline());

        // Dat file should be re-trimmed to heap1 only.
        assert_eq!(
            std::fs::metadata(&dat_path).unwrap().len(),
            5,
            "dat should remain exactly heap1's bytes after recovery"
        );

        cleanup(&base);
    }
}
