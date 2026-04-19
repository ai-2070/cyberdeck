//! Adapter configuration: start position + fold error policy.

/// Where the fold task begins consuming the RedEX tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPosition {
    /// Replay from the beginning of the file (seq 0). Default.
    FromBeginning,
    /// Start live-only; skip backfill. Use when `State` is rehydrated
    /// from an external snapshot and the adapter should only see new
    /// post-open appends.
    LiveOnly,
    /// Start at a caller-supplied checkpoint. The fold task sees
    /// events with `RedexEntry::seq >= n`.
    FromSeq(u64),
}

/// What the fold task does when [`super::super::redex::RedexFold::apply`]
/// returns an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldErrorPolicy {
    /// First error stops the fold task. State remains readable as of
    /// the last successful apply. Subsequent ingests still succeed
    /// (the log is the source of truth; a broken fold is a bug in the
    /// fold, not in the data). A later process instance with a fixed
    /// fold can replay from the beginning and succeed. Default.
    Stop,
    /// Log + skip. The offending event is not folded; the task
    /// continues with the next event. Visible via
    /// [`super::CortexAdapter::fold_errors`]. Useful for development;
    /// production CortEX should prefer `Stop` so bugs don't silently
    /// corrupt derived state.
    LogAndContinue,
}

/// One-shot configuration for a [`super::CortexAdapter`] instance.
#[derive(Debug, Clone, Copy)]
pub struct CortexAdapterConfig {
    /// Where the fold task starts.
    pub start: StartPosition,
    /// What to do on fold error.
    pub on_fold_error: FoldErrorPolicy,
}

impl Default for CortexAdapterConfig {
    fn default() -> Self {
        Self {
            start: StartPosition::FromBeginning,
            on_fold_error: FoldErrorPolicy::Stop,
        }
    }
}

impl CortexAdapterConfig {
    /// Start from defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the start position.
    pub fn with_start(mut self, start: StartPosition) -> Self {
        self.start = start;
        self
    }

    /// Set the fold error policy.
    pub fn with_fold_error_policy(mut self, policy: FoldErrorPolicy) -> Self {
        self.on_fold_error = policy;
        self
    }
}
