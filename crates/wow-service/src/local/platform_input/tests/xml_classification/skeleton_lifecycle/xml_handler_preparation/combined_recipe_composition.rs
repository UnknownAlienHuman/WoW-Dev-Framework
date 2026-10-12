//! Bounded retained-input witness for composing explicit Primary recipes with
//! an original native platform-root facet. Root runs this only after product
//! integration; no fixture or source is regenerated here.

use super::*;
use serde_json::{Value, json};

const ROOT_RELATIVE: &str = ".codex/work/luna-combined-recipe-composition-witness-20261012";
const XML_RELATIVE: &str = ".codex/work/luna-platform-root-xml-handler-witness-20261012";
const CALLABLE_RELATIVE: &str = ".codex/work/platform-callable-witness-service-export-20261012";
const STATE_ROOT_RELATIVE: &str = ".codex/work/combined-state-l0-qualified-20261012";
const STATE_DETAIL_RELATIVE: &str = ".codex/work/combined-state-both-one-edge-qualified-20261012";
const REFERENCE_RELATIVE: &str = ".codex/work/native-combined-reference-20261012T183238Z";
const MAX_INPUT: usize = 4 * 1024 * 1024;
const MAX_OUTPUT: usize = 4 * 1024 * 1024;

fn bounded_json(path: &Path) -> TestResult<(Vec<u8>, Value)> {
    let bytes = bounded_bytes(path, MAX_INPUT)?;
    let value = serde_json::from_slice(&bytes)?;
    Ok((bytes, value))
}

fn one_lf_canonical(path: &Path, expected_sha256: &str) -> TestResult<Value> {
    let bytes = bounded_bytes(path, MAX_INPUT)?;
    if bytes.last() != Some(&b'\n')
        || bytes.iter().filter(|byte| **byte == b'\n').count() != 1
        || digest_hex(&bytes) != expected_sha256
    {
        return Err(format!("frozen CLI baseline changed: {}", path.display()).into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn combined_core(envelope: &Value) -> TestResult<&Value> {
    envelope["payload"]["combined_inspect"]["core"]
        .as_object()
        .map(|_| &envelope["payload"]["combined_inspect"]["core"])
        .ok_or_else(|| "retained combined Inspect core absent".into())
}

fn expected_facet(envelope: &Value, field: &str) -> TestResult<Value> {
    let core = combined_core(envelope)?;
    core[field]
        .as_object()
        .map(|_| core[field].clone())
        .ok_or_else(|| format!("retained native facet {field} absent").into())
}

struct FrozenCapture<'a> {
    stem: &'a str,
    request_sha: &'a str,
    stdout_sha: &'a str,
    stdout_len: usize,
    status: &'a str,
    facet: &'a str,
    host_path: PathBuf,
    host_sha: &'a str,
}

fn verify_frozen_cli_capture(work: &Path, capture: FrozenCapture<'_>) -> TestResult<Value> {
    let FrozenCapture {
        stem,
        request_sha,
        stdout_sha,
        stdout_len,
        status,
        facet,
        host_path,
        host_sha,
    } = capture;
    let request_path = work.join(format!("{stem}-request.json"));
    let stdout_path = work.join(format!("{stem}-inspect-stdout.json"));
    let proof_path = work.join(format!("{stem}-proof.json"));
    let request_bytes = bounded_bytes(&request_path, MAX_INPUT)?;
    let host_bytes = bounded_bytes(&host_path, MAX_INPUT)?;
    if digest_hex(&request_bytes) != request_sha {
        return Err(format!("frozen request changed: {}", request_path.display()).into());
    }
    if digest_hex(&host_bytes) != host_sha {
        return Err(format!("frozen capture host changed: {}", host_path.display()).into());
    }
    let (_, proof) = bounded_json(&proof_path)?;
    if proof["base"] != "bf5adcc448fa8d89ea5faa46e7d9ad1ac6114369"
        || proof["binary_sha256"]
            != "b4d24093858be7ba4a37ddb958eb031d4c179bca8b19f821ca1ba469782fe63e"
        || proof["stdout_sha256"] != stdout_sha
        || proof["stdout_bytes"] != stdout_len
        || proof["stderr_bytes"] != 0
        || proof["status"] != status
        || proof["closure"] != "closed"
        || proof["facet"] != facet
    {
        return Err(format!("frozen proof changed: {}", proof_path.display()).into());
    }
    one_lf_canonical(&stdout_path, stdout_sha)
}

struct Recipe {
    name: &'static str,
    host_path: PathBuf,
    inspect_request: Value,
    build_request: Value,
    old_inspect: Value,
    facet_name: &'static str,
    facet_field: &'static str,
    original_facet: Value,
}

struct NativeFacet {
    name: &'static str,
    field: &'static str,
    original: Value,
}

fn recipe(
    name: &'static str,
    host_path: PathBuf,
    inspect_path: &Path,
    build_path: &Path,
    old_inspect_path: &Path,
    facet: NativeFacet,
) -> TestResult<Recipe> {
    let NativeFacet {
        name: facet_name,
        field: facet_field,
        original: original_facet,
    } = facet;
    let (_, mut inspect_request) = bounded_json(inspect_path)?;
    let (_, mut build_request) = bounded_json(build_path)?;
    let (_, old_inspect) = bounded_json(old_inspect_path)?;
    inspect_request["inspection"]["platform_root_facet"] = json!(facet_name);
    build_request["primary"] = inspect_request["primary"].clone();
    build_request["platform"] = inspect_request["platform"].clone();
    build_request["operation"] = json!("build");
    build_request["build"]["platform_root_facet"] = json!(facet_name);
    build_request["build"]["renderer_profile_ids"] = json!([
        "wow-context/combined-pack-json/1",
        "wow-context/combined-pack-markdown/1"
    ]);
    if inspect_request["operation"] != "inspect"
        || inspect_request["primary"] != build_request["primary"]
        || inspect_request["platform"] != build_request["platform"]
    {
        return Err(format!("{name}: selected request pair changed owner custody").into());
    }
    let requested_root = &inspect_request["platform"]["input"]["neighborhood"]["query"]["node_id"];
    if original_facet["root"]["node_id"] != *requested_root
        || original_facet["selection"] != facet_name
    {
        return Err(
            format!("{name}: native facet does not match exact retained root/selection").into(),
        );
    }
    Ok(Recipe {
        name,
        host_path,
        inspect_request,
        build_request,
        old_inspect,
        facet_name,
        facet_field,
        original_facet,
    })
}

struct RecipeObservation {
    inspect: Value,
    build_bytes: Vec<u8>,
    proof: Value,
}

fn exercise_recipe(
    case: &Recipe,
    capture_dir: &Path,
    stop: &AtomicBool,
) -> TestResult<RecipeObservation> {
    let (host_bytes, host_wire) = bounded_json(&case.host_path)?;
    let host = crate::context::ContextCombinedHostInput::from_json_slice(&host_bytes, stop)?;
    let inspect = run_combined_service(&host, &case.inspect_request, stop)?;
    write_new_capped(
        &capture_dir.join(format!("{}-inspect-service.json", case.name)),
        &inspect.bytes,
        MAX_OUTPUT,
    )?;
    require_closed(&inspect.envelope)?;
    assert_eq!(inspect.failure_code, None, "{} Inspect", case.name);
    let selected_core = combined_core(&inspect.envelope)?;
    let old_core = combined_core(&case.old_inspect)?;
    assert_eq!(
        selected_core["primary_skeletons"], old_core["primary_skeletons"],
        "{} full original Primary skeleton",
        case.name
    );
    assert_eq!(
        selected_core[case.facet_field], case.original_facet,
        "{} original platform facet",
        case.name
    );
    assert_eq!(
        selected_core["map"], old_core["map"],
        "{} original full combined map",
        case.name
    );
    let selected_combined = &inspect.envelope["payload"]["combined_inspect"];

    let build = run_combined_service(&host, &case.build_request, stop)?;
    write_new_capped(
        &capture_dir.join(format!("{}-build-service.json", case.name)),
        &build.bytes,
        MAX_OUTPUT,
    )?;
    require_closed(&build.envelope)?;
    assert_eq!(build.failure_code, None, "{} Build", case.name);
    let pack = &build.envelope["payload"]["semantic_pack"];
    assert!(pack.is_object());
    assert_eq!(pack["core"]["origins"], *selected_combined);
    assert_eq!(
        pack["core"]["origins"]["core"]["primary_skeletons"],
        selected_core["primary_skeletons"]
    );
    assert_eq!(
        pack["core"]["origins"]["core"][case.facet_field],
        case.original_facet
    );
    let artifacts = build.envelope["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("both-renderer artifacts absent")?;
    assert_eq!(artifacts.len(), 2);
    let pack_id = &pack["semantic_pack_id"];
    let pack_bytes = wow_core::canonical_json_bytes(pack)?;
    let profiles: Value = serde_json::from_slice(wow_context::COMBINED_CONTEXT_PROFILES_BYTES)?;
    for (index, artifact) in artifacts.iter().enumerate() {
        let core = &artifact["core"];
        assert_eq!(core["semantic_pack_id"], *pack_id);
        assert_eq!(
            core["profile"],
            profiles[if index == 0 { "json" } else { "markdown" }]
        );
        let content = core["content"].as_str().ok_or("renderer content missing")?;
        assert_eq!(core["byte_length"], content.len());
        assert_eq!(core["content_digest"], digest(content.as_bytes()));
        if index == 0 {
            assert_eq!(content.as_bytes(), pack_bytes.as_slice());
        } else {
            let encoded = content
                .lines()
                .find_map(|line| line.strip_prefix("ORIGINS "))
                .ok_or("Markdown original origins missing")?;
            let quoted: String = serde_json::from_str(encoded)?;
            assert_eq!(serde_json::from_str::<Value>(&quoted)?, *selected_combined);
            let trace = core["trace"]
                .as_array()
                .ok_or("Markdown trace missing")?
                .iter()
                .find(|row| row["template_id"] == "combined_native_origins")
                .ok_or("Markdown origins trace missing")?;
            assert_eq!(trace["record_id"], selected_combined["inspect_id"]);
            assert_eq!(trace["semantic_fields"], json!(["core.origins"]));
        }
    }
    let usage = &build.envelope["payload"]["operation_budget_usage"];
    for (field, limit) in [
        ("work_reserved", "max_context_work"),
        ("bytes_reserved", "max_metadata_bytes"),
    ] {
        let total = usage[field].as_u64().ok_or("Build reservation")?;
        assert!(
            total
                > pack["core"]["budget"][field]
                    .as_u64()
                    .ok_or("original pack reservation")?
        );
        assert!(
            total
                <= host_wire["primary"]["limits"][limit]
                    .as_u64()
                    .ok_or("Primary hard limit")?
        );
    }
    let delivery_bytes = usage["bytes_reserved"]
        .as_u64()
        .ok_or("Build reserved bytes")?
        .checked_add(u64::try_from(build.bytes.len())?)
        .ok_or("Build delivery byte overflow")?;
    assert!(
        delivery_bytes
            <= host_wire["primary"]["limits"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("metadata hard limit")?
    );
    let proof = json!({
        "recipe": case.name,
        "facet": case.facet_name,
        "root": case.inspect_request["platform"]["input"]["neighborhood"]["query"]["node_id"],
        "inspect_status": inspect.envelope["status"],
        "build_status": build.envelope["status"],
        "primary_skeletons_sha256": digest(&wow_core::canonical_json_bytes(&selected_core["primary_skeletons"])?),
        "facet_sha256": digest(&wow_core::canonical_json_bytes(&selected_core[case.facet_field])?),
        "semantic_pack_id": pack_id,
        "origins_equal_selected_inspect": true,
        "renderer_count": artifacts.len(),
        "inspect_ms": {"decode": inspect.decode_ms, "service": inspect.service_ms},
        "build_ms": {"decode": build.decode_ms, "service": build.service_ms},
        "inspect_bytes": inspect.bytes.len(),
        "build_bytes": build.bytes.len(),
        "build_operation_budget_usage": usage,
        "pack_budget": pack["core"]["budget"],
        "continuing_budget_and_delivery_headroom": true,
        "full_primary_skeleton_equal": true,
        "full_original_platform_facet_equal": true
    });
    Ok(RecipeObservation {
        inspect: inspect.envelope,
        build_bytes: build.bytes,
        proof,
    })
}

#[test]
#[ignore = "retained combined-recipe composition witness; root runs once after product integration"]
fn retained_combined_recipe_platform_facet_composition() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let work = repo.join(ROOT_RELATIVE);
    let state_root = repo.join(STATE_ROOT_RELATIVE);
    let state_detail = repo.join(STATE_DETAIL_RELATIVE);
    let reference = repo.join(REFERENCE_RELATIVE);
    let xml = repo.join(XML_RELATIVE);
    let callable = repo.join(CALLABLE_RELATIVE);

    let _frozen_state_detail = verify_frozen_cli_capture(
        &work,
        FrozenCapture {
            stem: "state-detail-both-xml-root-old-baseline",
            request_sha: "2c8ec7feb56d020e65da29a3c72b3a4abf4f5cf5fd6bcc4b49e0d37d227ddcc5",
            stdout_sha: "29333cfa7fed46cc8da8bbbdb749f4dd7c65a91cb725c8671f95cef7397cd05e",
            stdout_len: 497_062,
            status: "truncated",
            facet: "",
            host_path: state_detail.join("inspect-root-both-host.json"),
            host_sha: "eb80a2da8fd2c454366de3aecf8fce639db63649fce03dd530f55ab329ea7af3",
        },
    )?;
    let _frozen_reference = verify_frozen_cli_capture(
        &work,
        FrozenCapture {
            stem: "reference-platform-callable-original",
            request_sha: "46905276eeeed2cc677e2b08713a735f4e85fa06d1b21bba539e9ac30e1371b7",
            stdout_sha: "d636cdbec5f33d777866747daf8851d183ea1aeb324bae832d68d4d72b269a31",
            stdout_len: 270_669,
            status: "partial",
            facet: "platform_root_callable",
            host_path: reference.join("host.json"),
            host_sha: "e0ef732e0ae07ebcff114bbea5c4caa606fe14d07603ccd695493c4d536222ea",
        },
    )?;
    let (_, callable_owner) = bounded_json(&callable.join("expected-inspect-envelope.json"))?;
    let callable_fc81 = expected_facet(&callable_owner, "platform_root_callable")?;
    let (_, xml_prepared) = bounded_json(&xml.join("prepared-xml-handler-root.json"))?;
    let source_case_path = Path::new(
        xml_prepared["source_absence"]["source_case"]
            .as_str()
            .ok_or("retained State source-absence case path")?,
    );
    if !source_case_path.is_absolute() {
        return Err("retained State source-absence case path must be absolute".into());
    }
    let (_, source_case) = bounded_json(source_case_path)?;
    let absent_paths = source_case["source_absent_paths"]
        .as_array()
        .ok_or("retained State source_absent_paths")?;
    if absent_paths.is_empty()
        || xml_prepared["source_absence"]["status"] != "verified_absent_paths"
        || xml_prepared["source_absence"]["absent_path_count"] != absent_paths.len()
    {
        return Err("retained State source-absence evidence is inconsistent".into());
    }
    for item in absent_paths {
        let path = Path::new(item.as_str().ok_or("absolute State source-absence path")?);
        if !path.is_absolute() || path.try_exists()? {
            return Err("retained State source path is not absent".into());
        }
    }
    let xml_original = xml_prepared["original"].clone();
    let (_, reference_capture) =
        bounded_json(&work.join("reference-platform-callable-original-inspect-stdout.json"))?;
    let callable_f250 = expected_facet(&reference_capture, "platform_root_callable")?;

    let cases = [
        recipe(
            "state_root_l0_callable_metadata",
            state_root.join("inspect-l0-host.json"),
            &state_root.join("inspect-l0-request.json"),
            &state_root.join("build-l0-both-renderers-request.json"),
            &state_root.join("inspect-l0-stdout.json"),
            NativeFacet {
                name: "callable_metadata",
                field: "platform_root_callable",
                original: callable_fc81,
            },
        )?,
        recipe(
            "state_detail_both_xml_handler_metadata",
            state_detail.join("inspect-root-both-host.json"),
            &work.join("state-detail-both-xml-root-old-baseline-request.json"),
            &state_detail.join("build-root-both-request.json"),
            &work.join("state-detail-both-xml-root-old-baseline-inspect-stdout.json"),
            NativeFacet {
                name: "xml_handler_metadata",
                field: "platform_root_xml_handler",
                original: xml_original,
            },
        )?,
        recipe(
            "reference_facts_l1_callable_metadata",
            reference.join("host.json"),
            &reference.join("inspect-request.json"),
            &reference.join("build-request.json"),
            &reference.join("expected-inspect-envelope.json"),
            NativeFacet {
                name: "callable_metadata",
                field: "platform_root_callable",
                original: callable_f250,
            },
        )?,
    ];
    let stop = AtomicBool::new(false);
    let mut recipe_proofs = Vec::with_capacity(cases.len());
    let mut cli_export: Option<(Vec<u8>, Value, Vec<u8>)> = None;
    let mut selected_for_cap: Option<(Vec<u8>, Value)> = None;
    for case in &cases {
        let RecipeObservation {
            inspect,
            build_bytes,
            proof,
        } = exercise_recipe(case, &work, &stop)?;
        if case.name == "state_detail_both_xml_handler_metadata" {
            let (host_bytes, _) = bounded_json(&case.host_path)?;
            let mut request = case.build_request.clone();
            request["build"]["renderer_profile_ids"] = json!([
                "wow-context/combined-pack-json/1",
                "wow-context/combined-pack-markdown/1"
            ]);
            cli_export = Some((host_bytes.clone(), request, build_bytes));
            selected_for_cap = Some((host_bytes, case.inspect_request.clone()));
            assert_eq!(
                combined_core(&inspect)?["platform_root_xml_handler"],
                xml_prepared["original"]
            );
        }
        recipe_proofs.push(proof);
    }

    // One strict work-cap refusal, then one byte-exact existing XML Build
    // compatibility observation. Calls remain sequential and bounded.
    let (bounded_host_bytes, cap_request) = selected_for_cap.ok_or("StateDetail case absent")?;
    let mut bounded_host_wire: Value = serde_json::from_slice(&bounded_host_bytes)?;
    for owner in ["primary", "platform"] {
        bounded_host_wire[owner]["limits"]["max_context_work"] = json!(1);
    }
    let bounded_host = crate::context::ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&bounded_host_wire)?,
        &stop,
    )?;
    let cap = run_combined_service(&bounded_host, &cap_request, &stop)?;
    require_closed(&cap.envelope)?;
    assert_eq!(cap.failure_code, Some(ServiceErrorCode::BudgetExceeded));
    assert_eq!(cap.envelope["failure_code"], "budget_exceeded");
    require_no_payload(&cap.envelope)?;

    let (xml_host_bytes, _) = bounded_json(&xml.join("xml-handler-build-host.json"))?;
    let (_, xml_request) = bounded_json(&xml.join("xml-handler-build-request.json"))?;
    let xml_expected = bounded_bytes(
        &xml.join("xml-handler-build-expected-service.json"),
        MAX_OUTPUT,
    )?;
    if xml_expected.last() == Some(&b'\n')
        || digest(&xml_expected)
            != "sha256:e2870ff049defd6fe0520db9057167711176fc54843c8ed81c43034d10f4b6ac"
    {
        return Err("retained XML Build compatibility bytes changed or are not canonical".into());
    }
    let xml_host =
        crate::context::ContextCombinedHostInput::from_json_slice(&xml_host_bytes, &stop)?;
    let xml_compat = run_combined_service(&xml_host, &xml_request, &stop)?;
    require_closed(&xml_compat.envelope)?;
    assert_eq!(xml_compat.failure_code, None);
    assert_eq!(
        xml_compat.bytes, xml_expected,
        "exact XML Build bytes/counters"
    );

    let (cli_host, cli_request, cli_expected) = cli_export.ok_or("CLI Build export case absent")?;
    let cli_request_bytes = wow_core::canonical_json_bytes(&cli_request)?;
    if cli_expected.last() == Some(&b'\n')
        || cli_expected.len() > MAX_OUTPUT
        || cli_request_bytes.len() > MAX_INPUT
    {
        return Err("both-renderer CLI Build export exceeds framing/size cap".into());
    }
    let proof = json!({
        "schema": "wow-service/combined-recipe-platform-facet-witness/1",
        "base": "bf5adcc448fa8d89ea5faa46e7d9ad1ac6114369",
        "calls": 8,
        "native_store_calls_sequential": true,
        "state_source_absent_paths_rechecked": absent_paths.len(),
        "reference_source_absence_paths_serialized": false,
        "reference_source_absence_current_recheck": "NotEvaluated; retained original removal/reopen owner evidence only",
        "recipes": recipe_proofs,
        "work_cap_1": {
            "failure_code": cap.envelope["failure_code"],
            "resource_closure": cap.envelope["resource_closure"],
            "payload": "absent",
            "decode_ms": cap.decode_ms,
            "service_ms": cap.service_ms,
            "bytes": cap.bytes.len()
        },
        "xml_build_compatibility": {
            "expected_sha256": digest(&xml_expected),
            "observed_sha256": digest(&xml_compat.bytes),
            "bytes_and_counters_exact": true,
            "decode_ms": xml_compat.decode_ms,
            "service_ms": xml_compat.service_ms,
            "bytes": xml_compat.bytes.len()
        },
        "cli_build_export": {
            "request_sha256": digest(&cli_request_bytes),
            "expected_service_sha256": digest(&cli_expected),
            "expected_service_bytes_without_lf": cli_expected.len(),
            "renderer_profile_ids": cli_request["build"]["renderer_profile_ids"]
        }
    });
    write_new_capped(
        &work.join("combined-recipe-cli-build-host.json"),
        &cli_host,
        MAX_INPUT,
    )?;
    write_new_capped(
        &work.join("combined-recipe-cli-build-request.json"),
        &cli_request_bytes,
        MAX_INPUT,
    )?;
    write_new_capped(
        &work.join("combined-recipe-cli-build-expected-service.json"),
        &cli_expected,
        MAX_OUTPUT,
    )?;
    write_new_capped(
        &work.join("combined-recipe-composition-proof.json"),
        &wow_core::canonical_json_bytes(&proof)?,
        64 * 1024,
    )?;
    println!(
        "retained combined-recipe witness exports: {}",
        work.display()
    );
    Ok(())
}
