use super::*;

#[test]
fn transaction_order_single_wrapper_and_sequential_results_agree() -> TestResult {
    let stop = AtomicBool::new(false);
    let base = initial()?;
    let a = request(&base, batch("producer:a", Some("A"))?)?;
    let b = request(&base, batch("producer:b", Some("B"))?)?;
    let forward = base.prepare_replacements(vec![a.clone(), b.clone()], &stop)?;
    let reverse = base.prepare_replacements(vec![b, a.clone()], &stop)?;
    assert_eq!(forward, reverse);
    assert_eq!(
        base.prepare_replacement(a.clone(), &stop)?,
        base.prepare_replacements(vec![a], &stop)?
    );
    let sequential = replace(
        &replace(&base, batch("producer:a", Some("A"))?)?,
        batch("producer:b", Some("B"))?,
    )?;
    assert_eq!(forward.candidate(), &sequential);
    assert_eq!(serde_json::to_vec(forward.candidate())?, serde_json::to_vec(&sequential)?);
    assert_eq!(forward.changes().len(), 2);
    for (change, id) in forward.changes().iter().zip(["producer:a", "producer:b"]) {
        assert_eq!(change.partition_id(), id);
        assert_eq!(change.previous_partition_digest(), None);
        assert_eq!(change.target_partition_digest(), sequential.partition(id).ok_or("partition")?.partition_digest());
    }
    let mut session = GraphPartitionSession::new(base, &stop)?;
    let before = session.view();
    assert!(before.partitions().is_empty());
    let after = session.publish(forward, &stop)?;
    assert!(!Arc::ptr_eq(&before, &after));
    assert!(before.partitions().is_empty());
    assert_eq!(after.partitions().len(), 2);
    let restored: GraphPartitionSnapshot = serde_json::from_slice(&serde_json::to_vec(after.as_ref())?)?;
    restored.validate(&stop)?;
    Ok(())
}

#[test]
fn transaction_removes_dependent_owners_together_and_preserves_survivors() -> TestResult {
    let stop = AtomicBool::new(false);
    let a = replace(&initial()?, batch("producer:a", Some("A"))?)?;
    let node = contributed_node(&a)?;
    let root = a.foundation().nodes()[0].node_id();
    let b = replace(&a, edge_batch("producer:b", root, node.node_id())?)?;
    let base = replace(&b, batch("producer:c", Some("C"))?)?;
    let mut session = GraphPartitionSession::new(base, &stop)?;
    let old = session.view();
    let saved_c = old.partition("producer:c").ok_or("surviving partition")?.clone();
    let mut remove_a = request(&old, batch("producer:a", None)?)?;
    let mut remove_b = request(&old, batch("producer:b", None)?)?;
    remove_a.coverage.clear();
    remove_b.coverage.clear();
    assert!(old.prepare_replacement(remove_a.clone(), &stop).is_err());
    let plan = old.prepare_replacements(vec![remove_b, remove_a], &stop)?;
    for change in plan.changes() {
        assert_eq!(change.previous_partition_digest(), Some(old.partition(change.partition_id()).ok_or("old partition")?.partition_digest()));
    }
    let after = session.publish(plan, &stop)?;
    assert_eq!(after.partition("producer:c"), Some(&saved_c));
    assert!(after.input_view(&stop)?.node(node.node_id()).is_none());
    assert!(after.snapshot().edges().is_empty());
    assert_eq!(old.snapshot().edges().len(), 1);
    assert!(old.input_view(&stop)?.node(node.node_id()).is_some());
    for id in ["producer:a", "producer:b"] {
        let tombstone = after.partition(id).ok_or("tombstone")?;
        assert!(tombstone.report().accepted_entities().is_empty());
        assert!(tombstone.report().accepted_relations().is_empty());
        assert_eq!(tombstone.coverage()[0].state(), GraphCoverageState::NotEvaluated);
    }
    assert!(!after.snapshot().coverage()[0].negative_authority());
    Ok(())
}

#[test]
fn transaction_checks_all_base_and_partition_guards_before_publication() -> TestResult {
    let stop = AtomicBool::new(false);
    let base = initial()?;
    let full = base.prepare_replacements(vec![
        request(&base, batch("producer:a", Some("A"))?)?,
        request(&base, batch("producer:b", Some("B"))?)?,
    ], &stop)?.candidate().clone();
    let session = GraphPartitionSession::new(full, &stop)?;
    let before = session.view();
    for guard in 0..3 {
        let valid = request(&before, batch("producer:a", Some("NewA"))?)?;
        let mut stale = request(&before, batch("producer:b", Some("NewB"))?)?;
        match guard {
            0 => stale.expected_snapshot_id = base.snapshot().snapshot_id().clone(),
            1 => stale.expected_partition_digest = None,
            _ => stale.expected_partition_digest = Some("stale-partition".into()),
        }
        assert_eq!(before.prepare_replacements(vec![valid, stale], &stop)
            .err().ok_or("stale transaction accepted")?.code(), GraphErrorCode::PartitionStale);
        assert!(Arc::ptr_eq(&before, &session.view()));
    }
    let duplicate = request(&before, batch("producer:a", Some("A"))?)?;
    assert_eq!(before.prepare_replacements(vec![duplicate.clone(), duplicate], &stop)
        .err().ok_or("duplicate accepted")?.code(), GraphErrorCode::PartitionInvalid);
    Ok(())
}

#[test]
fn rejected_peer_and_removed_peer_endpoint_abort_whole_transaction() -> TestResult {
    let stop = AtomicBool::new(false);
    let with_a = replace(&initial()?, batch("producer:a", Some("A"))?)?;
    let node = contributed_node(&with_a)?;
    let root = with_a.foundation().nodes()[0].node_id().clone();
    let session = GraphPartitionSession::new(with_a, &stop)?;
    let before = session.view();
    // Replacing a peer cannot lend its OLD endpoint, even when reasserting the
    // same semantic node. Independent batches use the fixed surviving view.
    let a = request(&before, batch("producer:a", Some("A"))?)?;
    let b = request(&before, edge_batch("producer:b", &root, node.node_id())?)?;
    for requests in [vec![a.clone(), b.clone()], vec![b, a]] {
        assert_eq!(before.prepare_replacements(requests, &stop)
            .err().ok_or("peer endpoint accepted")?.code(), GraphErrorCode::PartitionRejected);
        assert!(Arc::ptr_eq(&before, &session.view()));
    }
    let mut elevated = request(&before, batch("producer:b", Some("B"))?)?;
    elevated.coverage = vec![GraphCoverageRecord::new(
        GraphRelationKind::Calls, GraphCoverageState::Complete, true, Vec::new(), GraphLimits::default(),
    )?];
    assert_eq!(before.prepare_replacements(vec![
        request(&before, batch("producer:a", Some("NewA"))?)?, elevated,
    ], &stop).err().ok_or("elevated peer accepted")?.code(), GraphErrorCode::PartitionInvalid);
    assert!(Arc::ptr_eq(&before, &session.view()));
    Ok(())
}

#[test]
fn transaction_cancellation_stale_plan_and_no_change_preserve_arc_identity() -> TestResult {
    let stop = AtomicBool::new(false);
    let mut session = GraphPartitionSession::new(initial()?, &stop)?;
    let before = session.view();
    let requests = vec![
        request(&before, batch("producer:a", Some("A"))?)?,
        request(&before, batch("producer:b", Some("B"))?)?,
    ];
    assert_eq!(before.prepare_replacements(requests.clone(), &AtomicBool::new(true))
        .err().ok_or("cancelled transaction accepted")?.code(), GraphErrorCode::Cancelled);
    let plan = before.prepare_replacements(requests, &stop)?;
    let competing = plan.clone();
    assert_eq!(session.publish(plan.clone(), &AtomicBool::new(true))
        .err().ok_or("cancelled publication accepted")?.code(), GraphErrorCode::Cancelled);
    assert!(Arc::ptr_eq(&before, &session.view()));
    let active = session.publish(plan, &stop)?;
    assert_eq!(session.publish(competing, &stop)
        .err().ok_or("stale plan accepted")?.code(), GraphErrorCode::PartitionStale);
    let repeat = active.prepare_replacements(vec![
        request(&active, batch("producer:b", Some("B"))?)?,
        request(&active, batch("producer:a", Some("A"))?)?,
    ], &stop)?;
    assert!(repeat.changes().iter().all(|change| change.previous_partition_digest() == Some(change.target_partition_digest())));
    assert!(Arc::ptr_eq(&active, &session.publish(repeat, &stop)?));
    Ok(())
}

#[test]
fn transaction_cardinality_and_combined_assertion_limits_are_enforced() -> TestResult {
    let stop = AtomicBool::new(false);
    let base = initial()?;
    assert_eq!(base.prepare_replacements(Vec::new(), &stop)
        .err().ok_or("empty transaction accepted")?.code(), GraphErrorCode::PartitionInvalid);
    let request_a = request(&base, batch("producer:a", None)?)?;
    assert_eq!(base.prepare_replacements(vec![request_a; 65], &stop)
        .err().ok_or("oversized transaction accepted")?.code(), GraphErrorCode::BudgetExceeded);
    let mut requests = Vec::new();
    for index in 0..64 {
        requests.push(request(&base, batch(&format!("producer:{index:02}"), None)?)?);
    }
    let full = base.prepare_replacements(requests, &stop)?.candidate().clone();
    assert_eq!(full.partitions().len(), 64);
    assert_eq!(full.prepare_replacements(vec![request(&full, batch("producer:new", None)?)?], &stop)
        .err().ok_or("65th partition accepted")?.code(), GraphErrorCode::BudgetExceeded);
    full.prepare_replacements(vec![request(&full, batch("producer:00", None)?)?], &stop)?;
    let small = GraphSnapshot::build(
        universe()?, generation()?, GraphLimits::new(2, 10, 10, 10, 10)?,
        base.foundation().nodes().to_vec(), Vec::new(), coverage()?,
    )?;
    let limited = GraphPartitionSnapshot::new(registry()?, small, context()?, &stop)?;
    let a = request(&limited, batch("producer:a", Some("A"))?)?;
    let b = request(&limited, batch("producer:b", Some("B"))?)?;
    limited.prepare_replacement(a.clone(), &stop)?;
    limited.prepare_replacement(b.clone(), &stop)?;
    assert_eq!(limited.prepare_replacements(vec![a, b], &stop)
        .err().ok_or("combined budget exceeded")?.code(), GraphErrorCode::BudgetExceeded);
    Ok(())
}
