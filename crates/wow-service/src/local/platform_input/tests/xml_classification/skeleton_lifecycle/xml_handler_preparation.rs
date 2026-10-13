//! One bounded retained-native selection for the pre-integration CLI baseline.
//! This records static owner evidence only; it does not run an analyzer or CLI.

use super::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Instant,
};
use wow_graph::{GraphAssertionKind, GraphLocalAssertion, GraphNode};
use wow_project::{
    load::XmlScriptSource,
    skeleton_input::{OpenSkeletonInputRequest, open_skeleton_input_view},
};

const WORK_RELATIVE: &str = ".codex/work/luna-platform-root-xml-handler-witness-20261012";
const BASELINE_RELATIVE: &str = ".codex/work/platform-callable-default-baseline-20261012";
const MAX_HOST_BYTES: usize = 32 * 1024;
const MAX_REQUEST_BYTES: usize = 32 * 1024;
const MAX_PROOF_BYTES: usize = 16 * 1024;
const MAX_CASE_BYTES: usize = 64 * 1024;
const MAX_EXPORT_BYTES: usize = 512 * 1024;
const MAX_INLINE_HANDLERS: usize = 128;
const MAX_SCRIPT_SOURCES: usize = 4096;
const MAX_FUNCTIONS: usize = 4096;
const MAX_GRAPH_NODES: usize = 4096;
const INTERPRETATION: &str =
    "original_accepted_root_xml_handler_metadata_no_receiver_or_runtime_resolution";

fn bounded_bytes(path: &Path, limit: usize) -> TestResult<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(limit)? + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(format!(
            "bounded preparation input exceeded {limit} bytes: {}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> TestResult<()> {
    if bytes.len() > MAX_EXPORT_BYTES {
        return Err("XML-handler preparation export exceeded its byte cap".into());
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[test]
#[ignore = "PREPARATION ONLY: one retained platform namespace read; no product witness or CLI run"]
fn prepare_platform_xml_handler_root() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let baseline = repo.join(BASELINE_RELATIVE);
    let host_bytes = bounded_bytes(&baseline.join("host.json"), MAX_HOST_BYTES)?;
    let request_bytes = bounded_bytes(&baseline.join("inspect-request.json"), MAX_REQUEST_BYTES)?;
    let proof: Value = serde_json::from_slice(&bounded_bytes(
        &baseline.join("proof.json"),
        MAX_PROOF_BYTES,
    )?)?;
    let case_path = Path::new(
        proof["source_case"]
            .as_str()
            .ok_or("retained source-case path")?,
    );
    if !case_path.is_absolute() {
        return Err("retained source-case path must be absolute".into());
    }
    let source_case: Value = serde_json::from_slice(&bounded_bytes(case_path, MAX_CASE_BYTES)?)?;
    let absent = source_case["source_absent_paths"]
        .as_array()
        .ok_or("retained source_absent_paths")?;
    if proof["source_absent"] != true || absent.is_empty() {
        return Err("retained source-absence proof is incomplete".into());
    }
    for item in absent {
        let path = Path::new(item.as_str().ok_or("absolute source-absence path")?);
        if !path.is_absolute() || path.try_exists()? {
            return Err(format!("source-absence path is not absent: {}", path.display()).into());
        }
    }

    let host: Value = serde_json::from_slice(&host_bytes)?;
    let request: Value = serde_json::from_slice(&request_bytes)?;
    let primary_selection = request["primary"].clone();
    let primary_generation = primary_selection["store_generation_id"]
        .as_str()
        .ok_or("retained old Primary selection")?;
    if !primary_generation
        .ends_with("7c5d291377b9fea707a9dca75324a90d7cb45c7e97fa044a10f80b72a16cad83")
    {
        return Err("retained request is not the authorized old Primary selection".into());
    }
    let platform_host = &host["platform"];
    let platform_request = &request["platform"];
    let selection_bytes = wow_core::canonical_json_bytes(&platform_host["selection"])?;
    let stop = AtomicBool::new(false);
    let selection = PlatformProjectSelection::from_json_slice(&selection_bytes, &stop)?;
    let store_root = Path::new(
        platform_host["project_store_root"]
            .as_str()
            .ok_or("retained platform store root")?,
    );
    let generation = wow_store::project::StoreGenerationId::parse(
        platform_request["store_generation_id"]
            .as_str()
            .ok_or("retained exact platform generation")?,
    )?;
    let store = LiveProjectStore::open_in_namespace(store_root, selection.owner())?;
    let read =
        store.read_in_namespace(selection.owner(), &ReadSelector::Exact(generation), &stop)?;

    let observed = (|| -> TestResult<Value> {
        assert_eq!(
            read.publication_set_id(),
            platform_request["publication_set_id"]
                .as_str()
                .ok_or("platform publication ID")?
        );
        let candidate = read.candidate().ok_or("retained native candidate")?;
        assert_eq!(
            candidate.manifest().candidate_id(),
            platform_request["candidate_id"]
                .as_str()
                .ok_or("candidate ID")?
        );
        assert_eq!(
            candidate.manifest().candidate_digest(),
            platform_request["candidate_digest"]
                .as_str()
                .ok_or("candidate digest")?
        );
        let native = candidate.native();
        let provenance = native.source();
        let graph = candidate.graph();
        let handlers = provenance.inline_handlers();
        let sources = provenance.script_sources();
        let function_records = candidate
            .function_fingerprints()
            .ok_or("native function manifest")?
            .records();
        let graph_nodes = graph.snapshot().nodes();
        let counts = json!({
            "inline_handlers": handlers.len(),
            "script_sources": sources.len(),
            "function_records": function_records.len(),
            "graph_nodes": graph_nodes.len(),
            "inline_body_sources": sources.iter().filter(|row| row.source_kind == XmlScriptSource::InlineBody).count()
        });
        if handlers.len() > MAX_INLINE_HANDLERS
            || sources.len() > MAX_SCRIPT_SOURCES
            || function_records.len() > MAX_FUNCTIONS
            || graph_nodes.len() > MAX_GRAPH_NODES
        {
            return Err(
                format!("retained native population exceeded bound; counts={counts}").into(),
            );
        }

        // This exact old platform root is the independent wrong-kind refusal input.
        let wrong_kind_id =
            "graph-node:sha256:fc81e504e501c80a908f2ef2251b91c19230f136cf9eb7bf65c074ec989abaf7";
        if proof["platform_root"].as_str() != Some(wrong_kind_id)
            || proof["platform_root_kind"] != "lua_source_function"
        {
            return Err("retained callable proof does not name the authorized fc81 root".into());
        }
        let lookup = graph.producer_lookup(&stop).map_err(|error| {
            format!("native producer lookup unavailable; bounded counts={counts}; {error:?}")
        })?;
        let mut chosen = None;
        let mut matched_pairs = 0usize;
        for handler in handlers {
            let mut matching = sources.iter().filter(|source| {
                source.script_id == handler.script_id
                    && source.document == handler.document
                    && source.source_kind == XmlScriptSource::InlineBody
                    && source.source_handle_id == handler.source_handle_id
                    && source.evidence_id == handler.evidence_id
            });
            let Some(script_source) = matching.next() else {
                continue;
            };
            if matching.next().is_some()
                || !handler.semantic_context.admits_static_source_association()
            {
                return Err(
                    "native inline handler/source association is ambiguous or inadmissible".into(),
                );
            }
            matched_pairs += 1;
            if chosen.is_none() {
                chosen = Some((handler, script_source));
            }
        }
        let Some((handler, script_source)) = chosen else {
            return Err(format!("no original native XML handler owner; bounded counts={counts}, matched_inline_pairs={matched_pairs}, accepted_handler_roots=0").into());
        };
        let local = GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: handler.proposal_id.clone().into(),
        };
        let assertion = native.assertion(&local).ok_or_else(|| {
            format!("native handler producer assertion absent; bounded counts={counts}, matched_inline_pairs={matched_pairs}")
        })?;
        let resolved = lookup.entity(native.scope(), assertion, &stop).map_err(|error| {
            format!("no accepted native XML handler owner; bounded counts={counts}, matched_inline_pairs={matched_pairs}; {error:?}")
        })?;
        let accepted = resolved.accepted().node();
        if serde_json::to_value(resolved.reference())? != serde_json::to_value(assertion)?
            || accepted.kind() != "xml_source_handler"
            || resolved.proposal().source_handle_ids()
                != std::slice::from_ref(&handler.source_handle_id)
            || resolved.proposal().evidence_ids() != std::slice::from_ref(&handler.evidence_id)
        {
            return Err(format!("native producer lookup did not accept original XML handler custody; bounded counts={counts}, matched_inline_pairs={matched_pairs}").into());
        }
        let rebound = GraphNode::new(
            graph.snapshot().universe().clone(),
            graph.snapshot().generation().clone(),
            accepted.kind(),
            accepted.owner_key(),
            accepted.evidence_ids().to_vec(),
            graph.snapshot().limits(),
        )?;
        let root_id = serde_json::to_value(rebound.node_id())?;
        let mut platform_input = request["platform"]["input"].clone();
        platform_input["neighborhood"]["query"]["node_id"] = root_id.clone();
        let input = OpenSkeletonInputRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&platform_input)?,
            &stop,
        )?;
        let view = open_skeleton_input_view(native, graph, input, &stop)?;
        let page = view.read(&stop)?;
        let root = page.neighborhood().root();
        if root.kind() != "xml_source_handler" || root.node_id() != rebound.node_id() {
            return Err(
                "held Skeleton page did not retain the selected native XML handler root".into(),
            );
        }
        let final_member = graph
            .snapshot()
            .node(rebound.node_id())
            .ok_or("final accepted handler root")?;
        if !std::ptr::eq(final_member, root) {
            return Err("selected handler root is not the exact final graph member".into());
        }
        let source_handle = provenance
            .source_handles()
            .get(&handler.source_handle_id)
            .ok_or("original handler source handle")?;
        let evidence = provenance
            .evidence()
            .get(&handler.evidence_id)
            .ok_or("original handler evidence")?;
        let metadata = json!({
            "selection": "xml_handler_metadata",
            "interpretation": INTERPRETATION,
            "source_context": provenance.context(),
            "assertion": assertion,
            "root": final_member,
            "handler": handler,
            "script_source": script_source,
            "source_handle": source_handle,
            "evidence": evidence,
            "root_support": page.root_support()
        });
        let old_function_root =
            serde_json::from_value::<wow_graph::GraphNodeId>(proof["platform_root"].clone())?;
        let function_root = graph
            .snapshot()
            .node(&old_function_root)
            .ok_or("original callable baseline root")?;
        if function_root.kind() != "lua_source_function" {
            return Err("original callable baseline root kind changed".into());
        }
        let old_wrong_kind = json!({
            "id": proof["platform_root"],
            "graph_kind": function_root.kind()
        });

        let mut selected_request = request.clone();
        selected_request["platform"]["input"]["neighborhood"]["query"]["node_id"] = root_id.clone();
        Ok(json!({
            "schema": "wow-service/platform-xml-handler-preparation/1",
            "status": "prepared",
            "counts": {
                "native": counts,
                "matched_inline_pairs": matched_pairs,
                "accepted_handler_roots": 1,
                "final_root_members": 1
            },
            "primary_selection": primary_selection,
            "platform_root_id": root_id,
            "original": metadata,
            "selected_inspect_request": selected_request,
            "wrong_kind_root": old_wrong_kind,
            "source_absence": {
                "status": "verified_absent_paths",
                "source_case": proof["source_case"],
                "absent_path_count": absent.len()
            },
            "input_sha256": {
                "host": format!("sha256:{:x}", sha2::Sha256::digest(&host_bytes)),
                "request": format!("sha256:{:x}", sha2::Sha256::digest(&request_bytes))
            }
        }))
    })();
    let read_close = read.close();
    let store_close = store.close();
    read_close?;
    store_close?;
    let observed = observed?;
    if observed["status"] != "prepared" {
        return Err("native XML-handler preparation did not produce a root".into());
    }
    let work = repo.join(WORK_RELATIVE);
    let export = wow_core::canonical_json_bytes(&observed)?;
    write_new(&work.join("prepared-xml-handler-root.json"), &export)?;
    write_new(
        &work.join("xml-handler-root-inspect-request.json"),
        &wow_core::canonical_json_bytes(&observed["selected_inspect_request"])?,
    )?;
    println!(
        "prepared native XML-handler root: {}",
        work.join("prepared-xml-handler-root.json").display()
    );
    Ok(())
}

const CALLABLE_EXPORT_RELATIVE: &str =
    ".codex/work/platform-callable-witness-service-export-20261012";
const MAX_CALLABLE_EXPECTED_BYTES: usize = 2 * 1024 * 1024;
const MAX_SELECTED_EXPECTED_BYTES: usize = 4 * 1024 * 1024;

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_new_capped(path: &Path, bytes: &[u8], limit: usize) -> TestResult {
    if bytes.len() > limit {
        return Err(format!("witness export exceeds {limit} byte cap").into());
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn require_closed(result: &Value) -> TestResult {
    assert_eq!(result["resource_closure"], "closed");
    Ok(())
}

fn require_no_payload(result: &Value) -> TestResult {
    assert!(result.get("payload").is_none_or(Value::is_null));
    Ok(())
}

struct CombinedObservation {
    envelope: Value,
    bytes: Vec<u8>,
    failure_code: Option<ServiceErrorCode>,
    decode_ms: u128,
    service_ms: u128,
}

fn run_combined_service(
    host: &crate::context::ContextCombinedHostInput,
    wire: &Value,
    stop: &AtomicBool,
) -> TestResult<CombinedObservation> {
    let decode_started = Instant::now();
    let request = crate::context::ContextCombinedRequest::from_json_slice(
        &wow_core::canonical_json_bytes(wire)?,
        stop,
    )?;
    let decode_ms = decode_started.elapsed().as_millis();
    let service_started = Instant::now();
    let result = crate::context::execute_context_combined(host, &request, stop)?;
    let service_ms = service_started.elapsed().as_millis();
    let failure_code = result.failure_code();
    let raw = result.canonical_bytes().to_vec();
    eprintln!(
        "combined witness phase: decode_ms={decode_ms}, service_ms={service_ms}, bytes={}, failure={failure_code:?}",
        raw.len()
    );
    if raw.len() > MAX_SELECTED_EXPECTED_BYTES {
        return Err("combined result exceeded the selected witness output cap".into());
    }
    let envelope = serde_json::from_slice(&raw)?;
    Ok(CombinedObservation {
        envelope,
        bytes: raw,
        failure_code,
        decode_ms,
        service_ms,
    })
}

#[test]
#[ignore = "selected XML-handler consumer witness; root runs after product integration"]
fn selected_xml_handler_consumer_lifecycle() -> TestResult {
    const MAX_OLD_STDOUT_BYTES: usize = 512 * 1024;
    const MAX_WITNESS_PROOF_BYTES: usize = 64 * 1024;

    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let work = repo.join(WORK_RELATIVE);
    let baseline = repo.join(BASELINE_RELATIVE);
    let callable_export = repo.join(CALLABLE_EXPORT_RELATIVE);
    let prepared_bytes = bounded_bytes(
        &work.join("prepared-xml-handler-root.json"),
        MAX_EXPORT_BYTES,
    )?;
    let prepared: Value = serde_json::from_slice(&prepared_bytes)?;
    let request_bytes = bounded_bytes(
        &work.join("xml-handler-root-inspect-request.json"),
        MAX_REQUEST_BYTES,
    )?;
    let old_request: Value = serde_json::from_slice(&request_bytes)?;
    let source_case_path = Path::new(
        prepared["source_absence"]["source_case"]
            .as_str()
            .ok_or("prepared source-absence case path")?,
    );
    if !source_case_path.is_absolute() {
        return Err("prepared source-absence case path must be absolute".into());
    }
    let source_case: Value =
        serde_json::from_slice(&bounded_bytes(source_case_path, MAX_CASE_BYTES)?)?;
    let absent_paths = source_case["source_absent_paths"]
        .as_array()
        .ok_or("prepared source_absent_paths")?;
    if absent_paths.is_empty()
        || prepared["source_absence"]["status"] != "verified_absent_paths"
        || prepared["source_absence"]["absent_path_count"] != absent_paths.len()
    {
        return Err("prepared source-absence evidence is incomplete or inconsistent".into());
    }
    for item in absent_paths {
        let path = Path::new(item.as_str().ok_or("absolute source-absence path")?);
        if !path.is_absolute() || path.try_exists()? {
            return Err(format!("source-absence path is not absent: {}", path.display()).into());
        }
    }
    let expected_old_stdout = bounded_bytes(
        &work.join("handler-root-inspect-stdout.json"),
        MAX_OLD_STDOUT_BYTES,
    )?;
    let old_proof: Value = serde_json::from_slice(&bounded_bytes(
        &work.join("handler-root-old-cli-proof.json"),
        MAX_PROOF_BYTES,
    )?)?;
    if old_proof["base"] != "f8e978c274ff33444c79516765a1041d9e013182"
        || old_proof["closure"] != "closed"
        || old_proof["stderr_bytes"] != 0
        || old_proof["stdout_bytes"] != expected_old_stdout.len()
        || old_proof["stdout_sha256"] != digest_hex(&expected_old_stdout)
        || old_proof["baseline_request_sha256"] != digest_hex(&request_bytes)
        || old_proof["stdout_sha256"]
            != "fe18e67c73e55c51c6daade5b7c8c32a3a3d8b20990f0cc4e8259681353eea13"
        || old_proof["baseline_request_sha256"]
            != "fa9d8ec6693149f05071a21fa3a4e4c395462c3e2261a8b3a22adc25c56c8044"
    {
        return Err(
            "retained old CLI baseline/proof changed; refusing to bless replacement bytes".into(),
        );
    }
    if expected_old_stdout.last() != Some(&b'\n')
        || expected_old_stdout.ends_with(b"\r\n")
        || expected_old_stdout
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count()
            != 1
    {
        return Err("old CLI baseline must retain exactly one LF and no CRLF framing".into());
    }
    let old_cli_canonical = &expected_old_stdout[..expected_old_stdout.len() - 1];
    let old_envelope: Value = serde_json::from_slice(old_cli_canonical)?;
    assert_eq!(old_envelope["request"]["primary"], old_request["primary"]);
    assert_eq!(prepared["status"], "prepared");
    assert_eq!(prepared["selected_inspect_request"], old_request);
    assert_eq!(prepared["primary_selection"], old_request["primary"]);
    assert_eq!(
        prepared["source_absence"]["status"],
        "verified_absent_paths"
    );
    assert_eq!(prepared["original"]["selection"], "xml_handler_metadata");
    assert_eq!(prepared["original"]["interpretation"], INTERPRETATION);
    assert_eq!(prepared["original"]["root"]["kind"], "xml_source_handler");
    assert_eq!(
        prepared["wrong_kind_root"]["graph_kind"],
        "lua_source_function"
    );
    let host_bytes = bounded_bytes(&baseline.join("host.json"), MAX_HOST_BYTES)?;
    let host: crate::context::ContextCombinedHostInput =
        crate::context::ContextCombinedHostInput::from_json_slice(
            &host_bytes,
            &AtomicBool::new(false),
        )?;

    let callable_host_bytes = bounded_bytes(&callable_export.join("host.json"), MAX_HOST_BYTES)?;
    let callable_request_bytes = bounded_bytes(
        &callable_export.join("build-request.json"),
        MAX_REQUEST_BYTES,
    )?;
    let callable_expected = bounded_bytes(
        &callable_export.join("expected-build-envelope.json"),
        MAX_CALLABLE_EXPECTED_BYTES,
    )?;
    let callable_proof: Value = serde_json::from_slice(&bounded_bytes(
        &callable_export.join("proof.json"),
        MAX_WITNESS_PROOF_BYTES,
    )?)?;
    let callable_request: Value = serde_json::from_slice(&callable_request_bytes)?;
    let callable_expected_value: Value = serde_json::from_slice(&callable_expected)?;
    if callable_proof["default_bytes_equal_excluding_one_cli_lf"] != true
        || callable_proof["independent_native_read_closed"] != true
        || callable_request["operation"] != "build"
        || callable_request["build"]["platform_root_facet"] != "callable_metadata"
        || callable_expected.last() == Some(&b'\n')
        || callable_expected_value["resource_closure"] != "closed"
        || callable_request["primary"] != old_request["primary"]
    {
        return Err(
            "selected callable compatibility inputs are not the exact retained service export"
                .into(),
        );
    }
    let callable_host: crate::context::ContextCombinedHostInput =
        crate::context::ContextCombinedHostInput::from_json_slice(
            &callable_host_bytes,
            &AtomicBool::new(false),
        )?;

    let new_outputs = [
        "xml-handler-build-host.json",
        "xml-handler-build-request.json",
        "xml-handler-build-expected-service.json",
        "xml-handler-consumer-witness-proof.json",
    ];
    for name in new_outputs {
        if work.join(name).try_exists()? {
            return Err(format!("refusing to overwrite existing witness output: {name}").into());
        }
    }

    // The retained native calls are deliberately sequential.
    let stop = AtomicBool::new(false);
    let CombinedObservation {
        envelope: omitted,
        bytes: omitted_bytes,
        failure_code: omitted_failure,
        decode_ms: omitted_decode_ms,
        service_ms: omitted_service_ms,
    } = run_combined_service(&host, &old_request, &stop)?;
    require_closed(&omitted)?;
    assert_eq!(omitted_failure, None);
    assert_eq!(
        omitted_bytes.as_slice(),
        old_cli_canonical,
        "omitted request bytes changed"
    );

    let mut selected_inspect_request = old_request.clone();
    selected_inspect_request["inspection"]["platform_root_facet"] = json!("xml_handler_metadata");
    let CombinedObservation {
        envelope: selected_inspect,
        bytes: selected_inspect_bytes,
        failure_code: selected_inspect_failure,
        decode_ms: inspect_decode_ms,
        service_ms: inspect_service_ms,
    } = run_combined_service(&host, &selected_inspect_request, &stop)?;
    require_closed(&selected_inspect)?;
    assert_eq!(selected_inspect_failure, None);
    assert_eq!(
        selected_inspect["payload"]["combined_inspect"]["core"]["platform_root_xml_handler"],
        prepared["original"],
        "selected facet must equal independent original native evidence"
    );
    let inspect_core = &selected_inspect["payload"]["combined_inspect"]["core"];
    assert_eq!(selected_inspect["status"], old_envelope["status"]);
    assert_eq!(
        inspect_core["map"],
        omitted["payload"]["combined_inspect"]["core"]["map"]
    );
    assert_eq!(
        inspect_core["primary_skeletons"],
        omitted["payload"]["combined_inspect"]["core"]["primary_skeletons"]
    );
    assert_eq!(
        inspect_core["platform_inspect"]["map"],
        omitted["payload"]["combined_inspect"]["core"]["platform_inspect"]["map"]
    );
    assert_eq!(inspect_core["cross_universe_relations"], "not_evaluated");
    assert_eq!(inspect_core["absence_authoritative"], false);
    let native_page = &inspect_core["platform_inspect"]["map"]["core"]["native"]["body"];
    assert_eq!(
        native_page["request"]["neighborhood"]["snapshot_id"],
        old_request["platform"]["input"]["neighborhood"]["snapshot_id"]
    );
    assert_eq!(
        native_page["neighborhood"]["root"]["node_id"],
        prepared["platform_root_id"]
    );
    assert_eq!(
        native_page["neighborhood"]["root"]["kind"],
        "xml_source_handler"
    );
    assert!(inspect_core["map"]["combined_map_id"].is_string());
    assert!(inspect_core["platform_inspect"]["map"]["map_id"].is_string());

    let mut build_none_request = selected_inspect_request.clone();
    build_none_request
        .as_object_mut()
        .ok_or("combined request object")?
        .remove("inspection");
    build_none_request["operation"] = json!("build");
    build_none_request["build"] = json!({
        "context_profile_id": "wow-context/combined-native-structural-pack/1",
        "level": old_request["inspection"]["level"],
        "platform_root_facet": "xml_handler_metadata",
        "renderer_profile_ids": []
    });
    let CombinedObservation {
        envelope: build_none,
        bytes: build_none_bytes,
        failure_code: build_none_failure,
        decode_ms: none_decode_ms,
        service_ms: none_service_ms,
    } = run_combined_service(&host, &build_none_request, &stop)?;
    require_closed(&build_none)?;
    assert_eq!(build_none_failure, None);
    assert_eq!(build_none["status"], old_envelope["status"]);
    assert_eq!(build_none["payload"]["rendered_artifacts"], json!([]));
    assert!(build_none["payload"]["semantic_pack"].is_object());

    let mut build_both_request = build_none_request.clone();
    build_both_request["build"]["renderer_profile_ids"] = json!([
        "wow-context/combined-pack-json/1",
        "wow-context/combined-pack-markdown/1"
    ]);
    let CombinedObservation {
        envelope: build_both,
        bytes: build_both_bytes,
        failure_code: build_both_failure,
        decode_ms: both_decode_ms,
        service_ms: both_service_ms,
    } = run_combined_service(&host, &build_both_request, &stop)?;
    require_closed(&build_both)?;
    assert_eq!(build_both_failure, None);
    assert_eq!(build_both["status"], old_envelope["status"]);
    assert_eq!(
        build_both["payload"]["rendered_artifacts"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    let none_pack = &build_none["payload"]["semantic_pack"];
    let both_pack = &build_both["payload"]["semantic_pack"];
    assert_eq!(
        none_pack, both_pack,
        "one renderer-independent semantic pack"
    );
    let origins = &both_pack["core"]["origins"];
    let selected_combined = &selected_inspect["payload"]["combined_inspect"];
    assert_eq!(origins, selected_combined, "full original Inspect custody");
    let canonical_pack = wow_core::canonical_json_bytes(both_pack)?;
    let profiles: Value = serde_json::from_slice(wow_context::COMBINED_CONTEXT_PROFILES_BYTES)?;
    let renderers = build_both["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("both renderers")?;
    for (index, renderer) in renderers.iter().enumerate() {
        let row = &renderer["core"];
        assert_eq!(row["semantic_pack_id"], both_pack["semantic_pack_id"]);
        assert_eq!(
            row["profile"],
            profiles[if index == 0 { "json" } else { "markdown" }]
        );
        let content = row["content"].as_str().ok_or("renderer UTF8 content")?;
        assert_eq!(row["byte_length"], content.len());
        assert_eq!(row["content_digest"], digest(content.as_bytes()));
        if index == 0 {
            assert_eq!(content.as_bytes(), canonical_pack.as_slice());
        } else {
            let encoded = content
                .lines()
                .find_map(|line| line.strip_prefix("ORIGINS "))
                .ok_or("Markdown original origins")?;
            let quoted: String = serde_json::from_str(encoded)?;
            assert_eq!(serde_json::from_str::<Value>(&quoted)?, *selected_combined);
            let trace = row["trace"]
                .as_array()
                .ok_or("Markdown trace")?
                .iter()
                .find(|trace| trace["template_id"] == "combined_native_origins")
                .ok_or("original origins trace")?;
            assert_eq!(trace["record_id"], selected_combined["inspect_id"]);
            assert_eq!(trace["semantic_fields"], json!(["core.origins"]));
        }
    }
    let host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let usage = &build_both["payload"]["operation_budget_usage"];
    for (field, limit) in [
        ("work_reserved", "max_context_work"),
        ("bytes_reserved", "max_metadata_bytes"),
    ] {
        let total = usage[field].as_u64().ok_or("continuing operation usage")?;
        assert!(
            total
                > build_none["payload"]["operation_budget_usage"][field]
                    .as_u64()
                    .ok_or("None usage")?
        );
        assert!(
            total
                >= both_pack["core"]["budget"][field]
                    .as_u64()
                    .ok_or("pack reservation")?
        );
        assert!(
            total
                <= host_wire["primary"]["limits"][limit]
                    .as_u64()
                    .ok_or("original hard cap")?
        );
    }
    assert!(
        usage["bytes_reserved"].as_u64().ok_or("bytes")? + u64::try_from(build_both_bytes.len())?
            <= host_wire["primary"]["limits"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("metadata cap")?
    );

    let mut wrong_kind_request = selected_inspect_request.clone();
    wrong_kind_request["platform"]["input"]["neighborhood"]["query"]["node_id"] =
        prepared["wrong_kind_root"]["id"].clone();
    let CombinedObservation {
        envelope: wrong_kind,
        bytes: wrong_kind_bytes,
        failure_code: wrong_kind_failure,
        decode_ms: wrong_decode_ms,
        service_ms: wrong_service_ms,
    } = run_combined_service(&host, &wrong_kind_request, &stop)?;
    require_closed(&wrong_kind)?;
    assert_eq!(wrong_kind_failure, Some(ServiceErrorCode::InvalidRequest));
    assert_eq!(wrong_kind["failure_code"], "invalid_request");
    require_no_payload(&wrong_kind)?;

    let mut bounded_host = host_wire.clone();
    for owner in ["primary", "platform"] {
        bounded_host[owner]["limits"]["max_context_work"] = json!(1);
    }
    let bounded_host = crate::context::ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&bounded_host)?,
        &stop,
    )?;
    let CombinedObservation {
        envelope: work_cap,
        bytes: work_cap_bytes,
        failure_code: work_cap_failure,
        decode_ms: cap_decode_ms,
        service_ms: cap_service_ms,
    } = run_combined_service(&bounded_host, &selected_inspect_request, &stop)?;
    require_closed(&work_cap)?;
    assert_eq!(work_cap_failure, Some(ServiceErrorCode::BudgetExceeded));
    assert_eq!(work_cap["failure_code"], "budget_exceeded");
    require_no_payload(&work_cap)?;

    let CombinedObservation {
        envelope: callable_compat,
        bytes: callable_compat_bytes,
        failure_code: callable_compat_failure,
        decode_ms: compat_decode_ms,
        service_ms: compat_service_ms,
    } = run_combined_service(&callable_host, &callable_request, &stop)?;
    require_closed(&callable_compat)?;
    assert_eq!(callable_compat_failure, None);
    assert_eq!(
        callable_compat_bytes, callable_expected,
        "existing callable-selected bytes/counters changed"
    );

    if build_none_bytes.last() == Some(&b'\n')
        || build_none_bytes.len() > MAX_SELECTED_EXPECTED_BYTES
    {
        return Err(
            "selected CLI Build expectation must be bounded canonical JSON without LF".into(),
        );
    }
    let build_request_bytes = wow_core::canonical_json_bytes(&build_both_request)?;
    if build_both_bytes.last() == Some(&b'\n')
        || build_both_bytes.len() > MAX_SELECTED_EXPECTED_BYTES
        || build_request_bytes.len() > MAX_REQUEST_BYTES
    {
        return Err("both-renderer CLI Build case exceeded its byte/framing bounds".into());
    }
    let proof = json!({
        "schema": "wow-service/platform-xml-handler-consumer-witness/1",
        "base": "f8e978c274ff33444c79516765a1041d9e013182",
        "native_store_calls_sequential": true,
        "calls": 7,
        "old_cli_baseline": {
            "unchanged": true,
            "stdout_sha256": old_proof["stdout_sha256"],
            "stdout_bytes_with_one_lf": expected_old_stdout.len(),
            "canonical_bytes_without_lf": old_cli_canonical.len()
        },
        "selected_build_export": {
            "request_sha256": digest(&build_request_bytes),
            "expected_service_sha256": digest(&build_both_bytes),
            "expected_service_bytes_without_lf": build_both_bytes.len(),
            "status": build_both["status"],
            "semantic_pack_id": build_none["payload"]["semantic_pack"]["semantic_pack_id"],
            "renderer_profile_ids": build_both_request["build"]["renderer_profile_ids"],
            "semantic_pack_id_with_renderers": build_both["payload"]["semantic_pack"]["semantic_pack_id"],
            "semantic_pack_bytes": build_both["payload"]["semantic_pack_byte_length"],
            "operation_budget_usage": build_both["payload"]["operation_budget_usage"],
            "rendered_artifacts": build_both["payload"]["rendered_artifacts"].as_array().map(Vec::len)
        },
        "callable_compatibility": {
            "source_expected_sha256": digest(&callable_expected),
            "observed_sha256": digest(&callable_compat_bytes),
            "bytes_and_counters_unchanged": true,
            "expected_bytes": callable_expected.len()
        },
        "refusals": {
            "wrong_kind": {"failure_code": wrong_kind["failure_code"], "resource_closure": wrong_kind["resource_closure"], "payload": "absent"},
            "work_cap_1": {"failure_code": work_cap["failure_code"], "resource_closure": work_cap["resource_closure"], "payload": "absent"}
        },
        "phase_ms": {
            "omitted": {"decode": omitted_decode_ms, "service": omitted_service_ms},
            "selected_inspect": {"decode": inspect_decode_ms, "service": inspect_service_ms},
            "build_none": {"decode": none_decode_ms, "service": none_service_ms},
            "build_both_renderers": {"decode": both_decode_ms, "service": both_service_ms},
            "wrong_kind": {"decode": wrong_decode_ms, "service": wrong_service_ms},
            "work_cap_1": {"decode": cap_decode_ms, "service": cap_service_ms},
            "callable_compatibility": {"decode": compat_decode_ms, "service": compat_service_ms}
        },
        "result_bytes": {
            "omitted": omitted_bytes.len(),
            "selected_inspect": selected_inspect_bytes.len(),
            "build_none": build_none_bytes.len(),
            "build_both_renderers": build_both_bytes.len(),
            "wrong_kind": wrong_kind_bytes.len(),
            "work_cap_1": work_cap_bytes.len(),
            "callable_compatibility": callable_compat_bytes.len()
        }
    });
    let build_host_path = work.join("xml-handler-build-host.json");
    let build_request_path = work.join("xml-handler-build-request.json");
    let build_expected_path = work.join("xml-handler-build-expected-service.json");
    let proof_path = work.join("xml-handler-consumer-witness-proof.json");
    write_new_capped(&build_host_path, &host_bytes, MAX_HOST_BYTES)?;
    write_new_capped(&build_request_path, &build_request_bytes, MAX_REQUEST_BYTES)?;
    write_new_capped(
        &build_expected_path,
        &build_both_bytes,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    write_new_capped(
        &proof_path,
        &wow_core::canonical_json_bytes(&proof)?,
        MAX_WITNESS_PROOF_BYTES,
    )?;
    println!(
        "selected XML-handler service witness exports: {}",
        work.display()
    );
    Ok(())
}

mod combined_recipe_composition;
mod xml_declaration;
mod xml_reference;
