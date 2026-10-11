use super::*;
mod empty_packages;
mod project_budgets;
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
