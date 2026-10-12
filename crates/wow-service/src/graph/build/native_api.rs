//! Selected native callable endpoints from original, requalified producer input.
//! The held Library is authority; serialized provenance is only its inert receipt.
use super::{MAX_BUNDLE_BYTES, checkpoint, graph_error, project_error};
use crate::local::{
    NativeApiTargetOmission, NativeApiTargetResolution, RequalifiedNativeLibrary,
    native_library_input::rebuild_for_project,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Serialize, ser::SerializeSeq};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::sync::atomic::AtomicBool;
use wow_core::{
    CanonicalResult, ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord,
    GenerationContext, ProducerId, ProvenanceClass, SourceContent, SourceHandle,
    SourceHandleBuilder, SourceOriginKind, SourceSpan, StableHandleId, ToolVersion,
    canonical_json_bytes,
};
use wow_emmy::{bindings::SymbolTarget, function_calls::SourceCallTarget};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphEntityProposal,
    GraphLocalAssertion, GraphPartitionReplacement, GraphPartitionSnapshot, GraphProposalBatch,
    GraphProposalValue,
};
use wow_project::{NativeApiUseRecipe, ProjectView};
use wow_recognizers::source_calls::{SOURCE_API_ENTITY_KIND, SourceApiTargetBinding};

pub(super) const NATIVE_API_PARTITION: &str = "wow-service.native-api-callables";
pub(super) const NATIVE_API_PROFILE: &str = "wow-service/native-api-callables/1";
// Same finite limits as the current owners: graph.rs MAX_TEXT_BYTES and
// local/native_library_input.rs BINDING_MAX_WORK. No new selected cap or schema.
const ORDINARY_METADATA_BYTES: u64 = 4 * 1024 * 1024;
const NATIVE_JOIN_MAX_WORK: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NativeApiIdentity {
    reference_profile_id: Box<str>,
    reference_generation_id: Box<str>,
    reference_digest: Box<str>,
    record_key: Box<str>,
    record_digest: Box<str>,
}

impl NativeApiIdentity {
    fn semantic_key(&self) -> BTreeMap<Box<str>, GraphProposalValue> {
        BTreeMap::from([
            (
                "reference_profile_id".into(),
                GraphProposalValue::String(self.reference_profile_id.clone()),
            ),
            (
                "reference_generation_id".into(),
                GraphProposalValue::String(self.reference_generation_id.clone()),
            ),
            (
                "reference_digest".into(),
                GraphProposalValue::String(self.reference_digest.clone()),
            ),
            (
                "record_key".into(),
                GraphProposalValue::String(self.record_key.clone()),
            ),
            (
                "record_digest".into(),
                GraphProposalValue::String(self.record_digest.clone()),
            ),
        ])
    }
}

/// Original accepted input-generation address. Final materialization is a
/// separate graph-owned binding; no generated name or owner_key is interpreted.
#[derive(Debug, Serialize)]
struct NativeApiEndpoint {
    identity: NativeApiIdentity,
    assertion: GraphAssertionRef,
    source_handle_id: StableHandleId,
    evidence_id: EvidenceId,
}

#[derive(Debug, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
enum NativeApiOmissionReason {
    NotObservedInNativeCallableReport,
    NoSourceMapping,
    NoNativeCallableOrigin,
    AmbiguousMapping {
        precision: wow_annotations::navigation::MappingPrecision,
        candidate_count: usize,
    },
    UnsupportedMapping {
        precision: wow_annotations::navigation::MappingPrecision,
    },
    AmbiguousOrigin,
    ReferenceConflict {
        record_key: Box<str>,
        record_digest: Box<str>,
        source_id: Box<str>,
    },
}

#[derive(Debug, Serialize)]
struct NativeApiOmission {
    target: SymbolTarget,
    reason: NativeApiOmissionReason,
}

/// Only publish() constructs this proof. It owns the original native results
/// without borrowing itself; replay data cannot construct or deserialize it.
#[derive(Serialize)]
pub(crate) struct NativeApiProvenance {
    profile: &'static str,
    project_snapshot_id: Box<str>,
    analyzer_snapshot_id: Box<str>,
    context: GenerationContext,
    source_handles: BTreeMap<StableHandleId, SourceHandle>,
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    endpoints: BTreeMap<Box<str>, NativeApiEndpoint>,
    #[serde(serialize_with = "serialize_bindings")]
    bindings: BTreeMap<SymbolTarget, Box<str>>,
    omissions: Vec<NativeApiOmission>,
    #[serde(skip)]
    library: RequalifiedNativeLibrary,
}

// Structured target tuples remain values, including during bounded counting.
// JSON object keys cannot represent SymbolTarget; preserve the map's exact order.
fn serialize_bindings<S: serde::Serializer>(
    bindings: &BTreeMap<SymbolTarget, Box<str>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut entries = serializer.serialize_seq(Some(bindings.len()))?;
    for entry in bindings {
        entries.serialize_element(&entry)?;
    }
    entries.end()
}

impl NativeApiProvenance {
    pub(crate) const fn context(&self) -> &GenerationContext {
        &self.context
    }

    pub(crate) const fn evidence(&self) -> &BTreeMap<EvidenceId, EvidenceRecord> {
        &self.evidence
    }

    pub(crate) const fn source_handles(&self) -> &BTreeMap<StableHandleId, SourceHandle> {
        &self.source_handles
    }

    /// Compare with the actual rebuilt owner, never a serialized endpoint identity.
    pub(crate) fn matches_reference(
        &self,
        profile: &wow_core::ProfileIdentity,
        reference: &wow_reference::ReferenceView,
    ) -> bool {
        self.library.profile() == profile && &self.library.reference().view == reference
    }

    /// The held native proof supplies the producer declaration; graph admission
    /// independently binds the exact original support to accepted final entities.
    pub(crate) fn source_observations<'a>(
        &'a self,
        graph: &wow_graph::GraphPartitionRead<'a>,
        stop: &AtomicBool,
    ) -> ServiceResult<wow_graph::GraphPartitionSourceObservations<'a>> {
        let producer = wow_core::ProducerVersionEntry::new(
            "wow.service.native_api"
                .parse::<ProducerId>()
                .map_err(|_| incompatible())?,
            ToolVersion::parse(env!("CARGO_PKG_VERSION")).map_err(|_| incompatible())?,
        );
        wow_graph::GraphPartitionSourceObservations::new(
            graph,
            NATIVE_API_PARTITION,
            producer,
            &self.evidence,
            &self.source_handles,
            stop,
        )
        .map_err(graph_error)
    }

    /// Bounded by the used, deduplicated call targets admitted in publish().
    /// This clones only target tuples/addresses; support and Library stay borrowed.
    /// The downstream recognizer independently proves each accepted address.
    pub fn api_bindings(
        &self,
        stop: &AtomicBool,
    ) -> ServiceResult<BTreeMap<SymbolTarget, SourceApiTargetBinding<'_>>> {
        checkpoint(stop)?;
        if self.library.project_snapshot_id() != self.project_snapshot_id.as_ref()
            || self.library.reference().view.generation_id()
                != self.context.reference_generation().to_string()
        {
            return Err(incompatible());
        }
        let mut result = BTreeMap::new();
        for (target, proposal_id) in &self.bindings {
            checkpoint(stop)?;
            let endpoint = self
                .endpoints
                .get(proposal_id.as_ref())
                .ok_or_else(incompatible)?;
            let source_handle = self
                .source_handles
                .get(&endpoint.source_handle_id)
                .ok_or_else(incompatible)?;
            let evidence = self
                .evidence
                .get(&endpoint.evidence_id)
                .ok_or_else(incompatible)?;
            if evidence.source_handle_ids() != [source_handle.handle_id()]
                || evidence.context_id() != self.context.context_id()
            {
                return Err(incompatible());
            }
            result.insert(
                target.clone(),
                SourceApiTargetBinding {
                    assertion: endpoint.assertion.clone(),
                    source_handle,
                    evidence,
                },
            );
        }
        Ok(result)
    }
}

#[derive(Serialize)]
struct PendingEndpoint {
    identity: NativeApiIdentity,
    source_handle_id: StableHandleId,
    evidence_id: EvidenceId,
}

pub(super) fn publish(
    project: &ProjectView,
    source: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, NativeApiProvenance)> {
    checkpoint(stop)?;
    let config = project.configuration();
    config.validate().map_err(project_error)?;
    let context = project.snapshot().generation_context();
    if config.native_api_uses() != Some(NativeApiUseRecipe::ReferenceV1)
        || source.source_context_id() != context.context_id()
        || source.partition(NATIVE_API_PARTITION).is_some()
    {
        return Err(custody_error("selection/context/predecessor"));
    }
    let kind = source
        .registry()
        .entity_kind(SOURCE_API_ENTITY_KIND)
        .ok_or_else(incompatible)?;
    const FIELDS: [&str; 5] = [
        "record_digest",
        "record_key",
        "reference_digest",
        "reference_generation_id",
        "reference_profile_id",
    ];
    if kind.identity_fields().iter().map(AsRef::as_ref).ne(FIELDS)
        || !kind.allows_universe(source.foundation().universe())
        || !kind.allows_confidence(GraphConfidence::Derived)
    {
        return Err(custody_error("selected API registry"));
    }
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or_else(incompatible)?;
    let mut budget = Budget::new(project)?;
    budget.work(report.calls().len(), stop)?;
    // One genuine reconstruction and one prepared resolver, never one per call.
    let library = rebuild_for_project(project, stop)?;
    let mut source_handles = BTreeMap::new();
    let mut evidence = BTreeMap::new();
    let mut pending: BTreeMap<Box<str>, PendingEndpoint> = BTreeMap::new();
    let mut bindings = BTreeMap::new();
    let mut omissions = Vec::new();
    {
        let mut mapping_counts = BTreeMap::new();
        for file in library.sidecars().source_map().files() {
            budget.work(file.mappings().len(), stop)?;
            budget.reserve(&(file.path(), file.mappings().len()), stop)?;
            if mapping_counts
                .insert(file.path(), file.mappings().len())
                .is_some()
            {
                return Err(incompatible());
            }
        }
        let resolver = library.target_resolver(project, stop)?;
        let mut seen = BTreeSet::new();
        for call in report.calls() {
            checkpoint(stop)?;
            let SourceCallTarget::LibraryFunction { target } = call.target() else {
                continue;
            };
            if seen.contains(target) {
                continue;
            }
            // Before either a borrowed index entry or an owned tuple is retained.
            budget.reserve(target, stop)?;
            seen.insert(target);
            let mappings = *mapping_counts
                .get(target.path.as_str())
                .ok_or_else(incompatible)?;
            // Reserve the actual four bounded mapping populations upstream:
            // navigation's best-pass, tie-pass, candidate-copy, resolver's exact
            // mapping scan. Each is <= the held target file's mapping count.
            budget.work(mappings, stop)?;
            budget.work(mappings, stop)?;
            budget.work(mappings, stop)?;
            budget.work(mappings, stop)?;
            // This is an upper-bound reservation, NOT measured target work:
            // build() includes digest steps for every exact original candidate;
            // resolve_target() may digest one Found record from that same set.
            budget.work(library.callable_bindings().work(), stop)?;
            let outcome = resolver.resolve_target(target, stop)?;
            let reason = match outcome {
                NativeApiTargetResolution::Bound(association) => {
                    let native = association.binding();
                    let identity = NativeApiIdentity {
                        reference_profile_id: config
                            .selected_profile()
                            .profile_id()
                            .as_str()
                            .into(),
                        reference_generation_id: library.reference().view.generation_id().into(),
                        reference_digest: library.reference().view.self_digest().into(),
                        record_key: native.key().into(),
                        record_digest: native.record_digest().into(),
                    };
                    budget.reserve(&identity, stop)?;
                    // Identity material is bounded before canonical allocation.
                    checkpoint(stop)?;
                    let encoded = canonical_json_bytes(&(NATIVE_API_PROFILE, &identity))
                        .map_err(|_| incompatible())?;
                    checkpoint(stop)?;
                    let digest = ContentDigest::<CanonicalResult>::from_bytes(
                        Sha256::digest(&encoded).into(),
                    );
                    let proposal_id: Box<str> = format!("native-api:{digest}").into();
                    budget.reserve(&proposal_id, stop)?;
                    let raw = native.source();
                    if raw.scope().is_some() || raw.span().start() >= raw.span().end() {
                        return Err(incompatible());
                    }
                    let span = SourceSpan::byte_range(raw.span().start(), raw.span().end())
                        .map_err(|_| incompatible())?;
                    let digest = raw
                        .sha256()
                        .parse::<ContentDigest<SourceContent>>()
                        .map_err(|_| custody_error("original source digest"))?;
                    let handle = SourceHandleBuilder::new(
                        SourceOriginKind::ReferencePack,
                        native.source_id(),
                        library.callable_bindings().revision(),
                        raw.path(),
                        span,
                        digest,
                    )
                    .reference_generation(context.reference_generation())
                    .build()
                    .map_err(|_| custody_error("original source handle"))?;
                    let observation = EvidenceRecord::new(
                        context.context_id(),
                        ProvenanceClass::PlatformSource,
                        EvidenceConfidence::Proven,
                        ClaimScope::SourceObservation,
                        "wow.service.native_api"
                            .parse::<ProducerId>()
                            .map_err(|_| custody_error("producer identifier"))?,
                        ToolVersion::parse(env!("CARGO_PKG_VERSION"))
                            .map_err(|_| custody_error("producer version"))?,
                        vec![handle.handle_id()],
                        Vec::new(),
                        Vec::new(),
                    )
                    .map_err(|_| custody_error("original source evidence"))?;
                    checkpoint(stop)?;
                    let row = PendingEndpoint {
                        identity,
                        source_handle_id: handle.handle_id(),
                        evidence_id: observation.evidence_id(),
                    };
                    if let Some(previous) = pending.get(proposal_id.as_ref()) {
                        if previous.identity != row.identity
                            || previous.source_handle_id != row.source_handle_id
                            || previous.evidence_id != row.evidence_id
                        {
                            return Err(incompatible());
                        }
                    } else {
                        if pending.len() >= source.snapshot().limits().max_nodes as usize {
                            return Err(exhausted());
                        }
                        budget.reserve(&row, stop)?;
                        budget.reserve(&handle, stop)?;
                        budget.reserve(&observation, stop)?;
                        if source_handles
                            .get(&handle.handle_id())
                            .is_some_and(|old| old != &handle)
                            || evidence
                                .get(&observation.evidence_id())
                                .is_some_and(|old| old != &observation)
                        {
                            return Err(incompatible());
                        }
                        source_handles.insert(handle.handle_id(), handle);
                        evidence.insert(observation.evidence_id(), observation);
                        pending.insert(proposal_id.clone(), row);
                    }
                    budget.reserve(&(target, &proposal_id), stop)?;
                    bindings.insert(association.target().clone(), proposal_id);
                    continue;
                }
                NativeApiTargetResolution::Unmapped(reason) => match reason {
                    NativeApiTargetOmission::NotObservedInNativeCallableReport => {
                        NativeApiOmissionReason::NotObservedInNativeCallableReport
                    }
                    NativeApiTargetOmission::NoSourceMapping => {
                        NativeApiOmissionReason::NoSourceMapping
                    }
                    NativeApiTargetOmission::NoNativeCallableOrigin => {
                        NativeApiOmissionReason::NoNativeCallableOrigin
                    }
                },
                NativeApiTargetResolution::AmbiguousMapping {
                    precision,
                    candidate_count,
                } => NativeApiOmissionReason::AmbiguousMapping {
                    precision,
                    candidate_count,
                },
                NativeApiTargetResolution::UnsupportedMapping { precision } => {
                    NativeApiOmissionReason::UnsupportedMapping { precision }
                }
                NativeApiTargetResolution::AmbiguousOrigin => {
                    NativeApiOmissionReason::AmbiguousOrigin
                }
                NativeApiTargetResolution::ReferenceConflict { binding, .. } => {
                    NativeApiOmissionReason::ReferenceConflict {
                        record_key: binding.key().into(),
                        record_digest: binding.record_digest().into(),
                        source_id: binding.source_id().into(),
                    }
                }
            };
            budget.reserve(&(target, &reason), stop)?;
            omissions.push(NativeApiOmission {
                target: target.clone(),
                reason,
            });
        }
    } // Release resolver/borrowed target indexes before moving the genuine Library.

    let mut proposals = Vec::new();
    for (proposal_id, row) in &pending {
        checkpoint(stop)?;
        budget.reserve(&row.identity, stop)?;
        let proposal = GraphEntityProposal::new(
            proposal_id.clone(),
            SOURCE_API_ENTITY_KIND,
            row.identity.semantic_key(),
            GraphConfidence::Derived,
            vec![row.source_handle_id],
            vec![row.evidence_id],
            Vec::new(),
        )
        .map_err(graph_error)?;
        budget.reserve(&proposal, stop)?;
        proposals.push(proposal);
    }
    let batch = GraphProposalBatch::build(
        source.registry().bundle_id(),
        source.registry().registry_digest(),
        source.foundation().universe().clone(),
        source.foundation().generation().clone(),
        context.context_id(),
        NATIVE_API_PARTITION,
        proposals,
        Vec::new(),
    )
    .map_err(graph_error)?;
    budget.reserve(&batch, stop)?;
    checkpoint(stop)?;
    let prepared = source
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: source.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch,
                coverage: Vec::new(),
            },
            stop,
        )
        .map_err(graph_error)?;
    let owner = prepared.candidate();
    let partition = owner
        .partition(NATIVE_API_PARTITION)
        .ok_or_else(incompatible)?;
    budget.reserve(partition, stop)?;
    if !partition.report().ready_for_publication()
        || partition.report().accepted_entities().len() != pending.len()
        || !partition.report().accepted_relations().is_empty()
    {
        return Err(incompatible());
    }
    // One lookup for all rows, proving exact accepted input-generation addresses.
    let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
    let mut endpoints = BTreeMap::new();
    for (proposal_id, row) in pending {
        let assertion = GraphAssertionRef::Producer {
            partition_id: NATIVE_API_PARTITION.into(),
            batch_id: partition.batch().batch_id().into(),
            assertion: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: proposal_id.clone(),
            },
        };
        budget.work(1, stop)?;
        let resolved = lookup
            .entity(lookup.scope(), &assertion, stop)
            .map_err(graph_error)?;
        let proposal = resolved.proposal();
        if proposal.entity_kind_id() != SOURCE_API_ENTITY_KIND
            || proposal.semantic_key() != &row.identity.semantic_key()
            || proposal.confidence() != GraphConfidence::Derived
            || proposal.source_handle_ids() != [row.source_handle_id]
            || proposal.evidence_ids() != [row.evidence_id]
        {
            return Err(incompatible());
        }
        let endpoint = NativeApiEndpoint {
            identity: row.identity,
            assertion: resolved.reference(),
            source_handle_id: row.source_handle_id,
            evidence_id: row.evidence_id,
        };
        budget.reserve(&endpoint, stop)?;
        endpoints.insert(proposal_id, endpoint);
    }
    drop(lookup);
    // Reserve the consumer's owned tuple/address copies once before returning.
    // Support remains borrowed when api_bindings() constructs its finite map.
    for (target, proposal_id) in &bindings {
        let endpoint = endpoints
            .get(proposal_id.as_ref())
            .ok_or_else(incompatible)?;
        budget.reserve(&(target, &endpoint.assertion), stop)?;
    }
    let provenance = NativeApiProvenance {
        profile: NATIVE_API_PROFILE,
        project_snapshot_id: project.snapshot_id().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        context: context.clone(),
        source_handles,
        evidence,
        endpoints,
        bindings,
        omissions,
        library,
    };
    // Complete inert receipt; skipped Library retains its independent native bounds.
    budget.reserve(&provenance, stop)?;
    checkpoint(stop)?;
    Ok((owner.clone(), provenance))
}

/// Cumulative selected producer output/work. Reservations intentionally include
/// scratch batches + retained support + exact receipt + consumer address copies.
struct Budget {
    bytes: usize,
    max_bytes: usize,
    work: usize,
}

impl Budget {
    fn new(project: &ProjectView) -> ServiceResult<Self> {
        let policy = project.configuration().budget_policy();
        let metadata = policy
            .source_graph_budget()
            .map_or(ORDINARY_METADATA_BYTES, |budget| {
                budget.max_metadata_bytes()
            });
        let limit = metadata
            .min(policy.max_output_bytes())
            .min(MAX_BUNDLE_BYTES as u64);
        let max_bytes = usize::try_from(limit).map_err(|_| exhausted())?;
        if max_bytes == 0 {
            return Err(exhausted());
        }
        Ok(Self {
            bytes: 0,
            max_bytes,
            work: 0,
        })
    }

    fn work(&mut self, amount: usize, stop: &AtomicBool) -> ServiceResult<()> {
        checkpoint(stop)?;
        self.work = self.work.checked_add(amount).ok_or_else(exhausted)?;
        if self.work > NATIVE_JOIN_MAX_WORK {
            return Err(exhausted());
        }
        Ok(())
    }

    fn reserve<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        struct Counter<'a> {
            budget: &'a mut Budget,
            stop: &'a AtomicBool,
            failure: Option<ServiceErrorCode>,
        }
        impl Write for Counter<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if let Err(error) = self.budget.work(1, self.stop) {
                    self.failure = Some(error.code());
                    return Err(std::io::Error::other(
                        "native API producer work/cancellation",
                    ));
                }
                if bytes.len() > self.budget.max_bytes.saturating_sub(self.budget.bytes) {
                    self.failure = Some(ServiceErrorCode::BudgetExceeded);
                    return Err(std::io::Error::other("native API producer metadata"));
                }
                self.budget.bytes += bytes.len();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        checkpoint(stop)?;
        let mut counter = Counter {
            budget: self,
            stop,
            failure: None,
        };
        let encoded = serde_json::to_writer(&mut counter, value);
        checkpoint(stop)?;
        encoded.map_err(|_| {
            ServiceError::new(
                counter
                    .failure
                    .unwrap_or(ServiceErrorCode::CanonicalizationFailed),
                "native API producer bounded encoding rejected",
            )
        })
    }
}

fn incompatible() -> ServiceError {
    custody_error("native API association")
}
fn custody_error(stage: &str) -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::IdentityMismatch,
        format!("native API producer custody/identity mismatch: {stage}"),
    )
}
fn exhausted() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::BudgetExceeded,
        "native API producer cumulative bound exceeded",
    )
}
