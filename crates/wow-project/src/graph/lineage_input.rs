//! Native ordinary Project source-fingerprint partition for Candidate discovery.
//! Uses the existing exact-span primitive and original accepted source records.
use super::*;
use crate::graph::fingerprints::{
    FUNCTION_FINGERPRINT_PROFILE, FunctionFingerprintLimits, FunctionSourceSpanComponent,
    span_component,
};
use crate::replay::publication::{AcquiredProjectPair, RetainedGenerationPair};
use serde::Serialize;
use std::collections::BTreeSet;
use wow_emmy::function_calls::SourceFunctionKind;
use wow_graph::GraphNode;
use wow_graph::lineage::*;
use wow_graph::{GraphAssertionKind, GraphAssertionRef, GraphLocalAssertion, GraphPartitionRead};
mod source;
pub use source::build_source_fingerprint_partition;

pub fn build_project_source_fingerprint_partition(
    pair: &RetainedGenerationPair<'_>,
    limits: FunctionFingerprintLimits,
    graph_limits: LineageLimits,
    stop: &AtomicBool,
) -> ProjectResult<ValidatedLineageInput> {
    crate::analyzer::checkpoint(stop)?;
    if !pair.exact_configuration_and_registry() {
        return Err(deferred());
    }
    let before = pair.before();
    let after = pair.after();
    let mut retained = ProducerBudget::new(0, limits.max_metadata_bytes())?;
    // Lookup reconstructs the exact input-generation graph. Reserve its bounded
    // borrowed projection before that scratch copy, for both roles together.
    retained.charge_serialized(&(before.graph().snapshot(), after.graph().snapshot()), stop)?;
    for owner in [before, after] {
        retained.charge_serialized(&owner.project().snapshot().project_coverage_records(), stop)?;
        #[derive(Serialize)]
        struct Deferred<'a> {
            capability_id: &'a wow_core::CapabilityId,
            record: &'a wow_core::NotEvaluatedRecord,
        }
        for item in owner.project().snapshot().deferred_capabilities() {
            retained.charge_serialized(
                &Deferred {
                    capability_id: item.capability_id(),
                    record: item.record(),
                },
                stop,
            )?;
        }
    }
    let left = before.graph_read(stop).map_err(lineage_error)?;
    let right = after.graph_read(stop).map_err(lineage_error)?;
    let l = left.owner().producer_lookup(stop).map_err(lineage_error)?;
    let r = right.owner().producer_lookup(stop).map_err(lineage_error)?;
    let count = before
        .source_provenance()
        .ok_or_else(deferred)?
        .functions()
        .len()
        .checked_add(
            after
                .source_provenance()
                .ok_or_else(deferred)?
                .functions()
                .len(),
        )
        .ok_or_else(invalid)?;
    if count > limits.max_records() || count > graph_limits.max_entities {
        return Err(exhausted());
    }
    let binding = LineageComparisonBinding {
        universe_class: LineageUniverseClass::UserProject,
        profile_id: PROJECT_SPAN_LINEAGE_PROFILE.into(),
        pair_id: pair.pair_id().into(),
        before: binding(before, l.scope()),
        after: binding(after, r.scope()),
        source: None,
    };
    let mut builder = Builder {
        retained,
        source_bytes: 0,
        limits,
        stop,
        data: ProjectFingerprintPartition {
            schema: PROJECT_SPAN_INPUT_SCHEMA.into(),
            producer_id: PROJECT_SPAN_PRODUCER.into(),
            binding,
            records: Vec::new(),
            partitions: Vec::new(),
            coverage: "partial".into(),
            source_bodies: "excluded".into(),
            proof_ceiling: GraphConfidence::Candidate,
        },
    };
    builder
        .retained
        .charge_serialized(&builder.data.binding, stop)?;
    builder.role(before, LineageRole::Before, &left, &l)?;
    builder.role(after, LineageRole::After, &right, &r)?;
    wow_graph::lineage::validate_project_fingerprint_partition(
        builder.data,
        &left,
        &right,
        graph_limits,
        stop,
    )
    .map_err(lineage_error)
}
fn binding(
    owner: &AcquiredProjectPair,
    scope: &wow_graph::GraphAssertionRecordScope,
) -> LineageGenerationBinding {
    let (epoch, generation) = owner.read_identity();
    let project = owner.project();
    let config = project.configuration();
    LineageGenerationBinding {
        project_id: config.project_id().as_str().into(),
        project_kind: match config.project_kind() {
            crate::ProjectKind::Fixture => "fixture",
            crate::ProjectKind::Repository => "repository",
            crate::ProjectKind::BlizzardUiPlatformSource => "blizzard_ui_source",
        }
        .into(),
        selected_profile: config.selected_profile().clone(),
        project_generation_schema_version: project
            .snapshot()
            .generation_candidate()
            .project_generation_schema_version(),
        analyzer_pin_id: config.analyzer_binding().accepted_pin_id().into(),
        analyzer_configuration_digest: config
            .analyzer_binding()
            .analyzer_configuration_digest()
            .to_string()
            .into(),
        project_coverage: project.snapshot().project_coverage_records().to_vec(),
        project_deferred: project
            .snapshot()
            .deferred_capabilities()
            .iter()
            .map(|item| LineageDeferredCapability {
                capability_id: item.capability_id().clone(),
                record: item.record().clone(),
            })
            .collect(),
        store_epoch_id: epoch.as_str().into(),
        store_generation_id: generation.as_str().into(),
        publication_set_id: owner.publication_set_id().into(),
        project_snapshot_id: project.snapshot_id().into(),
        project_generation_id: project.project_generation().to_string().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        configuration_digest: config.configuration_digest().to_string().into(),
        reference_generation_id: config.reference_generation().to_string().into(),
        original_graph_scope: scope.clone(),
        published_graph_snapshot_id: owner.graph().snapshot().snapshot_id().clone(),
        published_graph_generation_id: owner.graph().snapshot().generation().clone(),
    }
}
struct Builder<'a> {
    retained: ProducerBudget,
    source_bytes: usize,
    limits: FunctionFingerprintLimits,
    stop: &'a AtomicBool,
    data: ProjectFingerprintPartition,
}
impl Builder<'_> {
    fn role(
        &mut self,
        owner: &AcquiredProjectPair,
        role: LineageRole,
        graph: &GraphPartitionRead<'_>,
        lookup: &wow_graph::GraphProducerLookup<'_>,
    ) -> ProjectResult<()> {
        let source = owner.source_provenance().ok_or_else(deferred)?;
        self.role_native(owner, source, None, role, graph, lookup)
    }
    fn role_native(
        &mut self,
        owner: &AcquiredProjectPair,
        source: &ProjectGraphProvenance,
        native: Option<&PlatformIndexSourceParts>,
        role: LineageRole,
        graph: &GraphPartitionRead<'_>,
        lookup: &wow_graph::GraphProducerLookup<'_>,
    ) -> ProjectResult<()> {
        let project = owner.project();
        let analyzer = project.snapshot().analyzer_binding();
        let report = source.function_call_report().ok_or_else(deferred)?;
        if source.context().context_id() != graph.owner().source_context_id()
            || report.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
            || report.functions().len() != source.functions().len()
            || report.budget()
                != project
                    .configuration()
                    .budget_policy()
                    .function_call_budget()
        {
            return Err(invalid());
        }
        let mut conflicts = BTreeSet::new();
        for part in graph.owner().partitions() {
            crate::analyzer::checkpoint(self.stop)?;
            let raw = part
                .batch()
                .assertion_records()
                .map_or(&[][..], |r| r.conflicts.as_slice());
            #[derive(Serialize)]
            struct Borrowed<'a> {
                role: LineageRole,
                partition_id: &'a str,
                partition_digest: &'a str,
                producer_version: &'a str,
                coverage: &'a [GraphCoverageRecord],
                conflicts: &'a [wow_graph::GraphConflictRecord],
            }
            self.retained.charge_serialized(
                &Borrowed {
                    role,
                    partition_id: part.partition_id(),
                    partition_digest: part.partition_digest(),
                    producer_version: part.producer_version(),
                    coverage: part.coverage(),
                    conflicts: raw,
                },
                self.stop,
            )?;
            for conflict in raw {
                crate::analyzer::checkpoint(self.stop)?;
                conflicts.insert(conflict.subject.clone());
                for item in &conflict.assertions {
                    crate::analyzer::checkpoint(self.stop)?;
                    conflicts.insert(item.clone());
                }
            }
            self.data.partitions.push(LineagePartitionCoverage {
                role,
                partition_id: part.partition_id().into(),
                partition_digest: part.partition_digest().into(),
                producer_version: part.producer_version().into(),
                coverage: part.coverage().to_vec(),
                conflicts: raw.to_vec(),
            });
        }
        for (fact, function) in report.functions().iter().zip(source.functions()) {
            crate::analyzer::checkpoint(self.stop)?;
            if function.function_id != fact.fact_id()
                || function.path != fact.path()
                || function.span != fact.span()
                || function.kind != fact.kind()
            {
                return Err(invalid());
            }
            let reference = |kind, proposal: &str| -> ProjectResult<GraphAssertionRef> {
                let assertion = GraphLocalAssertion {
                    kind,
                    proposal_id: proposal.into(),
                };
                if let Some(native) = native {
                    return native.assertion(&assertion).cloned().ok_or_else(invalid);
                }
                let part = graph
                    .owner()
                    .partition(SOURCE_GRAPH_PARTITION)
                    .ok_or_else(invalid)?;
                Ok(GraphAssertionRef::Producer {
                    partition_id: SOURCE_GRAPH_PARTITION.into(),
                    batch_id: part.batch().batch_id().into(),
                    assertion,
                })
            };
            let entity_ref = reference(GraphAssertionKind::Entity, &function.proposal_id)?;
            let owner_ref = reference(
                GraphAssertionKind::Relation,
                &function.ownership_proposal_id,
            )?;
            let entity = lookup
                .entity(lookup.scope(), &entity_ref, self.stop)
                .map_err(lineage_error)?;
            let ownership = lookup
                .relation(lookup.scope(), &owner_ref, self.stop)
                .map_err(lineage_error)?;
            let handle = source
                .source_handles()
                .get(&function.source_handle_id)
                .ok_or_else(invalid)?;
            let evidence = source
                .evidence()
                .get(&function.evidence_id)
                .ok_or_else(invalid)?;
            let digest =
                crate::identity::parse_source_digest(fact.content_digest(), ProjectPhase::View)?;
            if entity.proposal().entity_kind_id() != "lua_source_function"
                || entity.proposal().confidence() != GraphConfidence::Derived
                || ownership.proposal().relation_kind_id() != "source_declaration_owns"
                || ownership.accepted().edge().to() != entity.accepted().node().node_id()
                || lookup
                    .input_view()
                    .node(ownership.accepted().edge().from())
                    .is_none_or(|n| n.kind() != "source_file")
                || handle.path().as_str() != fact.path()
                || handle.span() != fact.span()
                || *handle.content_digest() != digest
                || evidence.context_id() != source.context().context_id()
                || !evidence
                    .source_handle_ids()
                    .contains(&function.source_handle_id)
            {
                return Err(invalid());
            }
            let file = analyzer
                .main_workspace()
                .file(fact.path())
                .ok_or_else(invalid)?;
            if file.content_sha256() != fact.content_digest() {
                return Err(invalid());
            }
            let text = if fact.kind() == SourceFunctionKind::Chunk
                && fact.span() == SourceSpan::whole_file()
            {
                Some(file.text())
            } else if let (Some(start), Some(end)) =
                (fact.span().byte_start(), fact.span().byte_end())
            {
                Some(
                    file.text()
                        .get(
                            usize::try_from(start).map_err(|_| invalid())?
                                ..usize::try_from(end).map_err(|_| invalid())?,
                        )
                        .ok_or_else(invalid)?,
                )
            } else {
                None
            };
            let feature = if conflicts.contains(&entity_ref) || conflicts.contains(&owner_ref) {
                LineageFingerprintFeature::ConflictBlocked
            } else {
                match text {
                    None => LineageFingerprintFeature::MissingExactSpan,
                    Some(text) => match span_component(
                        text.as_bytes(),
                        &mut self.source_bytes,
                        self.limits,
                        self.stop,
                    )? {
                        FunctionSourceSpanComponent::Available {
                            digest,
                            content_sha256,
                            byte_length,
                        } => LineageFingerprintFeature::Available {
                            source_span: LineageSourceSpanFeature {
                                profile_id: FUNCTION_FINGERPRINT_PROFILE.into(),
                                fingerprint:
                                    wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(
                                        *digest.as_bytes(),
                                    ),
                                content_sha256,
                                byte_length,
                            },
                        },
                        FunctionSourceSpanComponent::MissingExactSpan => {
                            LineageFingerprintFeature::MissingExactSpan
                        }
                    },
                }
            };
            let GraphAssertionRef::Producer {
                partition_id,
                assertion,
                ..
            } = &entity_ref
            else {
                return Err(invalid());
            };
            let node = graph
                .published_entity(partition_id, &assertion.proposal_id, self.stop)
                .map_err(lineage_error)?;
            #[derive(Serialize)]
            struct BorrowedRecord<'a> {
                role: LineageRole,
                node: &'a GraphNode,
                source_handle: &'a SourceHandle,
                evidence: &'a EvidenceRecord,
                function_id: &'a str,
                function_kind: LineageFunctionKind,
                source_assertion: &'a GraphAssertionRef,
                ownership_assertion: &'a GraphAssertionRef,
                fingerprint: &'a LineageFingerprintFeature,
            }
            self.retained.charge_serialized(
                &BorrowedRecord {
                    role,
                    node,
                    source_handle: handle,
                    evidence,
                    function_id: fact.fact_id(),
                    function_kind: match fact.kind() {
                        SourceFunctionKind::Chunk => LineageFunctionKind::Chunk,
                        SourceFunctionKind::Closure => LineageFunctionKind::Closure,
                    },
                    source_assertion: &entity_ref,
                    ownership_assertion: &owner_ref,
                    fingerprint: &feature,
                },
                self.stop,
            )?;
            self.data.records.push(LineageFunctionRecord {
                role,
                node: node.clone(),
                function_id: fact.fact_id().into(),
                function_kind: match fact.kind() {
                    SourceFunctionKind::Chunk => LineageFunctionKind::Chunk,
                    SourceFunctionKind::Closure => LineageFunctionKind::Closure,
                },
                source_assertion: entity_ref,
                ownership_assertion: owner_ref,
                source_handle: handle.clone(),
                evidence: evidence.clone(),
                fingerprint: feature,
            });
        }
        Ok(())
    }
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::View,
        "native lineage fingerprint input is inconsistent",
    )
}
fn deferred() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::DeferredCapability,
        ProjectPhase::View,
        "exact native lineage source input is unavailable or incompatible",
    )
}
fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        "lineage fingerprint population exceeds bound",
    )
}
fn lineage_error(error: wow_graph::GraphError) -> ProjectError {
    match error.code() {
        wow_graph::GraphErrorCode::Cancelled => ProjectError::new(
            ProjectErrorCode::AnalysisCancelled,
            ProjectPhase::View,
            "lineage input cancelled",
        ),
        wow_graph::GraphErrorCode::BudgetExceeded => exhausted(),
        _ => invalid(),
    }
}
