use super::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, path::PathBuf};
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
    let path = std::env::temp_dir().join(format!("wow-platform-transport-{}", std::process::id()));
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
