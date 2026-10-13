//! One same-input common-path diagnostic on the previously retained witness.
use super::*;

const CAPTURE: &str = ".codex/work/context-common-path-129-20261012";

#[test]
#[ignore = "one explicit before-repair measurement on the existing native fixture"]
fn common_path_before() -> TestResult {
    observe("before")
}

#[test]
#[ignore = "one explicit after-repair measurement on the same exact native fixture"]
fn common_path_after() -> TestResult {
    observe("after")
}

#[test]
#[ignore = "bounded identity, budget and cancellation refusals on the retained input"]
fn common_path_refusals() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let original = repo.join(".codex/work/platform-xml-declaration-consumer-20261012");
    let capture = repo.join(CAPTURE).join("refusals");
    std::fs::create_dir_all(&capture)?;
    let host_bytes = bounded_bytes(&original.join("host.json"), MAX_HOST_BYTES)?;
    let host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let request: Value = serde_json::from_slice(&bounded_bytes(
        &original.join("build-request.json"),
        MAX_REQUEST_BYTES,
    )?)?;
    let stop = AtomicBool::new(false);
    let host = crate::context::ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let mut mismatch = request.clone();
    let publication = request["primary"]["publication_set_id"]
        .as_str()
        .ok_or("publication")?;
    let mut digest = publication
        .strip_prefix("project-publication-set:sha256:")
        .ok_or("publication digest")?
        .to_owned();
    let replacement = if digest.starts_with('0') { "1" } else { "0" };
    digest.replace_range(..1, replacement);
    mismatch["primary"]["publication_set_id"] =
        json!(format!("project-publication-set:sha256:{digest}"));
    assert_ne!(
        mismatch["primary"]["publication_set_id"],
        request["primary"]["publication_set_id"]
    );
    let mismatch = run_combined_service(&host, &mismatch, &stop)?;
    require_closed(&mismatch.envelope)?;
    require_no_payload(&mismatch.envelope)?;
    assert_eq!(
        mismatch.failure_code,
        Some(ServiceErrorCode::IdentityMismatch)
    );

    let mut limited = host_wire;
    for owner in ["primary", "platform"] {
        limited[owner]["limits"]["max_context_work"] = json!(1);
    }
    let limited = crate::context::ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&limited)?,
        &stop,
    )?;
    let budget = run_combined_service(&limited, &request, &stop)?;
    require_closed(&budget.envelope)?;
    require_no_payload(&budget.envelope)?;
    assert_eq!(budget.failure_code, Some(ServiceErrorCode::BudgetExceeded));

    let admitted_request = crate::context::ContextCombinedRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&request)?,
        &stop,
    )?;
    let cancelled =
        crate::context::execute_context_combined(&host, &admitted_request, &AtomicBool::new(true))?;
    let cancelled_envelope: Value = serde_json::from_slice(cancelled.canonical_bytes())?;
    require_no_payload(&cancelled_envelope)?;
    assert_eq!(cancelled.failure_code(), Some(ServiceErrorCode::Cancelled));
    assert_eq!(cancelled_envelope["resource_closure"], "not_acquired");
    write_new_capped(
        &capture.join("cancelled.json"),
        cancelled.canonical_bytes(),
        MAX_EXPORT_BYTES,
    )?;
    for (name, observation) in [("identity", mismatch), ("budget", budget)] {
        write_new_capped(
            &capture.join(format!("{name}.json")),
            &observation.bytes,
            MAX_EXPORT_BYTES,
        )?;
    }
    Ok(())
}

fn observe(stage: &str) -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let original = repo.join(".codex/work/platform-xml-declaration-consumer-20261012");
    let capture = repo.join(CAPTURE).join(stage);
    std::fs::create_dir_all(&capture)?;
    let host_bytes = bounded_bytes(&original.join("host.json"), MAX_HOST_BYTES)?;
    let request_bytes = bounded_bytes(&original.join("build-request.json"), MAX_REQUEST_BYTES)?;
    let stop = AtomicBool::new(false);
    let host = crate::context::ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let request = serde_json::from_slice(&request_bytes)?;
    let observation = run_combined_service(&host, &request, &stop)?;
    require_closed(&observation.envelope)?;
    assert_eq!(observation.failure_code, None);
    let expected = bounded_bytes(
        &original.join("build-service.json"),
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    assert_eq!(
        observation.bytes, expected,
        "same original canonical envelope"
    );
    write_new_capped(
        &capture.join("service.json"),
        &observation.bytes,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    let proof = json!({
        "stage":stage,"fixture":"retained XML declaration 55-node native input",
        "host_sha256":digest_hex(&host_bytes),"request_sha256":digest_hex(&request_bytes),
        "service_ms":observation.service_ms,"decode_ms":observation.decode_ms,
        "result_bytes":observation.bytes.len(),"result_sha256":digest_hex(&observation.bytes),
        "canonical_envelope_equal":true,"resource_closure":"closed",
        "wall_time_is_not_cpu_or_RSS":true,"full_corpus_runtime_acceptance":"NotEvaluated"
    });
    write_new_capped(
        &capture.join("proof.json"),
        &wow_core::canonical_json_bytes(&proof)?,
        MAX_CASE_BYTES,
    )?;
    Ok(())
}
