use super::*;
use std::error::Error;
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
    SchemaVersionEntry, SourceKind, SourceLogicalSnapshot, ToolVersion,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_graph::{GraphPartitionReplacement, GraphSnapshot};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectInputFile, ProjectKind,
};
type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn input_bundle(source: &str) -> TestResult<ProjectInputBundle> {
    // Synthetic test identities authorize no product compatibility/acceptance.
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:project-pair-regression-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:live-pair-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:fixture-e0".parse()?,
        ToolVersion::parse("1.0.0")?,
    )])
    .fixture_scope("native live pair regression")
    .build()?;
    let binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-live-pair")?,
        ProjectKind::Fixture,
        profile,
        ReferenceGenerationId::derive(&"live-pair-reference")?,
        binding,
    )
    .workspace_id(wow_project::ProjectWorkspaceId::new(
        "workspace:main:project-pair-regression",
    )?)
    .source_origin_id(wow_project::ProjectSourceOriginId::new(
        "project-origin:project-pair-regression",
    )?)
    .logical_root("fixtures/project-pair/main")
    .capability_policy(ProjectCapabilityPolicy::degraded_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/Core.lua",
            "---@meta\n---@return number\nfunction External() end\n",
        )],
        LuaWorkspaceLimits::new(8, 4096, 16384, 65536)?,
    )?;
    Ok(ProjectInputBundle::closed(
        config,
        vec![ProjectInputFile::new("main.lua", source)?],
        vec![library],
    )?)
}
fn owners(source: &str) -> TestResult<(ProjectPublisher, GraphPartitionSnapshot)> {
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let view = publisher
        .publish_initial(input_bundle(source)?)?
        .open_view();
    let stop = AtomicBool::new(false);
    let (registry, batch, coverage, _, limits) =
        wow_project::graph::build_source_graph_proposals(&view, &stop)?.into_parts();
    let foundation = GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner =
        GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), &stop)?;
    let graph = owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch,
                coverage,
            },
            &stop,
        )?
        .candidate()
        .clone();
    Ok((publisher, graph))
}
fn root(name: &str) -> TestResult<std::path::PathBuf> {
    Ok(std::env::temp_dir().join(format!(
        "wow-live-pair-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    )))
}

#[test]
fn native_service_composition_publishes_once_and_rejects_stale_parent() -> TestResult {
    let stop = AtomicBool::new(false);
    let root = root("composition")?;
    let input = || -> TestResult<crate::LocalProjectInput> {
        let bundle = input_bundle("local value = External()\nreturn value\n")?;
        let reference = wow_reference::ReferenceView::new(
            bundle.configuration().reference_generation().to_string(),
            Vec::new(),
            Vec::new(),
        )?;
        Ok(crate::LocalProjectInput::new(bundle, reference)?)
    };
    let graph_request =
        crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    let request = LiveProjectPublishRequest::new("fixture:composed-pair", "absent", true, true)?;
    let receipt = operations::publish_input(input()?, &graph_request, &root, &request, &stop)?;
    assert_eq!(receipt.exit_code(), 2);
    let projected: serde_json::Value = serde_json::from_slice(&receipt.canonical_bytes()?)?;
    assert_eq!(projected["status"], "activated");
    assert_eq!(projected["operation"]["state"], "activated");
    let read = LiveProjectStore::open(&root)?;
    let acquired = read.read(&ReadSelector::Current, &stop)?;
    assert!(
        acquired.graph().partitions().len() > 1,
        "actual recognizer chain must be retained"
    );
    for producer in [
        wow_recognizers::source_bridge::W2_PARTITION,
        wow_recognizers::source_scripts::W5_HOOK_PARTITION,
        wow_recognizers::source_state::SOURCE_STATE_LIBRARY_PARTITION,
    ] {
        let partition = acquired
            .graph()
            .partition(producer)
            .ok_or("missing producer")?;
        assert!(
            partition
                .coverage()
                .iter()
                .all(|coverage| !coverage.negative_authority())
        );
    }
    let current = read.current()?;
    drop(acquired);
    drop(read);
    let stale = LiveProjectPublishRequest::new("fixture:composed-stale", "absent", false, true)?;
    assert_eq!(
        operations::publish_input(input()?, &graph_request, &root, &stale, &stop)
            .err()
            .ok_or("stale expected-current accepted")?
            .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    assert_eq!(LiveProjectStore::open(&root)?.current()?, current);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn live_pair_reopens_and_advancing_current_preserves_leased_readers() -> TestResult {
    let stop = AtomicBool::new(false);
    let (first, first_graph) = owners("local value = External()\nreturn value\n")?;
    let (second, second_graph) = owners("local value = External() + 1\nreturn value\n")?;
    let root = root("reopen")?;
    let mut store = LiveProjectStore::create(&root, first_graph.snapshot().universe().as_str())?;
    let one = store.publish(&first, &first_graph, "fixture:live-pair-one", None, &stop)?;
    assert_eq!(one.state, PublicationState::Activated);
    let old = store.read(&ReadSelector::Current, &stop)?;
    let old_id = old.project().snapshot_id().to_owned();
    assert_eq!(
        old_id,
        first
            .current_snapshot()
            .ok_or("missing first")?
            .snapshot_id()
    );
    let first_current = store.current()?.ok_or("missing current")?;
    store.publish(
        &second,
        &second_graph,
        "fixture:live-pair-two",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let new = store.read(&ReadSelector::Current, &stop)?;
    assert_ne!(old.project().snapshot_id(), new.project().snapshot_id());
    assert_ne!(
        old.graph().snapshot().snapshot_id(),
        new.graph().snapshot().snapshot_id()
    );
    assert_eq!(old.project().snapshot_id(), old_id);
    assert_eq!(old.graph(), &first_graph);
    assert_eq!(new.graph(), &second_graph);
    assert!(
        new.project().file_by_path("library/Core.lua")?.is_none(),
        "replayed Library sources remain outside the Main inventory"
    );
    let active = store.current()?.ok_or("missing current")?;
    assert_eq!(
        store
            .publish(
                &first,
                &first_graph,
                "fixture:live-pair-stale",
                Some(first_current.record_id),
                &stop
            )
            .err()
            .ok_or("stale activation accepted")?
            .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    assert_eq!(store.current()?, Some(active.clone()));
    let new_id = new.project().snapshot_id().to_owned();
    let set_id = new.publication_set_id().to_owned();
    let exact_id = old.store_generation_id().clone();
    drop(old);
    drop(new);
    drop(store);
    let reopened = LiveProjectStore::open(&root)?;
    let new = reopened.read(&ReadSelector::Current, &stop)?;
    let old = reopened.read(&ReadSelector::Exact(exact_id), &stop)?;
    assert_eq!(new.project().snapshot_id(), new_id);
    assert_eq!(new.publication_set_id(), set_id);
    assert_eq!(old.project().snapshot_id(), old_id);
    assert_eq!(new.current_at_acquisition(), Some(&active));
    assert!(
        new.project()
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .is_some()
    );
    drop(old);
    drop(new);
    drop(reopened);
    let projected: serde_json::Value =
        serde_json::from_slice(&read_live_project(&root, "current", &stop)?.canonical_bytes()?)?;
    assert_eq!(projected["pair"]["project_snapshot_id"], new_id);
    assert_eq!(projected["pair"]["publication_set_id"], set_id);
    assert_eq!(projected["pair"]["main_file_count"], 1);
    assert_eq!(projected["pair"]["library_count"], 1);
    let reconciled: serde_json::Value = serde_json::from_slice(
        &reconcile_live_project(&root, "fixture:live-pair-two", &stop)?.canonical_bytes()?,
    )?;
    assert_eq!(reconciled["status"], "observed");
    assert_eq!(
        LiveProjectStore::open(&root)?.current()?,
        Some(active.clone())
    );
    assert_eq!(
        read_live_project(&root, "not-a-generation", &stop)
            .err()
            .ok_or("invalid selector accepted")?
            .code(),
        ServiceErrorCode::InvalidRequest
    );
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn missing_or_mutated_replay_and_mixed_project_graph_never_activate() -> TestResult {
    let stop = AtomicBool::new(false);
    let (publisher, graph) = owners("return External()\n")?;
    let (other, _) = owners("return External() + 2\n")?;
    let root = root("reject")?;
    let mut store = LiveProjectStore::create(&root, graph.snapshot().universe().as_str())?;
    assert_eq!(
        store
            .publish(&other, &graph, "fixture:live-pair-mixed", None, &stop)
            .err()
            .ok_or("mixed pair accepted")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert!(store.current()?.is_none());
    assert_eq!(
        store
            .publish(
                &publisher,
                &graph,
                "fixture:live-pair-cancelled",
                None,
                &AtomicBool::new(true)
            )
            .err()
            .ok_or("cancelled publication accepted")?
            .code(),
        ServiceErrorCode::Cancelled
    );
    assert!(store.reconcile("fixture:live-pair-cancelled")?.is_none());
    for mutation in [false, true] {
        let bundle = ProjectPublicationBundle::build(&publisher, &graph, &stop)?;
        let (mut records, bindings) = bundle.into_parts();
        let index = records
            .iter()
            .position(|r| r.key() == "live.project.replay")
            .ok_or("missing replay")?;
        if mutation {
            let mut value: serde_json::Value = records[index].decode()?;
            value["files"][0]["text"] = serde_json::Value::String("return 42\n".into());
            records[index] = wow_store::project::PartitionRecord::new(
                "live.project.replay",
                "wow-project.live-replay.v1",
                &value,
            )?;
        } else {
            records.remove(index);
        }
        let request = PublicationRequest::new(
            store.store.epoch(),
            OperationId::new(if mutation {
                "fixture:mutated"
            } else {
                "fixture:missing"
            })?,
            None,
            bindings,
            records,
        )?;
        store.store.prepare(&request, &stop)?;
        assert_eq!(
            store
                .read(
                    &ReadSelector::Exact(request.generation().generation_id.clone()),
                    &stop
                )
                .err()
                .ok_or("incoherent pair acquired")?
                .code(),
            ServiceErrorCode::IdentityMismatch
        );
        assert!(store.current()?.is_none());
    }
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
