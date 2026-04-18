//! `Redex` — manager owning the `ChannelName -> RedexFile` map.
//!
//! Holds an optional reference to an [`AuthGuard`](super::super::AuthGuard)
//! plus a local origin-hash. When auth is wired up, `open_file` rejects
//! opens whose `(origin, channel_hash)` pair is not authorized.

use std::sync::Arc;

use dashmap::DashMap;

use super::super::channel::{AuthGuard, ChannelName};
use super::config::RedexFileConfig;
use super::error::RedexError;
use super::file::RedexFile;

/// Manager for a set of RedEX files bound to channel names.
pub struct Redex {
    files: DashMap<ChannelName, RedexFile>,
    auth: Option<Arc<AuthGuard>>,
    origin_hash: u32,
}

impl Redex {
    /// Create a manager without auth enforcement. Suitable for
    /// single-process tests and local workloads.
    pub fn new() -> Self {
        Self {
            files: DashMap::new(),
            auth: None,
            origin_hash: 0,
        }
    }

    /// Create a manager that rejects `open_file` unless the
    /// `(origin_hash, channel_hash)` pair is authorized by `guard`.
    pub fn with_auth(guard: Arc<AuthGuard>, origin_hash: u32) -> Self {
        Self {
            files: DashMap::new(),
            auth: Some(guard),
            origin_hash,
        }
    }

    /// Open (create if absent) a RedEX file bound to `name`.
    ///
    /// Re-opening an existing name returns the existing handle. The
    /// `config` argument is honored only on first open; subsequent
    /// opens ignore it and return the live file.
    pub fn open_file(
        &self,
        name: &ChannelName,
        config: RedexFileConfig,
    ) -> Result<RedexFile, RedexError> {
        if let Some(auth) = &self.auth {
            if !auth.is_authorized(self.origin_hash, name.hash()) {
                return Err(RedexError::Unauthorized);
            }
        }

        if let Some(existing) = self.files.get(name) {
            return Ok(existing.clone());
        }

        let entry = self
            .files
            .entry(name.clone())
            .or_insert_with(|| RedexFile::new(name.clone(), config));
        Ok(entry.clone())
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
        f.debug_struct("Redex")
            .field("files", &self.files.len())
            .field("auth", &self.auth.is_some())
            .field("origin_hash", &self.origin_hash)
            .finish()
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
        guard.authorize(0x1234_5678, name.hash());
        let r = Redex::with_auth(guard, 0x1234_5678);
        assert!(r.open_file(&name, RedexFileConfig::default()).is_ok());
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
