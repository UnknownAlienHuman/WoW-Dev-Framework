use std::collections::BTreeMap;

use wow_graph::{
    GraphConfidence, GraphEntityProposal, GraphGenerationId, GraphNodeId, GraphProposalBatch,
    GraphProposalEndpoint, GraphProposalValue, GraphRegistryBundle, GraphRelationProposal,
    GraphRelationProposalInput, GraphUniverseId,
};

use crate::{
    CompiledRecognizerPack, CompiledRecognizerPlan, RecognizerError, RecognizerErrorCode,
    RecognizerFactValue, RecognizerOutputConfidence, RecognizerOutputPartition,
    RecognizerProposedAssertion, RecognizerResult, RecognizerRuleOutcomeState,
};

/// Converts one complete immutable recognizer output partition into one graph-owned proposal batch.
///
/// This operation does not validate graph kinds/endpoints or publish graph state. Those remain
/// `wow-graph` responsibilities. Partial, cancelled, truncated, or blocked recognizer outcomes do
/// not become replacement batches under the default E2-B handoff policy.
pub fn adapt_recognizer_output_to_graph(
    output: &RecognizerOutputPartition,
    pack: &CompiledRecognizerPack,
    plan: &CompiledRecognizerPlan,
    registry: &GraphRegistryBundle,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
) -> RecognizerResult<GraphProposalBatch> {
    output.validate()?;
    pack.validate()?;
    plan.validate()?;
    registry.validate().map_err(graph_error)?;
    if output.source_pack_digest() != pack.pack_digest()
        || output.plan_id() != plan.plan_id()
        || plan.source_pack_digest() != pack.pack_digest()
        || pack.document().pack.graph_registry_bundle_id.as_ref() != registry.bundle_id()
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::GraphHandoffMismatch,
            "recognizer output, pack, plan, and graph registry identities do not match",
        ));
    }
    if output.outcomes().iter().any(|outcome| {
        outcome.truncated()
            || !outcome.blocker_ids().is_empty()
            || !matches!(
                outcome.state(),
                RecognizerRuleOutcomeState::Matched
                    | RecognizerRuleOutcomeState::EvaluatedNoMatch
            )
    }) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::GraphHandoffIncomplete,
            "partial, blocked, cancelled, or truncated recognizer output cannot replace a graph partition",
        ));
    }

    let mut entities = Vec::new();
    let mut relations = Vec::new();
    for proposal in output
        .outcomes()
        .iter()
        .flat_map(|outcome| outcome.proposals())
    {
        match proposal {
            RecognizerProposedAssertion::Entity {
                proposal_id,
                entity_kind_id,
                semantic_key,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } => {
                let semantic_key = semantic_key
                    .iter()
                    .map(|(field, value)| Ok((field.clone(), proposal_value(value)?)))
                    .collect::<RecognizerResult<BTreeMap<_, _>>>()?;
                entities.push(
                    GraphEntityProposal::new(
                        proposal_id.as_str(),
                        entity_kind_id.clone(),
                        semantic_key,
                        graph_confidence(*confidence),
                        source_handle_ids.clone(),
                        evidence_ids.clone(),
                        coverage_ids.clone(),
                    )
                    .map_err(graph_error)?,
                );
            }
            RecognizerProposedAssertion::Relation {
                proposal_id,
                relation_kind_id,
                source,
                target,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } => {
                relations.push(
                    GraphRelationProposal::new(
                        proposal_id.as_str(),
                        relation_kind_id.clone(),
                        GraphRelationProposalInput {
                            source: proposal_endpoint(source)?,
                            target: proposal_endpoint(target)?,
                            confidence: graph_confidence(*confidence),
                            source_handle_ids: source_handle_ids.clone(),
                            evidence_ids: evidence_ids.clone(),
                            coverage_ids: coverage_ids.clone(),
                        },
                    )
                    .map_err(graph_error)?,
                );
            }
        }
    }
    GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe,
        generation,
        output.context_id(),
        output.producer_partition_id(),
        entities,
        relations,
    )
    .map_err(graph_error)
}

fn proposal_value(value: &RecognizerFactValue) -> RecognizerResult<GraphProposalValue> {
    match value {
        RecognizerFactValue::Nil => Err(RecognizerError::new(
            RecognizerErrorCode::GraphHandoffValueUnsupported,
            "nil cannot be used as a graph semantic-key ingredient",
        )),
        RecognizerFactValue::Boolean(value) => Ok(GraphProposalValue::Boolean(*value)),
        RecognizerFactValue::Integer(value) => Ok(GraphProposalValue::Integer(*value)),
        RecognizerFactValue::String(value) => Ok(GraphProposalValue::String(value.clone())),
        RecognizerFactValue::Identifier(value) | RecognizerFactValue::Tag(value) => {
            Ok(GraphProposalValue::Identifier(value.clone()))
        }
        RecognizerFactValue::Reference(value) => Ok(GraphProposalValue::Reference(value.clone())),
    }
}

fn proposal_endpoint(value: &RecognizerFactValue) -> RecognizerResult<GraphProposalEndpoint> {
    let RecognizerFactValue::Reference(value) = value else {
        return Err(RecognizerError::new(
            RecognizerErrorCode::GraphHandoffValueUnsupported,
            "graph relation endpoints must be exact graph-node or recognizer-proposal references",
        ));
    };
    if value.starts_with("graph-node:sha256:") {
        return GraphNodeId::new(value.clone())
            .map(GraphProposalEndpoint::Existing)
            .map_err(graph_error);
    }
    if value.starts_with("recognizer-proposal:sha256:") {
        return Ok(GraphProposalEndpoint::Proposed(value.clone()));
    }
    Err(RecognizerError::new(
        RecognizerErrorCode::GraphHandoffValueUnsupported,
        "graph relation endpoint reference uses an unsupported identity domain",
    ))
}

const fn graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    RecognizerError::new(
        RecognizerErrorCode::GraphHandoffInvalid,
        format!("graph proposal handoff failed: {}", error.message()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_domain_is_explicit() {
        let proposed = RecognizerFactValue::Reference(
            format!("recognizer-proposal:sha256:{}", "a".repeat(64)).into(),
        );
        assert!(matches!(
            proposal_endpoint(&proposed),
            Ok(GraphProposalEndpoint::Proposed(_))
        ));
        let unsupported = RecognizerFactValue::Reference("display-name".into());
        assert_eq!(
            proposal_endpoint(&unsupported)
                .err()
                .expect("unsupported endpoint")
                .code(),
            RecognizerErrorCode::GraphHandoffValueUnsupported
        );
    }

    #[test]
    fn nil_semantic_key_is_not_widened() {
        assert_eq!(
            proposal_value(&RecognizerFactValue::Nil)
                .err()
                .expect("nil key")
                .code(),
            RecognizerErrorCode::GraphHandoffValueUnsupported
        );
    }
}
