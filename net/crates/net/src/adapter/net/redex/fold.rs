//! `RedexFold` — the integration hook for CortEX / NetDB.
//!
//! RedEX defines the trait; RedEX itself does not install a fold.
//! A consumer (typically a CortEX adapter) opens a `tail` stream on a
//! `RedexFile`, supplies an `impl RedexFold<State>`, and drives state
//! forward as events arrive.

use super::error::RedexError;
use super::event::RedexEvent;

/// Fold arbitrary state over a stream of RedEX events.
///
/// Implementations are called in sequence order by the driver that owns
/// the tail stream. RedEX itself does not invoke this trait.
pub trait RedexFold<State> {
    /// Apply one event to `state`. Return an error to stop the fold.
    fn apply(&mut self, ev: &RedexEvent, state: &mut State) -> Result<(), RedexError>;
}
