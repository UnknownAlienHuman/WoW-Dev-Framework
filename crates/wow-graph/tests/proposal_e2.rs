use std::collections::BTreeMap;
use std::error::Error;

use serde_json::json;
use wow_core::{CoverageId, EvidenceId, GenerationContextId, StableHandleId};
use wow_graph::{
    GraphConfidence, GraphEntityKindDefinition, GraphEntityProposal, GraphErrorCode,
    GraphGenerationId, GraphLimits, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalRejectionCode, GraphProposalValue, GraphRegistryBundle, GraphRelationKind,
    GraphRelationKindDefinition, GraphRelationProposal, GraphRelationProposalInput,
    GraphUniverseId, validate_graph_proposal_batch,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type ProposalSupport = (Vec<StableHandleId>, Vec<EvidenceId>, Vec<CoverageId>);

fn universe() -> Result<GraphUniverseId, wow_graph::GraphError> {
    GraphUniverseId::new("project:proposal-fixture")
}

fn generation() -> Result<GraphGenerationId, wow_graph::GraphError> {
    GraphGenerationId::new("project-generation:proposal-fixture:1")
}

fn context_id() -> Result<GenerationContextId, wow_core::CoreError> {
    GenerationContextId::derive(&"graph-proposal-fixture")
}

fn support(value: &str) -> Result<ProposalSupport, wow_core::CoreError> {
    Ok((
        vec![StableHandleId::derive(&(value, "handle"))?],
        vec![EvidenceId::derive(&(value, "evidence"))?],
        vec![CoverageId::derive(&(value, "coverage"))?],
    ))
}

fn registry() -> Result<GraphRegistryBundle, wow_graph::GraphError> {
    GraphRegistryBundle::build(
        "graph-registry:e2-a:fixture",
        "1.0.0",
        vec![
            GraphEntityKindDefinition::new(
                "api_symbol",
                vec!["project".into()],
                vec!["entity".into()],
                vec![GraphConfidence::Proven, GraphConfidence::Derived],
            )?,
            GraphEntityKindDefinition::new(
                "function",
                vec!["project".into()],
                vec!["symbol".into()],
                vec![
                    GraphConfidence::Proven,
                    GraphConfidence::Derived,
                    GraphConfidence::Possible,
                ],
            )?,
        ],
        vec![GraphRelationKindDefinition::new(
            "uses_api",
            GraphRelationKind::UsesApi,
            vec!["function".into()],
            vec!["api_symbol".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
    )
}

fn entity(
    proposal_id: &str,
    kind: &str,
    field: &str,
    value: &str,
    confidence: GraphConfidence,
) -> TestResult<GraphEntityProposal> {
    let (handles, evidence, coverage) = support(proposal_id)?;
    Ok(GraphEntityProposal::new(
        proposal_id,
        kind,
        BTreeMap::from([(
            Box::<str>::from(field),
            GraphProposalValue::Identifier(value.into()),
        )]),
        confidence,
        handles,
        evidence,
        coverage,
    )?)
}

fn batch(reverse: bool) -> TestResult<GraphProposalBatch> {
    let registry = registry()?;
    let function = entity(
        "recognizer-proposal:function",
        "function",
        "symbol",
        "function:OnLoad",
        GraphConfidence::Derived,
    )?;
    let api = entity(
        "recognizer-proposal:api",
        "api_symbol",
        "entity",
        "C_Fixture.KnownApi",
        GraphConfidence::Proven,
    )?;
    let (handles, evidence, coverage) = support("recognizer-proposal:uses-api")?;
    let relation = GraphRelationProposal::new(
        "recognizer-proposal:uses-api",
        "uses_api",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed("recognizer-proposal:function".into()),
            target: GraphProposalEndpoint::Proposed("recognizer-proposal:api".into()),
            confidence: GraphConfidence::Derived,
            source_handle_ids: handles,
            evidence_ids: evidence,
            coverage_ids: coverage,
        },
    )?;
    let mut entities = vec![function, api];
    if reverse {
        entities.reverse();
    }
    Ok(GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe()?,
        generation()?,
        context_id()?,
        "recognizer:core:project.main",
        entities,
        vec![relation],
    )?)
}

#[test]
fn registry_and_batch_are_content_addressed_and_order_invariant() -> TestResult {
    let registry = registry()?;
    registry.validate()?;
    assert!(
        registry
            .registry_digest()
            .starts_with("graph-registry:sha256:")
    );
    let left = batch(false)?;
    let right = batch(true)?;
    assert_eq!(left, right);
    assert_eq!(left.batch_id(), right.batch_id());
    left.validate()?;
    Ok(())
}

#[test]
fn same_batch_entities_and_relation_validate_to_exact_graph_records() -> TestResult {
    let registry = registry()?;
    let report =
        validate_graph_proposal_batch(&registry, None, &batch(false)?, GraphLimits::default())?;
    assert!(report.ready_for_publication());
    assert!(report.rejections().is_empty());
    assert_eq!(report.accepted_entities().len(), 2);
    assert_eq!(report.accepted_relations().len(), 1);
    assert_eq!(
        report.accepted_relations()[0].edge().relation(),
        GraphRelationKind::UsesApi
    );
    assert_eq!(
        report.accepted_relations()[0].edge().confidence(),
        GraphConfidence::Derived
    );
    report.validate(GraphLimits::default())?;
    Ok(())
}

#[test]
fn unknown_kind_wrong_identity_confidence_and_endpoint_pair_are_explicit_rejections() -> TestResult
{
    let registry = registry()?;
    let unknown = entity(
        "recognizer-proposal:unknown",
        "missing_kind",
        "symbol",
        "function:Unknown",
        GraphConfidence::Derived,
    )?;
    let wrong_fields = entity(
        "recognizer-proposal:wrong-fields",
        "function",
        "wrong",
        "function:Wrong",
        GraphConfidence::Derived,
    )?;
    let wrong_confidence = entity(
        "recognizer-proposal:wrong-confidence",
        "api_symbol",
        "entity",
        "C_Fixture.Possible",
        GraphConfidence::Possible,
    )?;
    let (handles, evidence, coverage) = support("recognizer-proposal:wrong-pair")?;
    let relation = GraphRelationProposal::new(
        "recognizer-proposal:wrong-pair",
        "uses_api",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed("recognizer-proposal:wrong-confidence".into()),
            target: GraphProposalEndpoint::Proposed("recognizer-proposal:wrong-confidence".into()),
            confidence: GraphConfidence::Derived,
            source_handle_ids: handles,
            evidence_ids: evidence,
            coverage_ids: coverage,
        },
    )?;
    let batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe()?,
        generation()?,
        context_id()?,
        "recognizer:invalid",
        vec![unknown, wrong_fields, wrong_confidence],
        vec![relation],
    )?;
    let report = validate_graph_proposal_batch(&registry, None, &batch, GraphLimits::default())?;
    assert!(!report.ready_for_publication());
    let codes = report
        .rejections()
        .iter()
        .map(|item| item.code())
        .collect::<Vec<_>>();
    assert!(codes.contains(&GraphProposalRejectionCode::UnknownEntityKind));
    assert!(codes.contains(&GraphProposalRejectionCode::IdentityFieldsMismatch));
    assert!(codes.contains(&GraphProposalRejectionCode::ConfidenceNotAllowed));
    assert!(codes.contains(&GraphProposalRejectionCode::EndpointMissing));
    Ok(())
}

#[test]
fn registry_or_report_identity_tampering_fails_closed() -> TestResult {
    let registry = registry()?;
    let batch = batch(false)?;
    let mut batch_value = serde_json::to_value(&batch)?;
    batch_value["registry_digest"] = json!(format!("graph-registry:sha256:{}", "0".repeat(64)));
    let tampered_batch: GraphProposalBatch = serde_json::from_value(batch_value)?;
    assert_eq!(
        validate_graph_proposal_batch(&registry, None, &tampered_batch, GraphLimits::default(),)
            .err()
            .ok_or("tampered registry binding must fail")?
            .code(),
        GraphErrorCode::ProposalBatchIdentityMismatch
    );

    let report = validate_graph_proposal_batch(&registry, None, &batch, GraphLimits::default())?;
    let mut report_value = serde_json::to_value(&report)?;
    report_value["report_id"] = json!(format!("graph-proposal-report:sha256:{}", "f".repeat(64)));
    let tampered_report =
        serde_json::from_value::<wow_graph::GraphProposalValidationReport>(report_value)?;
    assert_eq!(
        tampered_report
            .validate(GraphLimits::default())
            .err()
            .ok_or("tampered report identity must fail")?
            .code(),
        GraphErrorCode::ProposalReportIdentityMismatch
    );
    Ok(())
}
