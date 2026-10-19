from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one block, found {count}")
    file.write_text(text.replace(old, new))


replace(
    "crates/wow-service/src/graph/build.rs",
    "mod recognizer_handoff;\n",
    "",
)
replace(
    "crates/wow-service/src/graph/build.rs",
    '''    let snapshot = recognizer_handoff::apply_ready_handoffs(
        snapshot,
        addresses.source().context(),
        &state_core_recognition,
        stop,
    )
    .map_err(|e| stage_error("state-core-generic-handoff", e))?;
''',
    "",
)

state_tests = Path("crates/wow-service/src/graph/build/state_core_tests.rs")
text = state_tests.read_text()
marker = '''fn rejects_changed_access_handles(
'''
if text.count(marker) != 1:
    raise SystemExit("unexpected state_core_tests helper marker")
service_test = '''#[test]
fn service_graph_build_keeps_direct_state_core_partitions() -> Result<(), Box<dyn Error>> {
    let stop = std::sync::atomic::AtomicBool::new(false);
    let directory = ProjectInputDirectory::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/state-core-pipeline"),
    )?;
    let (files, load_plan) = directory
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("StateCore.toc"),
            &state_core_profile()?,
            Some(&TocLoadContext {
                game_types: std::collections::BTreeMap::from([("retail".into(), true)]),
                family: None,
                game: None,
                text_locale: None,
                location: None,
                environment: None,
            }),
            &stop,
        )?
        .into_parts();
    let identity = backend()?;
    let binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        identity.compatibility_report_sha256().to_owned(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        identity.clone(),
    )?;
    let library = LuaWorkspaceSnapshot::build(
        identity,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/state_core.lua",
            "---@meta _\\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let reference_generation = ReferenceGenerationId::derive(&"fixture-state-core-reference")?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-state-core-pipeline")?,
        ProjectKind::Fixture,
        state_core_profile()?,
        reference_generation,
        binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:state-core-pipeline")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:state-core-pipeline",
    )?)
    .logical_root("fixtures/state-core-pipeline")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .load_plan(&load_plan)?
    .build()?;
    let bundle = ProjectInputBundle::closed(config, files, vec![library])?;
    let reference = wow_reference::ReferenceView::new(
        reference_generation.to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    let input = LocalProjectInput::new_with_load_plan(bundle, reference, Some(load_plan))?;
    let request = GraphBuildRequest::new(
        "fixture-state-core-pipeline".into(),
        "current".into(),
    )?;
    let result = execute_graph_build(input, &request, &stop)?;
    let snapshot_bytes = result
        .snapshot_bytes()?
        .ok_or("Service did not return the final graph snapshot")?;
    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(&snapshot_bytes)?;
    snapshot.validate(&stop)?;
    assert!(snapshot.partition("wow-recognizers.state-read").is_some());
    assert!(snapshot.partition("wow-recognizers.state-write").is_some());
    assert!(!snapshot.snapshot().edges().is_empty());
    Ok(())
}

'''
state_tests.write_text(text.replace(marker, service_test + marker))

replace(
    "docs/IMPLEMENTATION.md",
    '''**Start #195 with `wow-recognizers/src/source_state_core/project.rs::execute`
and Service's `graph/build/recognizer_handoff.rs`.** Typed state accesses and
root/caller/target bindings already exist. The adapter creates generic facts,
fixed packs and matcher output; Service reconstructs the four read/write ×
root/path packs and plans to project those results into the graph again.

Map the admitted typed facts directly to the consumed relationships. Keep
root versus literal-path targets, account/character scope, source locations,
ambiguity and incomplete-input results. Remove `evaluations`, pack/plan digests
and the repeated handoff; they are not a required result DTO. Reuse the positive
and unresolved-path behavior in `graph/build/state_core_tests.rs`. Strip matcher
assertions from the handoff test while retaining its useful `execute_graph_build`
check in the existing suite; direct adapter tests alone do not exercise Service.
''',
    '''**Continue #195 in `wow-recognizers/src/source_state_core/{adapt,project}.rs`.**
Service's second pack compilation and graph reprojection are removed: the
recognizer-owned batch is published once and the Service suite checks that final
path. The recognizer still converts admitted typed accesses into generic facts,
fixed packs and matcher output before converting them back into graph proposals.

Map the admitted typed facts directly to the consumed relationships. Keep root
versus literal-path targets, account/character scope, source locations,
ambiguity, incomplete-input results and the existing `execute_graph_build`
check. Remove `evaluations`, pack/plan digests and generic matcher DTOs as their
last consumers disappear; they are not a required user result. Do not replace
them with another adapter or proof layer.
''',
)
replace(
    "docs/REFACTOR_QUEUE.md",
    '''| Next graph cut | [#195 — typed recognizers](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195) | Start with typed State Core and remove its repeated pack handoff; continue through live adapters. |
''',
    '''| Next graph cut | [#195 — typed recognizers](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195) | Service reprojection is removed; replace State Core's remaining typed-fact → generic pack/matcher → graph roundtrip, then trace live adapters. |
''',
)
