use super::*;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use wow_core::{ProfileId, SourceContent};
use wow_project::PlatformGraphProfile;
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_project::load::{ProjectPackageInput, ProjectPackageVariantInput};
use wow_project::platform_source::{
    BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
    PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
    PlatformLicenseState, PlatformMaterializer, PlatformRootInventory, PlatformRootSpec,
    PlatformSourceClass, PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision,
    PlatformTarget, ProfileExclusion, SourceAdmissionLimits,
};

const SOURCE_PROFILE: &str = "profile:fixture:service-platform-namespace-v1";

fn platform_bundle(
    path: &Path,
    opaque_revision: u8,
    stop: &AtomicBool,
) -> TestResult<ProjectInputBundle> {
    platform_bundle_with_bindings(path, opaque_revision, false, stop)
}

fn platform_bundle_with_bindings(
    path: &Path,
    opaque_revision: u8,
    selected_bindings: bool,
    stop: &AtomicBool,
) -> TestResult<ProjectInputBundle> {
    platform_bundle_with_profiles(path, opaque_revision, selected_bindings, None, stop)
}

fn platform_bundle_with_profiles(
    path: &Path,
    opaque_revision: u8,
    selected_bindings: bool,
    graph_profile: Option<PlatformGraphProfile>,
    stop: &AtomicBool,
) -> TestResult<ProjectInputBundle> {
    // The new graph fixture admits both roots before specialization. Existing
    // callers retain the exact single-root profile, inventory and declarations.
    let package_roots: &[(&str, &str)] = if graph_profile.is_some() {
        &[("UI", "Fixture"), ("UISecond", "Twin")]
    } else {
        &[("UI", "Fixture")]
    };
    let ordinary = input_bundle("return External()")?;
    let metadata = ordinary.configuration();
    let target = PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile: metadata.selected_profile().clone(),
        reference_generation: metadata.reference_generation(),
    };
    let profile = BlizzardUiSourceProfile::new(BlizzardUiSourceProfileRequest {
        profile_id: SOURCE_PROFILE.parse()?,
        source_class: PlatformSourceClass::SyntheticFixture,
        target: target.clone(),
        roots: package_roots
            .iter()
            .map(|(source_root, _)| PlatformRootSpec {
                root: (*source_root).into(),
                selected_tocs: vec![format!("{source_root}/Fixture.toc")],
            })
            .collect(),
        exclusions: package_roots
            .iter()
            .map(|(source_root, _)| ProfileExclusion {
                path: format!("{source_root}/omitted.txt"),
            })
            .collect(),
        limits: SourceAdmissionLimits::new(16, 1024 * 1024, 1024 * 1024, 32 * 1024)?,
    })?;
    let raw_digest =
        |bytes: &[u8]| ContentDigest::<SourceContent>::from_bytes(Sha256::digest(bytes).into());
    let opaque = [0xff, 0xfe, 0, opaque_revision];
    let members: [(&str, PlatformFileKind, &[u8]); 4] = [
        (
            "Fixture.toc",
            PlatformFileKind::Toc,
            include_bytes!("../../../../wow-project/tests/data/xml-facts/Fixture.toc"),
        ),
        (
            "frames.xml",
            PlatformFileKind::Xml,
            include_bytes!("../../../../wow-project/tests/data/xml-facts/frames.xml"),
        ),
        (
            "defs.lua",
            PlatformFileKind::Lua,
            include_bytes!("../../../../wow-project/tests/data/xml-facts/defs.lua"),
        ),
        ("opaque.bin", PlatformFileKind::Unknown, &opaque),
    ];
    let mut entries = Vec::new();
    for (source_root, _) in package_roots {
        std::fs::create_dir_all(path.join(source_root))?;
        for &(name, kind, bytes) in &members {
            std::fs::write(path.join(source_root).join(name), bytes)?;
            entries.push(PlatformInventoryEntry {
                path: format!("{source_root}/{name}"),
                kind,
                disposition: PlatformEntryDisposition::Included {
                    digest: raw_digest(bytes),
                    byte_length: bytes.len() as u64,
                    object_id: None,
                },
            });
        }
        entries.push(PlatformInventoryEntry {
            path: format!("{source_root}/omitted.txt"),
            kind: PlatformFileKind::Unknown,
            disposition: PlatformEntryDisposition::Excluded {
                rule_path: format!("{source_root}/omitted.txt"),
            },
        });
    }
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
        roots: package_roots
            .iter()
            .map(|(source_root, _)| PlatformRootInventory {
                root: (*source_root).into(),
                declared_entries: (members.len() + 1) as u64,
                scope: PlatformInventoryScope::DeclaredPartial,
                evidence_digest: raw_digest(b"explicit partial inventory fixture"),
            })
            .collect(),
        entries,
        license: PlatformLicenseRecord {
            state: PlatformLicenseState::Unknown,
            attribution: "project-owned handwritten fixture".into(),
            evidence_digest: raw_digest(b"synthetic fixture notice"),
        },
        compatibility_evidence: raw_digest(b"local-only compatibility assertion"),
    };
    let source = Arc::new(
        ProjectInputDirectory::open(path)?.admit_platform_source(&profile, inventory, stop)?,
    );
    std::fs::remove_dir_all(path)?;
    let packages = Arc::new(
        source.specialize_packages(
            &package_roots
                .iter()
                .map(|(source_root, package)| {
                    ProjectPackageInput::new(
                        *package,
                        *source_root,
                        true,
                        vec![ProjectPackageVariantInput::new(
                            ProjectDiskFile::new("Fixture.toc"),
                            true,
                        )],
                    )
                })
                .collect::<Vec<_>>(),
            None,
            stop,
        )?,
    );
    let builder = ProjectConfigurationBuilder::new(
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
    .platform_packages(packages.clone())?;
    let builder = if selected_bindings {
        builder.with_package_xml_bindings(wow_project::PackageXmlBindingProfile::SameSessionV1)
    } else {
        builder
    };
    let configuration = match graph_profile {
        Some(profile) => builder.with_platform_graph_profile(profile),
        None => builder,
    }
    .build()?;
    Ok(ProjectInputBundle::closed(
        configuration,
        packages.files().to_vec(),
        ordinary.libraries().to_vec(),
    )?)
}

fn platform_owners(
    path: &Path,
    opaque_revision: u8,
    stop: &AtomicBool,
) -> TestResult<(ProjectPublisher, GraphPartitionSnapshot)> {
    owners_from_bundle(platform_bundle(path, opaque_revision, stop)?)
}

#[test]
fn selected_package_bindings_reopen_exactly_and_frozen_v5_refuses_without_effects() -> TestResult {
    use wow_project::replay::ProjectReplay;
    let stop = AtomicBool::new(false);
    let path = root("platform-package-bindings-v6")?;
    let bundle = platform_bundle(&path.join("legacy-input"), 1, &stop)?;
    let selected_bundle =
        platform_bundle_with_bindings(&path.join("selected-input"), 1, true, &stop)?;
    let configuration = selected_bundle.configuration();
    let packages = configuration
        .platform_packages()
        .ok_or("platform missing")?;
    let selection = PlatformStoreSelection::new(
        configuration.project_id().clone(),
        packages.source().profile().profile_id().clone(),
    )?;
    let (selected, graph) = owners_from_bundle(selected_bundle)?;
    let view = selected.open_current()?;
    let binding_id = view
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("selected binding missing")?
        .analysis_id()
        .to_owned();
    let replay = ProjectReplay::capture(&selected, &stop)?;
    assert_eq!(
        serde_json::to_value(&replay)?["schema"],
        "wow-project/native-project-replay/6"
    );
    let mut store = LiveProjectStore::create_in_namespace(&path.join("new"), &selection)?;
    store.publish_in_namespace(
        &selection,
        &selected,
        &graph,
        "fixture:bindings-v6",
        None,
        &stop,
    )?;
    let current = store.current()?.ok_or("v6 Current missing")?;
    drop(store);
    let store = LiveProjectStore::open_in_namespace(&path.join("new"), &selection)?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), view.snapshot_id());
    assert_eq!(read.graph(), &graph);
    assert_eq!(
        read.project()
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .ok_or("reopened binding missing")?
            .analysis_id(),
        binding_id
    );
    assert_eq!(store.current()?, Some(current));
    drop(read);
    drop(store);

    let (legacy_owner, legacy_graph) = owners_from_bundle(bundle)?;
    let legacy_path = path.join("frozen-v5");
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create_with_namespace(
            &legacy_path,
            selection.namespace(),
            catalog_for(publication::STORAGE_SCHEMAS_V5)?,
        )?,
    };
    legacy.publish_in_namespace(
        &selection,
        &legacy_owner,
        &legacy_graph,
        "fixture:legacy-v5",
        None,
        &stop,
    )?;
    let old_current = legacy.current()?.ok_or("v5 Current missing")?;
    let old_epoch = legacy.store.epoch().clone();
    assert!(
        legacy
            .publish_in_namespace(
                &selection,
                &selected,
                &graph,
                "fixture:refused-v6",
                Some(old_current.record_id.clone()),
                &stop
            )
            .is_err()
    );
    assert_eq!(legacy.current()?, Some(old_current.clone()));
    assert!(legacy.reconcile("fixture:refused-v6")?.is_none());
    drop(legacy);
    let legacy = LiveProjectStore::open_in_namespace(&legacy_path, &selection)?;
    assert_eq!(legacy.store.epoch(), &old_epoch);
    assert_eq!(legacy.current()?, Some(old_current));
    let old_read = legacy.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(
        old_read.project().snapshot_id(),
        legacy_owner.open_current()?.snapshot_id()
    );
    assert!(
        old_read
            .project()
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .is_none()
    );
    assert_eq!(old_read.graph(), &legacy_graph);
    drop(old_read);
    drop(legacy);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn selected_package_graph_reopens_and_frozen_v6_refuses_without_effects() -> TestResult {
    use wow_project::replay::ProjectReplay;
    use wow_recognizers::source_xml::SourceXmlFamily;

    let stop = AtomicBool::new(false);
    let path = root("platform-package-graph-v7")?;
    let legacy_bundle = platform_bundle_with_bindings(&path.join("legacy-input"), 1, true, &stop)?;
    let selected_bundle = platform_bundle_with_profiles(
        &path.join("selected-input"),
        1,
        true,
        Some(PlatformGraphProfile::PackageProjectionV1),
        &stop,
    )?;
    assert!(!path.join("legacy-input").exists());
    assert!(!path.join("selected-input").exists());
    let packages = selected_bundle
        .configuration()
        .platform_packages()
        .ok_or("selected platform plans missing")?;
    assert_eq!(packages.load_plan().packages().len(), 2);
    assert_eq!(
        packages.source().source_bytes("UI/frames.xml")?,
        packages.source().source_bytes("UISecond/frames.xml")?
    );
    let first_xml = packages
        .load_plan()
        .package_plan("Fixture")
        .ok_or("first package missing")?
        .xml_documents()
        .get("frames.xml")
        .ok_or("first XML index missing")?;
    let second_xml = packages
        .load_plan()
        .package_plan("Twin")
        .ok_or("second package missing")?
        .xml_documents()
        .get("frames.xml")
        .ok_or("second XML index missing")?;
    assert_eq!(first_xml.source_digest(), second_xml.source_digest());
    assert_eq!(
        first_xml
            .elements()
            .iter()
            .map(|element| element.occurrence_id.as_str())
            .collect::<Vec<_>>(),
        second_xml
            .elements()
            .iter()
            .map(|element| element.occurrence_id.as_str())
            .collect::<Vec<_>>()
    );
    let selection = PlatformStoreSelection::new(
        selected_bundle.configuration().project_id().clone(),
        packages.source().profile().profile_id().clone(),
    )?;
    let (selected, selected_graph) = owners_from_bundle(selected_bundle.clone())?;
    let selected_view = selected.open_current()?;
    assert_eq!(
        wow_project::graph::source_graph_profile(selected_view.configuration()),
        "wow-project/source-load-proposals/20"
    );
    assert_eq!(
        serde_json::to_value(ProjectReplay::capture(&selected, &stop)?)?["schema"],
        "wow-project/native-project-replay/7"
    );
    let (legacy_owner, legacy_graph) = owners_from_bundle(legacy_bundle)?;
    let legacy_view = legacy_owner.open_current()?;
    assert!(
        legacy_view
            .configuration()
            .platform_graph_profile()
            .is_none()
    );
    assert_eq!(
        wow_project::graph::source_graph_profile(legacy_view.configuration()),
        "wow-project/source-load-proposals/19"
    );
    assert_eq!(
        serde_json::to_value(ProjectReplay::capture(&legacy_owner, &stop)?)?["schema"],
        "wow-project/native-project-replay/6"
    );

    let input = || -> TestResult<crate::LocalProjectInput> {
        let reference = wow_reference::ReferenceView::new(
            selected_bundle
                .configuration()
                .reference_generation()
                .to_string(),
            Vec::new(),
            Vec::new(),
        )?;
        Ok(crate::LocalProjectInput::new_with_package_plans(
            selected_bundle.clone(),
            reference,
            packages.load_plan().clone(),
            packages.main_plan().clone(),
        )?)
    };
    let old_request = crate::graph::GraphBuildRequest::new(
        selected_bundle.configuration().project_id().as_str().into(),
        "current".into(),
    )?;
    let refused_root = path.join("wrong-graph-request");
    let refused =
        LiveProjectPublishRequest::new("fixture:graph-v19-refused", "absent", true, true)?;
    assert_eq!(
        publish_input_in_namespace(
            input()?,
            &old_request,
            &refused_root,
            &selection,
            &refused,
            &stop,
        )
        .err()
        .ok_or("unselected graph request admitted the /20 owner")?
        .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert!(!refused_root.exists());

    let graph_request =
        old_request.with_platform_graph_profile(PlatformGraphProfile::PackageProjectionV1);
    let store_path = path.join("selected");
    let request = LiveProjectPublishRequest::new("fixture:package-graph-v7", "absent", true, true)?;
    let published = publish_input_in_namespace(
        input()?,
        &graph_request,
        &store_path,
        &selection,
        &request,
        &stop,
    )?;
    assert_eq!(published.exit_code(), 2);
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    let current = store.current()?.ok_or("selected graph Current missing")?;
    let epoch = store.store.epoch().clone();
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), selected_view.snapshot_id());
    assert_eq!(
        read.project().configuration(),
        selected_bundle.configuration()
    );
    assert!(read.read.manifest().members.iter().any(|member| {
        member.key == "live.project.replay" && member.schema == "wow-project.live-replay.v7"
    }));
    assert_eq!(
        read.graph()
            .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
            .ok_or("composed source partition missing")?
            .batch(),
        selected_graph
            .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
            .ok_or("native source partition missing")?
            .batch()
    );

    // Resolve original accepted object keys to final native nodes. Input receipt
    // IDs are generation-local and must not stand in for materialized IDs.
    let objects = read
        .graph()
        .partition(SourceXmlFamily::Object.partition_id())
        .ok_or("XML object matcher partition missing")?;
    let mut object_documents = BTreeMap::new();
    for proposal in objects.batch().entity_proposals() {
        if proposal.entity_kind_id() != "xml_object" {
            continue;
        }
        let Some(wow_graph::GraphProposalValue::String(document)) =
            proposal.semantic_key().get("document")
        else {
            return Err("XML object has no native document key".into());
        };
        let accepted = objects
            .report()
            .accepted_entities()
            .iter()
            .find(|accepted| accepted.proposal_id() == proposal.proposal_id())
            .ok_or("XML object proposal was not accepted")?;
        let node = read
            .graph()
            .snapshot()
            .nodes()
            .iter()
            .find(|node| {
                node.kind() == accepted.node().kind()
                    && node.owner_key() == accepted.node().owner_key()
            })
            .ok_or("accepted XML object is absent from the final graph")?;
        assert!(
            object_documents
                .insert(node.node_id().clone(), document.to_string())
                .is_none()
        );
    }
    let documents = BTreeSet::from([
        "packages/Fixture/frames.xml".to_owned(),
        "packages/Twin/frames.xml".to_owned(),
    ]);
    assert_eq!(
        object_documents.values().cloned().collect::<BTreeSet<_>>(),
        documents
    );
    let parents = read
        .graph()
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| edge.relation() == wow_graph::GraphRelationKind::ParentOf)
        .collect::<Vec<_>>();
    assert_eq!(parents.len(), 2);
    let mut parent_documents = BTreeSet::new();
    for edge in parents {
        let source = object_documents
            .get(edge.from())
            .ok_or("ParentOf source is not a final XML object")?;
        let target = object_documents
            .get(edge.to())
            .ok_or("ParentOf target is not a final XML object")?;
        assert_eq!(source, target, "ParentOf crossed native package scopes");
        parent_documents.insert(source.clone());
    }
    assert_eq!(parent_documents, documents);
    let graph = read.graph().clone();
    let publication_set = read.publication_set_id().to_owned();
    drop(read);
    drop(store);
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    assert_eq!(store.store.epoch(), &epoch);
    assert_eq!(store.current()?, Some(current));
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), selected_view.snapshot_id());
    assert_eq!(read.graph(), &graph);
    assert_eq!(read.publication_set_id(), publication_set);
    assert_eq!(
        read.project().configuration().platform_graph_profile(),
        Some(PlatformGraphProfile::PackageProjectionV1)
    );
    drop(read);
    drop(store);

    let legacy_path = path.join("frozen-v6");
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create_with_namespace(
            &legacy_path,
            selection.namespace(),
            catalog_for(publication::STORAGE_SCHEMAS_V6)?,
        )?,
    };
    legacy.publish_in_namespace(
        &selection,
        &legacy_owner,
        &legacy_graph,
        "fixture:before-graph-v7",
        None,
        &stop,
    )?;
    let old_current = legacy.current()?.ok_or("frozen v6 Current missing")?;
    let old_epoch = legacy.store.epoch().clone();
    assert!(
        legacy
            .publish_in_namespace(
                &selection,
                &selected,
                &selected_graph,
                "fixture:graph-v7-to-v6",
                Some(old_current.record_id.clone()),
                &stop,
            )
            .is_err()
    );
    assert_eq!(legacy.current()?, Some(old_current.clone()));
    assert_eq!(legacy.store.epoch(), &old_epoch);
    assert!(legacy.reconcile("fixture:graph-v7-to-v6")?.is_none());
    drop(legacy);
    let legacy = LiveProjectStore::open_in_namespace(&legacy_path, &selection)?;
    assert_eq!(legacy.store.epoch(), &old_epoch);
    assert_eq!(legacy.current()?, Some(old_current));
    let read = legacy.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), legacy_view.snapshot_id());
    assert_eq!(read.graph(), &legacy_graph);
    assert!(
        read.project()
            .configuration()
            .platform_graph_profile()
            .is_none()
    );
    drop(read);
    drop(legacy);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn platform_namespace_advances_current_with_exact_readers_and_root_independent_ids() -> TestResult {
    let stop = AtomicBool::new(false);
    let path = root("platform-namespace")?;
    let (first, first_graph) = platform_owners(&path.join("source-one"), 1, &stop)?;
    let (second, second_graph) = platform_owners(&path.join("source-two"), 2, &stop)?;
    let first_view = first.open_current()?;
    let second_view = second.open_current()?;
    let first_packages = first_view
        .configuration()
        .platform_packages()
        .ok_or("first platform owner missing")?;
    let second_packages = second_view
        .configuration()
        .platform_packages()
        .ok_or("second platform owner missing")?;
    assert!(!path.join("source-one").exists());
    assert!(!path.join("source-two").exists());
    assert_eq!(
        first_view.configuration().project_id(),
        second_view.configuration().project_id()
    );
    assert_eq!(
        first_packages.source().profile(),
        second_packages.source().profile()
    );
    assert_ne!(
        first_packages.source().receipt().source_snapshot_id(),
        second_packages.source().receipt().source_snapshot_id()
    );
    assert_ne!(
        first_view.project_generation(),
        second_view.project_generation()
    );
    assert_ne!(first_view.snapshot_id(), second_view.snapshot_id());
    assert_ne!(
        first_graph.snapshot().universe(),
        second_graph.snapshot().universe()
    );
    for view in [&first_view, &second_view] {
        let analyzer = view.snapshot().analyzer_binding();
        assert_eq!(
            analyzer.main_workspace().universe(),
            LuaWorkspaceUniverse::BlizzardUiMain
        );
        assert_eq!(analyzer.library_snapshot_ids().count(), 1);
        assert!(
            !analyzer
                .xml_lua_analysis()
                .ok_or("XML Main missing")?
                .units()
                .is_empty()
        );
    }
    let selection = PlatformStoreSelection::new(
        first_view.configuration().project_id().clone(),
        SOURCE_PROFILE.parse::<ProfileId>()?,
    )?;
    let (legacy_records, legacy_bindings) =
        ProjectPublicationBundle::build(&first, &first_graph, &stop)?.into_parts();
    let (namespace_records, namespace_bindings) = ProjectPublicationBundle::build_in_namespace(
        &first,
        &first_graph,
        selection.namespace(),
        &stop,
    )?
    .into_parts();
    assert_eq!(
        legacy_records
            .iter()
            .map(|r| (r.key(), r.version()))
            .collect::<Vec<_>>(),
        namespace_records
            .iter()
            .map(|r| (r.key(), r.version()))
            .collect::<Vec<_>>()
    );
    assert_ne!(legacy_bindings, namespace_bindings);
    assert_eq!(
        namespace_bindings["project_store_id"],
        selection.namespace().id().as_str()
    );

    let store_path = path.join("selected");
    let mut store = LiveProjectStore::create_in_namespace(&store_path, &selection)?;
    let epoch = store.store.epoch().clone();
    assert_eq!(epoch.namespace(), Some(selection.namespace()));
    let first_operation = store.publish_in_namespace(
        &selection,
        &first,
        &first_graph,
        "fixture:namespace-first",
        None,
        &stop,
    )?;
    assert_eq!(first_operation.state, PublicationState::Activated);
    let first_current = store.current()?.ok_or("first Current missing")?;
    let held = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    let first_members = held.read.manifest().members.clone();
    let first_set = held.publication_set_id().to_owned();
    assert_eq!(held.namespace(), Some(selection.namespace()));
    assert_eq!(held.project().snapshot_id(), first_view.snapshot_id());
    assert_eq!(held.graph(), &first_graph);

    let twin_path = path.join("same-namespace-other-root");
    let mut twin = LiveProjectStore::create_in_namespace(&twin_path, &selection)?;
    twin.publish_in_namespace(
        &selection,
        &first,
        &first_graph,
        "fixture:namespace-twin",
        None,
        &stop,
    )?;
    let twin_read = twin.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(
        twin_read.project().snapshot_id(),
        held.project().snapshot_id()
    );
    assert_eq!(
        twin_read.project().project_generation(),
        held.project().project_generation()
    );
    assert_eq!(
        twin_read.project().analyzer_snapshot_id(),
        held.project().analyzer_snapshot_id()
    );
    assert_eq!(twin_read.graph(), held.graph());
    assert_eq!(twin_read.publication_set_id(), first_set);
    assert_eq!(twin_read.read.manifest().members, first_members);
    drop(twin_read);
    drop(twin);

    let second_operation = store.publish_in_namespace(
        &selection,
        &second,
        &second_graph,
        "fixture:namespace-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    assert_eq!(second_operation.state, PublicationState::Activated);
    let second_current = store.current()?.ok_or("second Current missing")?;
    assert_ne!(second_current.record_id, first_current.record_id);
    assert_eq!(
        second_current.predecessor.as_ref(),
        Some(&first_current.record_id)
    );
    assert_eq!(second_operation.activation.as_ref(), Some(&second_current));
    assert_eq!(store.store.epoch(), &epoch);
    assert_eq!(held.current_at_acquisition(), Some(&first_current));
    assert_eq!(held.store_generation_id(), &first_operation.generation_id);
    assert_eq!(held.project().snapshot_id(), first_view.snapshot_id());
    assert_eq!(held.graph(), &first_graph);
    assert_eq!(held.publication_set_id(), first_set);
    assert_eq!(held.read.manifest().members, first_members);
    assert_eq!(
        held.project()
            .configuration()
            .platform_packages()
            .ok_or("held source missing")?
            .source()
            .source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 1]
    );

    let wrong_project = PlatformStoreSelection::new(
        ProjectId::new("fixture-other-owner")?,
        SOURCE_PROFILE.parse()?,
    )?;
    let wrong_profile = PlatformStoreSelection::new(
        first_view.configuration().project_id().clone(),
        "profile:fixture:service-other-namespace-v1".parse()?,
    )?;
    for (foreign, operation_id) in [
        (&wrong_project, "fixture:wrong-project"),
        (&wrong_profile, "fixture:wrong-profile"),
    ] {
        assert_ne!(foreign.namespace().id(), selection.namespace().id());
        assert!(
            ProjectPublicationBundle::build_in_namespace(
                &second,
                &second_graph,
                foreign.namespace(),
                &stop,
            )
            .is_err()
        );
        assert_eq!(
            store
                .publish_in_namespace(
                    foreign,
                    &second,
                    &second_graph,
                    operation_id,
                    Some(second_current.record_id.clone()),
                    &stop,
                )
                .err()
                .ok_or("foreign selection published")?
                .code(),
            ServiceErrorCode::IdentityMismatch
        );
        assert!(store.reconcile(operation_id)?.is_none());
        assert_eq!(store.current()?, Some(second_current.clone()));
        assert_eq!(store.store.epoch(), &epoch);
    }
    drop(held);
    drop(store);
    for foreign in [&wrong_project, &wrong_profile] {
        assert_eq!(
            LiveProjectStore::open_in_namespace(&store_path, foreign)
                .err()
                .ok_or("foreign selection reopened")?
                .code(),
            ServiceErrorCode::IdentityMismatch
        );
    }
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    assert_eq!(store.store.epoch(), &epoch);
    assert_eq!(store.current()?, Some(second_current.clone()));
    let latest = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(latest.namespace(), Some(selection.namespace()));
    assert_eq!(latest.current_at_acquisition(), Some(&second_current));
    assert_eq!(latest.project().snapshot_id(), second_view.snapshot_id());
    assert_eq!(latest.graph(), &second_graph);
    assert_ne!(latest.publication_set_id(), first_set);
    assert_eq!(
        latest.store_generation_id(),
        &second_operation.generation_id
    );
    assert_eq!(
        latest
            .project()
            .configuration()
            .platform_packages()
            .ok_or("reopened source missing")?
            .source()
            .source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 2]
    );
    let old = store.read_in_namespace(
        &selection,
        &ReadSelector::Publication(first_current.record_id),
        &stop,
    )?;
    assert_eq!(old.project().snapshot_id(), first_view.snapshot_id());
    assert_eq!(old.graph(), &first_graph);
    assert_eq!(old.publication_set_id(), first_set);
    assert_eq!(old.read.manifest().members, first_members);
    drop(old);
    drop(latest);
    drop(store);

    let legacy_path = path.join("legacy-v4");
    let (ordinary, ordinary_graph) = owners("return External()")?;
    let mut legacy = LiveProjectStore {
        store: ProjectStore::create_with_gc(
            &legacy_path,
            ordinary_graph.snapshot().universe().as_str(),
            catalog_for(publication::STORAGE_SCHEMAS_V4)?,
        )?,
    };
    let legacy_epoch = legacy.store.epoch().clone();
    assert!(legacy_epoch.namespace().is_none());
    legacy.publish(
        &ordinary,
        &ordinary_graph,
        "fixture:namespace-v4",
        None,
        &stop,
    )?;
    let legacy_current = legacy.current()?.ok_or("legacy Current missing")?;
    let read = legacy.read(&ReadSelector::Current, &stop)?;
    let legacy_set = read.publication_set_id().to_owned();
    let legacy_members = read.read.manifest().members.clone();
    drop(read);
    drop(legacy);
    assert_eq!(
        LiveProjectStore::open_in_namespace(&legacy_path, &selection)
            .err()
            .ok_or("legacy epoch became namespace epoch")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    let legacy = LiveProjectStore::open(&legacy_path)?;
    assert_eq!(legacy.store.epoch(), &legacy_epoch);
    assert_eq!(legacy.current()?, Some(legacy_current.clone()));
    let read = legacy.read(&ReadSelector::Publication(legacy_current.record_id), &stop)?;
    assert_eq!(
        read.project().snapshot_id(),
        ordinary.open_current()?.snapshot_id()
    );
    assert_eq!(read.graph(), &ordinary_graph);
    assert_eq!(read.publication_set_id(), legacy_set);
    assert_eq!(read.read.manifest().members, legacy_members);
    drop(read);
    drop(legacy);
    std::fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn selected_one_shot_platform_publication_retains_the_native_producer_chain() -> TestResult {
    let stop = AtomicBool::new(false);
    let path = root("platform-namespace-composed")?;
    let source_path = path.join("input");
    let bundle = platform_bundle(&source_path, 1, &stop)?;
    let packages = bundle
        .configuration()
        .platform_packages()
        .ok_or("platform plans missing")?;
    let selection = PlatformStoreSelection::new(
        bundle.configuration().project_id().clone(),
        packages.source().profile().profile_id().clone(),
    )?;
    let input = || -> TestResult<crate::LocalProjectInput> {
        let reference = wow_reference::ReferenceView::new(
            bundle.configuration().reference_generation().to_string(),
            Vec::new(),
            Vec::new(),
        )?;
        Ok(crate::LocalProjectInput::new_with_package_plans(
            bundle.clone(),
            reference,
            packages.load_plan().clone(),
            packages.main_plan().clone(),
        )?)
    };
    assert!(!source_path.exists());
    let graph_request = crate::graph::GraphBuildRequest::new(
        bundle.configuration().project_id().as_str().into(),
        "current".into(),
    )?;
    let foreign = PlatformStoreSelection::new(
        ProjectId::new("fixture-other-owner")?,
        packages.source().profile().profile_id().clone(),
    )?;
    let refused_root = path.join("refused");
    let refused =
        LiveProjectPublishRequest::new("fixture:namespace-composed-foreign", "absent", true, true)?;
    assert_eq!(
        publish_input_in_namespace(
            input()?,
            &graph_request,
            &refused_root,
            &foreign,
            &refused,
            &stop,
        )
        .err()
        .ok_or("foreign one-shot selection published")?
        .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert!(!refused_root.exists());

    let store_path = path.join("selected");
    let operation_id = "fixture:namespace-composed";
    let request = LiveProjectPublishRequest::new(operation_id, "absent", true, true)?;
    let published = publish_input_in_namespace(
        input()?,
        &graph_request,
        &store_path,
        &selection,
        &request,
        &stop,
    )?;
    assert_eq!(published.exit_code(), 2);
    let publication: serde_json::Value = serde_json::from_slice(&published.canonical_bytes()?)?;
    assert_eq!(publication["status"], "activated");
    assert_eq!(publication["operation"]["state"], "activated");
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    let current = store.current()?.ok_or("composed Current missing")?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.namespace(), Some(selection.namespace()));
    assert_eq!(read.project().configuration(), bundle.configuration());
    assert_eq!(
        read.graph().snapshot().universe().as_str(),
        packages.binding().universe_id()
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
        bundle
            .libraries()
            .iter()
            .map(|library| library.snapshot_id())
            .collect::<Vec<_>>()
    );
    for producer in [
        wow_project::graph::SOURCE_GRAPH_PARTITION,
        wow_recognizers::source_toc::SourceTocFamily::Package.partition_id(),
        wow_recognizers::source_xml::SourceXmlFamily::Object.partition_id(),
        wow_recognizers::source_bridge::W2_PARTITION,
        wow_recognizers::source_scripts::W5_HOOK_PARTITION,
        wow_recognizers::source_state::SOURCE_STATE_LIBRARY_PARTITION,
    ] {
        let partition = read
            .graph()
            .partition(producer)
            .ok_or("composed producer missing")?;
        assert!(
            partition
                .coverage()
                .iter()
                .all(|coverage| !coverage.negative_authority())
        );
    }
    let project_id = read.project().snapshot_id().to_owned();
    let graph_id = read.graph().snapshot().snapshot_id().as_str().to_owned();
    let set_id = read.publication_set_id().to_owned();
    assert_eq!(
        publication["operation"]["activation"],
        serde_json::to_value(&current)?
    );
    drop(read);
    drop(store);

    let selected = read_live_project_in_namespace(&store_path, &selection, "current", &stop)?;
    let acquired: serde_json::Value = serde_json::from_slice(&selected.canonical_bytes()?)?;
    assert_eq!(acquired["status"], "acquired");
    assert_eq!(acquired["current"], serde_json::to_value(&current)?);
    assert_eq!(acquired["pair"]["project_snapshot_id"], project_id);
    assert_eq!(acquired["pair"]["graph_snapshot_id"], graph_id);
    assert_eq!(acquired["pair"]["publication_set_id"], set_id);
    let reconciled =
        reconcile_live_project_in_namespace(&store_path, &selection, operation_id, &stop)?;
    let observed: serde_json::Value = serde_json::from_slice(&reconciled.canonical_bytes()?)?;
    assert_eq!(observed["status"], "observed");
    assert_eq!(observed["operation"], publication["operation"]);
    assert_eq!(observed["current"], serde_json::to_value(&current)?);
    for result in [&publication, &acquired, &observed] {
        assert_eq!(result["scope"], "native-platform-project-pair-namespace-v1");
        assert_eq!(
            result["namespace"]["project_store_id"],
            selection.namespace().id().as_str()
        );
        assert_eq!(
            result["namespace"]["logical_namespace"],
            packages.source().profile().profile_id().as_str()
        );
        assert_eq!(
            result["namespace"]["owner_project_id"],
            bundle.configuration().project_id().as_str()
        );
        assert_eq!(
            result["boundaries"][0],
            "admitted_platform_source_inputs_only"
        );
    }
    std::fs::remove_dir_all(path)?;
    Ok(())
}
