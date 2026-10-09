//! Real immutable-owner publication and explanation regressions for W12.
use std::{collections::BTreeMap, error::Error, sync::atomic::AtomicBool};
use wow_core::{EvidenceId, GenerationContextId, StableHandleId};
use wow_graph::*;
type ResultOf<T = ()> = Result<T, Box<dyn Error>>;
fn scope() -> ResultOf<GraphAssertionRecordScope> {
    Ok(GraphAssertionRecordScope {
        universe: GraphUniverseId::new("project:record-fixture")?,
        generation: GraphGenerationId::new("input-generation:record-fixture")?,
        source_context_id: GenerationContextId::derive(&"record-fixture")?,
    })
}
fn initial() -> ResultOf<GraphPartitionSnapshot> {
    let registry = GraphRegistryBundle::build(
        "registry:record-fixture",
        "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "function",
            vec!["project".into()],
            vec!["symbol".into()],
            vec![
                GraphConfidence::Proven,
                GraphConfidence::Derived,
                GraphConfidence::Possible,
                GraphConfidence::Candidate,
            ],
        )?],
        vec![GraphRelationKindDefinition::new(
            "calls",
            GraphRelationKind::Calls,
            vec!["function".into()],
            vec!["function".into()],
            vec![
                GraphConfidence::Proven,
                GraphConfidence::Derived,
                GraphConfidence::Possible,
                GraphConfidence::Candidate,
            ],
        )?],
    )?;
    let scope = scope()?;
    Ok(GraphPartitionSnapshot::new(
        registry,
        GraphSnapshot::build(
            scope.universe,
            scope.generation,
            GraphLimits::default(),
            Vec::new(),
            Vec::new(),
            coverage()?,
        )?,
        scope.source_context_id,
        &AtomicBool::new(false),
    )?)
}
fn coverage() -> ResultOf<Vec<GraphCoverageRecord>> {
    Ok(vec![GraphCoverageRecord::new(
        GraphRelationKind::Calls,
        GraphCoverageState::Complete,
        false,
        Vec::new(),
        GraphLimits::default(),
    )?])
}
fn entity(id: &str, key: &str, confidence: GraphConfidence) -> ResultOf<GraphEntityProposal> {
    Ok(GraphEntityProposal::new(
        id,
        "function",
        BTreeMap::from([("symbol".into(), GraphProposalValue::Identifier(key.into()))]),
        confidence,
        vec![StableHandleId::derive(&(id, "handle"))?],
        vec![EvidenceId::derive(&(id, "evidence"))?],
        Vec::new(),
    )?)
}
fn batch(
    owner: &GraphPartitionSnapshot,
    partition: &str,
    entities: Vec<GraphEntityProposal>,
) -> ResultOf<GraphProposalBatch> {
    let scope = scope()?;
    Ok(GraphProposalBatch::build(
        owner.registry().bundle_id(),
        owner.registry().registry_digest(),
        scope.universe,
        scope.generation,
        scope.source_context_id,
        partition,
        entities,
        Vec::new(),
    )?)
}
fn publish(
    owner: &GraphPartitionSnapshot,
    batch: GraphProposalBatch,
) -> ResultOf<GraphPartitionSnapshot> {
    Ok(owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: owner
                    .partition(batch.producer_partition_id())
                    .map(|partition| partition.partition_digest().into()),
                producer_version: "1.0.0".into(),
                batch,
                coverage: coverage()?,
            },
            &AtomicBool::new(false),
        )?
        .candidate()
        .clone())
}
fn local(id: &str) -> GraphLocalAssertion {
    GraphLocalAssertion {
        kind: GraphAssertionKind::Entity,
        proposal_id: id.into(),
    }
}
fn reference(id: &str) -> GraphAssertionRef {
    GraphAssertionRef::Local {
        assertion: local(id),
    }
}
fn external(
    owner: &GraphPartitionSnapshot,
    partition: &str,
    id: &str,
) -> ResultOf<GraphAssertionRef> {
    Ok(GraphAssertionRef::Producer {
        partition_id: partition.into(),
        batch_id: owner
            .partition(partition)
            .ok_or("missing producer")?
            .batch()
            .batch_id()
            .into(),
        assertion: local(id),
    })
}
fn derive(output: &str, inputs: Vec<GraphAssertionRef>) -> GraphDerivationRecord {
    GraphDerivationRecord {
        output: local(output),
        rule_id: "fixture.derive".into(),
        rule_version: 1,
        inputs,
        rebuttals: Vec::new(),
        missing: Vec::new(),
    }
}
fn with_records(
    batch: GraphProposalBatch,
    derivations: Vec<GraphDerivationRecord>,
    conflicts: Vec<GraphConflictRecord>,
) -> ResultOf<GraphProposalBatch> {
    Ok(batch.with_assertion_records(GraphAssertionRecords::build(
        scope()?,
        derivations,
        conflicts,
    )?)?)
}
fn node(owner: &GraphPartitionSnapshot, partition: &str, proposal: &str) -> ResultOf<GraphNodeId> {
    let accepted = owner
        .partition(partition)
        .ok_or("missing partition")?
        .report()
        .accepted_entities()
        .iter()
        .find(|accepted| accepted.proposal_id() == proposal)
        .ok_or("missing accepted node")?;
    Ok(owner
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.owner_key() == accepted.node().owner_key())
        .ok_or("missing materialized node")?
        .node_id()
        .clone())
}

#[test]
fn exact_cross_producer_chain_explains_and_stale_replacement_rejects() -> ResultOf {
    let stop = AtomicBool::new(false);
    let base = initial()?;
    let first = publish(
        &base,
        batch(
            &base,
            "source",
            vec![entity("leaf", "leaf", GraphConfidence::Proven)?],
        )?,
    )?;
    let next = batch(
        &first,
        "derived",
        vec![entity("output", "output", GraphConfidence::Derived)?],
    )?;
    let current = publish(
        &first,
        with_records(
            next,
            vec![derive("output", vec![external(&first, "source", "leaf")?])],
            Vec::new(),
        )?,
    )?;
    let subject = GraphExplainSubject::Entity(node(&current, "derived", "output")?);
    let full = GraphExplainQuery::new(
        current.snapshot().snapshot_id().clone(),
        subject.clone(),
        GraphExplainLimits::default(),
    )?
    .execute(&current, &stop)?;
    assert!(full.derivation_complete());
    assert_eq!(full.derivations().len(), 1);
    assert_eq!(full.assertion_supports().len(), 1);
    assert!(!full.absence_authoritative());
    let limits = GraphExplainLimits {
        max_derivation_depth: 0,
        ..GraphExplainLimits::default()
    };
    let short = GraphExplainQuery::new(current.snapshot().snapshot_id().clone(), subject, limits)?
        .execute(&current, &stop)?;
    assert!(!short.derivation_complete());
    assert!(
        short
            .truncations()
            .contains(&GraphExplanationTruncation::DerivationDepth)
    );
    let changed = batch(
        &current,
        "source",
        vec![entity("replacement-leaf", "leaf", GraphConfidence::Proven)?],
    )?;
    let failure = current.prepare_replacement(
        GraphPartitionReplacement {
            expected_snapshot_id: current.snapshot().snapshot_id().clone(),
            expected_partition_digest: current
                .partition("source")
                .map(|partition| partition.partition_digest().into()),
            producer_version: "1.0.1".into(),
            batch: changed,
            coverage: coverage()?,
        },
        &stop,
    );
    assert!(matches!(failure,Err(error) if error.code()==GraphErrorCode::PartitionStale));
    current.validate(&stop)?;
    Ok(())
}

#[test]
fn publication_rejects_cycles_and_confidence_promotion() -> ResultOf {
    let base = initial()?;
    for possible in [false, true] {
        let input = entity(
            "a",
            "a",
            if possible {
                GraphConfidence::Possible
            } else {
                GraphConfidence::Derived
            },
        )?;
        let output = entity("b", "b", GraphConfidence::Derived)?;
        let records = if possible {
            vec![derive("b", vec![reference("a")])]
        } else {
            vec![
                derive("a", vec![reference("b")]),
                derive("b", vec![reference("a")]),
            ]
        };
        let candidate = with_records(
            batch(&base, "invalid", vec![input, output])?,
            records,
            Vec::new(),
        )?;
        let outcome = base.prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: base.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: "1.0.0".into(),
                batch: candidate,
                coverage: coverage()?,
            },
            &AtomicBool::new(false),
        );
        assert!(matches!(outcome,Err(error) if error.code()==GraphErrorCode::PartitionInvalid));
    }
    Ok(())
}

#[test]
fn conflicts_keep_both_sides_and_downgrade_incident_relation_coverage() -> ResultOf {
    let base = initial()?;
    let one = batch(
        &base,
        "one",
        vec![
            entity("first", "shared", GraphConfidence::Proven)?,
            entity("target", "target", GraphConfidence::Proven)?,
        ],
    )?;
    let edge = GraphRelationProposal::new(
        "call",
        "calls",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed("first".into()),
            target: GraphProposalEndpoint::Proposed("target".into()),
            confidence: GraphConfidence::Proven,
            source_handle_ids: vec![StableHandleId::derive(&"call")?],
            evidence_ids: vec![EvidenceId::derive(&"call")?],
            coverage_ids: Vec::new(),
        },
    )?;
    let one = GraphProposalBatch::build(
        one.registry_bundle_id(),
        one.registry_digest(),
        one.universe().clone(),
        one.generation().clone(),
        one.source_context_id(),
        one.producer_partition_id(),
        one.entity_proposals().to_vec(),
        vec![edge],
    )?;
    let one = publish(&base, one)?;
    let two = publish(
        &one,
        batch(
            &one,
            "two",
            vec![entity("second", "shared", GraphConfidence::Possible)?],
        )?,
    )?;
    let first = external(&two, "one", "first")?;
    let second = external(&two, "two", "second")?;
    let conflict = GraphConflictRecord {
        kind: GraphConflictKind::EvidenceOrSourceHandleConflict,
        subject: first.clone(),
        assertions: vec![second.clone(), first.clone()],
        affected_capabilities: vec!["fixture.callables".into()],
        affected_axes: vec!["call".into()],
    };
    let current = publish(
        &two,
        with_records(
            batch(&two, "conflict", Vec::new())?,
            Vec::new(),
            vec![conflict],
        )?,
    )?;
    let query = GraphExplainQuery::new(
        current.snapshot().snapshot_id().clone(),
        GraphExplainSubject::Entity(node(&current, "one", "first")?),
        GraphExplainLimits::default(),
    )?;
    let explanation = query.execute(&current, &AtomicBool::new(false))?;
    assert_eq!(explanation.conflicts().len(), 1);
    assert_eq!(explanation.supports().len(), 2);
    assert!(explanation.supports().iter().any(|support|matches!(support,
        GraphAssertionSupport::ProducerEntity{proposal,..} if proposal.confidence()==GraphConfidence::Possible)));
    assert!(
        explanation.conflicts()[0]
            .record
            .assertions
            .contains(&first)
    );
    assert!(
        explanation.conflicts()[0]
            .record
            .assertions
            .contains(&second)
    );
    let coverage = current
        .snapshot()
        .coverage()
        .iter()
        .find(|record| record.relation() == GraphRelationKind::Calls)
        .ok_or("missing coverage")?;
    assert_eq!(coverage.state(), GraphCoverageState::Partial);
    assert!(!coverage.negative_authority());
    let restored: GraphPartitionSnapshot = serde_json::from_slice(&serde_json::to_vec(&current)?)?;
    restored.validate(&AtomicBool::new(false))?;
    assert_eq!(current, restored);
    stored_roundtrip(&current)?;
    stored_roundtrip(&one)?;
    Ok(())
}

fn stored_roundtrip(owner: &GraphPartitionSnapshot) -> ResultOf {
    use wow_store::project::{ProjectStore, PublicationRequest, ReadSelector, RecordCatalog};
    let stop = AtomicBool::new(false);
    let root = std::env::temp_dir().join(format!(
        "wow-graph-record-roundtrip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(
        GraphPartitionSnapshot::STORAGE_SCHEMAS,
        &[GraphPartitionSnapshot::STORAGE_CHECK],
    )?;
    // create() rejects a pre-existing root rather than adopting foreign data.
    let mut store = ProjectStore::create(&root, owner.snapshot().universe().as_str(), catalog)?;
    let request = PublicationRequest::new(
        store.epoch(),
        wow_store::OperationId::new("fixture:graph-record-roundtrip")?,
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
    let restored = GraphPartitionSnapshot::read_stored(&read, &stop)?;
    assert_eq!(owner, &restored);
    drop(read);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn canonical_records_preserve_reordered_batch_identity_and_legacy_encoding() -> ResultOf {
    let base = initial()?;
    let bare = batch(
        &base,
        "canonical",
        vec![
            entity("leaf", "leaf", GraphConfidence::Proven)?,
            entity("other", "other", GraphConfidence::Proven)?,
            entity("output", "output", GraphConfidence::Derived)?,
        ],
    )?;
    let old = serde_json::to_value(&bare)?;
    assert_eq!(old["schema"], GRAPH_PROPOSAL_BATCH_SCHEMA);
    assert!(old.get("assertion_records").is_none());
    let a = with_records(
        bare.clone(),
        vec![derive(
            "output",
            vec![reference("leaf"), reference("other")],
        )],
        Vec::new(),
    )?;
    let b = with_records(
        bare,
        vec![derive(
            "output",
            vec![reference("other"), reference("leaf")],
        )],
        Vec::new(),
    )?;
    assert_eq!(a, b);
    assert_eq!(publish(&base, a)?, publish(&base, b)?);
    Ok(())
}
