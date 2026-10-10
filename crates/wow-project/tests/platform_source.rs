use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, CoverageStatus, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
    SourceOriginKind, SourceSpan,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyMemberCallErrorCode, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectErrorCode, ProjectId, ProjectInputBundle, ProjectKind,
    ProjectPhase, ProjectPublisher, ProjectSourceOriginId, ProjectSourceOriginKind,
    ProjectWorkspaceId,
    disk::ProjectInputDirectory,
    load::{ProjectPackageInput, ProjectPackageVariantInput},
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformRootInventory, PlatformRootSpec,
        PlatformSourceClass, PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision,
        PlatformSpecialEntry, PlatformTarget, PlatformUnevaluatedCapability, ProfileExclusion,
        SourceAdmissionLimits,
    },
};

type TestResult = Result<(), Box<dyn Error>>;
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);
struct FixtureRoot(PathBuf);
impl FixtureRoot {
    fn new() -> Result<Self, Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-platform-source-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        std::fs::create_dir(root.join("UI"))?;
        let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/xml-facts");
        for name in ["Fixture.toc", "frames.xml", "defs.lua"] {
            std::fs::copy(corpus.join(name), root.join("UI").join(name))?;
        }
        std::fs::write(root.join("UI/opaque.bin"), [0xff, 0xfe, 0, 1])?;
        Ok(Self(root))
    }
    fn directory(&self) -> Result<ProjectInputDirectory, Box<dyn Error>> {
        Ok(ProjectInputDirectory::open(&self.0)?)
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
        "profile:fixture:platform-source-native-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "platform-source-handwritten-native-fixture-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([2; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:platform-source-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("platform-source-admission-local-fixture")
    .build()?;
    Ok(PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile,
        reference_generation: ReferenceGenerationId::from_hash([3; 32]),
    })
}
fn profile_request() -> Result<BlizzardUiSourceProfileRequest, Box<dyn Error>> {
    Ok(BlizzardUiSourceProfileRequest {
        profile_id: "profile:fixture:platform-source-admission-v1".parse()?,
        source_class: PlatformSourceClass::SyntheticFixture,
        target: target()?,
        roots: vec![PlatformRootSpec {
            root: "UI".into(),
            selected_tocs: vec!["UI/Fixture.toc".into()],
        }],
        exclusions: vec![ProfileExclusion {
            path: "UI/omitted.txt".into(),
        }],
        limits: SourceAdmissionLimits {
            max_entries: 16,
            max_total_bytes: 1024 * 1024,
            max_file_bytes: 1024 * 1024,
            max_manifest_bytes: 32 * 1024,
        },
    })
}
fn inventory(
    root: &FixtureRoot,
    profile: &BlizzardUiSourceProfile,
) -> Result<PlatformSourceInventory, Box<dyn Error>> {
    let mut entries = Vec::new();
    for (name, kind) in [
        ("Fixture.toc", PlatformFileKind::Toc),
        ("frames.xml", PlatformFileKind::Xml),
        ("defs.lua", PlatformFileKind::Lua),
        ("opaque.bin", PlatformFileKind::Unknown),
    ] {
        let bytes = std::fs::read(root.0.join("UI").join(name))?;
        entries.push(PlatformInventoryEntry {
            path: format!("UI/{name}"),
            kind,
            disposition: PlatformEntryDisposition::Included {
                digest: raw_digest(&bytes),
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
    entries.push(PlatformInventoryEntry {
        path: "UI/linked.lua".into(),
        kind: PlatformFileKind::Lua,
        disposition: PlatformEntryDisposition::Unsupported {
            kind: PlatformSpecialEntry::Symlink,
            evidence_digest: raw_digest(b"explicit unsupported fixture record"),
        },
    });
    Ok(PlatformSourceInventory {
        schema: "wow-project/platform-source-inventory/1".into(),
        profile_digest: profile.digest(),
        target: profile.target().clone(),
        origin: PlatformSourceOrigin {
            provider: "handwritten-fixture".into(),
            repository: "native-platform-input".into(),
            revision: PlatformSourceRevision::Fixture {
                digest: raw_digest(b"handwritten-fixture-revision"),
            },
        },
        materializer: PlatformMaterializer {
            producer: "wow.fixture_materializer".parse()?,
            version: "1.0.0".parse()?,
            configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
            report_digest: raw_digest(b"fixture materialization declaration, not attestation"),
        },
        roots: vec![PlatformRootInventory {
            root: "UI".into(),
            declared_entries: entries.len() as u64,
            scope: PlatformInventoryScope::DeclaredPartial,
            evidence_digest: raw_digest(b"fixture root accounting assertion"),
        }],
        entries,
        license: PlatformLicenseRecord {
            state: PlatformLicenseState::Unknown,
            attribution: "project-owned synthetic test input".into(),
            evidence_digest: raw_digest(b"local-only fixture notice"),
        },
        compatibility_evidence: raw_digest(b"caller fixture compatibility assertion"),
    })
}

fn fixture_packages(name: &str) -> Vec<ProjectPackageInput> {
    vec![ProjectPackageInput::new(
        name,
        "UI",
        true,
        vec![ProjectPackageVariantInput::new(
            wow_project::disk::ProjectDiskFile::new("Fixture.toc"),
            true,
        )],
    )]
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
        ProjectId::new("platform-source-lifecycle-fixture")?,
        kind,
        target.reference_profile.clone(),
        target.reference_generation,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new(
        "workspace:main:platform-source-fixture",
    )?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:platform-source-fixture",
    )?)
    .logical_root("fixtures/platform-source/UI")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?))
}

#[test]
fn native_admission_retains_bytes_and_separate_identity_scopes() -> TestResult {
    let root = FixtureRoot::new()?;
    let copied = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let admitted = root
        .directory()?
        .admit_platform_source(&profile, declared.clone(), &stop)?;
    let mut reordered = declared.clone();
    reordered.entries.reverse();
    let independent = copied
        .directory()?
        .admit_platform_source(&profile, reordered, &stop)?;
    assert_eq!(admitted.receipt(), independent.receipt());
    assert_eq!(admitted.source_bytes("UI/opaque.bin")?, &[0xff, 0xfe, 0, 1]);
    assert_eq!(admitted.receipt().coverage().verified_files(), 4);
    assert_eq!(
        admitted.receipt().coverage().inventory(),
        CoverageStatus::Partial
    );
    assert_eq!(
        admitted.receipt().coverage().declared_included_bytes(),
        CoverageStatus::Complete
    );
    assert!(
        admitted
            .receipt()
            .coverage()
            .unevaluated()
            .contains(&PlatformUnevaluatedCapability::Runtime)
    );
    assert!(
        admitted
            .receipt()
            .coverage()
            .unevaluated()
            .contains(&PlatformUnevaluatedCapability::RootCompleteness)
    );
    assert_eq!(admitted.receipt().inventory().entries.len(), 6);
    assert_eq!(
        admitted
            .source_bytes("UI/linked.lua")
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::FileNotPresent)
    );
    let original = admitted.source_bytes("UI/defs.lua")?.to_vec();
    std::fs::write(root.0.join("UI/defs.lua"), b"changed after admission")?;
    assert_eq!(admitted.source_bytes("UI/defs.lua")?, original);
    let mut renamed = declared.clone();
    renamed.origin.provider = "renamed-display-provider".into();
    let display_rename = copied
        .directory()?
        .admit_platform_source(&profile, renamed, &stop)?;
    assert_eq!(
        admitted.receipt().source_snapshot_id(),
        display_rename.receipt().source_snapshot_id()
    );
    assert_eq!(
        admitted.receipt().content_manifest_digest(),
        display_rename.receipt().content_manifest_digest()
    );
    assert_ne!(
        admitted.receipt().admission_digest(),
        display_rename.receipt().admission_digest()
    );
    let decoded =
        PlatformSourceInventory::from_json(&serde_json::to_vec(&declared)?, &profile, &stop)?;
    assert_eq!(decoded.entries, independent.receipt().inventory().entries);
    assert!(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)
            .is_err()
    );
    Ok(())
}

#[test]
fn native_admission_refuses_stale_binding_invalid_inventory_and_unresolved_content() -> TestResult {
    let root = FixtureRoot::new()?;
    let directory = root.directory()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let mut wrong = declared.clone();
    wrong.target.channel = "other-fixture".into();
    assert!(
        directory
            .admit_platform_source(&profile, wrong, &stop)
            .is_err()
    );
    let mut wrong = declared.clone();
    wrong.profile_digest = ContentDigest::from_bytes([9; 32]);
    assert!(
        directory
            .admit_platform_source(&profile, wrong, &stop)
            .is_err()
    );
    let mut collision = declared.clone();
    collision.entries[0].path = "UI/defs.lua".into();
    assert!(
        directory
            .admit_platform_source(&profile, collision, &stop)
            .is_err()
    );
    let mut escape = declared.clone();
    escape.entries[0].path = "UI/../outside.toc".into();
    assert!(
        directory
            .admit_platform_source(&profile, escape, &stop)
            .is_err()
    );
    let mut missing_accounting = declared.clone();
    missing_accounting.entries.pop();
    assert!(
        directory
            .admit_platform_source(&profile, missing_accounting, &stop)
            .is_err()
    );
    let mut limit_request = profile_request()?;
    limit_request.limits.max_entries = 1;
    let limited = BlizzardUiSourceProfile::new(limit_request)?;
    let mut too_many = declared.clone();
    too_many.profile_digest = limited.digest();
    assert_eq!(
        directory
            .admit_platform_source(&limited, too_many, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceBudgetExceeded)
    );
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        directory
            .admit_platform_source(&profile, declared.clone(), &cancelled)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceReadCancelled)
    );
    std::fs::remove_file(root.0.join("UI/defs.lua"))?;
    assert_eq!(
        directory
            .admit_platform_source(&profile, declared.clone(), &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::MissingDeclaredFile)
    );
    let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:fixture\nsize 1\n";
    std::fs::write(root.0.join("UI/defs.lua"), pointer)?;
    let mut unresolved = declared;
    let entry = unresolved
        .entries
        .iter_mut()
        .find(|entry| entry.path == "UI/defs.lua")
        .ok_or_else(|| std::io::Error::other("fixture Lua member missing"))?;
    entry.disposition = PlatformEntryDisposition::Included {
        digest: raw_digest(pointer),
        byte_length: pointer.len() as u64,
        object_id: None,
    };
    assert_eq!(
        directory
            .admit_platform_source(&profile, unresolved, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidInputInventory)
    );
    Ok(())
}

#[test]
fn platform_packages_reuse_native_load_from_retained_bytes_and_refuse_declared_omissions()
-> TestResult {
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let source = Arc::new(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)?,
    );
    let packages = fixture_packages("Fixture");
    // No filesystem input remains for the native TOC/XML/package owners to reread.
    std::fs::remove_dir_all(&root.0)?;
    let loaded = source.specialize_packages(&packages, None, &stop)?;
    assert!(Arc::ptr_eq(loaded.source(), &source));
    assert_eq!(loaded.files().len(), 1);
    assert_eq!(
        loaded.files()[0].relative_path().as_str(),
        "packages/Fixture/defs.lua"
    );
    assert_eq!(
        loaded.binding().source_snapshot_id(),
        source.receipt().source_snapshot_id()
    );
    assert_eq!(loaded.binding().load_digest(), loaded.load_plan().digest());
    assert_eq!(loaded.binding().main_digest(), loaded.main_plan().digest());
    assert_eq!(
        loaded.source().source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 1]
    );
    assert_eq!(
        loaded.source().receipt().coverage().inventory(),
        CoverageStatus::Partial
    );
    assert!(
        loaded
            .binding()
            .universe_id()
            .starts_with("blizzard_ui_source:")
    );

    let wrong_pin = vec![ProjectPackageInput::new(
        "Fixture",
        "UI",
        true,
        vec![ProjectPackageVariantInput::new(
            wow_project::disk::ProjectDiskFile::new("Fixture.toc").with_identity(
                ContentDigest::from_bytes([9; 32]),
                source.source_bytes("UI/Fixture.toc")?.len() as u64,
            ),
            true,
        )],
    )];
    assert_eq!(
        source
            .specialize_packages(&wrong_pin, None, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::FileDigestMismatch)
    );
    assert_eq!(
        source
            .specialize_packages(&packages, None, &AtomicBool::new(true))
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceReadCancelled)
    );

    let excluded_root = FixtureRoot::new()?;
    let mut exclusion_request = profile_request()?;
    exclusion_request.exclusions.push(ProfileExclusion {
        path: "UI/defs.lua".into(),
    });
    let exclusion_profile = BlizzardUiSourceProfile::new(exclusion_request)?;
    let mut excluded_inventory = inventory(&excluded_root, &exclusion_profile)?;
    let entry = excluded_inventory
        .entries
        .iter_mut()
        .find(|entry| entry.path == "UI/defs.lua")
        .ok_or_else(|| std::io::Error::other("fixture Lua record missing"))?;
    entry.disposition = PlatformEntryDisposition::Excluded {
        rule_path: "UI/defs.lua".into(),
    };
    let excluded = Arc::new(excluded_root.directory()?.admit_platform_source(
        &exclusion_profile,
        excluded_inventory,
        &stop,
    )?);
    let refused = excluded
        .specialize_packages(&packages, None, &stop)
        .err()
        .ok_or_else(|| std::io::Error::other("excluded load target was accepted"))?;
    assert_eq!(refused.code(), ProjectErrorCode::PackageTargetExcluded);
    assert_eq!(refused.relative_path(), Some("UI/defs.lua"));

    let unsupported_root = FixtureRoot::new()?;
    let toc_path = unsupported_root.0.join("UI/Fixture.toc");
    let mut toc = std::fs::read(&toc_path)?;
    toc.extend_from_slice(b"\nlinked.lua\n");
    std::fs::write(&toc_path, toc)?;
    let unsupported_inventory = inventory(&unsupported_root, &profile)?;
    let unsupported = Arc::new(unsupported_root.directory()?.admit_platform_source(
        &profile,
        unsupported_inventory,
        &stop,
    )?);
    assert_eq!(
        unsupported
            .specialize_packages(&packages, None, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidFileLanguage)
    );
    Ok(())
}

#[test]
fn platform_configuration_publishes_native_main_and_refuses_replay() -> TestResult {
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let source = Arc::new(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)?,
    );
    std::fs::remove_dir_all(&root.0)?;
    let loaded = Arc::new(source.specialize_packages(&fixture_packages("Fixture"), None, &stop)?);
    let competing =
        Arc::new(source.specialize_packages(&fixture_packages("Other"), None, &stop)?);

    let builder = configuration_builder(ProjectKind::BlizzardUiPlatformSource, profile.target())?;
    assert_eq!(
        builder.clone().build().err().map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    assert_eq!(
        builder
            .clone()
            .package_load_plan(loaded.load_plan(), loaded.main_plan())?
            .build()
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    for kind in [ProjectKind::Fixture, ProjectKind::Repository] {
        assert_eq!(
            configuration_builder(kind, profile.target())?
                .platform_packages(Arc::clone(&loaded))
                .err()
                .map(|error| error.code()),
            Some(ProjectErrorCode::InvalidConfiguration)
        );
    }
    let mut wrong_target = profile.target().clone();
    wrong_target.reference_generation = ReferenceGenerationId::from_hash([9; 32]);
    assert_eq!(
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, &wrong_target)?
            .platform_packages(Arc::clone(&loaded))
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    let builder = builder.platform_packages(Arc::clone(&loaded))?;
    let configuration = builder.clone().build()?;
    assert_eq!(
        builder
            .clone()
            .package_load_plan(loaded.load_plan(), loaded.main_plan())?
            .build()?,
        configuration
    );
    assert_eq!(
        builder
            .clone()
            .platform_packages(Arc::clone(&loaded))?
            .build()?,
        configuration
    );
    assert_eq!(
        builder
            .clone()
            .package_load_plan(competing.load_plan(), competing.main_plan())
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    assert_eq!(
        builder
            .clone()
            .platform_packages(competing)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    let single_plan = loaded
        .load_plan()
        .package_plan("Fixture")
        .ok_or_else(|| std::io::Error::other("fixture package receipt missing"))?;
    assert_eq!(
        builder
            .load_plan(single_plan)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );

    configuration.validate()?;
    assert_eq!(
        configuration.project_kind(),
        ProjectKind::BlizzardUiPlatformSource
    );
    assert_eq!(
        configuration.platform_package_binding(),
        Some(loaded.binding())
    );
    assert_eq!(
        configuration.platform_package_binding_digest(),
        Some(loaded.binding().binding_digest())
    );
    let retained = configuration
        .platform_packages()
        .ok_or_else(|| std::io::Error::other("platform owner missing"))?;
    assert!(Arc::ptr_eq(retained.source(), &source));
    assert_eq!(retained.files(), loaded.files());
    assert_eq!(configuration.package_load_plan(), Some(loaded.load_plan()));
    assert_eq!(configuration.package_main_plan(), Some(loaded.main_plan()));

    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::BlizzardUi,
        vec![LuaWorkspaceFileInput::new(
            "library/platform-fixture.lua",
            "---@meta _\n---@class C_PlatformFixture\n---@field KnownApi fun(): boolean\nC_PlatformFixture = {}\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let snapshot = publisher.publish_initial_cancellable(
        ProjectInputBundle::closed(
            configuration.clone(),
            loaded.files().to_vec(),
            vec![library.clone()],
        )?,
        &stop,
    )?;
    snapshot.validate()?;
    assert_eq!(snapshot.configuration(), &configuration);
    let analyzer = snapshot.analyzer_binding();
    assert_eq!(
        analyzer.main_workspace().universe(),
        LuaWorkspaceUniverse::BlizzardUiMain
    );
    assert_eq!(
        analyzer.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );
    let xml = analyzer
        .xml_lua_analysis()
        .ok_or_else(|| std::io::Error::other("XML virtual analysis missing"))?;
    assert!(!xml.units().is_empty());
    let mut virtual_files = Vec::new();
    for unit in xml.units() {
        assert_eq!(unit.package.as_deref(), Some("Fixture"));
        let index = single_plan
            .xml_documents()
            .iter()
            .find_map(|(path, index)| {
                (loaded.load_plan().source_path("Fixture", path).as_deref()
                    == Some(unit.document.as_str()))
                .then_some(index)
            })
            .ok_or_else(|| std::io::Error::other("virtual unit document missing"))?;
        let body = index
            .element(&unit.script_occurrence_id)
            .and_then(|element| element.script.as_ref())
            .and_then(|script| script.inline_lua.as_ref())
            .ok_or_else(|| std::io::Error::other("virtual unit source missing"))?;
        assert_eq!(unit.extracted_unit_id, body.unit_id);
        assert_eq!(unit.content_digest, body.content_digest);
        virtual_files.push(LuaWorkspaceFileInput::new(&unit.virtual_path, body.text()));
    }
    // Workspace identity includes the universe; use native unwrapped retained units.
    let virtual_limits = LuaWorkspaceLimits::new(64, 16_384, 256 * 1024, 2 * 1024 * 1024)?;
    let mismatched_virtual = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Project,
        virtual_files.clone(),
        virtual_limits,
    )?;
    let expected_virtual = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::BlizzardUiMain,
        virtual_files,
        virtual_limits,
    )?;
    assert_eq!(
        wow_emmy::analyze_member_calls(analyzer.main_workspace(), &[analyzer.main_workspace()])
            .err()
            .map(|error| error.code()),
        Some(EmmyMemberCallErrorCode::InvalidLibraryWorkspace)
    );
    assert_eq!(
        wow_emmy::references::analyze_member_call_session_with_virtual(
            analyzer.main_workspace(),
            &[&library],
            &mismatched_virtual,
            snapshot.project_generation(),
            &[],
            false,
            &stop,
        )
        .err()
        .map(|error| error.code()),
        Some(EmmyMemberCallErrorCode::InvalidMainWorkspace)
    );
    let semantics = xml
        .semantic_report()
        .ok_or_else(|| std::io::Error::other("XML virtual semantics missing"))?;
    assert_eq!(
        semantics.virtual_snapshot_id(),
        expected_virtual.snapshot_id()
    );
    assert_eq!(
        semantics.main_snapshot_id(),
        analyzer.main_workspace().snapshot_id()
    );
    assert_eq!(
        semantics.project_generation(),
        snapshot.project_generation()
    );
    assert_eq!(
        semantics.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );

    let registry = snapshot.source_registry();
    let origin = registry.source_origin();
    assert_eq!(
        origin.origin_kind(),
        ProjectSourceOriginKind::BlizzardUiPlatformSource
    );
    assert_eq!(
        origin.revision_identity(),
        loaded.binding().source_snapshot_id()
    );
    assert_eq!(origin.project_generation(), snapshot.project_generation());
    let view = publisher.open_current()?;
    for file in snapshot.file_manifest() {
        registry.validate_source_handle(file.source_handle_base())?;
        assert_eq!(
            file.source_handle_base().origin_kind(),
            SourceOriginKind::Fixture
        );
        assert_eq!(
            file.source_handle_base().revision(),
            loaded.binding().source_snapshot_id()
        );
        assert_eq!(
            file.source_handle_base().project_generation(),
            Some(snapshot.project_generation())
        );
        assert_eq!(
            view.source_handle(
                file.relative_path().as_str(),
                SourceSpan::whole_file(),
                file.source_handle_base().entity_key().cloned()
            )?,
            *file.source_handle_base()
        );
    }
    let (registry, batch, coverage, provenance, limits) =
        wow_project::graph::build_source_graph_proposals(&view, &stop)?.into_parts();
    assert_eq!(batch.universe().as_str(), loaded.binding().universe_id());
    for kind in ["toc_manifest", "xml_object"] {
        let definition = registry
            .entity_kind(kind)
            .ok_or_else(|| std::io::Error::other("platform structural registry kind missing"))?;
        assert!(definition.allows_universe(batch.universe()));
    }
    assert_eq!(provenance.context(), snapshot.generation_context());
    assert!(
        provenance
            .source_handles()
            .values()
            .any(|handle| handle.path().as_str() == "packages/Fixture/frames.xml")
    );
    for handle in provenance.source_handles().values() {
        assert_eq!(handle.origin_kind(), SourceOriginKind::Fixture);
        assert_eq!(
            handle.origin_id(),
            configuration.source_origin_id().as_str()
        );
        assert_eq!(handle.revision(), loaded.binding().source_snapshot_id());
        assert_eq!(
            handle.project_generation(),
            Some(snapshot.project_generation())
        );
        assert_eq!(
            handle.reference_generation(),
            Some(configuration.reference_generation())
        );
        assert_eq!(
            view.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned()
            )?,
            *handle
        );
    }
    let foundation = wow_graph::GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner = wow_graph::GraphPartitionSnapshot::new(
        registry,
        foundation,
        batch.source_context_id(),
        &stop,
    )?;
    let replacement = owner.prepare_replacement(
        wow_graph::GraphPartitionReplacement {
            expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
            expected_partition_digest: None,
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch,
            coverage,
        },
        &stop,
    )?;
    replacement.candidate().validate(&stop)?;
    assert!(!replacement.candidate().snapshot().nodes().is_empty());
    let refused = wow_project::replay::ProjectReplay::capture(&publisher, &stop)
        .err()
        .ok_or_else(|| std::io::Error::other("platform replay was accepted"))?;
    assert_eq!(refused.code(), ProjectErrorCode::DeferredCapability);
    assert_eq!(refused.phase(), ProjectPhase::Publication);
    assert_eq!(
        publisher.open_current()?.snapshot_id(),
        snapshot.snapshot_id()
    );

    // A retained inventory-only byte change must bypass NoChange even when
    // every parsed load/Main input and Library is identical.
    let changed_root = FixtureRoot::new()?;
    std::fs::write(changed_root.0.join("UI/opaque.bin"), [0xff, 0xfe, 0, 2])?;
    let changed_inventory = inventory(&changed_root, &profile)?;
    let changed_source = Arc::new(changed_root.directory()?.admit_platform_source(
        &profile,
        changed_inventory,
        &stop,
    )?);
    std::fs::remove_dir_all(&changed_root.0)?;
    let changed_packages =
        Arc::new(changed_source.specialize_packages(&fixture_packages("Fixture"), None, &stop)?);
    assert_eq!(changed_packages.files(), loaded.files());
    assert_eq!(changed_packages.load_plan(), loaded.load_plan());
    assert_eq!(changed_packages.main_plan(), loaded.main_plan());
    let changed_configuration =
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, profile.target())?
            .platform_packages(changed_packages)?
            .build()?;
    assert_ne!(
        changed_configuration.configuration_digest(),
        configuration.configuration_digest()
    );
    let request = wow_project::ProjectUpdateRequest::new(changed_configuration, Vec::new())
        .expected_generation(snapshot.project_generation())
        .expected_snapshot_digest(snapshot.canonical_snapshot_digest());
    let wow_project::ProjectUpdateOutcome::Published(changed_snapshot) =
        publisher.apply_update_cancellable(request, &stop)?
    else {
        return Err(std::io::Error::other("changed raw inventory was treated as NoChange").into());
    };
    assert_ne!(
        changed_snapshot.project_generation(),
        snapshot.project_generation()
    );
    assert_eq!(
        changed_snapshot
            .analyzer_binding()
            .main_workspace()
            .snapshot_id(),
        analyzer.main_workspace().snapshot_id()
    );
    snapshot.validate()?;
    Ok(())
}
