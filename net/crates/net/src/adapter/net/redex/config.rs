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

    /// Initial reservation hint for the heap payload segment. Used
    /// only as the capacity passed to the backing `Vec` on open,
    /// capped at 64 MiB internally — the segment grows past this
    /// value on append up to a 3 GB hard limit. **Retention is NOT
    /// driven by this field** in v1; use `retention_max_events`,
    /// `retention_max_bytes`, or `retention_max_age_ns` for that.
    ///
    /// v2's warm-tier rollover will consume this value as the
    /// rollover trigger (see REDEX_V2_PLAN §3).
    pub max_memory_bytes: usize,

    /// Keep only the newest K events. `None` = unbounded.
    pub retention_max_events: Option<u64>,

    /// Keep only the newest M bytes of payload. `None` = unbounded.
    pub retention_max_bytes: Option<u64>,

    /// Drop entries older than this many nanoseconds at the next
    /// [`super::RedexFile::sweep_retention`] tick. Age is measured
    /// against `SystemTime::now()` at append time.
    ///
    /// v2 limitation: per-entry timestamps are in-memory only. On
    /// reopen of a persistent file, all recovered entries get "now"
    /// as their fake timestamp — age retention starts fresh from
    /// the reopen moment. v2 mmap tier will persist timestamps.
    pub retention_max_age_ns: Option<u64>,
}

impl Default for RedexFileConfig {
    fn default() -> Self {
        Self {
            persistent: false,
            max_memory_bytes: 64 * 1024 * 1024, // 64 MiB soft cap
            retention_max_events: None,
            retention_max_bytes: None,
            retention_max_age_ns: None,
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

    /// Set the initial reservation size for the heap segment (capped
    /// at 64 MiB internally). Does NOT enforce a retention cap — use
    /// [`Self::with_retention_max_bytes`] for that.
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

    /// Drop entries older than `max_age`. Measured in nanoseconds
    /// against `SystemTime::now()` at append time.
    pub fn with_retention_max_age(mut self, max_age: Duration) -> Self {
        self.retention_max_age_ns = Some(max_age.as_nanos() as u64);
        self
    }
}
