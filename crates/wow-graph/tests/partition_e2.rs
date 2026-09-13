use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, atomic::AtomicBool};

use wow_core::{EvidenceId, GenerationContextId, StableHandleId};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEntityKindDefinition,
    GraphEntityProposal, GraphErrorCode, GraphGenerationId, GraphLimits, GraphNode, GraphNodeId,
    GraphPartitionReplacement, GraphPartitionSession, GraphPartitionSnapshot, GraphProposalBatch,
    GraphProposalEndpoint, GraphProposalValue, GraphRegistryBundle, GraphRelationKind,
    GraphRelationKindDefinition, GraphRelationProposal, GraphRelationProposalInput, GraphSnapshot,
    GraphUniverseId,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn registry() -> TestResult<GraphRegistryBundle> {
    Ok(GraphRegistryBundle::build(
        "graph-registry:partition-fixture", "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "function", vec!["project".into()], vec!["symbol".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
        vec![GraphRelationKindDefinition::new(
            "calls", GraphRelationKind::Calls, vec!["function".into()], vec!["function".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
    )?)
}

fn universe() -> TestResult<GraphUniverseId> {
    Ok(GraphUniverseId::new("project:partition-fixture")?)
}

fn generation() -> TestResult<GraphGenerationId> {
    Ok(GraphGenerationId::new("project-generation:partition-fixture:1")?)
}

fn context() -> TestResult<GenerationContextId> {
    Ok(GenerationContextId::derive(&"graph-partition-inputs")?)
}

fn coverage() -> TestResult<Vec<GraphCoverageRecord>> {
    Ok(vec![GraphCoverageRecord::new(
        GraphRelationKind::Calls, GraphCoverageState::Complete, false, Vec::new(),
        GraphLimits::default(),
    )?])
}

fn initial() -> TestResult<GraphPartitionSnapshot> {
    let limits = GraphLimits::default();
    let node = GraphNode::new(
        universe()?, generation()?, "function", "source:root", vec!["evidence:root".into()], limits,
    )?;
    let foundation = GraphSnapshot::build(
        universe()?, generation()?, limits, vec![node], Vec::new(), coverage()?,
    )?;
    Ok(GraphPartitionSnapshot::new(registry()?, foundation, context()?, &AtomicBool::new(false))?)
}

fn batch(id: &str, symbol: Option<&str>) -> TestResult<GraphProposalBatch> {
    let registry = registry()?;
    let entities = if let Some(symbol) = symbol {
        vec![GraphEntityProposal::new(
            format!("proposal:{id}"), "function",
            BTreeMap::from([(Box::<str>::from("symbol"), GraphProposalValue::Identifier(symbol.into()))]),
            GraphConfidence::Derived,
            vec![StableHandleId::derive(&(id, "source"))?],
            vec![EvidenceId::derive(&(id, "evidence"))?], Vec::new(),
        )?]
    } else { Vec::new() };
    Ok(GraphProposalBatch::build(
        registry.bundle_id(), registry.registry_digest(), universe()?, generation()?,
        context()?, id, entities, Vec::new(),
    )?)
}

fn edge_batch(id: &str, from: &GraphNodeId, to: &GraphNodeId) -> TestResult<GraphProposalBatch> {
    let registry = registry()?;
    let edge = GraphRelationProposal::new(
        format!("proposal:{id}"), "calls",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Existing(from.clone()),
            target: GraphProposalEndpoint::Existing(to.clone()),
            confidence: GraphConfidence::Possible,
            source_handle_ids: vec![StableHandleId::derive(&(id, "source"))?],
            evidence_ids: vec![EvidenceId::derive(&(id, "edge"))?], coverage_ids: Vec::new(),
        },
    )?;
    Ok(GraphProposalBatch::build(
        registry.bundle_id(), registry.registry_digest(), universe()?, generation()?,
        context()?, id, Vec::new(), vec![edge],
    )?)
}

fn request(view: &GraphPartitionSnapshot, batch: GraphProposalBatch) -> TestResult<GraphPartitionReplacement> {
    Ok(GraphPartitionReplacement {
        expected_snapshot_id: view.snapshot().snapshot_id().clone(),
        expected_partition_digest: view.partition(batch.producer_partition_id())
            .map(|item| item.partition_digest().into()),
        producer_version: "1.0.0".into(), batch, coverage: coverage()?,
    })
}

fn replace(view: &GraphPartitionSnapshot, batch: GraphProposalBatch) -> TestResult<GraphPartitionSnapshot> {
    Ok(view.prepare_replacement(request(view, batch)?, &AtomicBool::new(false))?.candidate().clone())
}

fn contributed_node(view: &GraphPartitionSnapshot) -> TestResult<GraphNode> {
    Ok(view.input_view(&AtomicBool::new(false))?.nodes().iter()
        .find(|node| node.owner_key() != "source:root").ok_or("contributed node missing")?.clone())
}

#[test]
fn independent_partition_order_has_identical_generation_and_bytes() -> TestResult {
    let initial = initial()?;
    let left = replace(&replace(&initial, batch("producer:a", Some("A"))?)?, batch("producer:b", Some("B"))?)?;
    let right = replace(&replace(&initial, batch("producer:b", Some("B"))?)?, batch("producer:a", Some("A"))?)?;
    assert_eq!(left, right);
    assert_eq!(serde_json::to_vec(&left)?, serde_json::to_vec(&right)?);
    assert_ne!(left.snapshot().generation(), initial.snapshot().generation());
    assert_eq!(left.input_view(&AtomicBool::new(false))?.generation(), &generation()?);
    assert!(left.snapshot().generation().as_str().starts_with("graph-generation:sha256:"));
    assert!(left.snapshot().nodes().iter().all(|node| node.generation() == left.snapshot().generation()));
    left.validate(&AtomicBool::new(false))?;
    Ok(())
}

#[test]
fn shared_assertions_survive_other_producer_removal_with_exact_support() -> TestResult {
    let initial = initial()?;
    let a = replace(&initial, batch("producer:a", Some("Shared"))?)?;
    let both = replace(&a, batch("producer:b", Some("Shared"))?)?;
    assert_eq!(contributed_node(&both)?.evidence_ids().len(), 2);
    let saved_b = both.partition("producer:b").ok_or("partition b")?.clone();
    let mut removal = request(&both, batch("producer:a", None)?)?;
    removal.coverage.clear();
    let plan = both.prepare_replacement(removal, &AtomicBool::new(false))?;
    let after = plan.candidate();
    assert_eq!(after.partition("producer:b"), Some(&saved_b));
    assert_eq!(contributed_node(after)?.evidence_ids().len(), 1);
    assert_eq!(after.snapshot().nodes().len(), 2);
    assert_eq!(after.snapshot().coverage()[0].state(), GraphCoverageState::NotEvaluated);
    assert!(!after.snapshot().coverage()[0].negative_authority());
    assert!(after.partition("producer:a").ok_or("tombstone")?.report().accepted_entities().is_empty());
    assert_eq!(contributed_node(&both)?.evidence_ids().len(), 2);
    Ok(())
}

#[test]
fn replacement_removes_stale_nodes_and_preserves_old_readers() -> TestResult {
    let stopped = AtomicBool::new(false);
    let first = replace(&initial()?, batch("producer:a", Some("Old"))?)?;
    let mut session = GraphPartitionSession::new(first, &stopped)?;
    let old = session.view();
    let old_node = contributed_node(&old)?.node_id().clone();
    let plan = old.prepare_replacement(request(&old, batch("producer:a", Some("New"))?)?, &stopped)?;
    let current = session.publish(plan, &stopped)?;
    assert!(old.input_view(&stopped)?.node(&old_node).is_some());
    assert!(current.input_view(&stopped)?.node(&old_node).is_none());
    assert!(!Arc::ptr_eq(&old, &current));
    assert_eq!(current.snapshot().nodes().len(), 2);
    let repeat = current.prepare_replacement(request(&current, batch("producer:a", Some("New"))?)?, &stopped)?;
    assert!(Arc::ptr_eq(&current, &session.publish(repeat, &stopped)?));
    Ok(())
}

#[test]
fn stale_snapshot_partition_version_and_competing_plan_do_not_publish() -> TestResult {
    let stopped = AtomicBool::new(false);
    let first = initial()?;
    let mut session = GraphPartitionSession::new(first.clone(), &stopped)?;
    let plan_a = first.prepare_replacement(request(&first, batch("producer:a", Some("A"))?)?, &stopped)?;
    let plan_b = first.prepare_replacement(request(&first, batch("producer:b", Some("B"))?)?, &stopped)?;
    let active = session.publish(plan_a, &stopped)?;
    assert_eq!(session.publish(plan_b, &stopped).err().ok_or("stale plan accepted")?.code(), GraphErrorCode::PartitionStale);
    assert!(Arc::ptr_eq(&active, &session.view()));
    let mut stale = request(&active, batch("producer:a", Some("Changed"))?)?;
    stale.expected_partition_digest = Some("graph-partition:sha256:stale".into());
    assert_eq!(active.prepare_replacement(stale, &stopped).err().ok_or("stale partition accepted")?.code(), GraphErrorCode::PartitionStale);
    let mut versioned = request(&active, batch("producer:a", Some("A"))?)?;
    versioned.producer_version = "2.0.0".into();
    let next = active.prepare_replacement(versioned, &stopped)?;
    assert_ne!(next.candidate().snapshot().generation(), active.snapshot().generation());
    Ok(())
}

#[test]
fn dangling_survivor_and_removed_self_endpoint_fail_closed() -> TestResult {
    let stopped = AtomicBool::new(false);
    let a = replace(&initial()?, batch("producer:a", Some("A"))?)?;
    let contributed = contributed_node(&a)?;
    let root = a.foundation().nodes()[0].node_id();
    let b = replace(&a, edge_batch("producer:b", root, contributed.node_id())?)?;
    assert_eq!(b.snapshot().edges()[0].confidence(), GraphConfidence::Possible);
    let removal = request(&b, batch("producer:a", None)?)?;
    assert_eq!(b.prepare_replacement(removal, &stopped).err().ok_or("dangling edge accepted")?.code(), GraphErrorCode::EndpointMissing);
    let self_reference = request(&a, edge_batch("producer:a", root, contributed.node_id())?)?;
    assert_eq!(a.prepare_replacement(self_reference, &stopped).err().ok_or("removed self endpoint accepted")?.code(), GraphErrorCode::PartitionRejected);
    // Reasserting the same semantic node is permitted while other producers use it.
    replace(&b, batch("producer:a", Some("A"))?)?.validate(&stopped)?;
    Ok(())
}

#[test]
fn cancellation_before_validation_or_publication_preserves_current() -> TestResult {
    let first = initial()?;
    let request = request(&first, batch("producer:a", Some("A"))?)?;
    assert_eq!(first.prepare_replacement(request.clone(), &AtomicBool::new(true)).err().ok_or("cancelled plan accepted")?.code(), GraphErrorCode::Cancelled);
    let plan = first.prepare_replacement(request, &AtomicBool::new(false))?;
    let mut session = GraphPartitionSession::new(first, &AtomicBool::new(false))?;
    let before = session.view();
    assert_eq!(session.publish(plan, &AtomicBool::new(true)).err().ok_or("cancelled publication accepted")?.code(), GraphErrorCode::Cancelled);
    assert!(Arc::ptr_eq(&before, &session.view()));
    Ok(())
}

#[test]
fn serialized_manifest_report_and_projection_tampering_is_rejected() -> TestResult {
    let view = replace(&initial()?, batch("producer:a", Some("A"))?)?;
    let bytes = serde_json::to_vec(&view)?;
    let restored: GraphPartitionSnapshot = serde_json::from_slice(&bytes)?;
    restored.validate(&AtomicBool::new(false))?;
    for (pointer, replacement) in [
        ("/partitions/0/producer_version", serde_json::json!("9.0.0")),
        ("/partitions/0/report/accepted_entities", serde_json::json!([])),
        ("/snapshot/nodes", serde_json::json!([])),
        ("/partitions/0/coverage/0/negative_authority", serde_json::json!(true)),
        ("/schema", serde_json::json!("unrecognized")),
    ] {
        let mut value = serde_json::to_value(&view)?;
        *value.pointer_mut(pointer).ok_or("missing mutation target")? = replacement;
        let changed: GraphPartitionSnapshot = serde_json::from_value(value)?;
        assert!(changed.validate(&AtomicBool::new(false)).is_err(), "{pointer}");
    }
    Ok(())
}

#[test]
fn producer_cannot_grant_negative_authority_or_exceed_assertion_budget() -> TestResult {
    let view = initial()?;
    let mut elevated = request(&view, batch("producer:a", Some("A"))?)?;
    elevated.coverage = vec![GraphCoverageRecord::new(
        GraphRelationKind::Calls, GraphCoverageState::Complete, true, Vec::new(), GraphLimits::default(),
    )?];
    assert_eq!(view.prepare_replacement(elevated, &AtomicBool::new(false)).err().ok_or("authority upgrade accepted")?.code(), GraphErrorCode::PartitionInvalid);
    let small = GraphLimits::new(1, 10, 10, 10, 10)?;
    let base = GraphSnapshot::build(universe()?, generation()?, small, view.foundation().nodes().to_vec(), Vec::new(), coverage()?)?;
    let limited = GraphPartitionSnapshot::new(registry()?, base, context()?, &AtomicBool::new(false))?;
    assert_eq!(limited.prepare_replacement(request(&limited, batch("producer:a", Some("A"))?)?, &AtomicBool::new(false)).err().ok_or("budget exceeded")?.code(), GraphErrorCode::BudgetExceeded);
    Ok(())
}

#[test]
fn mixed_source_context_is_rejected_before_plan_creation() -> TestResult {
    let view = initial()?;
    let registry = registry()?;
    let foreign = GraphProposalBatch::build(
        registry.bundle_id(), registry.registry_digest(), universe()?, generation()?,
        GenerationContextId::derive(&"another-project-generation")?, "producer:a", Vec::new(), Vec::new(),
    )?;
    assert_eq!(view.prepare_replacement(request(&view, foreign)?, &AtomicBool::new(false)).err().ok_or("mixed context accepted")?.code(), GraphErrorCode::GenerationMismatch);
    Ok(())
}
