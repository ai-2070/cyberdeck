//! `Redex` — manager owning the `ChannelName -> RedexFile` map.
//!
//! Holds an optional reference to an [`AuthGuard`](super::super::AuthGuard)
//! plus a local origin-hash. When auth is wired up, `open_file` rejects
//! opens unless `(origin, canonical channel name)` has been explicitly
//! authorized via [`AuthGuard::allow_channel`]. The 16-bit wire
//! `channel_hash` alone is not sufficient here — at mesh scale it
//! collides often enough to allow ACL bypass between unrelated names,
//! and even a 64-bit non-cryptographic hash would be crackable by
//! birthday search offline. Keying on the canonical name is the only
//! collision-free answer.

use std::sync::Arc;

use dashmap::DashMap;

use super::super::channel::{AuthGuard, ChannelName};
use super::config::RedexFileConfig;
use super::error::RedexError;
use super::file::RedexFile;

#[cfg(feature = "redex-disk")]
use std::path::PathBuf;

/// Manager for a set of RedEX files bound to channel names.
pub struct Redex {
    files: DashMap<ChannelName, RedexFile>,
    auth: Option<Arc<AuthGuard>>,
    origin_hash: u32,
    #[cfg(feature = "redex-disk")]
    persistent_dir: Option<PathBuf>,
}

impl Redex {
    /// Create a manager without auth enforcement. Suitable for
    /// single-process tests and local workloads.
    pub fn new() -> Self {
        Self {
            files: DashMap::new(),
            auth: None,
            origin_hash: 0,
            #[cfg(feature = "redex-disk")]
            persistent_dir: None,
        }
    }

    /// Create a manager that rejects `open_file` unless the
    /// `(origin_hash, channel)` pair has been authorized by `guard`
    /// via [`AuthGuard::allow_channel`]. Uses the exact 64-bit
    /// channel identity, not the 16-bit wire hash — see the module
    /// docs for rationale.
    pub fn with_auth(guard: Arc<AuthGuard>, origin_hash: u32) -> Self {
        Self {
            files: DashMap::new(),
            auth: Some(guard),
            origin_hash,
            #[cfg(feature = "redex-disk")]
            persistent_dir: None,
        }
    }

    /// Set the base directory for disk-backed (`persistent: true`)
    /// files. All files opened with `persistent: true` use
    /// `<dir>/<channel_path>/{idx,dat}` for durability.
    #[cfg(feature = "redex-disk")]
    pub fn with_persistent_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.persistent_dir = Some(dir.into());
        self
    }

    /// Open (create if absent) a RedEX file bound to `name`.
    ///
    /// Re-opening an existing name returns the existing handle. The
    /// `config` argument is honored only on first open; subsequent
    /// opens ignore it and return the live file.
    ///
    /// With `persistent: true`, the manager must have been configured
    /// via `with_persistent_dir` (feature `redex-disk`) — otherwise
    /// `open_file` returns a [`RedexError::Channel`] that describes
    /// the missing base dir.
    pub fn open_file(
        &self,
        name: &ChannelName,
        config: RedexFileConfig,
    ) -> Result<RedexFile, RedexError> {
        if let Some(auth) = &self.auth {
            // Use the canonical-name ACL for the storage decision —
            // `is_authorized` (16-bit hash) is reserved for the
            // fast-path packet check where AEAD integrity backstops
            // any bloom-filter false positives. Storage access has
            // no such backstop, and even a 64-bit non-cryptographic
            // hash would be birthday-crackable offline, so the ACL
            // keys on the full canonical name.
            // Widen the 32-bit local origin_hash to match
            // `AuthGuard`'s 64-bit key. The guard keeps the local
            // entity and remote subscribers in disjoint key ranges
            // simply by the natural spread of node_ids — the local
            // entity lives in the lower 2^32 and remote subscribers'
            // full node_ids occupy the full range, so there is no
            // cross-contamination.
            if !auth.is_authorized_full(self.origin_hash as u64, name) {
                return Err(RedexError::Unauthorized);
            }
        }

        if let Some(existing) = self.files.get(name) {
            return Ok(existing.clone());
        }

        // Build may fail (e.g. disk path creation under `persistent`).
        // On failure we still re-check the map — a concurrent opener
        // may have succeeded in between. Returning their file is
        // correct since `open_file` semantically "returns the live
        // file for this name."
        let file = match self.build_file(name, config) {
            Ok(file) => file,
            Err(err) => {
                if let Some(existing) = self.files.get(name) {
                    return Ok(existing.clone());
                }
                return Err(err);
            }
        };
        let entry = self.files.entry(name.clone()).or_insert(file);
        Ok(entry.clone())
    }

    fn build_file(
        &self,
        name: &ChannelName,
        config: RedexFileConfig,
    ) -> Result<RedexFile, RedexError> {
        #[cfg(feature = "redex-disk")]
        if config.persistent {
            let dir = self.persistent_dir.as_ref().ok_or_else(|| {
                RedexError::Channel(
                    "config.persistent=true requires Redex::with_persistent_dir(...)".into(),
                )
            })?;
            return RedexFile::open_persistent(name.clone(), config, dir);
        }
        Ok(RedexFile::new(name.clone(), config))
    }

    /// Look up an already-opened file.
    pub fn get_file(&self, name: &ChannelName) -> Option<RedexFile> {
        self.files.get(name).map(|r| r.clone())
    }

    /// Close and remove a file. Outstanding tail streams receive
    /// `RedexError::Closed`. No-op if no file is open under `name`.
    pub fn close_file(&self, name: &ChannelName) -> Result<(), RedexError> {
        if let Some((_, file)) = self.files.remove(name) {
            file.close()?;
        }
        Ok(())
    }

    /// Snapshot list of currently open files. Cheap clone.
    pub fn open_files(&self) -> Vec<RedexFile> {
        self.files.iter().map(|r| r.value().clone()).collect()
    }

    /// Run retention on every open file. Typically called on a
    /// heartbeat tick by the owning runtime.
    pub fn sweep_retention(&self) {
        for entry in self.files.iter() {
            entry.value().sweep_retention();
        }
    }
}

impl Default for Redex {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Redex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Redex");
        dbg.field("files", &self.files.len())
            .field("auth", &self.auth.is_some())
            .field("origin_hash", &self.origin_hash);
        #[cfg(feature = "redex-disk")]
        dbg.field("persistent_dir", &self.persistent_dir);
        dbg.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cn(s: &str) -> ChannelName {
        ChannelName::new(s).unwrap()
    }

    #[test]
    fn test_open_and_get() {
        let r = Redex::new();
        let name = cn("sensors/lidar");
        let f = r.open_file(&name, RedexFileConfig::default()).unwrap();
        f.append(b"x").unwrap();

        let g = r.get_file(&name).unwrap();
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn test_reopen_returns_same_file() {
        let r = Redex::new();
        let name = cn("shared");
        let f = r.open_file(&name, RedexFileConfig::default()).unwrap();
        f.append(b"a").unwrap();
        let f2 = r.open_file(&name, RedexFileConfig::default()).unwrap();
        assert_eq!(f2.len(), 1); // sees existing append
        f2.append(b"b").unwrap();
        assert_eq!(f.len(), 2); // original handle also sees it
    }

    #[test]
    fn test_get_file_missing_returns_none() {
        let r = Redex::new();
        assert!(r.get_file(&cn("missing")).is_none());
    }

    #[test]
    fn test_auth_denies_unknown_origin() {
        let guard = Arc::new(AuthGuard::new());
        let r = Redex::with_auth(guard, 0xAAAA_BBBB);
        let name = cn("restricted");
        assert!(matches!(
            r.open_file(&name, RedexFileConfig::default()),
            Err(RedexError::Unauthorized)
        ));
    }

    #[test]
    fn test_auth_allows_authorized_origin() {
        let guard = Arc::new(AuthGuard::new());
        let name = cn("allowed");
        // `allow_channel` populates the exact (control-plane) ACL
        // used by `open_file`, plus the fast-path bloom so packet
        // checks on the same channel also pass.
        guard.allow_channel(0x1234_5678, &name);
        let r = Redex::with_auth(guard, 0x1234_5678);
        assert!(r.open_file(&name, RedexFileConfig::default()).is_ok());
    }

    #[test]
    fn test_auth_fast_path_alone_does_not_authorize_open_file() {
        // Regression: `open_file` used to accept any origin that
        // had the 16-bit `channel_hash` in its fast-path bloom. A
        // different channel name whose 16-bit hash collided with an
        // authorized one would then grant unauthorized storage
        // access. The fix requires the canonical channel name in
        // the exact ACL, so a fast-path-only authorization is
        // insufficient.
        let guard = Arc::new(AuthGuard::new());
        let name = cn("sensitive");
        // Authorize the fast path ONLY (no allow_channel).
        guard.authorize(0x1234_5678, name.hash());
        let r = Redex::with_auth(guard, 0x1234_5678);
        assert!(matches!(
            r.open_file(&name, RedexFileConfig::default()),
            Err(RedexError::Unauthorized)
        ));
    }

    #[test]
    fn test_close_file_rejects_append_on_existing_handle() {
        let r = Redex::new();
        let name = cn("closable");
        let f = r.open_file(&name, RedexFileConfig::default()).unwrap();
        f.append(b"x").unwrap();
        r.close_file(&name).unwrap();
        assert!(f.append(b"y").is_err());
    }

    #[test]
    fn test_sweep_retention_runs_on_all_open_files() {
        let r = Redex::new();
        let cfg = RedexFileConfig::default().with_retention_max_events(1);
        let f1 = r.open_file(&cn("f1"), cfg).unwrap();
        let f2 = r.open_file(&cn("f2"), cfg).unwrap();
        for i in 0..3 {
            f1.append(format!("{}", i).as_bytes()).unwrap();
            f2.append(format!("{}", i).as_bytes()).unwrap();
        }
        r.sweep_retention();
        assert_eq!(f1.len(), 1);
        assert_eq!(f2.len(), 1);
    }
}
