//! Explicit source-root orchestration. The project owner alone opens source files;
//! the graph explanation and its historical metadata remain unchanged.
use std::path::Path;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_graph::{GraphEvidenceCatalog, GraphResolvedExplanation};
use wow_project::disk::ProjectInputDirectory;
use wow_project::graph::{ProjectSourceReadLimits, RetainedProjectSourceManifest};

use super::{GraphReadFailure, GraphReadStage, checkpoint, value};
use crate::ServiceErrorCode;

pub(super) struct SourceRead<'a> {
    pub manifest: &'a RetainedProjectSourceManifest,
    pub root: &'a Path,
    pub limits: ProjectSourceReadLimits,
}

impl SourceRead<'_> {
    pub fn execute(
        &self,
        retained: &GraphResolvedExplanation<'_>,
        catalog: &GraphEvidenceCatalog,
        max_output_bytes: usize,
        stop: &AtomicBool,
    ) -> Result<(serde_json::Value, bool), GraphReadFailure> {
        self.limits.validate().map_err(owner_error)?;
        checkpoint(stop)?;
        // The original explanation budget bounds the entire combined payload.
        // Never rerun the graph query with different limits to make room.
        let remaining = max_output_bytes
            .checked_sub(encoded_len(retained, max_output_bytes)?)
            .and_then(|n| n.checked_sub(b",\"source_read\":".len()))
            .ok_or_else(budget)?;
        // No descendant can be selected by a host path in request/bundle data.
        // Opening this explicit root registers the sole ambient capability.
        let directory = ProjectInputDirectory::open(self.root).map_err(owner_error)?;
        let report = self
            .manifest
            .read_sources(
                &directory,
                catalog,
                retained.evidence_resolution().source_handles(),
                self.limits,
                remaining,
                stop,
            )
            .map_err(owner_error)?;
        // All filesystem resources close before transport serialization/output.
        drop(directory);
        checkpoint(stop)?;
        let truncated = !report.truncations().is_empty();
        #[derive(Serialize)]
        struct Payload<'a, 'b> {
            #[serde(flatten)]
            retained: &'a GraphResolvedExplanation<'b>,
            source_read: wow_project::graph::ProjectSourceReadReport,
        }
        let payload = Payload {
            retained,
            source_read: report,
        };
        encoded_len(&payload, max_output_bytes)?;
        Ok((value(&payload)?, truncated))
    }
}
fn encoded_len(value: &impl Serialize, limit: usize) -> Result<usize, GraphReadFailure> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("source explanation limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value).map_err(|_| budget())?;
    Ok(counter.bytes)
}
fn budget() -> GraphReadFailure {
    GraphReadFailure::service(GraphReadStage::Source, ServiceErrorCode::BudgetExceeded)
}
fn owner_error(error: wow_project::ProjectError) -> GraphReadFailure {
    use wow_project::ProjectErrorCode as Code;
    GraphReadFailure::service(
        GraphReadStage::Source,
        match error.code() {
            Code::SourceReadCancelled | Code::AnalysisCancelled => ServiceErrorCode::Cancelled,
            Code::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
            Code::SourceRegistryInvalid | Code::SourceHandleInvalid => {
                ServiceErrorCode::IdentityMismatch
            }
            _ => ServiceErrorCode::InvalidRequest,
        },
    )
}
