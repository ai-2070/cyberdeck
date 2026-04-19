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
//! - Every append writes to the OS page cache (no per-append fsync).
//! - `close()` fsyncs both files.
//! - Explicit `sync()` calls (and `close()`) fsync both files
//!   calls `sync_all` on both files; `None` (default) = sync on close
//!   only.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

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
}

impl DiskSegment {
    /// Open (or create) the idx + dat files for `name` under `base_dir`
    /// and recover the index from disk.
    pub(super) fn open(
        base_dir: &Path,
        name: &ChannelName,
    ) -> Result<RecoveredSegment, RedexError> {
        let dir = channel_dir(base_dir, name);
        std::fs::create_dir_all(&dir).map_err(RedexError::io)?;
        let idx_path = dir.join("idx");
        let dat_path = dir.join("dat");

        // Recover existing index.
        let (index, idx_len_truncated) = read_index(&idx_path)?;
        let payload_bytes = read_payload(&dat_path)?;

        // Open for append. Truncate idx to a whole multiple of 20 bytes
        // if the last write was torn (crash mid-append).
        if idx_len_truncated {
            let file = OpenOptions::new()
                .write(true)
                .open(&idx_path)
                .map_err(RedexError::io)?;
            file.set_len((index.len() * REDEX_ENTRY_SIZE) as u64)
                .map_err(RedexError::io)?;
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
            },
            index,
            payload_bytes,
        })
    }

    /// Append an entry and (for heap entries) its payload to disk.
    /// Inline entries skip the dat write — their payload rides in the
    /// 20-byte idx record.
    ///
    /// Writes go through the OS page cache with no per-append fsync.
    /// Durability is realized on `sync()` or `close()`.
    pub(super) fn append_entry(
        &self,
        entry: &RedexEntry,
        payload: &[u8],
    ) -> Result<(), RedexError> {
        if !entry.is_inline() {
            let mut dat = self.dat_file.lock();
            dat.write_all(payload).map_err(RedexError::io)?;
        }
        let mut idx = self.idx_file.lock();
        idx.write_all(&entry.to_bytes()).map_err(RedexError::io)?;
        Ok(())
    }

    /// Append several entries and their payloads atomically (per-file:
    /// each file's buffered write is contiguous). Inline entries only
    /// touch the idx file.
    pub(super) fn append_entries(
        &self,
        entries_and_payloads: &[(RedexEntry, &[u8])],
    ) -> Result<(), RedexError> {
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
            let recovered = DiskSegment::open(&base, &name).unwrap();
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
        let recovered = DiskSegment::open(&base, &name).unwrap();
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

        let recovered = DiskSegment::open(&base, &name).unwrap();
        let payload = *b"abcdefgh";
        let entry = RedexEntry::new_inline(0, &payload, payload_checksum(&payload));
        recovered.disk.append_entry(&entry, &payload).unwrap();
        recovered.disk.sync().unwrap();
        drop(recovered);

        let recovered = DiskSegment::open(&base, &name).unwrap();
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
            let recovered = DiskSegment::open(&base, &name).unwrap();
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
        let recovered = DiskSegment::open(&base, &name).unwrap();
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
}
