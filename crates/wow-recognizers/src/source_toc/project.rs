//! Matcher assertion projection. Same-batch references bind by exact match and
//! output identity; existing endpoints must exist in the graph input view.
use super::adapt::{Seed, Seeds};
use super::*;
use crate::{
    RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage, RecognizerFactCoverageInput,
    RecognizerFactCoverageState, RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope,
    RecognizerFactScopeKind, RecognizerFactValue, RecognizerOutputConfidence,
    RecognizerProposedAssertion, compile_recognizer_plan, execute_recognizer_plan,
};
use std::collections::BTreeSet;
use wow_graph::{
    GraphConfidence, GraphCoverageState, GraphEntityProposal, GraphNodeId, GraphProposalEndpoint,
    GraphProposalValue, GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
    GraphSnapshot,
};

pub(super) fn execute(
    input: &SourceTocInput<'_>,
    family: SourceTocFamily,
    seeds: Seeds,
    stop: &AtomicBool,
) -> RecognizerResult<SourceTocProposals> {
    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let limits = RecognizerFactLimits::new(MAX_FACTS as u32, 8, 8, 32, 64, 64, 16)?;
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut receipts = BTreeMap::<String, SourceTocReceipt>::new();
    let mut evaluations = Vec::new();
    for recipe in Recipe::for_family(family) {
        checkpoint(stop)?;
        let mut origins = BTreeMap::new();
        let facts = seeds
            .values
            .iter()
            .filter(|seed| seed.recipe == recipe)
            .map(|seed| {
                validate_support(input, seed)?;
                let fact = fact(input, seed, limits)?;
                origins.insert(fact.fact_id().clone(), seed.origins.clone());
                Ok(fact)
            })
            .collect::<RecognizerResult<Vec<_>>>()?;
        // Coverage describes captured structure only. Omissions and empty
        // inputs cannot certify complete source scope or an absent declaration.
        let state = if input.facts.is_empty() {
            RecognizerFactCoverageState::NotEvaluated
        } else {
            RecognizerFactCoverageState::Partial
        };
        let coverage = RecognizerFactCoverage::new(
            RecognizerFactCoverageInput {
                context_id: input.context.context_id(),
                partition_id: recipe.name().into(),
                capability_id: family.capability_id().into(),
                producer_id: "wow.project".into(),
                producer_version: FACT_PROFILE.into(),
                state,
                blocker_ids: vec!["toc.captured_structure_no_complete_or_runtime_authority".into()],
            },
            limits,
        )?;
        let bundle = RecognizerFactBundle::build(
            input.context,
            recipe.name(),
            Vec::new(),
            facts,
            vec![coverage],
            limits,
        )?;
        let pack = pack::compile(recipe, input.owner.registry().bundle_id())?;
        let plan = compile_recognizer_plan(&pack)?;
        let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, limits, stop)?;
        for outcome in output.outcomes() {
            checkpoint(stop)?;
            if outcome.rule_id() != family.rule_id() || outcome.rule_version() != 1 {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            for matched in outcome.matches() {
                let mut fact_ids = BTreeSet::new();
                for decisive in matched.decisive_fact_ids() {
                    fact_ids.extend(
                        origins
                            .get(decisive)
                            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
                            .iter()
                            .cloned(),
                    );
                }
                let match_id = matched.match_id().to_string();
                if receipts
                    .insert(
                        match_id.clone(),
                        SourceTocReceipt {
                            match_id,
                            fact_ids: fact_ids.into_iter().collect(),
                            entity_proposal_ids: Vec::new(),
                            relation_proposal_ids: Vec::new(),
                        },
                    )
                    .is_some()
                {
                    return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                }
            }
            // Proposal IDs are canonical order, not dependency order. Bind all
            // entities before resolving any same-match relation endpoint.
            let mut proposed = BTreeMap::<(String, String), String>::new();
            let mut grouped = BTreeMap::<
                (String, Vec<u8>),
                (GraphEntityProposal, BTreeSet<wow_core::CoverageId>),
            >::new();
            for proposal in outcome.proposals() {
                checkpoint(stop)?;
                if let RecognizerProposedAssertion::Entity {
                    proposal_id,
                    output_id,
                    entity_kind_id,
                    semantic_key,
                    confidence,
                    match_id,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } = proposal
                {
                    let key = semantic_key
                        .iter()
                        .map(|(name, value)| {
                            let RecognizerFactValue::String(value) = value else {
                                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                            };
                            Ok((name.clone(), GraphProposalValue::String(value.clone())))
                        })
                        .collect::<RecognizerResult<BTreeMap<_, _>>>()?;
                    let group_key = (
                        entity_kind_id.to_string(),
                        wow_core::canonical_json_bytes(&key)
                            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                    );
                    let mut entity = GraphEntityProposal::new(
                        proposal_id.as_str(),
                        entity_kind_id.clone(),
                        key,
                        confidence_of(*confidence),
                        source_handle_ids.clone(),
                        evidence_ids.clone(),
                        coverage_ids.clone(),
                    )
                    .map_err(graph_error)?;
                    let mut group_coverage = coverage_ids.iter().cloned().collect::<BTreeSet<_>>();
                    if let Some((previous, previous_coverage)) = grouped.get(&group_key) {
                        group_coverage.extend(previous_coverage.iter().cloned());
                        entity = merge_entity(previous, &entity, &group_coverage)?;
                    }
                    let id = entity.proposal_id().to_owned();
                    grouped.insert(group_key, (entity, group_coverage));
                    let matched = match_id.to_string();
                    if proposed
                        .insert((matched.clone(), output_id.to_string()), id.clone())
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    receipts
                        .get_mut(&matched)
                        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
                        .entity_proposal_ids
                        .push(id);
                }
            }
            entities.extend(grouped.into_values().map(|(entity, _)| entity));
            for proposal in outcome.proposals() {
                checkpoint(stop)?;
                if let RecognizerProposedAssertion::Relation {
                    proposal_id,
                    relation_kind_id,
                    source,
                    target,
                    confidence,
                    match_id,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } = proposal
                {
                    let matched = match_id.to_string();
                    let id = proposal_id.to_string();
                    relations.push(
                        GraphRelationProposal::new(
                            id.as_str(),
                            relation_kind_id.clone(),
                            GraphRelationProposalInput {
                                source: endpoint(source, &matched, &proposed, &graph)?,
                                target: endpoint(target, &matched, &proposed, &graph)?,
                                confidence: confidence_of(*confidence),
                                source_handle_ids: source_handle_ids.clone(),
                                evidence_ids: evidence_ids.clone(),
                                coverage_ids: coverage_ids.clone(),
                            },
                        )
                        .map_err(graph_error)?,
                    );
                    receipts
                        .get_mut(&matched)
                        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
                        .relation_proposal_ids
                        .push(id);
                }
            }
        }
        evaluations.push(SourceTocEvaluation {
            recipe: recipe.name(),
            pack_digest: pack.pack_digest().into(),
            fact_bundle: bundle,
            output,
        });
    }
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        family.partition_id(),
        entities,
        relations,
    )
    .map_err(graph_error)?;
    let covered = match family {
        SourceTocFamily::Package => vec![GraphRelationKind::Contains, GraphRelationKind::Defines],
        SourceTocFamily::FileOrder => {
            vec![GraphRelationKind::Loads, GraphRelationKind::LoadsBefore]
        }
        SourceTocFamily::Dependencies => vec![
            GraphRelationKind::DependsOn,
            GraphRelationKind::OptionalDependsOn,
        ],
        SourceTocFamily::LoadOnDemand => vec![GraphRelationKind::Defines],
        SourceTocFamily::SavedVariables | SourceTocFamily::SavedVariableRoot => {
            vec![GraphRelationKind::Owns]
        }
    };
    let coverage = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|r| r.relation())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|relation| {
            GraphCoverageRecord::new(
                relation,
                if covered.contains(&relation) {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if covered.contains(&relation) {
                    "toc.static_structure_no_negative_or_runtime_authority".into()
                } else {
                    "toc.relation_owned_by_other_producer".into()
                }],
                graph.limits(),
            )
            .map_err(graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    checkpoint(stop)?;
    for receipt in receipts.values_mut() {
        receipt.entity_proposal_ids.sort();
        receipt.entity_proposal_ids.dedup();
        receipt.relation_proposal_ids.sort();
        receipt.relation_proposal_ids.dedup();
    }
    Ok(SourceTocProposals {
        batch,
        coverage,
        recognition: SourceTocRecognition {
            profile: SOURCE_TOC_PROFILE,
            family,
            evaluations,
            receipts: receipts.into_values().collect(),
            omissions: seeds.omissions,
        },
    })
}

/// Equivalent matcher assertions share one graph entity. The original matcher
/// proposals remain in evaluations, while graph support retains every witness.
fn merge_entity(
    left: &GraphEntityProposal,
    right: &GraphEntityProposal,
    coverage: &BTreeSet<wow_core::CoverageId>,
) -> RecognizerResult<GraphEntityProposal> {
    let handles = left
        .source_handle_ids()
        .iter()
        .chain(right.source_handle_ids())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let evidence = left
        .evidence_ids()
        .iter()
        .chain(right.evidence_ids())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let confidence = if left.confidence() == GraphConfidence::Possible
        || right.confidence() == GraphConfidence::Possible
    {
        GraphConfidence::Possible
    } else {
        GraphConfidence::Derived
    };
    GraphEntityProposal::new(
        left.proposal_id(),
        left.entity_kind_id(),
        left.semantic_key().clone(),
        confidence,
        handles,
        evidence,
        coverage.iter().cloned().collect(),
    )
    .map_err(graph_error)
}

fn fact(
    input: &SourceTocInput<'_>,
    seed: &Seed,
    limits: RecognizerFactLimits,
) -> RecognizerResult<RecognizerFact> {
    RecognizerFact::new(
        input.context.context_id(),
        RecognizerFactInput {
            kind: seed.recipe.name().into(),
            partition_id: seed.recipe.name().into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Package,
                seed.document.clone(),
            )?,
            producer_id: "wow.project".into(),
            producer_version: FACT_PROFILE.into(),
            confidence: seed.confidence,
            fields: seed.fields.clone(),
            source_handle_ids: seed.handles.iter().copied().collect(),
            evidence_ids: seed.evidence.iter().copied().collect(),
        },
        limits,
    )
}
fn validate_support(input: &SourceTocInput<'_>, seed: &Seed) -> RecognizerResult<()> {
    let mut supported = BTreeSet::new();
    for id in &seed.evidence {
        let evidence = input
            .evidence
            .get(id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        evidence
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if evidence.evidence_id() != *id || evidence.context_id() != input.context.context_id() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        supported.extend(evidence.source_handle_ids().iter().copied());
    }
    for id in &seed.handles {
        let handle = input
            .source_handles
            .get(id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        handle
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if handle.handle_id() != *id
            || !supported.contains(id)
            || handle.project_generation() != input.context.project_generation()
            || handle.reference_generation() != Some(input.context.reference_generation())
        {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
    }
    Ok(())
}
fn confidence_of(value: RecognizerOutputConfidence) -> GraphConfidence {
    match value {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}
fn endpoint(
    value: &RecognizerFactValue,
    matched: &str,
    proposed: &BTreeMap<(String, String), String>,
    graph: &GraphSnapshot,
) -> RecognizerResult<GraphProposalEndpoint> {
    let RecognizerFactValue::Reference(token) = value else {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    if let Some(role) = token.strip_prefix("entity:") {
        let id = proposed
            .get(&(matched.to_owned(), role.to_owned()))
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        Ok(GraphProposalEndpoint::Proposed(id.clone().into()))
    } else {
        let node = GraphNodeId::new(token.clone()).map_err(graph_error)?;
        if graph.node(&node).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
        }
        Ok(GraphProposalEndpoint::Existing(node))
    }
}
