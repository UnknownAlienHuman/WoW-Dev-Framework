//! One synthetic retained native pack/render lifecycle; no live/Ketho authority.
use super::*;
use crate::context::build::CONTEXT_BUILD_RESULT_SCHEMA;
use std::collections::BTreeMap;
mod external_primary_pack_witness;

const ADMITTED_PATH: &str = "main [note](x)#&`tick.lua";
const FIXTURE_WORK: u64 = 8_000_000;
const FIXTURE_METADATA: u64 = 4 * 1024 * 1024;

fn rows(value: &Value) -> TestResult<&[Value]> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| "expected array".into())
}

fn exact_rows(actual: &Value, expected: &[Value]) -> TestResult {
    let actual = rows(actual)?;
    assert_eq!(actual.len(), expected.len());
    for row in expected {
        assert_eq!(actual.iter().filter(|other| *other == row).count(), 1);
    }
    Ok(())
}

fn retained_build(
    root: &Path,
    generation: &wow_store::project::StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<Value> {
    let mut expected = retained_project_expectation(root, generation, stop)?;
    let store = LiveProjectStore::open(root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
    let proof = read.pair().graph_read(stop)?;
    expected["project_coverage"] = serde_json::to_value(read.project().project_coverage_records())?;
    let provenance = read.pair().source_provenance().ok_or("native provenance")?;
    let mut callables = BTreeMap::new();
    for function in provenance.functions() {
        let node = proof.published_entity(SOURCE_GRAPH_PARTITION, &function.proposal_id, stop)?;
        let metadata = provenance
            .callable_metadata(&function.function_id)?
            .ok_or("retained original callable metadata")?;
        assert!(
            callables
                .insert(node.node_id().to_string(), serde_json::to_value(metadata)?)
                .is_none()
        );
    }
    assert_eq!(callables.len(), 4, "two genuine Chunks and two Closures");
    expected["callables"] = serde_json::to_value(callables)?;
    read.close()?;
    store.close()?;
    Ok(expected)
}

fn request(
    generation: &wow_store::project::StoreGenerationId,
    root: &GraphNodeId,
    render: ContextRenderSelection,
    profile: &ProfileIdentity,
    reference: &ReferenceView,
) -> TestResult<ContextBuildRequest> {
    Ok(ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(generation.as_str())?,
        ContextInspectLevel::Both,
        ContextMapRoot::graph_entity(root.as_str())?,
        render,
        None,
        Some(reference_guard(profile, reference)?),
    )?)
}

fn assert_support(
    origins: &Value,
    expected: &Value,
    nodes: &[Value],
    edges: &[Value],
) -> TestResult {
    let core = &origins["core"];
    let mut pending = BTreeSet::new();
    for (kind, records, field) in [("entity", nodes, "node_id"), ("relation", edges, "edge_id")] {
        for record in records {
            assert!(pending.insert((
                kind.to_string(),
                record[field].as_str().ok_or("native ID")?.to_string()
            )));
        }
    }
    assert_eq!(rows(&core["supports"])?.len(), pending.len());
    for receipt in rows(&core["supports"])? {
        let explanation = &receipt["explanation"];
        assert_eq!(
            explanation["source_context_id"],
            expected["source_context_id"]
        );
        assert_eq!(explanation["support_complete"], true);
        assert!(rows(&explanation["truncations"])?.is_empty());
        let record = &explanation["record"];
        let (kind, id) = match record["kind"].as_str() {
            Some("entity") => {
                assert!(nodes.contains(&record["node"]));
                (
                    "entity",
                    record["node"]["node_id"].as_str().ok_or("explained node")?,
                )
            }
            Some("relation") => {
                assert!(edges.contains(&record["edge"]));
                (
                    "relation",
                    record["edge"]["edge_id"].as_str().ok_or("explained edge")?,
                )
            }
            _ => return Err("unexpected native record".into()),
        };
        assert!(pending.remove(&(kind.to_string(), id.to_string())));
        let supports = rows(&explanation["supports"])?;
        assert!(!supports.is_empty());
        for support in supports {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("unexpected foundation support".into()),
            };
            assert!(rows(&expected[proposals])?.contains(&support["proposal"]));
            assert!(rows(&expected[accepted])?.contains(&support["accepted"]));
        }
        let resolution = &receipt["evidence_resolution"];
        assert_eq!(resolution["closure_complete"], true);
        assert!(rows(&resolution["unresolved"])?.is_empty());
        assert!(rows(&resolution["truncations"])?.is_empty());
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            let records = rows(&resolution[actual])?;
            assert!(!records.is_empty());
            for row in records {
                assert!(rows(&expected[retained])?.contains(row));
            }
        }
    }
    assert!(
        pending.is_empty(),
        "all selected native records have exact support"
    );
    for custody in rows(&core["source_custody"])? {
        assert_eq!(custody["scope"], "primary");
        assert!(rows(&expected["handles"])?.contains(&custody["handle"]));
    }
    Ok(())
}

fn assert_pack(
    result: &ContextMapResult,
    expected: &Value,
    selected: &Value,
    scratch: &Scratch,
    state: ContextMapStatus,
) -> TestResult<Value> {
    assert_eq!(result.status(), state);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], CONTEXT_BUILD_RESULT_SCHEMA);
    assert_eq!(envelope["operation"], "context_build");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    for field in ["publication_set_id", "store_generation_id"] {
        assert_eq!(payload[field], expected[field]);
    }
    assert_eq!(payload["current_at_acquisition"], expected["current"]);
    let current: CurrentPublication = serde_json::from_value(expected["current"].clone())?;
    assert_eq!(
        payload["current_record_digest"],
        observed_current_digest(&current)?
    );
    assert_eq!(payload["configured_registration_id"], STORE_ID);
    assert!(payload.get("native_project_store_id").is_none());
    assert_eq!(
        payload["validation"],
        "exact_native_binding_and_canonical_bytes"
    );

    let pack = &payload["semantic_pack"];
    let canonical = wow_core::canonical_json_bytes(pack)?;
    assert_eq!(payload["semantic_pack_byte_length"], canonical.len());
    let core = &pack["core"];
    assert_eq!(core["schema"], "wow-context/semantic-pack/primary/1");
    assert_eq!(
        core["profile"]["profile_id"],
        PRIMARY_CONTEXT_PACK_PROFILE_ID
    );
    assert_eq!(core["profile"]["layer_order"], json!(["l0", "l1"]));
    assert_eq!(core["profile"]["source_bodies"], "excluded");
    assert_eq!(core["profile"]["tokens"], "unavailable");
    assert_eq!(core["state"], serde_json::to_value(state)?);
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(
        core["budget"]["reservation_basis"],
        "before_semantic_pack_finalization"
    );
    assert_eq!(
        payload["operation_budget_usage"]["reservation_basis"],
        "before_service_delivery"
    );
    assert!(
        payload["operation_budget_usage"]["work_reserved"]
            .as_u64()
            .ok_or("work")?
            >= core["budget"]["work_reserved"]
                .as_u64()
                .ok_or("pack work")?
    );
    assert!(
        payload["operation_budget_usage"]["bytes_reserved"]
            .as_u64()
            .ok_or("bytes")?
            >= core["budget"]["bytes_reserved"]
                .as_u64()
                .ok_or("pack bytes")?
    );
    assert!(
        payload["operation_budget_usage"]["work_reserved"]
            .as_u64()
            .ok_or("work")?
            <= core["budget"]["max_context_work"]
                .as_u64()
                .ok_or("work limit")?
    );
    assert!(
        payload["operation_budget_usage"]["bytes_reserved"]
            .as_u64()
            .ok_or("bytes")?
            <= core["budget"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("byte limit")?
    );

    let skeletons = &core["skeletons"];
    assert_eq!(skeletons["level"], "both");
    let l0 = &skeletons["l0"]["core"];
    let l1 = &skeletons["l1"]["core"];
    let root = &selected["node"];
    assert_eq!(l0["scope"], json!({"kind":"entity","node":root}));
    assert_eq!(l1["root"], *root);
    let edges = rows(&l1["direct_relations"])?;
    assert_eq!(
        edges.len(),
        if state == ContextMapStatus::Truncated {
            1
        } else {
            2
        }
    );
    for edge in edges {
        assert!(rows(&selected["edges"])?.contains(edge));
    }
    exact_rows(&l0["direct_relations"], edges)?;
    let mut ids = BTreeSet::from([root["node_id"].as_str().ok_or("root ID")?]);
    for edge in edges {
        ids.insert(edge["from"].as_str().ok_or("edge source")?);
        ids.insert(edge["to"].as_str().ok_or("edge target")?);
    }
    let nodes = rows(&expected["nodes"])?
        .iter()
        .filter(|node| node["node_id"].as_str().is_some_and(|id| ids.contains(id)))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), edges.len() + 1);
    let members = nodes
        .iter()
        .filter(|node| *node != root)
        .cloned()
        .collect::<Vec<_>>();
    exact_rows(&l1["local_entities"], &nodes)?;
    exact_rows(&l0["direct_members"], &members)?;
    assert_eq!(l0["selected_member_count"], members.len());
    exact_rows(
        &l0["detail_routes"],
        &members
            .iter()
            .map(|node| node["node_id"].clone())
            .collect::<Vec<_>>(),
    )?;
    assert_eq!(l0["context"]["origins"], l1["context"]["origins"]);
    let origins = &l0["context"]["origins"];
    assert_eq!(
        origins["core"]["project_coverage"],
        expected["project_coverage"]
    );
    assert_support(origins, expected, &nodes, edges)?;
    let navigation = &origins["core"]["navigation"];
    assert_eq!(navigation["root"], *root);
    assert_eq!(navigation["matching_edges"], 2);
    assert_eq!(navigation["omitted_edges"], 2 - edges.len());
    exact_rows(&navigation["edges"], edges)?;
    for skeleton in [l0, l1] {
        let context = &skeleton["context"];
        assert_eq!(context["origin_map_id"], origins["project_map_id"]);
        assert_eq!(context["universe_set_id"], core["universe_set_id"]);
        assert_eq!(context["state"], core["state"]);
        assert_eq!(context["absence_authoritative"], false);
        assert_eq!(context["source_bodies"], "excluded");
        assert_eq!(rows(&skeleton["declarations"])?.len(), members.len());
        for declaration in rows(&skeleton["declarations"])? {
            assert_eq!(declaration["kind"], "callable");
            let id = declaration["node"]["node_id"]
                .as_str()
                .ok_or("callable ID")?;
            assert!(members.contains(&declaration["node"]));
            assert_eq!(declaration["metadata"], expected["callables"][id]);
            for facet in ["signature", "declaration"] {
                assert_eq!(declaration["metadata"][facet]["state"], "not_selected");
            }
        }
    }

    let items = rows(&core["items"])?;
    assert_eq!(items.len(), nodes.len() + edges.len());
    let trace = rows(&core["selection_trace"])?;
    assert_eq!(trace.len(), items.len());
    let mut selected_records = BTreeSet::new();
    for (item, selection) in items.iter().zip(trace) {
        let item_core = &item["core"];
        assert_eq!(
            item_core["root"],
            json!({"kind":"entity","id":root["node_id"]})
        );
        assert_eq!(item_core["origin_map_id"], origins["project_map_id"]);
        assert_eq!(item_core["universe_set_id"], core["universe_set_id"]);
        assert_eq!(item_core["privacy"], "local_metadata_only");
        assert_eq!(item_core["license"], "source_inclusion_not_permitted");
        assert_eq!(item_core["source_boundary"], "untrusted_native_metadata");
        let payload = &item_core["payload"];
        let (kind, id, reason) = match payload["kind"].as_str() {
            Some("entity") => {
                assert!(nodes.contains(&payload["node"]));
                let reason = if payload["node"] == *root {
                    "exact_native_root"
                } else {
                    "selected_direct_native_neighbor"
                };
                (
                    "entity",
                    payload["node"]["node_id"].as_str().ok_or("item node")?,
                    reason,
                )
            }
            Some("relation") => {
                assert!(edges.contains(&payload["edge"]));
                (
                    "relation",
                    payload["edge"]["edge_id"].as_str().ok_or("item edge")?,
                    "original_selected_direct_native_assertion",
                )
            }
            _ => return Err("unexpected context item payload".into()),
        };
        assert!(selected_records.insert((kind, id)));
        assert_eq!(item_core["inclusion_reason"], reason);
        assert_eq!(
            *selection,
            json!({
                "item_id": item["context_item_id"],
                "origin_map_id": origins["project_map_id"],
                "reason": reason,
                "required": true
            })
        );
    }
    assert_eq!(selected_records.len(), nodes.len() + edges.len());
    assert_eq!(
        core["omissions"]["upstream"],
        json!([
            {"producer":"project_map","artifact_id":origins["project_map_id"]},
            {"producer":"l0_skeleton","artifact_id":skeletons["l0"]["l0_skeleton_id"]},
            {"producer":"l1_skeleton","artifact_id":skeletons["l1"]["l1_skeleton_id"]}
        ])
    );
    let text = std::str::from_utf8(result.canonical_bytes())?;
    for excluded in [
        "library/Core.lua",
        "function External",
        "project_store_root",
    ] {
        assert!(
            !text.contains(excluded),
            "excluded source/host data: {excluded}"
        );
    }
    // Owner-wide coverage retains the sibling's partition IDs. It is metadata
    // about the exact input universe, not a selected sibling entity or excerpt.
    // Keep that original coverage while excluding sibling data from selected records.
    for selected_content in [
        &core["items"],
        &l0["direct_members"],
        &l1["local_entities"],
        &origins["core"]["supports"],
        &origins["core"]["source_custody"],
    ] {
        assert!(
            !std::str::from_utf8(&wow_core::canonical_json_bytes(selected_content)?)?
                .contains("other.lua")
        );
    }
    assert!(
        !text.contains(
            scratch
                .root
                .file_name()
                .ok_or("scratch filename")?
                .to_str()
                .ok_or("UTF-8 scratch")?
        )
    );
    for body in [LUA, PROJECT_OTHER_LUA] {
        assert!(!text.contains(body.lines().next().ok_or("Lua body prefix")?));
        assert!(!text.contains(&serde_json::to_string(body)?));
    }
    assert!(std::str::from_utf8(&canonical)?.contains(ADMITTED_PATH));
    Ok(payload.clone())
}

fn assert_artifact(artifact: &Value, pack: &Value) -> TestResult {
    let core = &artifact["core"];
    let content = core["content"].as_str().ok_or("artifact content")?;
    assert_eq!(
        core["schema"],
        "wow-context/rendered-context-artifact/primary/1"
    );
    assert_eq!(core["semantic_pack_id"], pack["semantic_pack_id"]);
    assert_eq!(core["byte_length"], content.len());
    assert_eq!(
        core["content_digest_basis"],
        "sha256_exact_rendered_utf8_bytes"
    );
    assert_eq!(
        core["content_digest"],
        ContentDigest::<CanonicalResult>::from_bytes(Sha256::digest(content.as_bytes()).into())
            .to_string()
    );
    assert_eq!(core["profile"]["rendering_loss"], "none");
    let mut end = 0usize;
    for record in rows(&core["trace"])? {
        let start = usize::try_from(record["byte_start"].as_u64().ok_or("start")?)?;
        let next = usize::try_from(record["byte_end"].as_u64().ok_or("end")?)?;
        assert_eq!(start, end, "ranges are consecutive and exhaustive");
        assert!(next > start);
        content
            .get(start..next)
            .ok_or("complete UTF-8 record range")?;
        end = next;
    }
    assert_eq!(end, content.len());
    Ok(())
}

fn assert_json(artifact: &Value, pack: &Value) -> TestResult {
    assert_artifact(artifact, pack)?;
    let core = &artifact["core"];
    assert_eq!(
        core["profile"]["profile_id"],
        PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID
    );
    let content = core["content"].as_str().ok_or("JSON content")?;
    assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
    let decoded: Value = serde_json::from_str(content)?;
    assert_eq!(decoded, *pack);
    assert_eq!(
        wow_core::canonical_json_bytes(&decoded)?,
        content.as_bytes()
    );
    assert_eq!(
        core["trace"],
        json!([{
            "record_id":pack["semantic_pack_id"], "template_id":"canonical_pack_json",
            "semantic_fields":["semantic_pack_id","core"], "byte_start":0, "byte_end":content.len()
        }])
    );
    Ok(())
}

pub(super) fn assert_markdown_lossless(artifact: &Value, pack: &Value) -> TestResult {
    assert_artifact(artifact, pack)?;
    let artifact_core = &artifact["core"];
    assert_eq!(
        artifact_core["profile"]["profile_id"],
        PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID
    );
    let content = artifact_core["content"]
        .as_str()
        .ok_or("Markdown content")?;
    let mut specifications = vec![
        (
            "pack_header",
            "PACK",
            vec!["semantic_pack_id", "core.schema"],
            None,
        ),
        ("pack_profile", "PROFILE", vec!["core.profile"], None),
        (
            "universe_binding",
            "UNIVERSE",
            vec!["core.universe_set_id", "core.binding"],
            None,
        ),
        ("pack_request", "REQUEST", vec!["core.request"], None),
        (
            "boundary_notice",
            "BOUNDARY",
            vec!["core.boundary_notice"],
            None,
        ),
        (
            "native_skeletons",
            "SKELETONS",
            vec!["core.skeletons"],
            None,
        ),
    ];
    for (index, item) in rows(&pack["core"]["items"])?.iter().enumerate() {
        let (template, label) = match item["core"]["payload"]["kind"].as_str() {
            Some("entity") => ("native_entity_item", "ENTITY"),
            Some("relation") => ("native_relation_item", "RELATION"),
            _ => return Err("non-native item".into()),
        };
        specifications.push((template, label, vec!["core.items"], Some(index)));
    }
    specifications.extend([
        (
            "selection_trace",
            "SELECTION_TRACE",
            vec!["core.selection_trace"],
            None,
        ),
        (
            "coverage_and_omissions",
            "COVERAGE",
            vec!["core.omissions", "core.state", "core.absence_authoritative"],
            None,
        ),
        ("pack_budget", "BUDGET", vec!["core.budget"], None),
    ]);
    let trace = rows(&artifact_core["trace"])?;
    assert_eq!(trace.len(), specifications.len());
    let mut reconstructed = json!({"semantic_pack_id":null,"core":{"items":[]}});
    for (record, (template, label, fields, item_index)) in trace.iter().zip(specifications) {
        assert_eq!(record["template_id"], template);
        assert_eq!(record["semantic_fields"], json!(fields));
        assert_eq!(
            record.get("item_index"),
            item_index.map(|index| json!(index)).as_ref()
        );
        let expected_id = item_index.map_or(&pack["semantic_pack_id"], |index| {
            &pack["core"]["items"][index]["context_item_id"]
        });
        assert_eq!(&record["record_id"], expected_id);
        let start = usize::try_from(record["byte_start"].as_u64().ok_or("start")?)?;
        let end = usize::try_from(record["byte_end"].as_u64().ok_or("end")?)?;
        let line = content.get(start..end).ok_or("UTF-8 line")?;
        assert_eq!(line.bytes().filter(|byte| *byte == b'\n').count(), 1);
        let quoted = line
            .strip_prefix(label)
            .and_then(|rest| rest.strip_prefix(' '))
            .and_then(|rest| rest.strip_suffix('\n'))
            .ok_or("fixed record framing")?;
        // Decode presentation JSON-string escaping, then its complete typed
        // record JSON. Decoded Values are transport checks, never owner authority.
        let inner: String = serde_json::from_str(quoted)?;
        let value: Value = serde_json::from_str(&inner)?;
        let core = &mut reconstructed["core"];
        match template {
            "pack_header" => {
                assert_eq!(value.as_object().ok_or("header")?.len(), 3);
                assert_eq!(value["item_count"], rows(&pack["core"]["items"])?.len());
                core["schema"] = value["schema"].clone();
                reconstructed["semantic_pack_id"] = value["semantic_pack_id"].clone();
            }
            "pack_profile" => core["profile"] = value,
            "universe_binding" => {
                assert_eq!(rows(&value)?.len(), 2);
                core["universe_set_id"] = value[0].clone();
                core["binding"] = value[1].clone();
            }
            "pack_request" => core["request"] = value,
            "boundary_notice" => core["boundary_notice"] = value,
            "native_skeletons" => core["skeletons"] = value,
            "native_entity_item" | "native_relation_item" => {
                assert_eq!(rows(&core["items"])?.len(), item_index.ok_or("item index")?);
                core["items"].as_array_mut().ok_or("items")?.push(value);
            }
            "selection_trace" => core["selection_trace"] = value,
            "coverage_and_omissions" => {
                assert_eq!(rows(&value)?.len(), 3);
                core["omissions"] = value[0].clone();
                core["state"] = value[1].clone();
                core["absence_authoritative"] = value[2].clone();
            }
            "pack_budget" => core["budget"] = value,
            _ => return Err("unknown template".into()),
        }
    }
    assert_eq!(
        reconstructed, *pack,
        "all semantic fields, including tuples, are lossless"
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&reconstructed)?,
        wow_core::canonical_json_bytes(pack)?
    );
    Ok(())
}

fn assert_markdown(artifact: &Value, pack: &Value) -> TestResult {
    assert_markdown_lossless(artifact, pack)?;
    let content = artifact["core"]["content"]
        .as_str()
        .ok_or("Markdown content")?;
    assert!(
        !content.contains(ADMITTED_PATH),
        "admitted source metadata cannot alter Markdown framing"
    );
    for escaped in [
        r"\u005b", r"\u005d", r"\u0028", r"\u0029", r"\u0023", r"\u0026", r"\u0060",
    ] {
        assert!(
            content.contains(escaped),
            "admitted delimiter is escaped: {escaped}"
        );
    }
    Ok(())
}

fn refused(
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    code: ServiceErrorCode,
    closure: &str,
    stop: &AtomicBool,
) -> TestResult {
    let result = execute_context_build(host, request, stop)?;
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
    assert_eq!(value["schema"], CONTEXT_BUILD_RESULT_SCHEMA);
    assert_eq!(value["resource_closure"], closure);
    assert_eq!(value["failure_code"], serde_json::to_value(code)?);
    assert!(
        value.get("payload").is_none(),
        "no partial pack or renderer draft escapes"
    );
    Ok(())
}

#[test]
fn retained_native_pack_is_lossless_across_renderers_and_keeps_one_budget() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let store_root = scratch.root.join("registered-store");
    let bundle = input_bundle_files(vec![
        ProjectInputFile::new(ADMITTED_PATH, LUA)?,
        ProjectInputFile::new("other.lua", PROJECT_OTHER_LUA)?,
    ])?;
    let (publisher, graph, reference, profile) = owners_from_bundle(bundle, &stop)?;
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let first = store.publish(
        &publisher,
        &graph,
        "fixture-context-build-initial",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    let expected = retained_build(&store_root, &first.generation_id, &stop)?;
    let selected = rows(&expected["files"])?
        .iter()
        .find(|file| file["file"]["path"] == ADMITTED_PATH)
        .ok_or("exact admitted file receipt")?;
    let root: GraphNodeId = serde_json::from_value(selected["node"]["node_id"].clone())?;
    let mut host_json = host_value(&store_root, &profile, &reference);
    // Explicit representative caller bounds, identical for all four positives.
    // Root measures this adversarial input; these are not global/default changes.
    host_json["limits"]["max_context_work"] = json!(FIXTURE_WORK);
    host_json["limits"]["max_metadata_bytes"] = json!(FIXTURE_METADATA);
    let host = decode_host(&host_json, &stop)?;
    let choices = [
        ContextRenderSelection::None,
        ContextRenderSelection::Json,
        ContextRenderSelection::Markdown,
        ContextRenderSelection::Both,
    ];
    let mut requests = Vec::new();
    let mut results = Vec::new();
    let mut payloads = Vec::new();
    for choice in choices {
        let request = request(&first.generation_id, &root, choice, &profile, &reference)?;
        let result = execute_context_build(&host, &request, &stop)?;
        let payload = assert_pack(
            &result,
            &expected,
            selected,
            &scratch,
            ContextMapStatus::Partial,
        )?;
        eprintln!(
            "context-build fixture {choice:?}: pack_bytes={} work={} bytes={}",
            payload["semantic_pack_byte_length"],
            payload["operation_budget_usage"]["work_reserved"],
            payload["operation_budget_usage"]["bytes_reserved"]
        );
        requests.push(request);
        results.push(result);
        payloads.push(payload);
    }
    let pack = &payloads[0]["semantic_pack"];
    let bytes = wow_core::canonical_json_bytes(pack)?;
    for payload in &payloads {
        assert_eq!(
            payload["semantic_pack"]["semantic_pack_id"],
            pack["semantic_pack_id"]
        );
        assert_eq!(
            wow_core::canonical_json_bytes(&payload["semantic_pack"])?,
            bytes
        );
    }
    assert!(rows(&payloads[0]["rendered_artifacts"])?.is_empty());
    let json_only = rows(&payloads[1]["rendered_artifacts"])?;
    let markdown_only = rows(&payloads[2]["rendered_artifacts"])?;
    let both_artifacts = rows(&payloads[3]["rendered_artifacts"])?;
    assert_eq!(json_only.len(), 1);
    assert_eq!(markdown_only.len(), 1);
    assert_eq!(both_artifacts.len(), 2);
    assert_json(&json_only[0], pack)?;
    assert_markdown(&markdown_only[0], pack)?;
    assert_eq!(
        both_artifacts,
        &[json_only[0].clone(), markdown_only[0].clone()],
        "fixed JSON then Markdown order; renderer identity excludes ledger counters"
    );
    let repeat = execute_context_build(&host, &requests[3], &stop)?;
    assert_eq!(repeat.canonical_bytes(), results[3].canonical_bytes());

    // Measure this fixture's actual single-render cumulative usage. Equal
    // decimal width preserves work charged per serialized budget digit; do not
    // subtract shared work, reset a ledger, or import the smoke fixture's counts.
    let individual_work = payloads[..3]
        .iter()
        .map(|payload| {
            payload["operation_budget_usage"]["work_reserved"]
                .as_u64()
                .ok_or("measured work")
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .ok_or("single usages")?;
    let both_work = payloads[3]["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("Both work")?;
    assert!(individual_work < both_work);
    assert_eq!(
        individual_work.to_string().len(),
        FIXTURE_WORK.to_string().len()
    );
    let mut bounded = host_json.clone();
    bounded["limits"]["max_context_work"] = json!(individual_work);
    let bounded = decode_host(&bounded, &stop)?;
    for request in &requests[..3] {
        let result = execute_context_build(&bounded, request, &stop)?;
        let payload = assert_pack(
            &result,
            &expected,
            selected,
            &scratch,
            ContextMapStatus::Partial,
        )?;
        assert!(
            payload["operation_budget_usage"]["work_reserved"]
                .as_u64()
                .ok_or("bounded work")?
                <= individual_work
        );
    }
    refused(
        &bounded,
        &requests[3],
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;

    let mut truncated = host_json.clone();
    truncated["limits"]["max_direct_edges"] = json!(1);
    let truncated = execute_context_build(&decode_host(&truncated, &stop)?, &requests[3], &stop)?;
    let truncated_payload = assert_pack(
        &truncated,
        &expected,
        selected,
        &scratch,
        ContextMapStatus::Truncated,
    )?;
    let truncated_pack = &truncated_payload["semantic_pack"];
    let truncated_artifacts = rows(&truncated_payload["rendered_artifacts"])?;
    assert_eq!(truncated_artifacts.len(), 2);
    assert_json(&truncated_artifacts[0], truncated_pack)?;
    assert_markdown(&truncated_artifacts[1], truncated_pack)?;
    for field in ["max_context_work", "max_metadata_bytes"] {
        let mut low = host_json.clone();
        low["limits"][field] = json!(1);
        refused(
            &decode_host(&low, &stop)?,
            &requests[3],
            ServiceErrorCode::BudgetExceeded,
            "closed",
            &stop,
        )?;
    }
    refused(
        &host,
        &requests[3],
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;

    let arguments = json!([
        "context",
        "build",
        "--native-host-config",
        scratch.root.join("host.json"),
        "--project-store",
        STORE_ID,
        "--publication",
        format!("store-generation:{}", first.generation_id.as_str()),
        "--root",
        root_token(&root),
        "--level",
        "both",
        "--context-profile-id",
        PRIMARY_CONTEXT_PACK_PROFILE_ID,
        "--renderer-profile-id",
        PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        "--renderer-profile-id",
        PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        "--platform",
        "omitted",
        "--expect-profile",
        profile.profile_id().as_str(),
        "--expect-reference-generation",
        reference.generation_id(),
        "--expect-reference-view",
        reference.self_digest(),
        "--output",
        "envelope-json"
    ]);
    assert_eq!(
        results[3].canonical_bytes(),
        include_bytes!("../../../e3/native-primary-pack/envelope.json")
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&requests[3])?,
        include_bytes!("../../../e3/native-primary-pack/request.json")
    );
    assert_eq!(
        bytes,
        include_bytes!("../../../e3/native-primary-pack/semantic-pack.json")
    );
    assert_eq!(
        markdown_only[0]["core"]["content"]
            .as_str()
            .ok_or("Markdown content")?
            .as_bytes(),
        include_bytes!("../../../e3/native-primary-pack/artifact.md")
    );
    assert_eq!(
        wow_context::PRIMARY_CONTEXT_PROFILES_BYTES,
        include_bytes!("../../../e3/native-primary-pack/profiles.json")
    );
    let checksums: Value = serde_json::from_slice(include_bytes!(
        "../../../e3/native-primary-pack/CHECKSUMS.json"
    ))?;
    let frozen: [(&str, &[u8]); 5] = [
        (
            "artifact.md",
            include_bytes!("../../../e3/native-primary-pack/artifact.md"),
        ),
        (
            "envelope.json",
            include_bytes!("../../../e3/native-primary-pack/envelope.json"),
        ),
        (
            "profiles.json",
            include_bytes!("../../../e3/native-primary-pack/profiles.json"),
        ),
        (
            "request.json",
            include_bytes!("../../../e3/native-primary-pack/request.json"),
        ),
        (
            "semantic-pack.json",
            include_bytes!("../../../e3/native-primary-pack/semantic-pack.json"),
        ),
    ];
    for (name, bytes) in frozen {
        assert_eq!(checksums[name]["bytes"], bytes.len());
        let digest =
            ContentDigest::<CanonicalResult>::from_bytes(Sha256::digest(bytes).into()).to_string();
        assert_eq!(
            checksums[name]["sha256"],
            digest.strip_prefix("sha256:").ok_or("frozen digest")?
        );
    }
    export_native_fixture(
        &scratch,
        &host_json,
        &requests[3],
        &arguments,
        &results[3],
        "synthetic two-file retained native Both pack; exact admitted adversarial path; no live/Ketho authority",
    )?;
    Ok(())
}
