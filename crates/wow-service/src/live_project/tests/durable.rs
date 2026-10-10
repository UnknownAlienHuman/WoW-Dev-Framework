use super::*;

fn input(files: &[(&str, &str)]) -> TestResult<crate::LocalProjectInput> {
    let files = files
        .iter()
        .map(|(path, text)| ProjectInputFile::new(*path, *text))
        .collect::<Result<Vec<_>, _>>()?;
    let bundle = input_bundle_with_plans(files, None, None)?;
    let reference = wow_reference::ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    Ok(crate::LocalProjectInput::new(bundle, reference)?)
}

fn result(value: &LiveProjectResult) -> TestResult<serde_json::Value> {
    Ok(serde_json::from_slice(&value.canonical_bytes()?)?)
}

#[test]
fn native_library_caller_survives_full_publication_and_foreign_binding_refuses() -> TestResult {
    use wow_recognizers::source_state::{
        SOURCE_STATE_LIBRARY_PARTITION, SourceLibraryInput, recognize_source_library,
    };
    let stop = AtomicBool::new(false);
    let store_root = root("native-library-caller")?;
    let request =
        crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    operations::publish_input(
        input(&[(
            "library-call.lua",
            "function LibStub(name) return {} end\nfunction Require() return LibStub(\"FixtureLibrary-1\") end\n",
        )])?,
        &request,
        &store_root,
        &LiveProjectPublishRequest::new("fixture:native-library-caller", "absent", true, true)?,
        &stop,
    )?;
    let store = LiveProjectStore::open(&store_root)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    held.graph().validate(&stop)?;
    let (_, _, _, provenance, _) =
        wow_project::graph::build_source_graph_proposals(held.project(), &stop)?.into_parts();
    let report = provenance
        .function_call_report()
        .ok_or("missing native call report")?;
    let call = report
        .calls()
        .iter()
        .find(|call| call.resolved_callable_key() == Some("LibStub"))
        .ok_or("native LibStub call not captured")?;
    let caller = provenance
        .functions()
        .iter()
        .find(|function| function.function_id == call.caller_function_id())
        .ok_or("caller proposal not captured")?;
    let lookup = held.graph().producer_lookup(&stop)?;
    let source = held
        .graph()
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or("source partition missing")?;
    let address = wow_graph::GraphAssertionRef::Producer {
        partition_id: source.partition_id().into(),
        batch_id: source.batch().batch_id().into(),
        assertion: wow_graph::GraphLocalAssertion {
            kind: wow_graph::GraphAssertionKind::Entity,
            proposal_id: caller.proposal_id.clone().into(),
        },
    };
    let expected_caller = lookup.entity(lookup.scope(), &address, &stop)?;
    let library = held
        .graph()
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("library partition missing")?;
    assert_eq!(library.report().accepted_entities().len(), 1);
    assert_eq!(library.report().accepted_relations().len(), 1);
    assert_eq!(
        library.report().accepted_relations()[0].edge().from(),
        expected_caller.accepted().node().node_id()
    );

    // Swapping two real accepted callable proposals must fail semantic admission.
    let mut functions = provenance
        .functions()
        .iter()
        .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let other = provenance
        .functions()
        .iter()
        .find(|function| function.function_id != caller.function_id)
        .ok_or("second callable missing")?;
    functions.insert(caller.function_id.as_str(), other.proposal_id.as_str());
    functions.insert(other.function_id.as_str(), caller.proposal_id.as_str());
    let error = recognize_source_library(
        SourceLibraryInput {
            owner: held.graph(),
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: functions,
            call_support: provenance
                .call_sites()
                .iter()
                .map(|site| {
                    (
                        site.call_id.as_str(),
                        (site.source_handle_id, site.evidence_id),
                    )
                })
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        &stop,
    )
    .err()
    .ok_or("permuted callable proposals accepted")?;
    assert_eq!(
        error.code(),
        wow_recognizers::RecognizerErrorCode::AdapterFactMismatch
    );
    drop(lookup);
    drop(held);
    drop(store);
    std::fs::remove_dir_all(store_root)?;
    Ok(())
}

#[test]
fn full_graph_update_matches_cold_and_preserves_original_base_and_readers() -> TestResult {
    let stop = AtomicBool::new(false);
    let store_root = root("durable-update")?;
    let cold_root = root("durable-cold")?;
    let graph = crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    let before = [
        (
            "main.lua",
            "function Target() return 1 end\nfunction Caller() return Target() end\n",
        ),
        (
            "removed.lua",
            "function Gone() return 3 end\nfunction LostCaller() return Gone() end\n",
        ),
        ("stable.lua", "function Stay() return External() end\n"),
    ];
    let after = [
        (
            "main.lua",
            "function Target() return 4 end\nfunction Caller() return Added() end\n",
        ),
        ("added.lua", "function Added() return External() end\n"),
        before[2],
    ];
    operations::publish_input(
        input(&before)?,
        &graph,
        &store_root,
        &LiveProjectPublishRequest::new("fixture:durable-base", "absent", true, true)?,
        &stop,
    )?;
    let mut store = LiveProjectStore::open(&store_root)?;
    let base = store.current()?.ok_or("missing base")?;
    let old = store.read(&ReadSelector::Current, &stop)?;
    let old_graph = old.graph().clone();
    assert!(
        old_graph
            .snapshot()
            .edges()
            .iter()
            .filter(|edge| edge.relation() == wow_graph::GraphRelationKind::Calls)
            .count()
            >= 2,
        "baseline must exercise actual Main call relations"
    );
    let old_project = old.project().snapshot_id().to_owned();
    let request = LiveProjectUpdateRequest::new(
        "fixture:durable-update",
        base.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    let receipt = store.update(input(&after)?, &graph, &request, &stop)?;
    assert_eq!(receipt.exit_code(), 2);
    assert_eq!(result(&receipt)?["status"], "activated");
    let active = store.current()?.ok_or("missing update")?;
    assert_ne!(active.record_id, base.record_id);
    let updated = store.read(&ReadSelector::Current, &stop)?;
    assert!(
        updated
            .graph()
            .snapshot()
            .edges()
            .iter()
            .any(|edge| edge.relation() == wow_graph::GraphRelationKind::Calls),
        "updated Main call relation must remain"
    );
    assert!(updated.project().file_by_path("removed.lua")?.is_none());
    assert!(updated.project().file_by_path("added.lua")?.is_some());
    assert!(old.project().file_by_path("removed.lua")?.is_some());
    assert_eq!(old.project().snapshot_id(), old_project);
    assert_eq!(old.graph(), &old_graph);
    assert_eq!(old.current_at_acquisition(), Some(&base));
    assert!(
        updated.graph().partitions().len() > 10,
        "full producer chain required"
    );
    assert!(
        updated
            .graph()
            .snapshot()
            .nodes()
            .iter()
            .all(|node| old_graph.snapshot().node(node.node_id()).is_none()),
        "old generation nodes survived"
    );
    assert!(
        updated
            .graph()
            .snapshot()
            .edges()
            .iter()
            .all(|edge| old_graph.snapshot().edge(edge.edge_id()).is_none()),
        "old generation edges survived"
    );

    // Build every producer independently over the final state, with a fresh analyzer.
    operations::publish_input(
        input(&after)?,
        &graph,
        &cold_root,
        &LiveProjectPublishRequest::new("fixture:durable-cold", "absent", true, true)?,
        &stop,
    )?;
    let cold_store = LiveProjectStore::open(&cold_root)?;
    let cold = cold_store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        updated.project().snapshot_id(),
        cold.project().snapshot_id()
    );
    assert_eq!(
        updated.project().analyzer_snapshot_id(),
        cold.project().analyzer_snapshot_id()
    );
    assert_eq!(updated.graph(), cold.graph());
    assert_eq!(updated.publication_set_id(), cold.publication_set_id());
    drop(cold);
    drop(cold_store);

    let no_change = LiveProjectUpdateRequest::new(
        "fixture:durable-no-change",
        active.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    assert_eq!(
        result(&store.update(input(&after)?, &graph, &no_change, &stop)?)?["status"],
        "no_change"
    );
    assert!(store.reconcile("fixture:durable-no-change")?.is_none());
    let wrong_project =
        crate::graph::GraphBuildRequest::new("another-project".into(), "current".into())?;
    assert_eq!(
        store
            .update(input(&after)?, &wrong_project, &no_change, &stop)
            .err()
            .ok_or("NoChange accepted foreign project")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    let wrong_generation = crate::graph::GraphBuildRequest::new(
        "fixture-live-pair".into(),
        old.project().project_generation().to_string(),
    )?;
    assert_eq!(
        store
            .update(input(&after)?, &wrong_generation, &no_change, &stop)
            .err()
            .ok_or("NoChange accepted stale generation")?
            .code(),
        ServiceErrorCode::ExactGenerationUnavailable
    );
    let stale = LiveProjectUpdateRequest::new(
        "fixture:durable-stale",
        base.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    assert_eq!(
        store
            .update(input(&after)?, &graph, &stale, &stop)
            .err()
            .ok_or("stale base accepted")?
            .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    assert_eq!(
        store
            .update(input(&after)?, &graph, &no_change, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled update accepted")?
            .code(),
        ServiceErrorCode::Cancelled
    );
    assert_eq!(store.current()?, Some(active.clone()));

    let third = [
        after[0],
        ("added.lua", "function Added() return External() + 2 end\n"),
        after[2],
    ];
    let third_request = LiveProjectUpdateRequest::new(
        "fixture:durable-third",
        active.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    store.update(input(&third)?, &graph, &third_request, &stop)?;
    let third_current = store.current()?;
    // Same ID/base/target returns the original receipt, even after a later current.
    assert_eq!(
        result(&store.update(input(&after)?, &graph, &request, &stop)?)?,
        result(&receipt)?
    );
    assert_eq!(store.current()?, third_current);
    assert_eq!(
        store
            .update(input(&third)?, &graph, &request, &stop)
            .err()
            .ok_or("operation ID target substitution accepted")?
            .code(),
        ServiceErrorCode::OperationConflict
    );
    assert_eq!(store.current()?, third_current);
    drop(updated);
    drop(old);
    drop(store);
    let reopened = LiveProjectStore::open(&store_root)?;
    let exact = reopened.read(&ReadSelector::Exact(base.generation_id.clone()), &stop)?;
    assert_eq!(exact.project().snapshot_id(), old_project);
    assert_eq!(exact.graph(), &old_graph);
    let historical = reopened.read(&ReadSelector::Publication(base.record_id), &stop)?;
    assert_eq!(historical.graph(), exact.graph());
    assert_eq!(historical.current_at_acquisition(), third_current.as_ref());
    drop(historical);
    drop(exact);
    drop(reopened);
    std::fs::remove_dir_all(store_root)?;
    std::fs::remove_dir_all(cold_root)?;
    Ok(())
}
