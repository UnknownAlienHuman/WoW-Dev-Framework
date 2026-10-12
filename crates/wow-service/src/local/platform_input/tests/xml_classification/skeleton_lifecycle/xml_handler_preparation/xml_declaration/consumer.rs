//! One selected application lifecycle on the exact prepared native declaration.
use super::*;

const CAPTURE: &str = ".codex/work/platform-xml-declaration-consumer-20261012";

#[test]
#[ignore = "one retained native XML declaration consumer; not full E3 acceptance"]
fn selected_xml_declaration_consumer_lifecycle() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let prepared_dir = repo.join(WORK);
    let capture = repo.join(CAPTURE);
    std::fs::create_dir(&capture)?;
    let prepared: Value = serde_json::from_slice(&bounded_bytes(
        &prepared_dir.join("prepared.json"),
        MAX_SELECTED_EXPECTED_BYTES,
    )?)?;
    let host_bytes = bounded_bytes(&prepared_dir.join("host.json"), MAX_HOST_BYTES)?;
    assert_eq!(digest_hex(&host_bytes), prepared["host_sha256"]);
    let host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let stop = AtomicBool::new(false);
    let host = crate::context::ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let inspect_wire: Value = serde_json::from_slice(&bounded_bytes(
        &prepared_dir.join("inspect-request.json"),
        MAX_REQUEST_BYTES,
    )?)?;
    assert_eq!(inspect_wire, prepared["selected_inspect_request"]);
    assert_eq!(prepared["resource_closure"], "closed");
    assert_eq!(
        prepared["original"]["root"]["kind"],
        "xml_source_declaration"
    );

    let inspect = run_combined_service(&host, &inspect_wire, &stop)?;
    write_new_capped(
        &capture.join("inspect-service.json"),
        &inspect.bytes,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    require_closed(&inspect.envelope)?;
    assert_eq!(inspect.failure_code, None);
    let original_inspect = &inspect.envelope["payload"]["combined_inspect"];
    let core = &original_inspect["core"];
    assert!(core.is_object());
    assert_eq!(core["platform_root_xml_declaration"], prepared["original"]);
    assert_eq!(
        core["map"]["core"]["platform"]["core"]["native"],
        prepared["original_page"]
    );
    assert_eq!(
        core["platform_inspect"]["map"],
        core["map"]["core"]["platform"]
    );
    assert!(core.get("platform_root_callable").is_none());
    assert!(core.get("platform_root_xml_handler").is_none());
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(core["cross_universe_relations"], "not_evaluated");

    // Unchanged original Primary map from the same exact retained tuple. The
    // platform root differs, so combined and skeleton ledgers are not byte oracles.
    let old_primary: Value = serde_json::from_slice(&bounded_bytes(
        &repo
            .join(BASELINE_RELATIVE)
            .join("expected-inspect-stdout.json"),
        MAX_SELECTED_EXPECTED_BYTES,
    )?)?;
    assert_eq!(old_primary["request"]["primary"], inspect_wire["primary"]);
    assert_eq!(
        core["map"]["core"]["primary"],
        old_primary["payload"]["combined_inspect"]["core"]["map"]["core"]["primary"]
    );

    let mut build_wire = inspect_wire.clone();
    build_wire["operation"] = json!("build");
    build_wire
        .as_object_mut()
        .ok_or("request object")?
        .remove("inspection");
    build_wire["build"] = json!({
        "level": inspect_wire["inspection"]["level"],
        "context_profile_id": "wow-context/combined-native-structural-pack/1",
        "platform_root_facet": "xml_declaration_metadata",
        "renderer_profile_ids": ["wow-context/combined-pack-json/1", "wow-context/combined-pack-markdown/1"]
    });
    write_new_capped(&capture.join("host.json"), &host_bytes, MAX_HOST_BYTES)?;
    write_new_capped(
        &capture.join("build-request.json"),
        &wow_core::canonical_json_bytes(&build_wire)?,
        MAX_REQUEST_BYTES,
    )?;
    let build = run_combined_service(&host, &build_wire, &stop)?;
    write_new_capped(
        &capture.join("build-service.json"),
        &build.bytes,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    require_closed(&build.envelope)?;
    assert_eq!(build.failure_code, None);
    let pack = &build.envelope["payload"]["semantic_pack"];
    assert!(pack.is_object());
    assert_eq!(pack["core"]["origins"], *original_inspect);
    let artifacts = build.envelope["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("rendered artifacts")?;
    assert_eq!(artifacts.len(), 2);
    let profiles: Value = serde_json::from_slice(wow_context::COMBINED_CONTEXT_PROFILES_BYTES)?;
    for (index, artifact) in artifacts.iter().enumerate() {
        let rendered = &artifact["core"];
        assert_eq!(rendered["semantic_pack_id"], pack["semantic_pack_id"]);
        assert_eq!(
            rendered["profile"],
            profiles[if index == 0 { "json" } else { "markdown" }]
        );
        let content = rendered["content"].as_str().ok_or("renderer content")?;
        assert_eq!(rendered["byte_length"], content.len());
        assert_eq!(rendered["content_digest"], digest(content.as_bytes()));
        if index == 0 {
            assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
        } else {
            let encoded = content
                .lines()
                .find_map(|line| line.strip_prefix("ORIGINS "))
                .ok_or("Markdown origins")?;
            let quoted: String = serde_json::from_str(encoded)?;
            assert_eq!(serde_json::from_str::<Value>(&quoted)?, *original_inspect);
            let trace = rendered["trace"]
                .as_array()
                .ok_or("Markdown trace")?
                .iter()
                .find(|row| row["template_id"] == "combined_native_origins")
                .ok_or("origins trace")?;
            assert_eq!(trace["record_id"], original_inspect["inspect_id"]);
        }
    }
    let usage = &build.envelope["payload"]["operation_budget_usage"];
    for (field, limit) in [
        ("work_reserved", "max_context_work"),
        ("bytes_reserved", "max_metadata_bytes"),
    ] {
        let reserved = usage[field].as_u64().ok_or("operation reservation")?;
        assert!(
            reserved
                > pack["core"]["budget"][field]
                    .as_u64()
                    .ok_or("pack reservation")?
        );
        assert!(
            reserved
                <= host_wire["primary"]["limits"][limit]
                    .as_u64()
                    .ok_or("hard limit")?
        );
    }
    assert!(
        usage["bytes_reserved"]
            .as_u64()
            .ok_or("byte reservation")?
            .checked_add(u64::try_from(build.bytes.len())?)
            .ok_or("delivery overflow")?
            <= host_wire["primary"]["limits"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("delivery limit")?
    );

    let mut wrong_wire = prepared["wrong_kind_request"].clone();
    wrong_wire["inspection"]["platform_root_facet"] = json!("xml_declaration_metadata");
    let wrong = run_combined_service(&host, &wrong_wire, &stop)?;
    write_new_capped(
        &capture.join("wrong-kind-service.json"),
        &wrong.bytes,
        MAX_EXPORT_BYTES,
    )?;
    require_closed(&wrong.envelope)?;
    require_no_payload(&wrong.envelope)?;
    assert_eq!(wrong.failure_code, Some(ServiceErrorCode::InvalidRequest));

    let mut low_host = host_wire.clone();
    for owner in ["primary", "platform"] {
        low_host[owner]["limits"]["max_context_work"] = json!(1);
    }
    let low_host = crate::context::ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&low_host)?,
        &stop,
    )?;
    let exhausted = run_combined_service(&low_host, &build_wire, &stop)?;
    write_new_capped(
        &capture.join("work-cap-service.json"),
        &exhausted.bytes,
        MAX_EXPORT_BYTES,
    )?;
    require_closed(&exhausted.envelope)?;
    require_no_payload(&exhausted.envelope)?;
    assert_eq!(
        exhausted.failure_code,
        Some(ServiceErrorCode::BudgetExceeded)
    );

    let proof = json!({
        "root": prepared["root"], "counts": prepared["counts"], "service_calls": 4,
        "prepared_original_facet_equal": true, "original_native_page_equal": true,
        "original_primary_map_equal": true, "full_inspect_in_pack_and_render_origins": true,
        "continuing_budget_and_delivery_headroom": true,
        "inspect": {"decode_ms":inspect.decode_ms,"service_ms":inspect.service_ms,"bytes":inspect.bytes.len(),"status":inspect.envelope["status"]},
        "build": {"decode_ms":build.decode_ms,"service_ms":build.service_ms,"bytes":build.bytes.len(),"status":build.envelope["status"],"sha256":digest_hex(&build.bytes)},
        "wrong_kind": {"service_ms":wrong.service_ms,"failure":wrong.envelope["failure_code"],"closure":wrong.envelope["resource_closure"]},
        "work_cap": {"service_ms":exhausted.service_ms,"failure":exhausted.envelope["failure_code"],"closure":exhausted.envelope["resource_closure"]},
        "operation_budget_usage":usage,
        "source_absence_current_recheck":"NotEvaluated", "full_E3_Tier_A_X_runtime_launch":"NotEvaluated"
    });
    write_new_capped(
        &capture.join("proof.json"),
        &wow_core::canonical_json_bytes(&proof)?,
        MAX_CASE_BYTES,
    )?;
    Ok(())
}
