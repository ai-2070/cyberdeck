//! Persistent producer identity for cross-process dedup.
//!
//! Adapters that rely on backend-side dedup keyed on
//! `(producer_nonce, shard, sequence_start, i)` (today: JetStream's
//! `Nats-Msg-Id` header) need a `producer_nonce` that survives
//! process restart. Without that, a producer that crashes mid-batch
//! and restarts gets a fresh nonce, the post-restart retry writes
//! new msg-ids, and JetStream's dedup window can't recognize them
//! as duplicates of the pre-crash partial — the accepted half ends
//! up persisted twice (BUG #56).
//!
//! `PersistentProducerNonce` provides exactly that: a u64 sampled
//! once and stored on disk. On startup, callers `load_or_create` it
//! against a known path; the second + Nth process loads the same
//! nonce, so retries' msg-ids match the pre-crash incarnation's.
//! Atomic write (`tempfile + rename`) so a crash between the
//! random-sample and the final rename leaves either no file (next
//! load creates fresh) or the complete file — never a partial
//! write.
//!
//! When the bus is configured WITHOUT a path
//! (`EventBusConfig::producer_nonce_path = None`), the existing
//! per-process nonce is used. That keeps the behavior of every
//! pre-fix caller unchanged and is documented as
//! "at-most-once-across-restarts."

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 8-byte u64 nonce, persisted little-endian on disk.
const NONCE_FILE_LEN: usize = 8;

/// Persistent u64 nonce loaded from (or created at) a stable path.
///
/// Callers construct via [`Self::load_or_create`] and read the value
/// via [`Self::nonce`]. The struct itself is cheap to clone — the
/// nonce is a `u64` and the path is a `PathBuf` retained for
/// debugging / logging.
#[derive(Debug, Clone)]
pub struct PersistentProducerNonce {
    nonce: u64,
    #[allow(dead_code)] // retained for diagnostic output
    path: PathBuf,
}

impl PersistentProducerNonce {
    /// Load (or create) the persistent nonce at `path`.
    ///
    /// On first call: samples a fresh u64 from `getrandom`, writes
    /// it to `path` atomically (write to `<path>.tmp`, fsync, rename
    /// to `path`), and returns the value.
    ///
    /// On subsequent calls (post-restart, same path): reads the
    /// existing 8-byte file and returns its little-endian u64.
    ///
    /// Errors:
    /// - `io::ErrorKind::NotFound` if the parent directory doesn't
    ///   exist. We don't auto-create the parent — that's a
    ///   configuration decision the caller should make explicitly.
    /// - `io::ErrorKind::InvalidData` if the file exists but has
    ///   length other than 8 bytes (corrupt or someone else's file
    ///   at this path).
    /// - Other `io::Error` from filesystem operations.
    pub fn load_or_create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Fast path: file exists.
        match fs::read(&path) {
            Ok(bytes) => {
                if bytes.len() != NONCE_FILE_LEN {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "producer-nonce file at {} has length {} (expected {})",
                            path.display(),
                            bytes.len(),
                            NONCE_FILE_LEN,
                        ),
                    ));
                }
                let mut buf = [0u8; NONCE_FILE_LEN];
                buf.copy_from_slice(&bytes);
                let nonce = u64::from_le_bytes(buf);
                Ok(Self { nonce, path })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // First-load path: sample, write atomically, return.
                Self::create_new(path)
            }
            Err(e) => Err(e),
        }
    }

    fn create_new(path: PathBuf) -> io::Result<Self> {
        // Sample a fresh nonce. We can't depend on `getrandom` here
        // — it's gated behind the `net` feature, but this module is
        // unconditional (the bus uses it whether `net` is on or
        // off, e.g. for JetStream/Redis-only deployments). Mix the
        // same set of entropy sources `event::batch_process_nonce`
        // uses, but DON'T share its `OnceLock` cache — distinct
        // create_new calls in the same process must produce distinct
        // nonces (e.g. two buses configured against different
        // nonce paths should not silently collide). The OnceLock
        // semantic is right for the per-process fallback nonce; it
        // would be wrong here.
        //
        // The mix is identical in spirit to `batch_process_nonce`:
        // wall-clock nanos + monotonic-clock marker + pid +
        // ASLR-derived stack address + thread id, all hashed
        // through xxh3. Adequate for a startup-time nonce — the
        // collision risk we care about is two-processes-on-the-
        // same-machine within a single nanosecond tick, which the
        // pid + stack marker covers.
        //
        // Refuse `0` to keep parity with `batch_process_nonce` —
        // some downstream consumers use 0 as a sentinel.
        use std::hash::{Hash, Hasher};
        use std::time::Instant;

        let wall_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let mono_marker = format!("{:?}", Instant::now());
        let pid = std::process::id() as u64;
        let stack_local: u64 = wall_nanos;
        let stack_marker = (&stack_local as *const u64) as usize;
        let mut tid_hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut tid_hasher);
        let tid = tid_hasher.finish();

        let mut hash_input = [0u8; 64];
        hash_input[..8].copy_from_slice(&wall_nanos.to_le_bytes());
        hash_input[8..16].copy_from_slice(&pid.to_le_bytes());
        hash_input[16..24].copy_from_slice(&(stack_marker as u64).to_le_bytes());
        hash_input[24..32].copy_from_slice(&tid.to_le_bytes());
        let mono_bytes = mono_marker.as_bytes();
        let n = mono_bytes.len().min(32);
        hash_input[32..32 + n].copy_from_slice(&mono_bytes[..n]);

        let mut nonce = xxhash_rust::xxh3::xxh3_64(&hash_input);
        if nonce == 0 {
            nonce = 1;
        }
        let buf = nonce.to_le_bytes();

        // Atomic write: create a sibling tempfile, fsync it, rename
        // over the target. Concurrent first-loaders racing on the
        // same path may both create temp files, but only one rename
        // wins; the loser overwrites the winner with the same bytes
        // (different nonces since each sampled independently —
        // last-rename-wins is the intended semantic, callers MUST
        // serialize the first-create externally if they care).
        // The probability of two-process race here is vanishingly
        // small in practice (caller is the bus's `new` path, which
        // runs once at startup); we don't add a lockfile because
        // the more common-by-far failure mode is a stale
        // tempfile from a prior crashed init, which `fs::rename`
        // happily overwrites.
        let tmp_path = {
            let mut p = path.clone();
            let mut name = p.file_name().map(|n| n.to_os_string()).unwrap_or_default();
            name.push(".tmp");
            p.set_file_name(name);
            p
        };
        fs::write(&tmp_path, buf)?;

        // Best-effort fsync of the file before rename. On Windows
        // `File::sync_all` is supported but the rename-durability
        // guarantee is weaker than POSIX; production callers that
        // need strict durability should use a filesystem with
        // appropriate ordering. We do the fsync regardless because
        // it's the ceiling-set part of the contract.
        if let Ok(f) = fs::File::open(&tmp_path) {
            let _ = f.sync_all();
        }
        fs::rename(&tmp_path, &path)?;

        Ok(Self { nonce, path })
    }

    /// The loaded (or freshly created) nonce.
    #[inline]
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(suffix: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        // Combine pid + nanos + suffix so concurrent test runs don't
        // collide on a shared `temp_dir()`.
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("net-test-nonce-{pid}-{nanos}-{suffix}"));
        p
    }

    #[test]
    fn first_load_creates_a_random_nonzero_nonce() {
        let path = temp_path("first");
        let nonce = PersistentProducerNonce::load_or_create(&path)
            .unwrap()
            .nonce();
        assert_ne!(nonce, 0, "first-load must sample a nonzero nonce");
        // Cleanup.
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn second_load_returns_the_same_nonce() {
        let path = temp_path("second");
        let first = PersistentProducerNonce::load_or_create(&path)
            .unwrap()
            .nonce();
        let second = PersistentProducerNonce::load_or_create(&path)
            .unwrap()
            .nonce();
        assert_eq!(
            first, second,
            "second load against same path must return the same nonce — \
             this is the load-bearing cross-restart property",
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_surfaces_invalid_data_error() {
        let path = temp_path("corrupt");
        // Write 7 bytes (one short of NONCE_FILE_LEN).
        fs::write(&path, b"shorty!").unwrap();

        let err = PersistentProducerNonce::load_or_create(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("length 7"),
            "error message should pin the actual length; got: {err}",
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_parent_directory_surfaces_not_found_error() {
        let mut path = temp_path("missing-parent");
        path.push("subdir-that-does-not-exist");
        path.push("nonce");

        let err = PersistentProducerNonce::load_or_create(&path).unwrap_err();
        // Either NotFound (Unix-y) or other kinds depending on platform;
        // we just need a clear failure rather than silent success.
        assert!(
            err.kind() == io::ErrorKind::NotFound
                || err.kind() == io::ErrorKind::PermissionDenied
                || err.kind() == io::ErrorKind::Other,
            "expected a clear filesystem error; got {err:?}",
        );
    }

    #[test]
    fn two_distinct_paths_produce_two_distinct_nonces() {
        let a = temp_path("a");
        let b = temp_path("b");
        let n_a = PersistentProducerNonce::load_or_create(&a).unwrap().nonce();
        let n_b = PersistentProducerNonce::load_or_create(&b).unwrap().nonce();
        assert_ne!(
            n_a, n_b,
            "two distinct nonce paths must produce distinct nonces (collision \
             probability is ~2^-63 — if this fires twice, suspect getrandom)",
        );
        let _ = fs::remove_file(&a);
        let _ = fs::remove_file(&b);
    }

    #[test]
    fn nonce_field_is_eight_byte_le_round_trip() {
        // Direct file-format pin: write 8 LE bytes by hand, verify
        // the load returns the matching u64.
        let path = temp_path("le-roundtrip");
        let expected: u64 = 0xDEAD_BEEF_CAFE_F00D;
        fs::write(&path, expected.to_le_bytes()).unwrap();

        let loaded = PersistentProducerNonce::load_or_create(&path)
            .unwrap()
            .nonce();
        assert_eq!(
            loaded, expected,
            "file format is 8 LE bytes — pin so a future refactor \
             that flips byte order doesn't silently produce a \
             different nonce on the same on-disk file",
        );
        let _ = fs::remove_file(&path);
    }
}
