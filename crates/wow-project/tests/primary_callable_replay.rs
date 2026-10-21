//! Focused ordinary Main + exact Library replay/catalog compatibility.
//! The inputs are inert native fixtures, not platform or external qualification.
use std::{
    error::Error,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use serde_json::Value;
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
    SchemaVersionEntry, SourceKind, SourceLogicalSnapshot, canonical_json_bytes,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    backend_identity_from_report,
    function_calls::{
        FunctionCallReport, PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
        SourceDeclaredParameter, SourceFunctionDeclaration, SourceFunctionDeclarationForm,
        SourceFunctionKind, SourceSignatureType,
    },
};
use wow_graph::{GraphPartitionReplacement, GraphPartitionSnapshot, GraphSnapshot};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfiguration,
    ProjectConfigurationBuilder, ProjectErrorCode, ProjectId, ProjectInputBundle, ProjectInputFile,
    ProjectKind, ProjectPublisher, ProjectSourceOriginId, ProjectView, ProjectWorkspaceId,
    replay::{ProjectReplay, publication},
};
use wow_store::{
    OperationId,
    project::{PartitionRecord, ProjectStore, PublicationRequest, RecordCatalog},
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Declarations = PhysicalFunctionDeclarationRecipe;
type Signatures = PhysicalFunctionSignatureRecipe;

const PRIMARY_STORAGE: &str = "wow-project.live-replay";
// Same typed native fixture used by the existing service signature regression.
const MAIN: &str = "---@param value number\n---@return number\nlocal function TypedSignatureFixture(value)\n    return value\nend\n";
// Retained synthetic compatibility import fixture used by the existing native
// service fixture. Its digest is frozen, not regenerated or a freshness claim.
const COMPATIBILITY_REPORT: &[u8] = br#"{"analysis_crate":{"manifest_path":"crates/emmylua_code_analysis/Cargo.toml","name":"emmylua_code_analysis","version":"0.25.1"},"compatibility":{"missing_symbols":[],"required_symbols":[],"status":"compatible"},"producer_version":1,"report_sha256":"sha256:e56096cd332cd4d35105772f6084c0c4116f885dff237097c38706b249f8f343","schema":"wow-dev-framework/emmylua-compatibility-report","schema_version":1,"source":{"network_checked":false,"relation":"unverified_current","revision":"aaaca68425d9362876228649b0b8d92f07654daa","tree":"9175c01384e650b9a5bd64da69c36f47dbeaaf67"},"surface":{"files":[],"surface_sha256":"sha256:3333333333333333333333333333333333333333333333333333333333333333","symbols":[]}}"#;

fn configuration_builder() -> TestResult<ProjectConfigurationBuilder> {
    let backend = backend_identity_from_report(COMPATIBILITY_REPORT)?;
    assert_eq!(backend.revision(), EMMYLUA_REVISION);
    assert_eq!(backend.tree(), EMMYLUA_TREE);
    assert_eq!(backend.crate_version(), Some(EMMYLUA_CODE_ANALYSIS_VERSION));
    let compatibility_digest = backend.compatibility_report_sha256().to_owned();
    let analyzer = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        compatibility_digest,
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend,
    )?;
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:retail-120100-e0-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:fixture-e0".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("wow-project-e0-d-fixture")
    .build()?;
    let builder = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-primary-callables-replay")?,
        ProjectKind::Fixture,
        profile,
        ReferenceGenerationId::derive(&"fixture-reference-e0")?,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:main:primary-callables")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:primary-callables",
    )?)
    .logical_root("fixtures/primary-callables/main")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?);
    Ok(builder)
}

fn configuration(
    declarations: Option<Declarations>,
    signatures: Option<Signatures>,
) -> TestResult<ProjectConfiguration> {
    let builder = configuration_builder()?;
    let builder = match declarations {
        Some(recipe) => builder.with_physical_function_declarations(recipe),
        None => builder,
    };
    Ok(match signatures {
        Some(recipe) => builder.with_physical_function_signatures(recipe),
        None => builder,
    }
    .build()?)
}

fn library(backend: EmmyBackendIdentity) -> TestResult<LuaWorkspaceSnapshot> {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../wow-emmy/examples/workspace-fixture.json"
    ))?;
    let source = fixture["files"]
        .as_array()
        .ok_or("existing workspace fixture files")?
        .iter()
        .find(|file| file["logical_path"] == "library/C_E0Fixture.lua")
        .and_then(|file| file["source_text"].as_str())
        .ok_or("existing exact Library fixture")?;
    Ok(LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            source,
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?)
}

fn bundle(
    config: ProjectConfiguration,
    library: &LuaWorkspaceSnapshot,
) -> TestResult<ProjectInputBundle> {
    Ok(ProjectInputBundle::closed(
        config,
        vec![ProjectInputFile::new("main/primary.lua", MAIN)?],
        vec![library.clone()],
    )?)
}

fn report(view: &ProjectView) -> TestResult<&FunctionCallReport> {
    Ok(view
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("genuine native function-call report")?)
}

fn assert_native_recipes(
    view: &ProjectView,
    library: &LuaWorkspaceSnapshot,
    declarations: Option<Declarations>,
    signatures: Option<Signatures>,
) -> TestResult {
    assert_eq!(view.configuration().project_kind(), ProjectKind::Fixture);
    assert!(view.configuration().load_plan().is_none());
    assert!(view.configuration().package_load_plan().is_none());
    assert!(view.configuration().platform_packages().is_none());
    assert!(view.configuration().platform_graph_profile().is_none());
    assert_eq!(
        view.configuration().physical_function_declarations(),
        declarations
    );
    assert_eq!(
        view.configuration().physical_function_signatures(),
        signatures
    );
    let report = report(view)?;
    report.validate()?;
    assert!(report.source_health_complete());
    assert_eq!(report.declaration_recipe(), declarations);
    assert_eq!(report.signature_recipe(), signatures);
    assert_eq!(
        report.library_snapshot_ids(),
        &[library.snapshot_id().to_owned()]
    );
    assert_eq!(
        report.main_snapshot_id(),
        view.snapshot()
            .analyzer_binding()
            .main_workspace()
            .snapshot_id()
    );
    assert_eq!(report.files().len(), 1);
    assert_eq!(report.files()[0].path, "main/primary.lua");
    assert_eq!(report.functions().len(), 2);
    let closure = report
        .functions()
        .iter()
        .find(|fact| fact.kind() == SourceFunctionKind::Closure)
        .ok_or("physical annotated native closure")?;
    assert_eq!(closure.path(), "main/primary.lua");
    assert!(report.functions().iter().all(|fact| {
        fact.shape().is_none()
            && fact.declaration().is_some() == declarations.is_some()
            && fact.signature().is_some() == signatures.is_some()
    }));
    if declarations.is_some() {
        let Some(SourceFunctionDeclaration::Observed {
            form: SourceFunctionDeclarationForm::LocalFunction,
            colon_definition: false,
            parameters,
            ..
        }) = closure.declaration()
        else {
            return Err("native declaration was not an observed local header".into());
        };
        assert!(matches!(
            parameters.as_slice(),
            [SourceDeclaredParameter::Name { name, .. }] if name == "value"
        ));
    }
    if signatures.is_some() {
        let signature = closure.signature().ok_or("native signature selection")?;
        let comparable = signature
            .comparable()
            .ok_or("native indexed numeric signature")?;
        assert!(signature.unavailable_reasons().is_empty());
        assert_eq!(comparable.parameters().len(), 1);
        assert_eq!(
            comparable.parameters()[0].type_shape(),
            &SourceSignatureType::Number
        );
        assert_eq!(comparable.returns().len(), 1);
        assert_eq!(
            comparable.returns()[0].types(),
            &[SourceSignatureType::Number]
        );
    }
    Ok(())
}

fn publication_parts(
    publisher: &ProjectPublisher,
    stop: &AtomicBool,
) -> TestResult<(
    Vec<PartitionRecord>,
    std::collections::BTreeMap<String, String>,
)> {
    let view = publisher.open_current()?;
    let (registry, batch, coverage, _, limits) =
        wow_project::graph::build_source_graph_proposals(&view, stop)?.into_parts();
    let foundation = GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner = GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), stop)?;
    let replacement = owner.prepare_replacement(
        GraphPartitionReplacement {
            expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
            expected_partition_digest: None,
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch,
            coverage,
        },
        stop,
    )?;
    Ok(
        publication::ProjectPublicationBundle::build(publisher, replacement.candidate(), stop)?
            .into_parts(),
    )
}

fn catalog(project_schemas: &[&'static str]) -> TestResult<RecordCatalog> {
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(project_schemas);
    Ok(RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )?)
}

static NEXT_STORE: AtomicUsize = AtomicUsize::new(0);
struct StoreRoot(PathBuf);
impl StoreRoot {
    fn new() -> TestResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "wow-primary-callables-replay-{}-{}",
            std::process::id(),
            NEXT_STORE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        Ok(Self(root))
    }
}
impl Drop for StoreRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn refused_archive(value: &Value, stop: &AtomicBool) -> TestResult {
    let error = ProjectReplay::from_json(&serde_json::to_vec(value)?, stop)
        .err()
        .ok_or("mutated primary archive was admitted")?;
    assert_eq!(error.code(), ProjectErrorCode::SnapshotInvalid);
    Ok(())
}

#[test]
fn selected_primary_recipes_capture_hydrate_recapture_exactly() -> TestResult {
    let stop = AtomicBool::new(false);
    let mut combined = None;
    let mut generations = std::collections::BTreeSet::new();
    for (declarations, signatures) in [
        (Some(Declarations::DeclaredSyntaxV1), None),
        (None, Some(Signatures::IndexedTypesV1)),
        (
            Some(Declarations::DeclaredSyntaxV1),
            Some(Signatures::IndexedTypesV1),
        ),
    ] {
        let config = configuration(declarations, signatures)?;
        let library = library(config.analyzer_binding().backend().clone())?;
        let mut publisher = ProjectPublisher::with_function_call_facts();
        publisher.publish_initial_cancellable(bundle(config, &library)?, &stop)?;
        let original = publisher.open_current()?;
        assert_native_recipes(&original, &library, declarations, signatures)?;
        assert!(generations.insert(original.project_generation().to_string()));
        let replay = ProjectReplay::capture(&publisher, &stop)?;
        let bytes = canonical_json_bytes(&replay)?;
        let wire: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(wire["function_calls"], true);
        assert!(wire.get("load").is_none());
        assert!(wire.get("packages").is_none());
        assert!(wire.get("platform").is_none());
        for (field, selected) in [
            (
                "physical_function_declarations",
                declarations.map(serde_json::to_value).transpose()?,
            ),
            (
                "physical_function_signatures",
                signatures.map(serde_json::to_value).transpose()?,
            ),
        ] {
            assert_eq!(wire["configuration"].get(field), selected.as_ref());
        }

        let decoded = ProjectReplay::from_json(&bytes, &stop)?;
        assert_eq!(canonical_json_bytes(&decoded)?, bytes);
        let restored = decoded.hydrate(&stop)?;
        restored.snapshot().validate()?;
        assert_native_recipes(&restored, &library, declarations, signatures)?;
        assert_eq!(restored.project_generation(), original.project_generation());
        assert_eq!(restored.snapshot_id(), original.snapshot_id());
        assert_eq!(
            restored.analyzer_snapshot_id(),
            original.analyzer_snapshot_id()
        );
        assert_eq!(restored.configuration(), original.configuration());
        assert_eq!(
            report(&restored)?.analysis_id(),
            report(&original)?.analysis_id()
        );
        assert_eq!(report(&restored)?, report(&original)?);

        // Recapture through a real publisher using held hydrated Main bytes and
        // the same exact Library; no assumed archive extraction DTO is needed.
        let files = restored
            .snapshot()
            .analyzer_binding()
            .main_workspace()
            .files()
            .iter()
            .map(|file| ProjectInputFile::new(file.path(), file.text()))
            .collect::<wow_project::ProjectResult<Vec<_>>>()?;
        let mut recaptured = ProjectPublisher::with_function_call_facts();
        recaptured.publish_initial_cancellable(
            ProjectInputBundle::closed(
                restored.configuration().clone(),
                files,
                vec![library.clone()],
            )?,
            &stop,
        )?;
        assert_eq!(
            canonical_json_bytes(&ProjectReplay::capture(&recaptured, &stop)?)?,
            bytes
        );
        if declarations.is_some() && signatures.is_some() {
            combined = Some((publisher, replay, library));
        }
    }
    let (publisher, replay, library) = combined.ok_or("combined native replay case")?;
    let wire = serde_json::to_value(&replay)?;
    for schema in [
        "obsolete-project-replay-schema",
        "obsolete-project-replay-schema",
    ] {
        let mut changed = wire.clone();
        changed["schema"] = schema.into();
        refused_archive(&changed, &stop)?;
    }
    for field in [
        "physical_function_declarations",
        "physical_function_signatures",
    ] {
        let mut changed = wire.clone();
        changed["configuration"][field] = Value::Null;
        refused_archive(&changed, &stop)?;
        changed["configuration"][field] = "unsupported-primary-recipe".into();
        refused_archive(&changed, &stop)?;
    }
    let mut no_sidecar = wire.clone();
    no_sidecar["function_calls"] = false.into();
    refused_archive(&no_sidecar, &stop)?;
    let mut body_shape = wire.clone();
    body_shape["configuration"]["physical_function_declarations"] =
        serde_json::to_value(Declarations::DeclaredSyntaxAndBodyShapeV1)?;
    refused_archive(&body_shape, &stop)?;
    assert_eq!(
        configuration_builder()?
            .with_physical_function_declarations(Declarations::DeclaredSyntaxAndBodyShapeV1)
            .build()
            .err()
            .ok_or("primary body-shape configuration was admitted")?
            .code(),
        ProjectErrorCode::InvalidConfiguration
    );
    let mut ordinary = ProjectPublisher::new();
    let error = ordinary
        .publish_initial_cancellable(
            bundle(publisher.open_current()?.configuration().clone(), &library)?,
            &stop,
        )
        .err()
        .ok_or("selected recipes were published without the sidecar")?;
    assert_eq!(error.code(), ProjectErrorCode::InvalidConfiguration);
    assert!(ordinary.current_snapshot().is_none());

    let (records, bindings) = publication_parts(&publisher, &stop)?;
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("actual native publication replay record")?;
    let expected = PartitionRecord::new("live.project.replay", PRIMARY_STORAGE, &replay)?;
    assert_eq!(record.version(), expected.version());
    assert_eq!(record.decode::<ProjectReplay>()?, replay);
    assert!(publication::STORAGE_SCHEMAS.contains(&PRIMARY_STORAGE));
    let root = StoreRoot::new()?;
    let store = ProjectStore::create(
        root.0.join("store"),
        "fixture.primary-callables",
        catalog(publication::STORAGE_SCHEMAS)?,
    )?;
    PublicationRequest::new(
        store.epoch(),
        OperationId::new("primary-selected")?,
        None,
        bindings,
        records,
    )?;
    store.close()?;
    Ok(())
}

#[test]
fn unselected_primary_keeps_v4_and_ordinary_catalog() -> TestResult {
    let stop = AtomicBool::new(false);
    let config = configuration(None, None)?;
    let library = library(config.analyzer_binding().backend().clone())?;
    let mut publisher = ProjectPublisher::new();
    publisher.publish_initial_cancellable(bundle(config, &library)?, &stop)?;
    let view = publisher.open_current()?;
    assert!(
        view.snapshot()
            .analyzer_binding()
            .function_call_report()
            .is_none()
    );
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let wire = serde_json::to_value(&replay)?;
    assert_eq!(wire["function_calls"], false);
    assert!(
        wire["configuration"]
            .get("physical_function_declarations")
            .is_none()
    );
    assert!(
        wire["configuration"]
            .get("physical_function_signatures")
            .is_none()
    );
    let (records, bindings) = publication_parts(&publisher, &stop)?;
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("ordinary replay record")?;
    assert_eq!(
        record.version(),
        PartitionRecord::new("live.project.replay", "wow-project.live-replay", &replay)?.version()
    );
    let root = StoreRoot::new()?;
    let store = ProjectStore::create(
        root.0.join("store"),
        "fixture.primary-callables",
        catalog(publication::STORAGE_SCHEMAS)?,
    )?;
    PublicationRequest::new(
        store.epoch(),
        OperationId::new("primary-unselected")?,
        None,
        bindings,
        records,
    )?;
    store.close()?;
    Ok(())
}

#[test]
fn legacy_non_global_queries_preserve_lookup_data_and_libstub_accessors() -> TestResult {
    let stop = AtomicBool::new(false);
    let config = configuration(None, None)?;
    let library = library(config.analyzer_binding().backend().clone())?;
    let source = concat!(
        "LibStub = {}\n",
        "function LibStub.GetLibrary(major, silent) return {} end\n",
        "function LibStub.NewLibrary(major, minor) return {}, nil end\n",
        "function LibStub.EmbedLibrary(addon, major) return true end\n",
        "function Probe(addon)\n",
        "  LibStub.GetLibrary(\"FixtureLibrary-1\", true)\n",
        "  LibStub.NewLibrary(\"FixtureLibrary-1\", 1)\n",
        "  LibStub.EmbedLibrary(addon, \"FixtureLibrary-1\")\n",
        "end\n",
    );
    let bundle = ProjectInputBundle::closed(
        config,
        vec![ProjectInputFile::new("main/library-queries.lua", source)?],
        vec![library.clone()],
    )?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(bundle, &stop)?;
    let view = publisher.open_current()?;
    let report = report(&view)?;
    report.validate()?;
    assert!(report.source_health_complete());

    for api in ["LibStub.GetLibrary", "LibStub.NewLibrary"] {
        assert!(matches!(
            report.named_targets().get(api),
            Some(wow_emmy::function_calls::SourceCallTarget::MainFunction { .. })
        ));
    }
    // This unchanged non-global recipe retains its original lookup inputs.
    // Named target data does not activate a W6 embed rule. Report10's global
    // recipe selects a separate query set; deleting legacy lookup inputs here
    // would change the old analyzer session and invalidate retained snapshots.
    assert!(matches!(
        report.named_targets().get("LibStub.EmbedLibrary"),
        Some(wow_emmy::function_calls::SourceCallTarget::MainFunction { .. })
    ));
    let archive = ProjectReplay::capture(&publisher, &stop)?;
    let restored = archive.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(
        restored.snapshot().analyzer_binding(),
        view.snapshot().analyzer_binding()
    );
    let files = restored
        .snapshot()
        .analyzer_binding()
        .main_workspace()
        .files()
        .iter()
        .map(|file| ProjectInputFile::new(file.path(), file.text()))
        .collect::<wow_project::ProjectResult<Vec<_>>>()?;
    let mut restored_owner = ProjectPublisher::with_function_call_facts();
    restored_owner.publish_initial_cancellable(
        ProjectInputBundle::closed(restored.configuration().clone(), files, vec![library])?,
        &stop,
    )?;
    assert_eq!(
        serde_json::to_value(ProjectReplay::capture(&restored_owner, &stop)?)?,
        serde_json::to_value(&archive)?
    );
    assert!(report.calls().iter().any(|call| {
        call.arguments().get(1).is_some_and(|argument| {
            argument.literal()
                == Some(&wow_emmy::function_calls::SourceCallLiteral::String(
                    "FixtureLibrary-1".into(),
                ))
        })
    }));
    Ok(())
}
