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

// Existing synthetic TOC/state fixture, admitted by the real ordinary load owner.
const STATE_TOC: &str = include_str!("../../../tests/data/state-core-pipeline/StateCore.toc");
const STATE_LUA: &str = include_str!("../../../tests/data/state-core-pipeline/state.lua");
const STATE_L0_PROFILE: &str = "wow-context/primary-state-root-l0/1";

fn state_consumer_bundle(input: &Path, stop: &AtomicBool) -> TestResult<ProjectInputBundle> {
    use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
    use wow_project::load::TocLoadContext;
    std::fs::create_dir(input)?;
    write_export(input, "StateCore.toc", STATE_TOC.as_bytes(), 8192)?;
    write_export(input, "state.lua", STATE_LUA.as_bytes(), 8192)?;
    let base = input_bundle(LUA)?;
    let original = base.configuration();
    let directory = ProjectInputDirectory::open(input)?;
    let (files, plan) = directory
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("StateCore.toc"),
            original.selected_profile(),
            Some(&TocLoadContext {
                game_types: BTreeMap::from([("retail".into(), true)]),
                family: None,
                game: None,
                text_locale: None,
                location: None,
                environment: None,
            }),
            stop,
        )?
        .into_parts();
    let configuration = ProjectConfigurationBuilder::new(
        original.project_id().clone(),
        original.project_kind(),
        original.selected_profile().clone(),
        original.reference_generation(),
        original.analyzer_binding().clone(),
    )
    .workspace_id(original.workspace_id().clone())
    .source_origin_id(original.source_origin_id().clone())
    .logical_root(original.logical_root().as_str())
    .capability_policy(original.capability_policy().clone())
    .budget_policy(original.budget_policy())
    .load_plan(&plan)?
    .build()?;
    Ok(ProjectInputBundle::closed(
        configuration,
        files,
        base.libraries().to_vec(),
    )?)
}

struct StateConsumerExpected {
    current: CurrentPublication,
    root_ids: Vec<GraphNodeId>,
    unrelated: GraphNodeId,
    records: Value,
}

fn retained_state_consumer(
    store_root: &Path,
    selector: &ReadSelector,
    stop: &AtomicBool,
) -> TestResult<StateConsumerExpected> {
    use wow_project::load::{TocSavedVariableScope, TocSavedVariableState};
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(selector, stop)?;
    let graph = read.graph();
    let proof = read.pair().graph_read(stop)?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("original native state provenance")?;
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("original source partition")?;
    let plan = read
        .project()
        .configuration()
        .load_plan()
        .ok_or("retained ordinary TOC plan")?;
    assert_eq!(plan.selected_toc(), "StateCore.toc");
    assert!(read.namespace().is_none() && read.candidate().is_none());
    assert!(read.project().configuration().package_load_plan().is_none());
    assert_eq!(provenance.state_roots().len(), 2);
    let mut root_ids = Vec::new();
    let mut roots = Vec::new();
    for origin in provenance.state_roots() {
        assert!(!origin.ambiguous);
        match origin.scope {
            TocSavedVariableScope::Account => assert_eq!(origin.name, "StateCoreAccountDB"),
            TocSavedVariableScope::Character => assert_eq!(origin.name, "StateCoreCharacterDB"),
        }
        let node = proof.published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)?;
        assert_eq!(node.kind(), "state_root");
        assert!(graph.snapshot().node(node.node_id()).is_some());
        let declarations = provenance
            .state_declarations()
            .iter()
            .filter(|row| {
                row.state == TocSavedVariableState::Declared
                    && row.name == origin.name
                    && row.scope == origin.scope
                    && origin.source_handle_ids.contains(&row.source_handle_id)
                    && origin.evidence_ids.contains(&row.evidence_id)
            })
            .collect::<Vec<_>>();
        assert!(!declarations.is_empty());
        let mut paths = Vec::new();
        for path in provenance
            .state_paths()
            .iter()
            .filter(|row| row.root_id == origin.root_id)
        {
            let path_node =
                proof.published_entity(SOURCE_GRAPH_PARTITION, &path.proposal_id, stop)?;
            assert_eq!(path_node.kind(), "state_path");
            assert!(graph.snapshot().edges().iter().any(|edge| {
                edge.from() == node.node_id()
                    && edge.to() == path_node.node_id()
                    && edge.relation() == GraphRelationKind::Owns
            }));
            paths.push(json!({"node":path_node,"origin":path}));
        }
        assert!(!paths.is_empty());
        root_ids.push(node.node_id().clone());
        roots.push(json!({"node":node,"origin":origin,"declarations":declarations,"paths":paths}));
    }
    let file = provenance
        .files()
        .iter()
        .find(|file| file.path == "state.lua")
        .ok_or("exact unrelated native file")?;
    let unrelated = proof
        .published_entity(SOURCE_GRAPH_PARTITION, &file.proposal_id, stop)?
        .node_id()
        .clone();
    let relations = source
        .report()
        .accepted_relations()
        .iter()
        .map(|accepted| {
            let proposal = source
                .batch()
                .relation_proposal(accepted.proposal_id())
                .ok_or("original accepted relation")?;
            Ok(serde_json::to_value(proposal)?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let current = read
        .current_at_acquisition()
        .ok_or("retained Current")?
        .clone();
    let records = json!({
        "roots":roots,"nodes":graph.snapshot().nodes(),"edges":graph.snapshot().edges(),
        "publication_set_id":read.publication_set_id(),"source_context_id":graph.source_context_id(),
        "entity_proposals":source.batch().entity_proposals(),
        "accepted_entities":source.report().accepted_entities(),
        "relation_proposals":relations,"accepted_relations":source.report().accepted_relations(),
        "evidence":provenance.evidence().values().collect::<Vec<_>>(),
        "handles":provenance.source_handles().values().collect::<Vec<_>>(),
        "load":{"digest":plan.digest(),"selected_toc":plan.selected_toc(),
            "load_context":plan.load_context(),"sources":plan.sources(),
            "records":plan.records(),"issues":plan.issues()}
    });
    read.close()?;
    store.close()?;
    Ok(StateConsumerExpected {
        current,
        root_ids,
        unrelated,
        records,
    })
}

fn assert_state_consumer_l0(
    l0: &Value,
    selected: &Value,
    expected: &StateConsumerExpected,
) -> TestResult {
    let core = &l0["core"];
    assert_eq!(
        core["schema"],
        "wow-context/l0-skeleton/primary-state-root/1"
    );
    assert_eq!(core["profile"], STATE_L0_PROFILE);
    assert_eq!(core["profile_descriptor"]["profile_id"], STATE_L0_PROFILE);
    assert_eq!(core["scope"]["node"], selected["node"]);
    assert_eq!(core["context"]["absence_authoritative"], false);
    assert_eq!(core["context"]["source_bodies"], "excluded");
    assert_eq!(core["context"]["tokens"], "unavailable");
    assert!(rows(&core["not_evaluated"])?.contains(&json!("complete_corpus_or_runtime_behavior")));
    let paths = rows(&selected["paths"])?;
    let members = paths
        .iter()
        .map(|row| row["node"].clone())
        .collect::<Vec<_>>();
    exact_rows(&core["direct_members"], &members)?;
    assert_eq!(core["selected_member_count"], members.len());
    let declarations = rows(&core["declarations"])?;
    assert_eq!(declarations.len(), paths.len() + 1);
    let roots = declarations
        .iter()
        .filter(|row| row["kind"] == "state_root")
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["node"], selected["node"]);
    assert_eq!(roots[0]["origin"], selected["origin"]);
    exact_rows(&roots[0]["declarations"], rows(&selected["declarations"])?)?;
    let actual_paths = declarations
        .iter()
        .filter(|row| row["kind"] == "state_path")
        .collect::<Vec<_>>();
    assert_eq!(actual_paths.len(), paths.len());
    for path in paths {
        assert_eq!(
            actual_paths
                .iter()
                .filter(|row| { row["node"] == path["node"] && row["origin"] == path["origin"] })
                .count(),
            1
        );
    }
    // Positive literal witnesses, not an exhaustive field/runtime census.
    let required: Vec<Vec<&str>> = match selected["origin"]["scope"].as_str() {
        Some("account") => vec![
            vec!["profile"],
            vec!["profile", "name"],
            vec!["settings", "volume"],
        ],
        Some("character") => vec![vec!["gold"]],
        _ => return Err("unexpected native SavedVariables scope".into()),
    };
    for keys in required {
        let keys = keys
            .iter()
            .map(|value| json!({"kind":"string","value":value}))
            .collect::<Vec<_>>();
        assert!(
            paths
                .iter()
                .any(|path| path["origin"]["keys"] == json!(keys))
        );
    }
    let root = &selected["node"]["node_id"];
    let edges = rows(&expected.records["edges"])?
        .iter()
        .filter(|edge| {
            (&edge["from"] == root || &edge["to"] == root)
                && matches!(edge["relation"].as_str(), Some("owns" | "contains"))
        })
        .cloned()
        .collect::<Vec<_>>();
    exact_rows(&core["direct_relations"], &edges)?;
    let mut nodes = rows(&expected.records["nodes"])?
        .iter()
        .filter(|node| {
            &node["node_id"] == root
                || edges
                    .iter()
                    .any(|edge| node["node_id"] == edge["from"] || node["node_id"] == edge["to"])
        })
        .cloned()
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a["node_id"].as_str().cmp(&b["node_id"].as_str()));
    let origins = &core["context"]["origins"]["core"];
    assert_eq!(origins["absence_authoritative"], false);
    assert_eq!(
        origins["load_custody"]["selected_toc"],
        expected.records["load"]
    );
    let mut pending = BTreeSet::new();
    for (kind, records, field) in [
        ("entity", nodes.as_slice(), "node_id"),
        ("relation", edges.as_slice(), "edge_id"),
    ] {
        for row in records {
            assert!(pending.insert((
                kind.to_owned(),
                row[field].as_str().ok_or("native ID")?.to_owned()
            )));
        }
    }
    for receipt in rows(&origins["supports"])? {
        let explanation = &receipt["explanation"];
        assert_eq!(explanation["support_complete"], true);
        assert_eq!(
            explanation["source_context_id"],
            expected.records["source_context_id"]
        );
        let record = &explanation["record"];
        let (kind, id) = match record["kind"].as_str() {
            Some("entity") => {
                assert!(nodes.contains(&record["node"]));
                (
                    "entity",
                    record["node"]["node_id"].as_str().ok_or("node ID")?,
                )
            }
            Some("relation") => {
                assert!(edges.contains(&record["edge"]));
                (
                    "relation",
                    record["edge"]["edge_id"].as_str().ok_or("edge ID")?,
                )
            }
            _ => return Err("unexpected nonnative support".into()),
        };
        assert!(pending.remove(&(kind.to_owned(), id.to_owned())));
        assert!(!rows(&explanation["supports"])?.is_empty());
        for support in rows(&explanation["supports"])? {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("unexpected foundation support".into()),
            };
            assert!(rows(&expected.records[proposals])?.contains(&support["proposal"]));
            assert!(rows(&expected.records[accepted])?.contains(&support["accepted"]));
        }
        let resolution = &receipt["evidence_resolution"];
        assert_eq!(resolution["closure_complete"], true);
        assert!(rows(&resolution["unresolved"])?.is_empty());
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            for row in rows(&resolution[actual])? {
                assert!(rows(&expected.records[retained])?.contains(row));
            }
        }
    }
    assert!(pending.is_empty());
    Ok(())
}

#[test]
fn retained_native_state_root_l0_preserves_toc_scopes_paths_and_source_absent_custody() -> TestResult
{
    use crate::context::ContextL0Profile;
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let input = scratch.root.join("state-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(state_consumer_bundle(&input, &stop)?, &stop)?;
    let store_root = scratch.root.join("registered-store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(
        &publisher,
        &graph,
        "fixture-context-state-root",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    std::fs::remove_dir_all(&input)?;
    assert!(!input.exists());
    let current = retained_state_consumer(&store_root, &ReadSelector::Current, &stop)?;
    let expected = retained_state_consumer(
        &store_root,
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    assert_eq!(current.current, expected.current);
    assert_eq!(current.records, expected.records);
    assert_eq!(current.root_ids, expected.root_ids);
    let mut host_json = host_value(&store_root, &profile, &reference);
    // One finite caller fixture selection, shared by inspect/build/JSON render.
    host_json["limits"]["max_context_work"] = json!(8_000_000);
    host_json["limits"]["max_metadata_bytes"] = json!(4 * 1024 * 1024);
    let host = decode_host(&host_json, &stop)?;
    let mut selected_requests = Vec::new();
    let mut selected_results = Vec::new();
    for (root, selected) in expected
        .root_ids
        .iter()
        .zip(rows(&expected.records["roots"])?)
    {
        let default = exact_inspect_request(
            &publication.generation_id,
            ContextInspectLevel::L0,
            ContextMapRoot::graph_entity(root.as_str())?,
            &profile,
            &reference,
        )?;
        assert_eq!(
            wow_core::canonical_json_bytes(&default)?,
            wow_core::canonical_json_bytes(
                &default
                    .clone()
                    .with_l0_profile(ContextL0Profile::ContainersV1)?
            )?
        );
        assert_inspect_refused(
            &host,
            &default,
            ServiceErrorCode::InvalidRequest,
            "closed",
            &stop,
        )?;
        let request = default.with_l0_profile(ContextL0Profile::StateRootV1)?;
        let result = execute_context_inspect(&host, &request, &stop)?;
        assert_eq!(result.status(), ContextMapStatus::Partial);
        assert_eq!(result.failure_code(), None);
        let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
        assert_eq!(envelope["resource_closure"], "closed");
        assert_eq!(
            envelope["payload"]["publication_set_id"],
            expected.records["publication_set_id"]
        );
        assert_state_consumer_l0(&envelope["payload"]["skeletons"]["l0"], selected, &expected)?;
        assert!(envelope["payload"]["skeletons"].get("l1").is_none());
        let text = std::str::from_utf8(result.canonical_bytes())?;
        for body in [STATE_LUA, STATE_TOC] {
            assert!(!text.contains(&serde_json::to_string(body)?));
        }
        assert!(!text.contains("StateCoreAccountDB = {}"));
        assert!(!text.contains("project_store_root"));
        assert!(
            !text.contains(
                scratch
                    .root
                    .file_name()
                    .ok_or("scratch name")?
                    .to_str()
                    .ok_or("UTF-8")?
            )
        );
        selected_requests.push(request);
        selected_results.push(result);
    }
    let request = &selected_requests[0];
    let result = &selected_results[0];
    assert_eq!(
        result.canonical_bytes(),
        execute_context_inspect(&host, request, &stop)?.canonical_bytes()
    );
    let unrelated = exact_inspect_request(
        &publication.generation_id,
        ContextInspectLevel::L0,
        ContextMapRoot::graph_entity(expected.unrelated.as_str())?,
        &profile,
        &reference,
    )?
    .with_l0_profile(ContextL0Profile::StateRootV1)?;
    assert_inspect_refused(
        &host,
        &unrelated,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    for field in ["max_context_work", "max_metadata_bytes"] {
        let mut low = host_json.clone();
        low["limits"][field] = json!(1);
        assert_inspect_refused(
            &decode_host(&low, &stop)?,
            request,
            ServiceErrorCode::BudgetExceeded,
            "closed",
            &stop,
        )?;
    }
    assert_inspect_refused(
        &host,
        request,
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;
    let build = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L0,
        ContextMapRoot::graph_entity(expected.root_ids[0].as_str())?,
        ContextRenderSelection::Json,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_l0_profile(ContextL0Profile::StateRootV1)?;
    let built = execute_context_build(&host, &build, &stop)?;
    assert_eq!(built.status(), ContextMapStatus::Partial);
    assert_eq!(built.failure_code(), None);
    let envelope: Value = serde_json::from_slice(built.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    let payload = &envelope["payload"];
    let pack = &payload["semantic_pack"];
    assert_state_consumer_l0(
        &pack["core"]["skeletons"]["l0"],
        &expected.records["roots"][0],
        &expected,
    )?;
    assert_eq!(pack["core"]["absence_authoritative"], false);
    let artifacts = rows(&payload["rendered_artifacts"])?;
    assert_eq!(artifacts.len(), 1);
    assert_eq!(
        artifacts[0]["core"]["profile"]["profile_id"],
        PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID
    );
    let content = artifacts[0]["core"]["content"]
        .as_str()
        .ok_or("JSON renderer content")?;
    assert_eq!(serde_json::from_str::<Value>(content)?, *pack);
    assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
    if scratch.export {
        let common = json!([
            "--native-host-config",
            scratch.root.join("host.json"),
            "--project-store",
            STORE_ID,
            "--publication",
            format!("store-generation:{}", publication.generation_id.as_str()),
            "--root",
            root_token(&expected.root_ids[0]),
            "--level",
            "l0",
            "--l0-profile-id",
            STATE_L0_PROFILE,
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
        let mut inspect_args = vec![json!("context"), json!("inspect")];
        inspect_args.extend(rows(&common)?.iter().cloned());
        export_native_fixture(
            &scratch,
            &host_json,
            request,
            &json!(inspect_args),
            result,
            "synthetic ordinary selected-TOC state-root L0; source-absent retained Current/Exact",
        )?;
        let mut build_args = vec![json!("context"), json!("build")];
        build_args.extend(rows(&common)?.iter().cloned());
        build_args.extend([
            json!("--context-profile-id"),
            json!(PRIMARY_CONTEXT_PACK_PROFILE_ID),
            json!("--renderer-profile-id"),
            json!(PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID),
        ]);
        for (name, bytes, limit) in [
            (
                "build-request.json",
                wow_core::canonical_json_bytes(&build)?,
                64 * 1024,
            ),
            (
                "build-cli-arguments.json",
                wow_core::canonical_json_bytes(&json!(build_args))?,
                64 * 1024,
            ),
            (
                "expected-build-envelope.json",
                built.canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
        ] {
            write_export(&scratch.root, name, &bytes, limit)?;
        }
    }
    assert!(!input.exists());
    Ok(())
}

const FRONTIER_PROFILE: &str = "wow-context/primary-native-frontier/1";

struct StateFrontierExpected {
    initial: Value,
    frontier: Value,
    items: Vec<Value>,
    added_entities: usize,
    added_relations: usize,
}

fn retained_state_frontier_views(
    store_root: &Path,
    generation: &wow_store::project::StoreGenerationId,
    initial: &GraphNodeId,
    frontier: &GraphNodeId,
    host: &ContextMapHostInput,
    stop: &AtomicBool,
) -> TestResult<StateFrontierExpected> {
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
    let universe = wow_context::bind_context_universe_set(
        read.pair(),
        &host.reference,
        &host.reference_profile,
        host.limits.owner_limits()?,
        stop,
    )?;
    let graph = read.pair().graph_read(stop)?;
    let initial_view = graph.neighbors(
        &wow_context::primary_detail_query(&universe, initial.clone(), stop)?,
        stop,
    )?;
    let frontier_view = graph.neighbors(
        &wow_context::primary_detail_query(&universe, frontier.clone(), stop)?,
        stop,
    )?;
    let mut items = BTreeMap::<(String, String), Value>::new();
    for (reason, view) in [
        ("initial_selection", &initial_view),
        ("frontier_selection", &frontier_view),
    ] {
        let root_id = view.root().node_id();
        for node in std::iter::once(view.root()).chain(view.adjacent_nodes().iter().copied()) {
            let row = items.entry(("entity".into(),node.node_id().to_string())).or_insert_with(|| {
                json!({"kind":"entity","node":node,"previously_selected":false,"reasons":[]})
            });
            if reason == "initial_selection" {
                row["previously_selected"] = json!(true);
            }
            row["reasons"]
                .as_array_mut()
                .ok_or("native reasons")?
                .push(json!({"kind":reason,"root_id":root_id}));
        }
        for edge in view.edges() {
            let row = items.entry(("relation".into(),edge.edge_id().to_string())).or_insert_with(|| {
                json!({"kind":"relation","edge":edge,"previously_selected":false,"reasons":[]})
            });
            if reason == "initial_selection" {
                row["previously_selected"] = json!(true);
            }
            row["reasons"]
                .as_array_mut()
                .ok_or("native reasons")?
                .push(json!({"kind":reason,"root_id":root_id}));
        }
    }
    let added = |kind: &str| {
        items
            .values()
            .filter(|row| row["kind"] == kind && row["previously_selected"] == false)
            .count()
    };
    let expected = StateFrontierExpected {
        initial: serde_json::to_value(&initial_view)?,
        frontier: serde_json::to_value(&frontier_view)?,
        added_entities: added("entity"),
        added_relations: added("relation"),
        items: items.into_values().collect(),
    };
    read.close()?;
    store.close()?;
    Ok(expected)
}

fn assert_state_frontier_expansion(
    result: &ContextMapResult,
    initial_payload: &Value,
    native: &StateConsumerExpected,
    expected: &StateFrontierExpected,
    state: ContextMapStatus,
) -> TestResult<Value> {
    assert_eq!(result.status(), state);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], "wow-service/context-inspect-result/1");
    assert_eq!(envelope["resource_closure"], "closed");
    let payload = &envelope["payload"];
    assert_eq!(
        payload["publication_set_id"],
        native.records["publication_set_id"]
    );
    assert_eq!(
        payload["skeletons"], initial_payload["skeletons"],
        "expansion retains the original initial L1 artifact"
    );
    let expansion = &payload["expansion"];
    assert!(
        expansion["expansion_id"]
            .as_str()
            .ok_or("sealed expansion ID")?
            .len()
            > 32
    );
    let core = &expansion["core"];
    assert_eq!(core["schema"], "wow-context/primary-frontier-expansion/1");
    assert_eq!(core["profile"]["profile_id"], FRONTIER_PROFILE);
    let l1 = &initial_payload["skeletons"]["l1"];
    assert_eq!(core["initial_l1_id"], l1["l1_skeleton_id"]);
    assert_eq!(
        core["initial_map_id"],
        l1["core"]["context"]["origins"]["project_map_id"]
    );
    assert_eq!(core["root"], expected.initial["root"]);
    exact_rows(&core["frontier"], &[expected.frontier["root"].clone()])?;
    assert_eq!(core["added_entity_count"], expected.added_entities);
    assert_eq!(core["added_relation_count"], expected.added_relations);
    assert_eq!(core["state"], serde_json::to_value(state)?);
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(core["stop_reason"], "one_round_profile_bound");
    assert_eq!(core["continuation"], "not_evaluated");
    let actual = rows(&core["items"])?;
    assert_eq!(actual.len(), expected.items.len());
    let mut pending = BTreeSet::new();
    for wanted in &expected.items {
        let (kind, field, id_field) = match wanted["kind"].as_str() {
            Some("entity") => ("entity", "node", "node_id"),
            Some("relation") => ("relation", "edge", "edge_id"),
            _ => return Err("unexpected native item kind".into()),
        };
        let id = wanted[field][id_field].as_str().ok_or("native item ID")?;
        let matches = actual
            .iter()
            .filter(|row| row["kind"] == kind && row[field][id_field] == id)
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "dedup is the exact original node/edge identity"
        );
        let row = matches[0];
        assert_eq!(
            row[field], wanted[field],
            "no synthesized or transitive graph record"
        );
        assert_eq!(row["previously_selected"], wanted["previously_selected"]);
        exact_rows(&row["reasons"], rows(&wanted["reasons"])?)?;
        assert!(pending.insert((kind.to_owned(), id.to_owned())));
    }
    let origins = &core["origins"]["core"];
    assert_eq!(
        origins["schema"],
        "wow-context/project-map/primary-frontier-origins/1"
    );
    assert_eq!(origins["state"], serde_json::to_value(state)?);
    assert_eq!(origins["absence_authoritative"], false);
    assert_eq!(
        origins["profile"]["profile"],
        "wow-context/primary-frontier-origins/1"
    );
    assert_eq!(
        origins["navigation"]["initial_map_id"],
        core["initial_map_id"]
    );
    exact_rows(
        &origins["navigation"]["neighborhoods"],
        std::slice::from_ref(&expected.frontier),
    )?;
    assert!(
        core["budget"]["work_reserved"]
            .as_u64()
            .ok_or("continued expansion work")?
            > initial_payload["skeletons"]["work_reserved"]
                .as_u64()
                .ok_or("initial work")?
    );
    assert_eq!(
        core["budget"]["reservation_basis"],
        "before_expansion_finalization"
    );
    assert_eq!(
        origins["load_custody"]["selected_toc"],
        native.records["load"]
    );
    for receipt in rows(&origins["supports"])? {
        let explanation = &receipt["explanation"];
        assert_eq!(explanation["support_complete"], true);
        assert_eq!(
            explanation["source_context_id"],
            native.records["source_context_id"]
        );
        let record = &explanation["record"];
        let (kind, field, id_field) = match record["kind"].as_str() {
            Some("entity") => ("entity", "node", "node_id"),
            Some("relation") => ("relation", "edge", "edge_id"),
            _ => return Err("unexpected explained item".into()),
        };
        let id = record[field][id_field]
            .as_str()
            .ok_or("explained native ID")?;
        assert!(pending.remove(&(kind.to_owned(), id.to_owned())));
        assert!(
            expected
                .items
                .iter()
                .any(|row| row["kind"] == kind && row[field] == record[field])
        );
        assert!(!rows(&explanation["supports"])?.is_empty());
        for support in rows(&explanation["supports"])? {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("unexpected foundation support".into()),
            };
            assert!(rows(&native.records[proposals])?.contains(&support["proposal"]));
            assert!(rows(&native.records[accepted])?.contains(&support["accepted"]));
        }
        let resolution = &receipt["evidence_resolution"];
        assert_eq!(resolution["closure_complete"], true);
        assert!(rows(&resolution["unresolved"])?.is_empty());
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            for row in rows(&resolution[actual])? {
                assert!(rows(&native.records[retained])?.contains(row));
            }
        }
    }
    assert!(
        pending.is_empty(),
        "every native selected record retains original support"
    );
    for row in rows(&origins["source_custody"])? {
        assert_eq!(row["scope"], "primary");
        assert!(rows(&native.records["handles"])?.contains(&row["handle"]));
    }
    let text = std::str::from_utf8(result.canonical_bytes())?;
    assert!(!text.contains("project_store_root"));
    assert!(!text.contains("StateCoreAccountDB = {}"));
    for body in [STATE_TOC, STATE_LUA] {
        assert!(!text.contains(&serde_json::to_string(body)?));
    }
    Ok(payload.clone())
}

const FRONTIER_PACK_PROFILE: &str = "wow-context/primary-frontier-pack/1";
const FRONTIER_JSON_PROFILE: &str = "wow-context/primary-frontier-pack-json/1";
const FRONTIER_MARKDOWN_PROFILE: &str = "wow-context/primary-frontier-pack-markdown/1";

fn assert_frontier_pack(
    result: &ContextMapResult,
    inspected: &Value,
    native: &StateConsumerExpected,
    scratch: &Scratch,
) -> TestResult<Value> {
    assert_eq!(
        result.status(),
        ContextMapStatus::Partial,
        "selected build refusal: {:?}",
        result.failure_code()
    );
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], "wow-service/context-build-result/1");
    assert_eq!(envelope["operation"], "context_build");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(
        payload["publication_set_id"],
        native.records["publication_set_id"]
    );
    assert_eq!(
        payload["store_generation_id"],
        inspected["store_generation_id"]
    );
    assert_eq!(
        payload["current_at_acquisition"],
        serde_json::to_value(&native.current)?
    );
    assert_eq!(
        payload["current_record_digest"],
        observed_current_digest(&native.current)?
    );
    let pack = &payload["semantic_pack"];
    assert_eq!(
        payload["semantic_pack_byte_length"],
        wow_core::canonical_json_bytes(pack)?.len()
    );
    let core = &pack["core"];
    assert_eq!(
        core["schema"],
        "wow-context/semantic-pack/primary-frontier/1"
    );
    assert_eq!(core["profile"]["profile_id"], FRONTIER_PACK_PROFILE);
    assert_eq!(core["profile"]["source_bodies"], "excluded");
    assert_eq!(core["state"], "partial");
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(core["skeletons"], inspected["skeletons"]);
    assert_eq!(
        core["expansion"], inspected["expansion"],
        "exact original Step, including all reasons, native support and omissions"
    );
    let expansion = &core["expansion"];
    assert_eq!(
        core["request"]["core"]["expansion_id"],
        expansion["expansion_id"]
    );
    let origins = &expansion["core"]["origins"];
    assert_eq!(core["universe_set_id"], origins["core"]["universe_set_id"]);
    let selected = rows(&core["items"])?;
    let original = rows(&expansion["core"]["items"])?;
    let trace = rows(&core["selection_trace"])?;
    assert_eq!(selected.len(), original.len());
    assert_eq!(trace.len(), selected.len());
    let mut seen = BTreeSet::new();
    for (index, (item, selection)) in selected.iter().zip(trace).enumerate() {
        let item_core = &item["core"];
        let binding = json!({"expansion_id":expansion["expansion_id"],"item_index":index});
        assert_eq!(
            item_core["schema"],
            "wow-context/context-item/primary-frontier/1"
        );
        assert_eq!(item_core["expansion"], binding);
        assert_eq!(selection["expansion"], binding);
        assert_eq!(item_core["origin_map_id"], origins["project_map_id"]);
        assert_eq!(selection["origin_map_id"], origins["project_map_id"]);
        assert_eq!(selection["item_id"], item["context_item_id"]);
        assert_eq!(selection["reason"], item_core["inclusion_reason"]);
        assert_eq!(selection["required"], true);
        assert_eq!(item_core["root"], core["request"]["core"]["root"]);
        assert_eq!(item_core["universe_set_id"], core["universe_set_id"]);
        let row = &original[index];
        let expected_payload = match row["kind"].as_str() {
            Some("entity") => json!({"kind":"entity","node":row["node"]}),
            Some("relation") => json!({"kind":"relation","edge":row["edge"]}),
            _ => return Err("non-native expansion item".into()),
        };
        assert_eq!(item_core["payload"], expected_payload);
        assert_eq!(
            item_core["payload_digest"],
            ContentDigest::<CanonicalResult>::from_bytes(wow_core::domain_separated_digest(
                "wow-context/context-item-payload/1",
                &expected_payload
            )?)
            .to_string()
        );
        assert!(seen.insert(item["context_item_id"].as_str().ok_or("item ID")?));
        assert!(!rows(&row["reasons"])?.is_empty());
        assert!(row["previously_selected"].is_boolean());
    }
    assert_eq!(
        core["budget"]["reservation_basis"],
        "before_semantic_pack_finalization"
    );
    assert_eq!(
        payload["operation_budget_usage"]["reservation_basis"],
        "before_service_delivery"
    );
    for resource in ["work_reserved", "bytes_reserved"] {
        assert!(
            core["budget"][resource]
                .as_u64()
                .ok_or("pack reservation")?
                > expansion["core"]["budget"][resource]
                    .as_u64()
                    .ok_or("Step reservation")?
        );
        assert!(
            payload["operation_budget_usage"][resource]
                .as_u64()
                .ok_or("delivery usage")?
                >= core["budget"][resource]
                    .as_u64()
                    .ok_or("pack reservation")?
        );
    }
    let text = std::str::from_utf8(result.canonical_bytes())?;
    let root = serde_json::to_string(&scratch.root.to_string_lossy())?;
    assert!(!text.contains(root.trim_matches('"')));
    assert!(!text.contains("project_store_root"));
    for body in [STATE_TOC, STATE_LUA] {
        assert!(!text.contains(&serde_json::to_string(body)?));
    }
    Ok(payload.clone())
}

fn assert_frontier_render(artifact: &Value, pack: &Value, markdown: bool) -> TestResult {
    let core = &artifact["core"];
    let content = core["content"].as_str().ok_or("rendered content")?;
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
    assert_eq!(core["profile"]["semantic_schema"], pack["core"]["schema"]);
    assert_eq!(
        core["profile"]["profile_id"],
        if markdown {
            FRONTIER_MARKDOWN_PROFILE
        } else {
            FRONTIER_JSON_PROFILE
        }
    );
    let trace = rows(&core["trace"])?;
    let mut end = 0usize;
    for record in trace {
        let start = usize::try_from(record["byte_start"].as_u64().ok_or("range start")?)?;
        let next = usize::try_from(record["byte_end"].as_u64().ok_or("range end")?)?;
        assert_eq!(start, end);
        assert!(next > start);
        content.get(start..next).ok_or("UTF-8 record boundary")?;
        end = next;
    }
    assert_eq!(end, content.len());
    if !markdown {
        assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
        assert_eq!(serde_json::from_str::<Value>(content)?, *pack);
        assert_eq!(
            core["trace"],
            json!([{
                "record_id":pack["semantic_pack_id"], "template_id":"canonical_pack_json",
                "semantic_fields":["semantic_pack_id","core"], "byte_start":0, "byte_end":content.len()
            }])
        );
        return Ok(());
    }
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
        (
            "native_frontier_expansion",
            "EXPANSION",
            vec!["core.expansion"],
            None,
        ),
    ];
    for (index, item) in rows(&pack["core"]["items"])?.iter().enumerate() {
        let (template, label) = match item["core"]["payload"]["kind"].as_str() {
            Some("entity") => ("native_entity_item", "ENTITY"),
            Some("relation") => ("native_relation_item", "RELATION"),
            _ => return Err("non-native pack item".into()),
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
    assert_eq!(trace.len(), specifications.len());
    let mut reconstructed = json!({"semantic_pack_id":"","core":{"items":[]}});
    for (record, (template, label, fields, item_index)) in trace.iter().zip(specifications) {
        assert_eq!(record["template_id"], template);
        assert_eq!(record["semantic_fields"], json!(fields));
        assert_eq!(
            record.get("item_index"),
            item_index.map(|index| json!(index)).as_ref()
        );
        let expected_id = if template == "native_frontier_expansion" {
            &pack["core"]["expansion"]["expansion_id"]
        } else {
            item_index.map_or(&pack["semantic_pack_id"], |index| {
                &pack["core"]["items"][index]["context_item_id"]
            })
        };
        assert_eq!(&record["record_id"], expected_id);
        let start = usize::try_from(record["byte_start"].as_u64().ok_or("record start")?)?;
        let end = usize::try_from(record["byte_end"].as_u64().ok_or("record end")?)?;
        let line = content.get(start..end).ok_or("complete Markdown record")?;
        assert_eq!(line.bytes().filter(|byte| *byte == b'\n').count(), 1);
        let quoted = line
            .strip_prefix(label)
            .and_then(|rest| rest.strip_prefix(' '))
            .and_then(|rest| rest.strip_suffix('\n'))
            .ok_or("fixed Markdown framing")?;
        let inner: String = serde_json::from_str(quoted)?;
        let value: Value = serde_json::from_str(&inner)?;
        // Decoding verifies presentation only; all authority is retained native custody.
        let rebuilt = &mut reconstructed["core"];
        match template {
            "pack_header" => {
                assert_eq!(value.as_object().ok_or("pack header")?.len(), 3);
                assert_eq!(value["item_count"], rows(&pack["core"]["items"])?.len());
                rebuilt["schema"] = value["schema"].clone();
                reconstructed["semantic_pack_id"] = value["semantic_pack_id"].clone();
            }
            "pack_profile" => rebuilt["profile"] = value,
            "universe_binding" => {
                assert_eq!(rows(&value)?.len(), 2);
                rebuilt["universe_set_id"] = value[0].clone();
                rebuilt["binding"] = value[1].clone();
            }
            "pack_request" => rebuilt["request"] = value,
            "boundary_notice" => rebuilt["boundary_notice"] = value,
            "native_skeletons" => rebuilt["skeletons"] = value,
            "native_frontier_expansion" => rebuilt["expansion"] = value,
            "native_entity_item" | "native_relation_item" => {
                assert_eq!(
                    rows(&rebuilt["items"])?.len(),
                    item_index.ok_or("item index")?
                );
                rebuilt["items"]
                    .as_array_mut()
                    .ok_or("reconstructed items")?
                    .push(value);
            }
            "selection_trace" => rebuilt["selection_trace"] = value,
            "coverage_and_omissions" => {
                assert_eq!(rows(&value)?.len(), 3);
                rebuilt["omissions"] = value[0].clone();
                rebuilt["state"] = value[1].clone();
                rebuilt["absence_authoritative"] = value[2].clone();
            }
            "pack_budget" => rebuilt["budget"] = value,
            _ => return Err("unknown Markdown record".into()),
        }
    }
    assert_eq!(
        reconstructed, *pack,
        "including the exact original expansion Step"
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&reconstructed)?,
        wow_core::canonical_json_bytes(pack)?
    );
    Ok(())
}

fn assert_frontier_build_refused(
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
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], "wow-service/context-build-result/1");
    assert_eq!(envelope["resource_closure"], closure);
    assert_eq!(envelope["failure_code"], serde_json::to_value(code)?);
    assert!(
        envelope.get("payload").is_none(),
        "no pack or renderer draft escapes"
    );
    Ok(())
}

#[test]
fn retained_native_state_path_frontier_adds_exact_siblings_under_one_inspect_budget() -> TestResult
{
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let input = scratch.root.join("state-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(state_consumer_bundle(&input, &stop)?, &stop)?;
    let store_root = scratch.root.join("registered-store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(&publisher, &graph, "fixture-context-frontier", None, &stop)?;
    store.close()?;
    drop(graph);
    drop(publisher);
    std::fs::remove_dir_all(&input)?;
    assert!(!input.exists());
    let native = retained_state_consumer(
        &store_root,
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    let current = retained_state_consumer(&store_root, &ReadSelector::Current, &stop)?;
    assert_eq!(native.records, current.records);
    assert_eq!(native.current, current.current);
    let roots = rows(&native.records["roots"])?;
    let account = roots
        .iter()
        .find(|root| root["origin"]["scope"] == "account")
        .ok_or("explicit account fixture root")?;
    let character = roots
        .iter()
        .find(|root| root["origin"]["scope"] == "character")
        .ok_or("explicit character fixture root")?;
    let profile_path = rows(&account["paths"])?
        .iter()
        .find(|path| path["origin"]["keys"] == json!([{"kind":"string","value":"profile"}]))
        .ok_or("exact native profile path")?;
    let initial: GraphNodeId = serde_json::from_value(profile_path["node"]["node_id"].clone())?;
    let frontier: GraphNodeId = serde_json::from_value(account["node"]["node_id"].clone())?;
    let outside: GraphNodeId = serde_json::from_value(character["node"]["node_id"].clone())?;
    let mut host_json = host_value(&store_root, &profile, &reference);
    host_json["limits"]["max_context_work"] = json!(32_000_000);
    host_json["limits"]["max_metadata_bytes"] = json!(8 * 1024 * 1024);
    let host = decode_host(&host_json, &stop)?;
    let baseline_request = exact_inspect_request(
        &publication.generation_id,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(initial.as_str())?,
        &profile,
        &reference,
    )?;
    let baseline = execute_context_inspect(&host, &baseline_request, &stop)?;
    assert_eq!(baseline.status(), ContextMapStatus::Partial);
    assert_eq!(baseline.failure_code(), None);
    let baseline_envelope: Value = serde_json::from_slice(baseline.canonical_bytes())?;
    let baseline_payload = &baseline_envelope["payload"];
    assert!(baseline_payload.get("expansion").is_none());
    let expected = retained_state_frontier_views(
        &store_root,
        &publication.generation_id,
        &initial,
        &frontier,
        &host,
        &stop,
    )?;
    assert!(
        rows(&baseline_payload["skeletons"]["l1"]["core"]["local_entities"])?
            .contains(&account["node"]),
        "frontier already belongs to the genuine initial map"
    );
    let request = baseline_request
        .clone()
        .with_frontier_expansion(vec![ContextMapRoot::graph_entity(frontier.as_str())?])?;
    let result = execute_context_inspect(&host, &request, &stop)?;
    let payload = assert_state_frontier_expansion(
        &result,
        baseline_payload,
        &native,
        &expected,
        ContextMapStatus::Partial,
    )?;
    let held_bytes = payload["expansion"]["core"]["budget"]["bytes_reserved"]
        .as_u64()
        .ok_or("actual expansion reservation")?
        .checked_add(u64::try_from(
            wow_core::canonical_json_bytes(&payload["expansion"])?.len(),
        )?)
        .ok_or("expansion reservation overflow")?;
    let envelope_bytes = u64::try_from(result.canonical_bytes().len())?;
    assert!(
        held_bytes + envelope_bytes
            <= host_json["limits"]["max_metadata_bytes"]
                .as_u64()
                .ok_or("selected metadata bound")?
    );
    // Each retained stage and the envelope fit independently; their sum must
    // still refuse when the final consumer would exceed the continuing ledger.
    let envelope_limit = held_bytes + envelope_bytes / 2;
    assert!(envelope_limit > held_bytes && envelope_limit > envelope_bytes);
    let mut envelope_bound = host_json.clone();
    envelope_bound["limits"]["max_metadata_bytes"] = json!(envelope_limit);
    assert_inspect_refused(
        &decode_host(&envelope_bound, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    assert_eq!(
        result.canonical_bytes(),
        execute_context_inspect(&host, &request, &stop)?.canonical_bytes()
    );
    assert!(expected.added_entities > 0 && expected.added_relations > 0);
    let items = rows(&payload["expansion"]["core"]["items"])?;
    for sibling in rows(&account["paths"])? {
        if sibling["node"] != profile_path["node"] {
            let item = items
                .iter()
                .find(|row| row["kind"] == "entity" && row["node"] == sibling["node"])
                .ok_or("exact newly admitted sibling")?;
            assert_eq!(item["previously_selected"], false);
            exact_rows(
                &item["reasons"],
                &[json!({"kind":"frontier_selection","root_id":frontier})],
            )?;
        }
    }
    assert!(
        !items
            .iter()
            .any(|row| row["kind"] == "entity" && row["node"] == character["node"]),
        "one round does not expand the new TOC-file neighbor"
    );
    // Continue this same source-absent native lifecycle through the selected
    // pack. Identical finite caller limits apply to all renderer selections.
    let mut build_requests = Vec::new();
    let mut build_results = Vec::new();
    let mut build_payloads = Vec::new();
    for render in [
        ContextRenderSelection::None,
        ContextRenderSelection::Json,
        ContextRenderSelection::Markdown,
        ContextRenderSelection::Both,
    ] {
        let build_request = ContextBuildRequest::primary(
            STORE_ID,
            ContextMapPublication::store_generation(publication.generation_id.as_str())?,
            ContextInspectLevel::L1,
            ContextMapRoot::graph_entity(initial.as_str())?,
            render,
            None,
            Some(reference_guard(&profile, &reference)?),
        )?
        .with_frontier_expansion(vec![ContextMapRoot::graph_entity(frontier.as_str())?])?;
        let built = execute_context_build(&host, &build_request, &stop)?;
        let built_payload = assert_frontier_pack(&built, &payload, &native, &scratch)?;
        eprintln!(
            "frontier-pack fixture {render:?}: pack_bytes={} work={} bytes={}",
            built_payload["semantic_pack_byte_length"],
            built_payload["operation_budget_usage"]["work_reserved"],
            built_payload["operation_budget_usage"]["bytes_reserved"]
        );
        build_requests.push(build_request);
        build_results.push(built);
        build_payloads.push(built_payload);
    }
    let pack = &build_payloads[0]["semantic_pack"];
    let pack_bytes = wow_core::canonical_json_bytes(pack)?;
    for built in &build_payloads {
        assert_eq!(
            built["semantic_pack"]["semantic_pack_id"],
            pack["semantic_pack_id"]
        );
        assert_eq!(
            wow_core::canonical_json_bytes(&built["semantic_pack"])?,
            pack_bytes
        );
    }
    assert!(rows(&build_payloads[0]["rendered_artifacts"])?.is_empty());
    let json_only = rows(&build_payloads[1]["rendered_artifacts"])?;
    let markdown_only = rows(&build_payloads[2]["rendered_artifacts"])?;
    let both = rows(&build_payloads[3]["rendered_artifacts"])?;
    assert_eq!(json_only.len(), 1);
    assert_eq!(markdown_only.len(), 1);
    assert_eq!(both.len(), 2);
    assert_frontier_render(&json_only[0], pack, false)?;
    assert_frontier_render(&markdown_only[0], pack, true)?;
    assert_eq!(
        both,
        &[json_only[0].clone(), markdown_only[0].clone()],
        "fixed JSON then Markdown; renderer identity excludes continuing usage"
    );
    assert_eq!(
        execute_context_build(&host, &build_requests[3], &stop)?.canonical_bytes(),
        build_results[3].canonical_bytes()
    );

    // Use only this fixture's measured usage. One single-render operation fits
    // its actual selected work bound, while Both must continue that ledger.
    let single_work = build_payloads[..3]
        .iter()
        .enumerate()
        .map(|(index, built)| {
            built["operation_budget_usage"]["work_reserved"]
                .as_u64()
                .map(|work| (index, work))
                .ok_or("measured single work")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (single_index, single_work) = single_work
        .into_iter()
        .max_by_key(|(_, work)| *work)
        .ok_or("single work observations")?;
    let both_work = build_payloads[3]["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("Both work")?;
    assert!(single_work < both_work);
    // Keep a measured margin for serialized limit-width changes instead of
    // assuming the smaller request has identical byte/work reservations.
    let single_limit = single_work + (both_work - single_work) / 2;
    let mut shared_work = host_json.clone();
    shared_work["limits"]["max_context_work"] = json!(single_limit);
    let shared_work = decode_host(&shared_work, &stop)?;
    let fits = execute_context_build(&shared_work, &build_requests[single_index], &stop)?;
    assert_eq!(fits.failure_code(), None);
    assert_eq!(fits.status(), ContextMapStatus::Partial);
    let fits_envelope: Value = serde_json::from_slice(fits.canonical_bytes())?;
    assert_eq!(fits_envelope["resource_closure"], "closed");
    assert!(
        fits_envelope["payload"]["operation_budget_usage"]["work_reserved"]
            .as_u64()
            .ok_or("selected single work")?
            <= single_limit
    );
    assert_frontier_build_refused(
        &shared_work,
        &build_requests[3],
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;

    // All owner/render reservations and the final envelope each fit, but their
    // cumulative sum must refuse at delivery without exposing a partial draft.
    let built_usage = build_payloads[3]["operation_budget_usage"]["bytes_reserved"]
        .as_u64()
        .ok_or("actual post-render reservation")?;
    let delivery_bytes = u64::try_from(build_results[3].canonical_bytes().len())?;
    let selected_bytes = host_json["limits"]["max_metadata_bytes"]
        .as_u64()
        .ok_or("selected byte bound")?;
    assert!(
        built_usage
            .checked_add(delivery_bytes)
            .ok_or("delivery sum")?
            <= selected_bytes
    );
    let delivery_limit = built_usage
        .checked_add(delivery_bytes / 2)
        .ok_or("delivery bound")?;
    assert!(delivery_limit > built_usage && delivery_limit > delivery_bytes);
    let mut shared_bytes = host_json.clone();
    shared_bytes["limits"]["max_metadata_bytes"] = json!(delivery_limit);
    assert_frontier_build_refused(
        &decode_host(&shared_bytes, &stop)?,
        &build_requests[3],
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    assert_frontier_build_refused(
        &host,
        &build_requests[3],
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;
    assert!(!input.exists());

    for frontier_selection in [
        Vec::new(),
        vec![
            ContextMapRoot::graph_entity(frontier.as_str())?,
            ContextMapRoot::graph_entity(frontier.as_str())?,
        ],
        vec![ContextMapRoot::project("fixture-context-primary-map")?],
        vec![ContextMapRoot::graph_entity(initial.as_str())?],
    ] {
        assert_eq!(
            baseline_request
                .clone()
                .with_frontier_expansion(frontier_selection)
                .err()
                .ok_or("invalid frontier admitted")?
                .code(),
            ServiceErrorCode::InvalidRequest
        );
    }
    let outside_request = baseline_request
        .clone()
        .with_frontier_expansion(vec![ContextMapRoot::graph_entity(outside.as_str())?])?;
    assert_inspect_refused(
        &host,
        &outside_request,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    let work = baseline_payload["skeletons"]["work_reserved"]
        .as_u64()
        .ok_or("actual initial work")?;
    assert!(work > 0);
    let mut low = host_json.clone();
    low["limits"]["max_context_work"] = json!(work);
    let low_host = decode_host(&low, &stop)?;
    let fits = execute_context_inspect(&low_host, &baseline_request, &stop)?;
    assert_eq!(fits.status(), ContextMapStatus::Partial);
    assert_eq!(
        fits.failure_code(),
        None,
        "the actual initial operation fits this selected work bound"
    );
    assert_inspect_refused(
        &low_host,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    let mut low_bytes = host_json.clone();
    low_bytes["limits"]["max_metadata_bytes"] = json!(1);
    assert_inspect_refused(
        &decode_host(&low_bytes, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    assert_inspect_refused(
        &host,
        &request,
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;
    let after = retained_state_consumer(&store_root, &ReadSelector::Current, &stop)?;
    assert_eq!(after.current, native.current);
    assert_eq!(after.records, native.records);
    assert_eq!(
        baseline.canonical_bytes(),
        execute_context_inspect(&host, &baseline_request, &stop)?.canonical_bytes()
    );
    if scratch.export {
        let arguments = json!([
            "context",
            "inspect",
            "--native-host-config",
            scratch.root.join("host.json"),
            "--project-store",
            STORE_ID,
            "--publication",
            format!("store-generation:{}", publication.generation_id.as_str()),
            "--root",
            root_token(&initial),
            "--level",
            "l1",
            "--l1-profile-id",
            PRIMARY_L1_PROFILE_ID,
            "--expansion-profile-id",
            FRONTIER_PROFILE,
            "--frontier",
            root_token(&frontier),
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
            &request,
            &arguments,
            &result,
            "synthetic ordinary source-absent native state_path L1; one explicit account-root frontier",
        )?;
        write_export(
            &scratch.root,
            "initial-request.json",
            &wow_core::canonical_json_bytes(&baseline_request)?,
            64 * 1024,
        )?;
        write_export(
            &scratch.root,
            "initial-envelope.json",
            baseline.canonical_bytes(),
            CONTEXT_MAP_RESULT_MAX_BYTES,
        )?;
        let build_arguments = json!([
            "context",
            "build",
            "--native-host-config",
            scratch.root.join("host.json"),
            "--project-store",
            STORE_ID,
            "--publication",
            format!("store-generation:{}", publication.generation_id.as_str()),
            "--root",
            root_token(&initial),
            "--level",
            "l1",
            "--l1-profile-id",
            PRIMARY_L1_PROFILE_ID,
            "--context-profile-id",
            FRONTIER_PACK_PROFILE,
            "--renderer-profile-id",
            FRONTIER_JSON_PROFILE,
            "--renderer-profile-id",
            FRONTIER_MARKDOWN_PROFILE,
            "--expansion-profile-id",
            FRONTIER_PROFILE,
            "--frontier",
            root_token(&frontier),
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
        write_export(
            &scratch.root,
            "build-request.json",
            &wow_core::canonical_json_bytes(&build_requests[3])?,
            64 * 1024,
        )?;
        write_export(
            &scratch.root,
            "expected-build-envelope.json",
            build_results[3].canonical_bytes(),
            CONTEXT_MAP_RESULT_MAX_BYTES,
        )?;
        write_export(
            &scratch.root,
            "build-cli-arguments.json",
            &wow_core::canonical_json_bytes(&build_arguments)?,
            64 * 1024,
        )?;
    }
    assert!(!input.exists());
    Ok(())
}
