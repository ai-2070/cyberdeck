//! `TasksFold` — decodes `EventMeta` + payload, routes on dispatch,
//! mutates [`super::state::TasksState`].

use super::super::super::redex::{RedexError, RedexEvent, RedexFold};
use super::super::meta::{compute_checksum, EventMeta, EVENT_META_SIZE};
use super::dispatch::{
    DISPATCH_TASK_COMPLETED, DISPATCH_TASK_CREATED, DISPATCH_TASK_DELETED, DISPATCH_TASK_RENAMED,
};
use super::state::TasksState;
use super::types::{
    Task, TaskCompletedPayload, TaskCreatedPayload, TaskDeletedPayload, TaskRenamedPayload,
    TaskStatus,
};

/// Fold implementation for the tasks model.
pub struct TasksFold;

impl RedexFold<TasksState> for TasksFold {
    fn apply(&mut self, ev: &RedexEvent, state: &mut TasksState) -> Result<(), RedexError> {
        if ev.payload.len() < EVENT_META_SIZE {
            return Err(RedexError::Encode(format!(
                "tasks payload too short: {} bytes (need >= {})",
                ev.payload.len(),
                EVENT_META_SIZE
            )));
        }
        let meta = EventMeta::from_bytes(&ev.payload[..EVENT_META_SIZE])
            .ok_or_else(|| RedexError::Encode("bad EventMeta prefix".into()))?;
        let tail = &ev.payload[EVENT_META_SIZE..];

        // Verify the checksum stamped at ingest against the tail we
        // received from RedEX. Catches disk corruption, tampered
        // on-disk files, and truncated tails. Under
        // `FoldErrorPolicy::Stop` this halts the fold task; under
        // `LogAndContinue` the event is counted and skipped.
        let expected = compute_checksum(tail);
        if meta.checksum != expected {
            return Err(RedexError::Encode(format!(
                "tasks fold: EventMeta checksum mismatch at seq {} (got {:#010x}, tail hashes to {:#010x})",
                ev.entry.seq, meta.checksum, expected
            )));
        }

        match meta.dispatch {
            DISPATCH_TASK_CREATED => {
                let p: TaskCreatedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                state.tasks.insert(
                    p.id,
                    Task {
                        id: p.id,
                        title: p.title,
                        status: TaskStatus::Pending,
                        created_ns: p.now_ns,
                        updated_ns: p.now_ns,
                    },
                );
            }
            DISPATCH_TASK_RENAMED => {
                let p: TaskRenamedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(t) = state.tasks.get_mut(&p.id) {
                    t.title = p.new_title;
                    t.updated_ns = p.now_ns;
                }
                // Rename on an unknown id is a no-op; the log is the
                // source of truth and a missing create simply means
                // the rename refers to state we never observed.
            }
            DISPATCH_TASK_COMPLETED => {
                let p: TaskCompletedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(t) = state.tasks.get_mut(&p.id) {
                    t.status = TaskStatus::Completed;
                    t.updated_ns = p.now_ns;
                }
            }
            DISPATCH_TASK_DELETED => {
                let p: TaskDeletedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                state.tasks.remove(&p.id);
            }
            other => {
                // Unknown dispatches in the CortEX-internal range are
                // treated as forward-compatibility — log and skip.
                tracing::debug!(
                    dispatch = other,
                    seq = ev.entry.seq,
                    "tasks fold: ignoring unknown dispatch"
                );
            }
        }
        Ok(())
    }
}
