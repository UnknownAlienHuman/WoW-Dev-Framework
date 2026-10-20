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

#[test]
fn small_native_fact_budget_preserves_default_callable_admission_and_replay() -> TestResult {
    use wow_project::platform_source::PlatformPackageRetentionLimits;

    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    for package in ["Alpha", "Beta"] {
        std::fs::write(
            root.0.join("UI").join(package).join("frames.xml"),
            EMPTY_XML,
        )?;
    }
    let ordinary = root.packages(&stop)?;
    let declarations = PACKAGES
        .iter()
        .map(|package| {
            ProjectPackageInput::new(
                *package,
                format!("UI/{package}"),
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new("Fixture.toc"),
                    true,
                )],
            )
        })
        .collect::<Vec<_>>();
    let encoded = ordinary.load_plan().packages().iter().try_fold(
        0u64,
        |total, node| -> Result<u64, Box<dyn Error>> {
            let plan = ordinary
                .load_plan()
                .package_plan(&node.package)
                .ok_or("package plan missing")?;
            total
                .checked_add(u64::try_from(serde_json::to_vec(plan)?.len())?)
                .ok_or_else(|| "fixture plan encoding overflow".into())
        },
    )?;
    let native = Arc::new(ordinary.source().specialize_packages_with_retention(
        &declarations,
        None,
        PlatformPackageRetentionLimits::new(3, encoded)?,
        &stop,
    )?);
    let budget = ProjectBudgetPolicy::fixture_e0()?;
    let configuration = configuration_builder(
        ProjectKind::BlizzardUiPlatformSource,
        native.source().profile().target(),
    )?
    .platform_packages(Arc::clone(&native))?
    .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
    .budget_policy(ProjectBudgetPolicy::new(
        budget.max_files(),
        budget.max_total_source_bytes(),
        budget.max_single_file_bytes(),
        budget.max_update_operations(),
        16,
        budget.max_generic_findings(),
        budget.max_output_bytes(),
    )?)
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let input = ProjectInputBundle::closed(configuration, native.files().to_vec(), vec![library])?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input, &stop)?;
    let view = publisher.open_current()?;
    assert!(
        view.snapshot()
            .analyzer_binding()
            .function_call_report()
            .is_some()
    );
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    Ok(())
}

#[test]
fn retained_native_member_pairs_use_the_fact_budget_and_replay_exactly() -> TestResult {
    use wow_project::{ProjectErrorCode, platform_source::PlatformPackageRetentionLimits};

    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    for package in PACKAGES {
        std::fs::write(
            root.0.join("UI").join(package).join("frames.xml"),
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"/>\n",
        )?;
    }
    std::fs::write(root.0.join("UI/Empty/defs.lua"), "return true\n")?;
    for package in ["Alpha", "Beta"] {
        let source = if package == "Alpha" {
            // One binding also proves flow consumes the admitted member report.
            format!(
                "local retained = Lib.Step(1)\n{}",
                "Lib.Step(1)\n".repeat(32_767)
            )
        } else {
            "Lib.Step(1)\n".repeat(32_769)
        };
        std::fs::write(root.0.join("UI").join(package).join("defs.lua"), source)?;
    }
    let ordinary = root.packages(&stop)?;
    let declarations = PACKAGES
        .iter()
        .map(|package| {
            ProjectPackageInput::new(
                *package,
                format!("UI/{package}"),
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new("Fixture.toc"),
                    true,
                )],
            )
        })
        .collect::<Vec<_>>();
    let encoded = ordinary.load_plan().packages().iter().try_fold(
        0u64,
        |total, node| -> Result<u64, Box<dyn Error>> {
            let plan = ordinary
                .load_plan()
                .package_plan(&node.package)
                .ok_or("plan missing")?;
            total
                .checked_add(u64::try_from(serde_json::to_vec(plan)?.len())?)
                .ok_or_else(|| "fixture plan overflow".into())
        },
    )?;
    let native = Arc::new(ordinary.source().specialize_packages_with_retention(
        &declarations,
        None,
        PlatformPackageRetentionLimits::new(3, encoded)?,
        &stop,
    )?);
    let input = |packages: &Arc<PlatformPackageSpecialization>,
                 max_facts|
     -> Result<ProjectInputBundle, Box<dyn Error>> {
        let configuration = configuration_builder(
            ProjectKind::BlizzardUiPlatformSource,
            packages.source().profile().target(),
        )?
        .platform_packages(Arc::clone(packages))?
        .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
        .budget_policy(ProjectBudgetPolicy::new(
            16,
            2 * 1024 * 1024,
            1024 * 1024,
            16,
            max_facts,
            1024,
            128 * 1024 * 1024,
        )?)
        .build()?;
        let library = LuaWorkspaceSnapshot::build(
            configuration.analyzer_binding().backend().clone(),
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                "library/Lib.lua",
                "---@meta _\nLib = {}\nfunction Lib.Step(value) return value end\n",
            )],
            LuaWorkspaceLimits::new(8, 16_384, 1024 * 1024, 2 * 1024 * 1024)?,
        )?;
        Ok(ProjectInputBundle::closed(
            configuration,
            packages.files().to_vec(),
            vec![library],
        )?)
    };
    let pairs = 65_537;
    // Three package groups and their three XML documents are also charged.
    let facts = u64::try_from(pairs * 2 + 1 + 6)?;
    let mut ordinary_publisher = ProjectPublisher::new();
    assert_eq!(
        ordinary_publisher
            .publish_initial_cancellable(input(&ordinary, facts)?, &stop)
            .err()
            .ok_or("ordinary member capacity admitted")?
            .code(),
        ProjectErrorCode::SourceBudgetExceeded
    );
    assert!(ordinary_publisher.open_current().is_err());

    let mut publisher = ProjectPublisher::new();
    publisher.publish_initial_cancellable(input(&native, facts)?, &stop)?;
    let view = publisher.open_current()?;
    let binding = view.snapshot().analyzer_binding();
    assert_eq!(binding.member_call_report().references().len(), pairs);
    assert_eq!(binding.member_call_report().calls().len(), pairs);
    let flow = binding.local_flow_report();
    assert_eq!(flow.bindings().len(), 1);
    assert_eq!(
        flow.member_call_analysis_id(),
        binding.member_call_report().analysis_id()
    );
    assert!(flow.bindings()[0].initializer_call_fact_id().is_some());
    assert!(flow.uses().is_empty());
    assert!(flow.operations().is_empty());
    assert!(flow.guards().is_empty());
    assert!(flow.control_flow().is_empty());
    let package_bindings = binding
        .package_xml_bindings()
        .ok_or("package bindings missing")?;
    assert_eq!(package_bindings.groups().len(), 3);
    assert_eq!(
        package_bindings
            .groups()
            .iter()
            .map(|group| group.documents().len())
            .sum::<usize>(),
        3
    );
    assert!(package_bindings.symbol_lookup().is_none());
    for group in package_bindings.groups() {
        assert!(group.bindings().is_empty());
        assert!(group.inherited_script_sources().is_empty());
        assert!(group.receiver_sources().is_empty());
    }
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(restored.snapshot().analyzer_binding(), binding);

    // Both budgets admit65537 raw pairs. Only the aggregate131081-fact gate
    // distinguishes them: flow and package facts cannot be waived by selection.
    let mut aggregate_refusal = ProjectPublisher::new();
    let refused_input = input(&native, facts - 1)?;
    let finding_limit = refused_input
        .configuration()
        .budget_policy()
        .max_generic_findings();
    let error = aggregate_refusal
        .publish_initial_cancellable(refused_input, &stop)
        .err()
        .ok_or("aggregate fact overflow admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::UpdateBudgetExceeded);
    assert_eq!(
        error.message(),
        format!(
            "analyzer output exceeds the configured fact or finding budget: facts {facts}/{}, findings 0/{}",
            facts - 1,
            finding_limit
        )
    );
    assert!(aggregate_refusal.open_current().is_err());
    let mut cancelled = ProjectPublisher::new();
    assert_eq!(
        cancelled
            .publish_initial_cancellable(input(&native, facts)?, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled native input admitted")?
            .code(),
        ProjectErrorCode::AnalysisCancelled
    );
    assert!(cancelled.open_current().is_err());
    Ok(())
}

#[test]
fn retained_native_bindings_use_the_fact_budget_and_replay_exactly() -> TestResult {
    use wow_project::{ProjectErrorCode, platform_source::PlatformPackageRetentionLimits};

    let stop = AtomicBool::new(false);
    let create_packages = |per_package: usize,
                           methods: bool|
     -> Result<Arc<PlatformPackageSpecialization>, Box<dyn Error>> {
        let root = FixtureRoot::new()?;
        let xml = format!(
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\">{}</Ui>\n",
            if methods {
                "<Frame mixin=\"KnownMixin\"><Scripts><OnLoad method=\"OnLoad\"/></Scripts></Frame>"
            } else {
                "<Frame mixin=\"KnownMixin\"/>"
            }
            .repeat(per_package)
        );
        for package in ["Alpha", "Beta"] {
            std::fs::write(root.0.join("UI").join(package).join("frames.xml"), &xml)?;
        }
        root.packages(&stop)
    };
    let input = |packages: &Arc<PlatformPackageSpecialization>,
                 max_facts: u64|
     -> Result<ProjectInputBundle, Box<dyn Error>> {
        let budget = ProjectBudgetPolicy::fixture_e0()?;
        let configuration = configuration_builder(
            ProjectKind::BlizzardUiPlatformSource,
            packages.source().profile().target(),
        )?
        .platform_packages(Arc::clone(packages))?
        .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
        .budget_policy(ProjectBudgetPolicy::new(
            budget.max_files(),
            budget.max_total_source_bytes(),
            budget.max_single_file_bytes(),
            budget.max_update_operations(),
            max_facts,
            budget.max_generic_findings(),
            budget.max_output_bytes(),
        )?)
        .build()?;
        let library = LuaWorkspaceSnapshot::build(
            configuration.analyzer_binding().backend().clone(),
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                "library/package-xml-capacity.lua",
                "---@meta _\nKnownMixin = {}\nfunction KnownMixin:OnLoad() return self end\n",
            )],
            LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
        )?;
        Ok(ProjectInputBundle::closed(
            configuration,
            packages.files().to_vec(),
            vec![library],
        )?)
    };
    let row_count = |publisher: &ProjectPublisher| -> Result<usize, Box<dyn Error>> {
        let view = publisher.open_current()?;
        Ok(view
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .ok_or("package bindings missing")?
            .groups()
            .iter()
            .map(|group| group.bindings().len())
            .sum())
    };

    // The ordinary route still admits exactly 4096 rows.
    let exact = create_packages(2048, false)?;
    let mut ordinary = ProjectPublisher::with_function_call_facts();
    ordinary.publish_initial_cancellable(input(&exact, 8192)?, &stop)?;
    assert_eq!(row_count(&ordinary)?, 4096);

    let packages = create_packages(2049, false)?;
    let mut ordinary = ProjectPublisher::with_function_call_facts();
    let error = ordinary
        .publish_initial_cancellable(input(&packages, 8192)?, &stop)
        .err()
        .ok_or("ordinary binding overflow admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::SourceBudgetExceeded);
    assert!(
        error
            .message()
            .contains("before next record=4096, limit=4096")
    );

    let declarations = PACKAGES
        .iter()
        .map(|package| {
            ProjectPackageInput::new(
                *package,
                format!("UI/{package}"),
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new("Fixture.toc"),
                    true,
                )],
            )
        })
        .collect::<Vec<_>>();
    let specialize_native = |packages: &Arc<PlatformPackageSpecialization>| -> Result<Arc<PlatformPackageSpecialization>, Box<dyn Error>> {
        let encoded = packages.load_plan().packages().iter().try_fold(0u64, |total, node| -> Result<u64, Box<dyn Error>> {
            let plan = packages.load_plan().package_plan(&node.package).ok_or("package plan missing")?;
            total.checked_add(u64::try_from(serde_json::to_vec(plan)?.len())?)
                .ok_or_else(|| "fixture plan encoding overflow".into())
        })?;
        Ok(Arc::new(packages.source().specialize_packages_with_retention(
            &declarations, None, PlatformPackageRetentionLimits::new(3, encoded)?, &stop,
        )?))
    };
    let native = specialize_native(&packages)?;
    // 4098 rows exceed the selected row ceiling by one; no project is published.
    let mut bounded = ProjectPublisher::with_function_call_facts();
    let error = bounded
        .publish_initial_cancellable(input(&native, 4097)?, &stop)
        .err()
        .ok_or("native binding limit+1 admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::SourceBudgetExceeded);
    assert!(
        error
            .message()
            .contains("before next record=4097, limit=4097")
    );
    assert!(bounded.open_current().is_err());

    // Exact binding-row admission cannot waive the combined native fact budget.
    let mut combined = ProjectPublisher::with_function_call_facts();
    let error = combined
        .publish_initial_cancellable(input(&native, 4098)?, &stop)
        .err()
        .ok_or("combined fact overflow admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::UpdateBudgetExceeded);
    assert!(
        error
            .message()
            .contains("configured fact or finding budget")
    );
    assert!(combined.open_current().is_err());

    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input(&native, 8192)?, &stop)?;
    assert_eq!(row_count(&publisher)?, 4098);
    let view = publisher.open_current()?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let bytes = serde_json::to_vec(&replay)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes)?["schema"],
        "wow-project/native-project-replay/11"
    );
    let restored = ProjectReplay::from_json(&bytes, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings(),
        view.snapshot().analyzer_binding().package_xml_bindings()
    );
    let mut cancelled = ProjectPublisher::with_function_call_facts();
    let error = cancelled
        .publish_initial_cancellable(input(&native, 8192)?, &AtomicBool::new(true))
        .err()
        .ok_or("cancelled native input admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::AnalysisCancelled);
    assert!(cancelled.open_current().is_err());

    // The distinct receiver-owner population can also exceed ordinary4096.
    let packages = create_packages(2049, true)?;
    let native = specialize_native(&packages)?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input(&native, 32_768)?, &stop)?;
    assert_eq!(row_count(&publisher)?, 8196);
    let view = publisher.open_current()?;
    let bindings = view
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("native receiver bindings missing")?;
    assert_eq!(
        bindings
            .groups()
            .iter()
            .map(|group| group.receiver_sources().len())
            .sum::<usize>(),
        4098
    );
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings(),
        Some(bindings)
    );

    // Repeated package-local rows still share one distinct native query union.
    // The handwritten absent names exercise capacity, not WoW negative authority.
    let root = FixtureRoot::new()?;
    let mut xml = String::from("<Ui xmlns=\"http://www.blizzard.com/wow/ui/\">");
    for index in 0..4096 {
        xml.push_str(&format!("<Frame mixin=\"FixtureQuery_{index:04}\"/>"));
    }
    xml.push_str("<Frame mixin=\"KnownMixin\"/></Ui>\n");
    for package in ["Alpha", "Beta"] {
        std::fs::write(root.0.join("UI").join(package).join("frames.xml"), &xml)?;
    }
    let native = specialize_native(&root.packages(&stop)?)?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input(&native, 32_768)?, &stop)?;
    assert_eq!(row_count(&publisher)?, 8194);
    let view = publisher.open_current()?;
    let bindings = view
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("distinct native query bindings missing")?;
    let lookup = bindings
        .symbol_lookup()
        .ok_or("native query lookup missing")?;
    assert_eq!(lookup.lookups().len(), 4097);
    assert_eq!(
        lookup.lookups()["KnownMixin"].state,
        SymbolLookupState::UniqueAnalyzerDeclaration
    );
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings(),
        Some(bindings)
    );

    // A single captured inline handler can have many inherited source uses.
    // These rows enumerate candidates; they do not prove effective dispatch.
    let root = FixtureRoot::new()?;
    let xml = format!(
        "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Frame name=\"FixtureInheritedTemplate\" virtual=\"true\"><Scripts><OnLoad>local value = true</OnLoad></Scripts></Frame>{}</Ui>\n",
        "<Frame inherits=\"FixtureInheritedTemplate\"/>".repeat(2049)
    );
    for package in ["Alpha", "Beta"] {
        std::fs::write(root.0.join("UI").join(package).join("frames.xml"), &xml)?;
    }
    let packages = root.packages(&stop)?;
    let mut ordinary = ProjectPublisher::with_function_call_facts();
    assert_eq!(
        ordinary
            .publish_initial_cancellable(input(&packages, 32_768)?, &stop)
            .err()
            .ok_or("ordinary inherited source universe admitted")?
            .code(),
        ProjectErrorCode::SourceBudgetExceeded
    );
    assert!(ordinary.open_current().is_err());
    let native = specialize_native(&packages)?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input(&native, 32_768)?, &stop)?;
    let view = publisher.open_current()?;
    let bindings = view
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("inherited source bindings missing")?;
    assert_eq!(
        bindings
            .groups()
            .iter()
            .map(|group| group.inherited_script_sources().len())
            .sum::<usize>(),
        4098
    );
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings(),
        Some(bindings)
    );
    // Each population fits this ceiling individually, but their sum does not.
    let mut combined = ProjectPublisher::with_function_call_facts();
    assert_eq!(
        combined
            .publish_initial_cancellable(input(&native, 8194)?, &stop)
            .err()
            .ok_or("combined inherited source facts overflow admitted")?
            .code(),
        ProjectErrorCode::UpdateBudgetExceeded
    );
    assert!(combined.open_current().is_err());
    let mut cancelled = ProjectPublisher::with_function_call_facts();
    assert_eq!(
        cancelled
            .publish_initial_cancellable(input(&native, 32_768)?, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled inherited source input admitted")?
            .code(),
        ProjectErrorCode::AnalysisCancelled
    );
    assert!(cancelled.open_current().is_err());
    Ok(())
}

#[test]
fn retained_native_query_references_use_output_admission_and_replay_exactly() -> TestResult {
    use wow_project::{ProjectErrorCode, platform_source::PlatformPackageRetentionLimits};

    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    let xml = format!(
        "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\">{}</Ui>\n",
        "<Frame mixin=\"KnownMixin,OtherMixin\"><Scripts><OnLoad method=\"OnLoad\"/></Scripts></Frame>"
            .repeat(2049)
    );
    for package in ["Alpha", "Beta"] {
        std::fs::write(root.0.join("UI").join(package).join("frames.xml"), &xml)?;
    }
    // Admission removes the fixture directory. All subsequent paths use held bytes.
    let ordinary = root.packages(&stop)?;
    let declarations = PACKAGES
        .iter()
        .map(|package| {
            ProjectPackageInput::new(
                *package,
                format!("UI/{package}"),
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new("Fixture.toc"),
                    true,
                )],
            )
        })
        .collect::<Vec<_>>();
    let encoded = ordinary.load_plan().packages().iter().try_fold(
        0u64,
        |total, node| -> Result<u64, Box<dyn Error>> {
            let plan = ordinary
                .load_plan()
                .package_plan(&node.package)
                .ok_or("package plan missing")?;
            total
                .checked_add(u64::try_from(serde_json::to_vec(plan)?.len())?)
                .ok_or_else(|| "fixture plan encoding overflow".into())
        },
    )?;
    let native = Arc::new(ordinary.source().specialize_packages_with_retention(
        &declarations,
        None,
        PlatformPackageRetentionLimits::new(3, encoded)?,
        &stop,
    )?);
    let input = |packages: &Arc<PlatformPackageSpecialization>,
                 max_output_bytes: u64|
     -> Result<ProjectInputBundle, Box<dyn Error>> {
        let budget = ProjectBudgetPolicy::fixture_e0()?;
        let configuration = configuration_builder(
            ProjectKind::BlizzardUiPlatformSource,
            packages.source().profile().target(),
        )?
        .platform_packages(Arc::clone(packages))?
        .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
        .budget_policy(ProjectBudgetPolicy::new(
            budget.max_files(),
            budget.max_total_source_bytes(),
            budget.max_single_file_bytes(),
            budget.max_update_operations(),
            budget.max_analyzer_facts(),
            budget.max_generic_findings(),
            max_output_bytes,
        )?)
        .build()?;
        let library = LuaWorkspaceSnapshot::build(
            configuration.analyzer_binding().backend().clone(),
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                "library/package-xml-query-references.lua",
                "---@meta _\nKnownMixin = {}\nfunction KnownMixin:OnLoad() return self end\nOtherMixin = {}\nfunction OtherMixin:OnLoad() return self end\n",
            )],
            LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
        )?;
        Ok(ProjectInputBundle::closed(
            configuration,
            packages.files().to_vec(),
            vec![library],
        )?)
    };
    let output_budget = ProjectBudgetPolicy::fixture_e0()?.max_output_bytes();
    let mut unselected = ProjectPublisher::with_function_call_facts();
    assert_eq!(
        unselected
            .publish_initial_cancellable(input(&ordinary, output_budget)?, &stop)
            .err()
            .ok_or("ordinary capacity unexpectedly admitted")?
            .code(),
        ProjectErrorCode::SourceBudgetExceeded
    );
    assert!(unselected.open_current().is_err());

    // Two mixin queries plus two method candidates per Frame, repeated in two
    // packages: 16392 references, but only four distinct lookup names.
    let references = 2049 * 2 * 4;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input(&native, output_budget)?, &stop)?;
    let view = publisher.open_current()?;
    let binding = view.snapshot().analyzer_binding();
    let report = binding
        .package_xml_bindings()
        .ok_or("query-reference report missing")?;
    assert_eq!(
        report
            .groups()
            .iter()
            .flat_map(|group| group.bindings())
            .map(|row| row.queries.len())
            .sum::<usize>(),
        references
    );
    let lookup = report.symbol_lookup().ok_or("shared lookup missing")?;
    assert_eq!(lookup.lookups().len(), 4);
    for name in [
        "KnownMixin",
        "OtherMixin",
        "KnownMixin.OnLoad",
        "OtherMixin.OnLoad",
    ] {
        assert_eq!(
            lookup.lookups()[name].state,
            SymbolLookupState::UniqueAnalyzerDeclaration
        );
    }
    assert!(
        report
            .groups()
            .iter()
            .flat_map(|group| group.bindings())
            .filter(|row| row.kind == XmlLuaBindingKind::Method)
            .all(|row| row.queries.len() == 2 && row.state == XmlLuaBindingState::Ambiguous)
    );
    let calls = binding
        .function_call_report()
        .ok_or("callable report missing")?;
    assert!(
        calls
            .symbol_lookup_analysis_ids()
            .iter()
            .any(|id| id == lookup.analysis_id())
    );
    for name in ["KnownMixin.OnLoad", "OtherMixin.OnLoad"] {
        assert!(matches!(
            calls.named_targets()[name],
            SourceCallTarget::LibraryFunction { .. }
        ));
    }
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.project_generation(), view.project_generation());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings(),
        Some(report)
    );

    // At the exact reference ceiling, preparation AND classification complete;
    // final serialized-output admission still refuses this much larger report.
    let mut exact = ProjectPublisher::with_function_call_facts();
    let error = exact
        .publish_initial_cancellable(input(&native, u64::try_from(references)?)?, &stop)
        .err()
        .ok_or("combined output overflow admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::UpdateBudgetExceeded);
    let (observed, limit) = error
        .message()
        .strip_prefix("analyzer output exceeds the configured serialized-output budget: observed ")
        .ok_or("serialized output count missing")?
        .split_once(", limit ")
        .ok_or("serialized output limit missing")?;
    assert_eq!(limit.parse::<u64>()?, u64::try_from(references)?);
    assert!(observed.parse::<u64>()? > u64::try_from(references)?);
    assert!(exact.open_current().is_err());

    let mut bounded = ProjectPublisher::with_function_call_facts();
    let error = bounded
        .publish_initial_cancellable(input(&native, u64::try_from(references - 1)?)?, &stop)
        .err()
        .ok_or("query-reference limit+1 admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::SourceBudgetExceeded);
    assert!(
        error
            .message()
            .contains("expanded query references=16392, limit=16391")
    );
    assert!(bounded.open_current().is_err());
    let mut cancelled = ProjectPublisher::with_function_call_facts();
    assert_eq!(
        cancelled
            .publish_initial_cancellable(input(&native, output_budget)?, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled query-reference input admitted")?
            .code(),
        ProjectErrorCode::AnalysisCancelled
    );
    assert!(cancelled.open_current().is_err());
    Ok(())
}

#[test]
fn selected_callable_graph_counts() -> TestResult {
    use wow_emmy::function_calls::FunctionCallBudget;
    use wow_project::{
        ProjectErrorCode, ProjectPhase,
        graph::{SourceGraphBudget, build_source_graph_proposals},
        platform_source::PlatformPackageRetentionLimits,
    };

    const ROW: &str = "do local f=function()end;f()end\n";
    const EXTRA_CALLS: &str = "do local f=function()end;f();f();f();f()end\n";

    let stop = AtomicBool::new(false);
    let packages =
        |above_default: bool| -> Result<Arc<PlatformPackageSpecialization>, Box<dyn Error>> {
            let root = FixtureRoot::new()?;
            for package in PACKAGES {
                std::fs::write(
                    root.0.join("UI").join(package).join("frames.xml"),
                    "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"/>\n",
                )?;
            }
            let alpha = format!(
                "{}{EXTRA_CALLS}",
                ROW.repeat(if above_default { 4094 } else { 4093 }),
            );
            let beta = ROW.repeat(4095);
            let parent = ProjectBudgetPolicy::fixture_e0()?;
            for (package, text) in [
                ("Alpha", alpha),
                ("Beta", beta),
                ("Empty", "return true\n".into()),
            ] {
                assert!(u64::try_from(text.len())? <= parent.max_single_file_bytes());
                std::fs::write(root.0.join("UI").join(package).join("defs.lua"), text)?;
            }
            let ordinary = root.packages(&stop)?;
            assert!(!root.0.exists());
            let declarations = PACKAGES
                .iter()
                .map(|package| {
                    ProjectPackageInput::new(
                        *package,
                        format!("UI/{package}"),
                        true,
                        vec![ProjectPackageVariantInput::new(
                            ProjectDiskFile::new("Fixture.toc"),
                            true,
                        )],
                    )
                })
                .collect::<Vec<_>>();
            let encoded = ordinary.load_plan().packages().iter().try_fold(
                0u64,
                |total, node| -> Result<u64, Box<dyn Error>> {
                    let plan = ordinary
                        .load_plan()
                        .package_plan(&node.package)
                        .ok_or("native callable package plan missing")?;
                    total
                        .checked_add(u64::try_from(serde_json::to_vec(plan)?.len())?)
                        .ok_or_else(|| "native callable plan encoding overflow".into())
                },
            )?;
            Ok(Arc::new(
                ordinary.source().specialize_packages_with_retention(
                    &declarations,
                    None,
                    PlatformPackageRetentionLimits::new(3, encoded)?,
                    &stop,
                )?,
            ))
        };
    // Keep the existing finite parent fact/output/source policy. Only the
    // already-supported metadata component is selected for these pressure rows.
    let policy = ProjectBudgetPolicy::fixture_e0()?
        .with_source_graph_budget(SourceGraphBudget::new(32 * 1024 * 1024)?);
    assert_eq!(policy.function_call_budget(), None);

    {
        let packages = packages(false)?;
        let publisher = publish_graph_with_policy(
            &packages,
            Some(PackageXmlBindingProfile::SameSessionV1),
            true,
            policy,
            &stop,
        )?;
        let view = publisher.open_current()?;
        let report = view
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("exact native callable report missing")?;
        report.validate()?;
        assert!(report.source_health_complete());
        assert_eq!(report.files().len(), 3);
        assert_eq!(report.functions().len(), 8192);
        assert_eq!(report.calls().len(), 8192);
        assert_eq!(
            report.budget(),
            view.configuration().budget_policy().function_call_budget()
        );
        assert_eq!(report.budget(), None);
        let (_, _, _, provenance, _) = build_source_graph_proposals(&view, &stop)?.into_parts();
        assert_eq!(provenance.functions().len(), 8192);
        assert_eq!(provenance.call_sites().len(), 8192);
        assert_eq!(provenance.function_call_report(), Some(report));
    }

    let packages = packages(true)?;
    {
        let publisher = publish_graph_with_policy(
            &packages,
            Some(PackageXmlBindingProfile::SameSessionV1),
            true,
            policy,
            &stop,
        )?;
        let view = publisher.open_current()?;
        let report = view
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("default native callable report missing")?;
        report.validate()?;
        assert!(report.source_health_complete());
        assert_eq!(report.functions().len(), 8193);
        assert_eq!(report.calls().len(), 8193);
        assert_eq!(report.budget(), None);
        let error = build_source_graph_proposals(&view, &stop)
            .err()
            .ok_or("default graph callable count overflow admitted")?;
        assert_eq!(error.code(), ProjectErrorCode::SourceBudgetExceeded);
        assert_eq!(error.phase(), ProjectPhase::View);
        assert_eq!(
            error.message(),
            "source graph callable counts exceed their bounded profile: functions 8193/8192, calls 8193/8192",
        );
        assert_eq!(publisher.open_current()?.snapshot_id(), view.snapshot_id());
    }

    let defaults = FunctionCallBudget::default();
    let selected = FunctionCallBudget::new(
        defaults.max_ast_visits(),
        8193,
        8193,
        defaults.max_global_accesses(),
        defaults.max_named_targets(),
        defaults.max_report_bytes(),
    )?;
    let publisher = publish_graph_with_policy(
        &packages,
        Some(PackageXmlBindingProfile::SameSessionV1),
        true,
        policy.with_function_call_budget(selected),
        &stop,
    )?;
    let view = publisher.open_current()?;
    let report = view
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("selected native callable report missing")?;
    report.validate()?;
    assert!(report.source_health_complete());
    assert_eq!(report.files().len(), 3);
    assert_eq!(report.functions().len(), 8193);
    assert_eq!(
        report
            .functions()
            .iter()
            .filter(|function| function.kind() == SourceFunctionKind::Chunk)
            .count(),
        3,
    );
    assert_eq!(report.calls().len(), 8193);
    assert_eq!(report.budget(), Some(selected));
    assert_eq!(
        report.budget(),
        view.configuration().budget_policy().function_call_budget()
    );

    let (_, _, _, provenance, _) = build_source_graph_proposals(&view, &stop)?.into_parts();
    assert_eq!(provenance.functions().len(), 8193);
    assert_eq!(provenance.call_sites().len(), 8193);
    assert_eq!(provenance.function_call_report(), Some(report));
    assert_eq!(
        provenance
            .functions()
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<BTreeSet<_>>(),
        report
            .functions()
            .iter()
            .map(|function| function.fact_id())
            .collect::<BTreeSet<_>>(),
    );
    assert_eq!(
        provenance
            .call_sites()
            .iter()
            .map(|call| call.call_id.as_str())
            .collect::<BTreeSet<_>>(),
        report
            .calls()
            .iter()
            .map(|call| call.fact_id())
            .collect::<BTreeSet<_>>(),
    );

    let error = build_source_graph_proposals(&view, &AtomicBool::new(true))
        .err()
        .ok_or("cancelled native callable graph admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::AnalysisCancelled);
    assert_eq!(publisher.open_current()?.snapshot_id(), view.snapshot_id());

    Ok(())
}

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
    publish_graph_with_policy(
        packages,
        selected,
        graph,
        ProjectBudgetPolicy::fixture_e0()?,
        stop,
    )
}

fn publish_graph_with_policy(
    packages: &Arc<PlatformPackageSpecialization>,
    selected: Option<PackageXmlBindingProfile>,
    graph: bool,
    policy: ProjectBudgetPolicy,
    stop: &AtomicBool,
) -> Result<ProjectPublisher, Box<dyn Error>> {
    let builder = configuration_builder(
        ProjectKind::BlizzardUiPlatformSource,
        packages.source().profile().target(),
    )?
    .budget_policy(policy)
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
fn source_graph_diagnostics_preserve_native_semantic_bytes() -> TestResult {
    use wow_project::graph::{
        SourceGraphDiagnostics, build_source_graph_proposals,
        build_source_graph_proposals_diagnosed,
    };
    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    let packages = root.packages(&stop)?;
    assert!(!root.0.exists());
    let publisher = publish_graph(
        &packages,
        Some(PackageXmlBindingProfile::SameSessionV1),
        true,
        &stop,
    )?;
    let view = publisher.open_current()?;
    let expected = serde_json::to_vec(&build_source_graph_proposals(&view, &stop)?.into_parts())?;
    let mut diagnostics = SourceGraphDiagnostics::enabled();
    let actual = build_source_graph_proposals_diagnosed(&view, &stop, &mut diagnostics)?;
    assert_eq!(serde_json::to_vec(&actual.into_parts())?, expected);
    let observation = serde_json::to_value(&diagnostics)?;
    assert_eq!(
        observation["input"]["project_snapshot_id"],
        view.snapshot_id()
    );
    let report = view
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("report")?;
    assert_eq!(observation["input"]["functions"], report.functions().len());
    assert_eq!(observation["input"]["calls"], report.calls().len());
    let phases = observation["phases"].as_array().ok_or("phases")?;
    for name in ["functions", "calls"] {
        let phase = phases
            .iter()
            .find(|phase| phase["phase"] == name)
            .ok_or("phase missing")?;
        assert_eq!(phase["outcome"], "completed");
        assert_eq!(phase["completed_records"], phase["available_input_records"]);
        assert!(phase["elapsed_microseconds"].is_number());
        assert!(phase["metadata_refusal"].is_null());
    }
    let encoded = serde_json::to_string(&diagnostics)?;
    assert!(encoded.len() < 64 * 1024);
    assert!(!encoded.contains("UI/Alpha") && !encoded.contains("function "));
    diagnostics.reset();
    let reset = serde_json::to_value(&diagnostics)?;
    assert!(reset["input"].is_null());
    assert!(
        reset["phases"]
            .as_array()
            .ok_or("phases")?
            .iter()
            .all(|phase| phase["outcome"] == "not_evaluated"
                && phase["counter_after_bytes"].is_null())
    );
    Ok(())
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
fn package_binding_selection_requires_native_owner_and_replays_current_input() -> TestResult {
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
