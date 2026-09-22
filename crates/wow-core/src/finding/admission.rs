//! One immutable, registry-local admission shared by diagnostics and envelopes.
//! Coverage/source eligibility still belongs to the complete context/owner joins.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::validation_error;
use crate::{
    CoreErrorCode, CoreResult, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContextId,
    SourceHandle, StableHandleId, validate_evidence_derivation_graph,
};

pub(crate) struct DiagnosticRegistry<'a> {
    context_id: GenerationContextId,
    sources: BTreeSet<StableHandleId>,
    evidence: BTreeMap<EvidenceId, &'a EvidenceRecord>,
}

impl<'a> DiagnosticRegistry<'a> {
    pub(crate) fn new(
        context_id: GenerationContextId,
        source_handles: &[SourceHandle],
        evidence_records: &'a [EvidenceRecord],
        operation: &'static str,
    ) -> CoreResult<Self> {
        let mut handles: Vec<_> = source_handles.iter().collect();
        handles.sort_unstable_by_key(|handle| handle.handle_id());
        if handles
            .windows(2)
            .any(|pair| pair[0].handle_id() == pair[1].handle_id())
        {
            return Err(validation_error(
                operation,
                CoreErrorCode::ResultDuplicateId,
                "source_handles",
            ));
        }
        for handle in &handles {
            handle.validate()?;
        }
        if evidence_records
            .iter()
            .any(|record| record.context_id() != context_id)
        {
            return Err(validation_error(
                operation,
                CoreErrorCode::EvidenceContextMismatch,
                "evidence_records.context_id",
            ));
        }
        // Validate all retained evidence, including ancestors, before reading
        // confidence or accepting a supplied ID. No JSON/string confidence proxy.
        validate_evidence_derivation_graph(evidence_records)?;
        let registry = Self {
            context_id,
            sources: handles.iter().map(|handle| handle.handle_id()).collect(),
            evidence: evidence_records
                .iter()
                .map(|record| (record.evidence_id(), record))
                .collect(),
        };
        for record in registry.evidence.values() {
            registry.require_sources(
                record.source_handle_ids().iter().copied(),
                operation,
                "evidence_records.source_handle_ids",
            )?;
        }
        Ok(registry)
    }

    pub(crate) const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    pub(crate) fn require_sources(
        &self,
        ids: impl IntoIterator<Item = StableHandleId>,
        operation: &'static str,
        field: &'static str,
    ) -> CoreResult<()> {
        if ids.into_iter().any(|id| !self.sources.contains(&id)) {
            return Err(validation_error(
                operation,
                CoreErrorCode::MissingSourceHandle,
                field,
            ));
        }
        Ok(())
    }

    pub(crate) fn require_evidence(
        &self,
        ids: &[EvidenceId],
        operation: &'static str,
    ) -> CoreResult<bool> {
        let mut candidate = false;
        for id in ids {
            let record = self.evidence.get(id).ok_or_else(|| {
                validation_error(
                    operation,
                    CoreErrorCode::MissingEvidenceReference,
                    "evidence_ids",
                )
            })?;
            candidate |= record.confidence() == EvidenceConfidence::Candidate;
        }
        Ok(candidate)
    }
}
