use std::{
    collections::BTreeSet,
    error::Error,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
    SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    bindings::SymbolLookupState,
    function_calls::{SourceCallTarget, SourceFunctionKind},
};
use wow_project::{
    AnalyzerBindingDeclaration, PackageXmlBindingProfile, ProjectBudgetPolicy,
    ProjectCapabilityPolicy, ProjectConfigurationBuilder, ProjectId, ProjectInputBundle,
    ProjectKind, ProjectPublisher, ProjectSourceOriginId, ProjectWorkspaceId,
    disk::{ProjectDiskFile, ProjectInputDirectory},
    load::{ProjectPackageInput, ProjectPackageVariantInput},
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformPackageSpecialization,
        PlatformRootInventory, PlatformRootSpec, PlatformSourceClass, PlatformSourceInventory,
        PlatformSourceOrigin, PlatformSourceRevision, PlatformTarget, SourceAdmissionLimits,
    },
    replay::ProjectReplay,
    xml_bindings::{
        PACKAGE_XML_LUA_BINDING_PROFILE, XmlLuaBindingKind, XmlLuaBindingState,
        XmlReceiverBlockerKind,
    },
};

type TestResult = Result<(), Box<dyn Error>>;
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);
const PACKAGES: [&str; 3] = ["Alpha", "Beta", "Empty"];
const TOC: &str = "## Interface: 120100\ndefs.lua\nframes.xml\n";
const SHARED_XML: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
  <Frame name="Template" virtual="true">
    <Scripts><OnShow function="UniqueHandler"/></Scripts>
  </Frame>
  <Frame name="Receiver">
    <Scripts>
      <OnLoad function="SharedHandler"/>
      <OnClick function="UniqueHandler"/>
      <OnEvent function="BadHandler()"/>
    </Scripts>
  </Frame>
  <Frame name="Inherited" inherits="Template,MissingTemplate"/>
</Ui>
"#;
const EMPTY_XML: &str =
    "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Frame name=\"EmptyReceiver\"/></Ui>\n";

struct FixtureRoot(PathBuf);
impl FixtureRoot {
    fn new() -> Result<Self, Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-package-xml-bindings-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        for (package, lua, xml) in [
            (
                "Alpha",
                "function SharedHandler(self) return self end\nfunction UniqueHandler(self) return self end\n",
                SHARED_XML,
            ),
            (
                "Beta",
                "function SharedHandler(self) return self end\n",
                SHARED_XML,
            ),
            ("Empty", "local empty_package = true\n", EMPTY_XML),
        ] {
            let directory = root.join("UI").join(package);
            std::fs::create_dir_all(&directory)?;
            std::fs::write(directory.join("Fixture.toc"), TOC)?;
            std::fs::write(directory.join("defs.lua"), lua)?;
            std::fs::write(directory.join("frames.xml"), xml)?;
        }
        Ok(Self(root))
    }

    fn packages(
        &self,
        stop: &AtomicBool,
    ) -> Result<Arc<PlatformPackageSpecialization>, Box<dyn Error>> {
        self.packages_with_selection(None, stop)
    }

    fn packages_with_selection(
        &self,
        unselected_root: Option<&str>,
        stop: &AtomicBool,
    ) -> Result<Arc<PlatformPackageSpecialization>, Box<dyn Error>> {
        let profile = BlizzardUiSourceProfile::new(BlizzardUiSourceProfileRequest {
            profile_id: "profile:fixture:package-xml-bindings-v1".parse()?,
            source_class: PlatformSourceClass::SyntheticFixture,
            target: target()?,
            roots: vec![PlatformRootSpec {
                root: "UI".into(),
                selected_tocs: PACKAGES
                    .iter()
                    .map(|package| format!("UI/{package}/Fixture.toc"))
                    .collect(),
            }],
            exclusions: Vec::new(),
            limits: SourceAdmissionLimits {
                max_entries: 16,
                max_total_bytes: 1024 * 1024,
                max_file_bytes: 1024 * 1024,
                max_manifest_bytes: 32 * 1024,
            },
        })?;
        let mut entries = Vec::new();
        for package in PACKAGES {
            for (name, kind) in [
                ("Fixture.toc", PlatformFileKind::Toc),
                ("defs.lua", PlatformFileKind::Lua),
                ("frames.xml", PlatformFileKind::Xml),
            ] {
                let path = format!("UI/{package}/{name}");
                let bytes = std::fs::read(self.0.join(&path))?;
                entries.push(PlatformInventoryEntry {
                    path,
                    kind,
                    disposition: PlatformEntryDisposition::Included {
                        digest: raw_digest(&bytes),
                        byte_length: u64::try_from(bytes.len())?,
                        object_id: None,
                    },
                });
            }
        }
        let inventory = PlatformSourceInventory {
            schema: "wow-project/platform-source-inventory/1".into(),
            profile_digest: profile.digest(),
            target: profile.target().clone(),
            origin: PlatformSourceOrigin {
                provider: "handwritten-fixture".into(),
                repository: "native-package-xml-input".into(),
                revision: PlatformSourceRevision::Fixture {
                    digest: raw_digest(b"handwritten package XML fixture revision"),
                },
            },
            materializer: PlatformMaterializer {
                producer: "wow.fixture_materializer".parse()?,
                version: "1.0.0".parse()?,
                configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
                report_digest: raw_digest(b"fixture declaration, not source attestation"),
            },
            roots: vec![PlatformRootInventory {
                root: "UI".into(),
                declared_entries: u64::try_from(entries.len())?,
                scope: PlatformInventoryScope::DeclaredPartial,
                evidence_digest: raw_digest(b"fixture root accounting assertion"),
            }],
            entries,
            license: PlatformLicenseRecord {
                state: PlatformLicenseState::Unknown,
                attribution: "project-owned synthetic test input".into(),
                evidence_digest: raw_digest(b"local fixture notice"),
            },
            compatibility_evidence: raw_digest(b"caller fixture compatibility assertion"),
        };
        let source = Arc::new(
            ProjectInputDirectory::open(&self.0)?
                .admit_platform_source(&profile, inventory, stop)?,
        );
        assert_eq!(
            source.source_bytes("UI/Alpha/frames.xml")?,
            source.source_bytes("UI/Beta/frames.xml")?
        );
        std::fs::remove_dir_all(&self.0)?;
        assert!(!self.0.exists());
        let declarations = PACKAGES
            .iter()
            .map(|package| {
                ProjectPackageInput::new(
                    *package,
                    format!("UI/{package}"),
                    Some(*package) != unselected_root,
                    vec![ProjectPackageVariantInput::new(
                        ProjectDiskFile::new("Fixture.toc"),
                        true,
                    )],
                )
            })
            .collect::<Vec<_>>();
        Ok(Arc::new(source.specialize_packages(
            &declarations,
            None,
            stop,
        )?))
    }
}
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn raw_digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}

fn target() -> Result<PlatformTarget, Box<dyn Error>> {
    let reference_profile = ProfileIdentityBuilder::new(
        "profile:fixture:package-xml-reference-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "package-xml-handwritten-native-fixture-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([2; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:package-xml-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("package-xml-bindings-local-fixture")
    .build()?;
    Ok(PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile,
        reference_generation: ReferenceGenerationId::from_hash([3; 32]),
    })
}

fn configuration_builder(
    kind: ProjectKind,
    target: &PlatformTarget,
) -> Result<ProjectConfigurationBuilder, Box<dyn Error>> {
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let analyzer = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    Ok(ProjectConfigurationBuilder::new(
        ProjectId::new("package-xml-bindings-fixture")?,
        kind,
        target.reference_profile.clone(),
        target.reference_generation,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new(
        "workspace:main:package-xml-fixture",
    )?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:package-xml-fixture",
    )?)
    .logical_root("fixtures/package-xml/UI")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?))
}

fn publish(
    packages: &Arc<PlatformPackageSpecialization>,
    selected: Option<PackageXmlBindingProfile>,
    stop: &AtomicBool,
) -> Result<ProjectPublisher, Box<dyn Error>> {
    publish_graph(packages, selected, false, stop)
}

fn publish_graph(
    packages: &Arc<PlatformPackageSpecialization>,
    selected: Option<PackageXmlBindingProfile>,
    graph: bool,
    stop: &AtomicBool,
) -> Result<ProjectPublisher, Box<dyn Error>> {
    let builder = configuration_builder(
        ProjectKind::BlizzardUiPlatformSource,
        packages.source().profile().target(),
    )?
    .platform_packages(Arc::clone(packages))?;
    let builder = match selected {
        Some(profile) => builder.with_package_xml_bindings(profile),
        None => builder,
    };
    let configuration = if graph {
        builder.with_platform_graph_profile(wow_project::PlatformGraphProfile::PackageProjectionV1)
    } else {
        builder
    }
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/package-xml-fixture.lua",
            "---@meta _\nPackageXmlLibraryFixture = {}\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(
        ProjectInputBundle::closed(configuration, packages.files().to_vec(), vec![library])?,
        stop,
    )?;
    Ok(publisher)
}

#[test]
fn selected_package_graph_preserves_local_ids_shared_globals_and_native_replay() -> TestResult {
    use wow_project::graph::{
        ProjectGraphMixinOutcome, ProjectGraphScriptQueryOutcome, ProjectGraphStateOutcome,
        build_source_graph_proposals,
    };
    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    let xml = SHARED_XML
        .replace(
            "name=\"Receiver\"",
            "name=\"Receiver\" mixin=\"UniqueMixin\"",
        )
        .replace(
            "<OnClick function=\"UniqueHandler\"/>",
            "<OnClick function=\"UniqueHandler\"/><OnHide>self:Hide()</OnHide>",
        );
    for package in ["Alpha", "Beta"] {
        let directory = root.0.join("UI").join(package);
        std::fs::write(directory.join("frames.xml"), &xml)?;
        let declarations = if package == "Alpha" {
            "## SavedVariables: SharedState, AlphaState\n"
        } else {
            "## SavedVariables: SharedState\n"
        };
        let partial_toc = TOC.replace("defs.lua\n", "defs.lua\nmissing.lua\n");
        std::fs::write(
            directory.join("Fixture.toc"),
            format!("{declarations}{partial_toc}"),
        )?;
    }
    let lua = root.0.join("UI/Alpha/defs.lua");
    let mut text = std::fs::read_to_string(&lua)?;
    text.push_str("UniqueMixin = {}\nAlphaState = {}\nSharedState = {}\nfunction ReadState() return AlphaState.value, SharedState.value end\n");
    std::fs::write(lua, text)?;
    std::fs::write(
        root.0.join("UI/Empty/frames.xml"),
        "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Script>local captured_chunk = true</Script><Frame name=\"UnreachableReceiver\"><Scripts><OnShow>self:Show()</OnShow></Scripts></Frame></Ui>\n",
    )?;
    let packages = root.packages_with_selection(Some("Empty"), &stop)?;
    assert!(!root.0.exists());
    assert!(publish_graph(&packages, None, true, &stop).is_err());
    let old = publish(
        &packages,
        Some(PackageXmlBindingProfile::SameSessionV1),
        &stop,
    )?;
    let selected = publish_graph(
        &packages,
        Some(PackageXmlBindingProfile::SameSessionV1),
        true,
        &stop,
    )?;
    let old_view = old.open_current()?;
    let view = selected.open_current()?;
    assert_ne!(view.snapshot_id(), old_view.snapshot_id());
    let old_graph = build_source_graph_proposals(&old_view, &stop)?;
    let (_, _, _, old_provenance, _) = old_graph.into_parts();
    assert_eq!(
        serde_json::to_value(&old_provenance)?["profile"],
        wow_project::graph::SOURCE_GRAPH_PROFILE
    );
    assert!(old_provenance.xml_declarations().is_empty());
    let (registry, batch, coverage, provenance, limits) =
        build_source_graph_proposals(&view, &stop)?.into_parts();
    let wire = serde_json::to_value(&provenance)?;
    assert_eq!(
        wire["profile"],
        wow_project::graph::PACKAGE_SOURCE_GRAPH_PROFILE
    );
    assert!(wire.get("xml_binding_report").is_none());
    assert!(wire.get("package_xml_binding_report").is_some());
    assert!(provenance.script_sites().iter().any(|site| {
        site.package.as_deref() == Some("Empty")
            && site.consumer_id.is_none()
            && site.blockers.contains(&"package_unreachable")
            && site.blockers.contains(&"owner_not_captured")
    }));
    assert!(provenance.script_sites().iter().any(|site| {
        site.package.as_deref() == Some("Empty")
            && site.consumer_id.is_some()
            && site.blockers.contains(&"package_unreachable")
            && !site.blockers.contains(&"inline_parse_failed")
    }));
    assert!(provenance.package_loads().iter().any(|load| load.outcome
        == wow_project::graph::ProjectGraphPackageLoadOutcome::SourceNotRegistered));
    let alpha_plan = packages
        .load_plan()
        .package_plan("Alpha")
        .ok_or("alpha plan missing")?;
    let receiver = alpha_plan
        .xml_references()
        .declarations()
        .values()
        .find(|row| row.name.as_deref() == Some("Receiver"))
        .ok_or("receiver missing")?;
    let matching = provenance
        .xml_declarations()
        .iter()
        .filter(|row| row.occurrence_id == receiver.occurrence_id)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 2);
    assert_eq!(matching[0].occurrence_id, matching[1].occurrence_id);
    assert_ne!(matching[0].path, matching[1].path);
    assert_ne!(matching[0].proposal_id, matching[1].proposal_id);
    assert_eq!(provenance.inline_handlers().len(), 2);
    assert_ne!(
        provenance.inline_handlers()[0].unit_id,
        provenance.inline_handlers()[1].unit_id
    );
    assert_eq!(provenance.xml_mixins().len(), 2);
    assert!(
        provenance
            .xml_mixins()
            .iter()
            .any(|row| matches!(row.outcome, ProjectGraphMixinOutcome::Projected { .. }))
    );
    assert!(
        provenance
            .xml_mixins()
            .iter()
            .any(|row| row.outcome == ProjectGraphMixinOutcome::LoadOrderUnresolved)
    );
    let bindings = view
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("package bindings missing")?;
    for reference in provenance.xml_mixins() {
        let address = reference
            .package_binding_address
            .as_ref()
            .ok_or("native binding address missing")?;
        assert_eq!(
            bindings.resolve_binding(address)?.kind,
            XmlLuaBindingKind::Mixin
        );
    }
    assert!(
        provenance
            .script_sites()
            .iter()
            .flat_map(|site| &site.queries)
            .any(|query| query.outcome == ProjectGraphScriptQueryOutcome::LoadOrderUnresolved)
    );
    let shared = provenance
        .state_roots()
        .iter()
        .filter(|row| row.name == "SharedState")
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2);
    assert!(shared.iter().all(|row| row.ambiguous));
    assert_ne!(shared[0].root_id, shared[1].root_id);
    assert!(
        provenance
            .state_sites()
            .iter()
            .any(|row| row.outcome == ProjectGraphStateOutcome::AmbiguousDeclaration)
    );
    let unique = provenance
        .state_roots()
        .iter()
        .find(|row| row.name == "AlphaState")
        .ok_or("unique state root missing")?;
    assert!(!unique.ambiguous);
    assert!(
        provenance
            .state_bindings()
            .iter()
            .any(|row| row.root_id == unique.root_id)
    );
    let foundation = wow_graph::GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let graph = wow_graph::GraphPartitionSnapshot::new(
        registry,
        foundation,
        batch.source_context_id(),
        &stop,
    )?;
    let prepared = graph.prepare_replacement(
        wow_graph::GraphPartitionReplacement {
            expected_snapshot_id: graph.snapshot().snapshot_id().clone(),
            expected_partition_digest: None,
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch: batch.clone(),
            coverage: coverage.clone(),
        },
        &stop,
    )?;
    let graph = prepared.candidate();
    let decode =
        || serde_json::from_value::<wow_project::graph::RetainedProjectGraphEvidence>(wire.clone());
    assert!(decode()?.admit(graph, &stop).is_err());
    let (catalog, _) = decode()?.admit_with_platform_profile(
        graph,
        wow_project::PlatformGraphProfile::PackageProjectionV1,
        &stop,
    )?;
    assert_eq!(
        catalog.context().project_generation(),
        Some(view.project_generation())
    );
    assert!(wow_project::graph::persistence::records(&wire, graph, &stop).is_err());
    assert!(
        !wow_project::graph::persistence::records_with_platform_profile(
            &wire,
            graph,
            wow_project::PlatformGraphProfile::PackageProjectionV1,
            &stop,
        )?
        .is_empty()
    );
    let archive = ProjectReplay::capture(&selected, &stop)?;
    let replay_wire = serde_json::to_value(&archive)?;
    assert_eq!(replay_wire["schema"], "wow-project/native-project-replay/7");
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&archive)?, &stop)?.hydrate(&stop)?;
    let (_, restored_batch, restored_coverage, restored_provenance, _) =
        build_source_graph_proposals(&restored, &stop)?.into_parts();
    assert_eq!(restored_batch, batch);
    assert_eq!(restored_coverage, coverage);
    assert_eq!(restored_provenance, provenance);
    let mut wrong_recipe = replay_wire;
    wrong_recipe["schema"] = serde_json::json!("wow-project/native-project-replay/6");
    assert!(ProjectReplay::from_json(&serde_json::to_vec(&wrong_recipe)?, &stop).is_err());
    let cancelled = AtomicBool::new(true);
    assert!(build_source_graph_proposals(&view, &cancelled).is_err());
    Ok(())
}

#[test]
fn package_binding_selection_requires_native_owner_and_preserves_legacy_replay() -> TestResult {
    let stop = AtomicBool::new(false);
    let fixture_target = target()?;
    let ordinary = configuration_builder(ProjectKind::Fixture, &fixture_target)?;
    assert!(
        ordinary
            .clone()
            .build()?
            .package_xml_binding_profile()
            .is_none()
    );
    assert!(
        ordinary
            .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
            .build()
            .is_err()
    );
    assert!(
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, &fixture_target)?
            .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
            .build()
            .is_err()
    );

    let root = FixtureRoot::new()?;
    let packages = root.packages(&stop)?;
    let publisher = publish(&packages, None, &stop)?;
    let view = publisher.open_current()?;
    assert!(
        view.snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .is_none()
    );
    assert!(view.snapshot().analyzer_binding().xml_bindings().is_none());
    assert!(view.configuration().package_xml_binding_profile().is_none());
    let archive = ProjectReplay::capture(&publisher, &stop)?;
    let encoded = serde_json::to_vec(&archive)?;
    let wire: serde_json::Value = serde_json::from_slice(&encoded)?;
    assert_eq!(wire["schema"], "wow-project/native-project-replay/5");
    assert!(
        wire["configuration"]
            .get("package_xml_binding_profile")
            .is_none()
    );
    let restored = ProjectReplay::from_json(&encoded, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(restored.configuration(), view.configuration());
    assert!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .is_none()
    );
    assert!(!root.0.exists());

    let frozen: serde_json::Value =
        serde_json::from_str(include_str!("data/native-replay-legacy/packages.json"))?;
    assert_eq!(frozen["schema"], "wow-project/native-project-replay/3");
    let frozen_view =
        ProjectReplay::from_json(&serde_json::to_vec(&frozen)?, &stop)?.hydrate(&stop)?;
    assert_eq!(
        frozen_view.snapshot_id(),
        frozen["project_snapshot_id"]
            .as_str()
            .ok_or("frozen project ID missing")?
    );
    assert_eq!(
        frozen_view.analyzer_snapshot_id(),
        frozen["analyzer_snapshot_id"]
            .as_str()
            .ok_or("frozen analyzer ID missing")?
    );
    assert!(
        frozen_view
            .configuration()
            .package_xml_binding_profile()
            .is_none()
    );
    assert!(
        frozen_view
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .is_none()
    );
    Ok(())
}

#[test]
fn package_xml_bindings_keep_local_scopes_and_one_shared_native_lookup_in_replay() -> TestResult {
    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    let packages = root.packages(&stop)?;
    let publisher = publish(
        &packages,
        Some(PackageXmlBindingProfile::SameSessionV1),
        &stop,
    )?;
    let view = publisher.open_current()?;
    let analyzer = view.snapshot().analyzer_binding();
    let bindings = analyzer
        .package_xml_bindings()
        .ok_or("selected package binding owner missing")?;
    assert!(analyzer.xml_bindings().is_none());
    assert_eq!(bindings.profile(), PACKAGE_XML_LUA_BINDING_PROFILE);
    assert_eq!(bindings.project_generation(), view.project_generation());
    assert_eq!(
        bindings.package_load_plan_digest(),
        packages.load_plan().digest()
    );
    assert_eq!(
        bindings.package_main_plan_digest(),
        packages.main_plan().digest()
    );
    assert_eq!(
        bindings.main_snapshot_id(),
        analyzer.main_workspace().snapshot_id()
    );
    assert_eq!(
        bindings.library_snapshot_ids().collect::<Vec<_>>(),
        analyzer.library_snapshot_ids().collect::<Vec<_>>()
    );
    assert_eq!(
        bindings
            .groups()
            .iter()
            .map(|group| group.scope().package())
            .collect::<Vec<_>>(),
        PACKAGES
    );
    let lookup = bindings
        .symbol_lookup()
        .ok_or("nonempty union lookup missing")?;
    assert!(lookup.source_health_complete());
    assert_eq!(
        lookup
            .lookups()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["SharedHandler", "UniqueHandler"]
    );
    let shared = lookup
        .lookups()
        .get("SharedHandler")
        .ok_or("shared query missing")?;
    assert_eq!(shared.state, SymbolLookupState::Ambiguous);
    assert_eq!(shared.targets.len(), 2);
    assert_eq!(
        shared
            .targets
            .iter()
            .map(|target| target.path.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["packages/Alpha/defs.lua", "packages/Beta/defs.lua"])
    );
    for target in &shared.targets {
        assert_eq!(target.role, "main");
        assert_eq!(target.workspace_id, analyzer.main_workspace().snapshot_id());
        let file = analyzer
            .main_workspace()
            .file(&target.path)
            .ok_or("shared target file missing")?;
        assert_eq!(target.content_digest, file.content_sha256());
        assert!(
            target
                .span
                .byte_end()
                .is_some_and(|end| end <= file.text().len() as u64)
        );
    }

    for package in ["Alpha", "Beta"] {
        let group = bindings
            .group(package)
            .ok_or("named package group missing")?;
        let plan = packages
            .load_plan()
            .package_plan(package)
            .ok_or("native package plan missing")?;
        let node = packages
            .load_plan()
            .packages()
            .iter()
            .find(|node| node.package == package)
            .ok_or("native package node missing")?;
        assert_eq!(group.scope().package(), package);
        assert_eq!(group.scope().selected_toc(), plan.selected_toc());
        assert_eq!(group.scope().load_plan_digest(), plan.digest());
        assert_eq!(group.scope().reachability(), node.reachability);
        assert_eq!(group.scope().phase(), node.phase);
        assert_eq!(group.documents().len(), 1);
        let document = group
            .documents()
            .get("frames.xml")
            .ok_or("local XML mapping missing")?;
        assert_eq!(
            document.qualified_document(),
            format!("packages/{package}/frames.xml")
        );
        assert_eq!(document.content_digest(), raw_digest(SHARED_XML.as_bytes()));
        assert_eq!(document.byte_length(), u64::try_from(SHARED_XML.len())?);
        assert_eq!(group.bindings().len(), 5);
        for row in group.bindings() {
            assert_eq!(row.document, "frames.xml");
            assert_eq!(row.content_digest, document.content_digest());
            assert_eq!(row.kind, XmlLuaBindingKind::Function);
        }
        let ambiguous = group
            .bindings()
            .iter()
            .find(|row| row.queries == ["SharedHandler"])
            .ok_or("shared local binding missing")?;
        assert_eq!(ambiguous.state, XmlLuaBindingState::Ambiguous);
        let direct = group
            .bindings()
            .iter()
            .find(|row| row.queries == ["UniqueHandler"] && row.consumer_id.is_none())
            .ok_or("direct native declaration binding missing")?;
        assert_eq!(direct.state, XmlLuaBindingState::UniqueAnalyzerDeclaration);
        let malformed = group
            .bindings()
            .iter()
            .find(|row| row.state == XmlLuaBindingState::UnsupportedPath)
            .ok_or("malformed spelling was discarded")?;
        assert!(malformed.queries.is_empty());
        assert!(malformed.consumer_id.is_none());
        assert!(!group.inherited_sources_complete());
        let [inherited] = group.inherited_script_sources() else {
            return Err("expected one retained inherited handler source".into());
        };
        assert!(!inherited.source_complete);
        assert_eq!(inherited.binding_indices.len(), 1);
        let sources = group
            .receiver_sources()
            .get(&inherited.consumer_id)
            .ok_or("inherited receiver source missing")?;
        assert!(!sources.complete);
        assert!(
            sources
                .blockers
                .iter()
                .any(|blocker| blocker.reason == XmlReceiverBlockerKind::UnresolvedInheritance)
        );
        let row = group
            .bindings()
            .get(inherited.binding_indices[0])
            .ok_or("inherited local index is invalid")?;
        assert_eq!(
            row.consumer_id.as_deref(),
            Some(inherited.consumer_id.as_str())
        );
        assert_eq!(row.queries, ["UniqueHandler"]);
        assert_eq!(row.state, XmlLuaBindingState::ReceiverNotResolved);
    }
    let alpha = bindings.group("Alpha").ok_or("Alpha group missing")?;
    let beta = bindings.group("Beta").ok_or("Beta group missing")?;
    assert_eq!(alpha.bindings(), beta.bindings());
    let alpha_address = bindings.address("Alpha", 0)?;
    let beta_address = bindings.address("Beta", 0)?;
    assert_ne!(alpha_address, beta_address);
    assert_eq!(alpha_address.analysis_id(), bindings.analysis_id());
    assert_eq!(alpha_address.package(), "Alpha");
    assert_eq!(
        alpha_address.load_plan_digest(),
        alpha.scope().load_plan_digest()
    );
    assert_eq!(alpha_address.binding_index(), 0);
    assert!(std::ptr::eq(
        bindings.resolve_binding(&alpha_address)?,
        &alpha.bindings()[0]
    ));
    assert!(std::ptr::eq(
        bindings.resolve_binding(&beta_address)?,
        &beta.bindings()[0]
    ));
    let empty = bindings.group("Empty").ok_or("empty group missing")?;
    assert!(empty.bindings().is_empty());
    assert!(empty.receiver_sources().is_empty());
    assert!(empty.inherited_script_sources().is_empty());
    assert_eq!(empty.documents().len(), 1);
    assert!(bindings.address("Empty", 0).is_err());

    let unique = lookup
        .lookups()
        .get("UniqueHandler")
        .ok_or("unique query missing")?;
    assert_eq!(unique.state, SymbolLookupState::UniqueAnalyzerDeclaration);
    assert_eq!(unique.targets.len(), 1);
    assert_eq!(unique.targets[0].path, "packages/Alpha/defs.lua");
    let calls = analyzer
        .function_call_report()
        .ok_or("native callable report missing")?;
    assert_eq!(calls.main_snapshot_id(), bindings.main_snapshot_id());
    assert_eq!(
        calls
            .library_snapshot_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        bindings.library_snapshot_ids().collect::<Vec<_>>()
    );
    assert_eq!(calls.symbol_lookup_analysis_ids().len(), 2);
    assert!(
        calls
            .symbol_lookup_analysis_ids()
            .iter()
            .any(|id| id == lookup.analysis_id())
    );
    assert!(!calls.named_targets().contains_key("SharedHandler"));
    let Some(SourceCallTarget::MainFunction { function_id }) =
        calls.named_targets().get("UniqueHandler")
    else {
        return Err("unique handler has no captured native callable".into());
    };
    let function = calls
        .functions()
        .iter()
        .find(|function| function.fact_id() == function_id)
        .ok_or("callable closure receipt missing")?;
    assert_eq!(function.path(), "packages/Alpha/defs.lua");
    assert_eq!(function.kind(), SourceFunctionKind::Closure);
    assert_eq!(function.content_digest(), unique.targets[0].content_digest);
    assert_eq!(
        bindings.serialized_byte_length(),
        serde_json::to_vec(bindings)?.len()
    );

    let archive = ProjectReplay::capture(&publisher, &stop)?;
    let encoded = serde_json::to_vec(&archive)?;
    let wire: serde_json::Value = serde_json::from_slice(&encoded)?;
    assert_eq!(wire["schema"], "wow-project/native-project-replay/6");
    assert_eq!(
        wire["configuration"]["package_xml_binding_profile"],
        "same_session_v1"
    );
    let decoded = ProjectReplay::from_json(&encoded, &stop)?;
    assert_eq!(decoded, archive);
    let restored = decoded.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(restored.configuration(), view.configuration());
    let restored_analyzer = restored.snapshot().analyzer_binding();
    assert_eq!(restored_analyzer.package_xml_bindings(), Some(bindings));
    assert_eq!(restored_analyzer.function_call_report(), Some(calls));
    let restored_packages = restored
        .configuration()
        .platform_packages()
        .ok_or("replayed platform owner missing")?;
    assert_eq!(
        restored_packages.source().receipt(),
        packages.source().receipt()
    );
    assert_eq!(restored_packages.binding(), packages.binding());
    assert_eq!(
        restored_packages
            .source()
            .source_bytes("UI/Alpha/frames.xml")?,
        SHARED_XML.as_bytes()
    );
    assert_eq!(
        restored_packages
            .source()
            .source_bytes("UI/Beta/frames.xml")?,
        SHARED_XML.as_bytes()
    );
    assert!(!root.0.exists());

    let mut wrong_schema = wire.clone();
    wrong_schema["schema"] = serde_json::json!("wow-project/native-project-replay/5");
    let mut missing_selector = wire.clone();
    missing_selector["configuration"]
        .as_object_mut()
        .ok_or("archive configuration missing")?
        .remove("package_xml_binding_profile");
    let mut unknown_selector = wire;
    unknown_selector["configuration"]["package_xml_binding_profile"] = serde_json::json!("unknown");
    for changed in [wrong_schema, missing_selector, unknown_selector] {
        assert!(
            ProjectReplay::from_json(&serde_json::to_vec(&changed)?, &stop)
                .and_then(|replay| replay.hydrate(&stop))
                .is_err()
        );
    }
    Ok(())
}
