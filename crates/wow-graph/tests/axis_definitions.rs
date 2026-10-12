//! Distinct definitions can deliberately normalize to one materialized edge.
use std::{collections::BTreeMap, error::Error, sync::atomic::AtomicBool};
use wow_core::{EvidenceId, GenerationContextId, StableHandleId};
use wow_graph::*;
type ResultOf<T = ()> = Result<T, Box<dyn Error>>;

#[test]
fn sealed_neighbor_batch_uses_actual_aggregate_allowance() -> ResultOf {
    let stop = AtomicBool::new(false);
    let owner = fixture()?;
    let read = owner.open_read_view(&stop)?;
    let root = read
        .published_entity("producer:axis", "root", &stop)?
        .node_id();
    let child = read
        .published_entity("producer:axis", "child", &stop)?
        .node_id();
    let mut queries = Vec::new();
    for (node, direction, relation, max_edges, confidence) in [
        (
            root,
            GraphDirection::Both,
            GraphRelationKind::Owns,
            10,
            GraphPathConfidence::ProvenAndDerived,
        ),
        (
            root,
            GraphDirection::Outgoing,
            GraphRelationKind::Owns,
            1,
            GraphPathConfidence::ProvenAndDerived,
        ),
        (
            child,
            GraphDirection::Incoming,
            GraphRelationKind::Owns,
            10,
            GraphPathConfidence::ProvenAndDerived,
        ),
        (
            root,
            GraphDirection::Incoming,
            GraphRelationKind::Owns,
            10,
            GraphPathConfidence::ProvenAndDerived,
        ),
        (
            root,
            GraphDirection::Both,
            GraphRelationKind::Calls,
            10,
            GraphPathConfidence::ProvenAndDerived,
        ),
        (
            root,
            GraphDirection::Both,
            GraphRelationKind::Owns,
            10,
            GraphPathConfidence::Proven,
        ),
    ] {
        queries.push(
            GraphNeighborReadQuery::new(
                owner.snapshot().snapshot_id().clone(),
                GraphNeighborQuery::new(node.clone(), direction, vec![relation], max_edges)?,
                GraphNeighborReadLimits::default(),
            )?
            .with_confidence(confidence),
        );
    }
    let expected = queries
        .iter()
        .map(|query| {
            Ok(wow_core::canonical_json_bytes(
                &read.neighbors(query, &stop)?,
            )?)
        })
        .collect::<ResultOf<Vec<_>>>()?;
    let limits = GraphNeighborBatchLimits {
        max_logical_scanned_edges: (queries.len() * owner.snapshot().edges().len()) as u64,
        max_output_bytes: expected.iter().map(|bytes| bytes.len() as u64).sum(),
    };
    let views = read.neighbors_batch(&queries, limits, &stop)?;
    assert_eq!(views.len(), expected.len());
    for (view, bytes) in views.iter().zip(&expected) {
        assert_eq!(&wow_core::canonical_json_bytes(view)?, bytes);
        assert_eq!(
            view.scanned_edges() as usize,
            owner.snapshot().edges().len()
        );
    }
    assert_eq!(views[1].truncations(), &[GraphNeighborTruncation::Edges]);
    assert!(views[3..].iter().all(|view| view.edges().is_empty()));
    for refused in [
        GraphNeighborBatchLimits {
            max_output_bytes: limits.max_output_bytes - 1,
            ..limits
        },
        GraphNeighborBatchLimits {
            max_logical_scanned_edges: limits.max_logical_scanned_edges - 1,
            ..limits
        },
    ] {
        assert_eq!(
            read.neighbors_batch(&queries, refused, &stop)
                .err()
                .ok_or("batch accepted insufficient work or bytes")?
                .code(),
            GraphErrorCode::BudgetExceeded
        );
    }
    assert_eq!(
        read.neighbors_batch(&queries, limits, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled batch returned success")?
            .code(),
        GraphErrorCode::Cancelled
    );
    // Self-loops are rejected by the admitted E2-A owner, not a usable fixture.
    assert_eq!(
        GraphEdge::new(
            root.clone(),
            root.clone(),
            GraphRelationKind::Owns,
            GraphConfidence::Derived,
            Vec::new(),
            GraphLimits::default()
        )
        .err()
        .ok_or("self-loop admitted by the E2-A owner")?
        .code(),
        GraphErrorCode::SelfEdgeInvalid
    );
    Ok(())
}

fn registry() -> ResultOf<GraphRegistryBundle> {
    Ok(GraphRegistryBundle::build(
        "registry:axis-definitions",
        "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "item",
            vec!["project".into()],
            vec!["key".into()],
            vec![GraphConfidence::Derived],
        )?],
        ["owns_a", "owns_b", "maps"]
            .into_iter()
            .map(|id| {
                GraphRelationKindDefinition::new(
                    id,
                    GraphRelationKind::Owns,
                    vec!["item".into()],
                    vec!["item".into()],
                    vec![GraphConfidence::Derived],
                )
            })
            .collect::<GraphResult<Vec<_>>>()?,
    )?)
}

fn fixture() -> ResultOf<GraphPartitionSnapshot> {
    let stop = AtomicBool::new(false);
    let registry = registry()?;
    let foundation = GraphSnapshot::build(
        GraphUniverseId::new("project:axis-definitions")?,
        GraphGenerationId::new("input:axis-definitions")?,
        GraphLimits::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )?;
    let context = GenerationContextId::derive(&"axis-definitions")?;
    let entities = ["root", "child", "mapped"]
        .into_iter()
        .map(|id| -> ResultOf<_> {
            Ok(GraphEntityProposal::new(
                id,
                "item",
                BTreeMap::from([("key".into(), GraphProposalValue::Identifier(id.into()))]),
                GraphConfidence::Derived,
                vec![StableHandleId::derive(&id)?],
                vec![EvidenceId::derive(&id)?],
                Vec::new(),
            )?)
        })
        .collect::<ResultOf<Vec<_>>>()?;
    let relations = [
        ("a", "owns_a", "child"),
        ("b", "owns_b", "child"),
        ("m", "maps", "mapped"),
    ]
    .into_iter()
    .map(|(proposal, definition, target)| -> ResultOf<_> {
        Ok(GraphRelationProposal::new(
            proposal,
            definition,
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed("root".into()),
                target: GraphProposalEndpoint::Proposed(target.into()),
                confidence: GraphConfidence::Derived,
                source_handle_ids: vec![StableHandleId::derive(&"shared-source")?],
                evidence_ids: vec![EvidenceId::derive(&"shared-evidence")?],
                coverage_ids: Vec::new(),
            },
        )?)
    })
    .collect::<ResultOf<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        foundation.universe().clone(),
        foundation.generation().clone(),
        context,
        "producer:axis",
        entities,
        relations,
    )?;
    let owner = GraphPartitionSnapshot::new(registry, foundation, context, &stop)?;
    Ok(owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: "1.0.0".into(),
                batch,
                coverage: Vec::new(),
            },
            &stop,
        )?
        .candidate()
        .clone())
}

fn query(owner: &GraphPartitionSnapshot, profile: &GraphAxisProfile) -> ResultOf<GraphAxisQuery> {
    let root = &owner
        .partition("producer:axis")
        .ok_or("partition")?
        .report()
        .accepted_entities()
        .iter()
        .find(|item| item.proposal_id() == "root")
        .ok_or("root receipt")?
        .node();
    let published = owner
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.owner_key() == root.owner_key())
        .ok_or("published root")?;
    Ok(GraphAxisQuery::new(
        owner.snapshot().snapshot_id().clone(),
        profile,
        vec![published.node_id().clone()],
        GraphAxisTraversal::Children,
        GraphSubgraphLimits {
            max_depth: 1,
            ..GraphSubgraphLimits::default()
        },
    )?)
}

#[test]
fn colliding_definitions_remain_exact_and_excluded_edges_do_not_leak() -> ResultOf {
    let stop = AtomicBool::new(false);
    let owner = fixture()?;
    let accepted = owner
        .partition("producer:axis")
        .ok_or("partition")?
        .report()
        .accepted_relations();
    assert_eq!(accepted[0].edge().edge_id(), accepted[1].edge().edge_id());
    assert_eq!(owner.snapshot().edges().len(), 2);
    let profile = GraphAxisProfile::bind_definitions(
        owner.registry(),
        GraphAxis::Ownership,
        &["owns_b", "owns_a"],
    )?;
    let reordered = GraphAxisProfile::bind_definitions(
        owner.registry(),
        GraphAxis::Ownership,
        &["owns_a", "owns_b"],
    )?;
    assert_eq!(profile, reordered);
    let restored: GraphAxisProfile = serde_json::from_slice(&serde_json::to_vec(&profile)?)?;
    restored.validate(owner.registry())?;
    let query = query(&owner, &profile)?;
    let result = query.execute(&owner, &restored, &stop)?;
    assert_eq!(result.projection().nodes().len(), 2);
    assert_eq!(result.projection().edges().len(), 1);
    assert_eq!(
        result
            .edge_definitions()
            .iter()
            .map(GraphAxisEdgeDefinition::relation_id)
            .collect::<Vec<_>>(),
        vec!["owns_a", "owns_b"]
    );
    assert!(
        result
            .edge_definitions()
            .iter()
            .all(|row| row.edge_id() == result.projection().edges()[0].edge_id())
    );
    assert!(!result.projection().absence_authoritative());
    let child = result
        .projection()
        .nodes()
        .iter()
        .find(|node| node.depth() == 1)
        .ok_or("child projection")?;
    let parents = GraphAxisQuery::new(
        owner.snapshot().snapshot_id().clone(),
        &profile,
        vec![child.node().node_id().clone()],
        GraphAxisTraversal::Parents,
        GraphSubgraphLimits {
            max_depth: 1,
            ..GraphSubgraphLimits::default()
        },
    )?
    .execute(&owner, &profile, &stop)?;
    assert_eq!(parents.projection().edges(), result.projection().edges());
    assert_eq!(parents.edge_definitions(), result.edge_definitions());
    let capped = GraphAxisQuery::new(
        owner.snapshot().snapshot_id().clone(),
        &profile,
        query.roots().to_vec(),
        GraphAxisTraversal::Children,
        GraphSubgraphLimits {
            max_depth: 1,
            max_nodes: 1,
            ..GraphSubgraphLimits::default()
        },
    )?
    .execute(&owner, &profile, &stop)?;
    assert_eq!(capped.state(), GraphQueryState::Truncated);
    assert!(capped.projection().edges().is_empty());
    assert!(capped.edge_definitions().is_empty());
    assert!(!capped.absence_authoritative());
    assert!(
        result
            .projection()
            .query()
            .execute(owner.snapshot(), &stop)
            .is_err()
    );
    let substituted =
        GraphAxisProfile::bind_definitions(owner.registry(), GraphAxis::Ownership, &["maps"])?;
    assert_eq!(
        query
            .execute(&owner, &substituted, &stop)
            .err()
            .ok_or("substitution accepted")?
            .code(),
        GraphErrorCode::AxisProfileIdentityMismatch
    );
    assert!(
        GraphAxisProfile::bind_definitions(owner.registry(), GraphAxis::Ownership, &["unknown"])
            .is_err()
    );
    assert!(
        GraphAxisProfile::bind_definitions(
            owner.registry(),
            GraphAxis::Ownership,
            &["owns_a", "owns_a"]
        )
        .is_err()
    );
    let one =
        GraphAxisProfile::bind_definitions(owner.registry(), GraphAxis::Ownership, &["owns_b"])?;
    let one_result = self::query(&owner, &one)?.execute(&owner, &one, &stop)?;
    assert_eq!(one_result.edge_definitions().len(), 1);
    assert_eq!(one_result.edge_definitions()[0].relation_id(), "owns_b");
    assert_eq!(one_result.projection().edges(), result.projection().edges());
    assert_eq!(
        query
            .execute(&owner, &profile, &AtomicBool::new(true))
            .err()
            .ok_or("cancel ignored")?
            .code(),
        GraphErrorCode::Cancelled
    );
    let mut value = serde_json::to_value(&profile)?;
    value["relations"][0]["relation_id"] = "unknown".into();
    let mutated: GraphAxisProfile = serde_json::from_value(value)?;
    assert!(mutated.validate(owner.registry()).is_err());
    Ok(())
}

#[test]
fn existing_stored_partitions_reopen_without_rederiving_another_edge_dialect() -> ResultOf {
    use wow_store::project::{ProjectStore, PublicationRequest, ReadSelector, RecordCatalog};
    let stop = AtomicBool::new(false);
    let owner = fixture()?;
    let root = std::env::temp_dir().join(format!(
        "wow-axis-reopen-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(
        GraphPartitionSnapshot::STORAGE_SCHEMAS,
        &[GraphPartitionSnapshot::STORAGE_CHECK],
    )?;
    let mut store = ProjectStore::create(&root, owner.snapshot().universe().as_str(), catalog)?;
    let request = PublicationRequest::new(
        store.epoch(),
        wow_store::OperationId::new("fixture:axis-reopen")?,
        None,
        BTreeMap::from([(
            "snapshot".into(),
            owner.snapshot().snapshot_id().as_str().into(),
        )]),
        owner.storage_records(&stop)?,
    )?;
    store.prepare(&request, &stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        &stop,
    )?;
    let restored = GraphPartitionSnapshot::read_stored_validated(&read, &stop)?;
    assert_eq!(&owner, restored.as_snapshot());
    let membership = restored.validate_storage_membership(&read, &stop)?;
    assert_eq!(
        membership,
        read.manifest().members.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        restored
            .validate_storage_membership(&read, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled storage comparison accepted")?
            .code(),
        GraphErrorCode::Cancelled
    );
    let profile = GraphAxisProfile::bind_definitions(
        owner.registry(),
        GraphAxis::Ownership,
        &["owns_a", "owns_b"],
    )?;
    let query = query(&owner, &profile)?;
    assert_eq!(
        serde_json::to_vec(&query.execute(&owner, &profile, &stop)?)?,
        serde_json::to_vec(&query.execute(restored.as_snapshot(), &profile, &stop)?)?
    );

    // The generic store admits these internally consistent canonical bytes.
    // The retained graph proof must still compare its own typed commitments,
    // rather than lending that proof to this other read's changed manifest.
    let mut foreign_records = owner.storage_records(&stop)?;
    let header_index = foreign_records
        .iter()
        .position(|record| record.key() == "graph.header")
        .ok_or("graph header missing")?;
    let mut foreign_header: serde_json::Value = foreign_records[header_index].decode()?;
    foreign_header["schema"] = "foreign-stored-header".into();
    foreign_records[header_index] = wow_store::project::PartitionRecord::new(
        "graph.header",
        "wow-graph.header.v1",
        &foreign_header,
    )?;
    let foreign = PublicationRequest::new(
        store.epoch(),
        wow_store::OperationId::new("fixture:axis-foreign-read")?,
        None,
        request.generation().bindings.clone(),
        foreign_records,
    )?;
    store.prepare(&foreign, &stop)?;
    let foreign_read = store.read(
        &ReadSelector::Exact(foreign.generation().generation_id.clone()),
        &stop,
    )?;
    assert_eq!(
        restored
            .validate_storage_membership(&foreign_read, &stop)
            .err()
            .ok_or("changed canonical graph commitment accepted")?
            .code(),
        GraphErrorCode::PartitionInvalid
    );
    // The original held read and owner retain their exact membership.
    assert_eq!(
        membership,
        restored.validate_storage_membership(&read, &stop)?
    );
    foreign_read.close()?;
    read.close()?;
    drop(store);
    // This exact newly created fixture root is owned by this test.
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn ambiguous_foundation_authority_and_incomplete_scan_are_refused() -> ResultOf {
    let stop = AtomicBool::new(false);
    let owner = fixture()?;
    let profile =
        GraphAxisProfile::bind_definitions(owner.registry(), GraphAxis::Ownership, &["owns_a"])?;
    let original = owner.input_view(&stop)?;
    let foundation = GraphPartitionSnapshot::new(
        owner.registry().clone(),
        original,
        owner.source_context_id(),
        &stop,
    )?;
    let root = foundation.snapshot().edges()[0].from().clone();
    let query = GraphAxisQuery::new(
        foundation.snapshot().snapshot_id().clone(),
        &profile,
        vec![root],
        GraphAxisTraversal::Forward,
        GraphSubgraphLimits::default(),
    )?;
    assert_eq!(
        query
            .execute(&foundation, &profile, &stop)
            .err()
            .ok_or("ambiguous foundation accepted")?
            .code(),
        GraphErrorCode::AxisProfileInvalid
    );
    let root = owner.snapshot().edges()[0].from().clone();
    let query = GraphAxisQuery::new(
        owner.snapshot().snapshot_id().clone(),
        &profile,
        vec![root],
        GraphAxisTraversal::Forward,
        GraphSubgraphLimits {
            max_scanned_edges: 4,
            ..GraphSubgraphLimits::default()
        },
    )?;
    assert_eq!(
        query
            .execute(&owner, &profile, &stop)
            .err()
            .ok_or("scan cap ignored")?
            .code(),
        GraphErrorCode::BudgetExceeded
    );
    Ok(())
}
