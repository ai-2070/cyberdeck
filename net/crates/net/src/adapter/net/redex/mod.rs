//! RedEX — local append-only streaming log.
//!
//! A `RedexFile` is a named monotonic log whose on-disk/index entries are
//! 20 bytes each. Payloads live inline (≤8 bytes) or in a heap/disk
//! segment. v1 is strictly local — no replication, no dedicated wire
//! subprotocol. Files map 1:1 to [`ChannelName`](super::ChannelName) so
//! the existing [`AuthGuard`](super::AuthGuard) surface applies.
//!
//! See `docs/REDEX_PLAN.md` for the full design.

mod config;
#[cfg(feature = "redex-disk")]
mod disk;
mod entry;
mod error;
mod event;
mod file;
mod fold;
mod index;
mod manager;
mod ordered;
mod retention;
mod segment;
mod typed;

pub use config::{FsyncPolicy, RedexFileConfig};
pub use entry::{RedexEntry, RedexFlags, REDEX_ENTRY_SIZE};
pub use error::RedexError;
pub use event::RedexEvent;
pub use file::RedexFile;
pub use fold::RedexFold;
pub use index::{IndexOp, IndexStart, RedexIndex};
pub use manager::Redex;
pub use ordered::OrderedAppender;
pub use typed::TypedRedexFile;
