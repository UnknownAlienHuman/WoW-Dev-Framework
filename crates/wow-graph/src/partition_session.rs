use std::sync::{Arc, atomic::AtomicBool};

use crate::{
    GraphError, GraphErrorCode, GraphPartitionReplacementPlan, GraphPartitionSnapshot,
    GraphResult, partition::check_cancelled,
};

/// Synchronous in-memory owner. Retained Arc views never change after publication.
/// Durable activation belongs to the coherent E2-D ProjectStore publication set;
/// this session does not write legacy graph catalogs or claim crash durability.
pub struct GraphPartitionSession {
    current: Arc<GraphPartitionSnapshot>,
}

impl GraphPartitionSession {
    pub fn new(initial: GraphPartitionSnapshot, cancelled: &AtomicBool) -> GraphResult<Self> {
        initial.validate(cancelled)?;
        check_cancelled(cancelled)?;
        Ok(Self { current: Arc::new(initial) })
    }

    #[must_use]
    pub fn view(&self) -> Arc<GraphPartitionSnapshot> {
        Arc::clone(&self.current)
    }

    /// Validates before the single publication assignment. Concurrent callers
    /// need exclusive session access; no hidden mutex, worker or background task
    /// is created. Cancellation after the assignment cannot undo a publication.
    pub fn publish(
        &mut self,
        plan: GraphPartitionReplacementPlan,
        cancelled: &AtomicBool,
    ) -> GraphResult<Arc<GraphPartitionSnapshot>> {
        check_cancelled(cancelled)?;
        if plan.expected_snapshot_id() != self.current.snapshot().snapshot_id() {
            return Err(GraphError::new(
                GraphErrorCode::PartitionStale,
                "graph replacement plan was prepared against a stale snapshot",
            ));
        }
        plan.candidate().validate(cancelled)?;
        if plan.candidate() != self.current.as_ref() {
            let next = Arc::new(plan.candidate().clone());
            check_cancelled(cancelled)?;
            self.current = next;
        } else {
            check_cancelled(cancelled)?;
        }
        Ok(self.view())
    }
}
