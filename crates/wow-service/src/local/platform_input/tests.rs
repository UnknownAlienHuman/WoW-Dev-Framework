use super::*;
mod empty_packages;
mod function_fingerprints;
mod project_budgets;
mod xml_classification;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
    SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
};
use wow_emmy::{EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE};
use wow_project::{
    PlatformGraphCapabilities, PlatformGraphCapability,
    load::ProjectPackageVariantInput,
    platform_source::{
        PlatformLicenseRecord, PlatformLicenseState, PlatformMaterializer, PlatformRootSpec,
        PlatformSourceClass, PlatformSourceOrigin, PlatformSourceRevision, PlatformTarget,
        SourceAdmissionLimits,
    },
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
struct FixtureRoot(PathBuf);
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}
fn write_json(
    root: &FixtureRoot,
    name: &str,
    value: &impl serde::Serialize,
) -> TestResult<ProjectDiskFile> {
    let bytes = serde_json::to_vec(value)?;
    std::fs::write(root.0.join(name), &bytes)?;
    Ok(ProjectDiskFile::new(name).with_identity(digest(&bytes), bytes.len() as u64))
}

fn fixture() -> TestResult<(FixtureRoot, Value)> {
    let path = std::env::temp_dir().join(format!(
        "wow-platform-transport-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    std::fs::create_dir(&path)?;
    let root = FixtureRoot(path);
    std::fs::create_dir(root.0.join("UI"))?;
    let mut files = Vec::new();
    let mut total = 0;
    for (path, kind, text) in [
        ("UI/Fixture.toc", "toc", "main.lua\nframes.xml\n"),
        (
            "UI/frames.xml",
            "xml",
            "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Frame name=\"NativeFrame\"><Scripts><OnLoad>local loaded = true</OnLoad></Scripts></Frame></Ui>",
        ),
        (
            "UI/main.lua",
            "lua",
            "function FixtureFunction() return true end\n",
        ),
        ("version.txt", "version", "fixture-version\n"),
    ] {
        std::fs::write(root.0.join(path), text)?;
        total += text.len();
        files.push(json!({"path":path,"kind":kind,"bytes":text.len(),
            "git_blob_algorithm":"sha1","git_blob_id":"2".repeat(40),
            "content_sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}));
    }
    let mut manifest = json!({
        "schema_version":1,
        "source":{"source_id":"blizzard-ui","selector":"fixture","revision":"1".repeat(40),
            "git_object_format":"sha1","version":"fixture-version","acquisition":"local_git_object_database"},
        "selection":{"extensions":[".lua",".toc",".xml",".xsd"],"version_path":"version.txt",
            "non_regular_entries":"reject","working_tree":"ignored"},
        "coverage":{"tracked_files":4,"included_files":4,"excluded_files":0,"included_bytes":total,
            "kind_version":1,"kind_lua":1,"kind_toc":1,"kind_xml":1},"files":files,
    });
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let manifest_file = write_json(&root, "manifest.json", &manifest)?;
    let generation = ReferenceGenerationId::derive(&"manifested-platform-transport-fixture")?;
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:platform-transport".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "manifested-platform-transport-fixture",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:fixture-e0".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("manifested native platform transport")
    .build()?;
    let source_profile = BlizzardUiSourceProfileRequest {
        profile_id: "profile:fixture:platform-transport-source".parse()?,
        source_class: PlatformSourceClass::SyntheticFixture,
        target: PlatformTarget {
            product: "fixture".into(),
            channel: "fixture".into(),
            reference_profile: profile,
            reference_generation: generation,
        },
        roots: vec![PlatformRootSpec {
            root: "UI".into(),
            selected_tocs: vec!["UI/Fixture.toc".into()],
        }],
        exclusions: Vec::new(),
        limits: SourceAdmissionLimits::new(8, 1024 * 1024, 1024 * 1024, 32 * 1024)?,
    };
    let source = ManifestedPlatformInput {
        source_root: ".".into(),
        manifest: manifest_file,
        source_revision: "1".repeat(40),
        source_version: "fixture-version".into(),
        origin: PlatformSourceOrigin {
            provider: "handwritten-fixture".into(),
            repository: "native-platform-transport-fixture".into(),
            revision: PlatformSourceRevision::Fixture {
                digest: digest(&serde_json::to_vec(&manifest)?),
            },
        },
        materializer: PlatformMaterializer {
            producer: "wow.fixture_materializer".parse()?,
            version: "1.0.0".parse()?,
            configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
            report_digest: digest(b"fixture-only declaration"),
        },
        license: PlatformLicenseRecord {
            state: PlatformLicenseState::Unknown,
            attribution: "handwritten fixture".into(),
            evidence_digest: digest(b"fixture"),
        },
        compatibility_evidence: digest(b"no external compatibility attestation"),
    };
    let reference = write_json(
        &root,
        "reference.json",
        &ReferenceView::new(generation.to_string(), Vec::new(), Vec::new())?,
    )?;
    // Synthetic import evidence identifies the actual compiled backend. This
    // fixture does not establish a moving donor/public-surface qualification.
    let mut report = json!({
        "schema":"wow-dev-framework/emmylua-compatibility-report","schema_version":1,"producer_version":1,
        "source":{"revision":EMMYLUA_REVISION,"tree":EMMYLUA_TREE,"relation":"unverified_current","network_checked":false},
        "analysis_crate":{"manifest_path":"crates/emmylua_code_analysis/Cargo.toml","name":"emmylua_code_analysis","version":EMMYLUA_CODE_ANALYSIS_VERSION},
        "surface":{"files":[],"symbols":[],"surface_sha256":format!("sha256:{}","3".repeat(64))},
        "compatibility":{"required_symbols":[],"missing_symbols":[],"status":"compatible"},
    });
    report["report_sha256"] = json!(format!(
        "sha256:{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&report)?)
    ));
    let report = write_json(&root, "compatibility.json", &report)?;
    let capabilities = PlatformGraphCapabilities::new(&[
        PlatformGraphCapability::InventorySpansV1,
        PlatformGraphCapability::StructuralRolesV1,
        PlatformGraphCapability::XmlSourceMapsV1,
    ])?;
    let config = json!({"schema":LOCAL_PLATFORM_SCHEMA,"project_id":"fixture-platform-transport",
        "workspace_id":"workspace:platform-transport","source_origin_id":"project-origin:platform-transport",
        "logical_root":"fixtures/platform-transport","source_profile":source_profile,"source":source,
        "reference_view":reference,"analyzer":{"compatibility_report":report,"accepted_pin_id":format!("emmy-pin:{EMMYLUA_REVISION}"),
            "configuration_digest":ContentDigest::<CanonicalResult>::from_bytes([3;32]),"contract_id":"wow-emmy/e0-c/1",
            "fixture_contract_id":"wow-emmy/e0-c/1","library_contract_id":"wow-emmy-e0-c-library-v1"},
        "packages":[ProjectPackageInput::new("Fixture","UI",true,vec![ProjectPackageVariantInput::new(ProjectDiskFile::new("Fixture.toc"),true)])],
        "graph_profile":PlatformGraphProfile::DirectCapabilitiesV1(capabilities)});
    write_json(&root, "project.json", &config)?;
    Ok((root, config))
}

fn native_fixture() -> TestResult<(FixtureRoot, Value)> {
    let (root, mut config) = fixture()?;
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let api_root = "Blizzard_APIDocumentationGenerated";
    std::fs::create_dir(root.0.join(api_root))?;
    std::fs::write(root.0.join("version.txt"), "1.0.0.1\n")?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    files.retain(|file| file["path"] != "version.txt");
    for (path, kind, text) in [
        (
            "Blizzard_APIDocumentationGenerated/API.lua",
            "generated_api",
            "APIDocumentation:AddDocumentationTable({Name=\"Transport\",Type=\"System\",Namespace=\"C_Transport\",Environment=\"All\",Functions={{Name=\"Read\",Type=\"Function\"}}})",
        ),
        (
            "Blizzard_APIDocumentationGenerated/Bad.lua",
            "generated_api",
            "return unknown()",
        ),
        (
            "Blizzard_APIDocumentationGenerated/Generated.toc",
            "toc",
            "## Interface: 1\nAPI.lua\nBad.lua\n",
        ),
        ("version.txt", "version", "1.0.0.1\n"),
    ] {
        std::fs::write(root.0.join(path), text)?;
        files.push(json!({"path":path,"kind":kind,"bytes":text.len(),
            "git_blob_algorithm":"sha1","git_blob_id":"2".repeat(40),
            "content_sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}));
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let bytes: u64 = files
        .iter()
        .map(|file| file["bytes"].as_u64().unwrap_or(0))
        .sum();
    let count = files.len();
    manifest["source"]["version"] = json!("1.0.0.1");
    manifest["coverage"] = json!({"tracked_files":count,"included_files":count,
        "excluded_files":0,"included_bytes":bytes,"kind_version":1,"kind_lua":1,
        "kind_toc":2,"kind_xml":1,"kind_generated_api":2});
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let manifest_file = write_json(&root, "manifest.json", &manifest)?;
    let native = json!({"profile":{"profile_id":"profile:wow:platform-transport-native",
        "flavor":"retail","interface":1,"client_version":"1.0.0","client_build":1,
        "revision":"1".repeat(40),"environment":"Mainline"},
        "native_source":{"root":".","manifest":manifest_file,
            "toc":"Blizzard_APIDocumentationGenerated/Generated.toc"}});
    let mut reference_config = native.clone();
    reference_config["schema"] = json!(super::super::SOURCE_REFERENCE_INPUT_SCHEMA);
    write_json(&root, "source-reference.json", &reference_config)?;
    let expected = super::super::read_local_source_reference(
        &root.0.join("source-reference.json"),
        &AtomicBool::new(false),
    )?;
    // This fixture asserts mirror metadata solely to exercise the real Release
    // owner path. It does not attest Git membership or a current Blizzard source.
    config["source_profile"]["profile_id"] = json!("profile:wow:platform-transport-source");
    config["source_profile"]["source_class"] = json!("vendor_ui_source_mirror");
    config["source_profile"]["target"]["reference_profile"] =
        serde_json::to_value(expected.profile())?;
    config["source_profile"]["target"]["reference_generation"] =
        serde_json::to_value(expected.reference_generation())?;
    config["source"]["manifest"] = serde_json::to_value(manifest_file)?;
    config["source"]["source_version"] = json!("1.0.0.1");
    config["source"]["origin"]["revision"] = json!({"kind":"git","object_format":"sha1",
        "commit":"1".repeat(40),"tree":"3".repeat(40)});
    config
        .as_object_mut()
        .ok_or("configuration object")?
        .remove("reference_view");
    config["native_reference"] = native;
    write_json(&root, "reference.json", &expected.reference().view)?;
    write_json(&root, "project.json", &config)?;
    Ok((root, config))
}

#[test]
fn native_library_inputs_requalify_after_published_source_absent_reopen() -> TestResult {
    use crate::{
        graph::GraphBuildRequest,
        live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project},
    };
    use wow_project::{ProjectPublisher, replay::ProjectReplay};
    use wow_store::project::ReadSelector;
    let (root, mut config) = native_fixture()?;
    // Reach the real analyzer Library target in this same bounded native fixture.
    let main = "function FixtureFunction() return C_Transport.Read() end\n";
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"]
        .as_array_mut()
        .ok_or("fixture manifest files")?;
    let member = files
        .iter_mut()
        .find(|file| file["path"] == "UI/main.lua")
        .ok_or("fixture Main manifest member")?;
    member["bytes"] = json!(main.len());
    member["content_sha256"] = json!(format!("{:x}", Sha256::digest(main.as_bytes())));
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("member bytes")?)
                .ok_or("manifest source byte sum")?)
        })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("fixture manifest")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    std::fs::write(root.0.join("UI/main.lua"), main)?;
    let manifested = write_json(&root, "manifest.json", &manifest)?;
    config["native_reference"]["native_source"]["manifest"] = serde_json::to_value(manifested)?;
    let stop = AtomicBool::new(false);
    let native_manifest = json!({
        "schema": crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id": config["project_id"], "workspace_id": config["workspace_id"],
        "source_origin_id": config["source_origin_id"], "logical_root": config["logical_root"],
        "profile": config["native_reference"]["profile"], "analyzer": config["analyzer"],
        "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
        "native_source": config["native_reference"]["native_source"]
    });
    write_json(&root, "native-project.json", &native_manifest)?;
    let input = LocalProjectInput::from_config_path(&root.0.join("native-project.json"), &stop)?;
    let reference = serde_json::to_value(input.reference_view())?;
    let artifact = input
        .native_annotation_artifact()
        .ok_or("native artifact")?
        .artifact_id()
        .to_owned();
    let sidecars = input
        .native_annotation_sidecars()
        .ok_or("native sidecars")?;
    let map_bytes = sidecars.source_map_bytes().to_vec();
    let loss_bytes = sidecars.loss_report_bytes().to_vec();
    let expected_bindings = input
        .native_callable_bindings()
        .ok_or("native bindings")?
        .bindings()
        .iter()
        .map(|b| {
            (
                b.generated_path().to_owned(),
                b.generated_sha256().to_owned(),
                b.generated(),
                b.mapping_id().to_owned(),
                b.source_id().to_owned(),
                b.key().to_owned(),
                b.record_digest().to_owned(),
                b.status(),
                b.ambiguous_origin(),
            )
        })
        .collect::<Vec<_>>();
    let input_digest = input
        .bundle
        .configuration()
        .native_library_input()
        .ok_or("producer input")?
        .digest();
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let archive = serde_json::to_value(&replay)?;
    assert_eq!(
        archive["schema"],
        "wow-project/native-project-replay/native-library-input/1"
    );
    // Selected input cannot be silently dropped, relabeled or null-decoded.
    let mut dropped = archive.clone();
    dropped["configuration"]
        .as_object_mut()
        .ok_or("replay config")?
        .remove("native_library_input");
    assert!(ProjectReplay::from_json(&serde_json::to_vec(&dropped)?, &stop).is_err());
    let mut null = archive.clone();
    null["configuration"]["native_library_input"] = Value::Null;
    assert!(ProjectReplay::from_json(&serde_json::to_vec(&null)?, &stop).is_err());
    let mut old = archive.clone();
    old["schema"] = json!("wow-project/native-project-replay/4");
    assert!(ProjectReplay::from_json(&serde_json::to_vec(&old)?, &stop).is_err());
    let original_snapshot = publisher.open_current()?.snapshot_id().to_owned();
    let original_analyzer = publisher.open_current()?.analyzer_snapshot_id().to_owned();
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let store_root = FixtureRoot(root.0.with_extension("native-library-store"));
    assert!(!store_root.0.exists());
    publish_local_project(
        &root.0.join("native-project.json"),
        &request,
        &store_root.0,
        &LiveProjectPublishRequest::new("fixture-native-library-input", "absent", true, true)?,
        &stop,
    )?;
    drop(input);
    drop(publisher);
    std::fs::remove_dir_all(&root.0)?;
    assert!(!root.0.exists());
    let decoded = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    let hydrated = decoded.hydrate(&stop)?;
    assert_eq!(hydrated.snapshot_id(), original_snapshot);
    assert_eq!(hydrated.analyzer_snapshot_id(), original_analyzer);
    assert_eq!(
        hydrated
            .configuration()
            .native_library_input()
            .ok_or("hydrated producer input")?
            .digest(),
        input_digest
    );
    let store = LiveProjectStore::open(&store_root.0)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    let rebuilt = read.requalify_native_library(&stop)?;
    assert_eq!(read.project().snapshot_id(), original_snapshot);
    assert_eq!(serde_json::to_value(&rebuilt.reference().view)?, reference);
    assert_eq!(rebuilt.input_failure_count(), 1);
    assert!(!rebuilt.reference().negative_authority);
    assert_eq!(rebuilt.artifact().artifact_id(), artifact);
    assert_eq!(rebuilt.sidecars().source_map_bytes(), map_bytes);
    assert_eq!(rebuilt.sidecars().loss_report_bytes(), loss_bytes);
    let actual_bindings = rebuilt
        .callable_bindings()
        .bindings()
        .iter()
        .map(|b| {
            (
                b.generated_path().to_owned(),
                b.generated_sha256().to_owned(),
                b.generated(),
                b.mapping_id().to_owned(),
                b.source_id().to_owned(),
                b.key().to_owned(),
                b.record_digest().to_owned(),
                b.status(),
                b.ambiguous_origin(),
            )
        })
        .collect::<Vec<_>>();
    assert!(!actual_bindings.is_empty());
    assert_eq!(actual_bindings, expected_bindings);
    use wow_emmy::function_calls::SourceCallTarget;
    let report = read
        .project()
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("reopened native callable report")?;
    let mut targets = report
        .calls()
        .iter()
        .filter_map(|call| match call.target() {
            SourceCallTarget::LibraryFunction { target } => Some(target),
            _ => None,
        });
    let target = targets
        .next()
        .ok_or("missing genuine analyzer Library closure target")?
        .clone();
    assert!(
        targets.next().is_none(),
        "fixture must have one Library call"
    );
    assert_eq!(target.role, "library");
    let resolver = rebuilt.target_resolver(read.project(), &stop)?;
    let super::super::NativeApiTargetResolution::Bound(association) =
        resolver.resolve_target(&target, &stop)?
    else {
        return Err("genuine native Library target did not bind".into());
    };
    let (origin, record) = (association.binding(), association.reference_record());
    assert_eq!(association.target(), &target);
    let expected_origins = rebuilt.callable_bindings().bindings();
    assert_eq!(expected_origins.len(), 1);
    let expected_origin = &expected_origins[0];
    assert_eq!(origin.key(), expected_origin.key());
    assert_eq!(origin.mapping_id(), expected_origin.mapping_id());
    assert_eq!(origin.source(), expected_origin.source());
    assert_eq!(origin.source_id(), expected_origin.source_id());
    assert_eq!(origin.generated_path(), target.path);
    assert_eq!(origin.generated_sha256(), target.content_digest);
    assert!(!origin.ambiguous_origin());
    let wow_reference::LookupResult::Found(expected_record) = rebuilt.reference().view.lookup(
        wow_reference::native_view::NATIVE_API_PARTITION,
        expected_origin.key(),
    ) else {
        return Err("original native callable is not Found".into());
    };
    assert_eq!(record, expected_record);
    assert_eq!(record.digest()?, origin.record_digest());
    assert_eq!(record.source_ids()[0].as_ref(), origin.source_id());

    // Mutate only the received SymbolTarget; never rebuild the native fixture.
    let mut wrong_role = target.clone();
    wrong_role.role = "main";
    let mut wrong_snapshot = target.clone();
    wrong_snapshot.workspace_id.push_str("-stale");
    let mut wrong_path = target.clone();
    wrong_path.path.push_str(".missing");
    let mut wrong_digest = target.clone();
    wrong_digest.content_digest = format!("sha256:{}", "0".repeat(64));
    assert_ne!(wrong_digest.content_digest, target.content_digest);
    let file = rebuilt
        .sidecars()
        .source_map()
        .files()
        .iter()
        .find(|file| file.path() == target.path)
        .ok_or("actual generated Library file")?;
    let mut wrong_span = target.clone();
    wrong_span.span = wow_core::SourceSpan::byte_range(
        0,
        file.byte_length()
            .checked_add(1)
            .ok_or("Library span overflow")?,
    )?;
    for (label, invalid) in [
        ("role", wrong_role),
        ("snapshot", wrong_snapshot),
        ("path", wrong_path),
        ("digest", wrong_digest),
        ("span", wrong_span),
    ] {
        assert!(
            resolver.resolve_target(&invalid, &stop).is_err(),
            "mismatched Library target {label} admitted"
        );
    }
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        read.requalify_native_library(&cancelled)
            .err()
            .ok_or("pre-cancel admitted")?
            .code(),
        crate::ServiceErrorCode::Cancelled
    );
    assert_eq!(
        resolver
            .resolve_target(&target, &cancelled)
            .err()
            .ok_or("pre-cancel target resolution admitted")?
            .code(),
        crate::ServiceErrorCode::Cancelled
    );
    drop(rebuilt);
    read.close()?;
    store.close()?;
    Ok(())
}

#[test]
fn native_reference_composes_exact_platform_input_without_losing_partial_coverage() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    let (root, config) = native_fixture()?;
    let stop = AtomicBool::new(false);
    let path = root.0.join("project.json");
    let expected =
        super::super::read_local_source_reference(&root.0.join("source-reference.json"), &stop)?;
    assert!(expected.is_partial());
    assert!(!expected.reference().negative_authority);
    assert!(matches!(
        expected.reference().view.lookup(
            wow_reference::native_view::NATIVE_API_PARTITION,
            "function:C_Transport.Read"
        ),
        wow_reference::LookupResult::Found(_)
    ));
    {
        // Reuse the same source/TOC and Main bytes through the native manifest
        // loader. The platform loader below does not retain annotation bindings.
        let native_manifest = json!({
            "schema": crate::local::LOCAL_NATIVE_SCHEMA,
            "project_id": config["project_id"],
            "workspace_id": config["workspace_id"],
            "source_origin_id": config["source_origin_id"],
            "logical_root": config["logical_root"],
            "profile": config["native_reference"]["profile"],
            "analyzer": config["analyzer"],
            "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
            "native_source": config["native_reference"]["native_source"]
        });
        let directory = ProjectInputDirectory::open(&root.0)?;
        let native_input = LocalProjectInput::from_native_manifest(
            &serde_json::to_vec(&native_manifest)?,
            &directory,
            &stop,
        )?;
        assert_eq!(native_input.reference_view(), &expected.reference().view);
        let bindings = native_input
            .native_callable_bindings()
            .ok_or("missing retained native callable bindings")?;
        let source_map = native_input
            .native_annotation_sidecars()
            .ok_or("missing retained native annotation sidecars")?
            .source_map();
        assert_eq!(bindings.source_map_id(), source_map.source_map_id());
        assert_eq!(
            bindings.annotation_artifact_id(),
            source_map.annotation_artifact_id()
        );
        assert_eq!(bindings.profile_id(), source_map.profile_id());
        assert_eq!(bindings.revision(), source_map.revision());
        assert_eq!(
            bindings.reference_generation(),
            native_input.reference_view().generation_id()
        );
        assert_eq!(
            bindings.reference_generation(),
            source_map.source_generation_id()
        );
        assert_eq!(
            bindings.reference_digest(),
            native_input.reference_view().self_digest()
        );
        let mut matching = bindings
            .bindings()
            .iter()
            .filter(|binding| binding.key() == "function:C_Transport.Read");
        let binding = matching.next().ok_or("missing original Read binding")?;
        assert!(matching.next().is_none());
        assert_eq!(
            binding.status(),
            wow_annotations::reference_links::NativeCallableReferenceStatus::Found
        );
        let wow_reference::LookupResult::Found(callable) = native_input.reference_view().lookup(
            wow_reference::native_view::NATIVE_API_PARTITION,
            binding.key(),
        ) else {
            return Err("original Read callable is not Found".into());
        };
        assert_eq!(binding.record_digest(), callable.digest()?);
        assert_eq!(callable.source_ids().len(), 1);
        assert_eq!(callable.source_ids()[0].as_ref(), binding.source_id());
        let original = expected
            .reference()
            .sources
            .iter()
            .find(|source| source.id == binding.source_id())
            .ok_or("missing original callable source")?;
        assert_eq!(binding.source().scope(), None);
        assert_eq!(binding.source().path(), original.path);
        assert_eq!(binding.source().sha256(), original.sha256);
        assert_eq!(
            binding.source().span().start(),
            u64::try_from(original.span.start)?
        );
        assert_eq!(
            binding.source().span().end(),
            u64::try_from(original.span.end)?
        );
        let mapped_file = source_map
            .files()
            .iter()
            .find(|file| file.path() == binding.generated_path())
            .ok_or("missing bound generated file")?;
        assert_eq!(binding.generated_sha256(), mapped_file.sha256());
        let mapping = mapped_file
            .mappings()
            .iter()
            .find(|mapping| mapping.mapping_id() == binding.mapping_id())
            .ok_or("missing bound declaration mapping")?;
        assert_eq!(mapping.granularity(), "declaration");
        assert_eq!(binding.generated(), mapping.generated());
        assert_eq!(binding.source(), mapping.source());
    }
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    assert_eq!(input.reference_view(), &expected.reference().view);
    assert_eq!(
        input.project_bundle().configuration().selected_profile(),
        expected.profile()
    );
    assert_eq!(
        input.project_bundle().libraries()[0].universe(),
        wow_emmy::LuaWorkspaceUniverse::BlizzardUi
    );
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    // The positive graph consumes the retained project bytes. Failed generated
    // documentation remains Partial in the exact Reference view handed to it.
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{}",
        serde_json::to_string(&result)?
    );
    assert!(result.snapshot_bytes()?.is_some());

    let reference_file = write_json(&root, "reference.json", &expected.reference().view)?;
    let mut artifact = config.clone();
    artifact
        .as_object_mut()
        .ok_or("configuration object")?
        .remove("native_reference");
    artifact["reference_view"] = serde_json::to_value(&reference_file)?;
    write_json(&root, "project.json", &artifact)?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &stop)?.reference_view(),
        &expected.reference().view
    );

    let alias = write_json(
        &root,
        "alias-manifest.json",
        &serde_json::from_slice::<Value>(&std::fs::read(root.0.join("manifest.json"))?)?,
    )?;
    let wrong_generation = serde_json::to_value(ReferenceGenerationId::derive(
        &"different-native-reference",
    )?)?;
    for (pointer, value) in [
        ("/reference_view", serde_json::to_value(&reference_file)?),
        ("/reference_view", Value::Null),
        ("/native_reference", Value::Null),
        ("/native_reference/native_source/root", json!("UI")),
        (
            "/native_reference/native_source/manifest",
            serde_json::to_value(alias)?,
        ),
        ("/native_reference/native_source/files", json!([])),
        ("/native_reference/profile/revision", json!("4".repeat(40))),
        ("/native_reference/profile/client_build", json!(2)),
        ("/native_reference/profile/interface", json!(2)),
        ("/native_reference/profile/environment", json!("Glue")),
        (
            "/source_profile/target/reference_profile/profile_id",
            json!("profile:wow:different-native-profile"),
        ),
        (
            "/source_profile/target/reference_generation",
            wrong_generation,
        ),
    ] {
        let mut changed = config.clone();
        if pointer == "/reference_view" {
            changed["reference_view"] = value;
        } else if pointer.ends_with("/files") {
            changed["native_reference"]["native_source"]["files"] = value;
        } else {
            *changed.pointer_mut(pointer).ok_or("mutation pointer")? = value;
        }
        write_json(&root, "project.json", &changed)?;
        assert_eq!(
            LocalProjectInput::from_config_path(&path, &stop)
                .err()
                .ok_or("native mismatch admitted")?
                .code(),
            crate::ServiceErrorCode::InvalidConfiguration,
            "{pointer}"
        );
    }
    for remove_toc in [false, true] {
        let mut changed = config.clone();
        let object = if remove_toc {
            changed["native_reference"]["native_source"]
                .as_object_mut()
                .ok_or("native source object")?
        } else {
            changed.as_object_mut().ok_or("configuration object")?
        };
        object.remove(if remove_toc {
            "toc"
        } else {
            "native_reference"
        });
        write_json(&root, "project.json", &changed)?;
        assert_eq!(
            LocalProjectInput::from_config_path(&path, &stop)
                .err()
                .ok_or("missing native selection admitted")?
                .code(),
            crate::ServiceErrorCode::InvalidConfiguration
        );
    }
    write_json(&root, "project.json", &config)?;
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &cancelled)
            .err()
            .ok_or("cancelled native input admitted")?
            .code(),
        crate::ServiceErrorCode::Cancelled
    );
    std::fs::write(
        root.0.join("Blizzard_APIDocumentationGenerated/API.lua"),
        "changed",
    )?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &stop)
            .err()
            .ok_or("changed API bytes admitted")?
            .code(),
        crate::ServiceErrorCode::InvalidConfiguration
    );
    Ok(())
}

#[test]
fn manifested_platform_input_reaches_native_selected_graph_and_refuses_mutations() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    let (root, config) = fixture()?;
    let stop = AtomicBool::new(false);
    let path = root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let legacy = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?;
    let refused = execute_graph_build(input, &legacy, &stop)?;
    assert_eq!(refused.status(), GraphReadStatus::Failed);
    assert_eq!(
        serde_json::to_value(&refused)?["failure"],
        serde_json::to_value(crate::ServiceErrorCode::IdentityMismatch)?
    );
    let mut mismatched = config.clone();
    mismatched["source_profile"]["target"]["reference_generation"] =
        serde_json::to_value(ReferenceGenerationId::derive(&"different-reference")?)?;
    write_json(&root, "project.json", &mismatched)?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &stop)
            .err()
            .ok_or("source target/reference mismatch admitted")?
            .code(),
        crate::ServiceErrorCode::InvalidConfiguration
    );
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    assert_eq!(
        input.project_bundle().configuration().project_kind(),
        wow_project::ProjectKind::BlizzardUiPlatformSource
    );
    assert_eq!(input.project_bundle().inventory().files().len(), 1);
    let request = legacy.with_input_profile(&input);
    assert_eq!(
        serde_json::to_value(&request)?["schema"],
        "wow-service/graph-build-request/13"
    );
    // Once admitted, graph execution requires the retained bytes, not reopening
    // the original files. An inline body must become an actual native map node.
    std::fs::remove_file(root.0.join("UI/frames.xml"))?;
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(result.status(), GraphReadStatus::Partial);
    let snapshot: wow_graph::GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("native snapshot missing")?)?;
    assert!(
        snapshot
            .snapshot()
            .nodes()
            .iter()
            .any(|node| node.kind() == "xml_source_virtual_lua_unit")
    );
    assert!(
        snapshot
            .snapshot()
            .nodes()
            .iter()
            .any(|node| node.kind() == "xml_source_virtual_lua_map_piece")
    );
    assert!(LocalProjectInput::from_config_path(&path, &stop).is_err());
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &cancelled)
            .err()
            .ok_or("cancelled input admitted")?
            .code(),
        crate::ServiceErrorCode::Cancelled
    );
    let mut changed = config;
    changed["graph_profile"] = json!({"direct_capabilities_v1":["xml_source_maps_v1"]});
    write_json(&root, "project.json", &changed)?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &stop)
            .err()
            .ok_or("invalid dependency set admitted")?
            .code(),
        crate::ServiceErrorCode::InvalidConfiguration
    );
    Ok(())
}
