use std::collections::BTreeMap;
use std::error::Error;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use sha2::{Digest, Sha256};
use wow_core::{EvidenceId, GenerationContextId, StableHandleId, canonical_json_bytes};

use super::*;
use crate::{GraphEntityKindDefinition, GraphRelationKind, GraphRelationKindDefinition};

#[test]
fn qualified_base_preserves_raw_reports_and_errors_without_rebuilding_it()
-> Result<(), Box<dyn Error>> {
    let stop = AtomicBool::new(false);
    let limits = GraphLimits::default();
    let universe = GraphUniverseId::new("project:qualified-base-fixture")?;
    let generation = GraphGenerationId::new("project-generation:qualified-base-fixture:1")?;
    let registry = GraphRegistryBundle::build(
        "graph-registry:qualified-base-fixture",
        "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "function",
            vec!["project".into()],
            vec!["symbol".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
        vec![GraphRelationKindDefinition::new(
            "calls",
            GraphRelationKind::Calls,
            vec!["function".into()],
            vec!["function".into()],
            vec![GraphConfidence::Possible],
        )?],
    )?;
    let first = GraphNode::new(
        universe.clone(),
        generation.clone(),
        "function",
        "source:first",
        vec!["evidence:first".into()],
        limits,
    )?;
    let second = GraphNode::new(
        universe.clone(),
        generation.clone(),
        "function",
        "source:second",
        vec!["evidence:second".into()],
        limits,
    )?;
    let missing = GraphNode::new(
        universe.clone(),
        generation.clone(),
        "function",
        "source:missing",
        Vec::new(),
        limits,
    )?;
    let edge = GraphEdge::new(
        first.node_id().clone(),
        second.node_id().clone(),
        GraphRelationKind::Calls,
        GraphConfidence::Possible,
        vec!["evidence:edge".into()],
        limits,
    )?;
    let base = GraphSnapshot::build_validated(
        universe.clone(),
        generation.clone(),
        limits,
        vec![second, first.clone()],
        vec![edge],
        Vec::new(),
        &stop,
    )?;
    // Reconstruct the original identity object independently of the checked
    // sequence adapter, preserving the existing canonical identity recipe.
    let mut identity = serde_json::to_value(base.as_snapshot())?;
    identity
        .as_object_mut()
        .ok_or("snapshot is not an object")?
        .remove("snapshot_id")
        .ok_or("snapshot identity absent")?;
    assert_eq!(
        base.as_snapshot().snapshot_id().as_str(),
        format!(
            "graph-snapshot:sha256:{}",
            Sha256::digest(canonical_json_bytes(&identity)?)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join("")
        )
    );

    let mut batches = Vec::new();
    for index in 0..3 {
        let child_id = format!("proposal:child:{index}");
        let child = GraphEntityProposal::new(
            child_id.as_str(),
            "function",
            BTreeMap::from([(
                "symbol".into(),
                GraphProposalValue::Identifier(format!("child:{index}").into()),
            )]),
            GraphConfidence::Derived,
            vec![StableHandleId::derive(&(index, "source"))?],
            vec![EvidenceId::derive(&(index, "evidence"))?],
            Vec::new(),
        )?;
        let mut relations = Vec::new();
        for (name, target) in [
            ("accepted", GraphProposalEndpoint::Proposed(child_id.into())),
            (
                "missing",
                GraphProposalEndpoint::Existing(missing.node_id().clone()),
            ),
        ] {
            relations.push(GraphRelationProposal::new(
                format!("proposal:{name}:{index}"),
                "calls",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(first.node_id().clone()),
                    target,
                    confidence: GraphConfidence::Possible,
                    source_handle_ids: vec![StableHandleId::derive(&(index, name, "source"))?],
                    evidence_ids: vec![EvidenceId::derive(&(index, name, "evidence"))?],
                    coverage_ids: Vec::new(),
                },
            )?);
        }
        batches.push(GraphProposalBatch::build(
            registry.bundle_id(),
            registry.registry_digest(),
            universe.clone(),
            generation.clone(),
            GenerationContextId::derive(&"qualified-base-fixture")?,
            format!("producer:{index}"),
            vec![child],
            relations,
        )?);
    }

    let before_raw = crate::model::snapshot_work_counts();
    let raw_start = Instant::now();
    let expected = batches
        .iter()
        .map(|batch| {
            validate_graph_proposal_batch(&registry, Some(base.as_snapshot()), batch, limits)
        })
        .collect::<GraphResult<Vec<_>>>()?;
    let raw_elapsed = raw_start.elapsed();
    let after_raw = crate::model::snapshot_work_counts();
    assert_eq!(
        (after_raw.0 - before_raw.0, after_raw.1 - before_raw.1),
        (3, 3)
    );

    let before_shared = crate::model::snapshot_work_counts();
    let shared_start = Instant::now();
    let validator = GraphProposalValidator::new(&registry, &stop)?;
    for (batch, raw) in batches.iter().zip(&expected) {
        let shared = validator.validate(&base, batch, limits, &stop)?;
        assert_eq!(canonical_json_bytes(&shared)?, canonical_json_bytes(raw)?);
        assert_eq!(shared.accepted_entities().len(), 1);
        assert_eq!(shared.accepted_relations().len(), 1);
        assert_eq!(shared.rejections().len(), 1);
    }
    let shared_elapsed = shared_start.elapsed();
    let after_shared = crate::model::snapshot_work_counts();
    assert_eq!(after_shared, before_shared);

    let different_generation = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe,
        GraphGenerationId::new("project-generation:qualified-base-fixture:2")?,
        GenerationContextId::derive(&"qualified-base-fixture")?,
        "producer:wrong-generation",
        Vec::new(),
        Vec::new(),
    )?;
    let raw_error = validate_graph_proposal_batch(
        &registry,
        Some(base.as_snapshot()),
        &different_generation,
        limits,
    )
    .err()
    .ok_or("raw generation mismatch accepted")?;
    let shared_error = validator
        .validate(&base, &different_generation, limits, &stop)
        .err()
        .ok_or("qualified generation mismatch accepted")?;
    assert_eq!(raw_error.code(), shared_error.code());
    assert_eq!(shared_error.code(), GraphErrorCode::GenerationMismatch);
    assert_eq!(
        validator
            .validate(&base, &batches[0], limits, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled batch accepted")?
            .code(),
        GraphErrorCode::Cancelled
    );

    println!(
        "qualified_base_witness debug_assertions={} vertices=2 edges=1 batches=3 raw_base_validations=3 raw_snapshot_builds=3 qualified_base_validations=0 qualified_snapshot_builds=0 raw_us={} qualified_us={}",
        cfg!(debug_assertions),
        raw_elapsed.as_micros(),
        shared_elapsed.as_micros()
    );
    Ok(())
}
