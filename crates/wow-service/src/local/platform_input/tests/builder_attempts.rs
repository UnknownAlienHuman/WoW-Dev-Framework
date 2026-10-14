//! Reuse the native fixture to exercise the compiled Builder's real filesystem
//! and durable journal. These synthetic sources do not qualify current WoW APIs.
use super::*;
use crate::reference_pack::{
    LOCAL_NATIVE_CANONICAL_EXECUTION_PROFILE, LOCAL_NATIVE_REVERSED_EXECUTION_PROFILE,
    ReferencePackBudgets, ReferencePackBuildRequest, ReferencePackEligibilityTarget,
};
use std::{path::Path, process::Command};

#[test]
fn build_execution_selector_admits_only_its_two_supported_profiles() -> TestResult {
    for profile in [
        LOCAL_NATIVE_CANONICAL_EXECUTION_PROFILE,
        LOCAL_NATIVE_REVERSED_EXECUTION_PROFILE,
    ] {
        ReferencePackBuildRequest::new(
            "fixture-attempt",
            "profile:fixture:builder",
            "fixture-generation",
            ReferencePackEligibilityTarget::Candidate,
            profile,
            ReferencePackBudgets::default(),
        )?;
    }
    for (id, profile) in [
        ("fixture-attempt", "unsupported/execution/1"),
        (
            "invalid/request/id",
            LOCAL_NATIVE_CANONICAL_EXECUTION_PROFILE,
        ),
    ] {
        assert!(
            ReferencePackBuildRequest::new(
                id,
                "profile:fixture:builder",
                "fixture-generation",
                ReferencePackEligibilityTarget::Candidate,
                profile,
                ReferencePackBudgets::default(),
            )
            .is_err()
        );
    }
    Ok(())
}

fn source(member: &str) -> TestResult<FixtureRoot> {
    let (root, config) = native_fixture()?;
    let api_path = "Blizzard_APIDocumentationGenerated/API.lua";
    let toc_path = "Blizzard_APIDocumentationGenerated/Generated.toc";
    let api = std::fs::read_to_string(root.0.join(api_path))?
        .replace("Name=\"Read\"", &format!("Name=\"{member}\""));
    let toc = "## Interface: 1\nAPI.lua\n";
    std::fs::write(root.0.join(api_path), &api)?;
    std::fs::write(root.0.join(toc_path), toc)?;
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    // The original fixture intentionally contains one bad API input. This
    // positive candidate selects only the genuine supported API documentation.
    files.retain(|file| file["path"] != "Blizzard_APIDocumentationGenerated/Bad.lua");
    for (path, bytes) in [(api_path, api.as_bytes()), (toc_path, toc.as_bytes())] {
        let file = files
            .iter_mut()
            .find(|file| file["path"] == path)
            .ok_or("manifest member")?;
        file["bytes"] = json!(bytes.len());
        file["content_sha256"] = json!(format!("{:x}", Sha256::digest(bytes)));
    }
    let count = files.len();
    let bytes: u64 = files
        .iter()
        .map(|file| file["bytes"].as_u64().unwrap_or(0))
        .sum();
    manifest["coverage"] = json!({"tracked_files":count,"included_files":count,
        "excluded_files":0,"included_bytes":bytes,"kind_version":1,"kind_lua":1,
        "kind_toc":2,"kind_xml":1,"kind_generated_api":1});
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let manifest_file = write_json(&root, "manifest.json", &manifest)?;
    let mut native_source = config["native_reference"]["native_source"].clone();
    native_source["manifest"] = serde_json::to_value(manifest_file)?;
    let native = json!({
        "schema":crate::local::native_input::LOCAL_NATIVE_SCHEMA,
        "project_id":config["project_id"],"workspace_id":config["workspace_id"],
        "source_origin_id":config["source_origin_id"],"logical_root":config["logical_root"],
        "profile":config["native_reference"]["profile"],"analyzer":config["analyzer"],
        "main":{"root":"UI","files":[{"path":"main.lua"}]},
        "native_source":native_source
    });
    write_json(&root, "builder-native.json", &native)?;
    Ok(root)
}

fn request(root: &FixtureRoot, id: &str) -> TestResult<PathBuf> {
    let input = crate::LocalProjectInput::from_config_path(
        &root.0.join("builder-native.json"),
        &AtomicBool::new(false),
    )?;
    let receipt = input.native_input_receipt().ok_or("native receipt")?;
    let request = ReferencePackBuildRequest::new(
        id,
        receipt.profile.profile_id().as_str(),
        input.reference_view().generation_id(),
        ReferencePackEligibilityTarget::Candidate,
        LOCAL_NATIVE_CANONICAL_EXECUTION_PROFILE,
        ReferencePackBudgets::default(),
    )?;
    let file = format!("{id}.json");
    write_json(
        root,
        &file,
        &json!({
            "schema":"wow-reference-builder/build-request/1",
            "source_config":"builder-native.json","request":request
        }),
    )?;
    Ok(root.0.join(file))
}

fn run(
    binary: &Path,
    root: &FixtureRoot,
    id: &str,
    output: &Path,
) -> TestResult<(i32, Value, String)> {
    let request = request(root, id)?;
    let result = Command::new(binary)
        .arg("build")
        .arg("--request")
        .arg(request)
        .arg("--source-root")
        .arg(&root.0)
        .arg("--output")
        .arg(output)
        .arg("--json")
        .output()?;
    let payload = if result.stdout.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&result.stdout)?
    };
    Ok((
        result.status.code().ok_or("builder termination")?,
        payload,
        String::from_utf8(result.stderr)?,
    ))
}

#[test]
#[ignore = "requires WOW_BUILDER_LIFECYCLE_BIN pointing to the exact compiled candidate"]
fn explicit_attempts_reach_native_a_b_a_install_and_conflicting_retry() -> TestResult {
    let binary = PathBuf::from(
        std::env::var_os("WOW_BUILDER_LIFECYCLE_BIN").ok_or("compiled builder path required")?,
    );
    assert!(binary.is_absolute() && binary.is_file());
    let a = source("Read")?;
    let b = source("Write")?;
    let output_root = FixtureRoot(std::env::temp_dir().join(format!(
        "wow-builder-attempts-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    )));
    std::fs::create_dir(&output_root.0)?;
    let output = output_root.0.join("pack");
    let (code, first, error) = run(&binary, &a, "attempt-a-first", &output)?;
    assert_eq!(code, 0, "first native candidate: {error}; {first}");
    assert_eq!(first["destination_state"], "created");
    assert_eq!(first["candidate_eligible"], true);
    let (code, retry, error) = run(&binary, &a, "attempt-a-first", &output)?;
    assert_eq!(code, 0, "same-request retry: {error}");
    assert_eq!(retry["pack_id"], first["pack_id"]);
    assert_eq!(retry["plan_id"], first["plan_id"]);
    assert_eq!(
        retry["materialization_operation_id"],
        first["materialization_operation_id"]
    );
    assert_eq!(retry["materialization_replayed"], true);
    let (code, second, error) = run(&binary, &b, "attempt-b", &output)?;
    assert_eq!(code, 0, "native B replacement: {error}");
    assert_eq!(second["destination_state"], "replaced");
    assert_ne!(second["pack_id"], first["pack_id"]);
    let before = std::fs::read(output.join("manifest.json"))?;
    let (code, payload, error) = run(&binary, &b, "attempt-a-first", &output)?;
    assert_eq!(code, 5, "conflicting attempt: {error}; {payload}");
    assert!(error.contains("materialization_operation_conflict"));
    assert_eq!(std::fs::read(output.join("manifest.json"))?, before);
    let (code, final_a, error) = run(&binary, &a, "attempt-a-new", &output)?;
    assert_eq!(code, 0, "native A after B: {error}; {final_a}");
    assert_eq!(final_a["destination_state"], "replaced");
    assert_eq!(final_a["pack_id"], first["pack_id"]);
    assert_eq!(final_a["plan_id"], first["plan_id"]);
    assert_ne!(
        final_a["materialization_operation_id"],
        first["materialization_operation_id"]
    );
    assert_eq!(final_a["materialization_replayed"], false);
    let (code, retry_a, error) = run(&binary, &a, "attempt-a-new", &output)?;
    assert_eq!(code, 0, "new-A retry: {error}");
    assert_eq!(
        retry_a["materialization_operation_id"],
        final_a["materialization_operation_id"]
    );
    assert_eq!(retry_a["materialization_replayed"], true);
    eprintln!(
        "native Builder A/retry/B/conflict/new-A/retry passed; semantic pack and plan stable"
    );
    Ok(())
}
