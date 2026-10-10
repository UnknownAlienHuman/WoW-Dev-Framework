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
#[path = "current_recovery_tests.rs"]
mod current_recovery;
mod durable;
mod gc;
mod library_modes;
#[path = "migration_tests.rs"]
mod migration;
#[path = "quarantine_tests.rs"]
mod quarantine;
mod retention;

fn input_bundle(source: &str) -> TestResult<ProjectInputBundle> {
    input_bundle_with_plan(vec![ProjectInputFile::new("main.lua", source)?], None)
}
fn input_bundle_with_plan(
    files: Vec<ProjectInputFile>,
    plan: Option<&wow_project::load::ProjectLoadPlan>,
) -> TestResult<ProjectInputBundle> {
    input_bundle_with_plans(files, plan, None)
}
fn input_bundle_with_plans(
    files: Vec<ProjectInputFile>,
    plan: Option<&wow_project::load::ProjectLoadPlan>,
    packages: Option<(
        &wow_project::load::ProjectPackageLoadPlan,
        &wow_project::load::ProjectPackageMainPlan,
    )>,
) -> TestResult<ProjectInputBundle> {
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
    let builder = ProjectConfigurationBuilder::new(
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
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?);
    let builder = match plan {
        Some(plan) => builder.load_plan(plan)?,
        None => builder,
    };
    let config = match packages {
        Some((load, main)) => builder.package_load_plan(load, main)?,
        None => builder,
    }
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
    Ok(ProjectInputBundle::closed(config, files, vec![library])?)
}
fn owners(source: &str) -> TestResult<(ProjectPublisher, GraphPartitionSnapshot)> {
    owners_from_bundle(input_bundle(source)?)
}
fn owners_from_bundle(
    bundle: ProjectInputBundle,
) -> TestResult<(ProjectPublisher, GraphPartitionSnapshot)> {
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let view = publisher.publish_initial(bundle)?.open_view();
    Ok((publisher, graph_from_view(&view)?))
}
fn graph_from_view(view: &ProjectView) -> TestResult<GraphPartitionSnapshot> {
    let stop = AtomicBool::new(false);
    let (registry, batch, coverage, _, limits) =
        wow_project::graph::build_source_graph_proposals(view, &stop)?.into_parts();
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
    Ok(graph)
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
fn package_pair_replays_complete_scopes_and_never_admits_unreachable_main() -> TestResult {
    use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
    use wow_project::load::{
        ProjectPackageInput, ProjectPackageReachability, ProjectPackageVariantInput,
    };
    use wow_project::replay::ProjectReplay;
    let stop = AtomicBool::new(false);
    let root = root("packages")?;
    let source_root = root.join("input");
    for name in ["App", "Dep", "Other"] {
        std::fs::create_dir_all(source_root.join(name))?;
    }
    for (path, text) in [
        (
            "App/App.toc",
            "## Interface: 120100\n## Dependencies: Dep\n## OptionalDeps: MissingOptional\nmain.lua\nframes.xml\n",
        ),
        ("App/Inactive.toc", "## Interface: 120100\nignored.lua\n"),
        ("App/main.lua", "local value = External()\nreturn value\n"),
        (
            "App/frames.xml",
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Frame name=\"PackageFrame\"><Scripts><OnLoad>local package_body = true</OnLoad></Scripts></Frame></Ui>",
        ),
        (
            "Dep/Dep.toc",
            "## Interface: 120100\n## LoadOnDemand: 1\nmain.lua\n",
        ),
        ("Dep/main.lua", "local prerequisite = true\n"),
        ("Other/Other.toc", "## Interface: 120100\nmain.lua\n"),
        ("Other/main.lua", "local unreachable_package = 42\n"),
    ] {
        std::fs::write(source_root.join(path), text)?;
    }
    let declarations = ["App", "Dep", "Other"].map(|name| {
        let mut variants = vec![ProjectPackageVariantInput::new(
            ProjectDiskFile::new(format!("{name}.toc")),
            true,
        )];
        if name == "App" {
            variants.push(ProjectPackageVariantInput::new(
                ProjectDiskFile::new("Inactive.toc"),
                false,
            ));
        }
        ProjectPackageInput::new(name, name, name == "App", variants)
    });
    let profile = input_bundle("return true")?
        .configuration()
        .selected_profile()
        .clone();
    let (files, load, main) = ProjectInputDirectory::open(&source_root)?
        .read_package_project_with_context(&declarations, &profile, None, &stop)?
        .into_namespaced_main()?
        .into_parts();
    assert_eq!(files.len(), 2);
    assert_eq!(
        load.packages()
            .iter()
            .find(|package| package.package == "Other")
            .ok_or("Other missing")?
            .reachability,
        ProjectPackageReachability::Unreachable
    );
    assert_eq!(
        load.package_plan("Other")
            .ok_or("Other plan missing")?
            .sources()
            .len(),
        2
    );
    let bundle = input_bundle_with_plans(files.clone(), None, Some((&load, &main)))?;
    let reference = wow_reference::ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    let service_input = crate::LocalProjectInput::new_with_package_plans(
        bundle,
        reference,
        load.clone(),
        main.clone(),
    )?;
    let (publisher, graph) =
        owners_from_bundle(input_bundle_with_plans(files, None, Some((&load, &main)))?)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let archive = serde_json::to_value(&replay)?;
    assert_eq!(archive["schema"], "wow-project/native-project-replay/4");
    assert_eq!(archive["generation_schema_version"], 2);
    assert_eq!(archive["files"].as_array().ok_or("files missing")?.len(), 0);
    assert_eq!(
        archive["packages"]["packages"]
            .as_array()
            .ok_or("packages missing")?
            .len(),
        3
    );
    std::fs::remove_dir_all(&source_root)?;
    let restored = replay.hydrate(&stop)?;
    assert_eq!(restored.configuration().package_load_plan(), Some(&load));
    assert_eq!(restored.configuration().package_main_plan(), Some(&main));
    assert_eq!(
        restored.snapshot_id(),
        publisher
            .current_snapshot()
            .ok_or("owner missing")?
            .snapshot_id()
    );
    assert!(restored.file_by_path("packages/App/main.lua")?.is_some());
    assert!(restored.file_by_path("packages/Dep/main.lua")?.is_some());
    assert!(restored.file_by_path("packages/Other/main.lua")?.is_none());
    assert!(
        load.package_plan("App")
            .ok_or("App missing")?
            .sources()
            .iter()
            .all(|source| source.path != "ignored.lua")
    );
    for mutation in 0..5 {
        let mut changed = archive.clone();
        match mutation {
            0 => {
                changed["packages"]["packages"][2]["sources"][1]["text"] =
                    "local fabricated = true".into()
            }
            1 => {
                changed["packages"]["packages"][2]["sources"]
                    .as_array_mut()
                    .ok_or("sources missing")?
                    .pop();
            }
            2 => {
                changed["packages"]["packages"][0]["sources"][1]["text"] =
                    "## Interface: 120100\nchanged.lua\n".into()
            }
            3 => changed["packages"]["packages"][2]["selected_root"] = true.into(),
            _ => {
                changed["files"] = serde_json::json!([{"path":"foreign.lua", "text":"return true"}])
            }
        }
        let substituted: ProjectReplay = serde_json::from_value(changed)?;
        assert!(
            substituted.hydrate(&stop).is_err(),
            "package archive mutation {mutation} was accepted"
        );
    }
    let request = LiveProjectPublishRequest::new("fixture:package-composed", "absent", true, true)?;
    let graph_request =
        crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    let store_root = root.join("store");
    let result =
        operations::publish_input(service_input, &graph_request, &store_root, &request, &stop)?;
    assert_eq!(result.exit_code(), 2);
    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        read.project().configuration().package_load_plan(),
        Some(&load)
    );
    assert_eq!(read.project().snapshot_id(), restored.snapshot_id());
    assert_eq!(read.graph().source_context_id(), graph.source_context_id());
    let generation = read.store_generation_id().clone();
    let set_id = read.publication_set_id().to_owned();
    drop(read);
    drop(store);
    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(&ReadSelector::Exact(generation), &stop)?;
    assert_eq!(
        read.project().configuration().package_main_plan(),
        Some(&main)
    );
    assert_eq!(read.publication_set_id(), set_id);
    drop(read);
    drop(store);
    let legacy_root = root.join("legacy-v2");
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create(
            &legacy_root,
            graph.snapshot().universe().as_str(),
            catalog_for(publication::STORAGE_SCHEMAS_V2)?,
        )?,
    };
    let legacy_epoch = legacy.store.epoch().clone();
    assert!(
        legacy
            .publish(&publisher, &graph, "fixture:package-to-legacy", None, &stop)
            .is_err()
    );
    assert!(legacy.current()?.is_none());
    assert_eq!(legacy.store.epoch(), &legacy_epoch);
    drop(legacy);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn standalone_toc_xml_pair_replays_without_disk_and_rejects_archive_substitution() -> TestResult {
    use std::collections::BTreeMap;
    use wow_project::{
        disk::{ProjectDiskFile, ProjectInputDirectory},
        load::{LoadIssueKind, LoadSelection, TocLoadContext},
        replay::ProjectReplay,
    };
    let stop = AtomicBool::new(false);
    let root = root("toc-xml")?;
    let source_root = root.join("input");
    std::fs::create_dir_all(source_root.join("nested"))?;
    for (path, text) in [
        (
            "Fixture.toc",
            "## Interface: 120100\nmain.lua\nframes.xml\nmissing.lua\nexcluded.lua [AllowLoadGameType classic]\n",
        ),
        (
            "main.lua",
            "function NamedHandler() return External() end\n",
        ),
        (
            "frames.xml",
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Include file=\"nested/child.xml\"/></Ui>",
        ),
        (
            "nested/child.xml",
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Script file=\"helper.lua\"/><Frame name=\"ReplayFrame\"><Scripts><OnLoad>local value = External()</OnLoad><OnShow function=\"NamedHandler\"/></Scripts></Frame></Ui>",
        ),
        ("nested/helper.lua", "local helper = true\n"),
    ] {
        std::fs::write(source_root.join(path), text)?;
    }
    let profile = input_bundle("return true")?
        .configuration()
        .selected_profile()
        .clone();
    let context = TocLoadContext {
        game_types: BTreeMap::from([("classic".into(), false)]),
        family: None,
        game: None,
        text_locale: None,
        location: None,
        environment: None,
    };
    let (files, plan) = ProjectInputDirectory::open(&source_root)?
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("Fixture.toc"),
            &profile,
            Some(&context),
            &stop,
        )?
        .into_parts();
    assert!(
        plan.issues()
            .iter()
            .any(|issue| issue.kind == LoadIssueKind::MissingFile)
    );
    assert!(
        plan.records()
            .iter()
            .any(|record| record.selection == LoadSelection::Excluded)
    );
    assert_eq!(plan.sources().len(), 5);
    let reference = wow_reference::ReferenceView::new(
        input_bundle_with_plan(files.clone(), Some(&plan))?
            .configuration()
            .reference_generation()
            .to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    let service_input = crate::LocalProjectInput::new_with_load_plan(
        input_bundle_with_plan(files.clone(), Some(&plan))?,
        reference,
        Some(plan.clone()),
    )?;
    let (publisher, graph) = owners_from_bundle(input_bundle_with_plan(files, Some(&plan))?)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let archive = serde_json::to_value(&replay)?;
    assert_eq!(archive["schema"], "wow-project/native-project-replay/4");
    assert_eq!(archive["generation_schema_version"], 2);
    assert_eq!(
        archive["load"]["documents"]
            .as_array()
            .ok_or("documents missing")?
            .len(),
        3
    );
    std::fs::remove_dir_all(&source_root)?;
    let restored = replay.hydrate(&stop)?;
    assert_eq!(restored.configuration().load_plan(), Some(&plan));
    assert_eq!(
        restored.snapshot_id(),
        publisher
            .current_snapshot()
            .ok_or("missing owner")?
            .snapshot_id()
    );
    assert!(
        restored
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .is_some()
    );
    for mutation in 0..5 {
        let mut changed = archive.clone();
        match mutation {
            0 => {
                changed["load"]["documents"][0]["text"] = "## Interface: 120100\nmain.lua\n".into()
            }
            1 => {
                changed["load"]["documents"]
                    .as_array_mut()
                    .ok_or("documents missing")?
                    .pop();
            }
            2 => {
                changed["load"]["documents"]
                    .as_array_mut()
                    .ok_or("documents missing")?
                    .push(serde_json::json!({"path":"unused.xml", "text":"<Ui/>"}));
            }
            3 => changed["load"]["context"]["game_types"]["classic"] = true.into(),
            _ => changed["schema"] = "wow-project/native-project-replay/1".into(),
        }
        let substituted: ProjectReplay = serde_json::from_value(changed)?;
        assert!(
            substituted.hydrate(&stop).is_err(),
            "archive mutation {mutation} was accepted"
        );
    }
    assert_eq!(
        replay
            .hydrate(&AtomicBool::new(true))
            .err()
            .ok_or("cancellation accepted")?
            .code(),
        wow_project::ProjectErrorCode::AnalysisCancelled
    );
    let store_root = root.join("store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let epoch = store.store.epoch().clone();
    store.publish(&publisher, &graph, "fixture:toc-xml-pair", None, &stop)?;
    let before = store.read(&ReadSelector::Current, &stop)?;
    let generation = before.store_generation_id().clone();
    let set_id = before.publication_set_id().to_owned();
    drop(before);
    drop(store);
    let store = LiveProjectStore::open(&store_root)?;
    assert_eq!(store.store.epoch(), &epoch);
    let reopened = store.read(&ReadSelector::Exact(generation), &stop)?;
    assert_eq!(reopened.project().configuration().load_plan(), Some(&plan));
    assert_eq!(reopened.graph(), &graph);
    assert_eq!(reopened.publication_set_id(), set_id);
    drop(reopened);
    drop(store);
    let composed_root = root.join("composed");
    let request = LiveProjectPublishRequest::new("fixture:toc-xml-composed", "absent", true, true)?;
    let graph_request =
        crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    let result = operations::publish_input(
        service_input,
        &graph_request,
        &composed_root,
        &request,
        &stop,
    )?;
    assert_eq!(result.exit_code(), 2);
    let composed = LiveProjectStore::open(&composed_root)?;
    let read = composed.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.project().configuration().load_plan(), Some(&plan));
    assert_eq!(read.project().snapshot_id(), restored.snapshot_id());
    assert!(
        read.graph()
            .partition(wow_recognizers::source_xml::SourceXmlFamily::Object.partition_id())
            .is_some()
    );
    drop(read);
    drop(composed);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn platform_pair_publishes_retained_source_and_old_catalog_refuses_v5() -> TestResult {
    use sha2::{Digest, Sha256};
    use std::sync::Arc;
    use wow_core::{CoverageStatus, SourceContent};
    use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
    use wow_project::load::{ProjectPackageInput, ProjectPackageVariantInput};
    use wow_project::platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformRootInventory, PlatformRootSpec,
        PlatformSourceClass, PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision,
        PlatformTarget, ProfileExclusion, SourceAdmissionLimits,
    };
    use wow_project::replay::ProjectReplay;

    let stop = AtomicBool::new(false);
    let path = root("platform-replay-v5")?;
    let source_root = path.join("input");
    std::fs::create_dir_all(source_root.join("UI"))?;
    let ordinary = input_bundle("return External()")?;
    let metadata = ordinary.configuration();
    let target = PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile: metadata.selected_profile().clone(),
        reference_generation: metadata.reference_generation(),
    };
    let profile = BlizzardUiSourceProfile::new(BlizzardUiSourceProfileRequest {
        profile_id: "profile:fixture:service-platform-replay-v1".parse()?,
        source_class: PlatformSourceClass::SyntheticFixture,
        target: target.clone(),
        roots: vec![PlatformRootSpec {
            root: "UI".into(),
            selected_tocs: vec!["UI/Fixture.toc".into()],
        }],
        exclusions: vec![ProfileExclusion {
            path: "UI/omitted.txt".into(),
        }],
        limits: SourceAdmissionLimits::new(16, 1024 * 1024, 1024 * 1024, 32 * 1024)?,
    })?;
    let raw_digest =
        |bytes: &[u8]| ContentDigest::<SourceContent>::from_bytes(Sha256::digest(bytes).into());
    let mut entries = Vec::new();
    let members: [(&str, PlatformFileKind, &[u8]); 4] = [
        (
            "Fixture.toc",
            PlatformFileKind::Toc,
            include_bytes!("../../../wow-project/tests/data/xml-facts/Fixture.toc"),
        ),
        (
            "frames.xml",
            PlatformFileKind::Xml,
            include_bytes!("../../../wow-project/tests/data/xml-facts/frames.xml"),
        ),
        (
            "defs.lua",
            PlatformFileKind::Lua,
            include_bytes!("../../../wow-project/tests/data/xml-facts/defs.lua"),
        ),
        ("opaque.bin", PlatformFileKind::Unknown, &[0xff, 0xfe, 0, 1]),
    ];
    for (name, kind, bytes) in members {
        std::fs::write(source_root.join("UI").join(name), bytes)?;
        entries.push(PlatformInventoryEntry {
            path: format!("UI/{name}"),
            kind,
            disposition: PlatformEntryDisposition::Included {
                digest: raw_digest(bytes),
                byte_length: bytes.len() as u64,
                object_id: None,
            },
        });
    }
    entries.push(PlatformInventoryEntry {
        path: "UI/omitted.txt".into(),
        kind: PlatformFileKind::Unknown,
        disposition: PlatformEntryDisposition::Excluded {
            rule_path: "UI/omitted.txt".into(),
        },
    });
    let inventory = PlatformSourceInventory {
        schema: "wow-project/platform-source-inventory/1".into(),
        profile_digest: profile.digest(),
        target,
        origin: PlatformSourceOrigin {
            provider: "handwritten-fixture".into(),
            repository: "service-platform-replay-fixture".into(),
            revision: PlatformSourceRevision::Fixture {
                digest: raw_digest(b"service-platform-replay-fixture-v1"),
            },
        },
        materializer: PlatformMaterializer {
            producer: "wow.fixture_materializer".parse()?,
            version: "1.0.0".parse()?,
            configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
            report_digest: raw_digest(b"local handwritten source declaration"),
        },
        roots: vec![PlatformRootInventory {
            root: "UI".into(),
            declared_entries: entries.len() as u64,
            scope: PlatformInventoryScope::DeclaredPartial,
            evidence_digest: raw_digest(b"explicit partial inventory fixture"),
        }],
        entries,
        license: PlatformLicenseRecord {
            state: PlatformLicenseState::Unknown,
            attribution: "project-owned handwritten fixture".into(),
            evidence_digest: raw_digest(b"synthetic fixture notice"),
        },
        compatibility_evidence: raw_digest(b"local-only compatibility assertion"),
    };
    let source = Arc::new(
        ProjectInputDirectory::open(&source_root)?
            .admit_platform_source(&profile, inventory, &stop)?,
    );
    std::fs::remove_dir_all(&source_root)?;
    let packages = Arc::new(source.specialize_packages(
        &[ProjectPackageInput::new(
            "Fixture",
            "UI",
            true,
            vec![ProjectPackageVariantInput::new(
                ProjectDiskFile::new("Fixture.toc"),
                true,
            )],
        )],
        None,
        &stop,
    )?);
    let configuration = ProjectConfigurationBuilder::new(
        metadata.project_id().clone(),
        ProjectKind::BlizzardUiPlatformSource,
        metadata.selected_profile().clone(),
        metadata.reference_generation(),
        metadata.analyzer_binding().clone(),
    )
    .workspace_id(metadata.workspace_id().clone())
    .source_origin_id(metadata.source_origin_id().clone())
    .logical_root(metadata.logical_root().as_str())
    .capability_policy(metadata.capability_policy().clone())
    .budget_policy(metadata.budget_policy())
    .platform_packages(Arc::clone(&packages))?
    .build()?;
    let (publisher, graph) = owners_from_bundle(ProjectInputBundle::closed(
        configuration.clone(),
        packages.files().to_vec(),
        ordinary.libraries().to_vec(),
    )?)?;
    let project = publisher.open_current()?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    assert_eq!(
        serde_json::to_value(&replay)?["schema"],
        "wow-project/native-project-replay/5"
    );
    let store_path = path.join("store");
    let mut store = LiveProjectStore::create(&store_path, graph.snapshot().universe().as_str())?;
    let operation = store.publish(&publisher, &graph, "fixture:platform-v5", None, &stop)?;
    assert_eq!(operation.state, PublicationState::Activated);
    let current = store.current()?.ok_or("platform current missing")?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert!(
        read.read
            .manifest()
            .members
            .iter()
            .any(|member| member.key == "live.project.replay"
                && member.schema == "wow-project.live-replay.v5")
    );
    assert_eq!(read.project().snapshot_id(), project.snapshot_id());
    assert_eq!(read.graph(), &graph);
    let set_id = read.publication_set_id().to_owned();
    drop(read);
    drop(store);

    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Publication(current.record_id.clone()), &stop)?;
    assert_eq!(read.store_generation_id(), &operation.generation_id);
    assert_eq!(read.project().configuration(), &configuration);
    assert_eq!(read.project().snapshot_id(), project.snapshot_id());
    assert_eq!(read.graph(), &graph);
    assert_eq!(read.publication_set_id(), set_id);
    assert_eq!(read.current_at_acquisition(), Some(&current));
    let restored = read
        .project()
        .configuration()
        .platform_packages()
        .ok_or("restored platform owner missing")?;
    assert_eq!(restored.binding(), packages.binding());
    assert_eq!(restored.source().receipt(), source.receipt());
    assert_eq!(
        restored.source().source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 1]
    );
    assert_eq!(
        restored.source().receipt().coverage().inventory(),
        CoverageStatus::Partial
    );
    assert_eq!(
        read.project()
            .snapshot()
            .analyzer_binding()
            .main_workspace()
            .universe(),
        LuaWorkspaceUniverse::BlizzardUiMain
    );
    assert_eq!(
        read.project()
            .snapshot()
            .analyzer_binding()
            .library_snapshot_ids()
            .collect::<Vec<_>>(),
        project
            .snapshot()
            .analyzer_binding()
            .library_snapshot_ids()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        read.graph().snapshot().universe().as_str(),
        packages.binding().universe_id()
    );
    assert_eq!(
        read.into_update_publisher()
            .err()
            .ok_or("platform replay admitted physical-only update capability")?
            .code(),
        ServiceErrorCode::OperationNotImplementedForMilestone
    );
    drop(store);

    // The public service rejection uses the exact platform owner in both epochs.
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create_with_gc(
            path.join("platform-v4"),
            graph.snapshot().universe().as_str(),
            catalog_for(publication::STORAGE_SCHEMAS_V4)?,
        )?,
    };
    let epoch = legacy.store.epoch().clone();
    let before = legacy.current()?;
    assert_eq!(
        legacy
            .publish(&publisher, &graph, "fixture:v5-to-v4", None, &stop)
            .err()
            .ok_or("v4 catalog admitted platform v5")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert_eq!(legacy.current()?, before);
    assert_eq!(legacy.store.epoch(), &epoch);
    assert!(legacy.reconcile("fixture:v5-to-v4")?.is_none());
    drop(legacy);

    // Independently prove the catalog gate with an already populated v4 Current.
    let (ordinary_publisher, ordinary_graph) = owners_from_bundle(ordinary)?;
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create_with_gc(
            path.join("populated-v4"),
            ordinary_graph.snapshot().universe().as_str(),
            catalog_for(publication::STORAGE_SCHEMAS_V4)?,
        )?,
    };
    legacy.publish(
        &ordinary_publisher,
        &ordinary_graph,
        "fixture:before-v5",
        None,
        &stop,
    )?;
    let before = legacy.current()?.ok_or("v4 current missing")?;
    let epoch = legacy.store.epoch().clone();
    let id = OperationId::new("fixture:v5-catalog-request")?;
    let (records, bindings) =
        ProjectPublicationBundle::build(&publisher, &graph, &stop)?.into_parts();
    assert_eq!(
        PublicationRequest::new(
            &epoch,
            id.clone(),
            Some(before.record_id.clone()),
            bindings,
            records
        )
        .err()
        .ok_or("v4 catalog admitted a real v5 replay record")?
        .code(),
        StoreErrorCode::IntegrityViolation
    );
    assert_eq!(legacy.current()?, Some(before));
    assert_eq!(legacy.store.epoch(), &epoch);
    assert!(legacy.store.operation(&id)?.is_none());
    drop(legacy);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn original_v4_catalog_reopens_exactly_without_relabeling() -> TestResult {
    let stop = AtomicBool::new(false);
    let path = root("legacy-v4-catalog")?;
    let (publisher, graph) = owners("return External()")?;
    let mut store = LiveProjectStore {
        store: ProjectStore::create_with_gc(
            &path,
            graph.snapshot().universe().as_str(),
            catalog_for(publication::STORAGE_SCHEMAS_V4)?,
        )?,
    };
    let epoch = store.store.epoch().clone();
    let operation = store.publish(&publisher, &graph, "fixture:original-v4", None, &stop)?;
    let current = store.current()?.ok_or("missing v4 current")?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    let snapshot_id = read.project().snapshot_id().to_owned();
    let publication_set_id = read.publication_set_id().to_owned();
    drop(read);
    drop(store);
    assert_eq!(
        ProjectStore::open(&path, &catalog()?)
            .err()
            .ok_or("latest catalog opened an original v4 epoch")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    let store = LiveProjectStore::open(&path)?;
    assert_eq!(store.store.epoch(), &epoch);
    assert_eq!(store.current()?, Some(current.clone()));
    let read = store.read(&ReadSelector::Publication(current.record_id), &stop)?;
    assert_eq!(read.store_generation_id(), &operation.generation_id);
    assert_eq!(read.project().snapshot_id(), snapshot_id);
    assert_eq!(read.graph(), &graph);
    assert_eq!(read.publication_set_id(), publication_set_id);
    drop(read);
    drop(store);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn original_epochs_reopen_exactly_and_refuse_new_generation_recipe() -> TestResult {
    use wow_project::replay::ProjectReplay;
    let stop = AtomicBool::new(false);
    let root = root("legacy-epochs")?;
    std::fs::create_dir(&root)?;
    let (new, new_graph) = owners("return External()")?;
    for (name, bytes, schemas) in [
        (
            "physical",
            include_str!("../../../wow-project/tests/data/native-replay-legacy/physical.json"),
            publication::STORAGE_SCHEMAS_V1,
        ),
        (
            "standalone",
            include_str!("../../../wow-project/tests/data/native-replay-legacy/standalone.json"),
            publication::STORAGE_SCHEMAS_V2,
        ),
        (
            "packages",
            include_str!("../../../wow-project/tests/data/native-replay-legacy/packages.json"),
            publication::STORAGE_SCHEMAS_V3,
        ),
    ] {
        let replay: ProjectReplay = serde_json::from_str(bytes)?;
        let project = replay.hydrate(&stop)?;
        let graph = graph_from_view(&project)?;
        let store_root = root.join(name);
        let mut store = LiveProjectStore {
            store: ProjectStore::create(
                &store_root,
                graph.snapshot().universe().as_str(),
                catalog_for(schemas)?,
            )?,
        };
        let epoch = store.store.epoch().clone();
        let bundle = ProjectPublicationBundle::from_replay(&replay, &graph, &stop)?;
        let operation =
            store.publish_bundle(bundle, &format!("fixture:legacy-{name}"), None, &stop)?;
        let current = store.current()?.ok_or("missing legacy current")?;
        let read = store.read(&ReadSelector::Current, &stop)?;
        let set_id = read.publication_set_id().to_owned();
        assert_eq!(read.project().snapshot_id(), project.snapshot_id());
        assert_eq!(read.graph(), &graph);
        drop(read);
        assert!(
            store
                .publish(
                    &new,
                    &new_graph,
                    &format!("fixture:v4-to-{name}"),
                    Some(current.record_id.clone()),
                    &stop
                )
                .is_err()
        );
        assert_eq!(store.current()?, Some(current.clone()));
        assert_eq!(store.store.epoch(), &epoch);
        drop(store);
        let store = LiveProjectStore::open(&store_root)?;
        assert_eq!(store.store.epoch(), &epoch);
        assert_eq!(store.current()?, Some(current));
        let read = store.read(&ReadSelector::Exact(operation.generation_id), &stop)?;
        assert_eq!(read.project().snapshot_id(), project.snapshot_id());
        assert_eq!(
            read.project().project_generation(),
            project.project_generation()
        );
        assert_eq!(read.graph(), &graph);
        assert_eq!(read.publication_set_id(), set_id);
        drop(read);
        drop(store);
    }
    std::fs::remove_dir_all(root)?;
    Ok(())
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
                "wow-project.live-replay.v4",
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
#[path = "recovery_tests.rs"]
mod recovery_tests;
#[path = "replacement_tests.rs"]
mod replacement_tests;
