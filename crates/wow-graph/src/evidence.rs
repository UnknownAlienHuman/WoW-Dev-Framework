//! Validated, context-bound retained evidence. No source acquisition or inference.
mod resolve;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{EvidenceId, EvidenceRecord, GenerationContext, SourceHandle, StableHandleId};

use crate::{GraphError, GraphErrorCode, GraphResult};
pub use resolve::{
    GraphEvidenceResolution, GraphEvidenceTruncation, GraphResolvedExplanation,
    GraphUnresolvedEvidence, GraphUnresolvedEvidenceReason,
};

const MAX_RECORDS: usize = 131_072;
const MAX_REFERENCES: usize = 1_000_000;
const MAX_CATALOG_BYTES: usize = 32 * 1024 * 1024;

/// Construction validates every retained record, not only the requested roots.
/// Missing graph references remain query outcomes. Retained EvidenceRecords must
/// themselves form a closed, valid derivation DAG with present source handles.
/// This is content integrity, never proof of origin authenticity or source bytes.
#[derive(Debug)]
pub struct GraphEvidenceCatalog {
    context: GenerationContext,
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    source_handles: BTreeMap<StableHandleId, SourceHandle>,
    digest: Box<str>,
}
impl GraphEvidenceCatalog {
    pub fn new(
        context: GenerationContext,
        evidence: BTreeMap<EvidenceId, EvidenceRecord>,
        source_handles: BTreeMap<StableHandleId, SourceHandle>,
        stop: &AtomicBool,
    ) -> GraphResult<Self> {
        checkpoint(stop)?;
        if evidence.len() > MAX_RECORDS || source_handles.len() > MAX_RECORDS {
            return Err(budget());
        }
        let identity = (&context, &evidence, &source_handles);
        encoded_len(&("graph-evidence-catalog/1", &identity), MAX_CATALOG_BYTES)?;
        context.validate().map_err(|_| invalid())?;
        for (id, handle) in &source_handles {
            checkpoint(stop)?;
            if *id != handle.handle_id()
                || handle
                    .reference_generation()
                    .is_some_and(|generation| generation != context.reference_generation())
                || handle
                    .project_generation()
                    .is_some_and(|generation| Some(generation) != context.project_generation())
            {
                return Err(invalid());
            }
            handle.validate().map_err(|_| invalid())?;
        }
        let producers: BTreeMap<_, _> = context
            .producer_versions()
            .iter()
            .map(|entry| (entry.producer_id().clone(), entry.version()))
            .collect();
        let mut references = 0usize;
        for (id, record) in &evidence {
            checkpoint(stop)?;
            if *id != record.evidence_id()
                || record.context_id() != context.context_id()
                || producers.get(record.producer_id()).copied() != Some(record.producer_version())
            {
                return Err(invalid());
            }
            references = references
                .checked_add(record.source_handle_ids().len())
                .and_then(|n| n.checked_add(record.derivation_input_ids().len()))
                .and_then(|n| n.checked_add(record.coverage_refs().len()))
                .ok_or_else(budget)?;
            if references > MAX_REFERENCES
                || record
                    .source_handle_ids()
                    .iter()
                    .any(|id| !source_handles.contains_key(id))
                || record
                    .derivation_input_ids()
                    .iter()
                    .any(|id| !evidence.contains_key(id))
            {
                return Err(if references > MAX_REFERENCES {
                    budget()
                } else {
                    invalid()
                });
            }
        }
        // Reuse the core validator for IDs, sorted references, confidence ceilings,
        // cycles and runtime-ancestry restrictions instead of a second DAG policy.
        let records: Vec<_> = evidence.values().cloned().collect();
        wow_core::validate_evidence_derivation_graph(&records).map_err(|_| invalid())?;
        checkpoint(stop)?;
        let digest = digest("graph-evidence-catalog/1", &identity)?;
        checkpoint(stop)?;
        Ok(Self {
            context,
            evidence,
            source_handles,
            digest,
        })
    }
    pub fn context(&self) -> &GenerationContext {
        &self.context
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn evidence(&self, id: &EvidenceId) -> Option<&EvidenceRecord> {
        self.evidence.get(id)
    }
    pub fn source_handle(&self, id: &StableHandleId) -> Option<&SourceHandle> {
        self.source_handles.get(id)
    }
}

/// Independent work/record caps. The *combined* explanation and resolved records
/// must also fit the original GraphExplainLimits.max_output_bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEvidenceResolveLimits {
    pub max_evidence_records: u32,
    pub max_source_handles: u32,
    pub max_derivation_depth: u32,
    pub max_work: u32,
}
impl Default for GraphEvidenceResolveLimits {
    fn default() -> Self {
        Self {
            max_evidence_records: 1024,
            max_source_handles: 4096,
            max_derivation_depth: 32,
            max_work: 100_000,
        }
    }
}
impl GraphEvidenceResolveLimits {
    pub fn validate(self) -> GraphResult<()> {
        if !(1..=16_384).contains(&self.max_evidence_records)
            || !(1..=65_536).contains(&self.max_source_handles)
            || self.max_derivation_depth > 256
            || !(1..=1_000_000).contains(&self.max_work)
        {
            return Err(GraphError::new(
                GraphErrorCode::LimitsInvalid,
                "evidence resolution limits are outside the bounded profile",
            ));
        }
        Ok(())
    }
}

pub(super) fn checkpoint(stop: &AtomicBool) -> GraphResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "evidence read cancelled",
        ))
    } else {
        Ok(())
    }
}
fn invalid() -> GraphError {
    GraphError::new(
        GraphErrorCode::EvidenceInvalid,
        "retained evidence catalog is inconsistent",
    )
}
pub(super) fn budget() -> GraphError {
    GraphError::new(
        GraphErrorCode::BudgetExceeded,
        "evidence read exceeds its bounded profile",
    )
}
pub(super) fn digest(schema: &str, value: &impl Serialize) -> GraphResult<Box<str>> {
    let bytes = wow_core::canonical_json_bytes(&(schema, value)).map_err(|_| invalid())?;
    let hash = Sha256::digest(bytes);
    let hex = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("{schema}:sha256:{hex}").into())
}

/// Compact and canonical JSON have equal byte lengths for these typed records;
/// count before allocation. Ordering affects byte order, not serialized length.
pub(super) fn encoded_len(value: &impl Serialize, limit: usize) -> GraphResult<usize> {
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
                .ok_or_else(|| std::io::Error::other("evidence byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut count, value).map_err(|_| budget())?;
    Ok(count.bytes)
}
