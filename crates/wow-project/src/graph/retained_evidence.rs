//! Admission of the evidence-bearing projection of a retained source-graph receipt.
//! Other sidecars are deliberately not reconstructed as live project/analyzer owners.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::Deserialize;
use wow_core::{
    EvidenceId, EvidenceRecord, GenerationContext, NormalizedSourcePath, SourceHandle,
    SourceSpanKind, StableHandleId,
};
use wow_graph::{GraphEvidenceCatalog, GraphPartitionSnapshot, GraphProposalValue};

use super::{MAX_FILES, ProjectGraphFile, SOURCE_GRAPH_PARTITION, SOURCE_GRAPH_PROFILE};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

/// A source-manifest projection, not a deserialized ProjectView. Its enclosing
/// graph-build receipt must be integrity-checked by the service before use.
/// Unknown fields are other retained receipts, not additional authority.
#[derive(Debug, Deserialize)]
pub struct RetainedProjectGraphEvidence {
    profile: String,
    context: GenerationContext,
    files: Vec<ProjectGraphFile>,
    source_handles: BTreeMap<StableHandleId, SourceHandle>,
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
}
impl RetainedProjectGraphEvidence {
    /// Cross-check original path/digest/length and proposal support before the
    /// graph owner's generic record/DAG admission. No source bytes are opened.
    pub fn admit(
        self,
        owner: &GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<GraphEvidenceCatalog> {
        self.admit_with_sources(owner, stop)
            .map(|(catalog, _)| catalog)
    }

    /// Admit the same catalog and retain its validated, closed source manifest
    /// for an explicitly authorized local read-back. Does not open a directory.
    pub fn admit_with_sources(
        self,
        owner: &GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<(GraphEvidenceCatalog, super::RetainedProjectSourceManifest)> {
        crate::analyzer::checkpoint(stop)?;
        if self.profile != SOURCE_GRAPH_PROFILE
            || self.files.is_empty()
            || self.files.len() > MAX_FILES
            || self.context.context_id() != owner.source_context_id()
            || self.context.project_generation().is_none()
        {
            return Err(invalid());
        }
        self.context.validate().map_err(|_| invalid())?;
        owner.validate(stop).map_err(|error| {
            ProjectError::new(
                match error.code() {
                    wow_graph::GraphErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
                    wow_graph::GraphErrorCode::BudgetExceeded => {
                        ProjectErrorCode::SourceBudgetExceeded
                    }
                    _ => ProjectErrorCode::SourceRegistryInvalid,
                },
                ProjectPhase::View,
                "retained graph failed source-evidence admission",
            )
        })?;
        let source_partition = owner
            .partitions()
            .iter()
            .find(|partition| partition.partition_id() == SOURCE_GRAPH_PARTITION)
            .ok_or_else(invalid)?;
        let mut files = BTreeMap::new();
        let mut proposals = BTreeSet::new();
        for file in &self.files {
            crate::analyzer::checkpoint(stop)?;
            file.path
                .parse::<NormalizedSourcePath>()
                .map_err(|_| invalid())?;
            if file.byte_length > 9_007_199_254_740_991
                || files.insert(file.path.as_str(), file).is_some()
                || !proposals.insert(file.proposal_id.as_str())
            {
                return Err(invalid());
            }
            let proposal = source_partition
                .batch()
                .entity_proposal(&file.proposal_id)
                .ok_or_else(invalid)?;
            if proposal.entity_kind_id() != "source_file"
                || proposal.semantic_key().get("path")
                    != Some(&GraphProposalValue::String(file.path.clone().into()))
                || !proposal
                    .source_handle_ids()
                    .contains(&file.source_handle_id)
                || !proposal.evidence_ids().contains(&file.evidence_id)
            {
                return Err(invalid());
            }
            let base = self
                .source_handles
                .get(&file.source_handle_id)
                .ok_or_else(invalid)?;
            let evidence = self.evidence.get(&file.evidence_id).ok_or_else(invalid)?;
            if base.path().as_str() != file.path
                || base.span().kind() != SourceSpanKind::WholeFile
                || *base.content_digest() != file.content_digest
                || !evidence
                    .source_handle_ids()
                    .contains(&file.source_handle_id)
            {
                return Err(invalid());
            }
        }
        // Ensure the manifest did not omit an accepted captured source file.
        if source_partition
            .batch()
            .entity_proposals()
            .iter()
            .filter(|proposal| proposal.entity_kind_id() == "source_file")
            .count()
            != files.len()
        {
            return Err(invalid());
        }
        for handle in self.source_handles.values() {
            crate::analyzer::checkpoint(stop)?;
            let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
            let base = self
                .source_handles
                .get(&file.source_handle_id)
                .ok_or_else(invalid)?;
            if *handle.content_digest() != file.content_digest
                || handle.origin_kind() != base.origin_kind()
                || handle.origin_id() != base.origin_id()
                || handle.revision() != base.revision()
                || handle.project_generation() != self.context.project_generation()
                || handle.reference_generation() != Some(self.context.reference_generation())
                || handle
                    .span()
                    .byte_end()
                    .is_some_and(|end| end > file.byte_length)
            {
                return Err(invalid());
            }
        }
        let context_id = self.context.context_id();
        let catalog =
            GraphEvidenceCatalog::new(self.context, self.evidence, self.source_handles, stop)
                .map_err(|error| {
                    ProjectError::new(
                        match error.code() {
                            wow_graph::GraphErrorCode::Cancelled => {
                                ProjectErrorCode::AnalysisCancelled
                            }
                            wow_graph::GraphErrorCode::BudgetExceeded => {
                                ProjectErrorCode::SourceBudgetExceeded
                            }
                            _ => ProjectErrorCode::SourceRegistryInvalid,
                        },
                        ProjectPhase::View,
                        "retained graph evidence failed owner admission",
                    )
                })?;
        let sources = super::RetainedProjectSourceManifest::from_admitted(
            context_id,
            catalog.digest().into(),
            self.files,
        );
        Ok((catalog, sources))
    }
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::View,
        "retained source evidence disagrees with its manifest or graph",
    )
}
