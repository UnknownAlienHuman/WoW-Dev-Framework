use std::error::Error;

use serde_json::json;
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphDirection, GraphEdge,
    GraphErrorCode, GraphGenerationId, GraphLimits, GraphNeighborQuery, GraphNode,
    GraphPublicationKey, GraphQueryState, GraphRelationKind, GraphSnapshot, GraphUniverseId,
    PersistentGraphStore,
};
use wow_store::{
    CatalogExpectation, PendingObject, Store, StoreConfiguration, StoreLimits, WriteBatch,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn limits() -> GraphLimits {
    GraphLimits::default()
}

fn universe() -> Result<GraphUniverseId, wow_graph::GraphError> {
    GraphUniverseId::new("project:fixture")
}

fn generation() -> Result<GraphGenerationId, wow_graph::GraphError> {
    GraphGenerationId::new("project-generation:fixture:1")
}

fn node(key: &str) -> Result<GraphNode, wow_graph::GraphError> {
    GraphNode::new(
        universe()?,
        generation()?,
        "function",
        key,
        vec![format!("evidence:{key}").into_boxed_str()],
        limits(),
    )
}

fn coverage(
    relation: GraphRelationKind,
    state: GraphCoverageState,
    negative_authority: bool,
) -> Result<GraphCoverageRecord, wow_graph::GraphError> {
    let blockers = if state == GraphCoverageState::Failed {
        vec!["blocker:fixture".into()]
    } else {
        Vec::new()
    };
    GraphCoverageRecord::new(relation, state, negative_authority, blockers, limits())
}

fn snapshot(reverse: bool) -> Result<GraphSnapshot, wow_graph::GraphError> {
    let alpha = node("alpha")?;
    let beta = node("beta")?;
    let gamma = node("gamma")?;
    let call_beta = GraphEdge::new(
        alpha.node_id().clone(),
        beta.node_id().clone(),
        GraphRelationKind::Calls,
        GraphConfidence::Proven,
        vec!["evidence:call-alpha-beta".into()],
        limits(),
    )?;
    let call_gamma = GraphEdge::new(
        alpha.node_id().clone(),
        gamma.node_id().clone(),
        GraphRelationKind::Calls,
        GraphConfidence::Derived,
        vec!["evidence:call-alpha-gamma".into()],
        limits(),
    )?;
    let signal = GraphEdge::new(
        beta.node_id().clone(),
        gamma.node_id().clone(),
        GraphRelationKind::EmitsCustomSignal,
        GraphConfidence::Possible,
        vec!["evidence:custom-signal".into()],
        limits(),
    )?;
    let api = GraphEdge::new(
        alpha.node_id().clone(),
        gamma.node_id().clone(),
        GraphRelationKind::UsesApi,
        GraphConfidence::Derived,
        vec!["evidence:api-use".into()],
        limits(),
    )?;
    let mut nodes = vec![alpha, beta, gamma];
    let mut edges = vec![call_beta, call_gamma, signal, api];
    let mut coverage = vec![
        coverage(
            GraphRelationKind::Calls,
            GraphCoverageState::Complete,
            true,
        )?,
        coverage(
            GraphRelationKind::UsesApi,
            GraphCoverageState::Complete,
            true,
        )?,
        coverage(
            GraphRelationKind::HandlesNativeEvent,
            GraphCoverageState::Complete,
            true,
        )?,
        coverage(
            GraphRelationKind::EmitsCustomSignal,
            GraphCoverageState::Partial,
            false,
        )?,
    ];
    if reverse {
        nodes.reverse();
        edges.reverse();
        coverage.reverse();
    }
    GraphSnapshot::build(
        universe()?,
        generation()?,
        limits(),
        nodes,
        edges,
        coverage,
    )
}

#[test]
fn shuffled_inputs_produce_identical_snapshot_identity_and_bytes() -> TestResult {
    let left = snapshot(false)?;
    let right = snapshot(true)?;
    assert_eq!(left, right);
    assert_eq!(left.snapshot_id(), right.snapshot_id());
    assert_eq!(serde_json::to_vec(&left)?, serde_json::to_vec(&right)?);
    left.validate()?;
    Ok(())
}

#[test]
fn direct_queries_preserve_relation_direction_and_authority() -> TestResult {
    let snapshot = snapshot(false)?;
    let alpha = snapshot
        .nodes()
        .iter()
        .find(|node| node.owner_key() == "alpha")
        .ok_or("missing alpha")?;
    let beta = snapshot
        .nodes()
        .iter()
        .find(|node| node.owner_key() == "beta")
        .ok_or("missing beta")?;

    let outgoing = GraphNeighborQuery::new(
        alpha.node_id().clone(),
        GraphDirection::Outgoing,
        vec![GraphRelationKind::Calls],
        10,
    )?
    .execute(&snapshot)?;
    assert_eq!(outgoing.state(), GraphQueryState::Complete);
    assert_eq!(outgoing.edges().len(), 2);
    assert_eq!(outgoing.adjacent_nodes().len(), 2);
    assert!(!outgoing.absence_authoritative());
    assert!(
        outgoing
            .edges()
            .iter()
            .all(|edge| edge.relation() == GraphRelationKind::Calls)
    );

    let incoming = GraphNeighborQuery::new(
        beta.node_id().clone(),
        GraphDirection::Incoming,
        vec![GraphRelationKind::Calls],
        10,
    )?
    .execute(&snapshot)?;
    assert_eq!(incoming.edges().len(), 1);
    assert_eq!(incoming.adjacent_nodes()[0].owner_key(), "alpha");

    let absent = GraphNeighborQuery::new(
        alpha.node_id().clone(),
        GraphDirection::Outgoing,
        vec![GraphRelationKind::HandlesNativeEvent],
        10,
    )?
    .execute(&snapshot)?;
    assert_eq!(absent.state(), GraphQueryState::Complete);
    assert!(absent.edges().is_empty());
    assert!(absent.absence_authoritative());

    let partial = GraphNeighborQuery::new(
        alpha.node_id().clone(),
        GraphDirection::Outgoing,
        vec![GraphRelationKind::EmitsCustomSignal],
        10,
    )?
    .execute(&snapshot)?;
    assert_eq!(partial.state(), GraphQueryState::Partial);
    assert!(!partial.absence_authoritative());

    let not_evaluated = GraphNeighborQuery::new(
        alpha.node_id().clone(),
        GraphDirection::Outgoing,
        vec![GraphRelationKind::Owns],
        10,
    )?
    .execute(&snapshot)?;
    assert_eq!(not_evaluated.state(), GraphQueryState::NotEvaluated);
    assert!(!not_evaluated.absence_authoritative());
    Ok(())
}

#[test]
fn truncation_never_becomes_complete_or_negative_authority() -> TestResult {
    let snapshot = snapshot(false)?;
    let alpha = snapshot
        .nodes()
        .iter()
        .find(|node| node.owner_key() == "alpha")
        .ok_or("missing alpha")?;
    let result = GraphNeighborQuery::new(
        alpha.node_id().clone(),
        GraphDirection::Outgoing,
        vec![GraphRelationKind::Calls],
        1,
    )?
    .execute(&snapshot)?;
    assert_eq!(result.state(), GraphQueryState::Truncated);
    assert_eq!(result.edges().len(), 1);
    assert!(!result.absence_authoritative());
    Ok(())
}

#[test]
fn invalid_endpoints_duplicates_self_edges_and_mixed_generations_fail() -> TestResult {
    let alpha = node("alpha")?;
    assert_eq!(
        GraphEdge::new(
            alpha.node_id().clone(),
            alpha.node_id().clone(),
            GraphRelationKind::Calls,
            GraphConfidence::Proven,
            Vec::new(),
            limits(),
        )
        .err()
        .ok_or("expected self-edge failure")?
        .code(),
        GraphErrorCode::SelfEdgeInvalid
    );

    let missing = node("missing")?;
    let edge = GraphEdge::new(
        alpha.node_id().clone(),
        missing.node_id().clone(),
        GraphRelationKind::Calls,
        GraphConfidence::Proven,
        Vec::new(),
        limits(),
    )?;
    assert_eq!(
        GraphSnapshot::build(
            universe()?,
            generation()?,
            limits(),
            vec![alpha.clone()],
            vec![edge],
            Vec::new(),
        )
        .err()
        .ok_or("expected missing endpoint")?
        .code(),
        GraphErrorCode::EndpointMissing
    );

    assert_eq!(
        GraphSnapshot::build(
            universe()?,
            generation()?,
            limits(),
            vec![alpha.clone(), alpha],
            Vec::new(),
            Vec::new(),
        )
        .err()
        .ok_or("expected duplicate node")?
        .code(),
        GraphErrorCode::NodeDuplicate
    );

    let foreign = GraphNode::new(
        universe()?,
        GraphGenerationId::new("project-generation:fixture:2")?,
        "function",
        "foreign",
        Vec::new(),
        limits(),
    )?;
    assert_eq!(
        GraphSnapshot::build(
            universe()?,
            generation()?,
            limits(),
            vec![foreign],
            Vec::new(),
            Vec::new(),
        )
        .err()
        .ok_or("expected generation mismatch")?
        .code(),
        GraphErrorCode::GenerationMismatch
    );
    Ok(())
}

#[test]
fn persistence_revalidates_snapshot_identity_and_publication_universe() -> TestResult {
    let mut store = Store::open_in_memory(StoreConfiguration::new(
        "graph-store-test",
        StoreLimits::default(),
    )?)?;
    let snapshot = snapshot(false)?;
    let key = GraphPublicationKey::new(universe()?, "default")?;
    let mut facade = PersistentGraphStore::new(&mut store);
    let stored = facade.publish_current(key.clone(), &snapshot, CatalogExpectation::Absent)?;
    let current = facade.read_current(&key)?.ok_or("missing current graph")?;
    assert_eq!(current.object_id(), stored.object_id());
    assert_eq!(current.snapshot(), &snapshot);
    assert!(facade.validate_integrity(100)?.complete());

    let conflicting = facade
        .publish_current(key.clone(), &snapshot, CatalogExpectation::Absent)
        .err()
        .ok_or("expected publication conflict")?;
    assert_eq!(conflicting.code(), GraphErrorCode::StoreFailure);

    let wrong_key = GraphPublicationKey::new(GraphUniverseId::new("project:other")?, "default")?;
    assert_eq!(
        facade
            .publish_current(wrong_key, &snapshot, CatalogExpectation::Absent)
            .err()
            .ok_or("expected universe mismatch")?
            .code(),
        GraphErrorCode::UniverseMismatch
    );
    Ok(())
}

#[test]
fn tampered_snapshot_and_wrong_store_kind_fail_closed() -> TestResult {
    let snapshot = snapshot(false)?;
    let mut value = serde_json::to_value(&snapshot)?;
    value["snapshot_id"] = json!(format!("graph-snapshot:sha256:{}", "0".repeat(64)));
    let tampered: GraphSnapshot = serde_json::from_value(value)?;
    assert_eq!(
        tampered
            .validate()
            .err()
            .ok_or("expected snapshot identity failure")?
            .code(),
        GraphErrorCode::SnapshotIdentityMismatch
    );

    let mut store = Store::open_in_memory(StoreConfiguration::new(
        "graph-kind-test",
        StoreLimits::default(),
    )?)?;
    let pending = PendingObject::from_json(
        "fixture.not-graph",
        1,
        &json!({"value":1}),
        StoreLimits::default(),
    )?;
    let object_id = pending.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(pending)?;
    store.commit(batch)?;
    let facade = PersistentGraphStore::new(&mut store);
    assert_eq!(
        facade
            .read_exact(&object_id)
            .err()
            .ok_or("expected object kind mismatch")?
            .code(),
        GraphErrorCode::ArtifactKindMismatch
    );
    Ok(())
}
