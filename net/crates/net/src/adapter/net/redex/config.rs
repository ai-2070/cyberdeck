//! Per-file configuration for RedEX.

use std::time::Duration;

/// Per-file configuration supplied at `Redex::open_file` time.
#[derive(Debug, Clone, Copy)]
pub struct RedexFileConfig {
    /// Heap-only (`false`) vs heap + simple disk segment (`true`).
    ///
    /// `true` requires the `redex-disk` feature **and** a persistent
    /// base directory configured on the owning `Redex` manager via
    /// `Redex::with_persistent_dir`. With no base dir, `open_file`
    /// returns an error.
    ///
    /// With `redex-disk` off, this field is silently ignored — the
    /// file is heap-only regardless.
    pub persistent: bool,

    /// Soft cap on heap payload bytes. When retention is configured,
    /// eviction sweeps evict the oldest entries on the next maintenance
    /// tick once this is exceeded.
    ///
    /// v2's warm-tier rollover is not implemented in v1.
    pub max_memory_bytes: usize,

    /// Keep only the newest K events. `None` = unbounded.
    pub retention_max_events: Option<u64>,

    /// Keep only the newest M bytes of payload. `None` = unbounded.
    pub retention_max_bytes: Option<u64>,

    /// If set, a background task fsyncs the disk segment at this
    /// interval. `None` (default) = sync on close only. Honored only
    /// when `persistent` is true.
    pub sync_interval: Option<Duration>,
}

impl Default for RedexFileConfig {
    fn default() -> Self {
        Self {
            persistent: false,
            max_memory_bytes: 64 * 1024 * 1024, // 64 MiB soft cap
            retention_max_events: None,
            retention_max_bytes: None,
            sync_interval: None,
        }
    }
}

impl RedexFileConfig {
    /// Start from defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable persistent (disk-backed) storage.
    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// Set the soft memory cap that triggers retention sweeps.
    pub fn with_max_memory_bytes(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Keep at most `events` entries.
    pub fn with_retention_max_events(mut self, events: u64) -> Self {
        self.retention_max_events = Some(events);
        self
    }

    /// Keep at most `bytes` bytes of payload.
    pub fn with_retention_max_bytes(mut self, bytes: u64) -> Self {
        self.retention_max_bytes = Some(bytes);
        self
    }

    /// Set the periodic fsync interval for persistent files.
    pub fn with_sync_interval(mut self, interval: Duration) -> Self {
        self.sync_interval = Some(interval);
        self
    }
}
