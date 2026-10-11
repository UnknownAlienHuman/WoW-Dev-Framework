//! One bounded synthetic retained native inspect lifecycle; no live/Ketho authority.
use super::*;
use std::collections::BTreeMap;
use wow_emmy::function_calls::SourceFunctionKind;

struct InspectExpected {
    file_root: GraphNodeId,
    closure_root: GraphNodeId,
    current: CurrentPublication,
    records: Value,
}

fn retained_inspect(
    store_root: &Path,
    generation: &wow_store::project::StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<InspectExpected> {
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
    let graph = read.graph();
    let proof = read.pair().graph_read(stop)?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("native source provenance")?;
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("source partition")?;
    let report = provenance
        .function_call_report()
        .ok_or("native callable report")?;
    assert_eq!(provenance.files().len(), 2);
    assert_eq!(report.declaration_recipe(), None);
    assert_eq!(report.signature_recipe(), None);
    assert!(
        graph
            .snapshot()
            .nodes()
            .iter()
            .all(|node| node.kind() != "project")
    );
    let main = provenance
        .files()
        .iter()
        .find(|file| file.path == "main.lua")
        .ok_or("explicit fixture file")?;
    let file = proof.published_entity(SOURCE_GRAPH_PARTITION, &main.proposal_id, stop)?;
    let closures = provenance
        .functions()
        .iter()
        .filter(|row| row.path == "main.lua" && row.kind == SourceFunctionKind::Closure)
        .collect::<Vec<_>>();
    assert_eq!(
        closures.len(),
        1,
        "this input has one exact native Main closure"
    );
    let closure = proof.published_entity(SOURCE_GRAPH_PARTITION, &closures[0].proposal_id, stop)?;
    let file_root = file.node_id().clone();
    let closure_root = closure.node_id().clone();
    let files = provenance
        .files()
        .iter()
        .map(|file| {
            let node = proof.published_entity(SOURCE_GRAPH_PARTITION, &file.proposal_id, stop)?;
            Ok(json!({ "file": file, "node": node }))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut callables = BTreeMap::new();
    for function in provenance.functions() {
        let node = proof.published_entity(SOURCE_GRAPH_PARTITION, &function.proposal_id, stop)?;
        let fact = report
            .functions()
            .iter()
            .find(|fact| fact.fact_id() == function.function_id)
            .ok_or("original native callable fact")?;
        let file = provenance
            .files()
            .iter()
            .find(|file| file.path == function.path)
            .ok_or("original callable file")?;
        assert_eq!(fact.span(), function.span);
        assert_eq!(fact.kind(), function.kind);
        assert!(fact.declaration().is_none() && fact.signature().is_none());
        assert!(
            callables
                .insert(
                    node.node_id().to_string(),
                    json!({
                        "node": node, "function": function, "fact": fact,
                        "content_digest": file.content_digest,
                        "function_analysis_id": report.analysis_id()
                    })
                )
                .is_none()
        );
    }
    assert_eq!(callables.len(), 4, "two genuine Chunks and two Closures");
    let file_edges = graph
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| edge.from() == &file_root)
        .collect::<Vec<_>>();
    assert_eq!(file_edges.len(), 2);
    assert!(
        file_edges
            .iter()
            .all(|edge| edge.relation() == GraphRelationKind::Owns)
    );
    let closure_edges = graph
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| edge.from() == &closure_root || edge.to() == &closure_root)
        .collect::<Vec<_>>();
    assert_eq!(closure_edges.len(), 1);
    assert_eq!(closure_edges[0].from(), &file_root);
    let relations = source
        .report()
        .accepted_relations()
        .iter()
        .map(|accepted| {
            let proposal = source
                .batch()
                .relation_proposal(accepted.proposal_id())
                .ok_or("original accepted relation proposal")?;
            Ok(serde_json::to_value(proposal)?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let records = json!({
        "project_id": read.project().configuration().project_id(),
        "publication_set_id": read.publication_set_id(),
        "store_generation_id": generation.as_str(),
        "source_context_id": graph.source_context_id(),
        "files": files, "file_node": file, "closure_node": closure,
        "file_edges": file_edges, "closure_edges": closure_edges,
        "nodes": graph.snapshot().nodes(), "edges": graph.snapshot().edges(),
        "callables": callables,
        "entity_proposals": source.batch().entity_proposals(),
        "accepted_entities": source.report().accepted_entities(),
        "relation_proposals": relations,
        "accepted_relations": source.report().accepted_relations(),
        "evidence": provenance.evidence().values().collect::<Vec<_>>(),
        "handles": provenance.source_handles().values().collect::<Vec<_>>()
    });
    let current = read
        .current_at_acquisition()
        .ok_or("exact Current observation")?
        .clone();
    read.close()?;
    store.close()?;
    Ok(InspectExpected {
        file_root,
        closure_root,
        current,
        records,
    })
}

fn exact_inspect_request(
    generation: &wow_store::project::StoreGenerationId,
    level: ContextInspectLevel,
    root: ContextMapRoot,
    profile: &ProfileIdentity,
    reference: &ReferenceView,
) -> TestResult<ContextInspectRequest> {
    Ok(ContextInspectRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(generation.as_str())?,
        level,
        root,
        None,
        Some(reference_guard(profile, reference)?),
    )?)
}

fn rows(value: &Value) -> TestResult<&[Value]> {
    Ok(value
        .as_array()
        .ok_or("required native record array")?
        .as_slice())
}

fn exact_rows(actual: &Value, expected: &[Value]) -> TestResult {
    let actual = rows(actual)?;
    assert_eq!(actual.len(), expected.len());
    for row in expected {
        assert_eq!(
            actual.iter().filter(|candidate| *candidate == row).count(),
            1
        );
    }
    Ok(())
}

fn selected_nodes(
    expected: &InspectExpected,
    root: &Value,
    edges: &[Value],
) -> TestResult<Vec<Value>> {
    let mut ids = BTreeSet::from([root["node_id"].as_str().ok_or("root ID")?]);
    for edge in edges {
        ids.insert(edge["from"].as_str().ok_or("native edge source")?);
        ids.insert(edge["to"].as_str().ok_or("native edge target")?);
    }
    Ok(rows(&expected.records["nodes"])?
        .iter()
        .filter(|node| node["node_id"].as_str().is_some_and(|id| ids.contains(id)))
        .cloned()
        .collect())
}

fn assert_origins(
    context: &Value,
    expected: &InspectExpected,
    nodes: &[Value],
    edges: &[Value],
    state: ContextMapStatus,
) -> TestResult {
    let origins = &context["origins"];
    let core = &origins["core"];
    assert_eq!(context["origin_map_id"], origins["project_map_id"]);
    assert_eq!(context["universe_set_id"], core["universe_set_id"]);
    assert_eq!(context["state"], serde_json::to_value(state)?);
    assert_eq!(core["state"], serde_json::to_value(state)?);
    assert_eq!(context["absence_authoritative"], false);
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(context["source_bodies"], "excluded");
    let mut pending = BTreeSet::new();
    for node in nodes {
        pending.insert((
            "entity".to_string(),
            node["node_id"].as_str().ok_or("node ID")?.to_string(),
        ));
    }
    for edge in edges {
        pending.insert((
            "relation".to_string(),
            edge["edge_id"].as_str().ok_or("edge ID")?.to_string(),
        ));
    }
    let supports = rows(&core["supports"])?;
    assert_eq!(
        supports.len(),
        pending.len(),
        "every selected native record requires support"
    );
    for receipt in supports {
        let explanation = &receipt["explanation"];
        assert_eq!(
            explanation["source_context_id"],
            expected.records["source_context_id"]
        );
        assert_eq!(explanation["support_complete"], true);
        assert!(rows(&explanation["truncations"])?.is_empty());
        let record = &explanation["record"];
        let (kind, id) = match record["kind"].as_str() {
            Some("entity") => {
                assert!(nodes.contains(&record["node"]));
                (
                    "entity",
                    record["node"]["node_id"]
                        .as_str()
                        .ok_or("explained node ID")?,
                )
            }
            Some("relation") => {
                assert!(edges.contains(&record["edge"]));
                (
                    "relation",
                    record["edge"]["edge_id"]
                        .as_str()
                        .ok_or("explained edge ID")?,
                )
            }
            _ => return Err("unexpected non-native explained record".into()),
        };
        assert!(pending.remove(&(kind.to_string(), id.to_string())));
        let native = rows(&explanation["supports"])?;
        assert!(!native.is_empty());
        for support in native {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("fixture has no foundation-origin support".into()),
            };
            assert!(rows(&expected.records[proposals])?.contains(&support["proposal"]));
            assert!(rows(&expected.records[accepted])?.contains(&support["accepted"]));
        }
        let resolution = &receipt["evidence_resolution"];
        assert_eq!(resolution["closure_complete"], true);
        assert!(rows(&resolution["unresolved"])?.is_empty());
        assert!(rows(&resolution["truncations"])?.is_empty());
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            let native = rows(&resolution[actual])?;
            assert!(!native.is_empty());
            for row in native {
                assert!(rows(&expected.records[retained])?.contains(row));
            }
        }
    }
    assert!(pending.is_empty());
    for custody in rows(&core["source_custody"])? {
        assert_eq!(custody["scope"], "primary");
        assert!(rows(&expected.records["handles"])?.contains(&custody["handle"]));
    }
    Ok(())
}

fn assert_declarations(actual: &Value, expected: &InspectExpected, nodes: &[Value]) -> TestResult {
    let callables = expected.records["callables"]
        .as_object()
        .ok_or("retained callable joins")?;
    let selected = nodes
        .iter()
        .filter(|node| node["kind"] == "lua_source_function")
        .collect::<Vec<_>>();
    assert_eq!(rows(actual)?.len(), selected.len());
    for node in selected {
        let id = node["node_id"].as_str().ok_or("exact callable node ID")?;
        let retained = callables.get(id).ok_or("original callable observation")?;
        let matched = rows(actual)?
            .iter()
            .filter(|row| row["node"] == *node)
            .collect::<Vec<_>>();
        assert_eq!(matched.len(), 1);
        let row = matched[0];
        assert_eq!(row["kind"], "callable");
        let metadata = &row["metadata"];
        assert_eq!(metadata["function"], retained["function"]);
        assert_eq!(
            metadata["function_analysis_id"],
            retained["function_analysis_id"]
        );
        assert_eq!(metadata["content_digest"], retained["content_digest"]);
        assert_eq!(
            metadata.get("parent_function_id"),
            retained["fact"].get("parent_function_id")
        );
        assert_eq!(metadata["function"]["span"], retained["fact"]["span"]);
        assert_eq!(metadata["function"]["kind"], retained["fact"]["kind"]);
        for facet in ["declaration", "signature"] {
            assert_eq!(metadata[facet]["state"], "not_selected");
            assert!(metadata[facet].get("observation").is_none());
            assert!(metadata[facet].get("recipe").is_none());
        }
    }
    Ok(())
}

fn inspect_payload(
    result: &ContextMapResult,
    expected: &InspectExpected,
    scratch: &Scratch,
    state: ContextMapStatus,
) -> TestResult<Value> {
    assert_eq!(result.status(), state);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], "wow-service/context-inspect-result/1");
    assert_eq!(envelope["operation"], "context_inspect");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(
        payload["publication_set_id"],
        expected.records["publication_set_id"]
    );
    assert_eq!(
        payload["store_generation_id"],
        expected.records["store_generation_id"]
    );
    assert_eq!(
        payload["current_at_acquisition"],
        serde_json::to_value(&expected.current)?
    );
    assert_eq!(
        payload["current_record_digest"],
        observed_current_digest(&expected.current)?
    );
    assert_eq!(payload["configured_registration_id"], STORE_ID);
    assert!(payload.get("native_project_store_id").is_none());
    assert!(
        payload["resolved_request_id"]
            .as_str()
            .ok_or("resolved inspect request")?
            .starts_with("context-inspect-resolved-request:sha256:")
    );
    assert_eq!(
        payload["skeletons"]["schema"],
        "wow-context/primary-skeleton-inspection/1"
    );
    let text = std::str::from_utf8(result.canonical_bytes())?;
    assert!(!text.contains("project_store_root"));
    assert!(!text.contains(&serde_json::to_string(
        &scratch.root.join("registered-store")
    )?));
    assert!(
        !text.contains(
            scratch
                .root
                .file_name()
                .ok_or("scratch name")?
                .to_str()
                .ok_or("scratch UTF-8")?
        )
    );
    for body in [LUA, PROJECT_OTHER_LUA] {
        assert!(!text.contains(&serde_json::to_string(body)?));
        assert!(!text.contains(body.lines().next().ok_or("Lua body prefix")?));
    }
    Ok(payload.clone())
}

fn assert_file_both(
    result: &ContextMapResult,
    expected: &InspectExpected,
    scratch: &Scratch,
    state: ContextMapStatus,
) -> TestResult<Value> {
    let payload = inspect_payload(result, expected, scratch, state)?;
    let skeletons = &payload["skeletons"];
    assert_eq!(skeletons["level"], "both");
    let root = &expected.records["file_node"];
    let all_edges = rows(&expected.records["file_edges"])?;
    let edges = if state == ContextMapStatus::Truncated {
        &all_edges[..1]
    } else {
        all_edges
    };
    let nodes = selected_nodes(expected, root, edges)?;
    let members = nodes
        .iter()
        .filter(|node| *node != root)
        .cloned()
        .collect::<Vec<_>>();
    let l0 = &skeletons["l0"]["core"];
    let l1 = &skeletons["l1"]["core"];
    assert_eq!(l0["profile"], PRIMARY_L0_PROFILE_ID);
    assert_eq!(l1["profile"], PRIMARY_L1_PROFILE_ID);
    assert_eq!(l0["scope"]["kind"], "entity");
    assert_eq!(l0["scope"]["node"], *root);
    assert_eq!(l1["root"], *root);
    exact_rows(&l0["direct_members"], &members)?;
    exact_rows(&l1["local_entities"], &nodes)?;
    exact_rows(&l0["direct_relations"], edges)?;
    exact_rows(&l1["direct_relations"], edges)?;
    exact_rows(
        &l0["detail_routes"],
        &members
            .iter()
            .map(|node| node["node_id"].clone())
            .collect::<Vec<_>>(),
    )?;
    assert_eq!(l0["selected_member_count"], members.len());
    assert_eq!(
        l0["member_enumeration"],
        "selected_outgoing_native_owns_or_contains"
    );
    assert_eq!(l0["context"]["origins"], l1["context"]["origins"]);
    for core in [l0, l1] {
        assert_origins(&core["context"], expected, &nodes, edges, state)?;
        assert_declarations(&core["declarations"], expected, &nodes)?;
        let navigation = &core["context"]["origins"]["core"]["navigation"];
        assert_eq!(
            navigation["root"], *root,
            "truncation cannot prune the mandatory root"
        );
        exact_rows(&navigation["edges"], edges)?;
        assert_eq!(navigation["matching_edges"], 2);
        assert_eq!(navigation["omitted_edges"], all_edges.len() - edges.len());
    }
    Ok(payload)
}

fn assert_inspect_refused(
    host: &ContextMapHostInput,
    request: &ContextInspectRequest,
    code: ServiceErrorCode,
    closure: &str,
    stop: &AtomicBool,
) -> TestResult {
    let result = execute_context_inspect(host, request, stop)?;
    assert_eq!(result.failure_code(), Some(code));
    assert_eq!(
        result.status(),
        if code == ServiceErrorCode::Cancelled {
            ContextMapStatus::Cancelled
        } else {
            ContextMapStatus::Failed
        }
    );
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], "wow-service/context-inspect-result/1");
    assert_eq!(envelope["failure_code"], serde_json::to_value(code)?);
    assert_eq!(envelope["resource_closure"], closure);
    assert!(
        envelope.get("payload").is_none(),
        "no failed skeleton draft escapes"
    );
    Ok(())
}

#[test]
fn retained_native_inspect_projects_exact_file_and_closure_without_losing_custody() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let store_root = scratch.root.join("registered-store");
    let bundle = input_bundle_files(vec![
        ProjectInputFile::new("main.lua", LUA)?,
        ProjectInputFile::new("other.lua", PROJECT_OTHER_LUA)?,
    ])?;
    let project_id = bundle.configuration().project_id().clone();
    let (publisher, graph, reference, profile) = owners_from_bundle(bundle, &stop)?;
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let first = store.publish(
        &publisher,
        &graph,
        "fixture-context-inspect-initial",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    let mut expected = retained_inspect(&store_root, &first.generation_id, &stop)?;
    let host_json = host_value(&store_root, &profile, &reference);
    let host = decode_host(&host_json, &stop)?;
    let both = exact_inspect_request(
        &first.generation_id,
        ContextInspectLevel::Both,
        ContextMapRoot::graph_entity(expected.file_root.as_str())?,
        &profile,
        &reference,
    )?;
    let positive = execute_context_inspect(&host, &both, &stop)?;
    let original = assert_file_both(&positive, &expected, &scratch, ContextMapStatus::Partial)?;
    let repeat = execute_context_inspect(&host, &both, &stop)?;
    assert_eq!(positive.canonical_bytes(), repeat.canonical_bytes());

    // Either level fits this measured work allowance independently. Both must
    // retain the first level's reservation instead of resetting for the second.
    let mut individual_work = 0;
    let mut individual_requests = Vec::new();
    for level in [ContextInspectLevel::L0, ContextInspectLevel::L1] {
        let request = exact_inspect_request(
            &first.generation_id,
            level,
            ContextMapRoot::graph_entity(expected.file_root.as_str())?,
            &profile,
            &reference,
        )?;
        let result = execute_context_inspect(&host, &request, &stop)?;
        let value: Value = serde_json::from_slice(result.canonical_bytes())?;
        individual_work = individual_work.max(
            value["payload"]["skeletons"]["work_reserved"]
                .as_u64()
                .ok_or("individual work reservation")?,
        );
        individual_requests.push(request);
    }
    assert!(
        original["skeletons"]["work_reserved"]
            .as_u64()
            .ok_or("combined work reservation")?
            > individual_work
    );
    let mut bounded_host = host_json.clone();
    bounded_host["limits"]["max_context_work"] = json!(individual_work);
    let bounded_host = decode_host(&bounded_host, &stop)?;
    for request in individual_requests {
        assert_eq!(
            execute_context_inspect(&bounded_host, &request, &stop)?.status(),
            ContextMapStatus::Partial
        );
    }
    assert_inspect_refused(
        &bounded_host,
        &both,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;

    let closure_request = exact_inspect_request(
        &first.generation_id,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(expected.closure_root.as_str())?,
        &profile,
        &reference,
    )?;
    let closure_result = execute_context_inspect(&host, &closure_request, &stop)?;
    let payload = inspect_payload(
        &closure_result,
        &expected,
        &scratch,
        ContextMapStatus::Partial,
    )?;
    assert!(payload["skeletons"].get("l0").is_none());
    let l1 = &payload["skeletons"]["l1"]["core"];
    assert_eq!(l1["root"], expected.records["closure_node"]);
    let closure_edges = rows(&expected.records["closure_edges"])?;
    let closure_nodes =
        selected_nodes(&expected, &expected.records["closure_node"], closure_edges)?;
    exact_rows(&l1["local_entities"], &closure_nodes)?;
    exact_rows(&l1["direct_relations"], closure_edges)?;
    assert_declarations(&l1["declarations"], &expected, &closure_nodes)?;
    assert_origins(
        &l1["context"],
        &expected,
        &closure_nodes,
        closure_edges,
        ContextMapStatus::Partial,
    )?;

    let project_request = exact_inspect_request(
        &first.generation_id,
        ContextInspectLevel::L0,
        ContextMapRoot::project(project_id.as_str())?,
        &profile,
        &reference,
    )?;
    let project_result = execute_context_inspect(&host, &project_request, &stop)?;
    for level in [ContextInspectLevel::L1, ContextInspectLevel::Both] {
        let incompatible = exact_inspect_request(
            &first.generation_id,
            level,
            ContextMapRoot::project(project_id.as_str())?,
            &profile,
            &reference,
        )?;
        assert_inspect_refused(
            &host,
            &incompatible,
            ServiceErrorCode::InvalidRequest,
            "closed",
            &stop,
        )?;
    }
    let payload = inspect_payload(
        &project_result,
        &expected,
        &scratch,
        ContextMapStatus::Partial,
    )?;
    assert!(payload["skeletons"].get("l1").is_none());
    let l0 = &payload["skeletons"]["l0"]["core"];
    assert_eq!(
        l0["scope"],
        json!({ "kind": "project", "project_id": project_id })
    );
    let files = rows(&expected.records["files"])?
        .iter()
        .map(|file| file["node"].clone())
        .collect::<Vec<_>>();
    exact_rows(&l0["direct_members"], &files)?;
    exact_rows(
        &l0["detail_routes"],
        &files
            .iter()
            .map(|file| file["node_id"].clone())
            .collect::<Vec<_>>(),
    )?;
    assert_eq!(l0["selected_member_count"], 2);
    let nodes = rows(&expected.records["nodes"])?;
    let edges = rows(&expected.records["edges"])?;
    exact_rows(&l0["direct_relations"], edges)?;
    assert_declarations(&l0["declarations"], &expected, nodes)?;
    assert_origins(
        &l0["context"],
        &expected,
        nodes,
        edges,
        ContextMapStatus::Partial,
    )?;
    exact_rows(
        &l0["context"]["origins"]["core"]["navigation"]["source_file_roots"],
        rows(&expected.records["files"])?,
    )?;

    let mut truncated_host = host_json.clone();
    truncated_host["limits"]["max_direct_edges"] = json!(1);
    let truncated = execute_context_inspect(&decode_host(&truncated_host, &stop)?, &both, &stop)?;
    assert_file_both(&truncated, &expected, &scratch, ContextMapStatus::Truncated)?;
    let invalid_project = exact_inspect_request(
        &first.generation_id,
        ContextInspectLevel::L0,
        ContextMapRoot::project("fixture-context-unselected")?,
        &profile,
        &reference,
    )?;
    assert_inspect_refused(
        &host,
        &invalid_project,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    for field in ["max_context_work", "max_metadata_bytes"] {
        let mut low = host_json.clone();
        low["limits"][field] = json!(1);
        assert_inspect_refused(
            &decode_host(&low, &stop)?,
            &both,
            ServiceErrorCode::BudgetExceeded,
            "closed",
            &stop,
        )?;
    }
    assert_inspect_refused(
        &host,
        &both,
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;

    // A real source update, not an invented ID, supplies the mixed-root/stale-Current controls.
    let (publisher, graph, next_reference, next_profile) = owners_from_bundle(
        input_bundle_files(vec![
            ProjectInputFile::new(
                "main.lua",
                "local function ContextFixture(value)\n    return value + 2\nend\n",
            )?,
            ProjectInputFile::new("other.lua", PROJECT_OTHER_LUA)?,
        ])?,
        &stop,
    )?;
    assert_eq!(reference, next_reference);
    assert_eq!(profile, next_profile);
    let mut store = LiveProjectStore::open(&store_root)?;
    let second = store.publish(
        &publisher,
        &graph,
        "fixture-context-inspect-next",
        Some(expected.current.record_id.clone()),
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    let next = retained_inspect(&store_root, &second.generation_id, &stop)?;
    assert_ne!(expected.closure_root, next.closure_root);
    assert!(
        !rows(&expected.records["nodes"])?
            .iter()
            .any(|node| node["node_id"].as_str() == Some(next.closure_root.as_str()))
    );
    let mixed = exact_inspect_request(
        &first.generation_id,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(next.closure_root.as_str())?,
        &profile,
        &reference,
    )?;
    assert_inspect_refused(
        &host,
        &mixed,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    let old_digest = observed_current_digest(&expected.current)?;
    assert_ne!(old_digest, observed_current_digest(&next.current)?);
    let stale = ContextInspectRequest::primary(
        STORE_ID,
        ContextMapPublication::Current,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(next.closure_root.as_str())?,
        Some(ContextMapCurrentGuard::new(
            Some(expected.current.record_id.as_str()),
            Some(&old_digest),
        )?),
        Some(reference_guard(&profile, &reference)?),
    )?;
    assert_inspect_refused(
        &host,
        &stale,
        ServiceErrorCode::StoreCurrentConflict,
        "closed",
        &stop,
    )?;
    // Reopen after every control sequence and export the actual exact-generation envelope
    // with the Current observation that a later CLI acquisition will see.
    let store = LiveProjectStore::open(&store_root)?;
    let observed = store.current()?.ok_or("post-control actual Current")?;
    assert_eq!(observed, next.current);
    store.close()?;
    expected.current = observed;
    let exported = execute_context_inspect(&host, &both, &stop)?;
    let final_payload =
        assert_file_both(&exported, &expected, &scratch, ContextMapStatus::Partial)?;
    assert_eq!(
        original["skeletons"], final_payload["skeletons"],
        "the exact retained semantic skeleton does not change when Current advances"
    );
    assert_eq!(
        exported.canonical_bytes(),
        include_bytes!("../../../e3/native-primary-inspect/envelope.json")
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&both)?,
        include_bytes!("../../../e3/native-primary-inspect/request.json")
    );
    let frozen_profiles = json!({
        "l0": final_payload["skeletons"]["l0"]["core"]["profile_descriptor"],
        "l1": final_payload["skeletons"]["l1"]["core"]["profile_descriptor"],
        "origins": final_payload["skeletons"]["l0"]["core"]["context"]["origins"]["core"]["profile"],
    });
    assert_eq!(
        wow_core::canonical_json_bytes(&frozen_profiles)?,
        include_bytes!("../../../e3/native-primary-inspect/profiles.json")
    );
    let arguments = json!([
        "context",
        "inspect",
        "--native-host-config",
        scratch.root.join("host.json"),
        "--project-store",
        STORE_ID,
        "--publication",
        format!("store-generation:{}", first.generation_id.as_str()),
        "--root",
        root_token(&expected.file_root),
        "--level",
        "both",
        "--l0-profile-id",
        PRIMARY_L0_PROFILE_ID,
        "--l1-profile-id",
        PRIMARY_L1_PROFILE_ID,
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
    export_native_fixture(
        &scratch,
        &host_json,
        &both,
        &arguments,
        &exported,
        "synthetic two-file native inspect; exact retained Both-file root after controls",
    )?;
    Ok(())
}
