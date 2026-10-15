use super::*;
mod builder_attempts;
mod empty_packages;
mod event_registry_keys;
mod event_registry_receivers;
mod function_fingerprints;
mod native_type_resolution;
mod project_budgets;
mod w5_secure_hooks;
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

// Assertions over the sealed original producer value; no payload decoding.
fn assert_transport_signature(
    signature: wow_reference::native_view::NativeCallableSignature<'_>,
) -> TestResult<Value> {
    let value = serde_json::to_value(signature)?;
    assert_eq!(
        value["profile"],
        "wow-reference/native-source-callable-signature/1"
    );
    let data = &value["signature"];
    assert_eq!(data["name"], "Read");
    assert_eq!(data["namespace"], "C_Transport");
    assert_eq!(
        data["type_qualification"],
        "original_source_type_identities_unresolved"
    );
    assert!(data.get("documentation").is_none());
    let arguments = data["arguments"]
        .as_array()
        .ok_or("original signature arguments")?;
    let returns = data["returns"]
        .as_array()
        .ok_or("original signature returns")?;
    assert_eq!(arguments.len(), 1);
    assert_eq!(returns.len(), 1);
    let argument = &arguments[0];
    let returned = &returns[0];
    assert_eq!(argument["name"], "flag");
    assert_eq!(argument["type_name"], "bool");
    assert_eq!(argument["nilable"], false);
    assert_eq!(argument["default"]["kind"], json!({"Boolean": false}));
    assert_eq!(returned["name"], "value");
    assert_eq!(returned["type_name"], "number");
    assert_eq!(returned["nilable"], true);
    assert!(returned.get("default").is_none());
    for field in [argument, returned] {
        assert!(field.get("inner_type").is_none());
        assert!(field.get("stride_index").is_none());
        let span = &field["source_span"];
        assert!(
            span["start"].as_u64().ok_or("field span start")?
                < span["end"].as_u64().ok_or("field span end")?
        );
    }
    let default_span = &argument["default"]["span"];
    let start = default_span["start"].as_u64().ok_or("default span start")?;
    let end = default_span["end"].as_u64().ok_or("default span end")?;
    assert_eq!(
        end.checked_sub(start),
        Some(5),
        "original false literal span"
    );
    assert!(
        start
            >= argument["source_span"]["start"]
                .as_u64()
                .ok_or("argument start")?
    );
    assert!(
        end <= argument["source_span"]["end"]
            .as_u64()
            .ok_or("argument end")?
    );
    Ok(value)
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
            "APIDocumentation:AddDocumentationTable({Name=\"Transport\",Type=\"System\",Namespace=\"C_Transport\",Environment=\"All\",Functions={{Name=\"Read\",Type=\"Function\",Arguments={{Name=\"flag\",Type=\"bool\",Nilable=false,Default=false}},Returns={{Name=\"value\",Type=\"number\",Nilable=true}}}}})",
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
    let signature = rebuilt.reference().callable_signature(record, &stop)?;
    assert_eq!(
        signature.reference_generation(),
        rebuilt.reference().view.generation_id()
    );
    assert_eq!(
        signature.reference_view_digest(),
        rebuilt.reference().view.self_digest()
    );
    assert_eq!(signature.record_key(), origin.key());
    assert_eq!(signature.record_digest(), origin.record_digest());
    assert_eq!(signature.source_id(), origin.source_id());
    assert_transport_signature(signature)?;
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
    // Requalified Library input alone is not the selected accepted API proof.
    use crate::context::{
        CONTEXT_MAP_HOST_SCHEMA, ContextInspectLevel, ContextInspectRequest, ContextL1Profile,
        ContextMapHostInput, ContextMapPublication, ContextMapRoot, ContextMapStatus,
        execute_context_inspect,
    };
    assert!(read.native_api().is_none());
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("unselected Main provenance")?;
    let caller = provenance
        .functions()
        .iter()
        .find(|function| function.function_id == report.calls()[0].caller_function_id())
        .ok_or("unselected actual physical caller")?;
    let graph_read = read.pair().graph_read(&stop)?;
    let node = graph_read.published_entity(
        wow_project::graph::SOURCE_GRAPH_PARTITION,
        &caller.proposal_id,
        &stop,
    )?;
    let host = ContextMapHostInput::from_json_slice(
        &serde_json::to_vec(&json!({
            "schema": CONTEXT_MAP_HOST_SCHEMA,
            "project_store_id": "fixture-native-library-reference-refusal",
            "project_store_root": store_root.0,
            "project_id": read.project().configuration().project_id(),
            "reference_profile": rebuilt.profile(),
            "reference": rebuilt.reference().view,
            "limits": {
                "neighbors": wow_graph::GraphNeighborReadLimits::default(),
                "explain": wow_graph::GraphExplainLimits {
                    max_output_bytes: 64 * 1024,
                    ..wow_graph::GraphExplainLimits::default()
                },
                "evidence": wow_graph::GraphEvidenceResolveLimits::default(),
                "max_direct_edges": 16,
                "max_input_bytes": 4 * 1024 * 1024,
                "max_metadata_bytes": 4 * 1024 * 1024,
                "max_context_work": 8_000_000
            }
        }))?,
        &stop,
    )?;
    let request = ContextInspectRequest::primary(
        "fixture-native-library-reference-refusal",
        ContextMapPublication::store_generation(read.store_generation_id().as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(node.node_id().as_str())?,
        None,
        None,
    )?
    .with_l1_profile(ContextL1Profile::ReferenceFactsV1)?;
    drop(resolver);
    drop(rebuilt);
    read.close()?;
    store.close()?;
    let missing_proof = execute_context_inspect(&host, &request, &stop)?;
    assert_eq!(missing_proof.status(), ContextMapStatus::Failed);
    assert_eq!(
        missing_proof.failure_code(),
        Some(crate::ServiceErrorCode::ComponentUnavailable)
    );
    let refused: Value = serde_json::from_slice(missing_proof.canonical_bytes())?;
    assert_eq!(refused["resource_closure"], "closed");
    assert!(refused.get("payload").is_none());
    assert!(!root.0.exists());
    Ok(())
}

#[test]
fn selected_native_api_uses_survive_published_source_absent_reopen() -> TestResult {
    use crate::{
        ServiceErrorCode,
        context::{
            CONTEXT_MAP_HOST_SCHEMA, ContextBuildRequest, ContextInspectLevel,
            ContextInspectRequest, ContextL1Profile, ContextMapHostInput, ContextMapPublication,
            ContextMapReferenceGuard, ContextMapResult, ContextMapRoot, ContextMapStatus,
            ContextRenderSelection, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
            PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID, PRIMARY_CONTEXT_PACK_PROFILE_ID,
            execute_context_build, execute_context_inspect,
        },
        graph::GraphBuildRequest,
        live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project},
    };
    use wow_emmy::function_calls::SourceCallTarget;
    use wow_graph::{
        GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborReadLimits,
        GraphProposalValue, GraphRelationKind,
    };
    use wow_project::{NativeApiUseRecipe, ProjectPublisher, replay::ProjectReplay};
    use wow_store::project::ReadSelector;

    let (root, mut config) = native_fixture()?;
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
    // Updating the manifest bytes also updates its admitted disk descriptor.
    config["native_reference"]["native_source"]["manifest"] = serde_json::to_value(manifested)?;
    let native_manifest = json!({
        "schema": crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id": config["project_id"], "workspace_id": config["workspace_id"],
        "source_origin_id": config["source_origin_id"], "logical_root": config["logical_root"],
        "profile": config["native_reference"]["profile"], "analyzer": config["analyzer"],
        "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
        "native_source": config["native_reference"]["native_source"],
        "native_api_uses": "reference_v1"
    });
    write_json(&root, "native-project.json", &native_manifest)?;
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("native-project.json"), &stop)?;
    assert_eq!(
        input.bundle.configuration().native_api_uses(),
        Some(NativeApiUseRecipe::ReferenceV1)
    );
    let expected_reference = serde_json::to_value(input.reference_view())?;
    let bindings = input
        .native_callable_bindings()
        .ok_or("genuine native callable origins")?;
    assert_eq!(bindings.bindings().len(), 1);
    let origin = &bindings.bindings()[0];
    assert!(!origin.ambiguous_origin());
    let expected_source = origin.source().clone();
    let expected_source_id = origin.source_id().to_owned();
    let expected_key = origin.key().to_owned();
    let expected_record_digest = origin.record_digest().to_owned();
    let expected_mapping = origin.mapping_id().to_owned();
    let expected_generation = bindings.reference_generation().to_owned();
    let expected_view_digest = bindings.reference_digest().to_owned();
    let expected_profile = input
        .bundle
        .configuration()
        .selected_profile()
        .profile_id()
        .to_owned();

    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let archive = serde_json::to_value(&replay)?;
    assert_eq!(archive["configuration"]["native_api_uses"], "reference_v1");
    const UNSELECTED_REPLAY: &str = "wow-project/native-project-replay/native-library-input/1";
    assert_eq!(
        archive["schema"],
        "wow-project/native-project-replay/native-api-uses/1"
    );
    let mut dropped = archive.clone();
    dropped["configuration"]
        .as_object_mut()
        .ok_or("replay configuration")?
        .remove("native_api_uses");
    let mut null = archive.clone();
    null["configuration"]["native_api_uses"] = Value::Null;
    let mut downgraded = archive.clone();
    downgraded["schema"] = json!(UNSELECTED_REPLAY);
    for (label, invalid) in [("drop", dropped), ("null", null), ("downgrade", downgraded)] {
        assert!(
            ProjectReplay::from_json(&serde_json::to_vec(&invalid)?, &stop).is_err(),
            "selected native API replay {label} admitted"
        );
    }
    let original_snapshot = publisher.open_current()?.snapshot_id().to_owned();
    let original_analyzer = publisher.open_current()?.analyzer_snapshot_id().to_owned();
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    // Own a fresh parent only after creation; ProjectStore::create needs an absent child.
    let store_parent_path = root.0.with_extension("selected-native-api-store");
    std::fs::create_dir(&store_parent_path)?;
    let store_parent = FixtureRoot(store_parent_path);
    let store_path = store_parent.0.join("store");
    publish_local_project(
        &root.0.join("native-project.json"),
        &request,
        &store_path,
        &LiveProjectPublishRequest::new("fixture-selected-native-api", "absent", true, true)?,
        &stop,
    )?;
    // A valid generic graph with a removed semantic output must not activate.
    // Reconstructing the real producers, rather than trusting stored acceptance,
    // is the boundary this selected publication adds.
    let mut store = LiveProjectStore::open(&store_path)?;
    let previous = store.current()?.ok_or("selected native API current")?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    let graph = read.graph();
    let calls = graph
        .partition(wow_recognizers::source_calls::SOURCE_CALL_PARTITION)
        .ok_or("selected call producer")?;
    let empty_calls = wow_graph::GraphProposalBatch::build(
        graph.registry().bundle_id(),
        graph.registry().registry_digest(),
        graph.foundation().universe().clone(),
        graph.foundation().generation().clone(),
        graph.source_context_id(),
        calls.partition_id(),
        Vec::new(),
        Vec::new(),
    )?;
    let altered = graph.prepare_replacement(
        wow_graph::GraphPartitionReplacement {
            expected_snapshot_id: graph.snapshot().snapshot_id().clone(),
            expected_partition_digest: Some(calls.partition_digest().into()),
            producer_version: calls.producer_version().into(),
            batch: empty_calls,
            coverage: calls.coverage().to_vec(),
        },
        &stop,
    )?;
    read.close()?;
    let error = store
        .publish(
            &publisher,
            altered.candidate(),
            "fixture-selected-native-api-altered",
            Some(previous.record_id.clone()),
            &stop,
        )
        .err()
        .ok_or("altered native call producer activated")?;
    assert_eq!(error.code(), crate::ServiceErrorCode::IdentityMismatch);
    assert_eq!(store.current()?, Some(previous));
    store.close()?;
    drop(input);
    drop(publisher);
    std::fs::remove_dir_all(&root.0)?;
    assert!(!root.0.exists());

    let hydrated =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(hydrated.snapshot_id(), original_snapshot);
    assert_eq!(hydrated.analyzer_snapshot_id(), original_analyzer);
    assert_eq!(
        hydrated.configuration().native_api_uses(),
        Some(NativeApiUseRecipe::ReferenceV1)
    );
    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), original_snapshot);
    assert_eq!(read.project().analyzer_snapshot_id(), original_analyzer);
    assert_eq!(
        read.project().configuration().native_api_uses(),
        Some(NativeApiUseRecipe::ReferenceV1)
    );
    let rebuilt = read.requalify_native_library(&stop)?;
    assert_eq!(
        serde_json::to_value(&rebuilt.reference().view)?,
        expected_reference
    );
    assert_eq!(rebuilt.input_failure_count(), 1);
    assert!(!rebuilt.reference().negative_authority);

    // The caller and target come from the actual replayed physical Main report.
    let report = read
        .project()
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("reopened physical Main callable report")?;
    assert_eq!(report.calls().len(), 1);
    let call = &report.calls()[0];
    let SourceCallTarget::LibraryFunction { target } = call.target() else {
        return Err("native fixture lost its genuine Library closure target".into());
    };
    let caller = report
        .functions()
        .iter()
        .find(|function| function.fact_id() == call.caller_function_id())
        .ok_or("actual physical caller")?;
    assert_eq!(caller.path(), "main.lua");
    let resolver = rebuilt.target_resolver(read.project(), &stop)?;
    let super::super::NativeApiTargetResolution::Bound(association) =
        resolver.resolve_target(target, &stop)?
    else {
        return Err("reopened native target did not bind".into());
    };
    assert_eq!(association.target(), target);
    assert_eq!(association.binding().source(), &expected_source);
    assert_eq!(association.binding().source_id(), expected_source_id);
    assert_eq!(association.binding().mapping_id(), expected_mapping);
    assert_eq!(association.binding().key(), expected_key);
    assert_eq!(
        association.reference_record().digest()?,
        expected_record_digest
    );
    assert!(
        association
            .reference_record()
            .source_ids()
            .iter()
            .any(|source_id| source_id.as_ref() == expected_source_id)
    );

    let owner = read.graph();
    let api = owner
        .partition("wow-service.native-api-callables")
        .ok_or("accepted native API producer")?;
    assert_eq!(api.report().accepted_entities().len(), 1);
    assert!(api.report().accepted_relations().is_empty());
    let accepted_api = &api.report().accepted_entities()[0];
    let api_proposal = api
        .batch()
        .entity_proposal(accepted_api.proposal_id())
        .ok_or("accepted original API proposal")?;
    assert_eq!(api_proposal.entity_kind_id(), "native_api_callable");
    for (field, expected) in [
        ("reference_profile_id", expected_profile.as_str()),
        ("reference_generation_id", expected_generation.as_str()),
        ("reference_digest", expected_view_digest.as_str()),
        ("record_key", expected_key.as_str()),
        ("record_digest", expected_record_digest.as_str()),
    ] {
        let Some(
            GraphProposalValue::String(actual)
            | GraphProposalValue::Identifier(actual)
            | GraphProposalValue::Reference(actual),
        ) = api_proposal.semantic_key().get(field)
        else {
            return Err(format!("native API identity field {field} missing").into());
        };
        assert_eq!(actual.as_ref(), expected);
    }
    assert_eq!(api_proposal.source_handle_ids().len(), 1);
    assert_eq!(api_proposal.evidence_ids().len(), 1);
    let api_handles = read
        .native_api()
        .ok_or("genuine native API reopen proof")?
        .source_handles();
    let api_handle = api_handles
        .get(&api_proposal.source_handle_ids()[0])
        .ok_or("retained original API source support")?;
    assert_eq!(api_handle.path().as_str(), expected_source.path());
    assert_eq!(
        api_handle.content_digest().canonical(),
        expected_source.sha256()
    );
    assert_eq!(
        api_handle.span().byte_start(),
        Some(expected_source.span().start())
    );
    assert_eq!(
        api_handle.span().byte_end(),
        Some(expected_source.span().end())
    );
    assert_eq!(
        api_handle
            .reference_generation()
            .map(|id| id.to_string())
            .as_deref(),
        Some(expected_generation.as_str())
    );

    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("retained Main provenance")?;
    let function = provenance
        .functions()
        .iter()
        .find(|function| function.function_id == caller.fact_id())
        .ok_or("actual Main function proposal")?;
    assert_eq!(function.span, caller.span());
    let native_source = owner
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or("physical Main producer")?;
    let accepted_caller = native_source
        .report()
        .accepted_entities()
        .iter()
        .find(|entity| entity.proposal_id() == function.proposal_id)
        .ok_or("accepted physical Main function")?;
    let calls = owner
        .partition(wow_recognizers::source_calls::SOURCE_CALL_PARTITION)
        .ok_or("existing calls producer")?;
    let mut uses = calls
        .report()
        .accepted_relations()
        .iter()
        .filter(|relation| relation.edge().relation() == GraphRelationKind::UsesApi);
    let uses = (uses.next().ok_or("accepted native UsesApi")?, uses.next());
    assert!(
        uses.1.is_none(),
        "fixture must have exactly one accepted UsesApi"
    );
    assert_eq!(uses.0.edge().from(), accepted_caller.node().node_id());
    assert_eq!(uses.0.edge().to(), accepted_api.node().node_id());
    let use_proposal = calls
        .batch()
        .relation_proposal(uses.0.proposal_id())
        .ok_or("accepted native UsesApi proposal")?;
    assert_eq!(use_proposal.relation_kind_id(), "lua_uses_native_api");
    assert!(
        use_proposal
            .source_handle_ids()
            .contains(&function.source_handle_id)
    );
    assert!(
        use_proposal
            .source_handle_ids()
            .contains(&api_handle.handle_id())
    );

    let graph_read = read.pair().graph_read(&stop)?;
    let final_caller = graph_read.published_entity(
        wow_project::graph::SOURCE_GRAPH_PARTITION,
        &function.proposal_id,
        &stop,
    )?;
    let final_api =
        graph_read.published_entity(api.partition_id(), accepted_api.proposal_id(), &stop)?;
    let final_uses = owner
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| edge.relation() == GraphRelationKind::UsesApi)
        .collect::<Vec<_>>();
    assert_eq!(final_uses.len(), 1);
    assert_eq!(final_uses[0].from(), final_caller.node_id());
    assert_eq!(final_uses[0].to(), final_api.node_id());
    // Context host and expectations come from this source-absent native proof.
    // Serialized values below are assertions only, never acquired authority.
    fn rows(value: &Value) -> TestResult<&[Value]> {
        value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| "expected context array".into())
    }
    const CONTEXT_STORE_ID: &str = "fixture-selected-native-api-context";
    let reference = &rebuilt.reference().view;
    let profile = rebuilt.profile();
    let native_api = read.native_api().ok_or("held native API proof")?;
    // The old context-only constructor must still reject this independent
    // producer. Only exact accepted-partition support may supplement it.
    assert!(
        wow_graph::GraphEvidenceCatalog::new(
            native_api.context().clone(),
            native_api.evidence().clone(),
            native_api.source_handles().clone(),
            &stop,
        )
        .is_err()
    );
    for producer in [
        wow_core::ProducerVersionEntry::new(
            "wow.service.unrelated".parse()?,
            wow_core::ToolVersion::parse(env!("CARGO_PKG_VERSION"))?,
        ),
        wow_core::ProducerVersionEntry::new(
            "wow.service.native_api".parse()?,
            wow_core::ToolVersion::parse("0.0.0")?,
        ),
    ] {
        let refusal = wow_graph::GraphPartitionSourceObservations::new(
            &graph_read,
            api.partition_id(),
            producer,
            native_api.evidence(),
            native_api.source_handles(),
            &stop,
        )
        .err()
        .ok_or("foreign source observation producer/version was admitted")?;
        assert_eq!(refusal.code(), wow_graph::GraphErrorCode::EvidenceInvalid);
    }
    let expected_caller = serde_json::to_value(final_caller)?;
    let expected_supports = [
        (
            "entity",
            serde_json::to_value(final_api)?,
            api.partition_id().to_owned(),
            serde_json::to_value(api_proposal)?,
            serde_json::to_value(accepted_api)?,
        ),
        (
            "relation",
            serde_json::to_value(final_uses[0])?,
            calls.partition_id().to_owned(),
            serde_json::to_value(use_proposal)?,
            serde_json::to_value(uses.0)?,
        ),
    ];
    let expected_api_handle = serde_json::to_value(api_handle)?;
    let expected_api_evidence = serde_json::to_value(
        native_api
            .evidence()
            .get(&api_proposal.evidence_ids()[0])
            .ok_or("original native API evidence")?,
    )?;
    // Capture owner records before closing this genuine source-absent read.
    let wow_reference::LookupResult::Found(original_record) = reference.lookup(
        wow_reference::native_view::NATIVE_API_PARTITION,
        &expected_key,
    ) else {
        return Err("held native Reference record is not Found".into());
    };
    assert_eq!(original_record.digest()?, expected_record_digest);
    let expected_reference_record = serde_json::to_value(original_record)?;
    let signature = rebuilt
        .reference()
        .callable_signature(original_record, &stop)?;
    assert_eq!(signature.reference_generation(), expected_generation);
    assert_eq!(signature.reference_view_digest(), expected_view_digest);
    assert_eq!(signature.record_key(), expected_key);
    assert_eq!(signature.record_digest(), expected_record_digest);
    assert_eq!(signature.source_id(), expected_source_id);
    let expected_reference_signature = assert_transport_signature(signature)?;
    let reference_partition = reference
        .partitions()
        .iter()
        .find(|partition| partition.id() == wow_reference::native_view::NATIVE_API_PARTITION)
        .ok_or("original native API Reference partition")?;
    assert_eq!(
        reference_partition.coverage(),
        wow_reference::CoverageStatus::Partial
    );
    let expected_reference_coverage = serde_json::to_value(reference_partition.coverage())?;
    let api_targets = native_api.api_bindings(&stop)?;
    let original_api_origin = api_targets
        .get(target)
        .ok_or("actual native target-to-accepted-endpoint association")?;
    assert_eq!(original_api_origin.source_handle, api_handle);
    assert_eq!(
        serde_json::to_value(original_api_origin.evidence)?,
        expected_api_evidence
    );
    let expected_api_assertion = serde_json::to_value(&original_api_origin.assertion)?;
    drop(api_targets);
    // Match the actual selected aggregate preflight. Both borrowed components
    // fit in isolation; accepting their combined frame would exceed this limit.
    let project_inputs = (
        reference,
        owner.snapshot(),
        provenance.context(),
        provenance.evidence(),
        provenance.source_handles(),
    );
    let api_inputs = (
        native_api.context(),
        native_api.evidence(),
        native_api.source_handles(),
    );
    let isolated_limit = wow_core::canonical_json_bytes(&project_inputs)?
        .len()
        .max(wow_core::canonical_json_bytes(&api_inputs)?.len());
    let aggregate_bytes = wow_core::canonical_json_bytes(&(
        &project_inputs,
        native_api.context(),
        native_api.evidence(),
        native_api.source_handles(),
    ))?
    .len();
    assert!(aggregate_bytes > isolated_limit);
    let host_json = json!({
        "schema": CONTEXT_MAP_HOST_SCHEMA,
        "project_store_id": CONTEXT_STORE_ID,
        "project_store_root": store_path,
        "project_id": read.project().configuration().project_id(),
        "reference_profile": profile,
        "reference": reference,
        "limits": {
            "neighbors": GraphNeighborReadLimits::default(),
            "explain": GraphExplainLimits {
                max_output_bytes: 64 * 1024,
                ..GraphExplainLimits::default()
            },
            "evidence": GraphEvidenceResolveLimits::default(),
            "max_direct_edges": 16,
            "max_input_bytes": 4 * 1024 * 1024,
            "max_metadata_bytes": 4 * 1024 * 1024,
            "max_context_work": 8_000_000
        }
    });
    let host = ContextMapHostInput::from_json_slice(&serde_json::to_vec(&host_json)?, &stop)?;
    let publication = ContextMapPublication::store_generation(read.store_generation_id().as_str())?;
    let context_root = ContextMapRoot::graph_entity(final_caller.node_id().as_str())?;
    let requests = |view: &wow_reference::ReferenceView| -> TestResult<(ContextInspectRequest, ContextBuildRequest)> {
        let guard = ContextMapReferenceGuard::new(
            profile.profile_id().as_str(),
            view.generation_id(),
            view.self_digest(),
        )?;
        Ok((
            ContextInspectRequest::primary(
                CONTEXT_STORE_ID,
                publication.clone(),
                ContextInspectLevel::L1,
                context_root.clone(),
                None,
                Some(guard.clone()),
            )?,
            ContextBuildRequest::primary(
                CONTEXT_STORE_ID,
                publication.clone(),
                ContextInspectLevel::L1,
                context_root.clone(),
                ContextRenderSelection::Both,
                None,
                Some(guard),
            )?,
        ))
    };
    let (inspect_request, build_request) = requests(reference)?;
    let wrong_reference =
        wow_reference::ReferenceView::new(reference.generation_id(), Vec::new(), Vec::new())?;
    wrong_reference.validate()?;
    assert_eq!(wrong_reference.generation_id(), reference.generation_id());
    assert_ne!(wrong_reference.self_digest(), reference.self_digest());
    // The wrong view remains valid and its guard agrees with its own host.
    // Only actual native Reference content custody can explain this refusal.
    let (wrong_inspect_request, wrong_build_request) = requests(&wrong_reference)?;
    let mut wrong_host_json = host_json.clone();
    wrong_host_json["reference"] = serde_json::to_value(&wrong_reference)?;
    let wrong_host =
        ContextMapHostInput::from_json_slice(&serde_json::to_vec(&wrong_host_json)?, &stop)?;
    let mut small_host_json = host_json.clone();
    small_host_json["limits"]["max_input_bytes"] = json!(isolated_limit);
    let small_host =
        ContextMapHostInput::from_json_slice(&serde_json::to_vec(&small_host_json)?, &stop)?;
    drop(resolver);
    drop(rebuilt);
    read.close()?;
    store.close()?;

    // Close the witness acquisition first. Each actual service operation opens
    // the selected publication independently, with the source still absent.
    assert!(!root.0.exists());
    let assert_native_l1 = |skeletons: &Value| -> TestResult {
        assert_eq!(skeletons["level"], "l1");
        assert!(skeletons.get("l0").is_none());
        let core = &skeletons["l1"]["core"];
        assert_eq!(core["schema"], "wow-context/l1-skeleton/primary/1");
        assert_eq!(core["root"], expected_caller);
        assert!(rows(&core["local_entities"])?.contains(&expected_supports[0].1));
        assert!(rows(&core["direct_relations"])?.contains(&expected_supports[1].1));
        assert!(rows(&core["not_evaluated"])?.contains(&json!("reference_fact_enrichment")));
        let origins = &core["context"]["origins"]["core"];
        assert!(
            rows(&origins["source_custody"])?
                .iter()
                .any(|custody| custody["handle"] == expected_api_handle)
        );
        for (kind, record, partition, proposal, accepted) in &expected_supports {
            let field = if *kind == "entity" { "node" } else { "edge" };
            let receipts = rows(&origins["supports"])?
                .iter()
                .filter(|receipt| {
                    let explained = &receipt["explanation"]["record"];
                    explained["kind"] == *kind && &explained[field] == record
                })
                .collect::<Vec<_>>();
            assert_eq!(receipts.len(), 1, "exact API/UsesApi native record support");
            let receipt = receipts[0];
            let explanation = &receipt["explanation"];
            assert_eq!(explanation["support_complete"], true);
            assert!(rows(&explanation["truncations"])?.is_empty());
            let origin = if *kind == "entity" {
                "producer_entity"
            } else {
                "producer_relation"
            };
            assert!(rows(&explanation["supports"])?.iter().any(|support| {
                support["origin"] == origin
                    && support["producer"]["partition_id"] == partition.as_str()
                    && &support["proposal"] == proposal
                    && &support["accepted"] == accepted
            }));
            let resolution = &receipt["evidence_resolution"];
            assert_eq!(resolution["closure_complete"], true);
            assert!(rows(&resolution["unresolved"])?.is_empty());
            assert!(rows(&resolution["truncations"])?.is_empty());
            assert!(rows(&resolution["source_handles"])?.contains(&expected_api_handle));
            assert!(rows(&resolution["evidence_records"])?.contains(&expected_api_evidence));
        }
        Ok(())
    };
    let inspected = execute_context_inspect(&host, &inspect_request, &stop)?;
    assert_eq!(inspected.status(), ContextMapStatus::Partial);
    assert_eq!(inspected.failure_code(), None);
    let inspected_json: Value = serde_json::from_slice(inspected.canonical_bytes())?;
    assert_eq!(
        inspected_json["schema"],
        "wow-service/context-inspect-result/1"
    );
    assert_eq!(inspected_json["operation"], "context_inspect");
    assert_eq!(inspected_json["resource_closure"], "closed");
    assert_native_l1(&inspected_json["payload"]["skeletons"])?;

    let built = execute_context_build(&host, &build_request, &stop)?;
    assert_eq!(built.status(), ContextMapStatus::Partial);
    assert_eq!(built.failure_code(), None);
    let built_json: Value = serde_json::from_slice(built.canonical_bytes())?;
    assert_eq!(built_json["schema"], "wow-service/context-build-result/1");
    assert_eq!(built_json["operation"], "context_build");
    assert_eq!(built_json["resource_closure"], "closed");
    let payload = &built_json["payload"];
    let pack = &payload["semantic_pack"];
    assert_eq!(
        pack["core"]["schema"],
        "wow-context/semantic-pack/primary/1"
    );
    assert_eq!(
        pack["core"]["profile"]["profile_id"],
        PRIMARY_CONTEXT_PACK_PROFILE_ID
    );
    assert_native_l1(&pack["core"]["skeletons"])?;
    let items = rows(&pack["core"]["items"])?;
    for (kind, record, _, _, _) in &expected_supports {
        let field = if *kind == "entity" { "node" } else { "edge" };
        assert_eq!(
            items
                .iter()
                .filter(|item| {
                    let item_payload = &item["core"]["payload"];
                    item_payload["kind"] == *kind && &item_payload[field] == record
                })
                .count(),
            1,
            "accepted API endpoint and UsesApi edge each enter the structural pack"
        );
    }
    let artifacts = rows(&payload["rendered_artifacts"])?;
    assert_eq!(artifacts.len(), 2);
    for (artifact, profile_id) in artifacts.iter().zip([
        PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    ]) {
        let core = &artifact["core"];
        assert_eq!(core["profile"]["profile_id"], profile_id);
        assert_eq!(core["semantic_pack_id"], pack["semantic_pack_id"]);
        let content = core["content"].as_str().ok_or("rendered context content")?;
        assert_eq!(core["byte_length"], content.len());
        assert_eq!(
            core["content_digest"],
            ContentDigest::<CanonicalResult>::from_bytes(Sha256::digest(content.as_bytes()).into())
                .to_string()
        );
    }
    let json_content = artifacts[0]["core"]["content"]
        .as_str()
        .ok_or("JSON artifact")?;
    assert_eq!(
        json_content.as_bytes(),
        wow_core::canonical_json_bytes(pack)?
    );
    let markdown = &artifacts[1]["core"];
    let skeleton_records = rows(&markdown["trace"])?
        .iter()
        .filter(|record| record["template_id"] == "native_skeletons")
        .collect::<Vec<_>>();
    assert_eq!(skeleton_records.len(), 1);
    let record = skeleton_records[0];
    let start = usize::try_from(record["byte_start"].as_u64().ok_or("trace start")?)?;
    let end = usize::try_from(record["byte_end"].as_u64().ok_or("trace end")?)?;
    let line = markdown["content"]
        .as_str()
        .and_then(|content| content.get(start..end))
        .ok_or("UTF-8 skeleton trace")?;
    let quoted = line
        .strip_prefix("SKELETONS ")
        .and_then(|value| value.strip_suffix('\n'))
        .ok_or("fixed Markdown skeleton framing")?;
    let inner: String = serde_json::from_str(quoted)?;
    let rendered_skeletons: Value = serde_json::from_str(&inner)?;
    assert_eq!(rendered_skeletons, pack["core"]["skeletons"]);

    // The explicit facet shares this publication and the original native support.
    let reference_profile = ContextL1Profile::ReferenceFactsV1;
    let reference_inspect_request = inspect_request.clone().with_l1_profile(reference_profile)?;
    let reference_build_request = build_request.clone().with_l1_profile(reference_profile)?;
    const REFERENCE_L1: &str = "wow-context/primary-reference-facts-l1/1";
    for request in [
        serde_json::to_value(&reference_inspect_request)?,
        serde_json::to_value(&reference_build_request)?,
    ] {
        assert_eq!(request["l1_profile_id"], REFERENCE_L1);
    }
    let assert_reference_l1 = |skeletons: &Value| -> TestResult {
        assert_eq!(skeletons["level"], "l1");
        assert!(skeletons.get("l0").is_none());
        let core = &skeletons["l1"]["core"];
        let default_core = &inspected_json["payload"]["skeletons"]["l1"]["core"];
        assert_eq!(
            core["schema"],
            "wow-context/l1-skeleton/primary-reference-facts/1"
        );
        assert_eq!(core["profile"], REFERENCE_L1);
        assert_eq!(core["root"], expected_caller);
        for field in ["local_entities", "direct_relations", "declarations"] {
            assert_eq!(
                core[field], default_core[field],
                "original {field} preserved"
            );
        }
        assert_eq!(
            core["context"]["origins"],
            default_core["context"]["origins"]
        );
        assert_eq!(core["context"]["absence_authoritative"], false);
        assert!(
            rows(&core["not_evaluated"])?.contains(&json!("complete_corpus_or_runtime_behavior"))
        );
        assert!(
            rows(&core["not_evaluated"])?
                .contains(&json!("reference_type_resolution_and_annotation_lowering"))
        );
        let facets = rows(&core["reference_records"])?;
        assert_eq!(
            facets.len(),
            1,
            "one exact selected native API Reference facet"
        );
        let facet = &facets[0];
        assert_eq!(facet["node"], expected_supports[0].1);
        assert_eq!(facet["assertion"], expected_api_assertion);
        assert_eq!(
            facet["partition_id"],
            wow_reference::native_view::NATIVE_API_PARTITION
        );
        assert_eq!(facet["partition_coverage"], expected_reference_coverage);
        assert_eq!(facet["record"], expected_reference_record);
        assert_eq!(facet["record"]["key"], expected_key);
        assert_eq!(facet["record_digest"], expected_record_digest);
        assert_eq!(facet["signature"], expected_reference_signature);
        // Payload remains an opaque string; its bytes, source IDs and restrictions
        // are compared as original data, never parsed into context semantics.
        assert_eq!(
            facet["record"]["payload"],
            expected_reference_record["payload"]
        );
        assert_eq!(
            facet["record"]["source_ids"],
            expected_reference_record["source_ids"]
        );
        assert_eq!(
            facet["record"]["restrictions"],
            expected_reference_record["restrictions"]
        );
        assert_eq!(facet["source"], expected_api_handle);
        assert_eq!(facet["evidence"], expected_api_evidence);
        assert_eq!(
            facet["source_association"],
            "exact_record_source_id_and_accepted_original_support"
        );
        assert_eq!(
            facet["payload_interpretation"],
            "original_canonical_reference_payload_no_context_parser"
        );
        assert_eq!(facet["absence_authoritative"], false);
        let mut facet_fields = facet
            .as_object()
            .ok_or("typed Reference facet")?
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        facet_fields.sort_unstable();
        assert_eq!(
            facet_fields,
            [
                "absence_authoritative",
                "assertion",
                "evidence",
                "node",
                "partition_coverage",
                "partition_id",
                "payload_interpretation",
                "record",
                "record_digest",
                "signature",
                "source",
                "source_association",
            ]
        );
        Ok(())
    };
    // Omitted selection keeps the existing /1 schema and omits the new facet.
    assert_native_l1(&inspected_json["payload"]["skeletons"])?;
    assert!(
        inspected_json["payload"]["skeletons"]["l1"]["core"]
            .get("reference_records")
            .is_none()
    );
    assert!(
        pack["core"]["skeletons"]["l1"]["core"]
            .get("reference_records")
            .is_none()
    );
    assert!(
        pack["core"]["request"]["core"]
            .get("l1_profile_id")
            .is_none()
    );
    let reference_inspected = execute_context_inspect(&host, &reference_inspect_request, &stop)?;
    assert_eq!(reference_inspected.status(), ContextMapStatus::Partial);
    assert_eq!(reference_inspected.failure_code(), None);
    let reference_inspected_json: Value =
        serde_json::from_slice(reference_inspected.canonical_bytes())?;
    assert_eq!(reference_inspected_json["resource_closure"], "closed");
    assert_reference_l1(&reference_inspected_json["payload"]["skeletons"])?;

    let reference_built = execute_context_build(&host, &reference_build_request, &stop)?;
    assert_eq!(reference_built.status(), ContextMapStatus::Partial);
    assert_eq!(reference_built.failure_code(), None);
    let reference_built_json: Value = serde_json::from_slice(reference_built.canonical_bytes())?;
    assert_eq!(reference_built_json["resource_closure"], "closed");
    let reference_payload = &reference_built_json["payload"];
    let reference_pack = &reference_payload["semantic_pack"];
    assert_eq!(reference_pack["core"]["schema"], pack["core"]["schema"]);
    assert_eq!(reference_pack["core"]["profile"], pack["core"]["profile"]);
    assert_eq!(
        reference_pack["core"]["request"]["core"]["l1_profile_id"],
        REFERENCE_L1
    );
    assert_reference_l1(&reference_pack["core"]["skeletons"])?;
    let omissions = rows(&reference_pack["core"]["omissions"]["not_evaluated"])?;
    assert!(!omissions.contains(&json!("reference_fact_enrichment")));
    assert!(omissions.contains(&json!("reference_type_resolution_and_annotation_lowering")));
    assert!(
        rows(&pack["core"]["omissions"]["not_evaluated"])?
            .contains(&json!("reference_fact_enrichment"))
    );
    assert_eq!(
        reference_pack["core"]["skeletons"],
        reference_inspected_json["payload"]["skeletons"]
    );
    assert_eq!(reference_pack["core"]["items"], pack["core"]["items"]);
    assert_eq!(
        reference_pack["core"]["selection_trace"],
        pack["core"]["selection_trace"]
    );
    for resource in ["work_reserved", "bytes_reserved"] {
        assert!(
            reference_payload["operation_budget_usage"][resource]
                .as_u64()
                .ok_or("selected Reference operation usage")?
                > payload["operation_budget_usage"][resource]
                    .as_u64()
                    .ok_or("default operation usage")?,
            "Reference facet construction and Both rendering continue {resource}"
        );
    }
    let reference_artifacts = rows(&reference_payload["rendered_artifacts"])?;
    assert_eq!(reference_artifacts.len(), 2);
    let reference_json_content = reference_artifacts[0]["core"]["content"]
        .as_str()
        .ok_or("Reference JSON artifact")?;
    assert_eq!(
        reference_json_content.as_bytes(),
        wow_core::canonical_json_bytes(reference_pack)?
    );
    let reference_markdown = &reference_artifacts[1]["core"];
    let records = rows(&reference_markdown["trace"])?
        .iter()
        .filter(|record| record["template_id"] == "native_skeletons")
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let start = usize::try_from(
        records[0]["byte_start"]
            .as_u64()
            .ok_or("Reference trace start")?,
    )?;
    let end = usize::try_from(
        records[0]["byte_end"]
            .as_u64()
            .ok_or("Reference trace end")?,
    )?;
    let line = reference_markdown["content"]
        .as_str()
        .and_then(|content| content.get(start..end))
        .ok_or("Reference UTF-8 skeleton trace")?;
    let quoted = line
        .strip_prefix("SKELETONS ")
        .and_then(|value| value.strip_suffix('\n'))
        .ok_or("Reference fixed Markdown skeleton framing")?;
    let inner: String = serde_json::from_str(quoted)?;
    let rendered_reference_skeletons: Value = serde_json::from_str(&inner)?;
    assert_eq!(
        rendered_reference_skeletons,
        reference_pack["core"]["skeletons"]
    );

    let assert_refused = |result: &ContextMapResult,
                          operation: &str,
                          schema: &str,
                          code: ServiceErrorCode,
                          closure: &str|
     -> TestResult {
        assert_eq!(result.failure_code(), Some(code));
        assert_eq!(
            result.status(),
            if code == ServiceErrorCode::Cancelled {
                ContextMapStatus::Cancelled
            } else {
                ContextMapStatus::Failed
            }
        );
        let value: Value = serde_json::from_slice(result.canonical_bytes())?;
        assert_eq!(value["schema"], schema);
        assert_eq!(value["operation"], operation);
        assert_eq!(value["resource_closure"], closure);
        assert!(value.get("payload").is_none());
        Ok(())
    };
    for (operation, schema, result) in [
        (
            "context_inspect",
            "wow-service/context-inspect-result/1",
            execute_context_inspect(&wrong_host, &wrong_inspect_request, &stop)?,
        ),
        (
            "context_build",
            "wow-service/context-build-result/1",
            execute_context_build(&wrong_host, &wrong_build_request, &stop)?,
        ),
        (
            "context_inspect",
            "wow-service/context-inspect-result/1",
            execute_context_inspect(
                &wrong_host,
                &wrong_inspect_request
                    .clone()
                    .with_l1_profile(reference_profile)?,
                &stop,
            )?,
        ),
        (
            "context_build",
            "wow-service/context-build-result/1",
            execute_context_build(
                &wrong_host,
                &wrong_build_request
                    .clone()
                    .with_l1_profile(reference_profile)?,
                &stop,
            )?,
        ),
    ] {
        assert_refused(
            &result,
            operation,
            schema,
            ServiceErrorCode::IdentityMismatch,
            "closed",
        )?;
    }
    assert_refused(
        &execute_context_build(&small_host, &build_request, &stop)?,
        "context_build",
        "wow-service/context-build-result/1",
        ServiceErrorCode::BudgetExceeded,
        "closed",
    )?;
    assert_refused(
        &execute_context_build(&host, &build_request, &AtomicBool::new(true))?,
        "context_build",
        "wow-service/context-build-result/1",
        ServiceErrorCode::Cancelled,
        "not_acquired",
    )?;
    assert_refused(
        &execute_context_build(&small_host, &reference_build_request, &stop)?,
        "context_build",
        "wow-service/context-build-result/1",
        ServiceErrorCode::BudgetExceeded,
        "closed",
    )?;
    let mut delivery_host_json = host_json;
    delivery_host_json["limits"]["max_metadata_bytes"] =
        reference_payload["operation_budget_usage"]["bytes_reserved"].clone();
    let delivery_host =
        ContextMapHostInput::from_json_slice(&serde_json::to_vec(&delivery_host_json)?, &stop)?;
    assert_refused(
        &execute_context_build(&delivery_host, &reference_build_request, &stop)?,
        "context_build",
        "wow-service/context-build-result/1",
        ServiceErrorCode::BudgetExceeded,
        "closed",
    )?;
    assert_refused(
        &execute_context_build(&host, &reference_build_request, &AtomicBool::new(true))?,
        "context_build",
        "wow-service/context-build-result/1",
        ServiceErrorCode::Cancelled,
        "not_acquired",
    )?;
    assert!(!root.0.exists());
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
