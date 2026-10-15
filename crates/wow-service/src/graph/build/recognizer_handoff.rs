//! Closed application composition for the held Project/Emmy state-core facts.
//!
//! The source owner already produced each fact bundle and matcher output. This
//! module reconstructs only that owner's four fixed read/write × root/path packs
//! to bind the retained output to its exact inputs. It returns every disposition
//! with the original output attached; the caller owns replacement and partial
//! publication policy.

use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use wow_core::canonical_json_bytes;
use wow_graph::{
    GraphEntityProposal, GraphNodeId, GraphPartitionReplacement, GraphProposalBatch,
    GraphProposalValidationReport, GraphRelationProposal,
};
use wow_recognizers::{
    RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerClause, RecognizerError, RecognizerErrorCode,
    RecognizerFactLimits, RecognizerFactValue, RecognizerGraphEndpointBinding,
    RecognizerGraphEndpointSide, RecognizerGraphHandoffDisposition, RecognizerOutput,
    RecognizerOutputConfidence, RecognizerOutputPartition, RecognizerPack, RecognizerPackBudgets,
    RecognizerPackDocument, RecognizerPackLiteral, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerProposedAssertion, RecognizerRule, adapt_recognizer_output_to_graph,
    compile_recognizer_plan, parse_recognizer_pack,
    source_state_core::{SourceStateCoreFamily, SourceStateCoreRecognition},
};

const STATE_CORE_FACT_PROFILE: &str = "wow-recognizers-state-facts-1";
const STATE_CORE_EVALUATION_PROFILE: &str = "wow-recognizers-state-structural-1";
const MAX_STATE_CORE_FACTS: usize = 8192;
const MAX_STATE_CORE_OUTPUT_PROPOSALS: usize = 16_384;
const MAX_STATE_CORE_OUTPUT_COPY_BYTES: usize = 32 * 1024 * 1024;
const STATE_PATH_OUTPUT_ID: &str = "path";
const STATE_PATH_ENTITY_KIND: &str = "state_path";
const STATE_PATH_ENDPOINT_REFERENCE: &str = "entity:path";

pub(super) struct HeldStateCoreHandoff<'a> {
    pub output: &'a RecognizerOutputPartition,
    pub disposition: RecognizerGraphHandoffDisposition,
    pub ready: Option<ReadyHandoffPayload>,
    pub failure: Option<RecognizerErrorCode>,
}

pub(super) struct ReadyHandoffPayload {
    pub producer_partition_id: Box<str>,
    pub batch: GraphProposalBatch,
    pub validation_report: GraphProposalValidationReport,
}

/// Compose generic handoffs from the exact Project/Emmy-produced evaluations.
/// Every returned disposition retains the source output. The caller owns the
/// choice to consume Ready parts or preserve the existing partial policy.
pub(super) fn compose_held_state_core_handoffs<'a>(
    graph: &GraphPartitionSnapshot,
    context: &wow_core::GenerationContext,
    family: SourceStateCoreFamily,
    recognition: &'a SourceStateCoreRecognition,
    stop: &AtomicBool,
) -> ServiceResult<Vec<HeldStateCoreHandoff<'a>>> {
    checkpoint(stop)?;
    if recognition.family != family || recognition.evaluations.len() != 2 {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let fact_count = recognition
        .evaluations
        .iter()
        .try_fold(0_usize, |count, evaluation| {
            count.checked_add(evaluation.fact_bundle.facts().len())
        })
        .ok_or_else(|| error(ServiceErrorCode::BudgetExceeded))?;
    if fact_count > MAX_STATE_CORE_FACTS {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }

    let mut recipes = BTreeSet::new();
    let mut handoffs = Vec::with_capacity(recognition.evaluations.len());
    for evaluation in &recognition.evaluations {
        checkpoint(stop)?;
        let has_path = match evaluation.recipe {
            "root_slot" => false,
            "literal_path" => true,
            _ => return Err(error(ServiceErrorCode::IdentityMismatch)),
        };
        if !recipes.insert(evaluation.recipe) {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }

        let pack = state_core_pack(family, has_path, graph.registry().bundle_id())?;
        if pack.pack_digest() != evaluation.pack_digest {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        let plan = compile_recognizer_plan(&pack).map_err(recognizer_error)?;
        let fact_limits =
            RecognizerFactLimits::new(MAX_STATE_CORE_FACTS as u32, 8, 8, 16, 32, 32, 8)
                .map_err(recognizer_error)?;
        let bindings = endpoint_bindings(
            graph,
            &evaluation.fact_bundle,
            &evaluation.output,
            family,
            has_path,
            stop,
        )?;
        let handoff =
            adapt_recognizer_output_to_graph(wow_recognizers::RecognizerGraphHandoffRequest {
                context,
                pack: &pack,
                plan: &plan,
                facts: &evaluation.fact_bundle,
                output: &evaluation.output,
                fact_limits,
                graph,
                graph_limits: graph.snapshot().limits(),
                endpoint_bindings: &bindings,
                cancelled: stop,
            })
            .map_err(recognizer_error)?;
        let disposition = handoff.disposition();
        match handoff.into_ready_parts() {
            Ok(parts) => {
                let wow_recognizers::RecognizerGraphHandoffParts {
                    source_output: _,
                    producer_partition_id,
                    producer_version: _,
                    batch,
                    validation_report,
                    coverage_records: _,
                } = parts;
                if batch.producer_partition_id() != producer_partition_id.as_ref() {
                    return Err(error(ServiceErrorCode::IdentityMismatch));
                }
                handoffs.push(HeldStateCoreHandoff {
                    output: &evaluation.output,
                    disposition,
                    ready: Some(ReadyHandoffPayload {
                        producer_partition_id,
                        batch,
                        validation_report,
                    }),
                    failure: None,
                });
            }
            Err(handoff) => handoffs.push(HeldStateCoreHandoff {
                output: &evaluation.output,
                disposition,
                ready: None,
                failure: handoff.failure().map(RecognizerError::code),
            }),
        }
        checkpoint(stop)?;
    }

    if !recipes.contains("root_slot") || !recipes.contains("literal_path") {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    Ok(handoffs)
}

/// Run the generic adapter on the canonical Service path. A complete result
/// replaces the existing W7 family partition with the adapter's exact outputs;
/// partial, unevaluated, and mixed results retain the original W7 publication.
pub(super) fn apply_ready_handoffs(
    mut graph: GraphPartitionSnapshot,
    context: &wow_core::GenerationContext,
    recognitions: &[SourceStateCoreRecognition],
    stop: &AtomicBool,
) -> ServiceResult<GraphPartitionSnapshot> {
    for recognition in recognitions {
        checkpoint(stop)?;
        let handoffs = compose_held_state_core_handoffs(
            &graph,
            context,
            recognition.family,
            recognition,
            stop,
        )?;
        let mut ready_count = 0_usize;
        for handoff in &handoffs {
            checkpoint(stop)?;
            match handoff.disposition {
                RecognizerGraphHandoffDisposition::Ready
                    if handoff.ready.is_some() && handoff.failure.is_none() =>
                {
                    ready_count += 1;
                }
                RecognizerGraphHandoffDisposition::Deferred
                    if handoff.ready.is_none() && handoff.failure.is_none() => {}
                RecognizerGraphHandoffDisposition::Cancelled => {
                    return Err(error(ServiceErrorCode::Cancelled));
                }
                RecognizerGraphHandoffDisposition::Rejected => {
                    let code = handoff
                        .failure
                        .unwrap_or(RecognizerErrorCode::GraphProjectionFailed);
                    return Err(recognizer_code_error(code));
                }
                RecognizerGraphHandoffDisposition::GraphRejected => {
                    return Err(error(ServiceErrorCode::InternalContractViolation));
                }
                _ => return Err(error(ServiceErrorCode::InternalContractViolation)),
            }
        }

        if ready_count == 0 || ready_count != handoffs.len() {
            continue;
        }
        graph = replace_ready_family(&graph, recognition, &handoffs, stop)?;
    }
    Ok(graph)
}

fn replace_ready_family(
    graph: &GraphPartitionSnapshot,
    recognition: &SourceStateCoreRecognition,
    handoffs: &[HeldStateCoreHandoff<'_>],
    stop: &AtomicBool,
) -> ServiceResult<GraphPartitionSnapshot> {
    if handoffs.len() != recognition.evaluations.len() || handoffs.len() != 2 {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let current = graph
        .partition(recognition.family.partition_id())
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;

    let mut entity_count = 0_usize;
    let mut relation_count = 0_usize;
    let mut copy_budget = ProposalCopyBudget::default();
    let mut first_batch: Option<&GraphProposalBatch> = None;
    for (evaluation, handoff) in recognition.evaluations.iter().zip(handoffs) {
        checkpoint(stop)?;
        if !std::ptr::eq(handoff.output, &evaluation.output) {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        let payload = handoff
            .ready
            .as_ref()
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        if !payload.validation_report.ready_for_publication()
            || payload.batch.producer_partition_id() != payload.producer_partition_id.as_ref()
            || payload.batch.registry_bundle_id() != graph.registry().bundle_id()
            || payload.batch.universe() != graph.snapshot().universe()
            || payload.batch.generation() != graph.snapshot().generation()
            || payload.batch.source_context_id() != graph.source_context_id()
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        if let Some(first) = first_batch {
            if payload.batch.registry_bundle_id() != first.registry_bundle_id()
                || payload.batch.registry_digest() != first.registry_digest()
                || payload.batch.universe() != first.universe()
                || payload.batch.generation() != first.generation()
                || payload.batch.source_context_id() != first.source_context_id()
            {
                return Err(error(ServiceErrorCode::IdentityMismatch));
            }
        } else {
            first_batch = Some(&payload.batch);
        }

        for outcome in evaluation.output.outcomes() {
            checkpoint(stop)?;
            for proposal in outcome.proposals() {
                checkpoint(stop)?;
                match proposal {
                    RecognizerProposedAssertion::Entity { proposal_id, .. } => {
                        entity_count = entity_count
                            .checked_add(1)
                            .ok_or_else(|| error(ServiceErrorCode::BudgetExceeded))?;
                        if entity_count.saturating_add(relation_count)
                            > MAX_STATE_CORE_OUTPUT_PROPOSALS
                        {
                            return Err(error(ServiceErrorCode::BudgetExceeded));
                        }
                        let graph_proposal = payload
                            .batch
                            .entity_proposal(proposal_id.as_str())
                            .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
                        copy_budget.account(graph_proposal, stop)?;
                    }
                    RecognizerProposedAssertion::Relation { proposal_id, .. } => {
                        relation_count = relation_count
                            .checked_add(1)
                            .ok_or_else(|| error(ServiceErrorCode::BudgetExceeded))?;
                        if entity_count.saturating_add(relation_count)
                            > MAX_STATE_CORE_OUTPUT_PROPOSALS
                        {
                            return Err(error(ServiceErrorCode::BudgetExceeded));
                        }
                        let graph_proposal = payload
                            .batch
                            .relation_proposal(proposal_id.as_str())
                            .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
                        copy_budget.account(graph_proposal, stop)?;
                    }
                }
            }
        }
    }
    if entity_count
        .checked_add(relation_count)
        .is_none_or(|count| count > MAX_STATE_CORE_OUTPUT_PROPOSALS)
    {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }
    let first_batch = first_batch.ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    let mut entities = Vec::<GraphEntityProposal>::with_capacity(entity_count);
    let mut relations = Vec::<GraphRelationProposal>::with_capacity(relation_count);
    for (evaluation, handoff) in recognition.evaluations.iter().zip(handoffs) {
        checkpoint(stop)?;
        let payload = handoff
            .ready
            .as_ref()
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        for outcome in evaluation.output.outcomes() {
            checkpoint(stop)?;
            for proposal in outcome.proposals() {
                checkpoint(stop)?;
                match proposal {
                    RecognizerProposedAssertion::Entity { proposal_id, .. } => {
                        entities.push(
                            payload
                                .batch
                                .entity_proposal(proposal_id.as_str())
                                .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?
                                .clone(),
                        );
                    }
                    RecognizerProposedAssertion::Relation { proposal_id, .. } => {
                        relations.push(
                            payload
                                .batch
                                .relation_proposal(proposal_id.as_str())
                                .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?
                                .clone(),
                        );
                    }
                }
            }
        }
    }
    checkpoint(stop)?;
    let batch = GraphProposalBatch::build(
        first_batch.registry_bundle_id().to_owned(),
        first_batch.registry_digest().to_owned(),
        first_batch.universe().clone(),
        first_batch.generation().clone(),
        first_batch.source_context_id(),
        recognition.family.partition_id(),
        entities,
        relations,
    )
    .map_err(graph_error)?;
    let plan = graph
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: graph.snapshot().snapshot_id().clone(),
                expected_partition_digest: Some(current.partition_digest().into()),
                producer_version: current.producer_version().into(),
                batch,
                coverage: current.coverage().to_vec(),
            },
            stop,
        )
        .map_err(graph_error)?;
    checkpoint(stop)?;
    Ok(plan.into_candidate())
}

#[derive(Default)]
struct ProposalCopyBudget {
    bytes: usize,
    exceeded: bool,
}

impl ProposalCopyBudget {
    fn account<T: serde::Serialize>(&mut self, value: &T, stop: &AtomicBool) -> ServiceResult<()> {
        checkpoint(stop)?;
        if serde_json::to_writer(&mut *self, value).is_err() {
            return Err(error(if self.exceeded {
                ServiceErrorCode::BudgetExceeded
            } else {
                ServiceErrorCode::InternalContractViolation
            }));
        }
        self.write_all(b"\n").map_err(|_| {
            error(if self.exceeded {
                ServiceErrorCode::BudgetExceeded
            } else {
                ServiceErrorCode::InternalContractViolation
            })
        })
    }
}

impl Write for ProposalCopyBudget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let Some(next) = self.bytes.checked_add(bytes.len()) else {
            self.exceeded = true;
            return Err(std::io::Error::other("state-core proposal copy ceiling"));
        };
        if next > MAX_STATE_CORE_OUTPUT_COPY_BYTES {
            self.exceeded = true;
            return Err(std::io::Error::other("state-core proposal copy ceiling"));
        }
        self.bytes = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn endpoint_bindings(
    graph: &GraphPartitionSnapshot,
    facts: &wow_recognizers::RecognizerFactBundle,
    output: &RecognizerOutputPartition,
    family: SourceStateCoreFamily,
    has_path: bool,
    stop: &AtomicBool,
) -> ServiceResult<Vec<RecognizerGraphEndpointBinding>> {
    let input = graph.input_view(stop).map_err(graph_error)?;
    let relation_kind_id = match family {
        SourceStateCoreFamily::Read => "source_reads_state",
        SourceStateCoreFamily::Write => "source_writes_state",
    };
    let mut seen = BTreeSet::<(RecognizerGraphEndpointSide, String)>::new();
    let mut bindings = Vec::with_capacity(facts.facts().len().saturating_mul(2));
    for fact in facts.facts() {
        checkpoint(stop)?;
        for (field, side) in [
            ("source", RecognizerGraphEndpointSide::Source),
            ("target", RecognizerGraphEndpointSide::Target),
        ] {
            let Some(RecognizerFactValue::Reference(reference)) = fact.fields().get(field) else {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            };
            if side == RecognizerGraphEndpointSide::Target
                && reference.as_ref() == STATE_PATH_ENDPOINT_REFERENCE
            {
                continue;
            }
            if !seen.insert((side, reference.to_string())) {
                continue;
            }
            let node_id = GraphNodeId::new(reference.as_ref())
                .map_err(|_| error(ServiceErrorCode::IdentityMismatch))?;
            if input.node(&node_id).is_none() {
                return Err(error(ServiceErrorCode::IdentityMismatch));
            }
            let binding = RecognizerGraphEndpointBinding::existing(
                relation_kind_id,
                side,
                RecognizerFactValue::Reference(reference.clone()),
                node_id,
            )
            .map_err(recognizer_error)?;
            bindings.push(binding);
        }
    }
    if has_path {
        for outcome in output.outcomes() {
            checkpoint(stop)?;
            for proposal in outcome.proposals() {
                checkpoint(stop)?;
                let RecognizerProposedAssertion::Relation {
                    relation_kind_id,
                    target,
                    match_id,
                    ..
                } = proposal
                else {
                    continue;
                };
                if !matches!(
                    target,
                    RecognizerFactValue::Reference(value)
                        if value.as_ref() == STATE_PATH_ENDPOINT_REFERENCE
                ) {
                    continue;
                }
                let mut path_entities =
                    outcome
                        .proposals()
                        .iter()
                        .filter_map(|candidate| match candidate {
                            RecognizerProposedAssertion::Entity {
                                proposal_id,
                                output_id,
                                entity_kind_id,
                                match_id: entity_match_id,
                                ..
                            } if output_id.as_ref() == STATE_PATH_OUTPUT_ID
                                && entity_kind_id.as_ref() == STATE_PATH_ENTITY_KIND
                                && entity_match_id == match_id =>
                            {
                                Some(proposal_id)
                            }
                            _ => None,
                        });
                let Some(path_proposal_id) = path_entities.next() else {
                    return Err(error(ServiceErrorCode::IdentityMismatch));
                };
                if path_entities.next().is_some() {
                    return Err(error(ServiceErrorCode::IdentityMismatch));
                }
                bindings.push(
                    RecognizerGraphEndpointBinding::proposed(
                        relation_kind_id.clone(),
                        RecognizerGraphEndpointSide::Target,
                        target.clone(),
                        match_id.clone(),
                        path_proposal_id.clone(),
                    )
                    .map_err(recognizer_error)?,
                );
            }
        }
    }
    Ok(bindings)
}

fn state_core_pack(
    family: SourceStateCoreFamily,
    has_path: bool,
    registry_bundle_id: &str,
) -> ServiceResult<wow_recognizers::CompiledRecognizerPack> {
    let family_name = match family {
        SourceStateCoreFamily::Read => "read",
        SourceStateCoreFamily::Write => "write",
    };
    let relation_id = match family {
        SourceStateCoreFamily::Read => "source_reads_state",
        SourceStateCoreFamily::Write => "source_writes_state",
    };
    let positive_fixture_ids = if family == SourceStateCoreFamily::Read && has_path {
        vec!["RECOG-STATE-002".into()]
    } else {
        Vec::new()
    };
    let mut outputs = Vec::new();
    if has_path {
        outputs.push(RecognizerOutput::EntityAssertion {
            output_id: STATE_PATH_OUTPUT_ID.into(),
            entity_kind_id: STATE_PATH_ENTITY_KIND.into(),
            semantic_key: BTreeMap::from([
                ("root".into(), "f.root".into()),
                ("path".into(), "f.path".into()),
            ]),
            confidence: RecognizerOutputConfidence::Derived,
        });
    }
    outputs.push(RecognizerOutput::RelationAssertion {
        output_id: relation_id.into(),
        relation_kind_id: relation_id.into(),
        source: "f.source".into(),
        target: "f.target".into(),
        confidence: RecognizerOutputConfidence::Derived,
    });

    let document = RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: format!(
                "wow-core-state-{family_name}-{}",
                if has_path { "path" } else { "root" }
            )
            .into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: STATE_CORE_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: STATE_CORE_EVALUATION_PROFILE.into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 1,
                max_clauses_per_rule: 4,
                max_clause_depth: 2,
                max_join_expansions_per_rule: 65_536,
                max_matches_per_rule_partition: MAX_STATE_CORE_FACTS as u32,
                max_proposals_per_rule_partition: 16_384,
                max_explanation_bytes: 8 * 1024 * 1024,
            },
            rules: vec![RecognizerRule {
                rule_id: family.rule_id().into(),
                version: 1,
                required_capabilities: vec![family.capability_id().into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "f".into(),
                        kind: "state_access".into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "f.has_path".into(),
                        value: RecognizerPackLiteral::Boolean(has_path),
                    },
                ],
                captures: Vec::new(),
                outputs,
                positive_fixture_ids,
                near_negative_fixture_ids: Vec::new(),
                partial_fixture_ids: Vec::new(),
                mutation_fixture_ids: Vec::new(),
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
    parse_recognizer_pack(&bytes).map_err(recognizer_error)
}

fn recognizer_error(failure: RecognizerError) -> ServiceError {
    recognizer_code_error(failure.code())
}

fn recognizer_code_error(code: RecognizerErrorCode) -> ServiceError {
    let service_code = match code {
        RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    };
    ServiceError::new(
        service_code,
        format!("state-core generic graph handoff rejected ({:?})", code),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LocalProjectInput;
    use crate::graph::{GraphBuildRequest, execute_graph_build};
    use std::error::Error;
    use std::sync::atomic::AtomicBool;
    use wow_core::{
        CanonicalResult, ContentDigest, ProfileIdentity, ProfileIdentityBuilder, ProfileKind,
        ReferenceGenerationId, SchemaVersionEntry, SourceKind, SourceLogicalSnapshot, ToolVersion,
    };
    use wow_emmy::{
        EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
        LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    };
    use wow_graph::GraphPartitionSnapshot;
    use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
    use wow_project::load::TocLoadContext;
    use wow_project::{
        AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
        ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectKind,
        ProjectSourceOriginId, ProjectWorkspaceId,
    };
    use wow_reference::ReferenceView;
    fn state_core_profile() -> Result<ProfileIdentity, Box<dyn Error>> {
        Ok(ProfileIdentityBuilder::new(
            "profile:fixture:state-core-pipeline-v1".parse()?,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
            ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
        )
        .schema_versions(vec![SchemaVersionEntry::new(
            "schema:wow:fixture-e0".parse()?,
            ToolVersion::parse("1.0.0")?,
        )])
        .fixture_scope("wow-state-core-pipeline-fixture")
        .build()?)
    }

    fn backend() -> Result<EmmyBackendIdentity, Box<dyn Error>> {
        Ok(EmmyBackendIdentity::new(
            "emmylua_code_analysis",
            Some(EMMYLUA_CODE_ANALYSIS_VERSION),
            EMMYLUA_REVISION,
            EMMYLUA_TREE,
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
        )?)
    }

    #[test]
    fn generic_handoff_keeps_real_project_emmy_facts_partial() -> Result<(), Box<dyn Error>> {
        let stop = AtomicBool::new(false);
        let directory = ProjectInputDirectory::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data/state-core-pipeline"),
        )?;
        let (files, load_plan) = directory
            .read_toc_project_with_context(
                ".",
                &ProjectDiskFile::new("StateCore.toc"),
                &state_core_profile()?,
                Some(&TocLoadContext {
                    game_types: BTreeMap::from([("retail".into(), true)]),
                    family: None,
                    game: None,
                    text_locale: None,
                    location: None,
                    environment: None,
                }),
                &stop,
            )?
            .into_parts();
        let identity = backend()?;
        let binding = AnalyzerBindingDeclaration::new(
            "wow-emmy/e0-c/1",
            format!("emmy-pin:{EMMYLUA_REVISION}"),
            identity.compatibility_report_sha256().to_owned(),
            ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
            "wow-emmy/e0-c/1",
            "wow-emmy-e0-c-library-v1",
            identity.clone(),
        )?;
        let library = LuaWorkspaceSnapshot::build(
            identity,
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                "library/state_core.lua",
                "---@meta _\n",
            )],
            LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
        )?;
        let reference_generation = ReferenceGenerationId::derive(&"fixture-state-core-reference")?;
        let config = ProjectConfigurationBuilder::new(
            ProjectId::new("fixture-state-core-pipeline")?,
            ProjectKind::Fixture,
            state_core_profile()?,
            reference_generation,
            binding,
        )
        .workspace_id(ProjectWorkspaceId::new("workspace:state-core-pipeline")?)
        .source_origin_id(ProjectSourceOriginId::new(
            "project-origin:state-core-pipeline",
        )?)
        .logical_root("fixtures/state-core-pipeline")
        .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
        .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
        .load_plan(&load_plan)?
        .build()?;
        let bundle = ProjectInputBundle::closed(config, files, vec![library])?;
        let reference =
            ReferenceView::new(reference_generation.to_string(), Vec::new(), Vec::new())?;
        let input = LocalProjectInput::new_with_load_plan(bundle, reference, Some(load_plan))?;
        let request =
            GraphBuildRequest::new("fixture-state-core-pipeline".into(), "current".into())?;
        let result = execute_graph_build(input, &request, &stop)?;
        let wire: serde_json::Value = serde_json::from_slice(&result.canonical_bytes()?)?;
        let recognitions = wire["state_core_recognition"]
            .as_array()
            .ok_or("Service result omitted state-core recognition")?;
        let mut saw_fact = false;
        for recognition in recognitions {
            let evaluations = recognition["evaluations"]
                .as_array()
                .ok_or("state-core result omitted evaluations")?;
            for evaluation in evaluations {
                let facts = evaluation["fact_bundle"]["facts"]
                    .as_array()
                    .ok_or("state-core result omitted its typed facts")?;
                let outcomes = evaluation["output"]["outcomes"]
                    .as_array()
                    .ok_or("state-core result omitted matcher outcomes")?;
                if facts.is_empty() {
                    assert!(
                        outcomes
                            .iter()
                            .all(|outcome| outcome["state"].as_str() == Some("not_evaluated"))
                    );
                } else {
                    saw_fact = true;
                    assert!(facts.iter().all(|fact| {
                        fact["producer_id"].as_str() == Some("wow.recognizers.state-admission")
                            && fact["fields"].is_object()
                    }));
                    assert!(
                        outcomes
                            .iter()
                            .all(|outcome| outcome["state"].as_str() == Some("partial"))
                    );
                }
            }
        }
        assert!(
            saw_fact,
            "held Project/Emmy source must produce a typed fact"
        );

        let snapshot_bytes = result
            .snapshot_bytes()?
            .ok_or("Service did not return the final graph snapshot")?;
        let snapshot: GraphPartitionSnapshot = serde_json::from_slice(&snapshot_bytes)?;
        snapshot.validate(&stop)?;
        assert!(snapshot.partition("wow-recognizers.state-read").is_some());
        assert!(snapshot.partition("wow-recognizers.state-write").is_some());
        assert!(!snapshot.snapshot().edges().is_empty());
        Ok(())
    }
}
