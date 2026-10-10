//! Exact producer addresses resolve real native receipts in the input generation.
use std::{collections::BTreeMap, error::Error, sync::atomic::AtomicBool};
use wow_core::{EvidenceId, GenerationContextId, StableHandleId};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphEntityKindDefinition, GraphEntityProposal, GraphErrorCode, GraphGenerationId, GraphLimits,
    GraphLocalAssertion, GraphPartitionReplacement, GraphPartitionSnapshot, GraphProposalBatch,
    GraphProposalEndpoint, GraphProposalValue, GraphRegistryBundle, GraphRelationKind,
    GraphRelationKindDefinition, GraphRelationProposal, GraphRelationProposalInput, GraphSnapshot,
    GraphUniverseId,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn initial() -> TestResult<GraphPartitionSnapshot> {
    let registry = GraphRegistryBundle::build(
        "registry:producer-lookup",
        "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "function",
            vec!["project".into()],
            vec!["symbol".into()],
            vec![GraphConfidence::Derived],
        )?],
        vec![GraphRelationKindDefinition::new(
            "calls",
            GraphRelationKind::Calls,
            vec!["function".into()],
            vec!["function".into()],
            vec![GraphConfidence::Derived],
        )?],
    )?;
    Ok(GraphPartitionSnapshot::new(
        registry,
        GraphSnapshot::build(
            GraphUniverseId::new("project:producer-lookup")?,
            GraphGenerationId::new("input-generation:producer-lookup")?,
            GraphLimits::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )?,
        GenerationContextId::derive(&"producer-lookup")?,
        &AtomicBool::new(false),
    )?)
}

fn entity(id: &str) -> TestResult<GraphEntityProposal> {
    Ok(GraphEntityProposal::new(
        id,
        "function",
        BTreeMap::from([("symbol".into(), GraphProposalValue::Identifier(id.into()))]),
        GraphConfidence::Derived,
        vec![StableHandleId::derive(&(id, "source"))?],
        vec![EvidenceId::derive(&(id, "evidence"))?],
        Vec::new(),
    )?)
}

fn publish(
    owner: &GraphPartitionSnapshot,
    partition: &str,
    entities: Vec<GraphEntityProposal>,
    relations: Vec<GraphRelationProposal>,
) -> TestResult<GraphPartitionSnapshot> {
    let batch = GraphProposalBatch::build(
        owner.registry().bundle_id(),
        owner.registry().registry_digest(),
        owner.foundation().universe().clone(),
        owner.foundation().generation().clone(),
        owner.source_context_id(),
        partition,
        entities,
        relations,
    )?;
    Ok(owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: "1.0.0".into(),
                batch,
                coverage: Vec::new(),
            },
            &AtomicBool::new(false),
        )?
        .candidate()
        .clone())
}

fn reference(
    owner: &GraphPartitionSnapshot,
    partition: &str,
    kind: GraphAssertionKind,
    proposal: &str,
) -> TestResult<GraphAssertionRef> {
    Ok(GraphAssertionRef::Producer {
        partition_id: partition.into(),
        batch_id: owner
            .partition(partition)
            .ok_or("missing producer")?
            .batch()
            .batch_id()
            .into(),
        assertion: GraphLocalAssertion {
            kind,
            proposal_id: proposal.into(),
        },
    })
}

#[test]
#[allow(clippy::too_many_lines)]
fn exact_lookup_keeps_native_input_ids_and_refuses_foreign_addresses() -> TestResult {
    let stop = AtomicBool::new(false);
    let first = publish(
        &initial()?,
        "inventory",
        vec![entity("caller")?],
        Vec::new(),
    )?;
    let caller_id = first
        .partition("inventory")
        .ok_or("missing inventory")?
        .report()
        .accepted_entities()[0]
        .node()
        .node_id()
        .clone();
    let relation = GraphRelationProposal::new(
        "call",
        "calls",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Existing(caller_id.clone()),
            target: GraphProposalEndpoint::Proposed("callee".into()),
            confidence: GraphConfidence::Derived,
            source_handle_ids: vec![StableHandleId::derive(&"call-source")?],
            evidence_ids: vec![EvidenceId::derive(&"call-evidence")?],
            coverage_ids: Vec::new(),
        },
    )?;
    let owner = publish(&first, "analyzer", vec![entity("callee")?], vec![relation])?;
    let lookup = owner.producer_lookup(&stop)?;
    let scope = GraphAssertionRecordScope {
        universe: owner.foundation().universe().clone(),
        generation: owner.foundation().generation().clone(),
        source_context_id: owner.source_context_id(),
    };
    assert_eq!(lookup.scope(), &scope);
    assert_eq!(lookup.input_view(), &owner.input_view(&stop)?);
    let caller = reference(&owner, "inventory", GraphAssertionKind::Entity, "caller")?;
    let callee = reference(&owner, "analyzer", GraphAssertionKind::Entity, "callee")?;
    for address in [&caller, &callee] {
        let resolved = lookup.entity(&scope, address, &stop)?;
        assert_eq!(&resolved.reference(), address);
        assert_eq!(
            resolved
                .partition()
                .batch()
                .entity_proposal(resolved.proposal().proposal_id()),
            Some(resolved.proposal())
        );
        let input_node = resolved.accepted().node();
        assert!(lookup.input_view().node(input_node.node_id()).is_some());
        assert_eq!(input_node.generation(), &scope.generation);
        let published = owner
            .snapshot()
            .nodes()
            .iter()
            .find(|node| node.owner_key() == input_node.owner_key())
            .ok_or("missing published node")?;
        assert_ne!(published.node_id(), input_node.node_id());
    }
    assert_eq!(
        lookup
            .entity(&scope, &caller, &stop)?
            .accepted()
            .node()
            .node_id(),
        &caller_id
    );
    let call = reference(&owner, "analyzer", GraphAssertionKind::Relation, "call")?;
    let resolved = lookup.relation(&scope, &call, &stop)?;
    assert_eq!(resolved.reference(), call);
    assert_eq!(
        resolved.partition().batch().relation_proposal("call"),
        Some(resolved.proposal())
    );
    let input_edge = resolved.accepted().edge();
    assert!(lookup.input_view().edge(input_edge.edge_id()).is_some());
    assert_eq!(input_edge.from(), &caller_id);
    assert_ne!(owner.snapshot().edges()[0].edge_id(), input_edge.edge_id());
    let mut wrong_scopes = vec![scope.clone(); 3];
    wrong_scopes[0].universe = GraphUniverseId::new("project:foreign")?;
    wrong_scopes[1].generation = GraphGenerationId::new("input-generation:foreign")?;
    wrong_scopes[2].source_context_id = GenerationContextId::derive(&"foreign")?;
    for wrong in wrong_scopes {
        assert_eq!(
            lookup
                .entity(&wrong, &caller, &stop)
                .err()
                .ok_or("expected producer lookup rejection")?
                .code(),
            GraphErrorCode::PartitionInvalid
        );
        assert_eq!(
            lookup
                .relation(&wrong, &call, &stop)
                .err()
                .ok_or("expected producer lookup rejection")?
                .code(),
            GraphErrorCode::PartitionInvalid
        );
    }
    let local = GraphAssertionRef::Local {
        assertion: GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: "caller".into(),
        },
    };
    let mut missing_producer = caller.clone();
    if let GraphAssertionRef::Producer { partition_id, .. } = &mut missing_producer {
        *partition_id = "absent".into();
    }
    let missing_proposal = reference(&owner, "inventory", GraphAssertionKind::Entity, "absent")?;
    for invalid in [&local, &missing_producer, &missing_proposal, &call] {
        assert_eq!(
            lookup
                .entity(&scope, invalid, &stop)
                .err()
                .ok_or("expected producer lookup rejection")?
                .code(),
            GraphErrorCode::PartitionInvalid
        );
    }
    assert_eq!(
        lookup
            .relation(&scope, &callee, &stop)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::PartitionInvalid
    );
    let missing_relation = reference(&owner, "analyzer", GraphAssertionKind::Relation, "absent")?;
    assert_eq!(
        lookup
            .relation(&scope, &missing_relation, &stop)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::PartitionInvalid
    );
    let mut stale = caller.clone();
    if let GraphAssertionRef::Producer { batch_id, .. } = &mut stale {
        *batch_id = owner
            .partition("analyzer")
            .ok_or("missing analyzer")?
            .batch()
            .batch_id()
            .into();
    }
    assert_eq!(
        lookup
            .entity(&scope, &stale, &stop)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::PartitionStale
    );
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        owner
            .producer_lookup(&cancelled)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::Cancelled
    );
    assert_eq!(
        lookup
            .entity(&scope, &caller, &cancelled)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::Cancelled
    );
    assert_eq!(
        lookup
            .relation(&scope, &call, &cancelled)
            .err()
            .ok_or("expected producer lookup rejection")?
            .code(),
        GraphErrorCode::Cancelled
    );
    Ok(())
}
