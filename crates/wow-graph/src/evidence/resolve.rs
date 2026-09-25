use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{EvidenceId, EvidenceRecord, SourceHandle, StableHandleId};

use super::{
    GraphEvidenceCatalog, GraphEvidenceResolveLimits, budget, checkpoint, digest, encoded_len,
};
use crate::{
    GraphAssertionSupport, GraphExplainQuery, GraphExplainedRecord, GraphExplanation,
    GraphPartitionSnapshot, GraphResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEvidenceTruncation {
    EvidenceRecords,
    SourceHandles,
    DerivationDepth,
    Work,
    OutputBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphUnresolvedEvidenceReason {
    NotRetained,
    UnsupportedIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphUnresolvedEvidence {
    Evidence {
        id: Box<str>,
        reason: GraphUnresolvedEvidenceReason,
    },
    SourceHandle {
        id: StableHandleId,
        reason: GraphUnresolvedEvidenceReason,
    },
}

/// Shared dependencies/sources appear once. Every retained record stays intact;
/// input IDs and coverage references remain on its original EvidenceRecord.
#[derive(Debug, Serialize)]
pub struct GraphEvidenceResolution<'a> {
    catalog_digest: &'a str,
    context: &'a wow_core::GenerationContext,
    limits: GraphEvidenceResolveLimits,
    evidence_records: Vec<&'a EvidenceRecord>,
    source_handles: Vec<&'a SourceHandle>,
    unresolved: Vec<GraphUnresolvedEvidence>,
    requested_evidence_roots: usize,
    requested_source_roots: usize,
    known_evidence: usize,
    known_sources: usize,
    known_pending_evidence: usize,
    known_pending_sources: usize,
    work: u32,
    closure_complete: bool,
    supports_complete: bool,
    truncations: BTreeSet<GraphEvidenceTruncation>,
    source_bytes_verified: bool,
    coverage_references_resolved: bool,
}
impl GraphEvidenceResolution<'_> {
    pub fn closure_complete(&self) -> bool {
        self.closure_complete
    }
    pub fn truncations(&self) -> &BTreeSet<GraphEvidenceTruncation> {
        &self.truncations
    }
    pub fn unresolved(&self) -> &[GraphUnresolvedEvidence] {
        &self.unresolved
    }
    pub fn evidence_records(&self) -> &[&EvidenceRecord] {
        &self.evidence_records
    }
    pub fn source_handles(&self) -> &[&SourceHandle] {
        &self.source_handles
    }
}

#[derive(Debug, Serialize)]
pub struct GraphResolvedExplanation<'a> {
    schema: &'static str,
    query_digest: Box<str>,
    explanation: GraphExplanation<'a>,
    evidence_resolution: GraphEvidenceResolution<'a>,
}
impl GraphResolvedExplanation<'_> {
    pub fn explanation(&self) -> &GraphExplanation<'_> {
        &self.explanation
    }
    pub fn evidence_resolution(&self) -> &GraphEvidenceResolution<'_> {
        &self.evidence_resolution
    }
}

impl GraphExplainQuery {
    /// One owner query over an immutable snapshot and admitted evidence catalog.
    /// Source bytes are never reopened. Evidence closure does not imply full
    /// graph coverage, provenance authentication, conflict resolution or runtime.
    pub fn execute_with_evidence<'a>(
        &self,
        owner: &'a GraphPartitionSnapshot,
        catalog: &'a GraphEvidenceCatalog,
        limits: GraphEvidenceResolveLimits,
        stop: &AtomicBool,
    ) -> GraphResult<GraphResolvedExplanation<'a>> {
        checkpoint(stop)?;
        limits.validate()?;
        if owner.source_context_id() != catalog.context().context_id() {
            return Err(super::invalid());
        }
        let explanation = self.execute(owner, stop)?;
        let mut evidence_roots = BTreeSet::new();
        let mut source_roots = BTreeSet::new();
        let mut work = 0u32;
        collect_roots(
            &explanation,
            &mut evidence_roots,
            &mut source_roots,
            &mut work,
            limits.max_work,
            stop,
        )?;
        let supports_complete = explanation.support_complete();
        let mut result = GraphResolvedExplanation {
            schema: "wow-graph/resolved-explanation/e2-a/1",
            query_digest: digest(
                "graph-resolved-explanation-query/1",
                &(self, limits, catalog.digest()),
            )?,
            explanation,
            evidence_resolution: GraphEvidenceResolution {
                catalog_digest: catalog.digest(),
                context: catalog.context(),
                limits,
                evidence_records: Vec::new(),
                source_handles: Vec::new(),
                unresolved: Vec::new(),
                requested_evidence_roots: evidence_roots.len(),
                requested_source_roots: source_roots.len(),
                known_evidence: evidence_roots.len(),
                known_sources: source_roots.len(),
                known_pending_evidence: 0,
                known_pending_sources: 0,
                work,
                closure_complete: false,
                supports_complete,
                truncations: BTreeSet::new(),
                source_bytes_verified: false,
                coverage_references_resolved: false,
            },
        };
        let max_bytes = self.limits().max_output_bytes as usize;
        // Reserve counter growth, the full fixed truncation vocabulary and the
        // changed boundary marker. Count entire entries before retaining them.
        let mut remaining = max_bytes
            .checked_sub(encoded_len(&result, max_bytes)?)
            .and_then(|n| n.checked_sub(1024))
            .ok_or_else(budget)?;
        resolve(
            &mut result.evidence_resolution,
            catalog,
            evidence_roots,
            source_roots,
            &mut remaining,
            stop,
        )?;
        result.explanation.mark_evidence_resolution(
            result.evidence_resolution.closure_complete && supports_complete,
        );
        checkpoint(stop)?;
        encoded_len(&result, max_bytes)?;
        Ok(result)
    }
}

fn tick(work: &mut u32, max: u32, stop: &AtomicBool) -> GraphResult<()> {
    checkpoint(stop)?;
    *work = work
        .checked_add(1)
        .filter(|n| *n <= max)
        .ok_or_else(budget)?;
    Ok(())
}
fn collect_roots(
    explanation: &GraphExplanation<'_>,
    evidence: &mut BTreeSet<Box<str>>,
    sources: &mut BTreeSet<StableHandleId>,
    work: &mut u32,
    max: u32,
    stop: &AtomicBool,
) -> GraphResult<()> {
    let records: Vec<&[Box<str>]> = match explanation.record() {
        GraphExplainedRecord::Entity { node } => vec![node.evidence_ids()],
        GraphExplainedRecord::Relation {
            edge,
            source,
            target,
        } => vec![
            edge.evidence_ids(),
            source.evidence_ids(),
            target.evidence_ids(),
        ],
    };
    for ids in records {
        for id in ids {
            tick(work, max, stop)?;
            evidence.insert(id.clone());
        }
    }
    for support in explanation.supports() {
        let (evidence_ids, source_ids) = match support {
            GraphAssertionSupport::ProducerEntity { proposal, .. } => {
                (proposal.evidence_ids(), proposal.source_handle_ids())
            }
            GraphAssertionSupport::ProducerRelation { proposal, .. } => {
                (proposal.evidence_ids(), proposal.source_handle_ids())
            }
            GraphAssertionSupport::FoundationEntity { node } => {
                for id in node.evidence_ids() {
                    tick(work, max, stop)?;
                    evidence.insert(id.clone());
                }
                continue;
            }
            GraphAssertionSupport::FoundationRelation { edge } => {
                for id in edge.evidence_ids() {
                    tick(work, max, stop)?;
                    evidence.insert(id.clone());
                }
                continue;
            }
        };
        for id in evidence_ids {
            tick(work, max, stop)?;
            evidence.insert(id.to_string().into());
        }
        for id in source_ids {
            tick(work, max, stop)?;
            sources.insert(*id);
        }
    }
    Ok(())
}

fn retain_bytes(value: &impl Serialize, remaining: &mut usize) -> bool {
    let Ok(bytes) = encoded_len(value, *remaining) else {
        return false;
    };
    let Some(left) = remaining.checked_sub(bytes.saturating_add(1)) else {
        return false;
    };
    *remaining = left;
    true
}

fn resolve<'a>(
    output: &mut GraphEvidenceResolution<'a>,
    catalog: &'a GraphEvidenceCatalog,
    evidence_roots: BTreeSet<Box<str>>,
    source_roots: BTreeSet<StableHandleId>,
    remaining: &mut usize,
    stop: &AtomicBool,
) -> GraphResult<()> {
    let mut scheduled = evidence_roots.clone();
    let mut queue: VecDeque<_> = evidence_roots.into_iter().map(|id| (id, 0u32)).collect();
    let mut sources = source_roots;
    let mut deferred_depth = BTreeSet::new();
    while let Some((id, depth)) = queue.pop_front() {
        checkpoint(stop)?;
        if output.work >= output.limits.max_work {
            queue.push_front((id, depth));
            output.truncations.insert(GraphEvidenceTruncation::Work);
            break;
        }
        output.work += 1;
        let parsed = id.parse::<EvidenceId>();
        let record = parsed.as_ref().ok().and_then(|id| catalog.evidence(id));
        let Some(record) = record else {
            let item = GraphUnresolvedEvidence::Evidence {
                id: id.clone(),
                reason: if parsed.is_ok() {
                    GraphUnresolvedEvidenceReason::NotRetained
                } else {
                    GraphUnresolvedEvidenceReason::UnsupportedIdentifier
                },
            };
            if !retain_bytes(&item, remaining) {
                queue.push_front((id, depth));
                output
                    .truncations
                    .insert(GraphEvidenceTruncation::OutputBytes);
                break;
            }
            output.unresolved.push(item);
            continue;
        };
        if output.evidence_records.len() >= output.limits.max_evidence_records as usize {
            queue.push_front((id, depth));
            output
                .truncations
                .insert(GraphEvidenceTruncation::EvidenceRecords);
            break;
        }
        if !retain_bytes(record, remaining) {
            queue.push_front((id, depth));
            output
                .truncations
                .insert(GraphEvidenceTruncation::OutputBytes);
            break;
        }
        output.evidence_records.push(record);
        // Charge each link, including duplicate ones. A record is retained whole
        // even if link expansion is stopped; no truncated record is fabricated.
        for source in record.source_handle_ids() {
            if output.work >= output.limits.max_work {
                output.truncations.insert(GraphEvidenceTruncation::Work);
                break;
            }
            output.work += 1;
            checkpoint(stop)?;
            sources.insert(*source);
        }
        for input in record.derivation_input_ids() {
            if output.work >= output.limits.max_work {
                output.truncations.insert(GraphEvidenceTruncation::Work);
                break;
            }
            output.work += 1;
            checkpoint(stop)?;
            let key: Box<str> = input.to_string().into();
            if scheduled.contains(&key) {
                continue;
            }
            if depth >= output.limits.max_derivation_depth {
                deferred_depth.insert(key);
            } else {
                deferred_depth.remove(&key);
                scheduled.insert(key.clone());
                queue.push_back((key, depth + 1));
            }
        }
        if output.truncations.contains(&GraphEvidenceTruncation::Work) {
            break;
        }
    }
    // A dependency skipped at one depth may already have been reached by a
    // shorter path. Only unresolved depth frontiers count as truncation.
    deferred_depth.retain(|id| !scheduled.contains(id));
    if !deferred_depth.is_empty() {
        output
            .truncations
            .insert(GraphEvidenceTruncation::DerivationDepth);
    }
    output.known_evidence = scheduled.len() + deferred_depth.len();
    output.known_pending_evidence = queue.len() + deferred_depth.len();
    output.known_sources = sources.len();
    let mut known_pending_sources = sources.len();
    for id in sources {
        checkpoint(stop)?;
        if output.work >= output.limits.max_work {
            output.truncations.insert(GraphEvidenceTruncation::Work);
            break;
        }
        output.work += 1;
        if let Some(handle) = catalog.source_handle(&id) {
            if output.source_handles.len() >= output.limits.max_source_handles as usize {
                output
                    .truncations
                    .insert(GraphEvidenceTruncation::SourceHandles);
                break;
            }
            if !retain_bytes(handle, remaining) {
                output
                    .truncations
                    .insert(GraphEvidenceTruncation::OutputBytes);
                break;
            }
            output.source_handles.push(handle);
        } else {
            let item = GraphUnresolvedEvidence::SourceHandle {
                id,
                reason: GraphUnresolvedEvidenceReason::NotRetained,
            };
            if !retain_bytes(&item, remaining) {
                output
                    .truncations
                    .insert(GraphEvidenceTruncation::OutputBytes);
                break;
            }
            output.unresolved.push(item);
        }
        known_pending_sources -= 1;
    }
    output.known_pending_sources = known_pending_sources;
    output
        .evidence_records
        .sort_by_key(|record| record.evidence_id());
    output.closure_complete = output.truncations.is_empty() && output.unresolved.is_empty();
    checkpoint(stop)
}
