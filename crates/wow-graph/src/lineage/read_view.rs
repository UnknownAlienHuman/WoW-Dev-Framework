//! Actual typed Store leases keep the exact snapshot and validation object alive
//! until checked closure. A descriptor or Drop cannot claim successful release.
use super::snapshot::SnapshotRecord;
use super::*;
use wow_store::{LeaseId, LogicalEpoch, OperationId, OperationState, RequestDigest, Store};
const HOLDER: &str = "wow-graph.lineage-candidate-reader.v1";

pub struct LineageReadLeaseRequest<'a> {
    pub operation: &'a OperationId,
    pub request_digest: &'a RequestDigest,
    pub expires_after: LogicalEpoch,
}
#[must_use = "lineage read views require checked close"]
pub struct LineageGraphView<'a> {
    store: &'a mut Store,
    record: SnapshotRecord,
    leases: [LeaseId; 2],
    limits: LineageQueryLimits,
    ledger: OperationBudget<'a>,
}
impl PersistentLineageStore<'_> {
    pub fn open_lineage_graph_view<'a>(
        &'a mut self,
        handle: &SealedLineageHandle,
        expected: &PreparedLineageSnapshot,
        request: LineageReadLeaseRequest<'_>,
        limits: LineageQueryLimits,
        stop: &'a AtomicBool,
    ) -> GraphResult<LineageGraphView<'a>> {
        let mut ledger = limits.ledger(stop)?;
        let LineageReadLeaseRequest {
            operation,
            request_digest,
            expires_after,
        } = request;
        if expires_after.get() == 0 || expires_after.get() > i64::MAX as u64 {
            return Err(query::query_invalid());
        }
        let registered = self
            .store
            .operation(operation)?
            .ok_or_else(query::query_invalid)?;
        if registered.state() != OperationState::Prepared
            || registered.request_digest() != request_digest
        {
            return Err(query::query_invalid());
        }
        ledger.metadata(&(&handle, &expected.record))?;
        self.require_roots(handle)?;
        let lease = |role: &str| -> GraphResult<LeaseId> {
            Ok(LeaseId::new(
                identity(
                    "wow-graph/lineage-read-lease/1",
                    &(operation, request_digest, handle, role),
                )?
                .replace('/', "-"),
            )?)
        };
        let leases = [lease("snapshot")?, lease("validation")?];
        let objects = [
            handle.object_id.clone(),
            handle.validation_object_id.clone(),
        ];
        for index in 0..2 {
            let acquired = checkpoint(stop).and_then(|()| {
                self.store
                    .acquire_lease(
                        leases[index].clone(),
                        objects[index].clone(),
                        HOLDER,
                        expires_after,
                    )
                    .map(|_| ())
                    .map_err(Into::into)
            });
            if let Err(error) = acquired {
                let mut failed = false;
                for id in leases[..index].iter().rev() {
                    if self.store.release_lease(id, HOLDER).is_err() {
                        failed = true;
                    }
                }
                return Err(if failed { close_failed() } else { error });
            }
        }
        match self.validated_record(handle, expected, stop) {
            Ok(record) => Ok(LineageGraphView {
                store: self.store,
                record,
                leases,
                limits,
                ledger,
            }),
            Err(error) => {
                let mut failed = false;
                for id in leases.iter().rev() {
                    if self.store.release_lease(id, HOLDER).is_err() {
                        failed = true;
                    }
                }
                Err(if failed { close_failed() } else { error })
            }
        }
    }
}
impl LineageGraphView<'_> {
    pub fn query(
        &mut self,
        request: &LineageCandidateQuery,
        sources: &LineageNativeSources<'_, '_>,
    ) -> GraphResult<LineageQueryResult> {
        let stop = self.ledger.stop;
        query::execute(
            &self.record,
            request,
            sources,
            self.limits,
            &mut self.ledger,
            stop,
        )
    }
    pub fn close(self) -> GraphResult<()> {
        let mut failed = false;
        for id in self.leases.iter().rev() {
            if self.store.release_lease(id, HOLDER).is_err() {
                failed = true;
            }
        }
        if failed { Err(close_failed()) } else { Ok(()) }
    }
}
fn close_failed() -> GraphError {
    GraphError::new(
        GraphErrorCode::LineageReadCloseFailed,
        "lineage read lease closure failed",
    )
}
