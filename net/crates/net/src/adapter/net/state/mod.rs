//! Layer 4: Distributed State for Net.
//!
//! Provides causal ordering, distributed event logs, state snapshots,
//! and the foundation for daemon migration. No global coordinator —
//! causal consistency by default.

pub mod causal;
pub mod horizon;
pub mod log;
pub mod snapshot;

pub use causal::{
    read_causal_events, write_causal_events, CausalChainBuilder, CausalEvent, CausalLink,
    ChainError, CAUSAL_LINK_SIZE, SUBPROTOCOL_CAUSAL, SUBPROTOCOL_SNAPSHOT,
};
pub use horizon::{HorizonEncoder, ObservedHorizon};
pub use log::{EntityLog, LogError, LogIndex};
pub use snapshot::{SnapshotStore, StateSnapshot};
