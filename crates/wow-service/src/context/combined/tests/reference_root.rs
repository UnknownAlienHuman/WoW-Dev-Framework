//! One retained native API-root Both witness; no source/fixture regeneration.
use super::*;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

const INPUT: &str = ".codex/work/native-combined-reference-20261012T183238Z";
const PREPARED: &str = ".codex/work/luna-reference-root-l0-witness-20261012";
const ORIGINAL: &str = ".codex/work/luna-combined-recipe-composition-witness-20261012";
const MAX_BYTES: usize = 4 * 1024 * 1024;

fn bytes(path: &Path) -> TestResult<Vec<u8>> {
    let mut result = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut result)?;
    if result.len() > MAX_BYTES {
        return Err("bounded witness input/output exceeded".into());
    }
    Ok(result)
}
fn write(path: &Path, value: &[u8]) -> TestResult {
    if value.len() > MAX_BYTES {
        return Err("bounded witness export exceeded".into());
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(value)?;
    Ok(())
}
fn value(path: &Path) -> TestResult<Value> {
    Ok(serde_json::from_slice(&bytes(path)?)?)
}

// Identity includes cumulative reservations. Compare the complete semantic core
// to the independent owner, removing only its two different accounting leaves.
fn semantic_core(skeleton: &Value) -> TestResult<Value> {
    let mut core = skeleton["core"].clone();
    let context = core["context"].as_object_mut().ok_or("skeleton context")?;
    for field in ["work_reserved", "facet_bytes_reserved"] {
        if context.remove(field).is_none() {
            return Err("mandatory accounting field absent".into());
        }
    }
    Ok(core)
}

fn native_oracles(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    stop: &AtomicBool,
) -> TestResult<(Value, Value, wow_graph::GraphNodeId)> {
    let selector = ReadSelector::Exact(request.primary.store_generation_id.clone());
    let (result, closure) = with_checked_context_read(
        &host.primary,
        &request.primary_request.project_store_id,
        &selector,
        stop,
        |read| {
            let root = qualify_primary(read, host, request)?;
            let scope = match &root {
                ContextMapRoot::GraphEntity(id) => wow_context::L0Scope::Entity(id.clone()),
                _ => return Err(failure(ServiceErrorCode::InvalidRequest)),
            };
            let files: Vec<_> = read
                .graph()
                .snapshot()
                .nodes()
                .iter()
                .filter(|node| node.kind() == "source_file")
                .map(|node| node.node_id().clone())
                .collect();
            let [wrong_kind] = files.as_slice() else {
                return Err(failure(ServiceErrorCode::InternalContractViolation));
            };
            let universe = wow_context::bind_context_universe_set(
                read.pair(),
                &host.primary.reference,
                &host.primary.reference_profile,
                host.primary.limits.owner_limits()?,
                stop,
            )
            .map_err(context_error)?;
            with_primary_composition(
                &universe,
                read,
                &host.primary,
                root,
                true,
                stop,
                |primary, graph| {
                    let api = read
                        .native_api()
                        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
                    let origins = api.reference_origins(primary.map(), graph, stop)?;
                    let l0 = wow_context::build_primary_reference_root_l0_skeleton(
                        &universe,
                        primary.map(),
                        graph,
                        &scope,
                        &origins,
                        stop,
                    )
                    .map_err(context_error)?;
                    let l1 = wow_context::build_primary_reference_skeletons(
                        &universe,
                        primary.map(),
                        graph,
                        &scope,
                        &origins,
                        stop,
                    )
                    .map_err(context_error)?;
                    let missing = wow_context::build_primary_reference_root_l0_skeleton(
                        &universe,
                        primary.map(),
                        graph,
                        &scope,
                        &[],
                        stop,
                    );
                    if !matches!(missing, Err(error) if error.code() == wow_context::ContextErrorCode::MissingRequiredCustody)
                    {
                        return Err(failure(ServiceErrorCode::InternalContractViolation));
                    }
                    let cancelled = wow_context::build_primary_reference_root_l0_skeleton(
                        &universe,
                        primary.map(),
                        graph,
                        &scope,
                        &origins,
                        &AtomicBool::new(true),
                    );
                    if !matches!(cancelled, Err(error) if error.code() == wow_context::ContextErrorCode::Cancelled)
                    {
                        return Err(failure(ServiceErrorCode::InternalContractViolation));
                    }
                    let l0 = serde_json::from_slice(l0.canonical_bytes())
                        .map_err(|_| failure(ServiceErrorCode::InternalContractViolation))?;
                    let l1 = serde_json::to_value(l1)
                        .map_err(|_| failure(ServiceErrorCode::InternalContractViolation))?;
                    Ok((l0, l1, wrong_kind.clone()))
                },
            )
        },
    );
    assert_eq!(closure, "closed");
    Ok(result?)
}

fn run(
    host: &ContextCombinedHostInput,
    request: &Value,
    out: &Path,
    stop: &AtomicBool,
) -> TestResult<(Value, u128, usize)> {
    let request =
        ContextCombinedRequest::from_json_slice(&wow_core::canonical_json_bytes(request)?, stop)?;
    let timer = Instant::now();
    let result = execute_context_combined(host, &request, stop)?;
    let elapsed = timer.elapsed().as_millis();
    write(out, result.canonical_bytes())?;
    Ok((
        serde_json::from_slice(result.canonical_bytes())?,
        elapsed,
        result.canonical_bytes().len(),
    ))
}

fn closed_failure(envelope: &Value, code: &str) {
    assert_eq!(envelope["status"], "failed");
    assert_eq!(envelope["failure_code"], code);
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("payload").is_none());
}

#[test]
#[ignore = "one explicit retained native Reference-root Both witness, fresh absolute export required"]
fn retained_native_reference_root_both_inspect_build() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let out = PathBuf::from(
        std::env::var_os("WOW_REFERENCE_ROOT_L0_WITNESS").ok_or("fresh witness export required")?,
    );
    if !out.is_absolute() || out.as_os_str().len() > 32768 {
        return Err("finite absolute export required".into());
    }
    std::fs::create_dir(&out)?;
    let stop = AtomicBool::new(false);
    let host_bytes = bytes(&repo.join(INPUT).join("host.json"))?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let inspect_request = value(&repo.join(PREPARED).join("inspect-request.json"))?;
    let build_request = value(&repo.join(PREPARED).join("build-request.json"))?;
    assert_eq!(inspect_request["primary"], build_request["primary"]);
    assert_eq!(inspect_request["platform"], build_request["platform"]);
    let original = value(
        &repo
            .join(ORIGINAL)
            .join("reference_facts_l1_callable_metadata-inspect-service.json"),
    )?;
    let native_request = ContextCombinedRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&inspect_request)?,
        &stop,
    )?;
    let timer = Instant::now();
    let (original_l0, original_l1, wrong_kind) = native_oracles(&host, &native_request, &stop)?;
    let owner_ms = timer.elapsed().as_millis();
    write(
        &out.join("original-l0.json"),
        &wow_core::canonical_json_bytes(&original_l0)?,
    )?;
    write(
        &out.join("original-l1.json"),
        &wow_core::canonical_json_bytes(&original_l1)?,
    )?;
    let (inspect, inspect_ms, inspect_bytes) = run(
        &host,
        &inspect_request,
        &out.join("inspect-service.json"),
        &stop,
    )?;
    assert_eq!(inspect["status"], "partial");
    assert_eq!(inspect["resource_closure"], "closed");
    assert!(inspect.get("failure_code").is_none());
    let selected = &inspect["payload"]["combined_inspect"];
    let core = &selected["core"];
    let old = &original["payload"]["combined_inspect"]["core"];
    assert_eq!(core["map"], old["map"]);
    assert_eq!(core["platform_inspect"], old["platform_inspect"]);
    assert_eq!(
        core["platform_root_callable"],
        old["platform_root_callable"]
    );
    let skeletons = &core["primary_skeletons"];
    assert_eq!(skeletons["level"], "both");
    assert_eq!(
        semantic_core(&skeletons["l0"])?,
        semantic_core(&original_l0)?
    );
    assert_eq!(
        semantic_core(&skeletons["l1"])?,
        semantic_core(&original_l1["l1"])?
    );
    assert_eq!(
        semantic_core(&skeletons["l1"])?,
        semantic_core(&old["primary_skeletons"]["l1"])?
    );
    let l0 = &skeletons["l0"]["core"];
    assert_eq!(
        l0["profile"],
        wow_context::PRIMARY_REFERENCE_ROOT_L0_PROFILE
    );
    assert_eq!(
        l0["schema"],
        "wow-context/l0-skeleton/primary-reference-root/1"
    );
    let root_facet = &l0["reference_root"];
    let record = &skeletons["l1"]["core"]["reference_records"][0];
    for field in [
        "node",
        "assertion",
        "partition_id",
        "partition_coverage",
        "record_digest",
        "source",
        "evidence",
        "source_association",
        "absence_authoritative",
    ] {
        assert_eq!(
            root_facet[field], record[field],
            "original L0/L1 custody {field}"
        );
    }
    assert_eq!(root_facet["record_key"], record["record"]["key"]);
    assert_eq!(
        root_facet["node"]["node_id"],
        inspect_request["primary"]["root"]["id"]
    );
    assert_eq!(
        l0["detail_routes"],
        json!([inspect_request["primary"]["root"]["id"]])
    );
    assert_eq!(l0["member_enumeration"], "not_evaluated");
    assert_eq!(l0["direct_members"], json!([]));
    assert_eq!(l0["direct_relations"], json!([]));
    assert!(root_facet.get("record").is_none() && root_facet.get("signature").is_none());
    let (build, build_ms, build_bytes) = run(
        &host,
        &build_request,
        &out.join("build-service.json"),
        &stop,
    )?;
    assert_eq!(build["status"], "partial");
    assert_eq!(build["resource_closure"], "closed");
    assert!(build.get("failure_code").is_none());
    let pack = &build["payload"]["semantic_pack"];
    assert_eq!(pack["core"]["origins"], *selected);
    let artifacts = build["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("render artifacts")?;
    assert_eq!(artifacts.len(), 2);
    for (index, artifact) in artifacts.iter().enumerate() {
        assert_eq!(
            artifact["core"]["semantic_pack_id"],
            pack["semantic_pack_id"]
        );
        let content = artifact["core"]["content"]
            .as_str()
            .ok_or("render content")?;
        assert_eq!(artifact["core"]["byte_length"], content.len());
        if index == 0 {
            assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
        } else {
            let encoded = content
                .lines()
                .find_map(|line| line.strip_prefix("ORIGINS "))
                .ok_or("Markdown origins")?;
            let decoded: String = serde_json::from_str(encoded)?;
            assert_eq!(serde_json::from_str::<Value>(&decoded)?, *selected);
            let trace = artifact["core"]["trace"]
                .as_array()
                .ok_or("origins trace")?
                .iter()
                .find(|row| row["template_id"] == "combined_native_origins")
                .ok_or("origins trace")?;
            assert_eq!(trace["record_id"], selected["inspect_id"]);
        }
    }
    let host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let usage = &build["payload"]["operation_budget_usage"];
    for (field, limit) in [
        ("work_reserved", "max_context_work"),
        ("bytes_reserved", "max_metadata_bytes"),
    ] {
        let reserved = usage[field].as_u64().ok_or("reservation")?;
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
            .ok_or("bytes")?
            .checked_add(build_bytes as u64)
            .ok_or("delivery overflow")?
            <= host_wire["primary"]["limits"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("metadata limit")?
    );
    let mut wrong_request = inspect_request.clone();
    wrong_request["primary"]["root"]["id"] = json!(wrong_kind);
    let (wrong, wrong_ms, _) = run(
        &host,
        &wrong_request,
        &out.join("wrong-kind-service.json"),
        &stop,
    )?;
    closed_failure(&wrong, "invalid_request");
    let mut limited = host_wire.clone();
    limited["primary"]["limits"]["max_context_work"] = json!(1);
    limited["platform"]["limits"]["max_context_work"] = json!(1);
    let limited = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&limited)?,
        &stop,
    )?;
    let (refused, refused_ms, _) = run(
        &limited,
        &build_request,
        &out.join("work-cap-service.json"),
        &stop,
    )?;
    closed_failure(&refused, "budget_exceeded");
    let proof = json!({"owner_ms":owner_ms,"inspect_ms":inspect_ms,"build_ms":build_ms,"wrong_kind_ms":wrong_ms,"budget_refusal_ms":refused_ms,
        "inspect_bytes":inspect_bytes,"build_bytes":build_bytes,"root":inspect_request["primary"]["root"]["id"],
        "semantic_pack_id":pack["semantic_pack_id"],"operation_budget_usage":usage,"independent_l0_l1_semantic_core_equal":true,
        "accounting_only_comparison_exclusions":["core.context.work_reserved","core.context.facet_bytes_reserved"],
        "full_original_map_platform_inspect_facet_equal":true,"full_inspect_pack_origins_equal":true,"renderers_equal_original_pack":true,
        "missing_custody_and_cancelled_owner_refusals":true,"wrong_kind_and_budget_service_refusals_closed":true,
        "current_source_absence_recheck":"NotEvaluated","graph_rebuilds":"unmeasured","physical_edge_visits":"unmeasured"});
    write(
        &out.join("proof.json"),
        &wow_core::canonical_json_bytes(&proof)?,
    )?;
    Ok(())
}
