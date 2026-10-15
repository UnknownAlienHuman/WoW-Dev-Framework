//! Capture a real retained native prefix before implementing its durable issuer.
//! This checkpoint is input evidence, never a decoded continuation capability.
use super::*;
use wow_project::load::TocSavedVariableScope;

fn cancel_when_operation_completes(
    store_path: std::path::PathBuf,
    operation_id: wow_store::OperationId,
    stop: std::sync::Arc<AtomicBool>,
) -> std::thread::JoinHandle<Result<bool, String>> {
    std::thread::spawn(move || {
        let configuration = crate::context::continuation::continuation_store_configuration()
            .map_err(|error| format!("continuation store configuration: {error:?}"))?;
        let store = wow_store::Store::open(&store_path, configuration)
            .map_err(|error| format!("open continuation observer: {error:?}"))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while std::time::Instant::now() < deadline {
            let record = store
                .operation(&operation_id)
                .map_err(|error| format!("read observed continuation operation: {error:?}"))?;
            if record.is_some_and(|record| record.state() == wow_store::OperationState::Completed) {
                stop.store(true, std::sync::atomic::Ordering::Release);
                return Ok(true);
            }
            std::thread::yield_now();
        }
        Ok(false)
    })
}

#[test]
fn preimplementation_retained_progressive_seed() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let input = scratch.root.join("source-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(inspect::state_consumer_bundle(&input, &stop)?, &stop)?;
    let store_root = scratch.root.join("registered-store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(
        &publisher,
        &graph,
        "fixture-context-continuation",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    std::fs::remove_dir_all(&input)?;
    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    let native = read.pair().graph_read(&stop)?;
    let provenance = read.pair().source_provenance().ok_or("native provenance")?;
    let account = provenance
        .state_roots()
        .iter()
        .find(|root| root.scope == TocSavedVariableScope::Account)
        .ok_or("account declaration")?;
    let path = provenance
        .state_paths()
        .iter()
        .find(|path| {
            path.root_id == account.root_id
                && serde_json::to_value(&path.keys).ok()
                    == Some(json!([{"kind":"string","value":"profile"}]))
        })
        .ok_or("native profile path")?;
    let file = provenance
        .files()
        .iter()
        .find(|file| file.path == account.document)
        .ok_or("declaration document")?;
    let mut selected = Vec::new();
    for proposal in [&path.proposal_id, &account.proposal_id, &file.proposal_id] {
        selected.push(
            native
                .published_entity(SOURCE_GRAPH_PARTITION, proposal, &stop)?
                .node_id()
                .clone(),
        );
    }
    assert_eq!(selected.iter().collect::<BTreeSet<_>>().len(), 3);
    read.close()?;
    store.close()?;
    let mut host = host_value(&store_root, &profile, &reference);
    host["limits"]["max_context_work"] = json!(32_000_000);
    host["limits"]["max_metadata_bytes"] = json!(16 * 1024 * 1024);
    let request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![vec![ContextMapRoot::graph_entity(
        selected[1].as_str(),
    )?]])?;
    let result = execute_context_build(&decode_host(&host, &stop)?, &request, &stop)?;
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    let payload = &envelope["payload"];
    let pack = &payload["semantic_pack"];
    let expansion = &pack["core"]["expansion"];
    assert_eq!(expansion["core"]["progression"]["round"], 1);
    assert_eq!(payload["rendered_artifacts"], json!([]));
    let retained = payload["operation_budget_usage"]["bytes_reserved"]
        .as_u64()
        .ok_or("owner byte ledger")?;
    let delivery = u64::try_from(result.canonical_bytes().len())?;
    let complete = retained.checked_add(delivery).ok_or("delivery overflow")?;
    assert!(complete <= 16 * 1024 * 1024);
    // Budget reported in the artifact is earlier than the operation/delivery.
    // A future durable issuer must preserve the latter, not restore that snapshot.
    assert!(
        complete
            > pack["core"]["budget"]["bytes_reserved"]
                .as_u64()
                .ok_or("pack byte snapshot")?
    );
    let checkpoint = json!({
        "schema":"wow-context/native-continuation-input-fixture/1",
        "source_scope":"synthetic_native_source_absent_exact_retained",
        "state":"preimplementation_input_checkpoint_not_issued",
        "store_generation_id":publication.generation_id,
        "publication_set_id":payload["publication_set_id"],
        "universe_set_id":pack["core"]["universe_set_id"],
        "semantic_pack_id":pack["semantic_pack_id"],
        "expansion_id":expansion["expansion_id"],
        "initial_root":selected[0],"first_frontier":[selected[1]],"next_frontier":[selected[2]],
        "limits":host["limits"],
        "before_delivery_usage":payload["operation_budget_usage"],
        "service_envelope_bytes":delivery,"through_delivery_reserved_bytes":complete,
        "service_envelope_sha256":format!("{:x}",Sha256::digest(result.canonical_bytes())),
        "request":request
    });
    let bytes = wow_core::canonical_json_bytes(&checkpoint)?;
    eprintln!(
        "retained prefix pack={} expansion={} reservation={complete} envelope={delivery} checkpoint_sha256={:x}",
        pack["semantic_pack_id"],
        expansion["expansion_id"],
        Sha256::digest(&bytes)
    );
    if scratch.export {
        write_export(
            &scratch.root,
            "seed-input.json",
            &bytes,
            CONTEXT_MAP_RESULT_MAX_BYTES,
        )?;
        write_export(
            &scratch.root,
            "host.json",
            &wow_core::canonical_json_bytes(&host)?,
            1024 * 1024,
        )?;
        write_export(
            &scratch.root,
            "prefix-envelope.json",
            result.canonical_bytes(),
            CONTEXT_MAP_RESULT_MAX_BYTES,
        )?;
    }
    Ok(())
}

fn open_continuation_store(path: &Path) -> TestResult<wow_store::Store> {
    Ok(wow_store::Store::open(
        path,
        crate::context::continuation::continuation_store_configuration()?,
    )?)
}

fn operation_result_object_id(
    store: &wow_store::Store,
    operation_id: &wow_store::OperationId,
) -> TestResult<wow_store::ObjectId> {
    let operation = store
        .operation(operation_id)?
        .ok_or("completed operation")?;
    Ok(operation
        .result_object_id()
        .ok_or("operation result object")?
        .clone())
}

fn operation_response_object_id(
    store: &wow_store::Store,
    operation_id: &wow_store::OperationId,
) -> TestResult<wow_store::ObjectId> {
    let result_id = operation_result_object_id(store, operation_id)?;
    let object = store.object(&result_id)?.ok_or("operation result bytes")?;
    let value: Value = serde_json::from_slice(object.canonical_json())?;
    Ok(wow_store::ObjectId::new(
        value["response_object_id"]
            .as_str()
            .ok_or("response object ID")?
            .to_owned(),
    )?)
}

fn history_receipt_object_ids(
    store: &wow_store::Store,
    original_request_id: &str,
) -> TestResult<Vec<wow_store::ObjectId>> {
    let catalog = wow_store::CatalogName::new("wow-service.context-continuation")?;
    let path = wow_store::CatalogPath::new(format!("history/{original_request_id}"))?;
    let entry = store
        .catalog_entry(&catalog, &path)?
        .ok_or("continuation history root")?;
    let object = store.object(entry.object_id())?.ok_or("history object")?;
    let value: Value = serde_json::from_slice(object.canonical_json())?;
    value["receipts"]
        .as_array()
        .ok_or("history receipt roots")?
        .iter()
        .map(|receipt| {
            Ok(wow_store::ObjectId::new(
                receipt["receipt_object_id"]
                    .as_str()
                    .ok_or("receipt object ID")?
                    .to_owned(),
            )?)
        })
        .collect()
}

fn history_index_object_id(
    store: &wow_store::Store,
    original_request_id: &str,
) -> TestResult<wow_store::ObjectId> {
    let catalog = wow_store::CatalogName::new("wow-service.context-continuation")?;
    let path = wow_store::CatalogPath::new(format!("history/{original_request_id}"))?;
    Ok(store
        .catalog_entry(&catalog, &path)?
        .ok_or("continuation history root")?
        .object_id()
        .clone())
}

fn collect_continuation_garbage(path: &Path) -> TestResult<Vec<wow_store::ObjectId>> {
    let mut store = open_continuation_store(path)?;
    let receipt = store.collect_garbage(wow_store::LogicalEpoch::new(0), 10_000)?;
    Ok(receipt.deleted().to_vec())
}

#[test]
fn native_primary_continuation_replays_exact_round_after_current_moves() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let frozen: Value = serde_json::from_str(include_str!(
        "../../../e3/examples/context-continuation-native-input-freeze.json"
    ))?;
    let input = scratch.root.join("source-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(inspect::state_consumer_bundle(&input, &stop)?, &stop)?;
    let store_root = scratch.root.join("registered-store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(
        &publisher,
        &graph,
        "fixture-context-continuation",
        None,
        &stop,
    )?;
    // The historical checkpoint names a retained publication, not the result
    // of rebuilding the shared state fixture after its producer/input changes.
    // This lifecycle binds the freshly published exact generation throughout.
    store.close()?;
    std::fs::remove_dir_all(&input)?;

    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    let native = read.pair().graph_read(&stop)?;
    let provenance = read.pair().source_provenance().ok_or("native provenance")?;
    let account = provenance
        .state_roots()
        .iter()
        .find(|root| root.scope == TocSavedVariableScope::Account)
        .ok_or("account declaration")?;
    let character = provenance
        .state_roots()
        .iter()
        .find(|root| root.scope == TocSavedVariableScope::Character)
        .ok_or("character declaration")?;
    let path = provenance
        .state_paths()
        .iter()
        .find(|path| {
            path.root_id == account.root_id
                && serde_json::to_value(&path.keys).ok()
                    == Some(json!([{"kind":"string","value":"profile"}]))
        })
        .ok_or("native profile path")?;
    let character_path = provenance
        .state_paths()
        .iter()
        .find(|path| {
            path.root_id == character.root_id
                && serde_json::to_value(&path.keys).ok()
                    == Some(json!([{"kind":"string","value":"gold"}]))
        })
        .ok_or("native character gold path")?;
    let file = provenance
        .files()
        .iter()
        .find(|file| file.path == account.document)
        .ok_or("declaration document")?;
    let mut selected = Vec::new();
    for proposal in [
        &path.proposal_id,
        &account.proposal_id,
        &file.proposal_id,
        &character.proposal_id,
        &character_path.proposal_id,
    ] {
        selected.push(
            native
                .published_entity(SOURCE_GRAPH_PARTITION, proposal, &stop)?
                .node_id()
                .clone(),
        );
    }
    read.close()?;
    store.close()?;
    let mut host_json = host_value(&store_root, &profile, &reference);
    host_json["schema"] = json!(CONTEXT_MAP_CONTINUATION_HOST_SCHEMA);
    host_json["continuation_store_file"] = json!(
        scratch
            .root
            .join("continuation-store.sqlite")
            .to_string_lossy()
            .to_string()
    );
    host_json["limits"]["max_context_work"] = json!(32_000_000);
    host_json["limits"]["max_metadata_bytes"] = json!(16 * 1024 * 1024);
    let selected_reference_profile =
        wow_core::canonical_json_bytes(&host_json["reference_profile"])?;
    let selected_reference_profile_sha256 =
        format!("{:x}", Sha256::digest(&selected_reference_profile));
    let host = decode_host(&host_json, &stop)?;
    let prefix_request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![vec![ContextMapRoot::graph_entity(
        selected[1].as_str(),
    )?]])?;
    let prefix = execute_context_build(&host, &prefix_request, &stop)?;
    assert_eq!(prefix.status(), ContextMapStatus::Partial);
    assert_eq!(prefix.failure_code(), None);
    let prefix_envelope: Value = serde_json::from_slice(prefix.canonical_bytes())?;
    let request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[3].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[4].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;

    let issued = execute_context_build(&host, &request, &stop)
        .map_err(|error| std::io::Error::other(format!("initial continuation issue: {error:?}")))?;
    assert_eq!(issued.status(), ContextMapStatus::Partial);
    assert_eq!(issued.failure_code(), None);
    let issued_envelope: Value = serde_json::from_slice(issued.canonical_bytes())?;
    let issued_payload = &issued_envelope["payload"];
    let token = issued_payload["continuation_id"]
        .as_str()
        .ok_or("service continuation token")?;
    let pack_id = issued_payload["semantic_pack"]["semantic_pack_id"]
        .as_str()
        .ok_or("issued semantic pack")?;
    assert_ne!(token, pack_id);
    assert_eq!(
        issued_payload["semantic_pack"], prefix_envelope["payload"]["semantic_pack"],
        "retained issuance must preserve the ordinary exact-generation first batch"
    );
    let selected_pack_profile =
        wow_core::canonical_json_bytes(&issued_payload["semantic_pack"]["core"]["profile"])?;
    assert_eq!(
        format!("{:x}", Sha256::digest(&selected_pack_profile)),
        frozen["selected_profiles"]["pack_sha256"]
    );
    let selected_expansion_profile = wow_core::canonical_json_bytes(
        &issued_payload["semantic_pack"]["core"]["expansion"]["core"]["profile"],
    )?;
    assert_eq!(
        format!("{:x}", Sha256::digest(&selected_expansion_profile)),
        frozen["selected_profiles"]["expansion_sha256"]
    );
    assert_eq!(
        selected_reference_profile_sha256,
        frozen["selected_profiles"]["reference_sha256"]
    );
    assert_eq!(
        issued_payload["store_generation_id"],
        publication.generation_id.as_str()
    );
    assert_eq!(
        issued_payload["semantic_pack"]["core"]["expansion"]["core"]["progression"]["round"],
        1
    );

    // Move Current through a real source update and fresh owner graph, not just a new label.
    // The Lua input is data consumed by the native project owner; it is never executed.
    let (next_publisher, next_graph, _, _) = owners_from_bundle(
        input_bundle_files(vec![ProjectInputFile::new(
            "main.lua",
            "local function ContextFixture(value)\n    return value + 2\nend\n",
        )?])?,
        &stop,
    )?;
    assert_ne!(
        graph.snapshot().generation(),
        next_graph.snapshot().generation(),
        "Current advance must publish a changed source graph"
    );
    let mut current_store = LiveProjectStore::open(&store_root)?;
    let before = current_store.current()?.ok_or("current before move")?;
    assert_eq!(before.generation_id, publication.generation_id);
    let advanced = current_store.publish(
        &next_publisher,
        &next_graph,
        "fixture-context-continuation-current-advance",
        Some(before.record_id.clone()),
        &stop,
    )?;
    let after = current_store.current()?.ok_or("current after move")?;
    assert_ne!(before.record_id, after.record_id);
    assert_ne!(
        after.generation_id, publication.generation_id,
        "Current advance must select a different generation from the issued prefix"
    );
    assert_eq!(after.generation_id, advanced.generation_id);
    current_store.close()?;

    // Model a process loss after the initial hold but before the response,
    // receipt, and token batch reaches Store::commit_operation. The durable
    // Prepared record must stop an exact retry before it can resolve Current
    // onto the newly advanced generation; the old hold remains conservative.
    let orphan_request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::Current,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[3].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[4].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
    let (orphan_request_id, orphan_operation_id, orphan_request_digest) =
        crate::context::continuation::build_operation_identity(&orphan_request)?;
    let continuation_store_path = scratch.root.join("continuation-store.sqlite");
    let configuration = crate::context::continuation::continuation_store_configuration()
        .map_err(|error| std::io::Error::other(format!("orphan store configuration: {error:?}")))?;
    let mut journal = wow_store::Store::open(&continuation_store_path, configuration)?;
    let prepared =
        journal.begin_operation(orphan_operation_id.clone(), orphan_request_digest.clone())?;
    assert!(matches!(prepared, wow_store::OperationBegin::Started(_)));
    drop(journal);

    let orphan_root = {
        let mut project_store = LiveProjectStore::open(&store_root)?;
        let root = crate::context::continuation::retention_root(
            &project_store,
            &orphan_request_id,
            publication.generation_id.clone(),
        )?;
        assert_eq!(project_store.put_retention_root(&root, &stop)?, root);
        project_store.close()?;
        root
    };
    let orphan_retry = execute_context_build(&host, &orphan_request, &stop)?;
    assert_eq!(
        orphan_retry.failure_code(),
        Some(crate::ServiceErrorCode::StoreOutcomeUnknown)
    );
    let journal = wow_store::Store::open(
        &continuation_store_path,
        crate::context::continuation::continuation_store_configuration().map_err(|error| {
            std::io::Error::other(format!("orphan replay configuration: {error:?}"))
        })?,
    )?;
    assert_eq!(
        journal
            .operation(&orphan_operation_id)?
            .ok_or("prepared orphan operation")?
            .state(),
        wow_store::OperationState::Prepared
    );
    drop(journal);
    let project_store = LiveProjectStore::open(&store_root)?;
    assert_eq!(
        project_store
            .current()?
            .ok_or("current after orphan retry")?
            .generation_id,
        advanced.generation_id
    );
    let roots_after_orphan_retry = project_store.retention_roots(&stop)?;
    assert!(roots_after_orphan_retry.contains(&orphan_root));
    let advanced_orphan_root = crate::context::continuation::retention_root(
        &project_store,
        &orphan_request_id,
        advanced.generation_id.clone(),
    )?;
    assert!(!roots_after_orphan_retry.contains(&advanced_orphan_root));
    project_store.close()?;

    let continuation_store_path = scratch.root.join("continuation-store.sqlite");
    let build_operation_id = crate::context::continuation::build_operation_identity(&request)?.1;
    let (initial_response_object_id, initial_receipt_object_id) = {
        let mut journal = open_continuation_store(&continuation_store_path)?;
        let operation = journal
            .operation(&build_operation_id)?
            .ok_or("initial completed continuation operation")?;
        let result_id = operation
            .result_object_id()
            .ok_or("initial outcome object")?;
        let outcome = journal.object(result_id)?.ok_or("initial outcome bytes")?;
        let value: Value = serde_json::from_slice(outcome.canonical_json())?;
        let response_id = operation_response_object_id(&journal, &build_operation_id)?;
        let receipt_id = wow_store::ObjectId::new(
            value["next_receipt_object_id"]
                .as_str()
                .ok_or("initial receipt object ID")?
                .to_owned(),
        )?;
        let gc = journal.collect_garbage(wow_store::LogicalEpoch::new(0), 10_000)?;
        assert!(!gc.deleted().contains(&response_id));
        assert!(!gc.deleted().contains(&receipt_id));
        assert!(journal.object(&response_id)?.is_some());
        assert!(journal.object(&receipt_id)?.is_some());
        (response_id, receipt_id)
    };
    assert_eq!(
        execute_context_build(&host, &request, &stop)?.canonical_bytes(),
        issued.canonical_bytes(),
        "generic GC must preserve the exact initial response retry"
    );

    let continuation =
        ContextContinueRequest::new(token, issued_envelope["request_id"].as_str(), Some(pack_id))?;
    let continuation_operation =
        crate::context::continuation::continue_operation_identity(&continuation)?.1;
    let cancelled_continue_stop = std::sync::Arc::new(AtomicBool::new(false));
    let continue_observer = cancel_when_operation_completes(
        scratch.root.join("continuation-store.sqlite"),
        continuation_operation,
        std::sync::Arc::clone(&cancelled_continue_stop),
    );
    let completed_continue =
        execute_context_continue(&host, &continuation, &cancelled_continue_stop);
    assert!(
        continue_observer
            .join()
            .map_err(|_| std::io::Error::other("continuation observer panicked"))?
            .map_err(std::io::Error::other)?
    );
    let completed_continue = completed_continue.map_err(|error| {
        std::io::Error::other(format!("completed continuation response: {error:?}"))
    })?;
    assert_eq!(completed_continue.status(), ContextMapStatus::Partial);
    assert_eq!(completed_continue.failure_code(), None);
    let resumed = execute_context_continue(&host, &continuation, &stop)
        .map_err(|error| std::io::Error::other(format!("continuation resume: {error:?}")))?;
    assert_eq!(resumed.status(), ContextMapStatus::Partial);
    assert_eq!(resumed.failure_code(), None);
    assert_eq!(
        completed_continue.canonical_bytes(),
        resumed.canonical_bytes(),
        "exact continuation retry must return the already committed response"
    );
    let resumed_envelope: Value = serde_json::from_slice(resumed.canonical_bytes())?;
    assert_eq!(resumed_envelope["operation"], "context_continue");
    assert_eq!(
        resumed_envelope["payload"]["store_generation_id"],
        publication.generation_id.as_str()
    );
    assert_eq!(
        resumed_envelope["payload"]["semantic_pack"]["core"]["expansion"]["core"]["progression"]["round"],
        2
    );
    let round_two_token = resumed_envelope["payload"]["continuation_id"]
        .as_str()
        .ok_or("round two continuation token")?;
    let round_two_pack_id = resumed_envelope["payload"]["semantic_pack"]["semantic_pack_id"]
        .as_str()
        .ok_or("round two semantic pack")?;
    let initial_work = issued_payload["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("initial operation work")?;
    let initial_bytes = issued_payload["operation_budget_usage"]["bytes_reserved"]
        .as_u64()
        .ok_or("initial operation bytes")?;
    assert!(
        resumed_envelope["payload"]["operation_budget_usage"]["work_reserved"]
            .as_u64()
            .ok_or("round two cumulative work")?
            > initial_work
    );
    assert!(
        resumed_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
            .as_u64()
            .ok_or("round two cumulative bytes")?
            > initial_bytes
    );
    let first_continue_operation_id =
        crate::context::continuation::continue_operation_identity(&continuation)?.1;
    let (round_two_receipt_object_id, receipt_chain_object_ids) = {
        let mut journal = open_continuation_store(&continuation_store_path)?;
        let operation = journal
            .operation(&first_continue_operation_id)?
            .ok_or("round two completed operation")?;
        let result_id = operation
            .result_object_id()
            .ok_or("round two outcome object")?;
        let outcome = journal
            .object(result_id)?
            .ok_or("round two outcome bytes")?;
        let value: Value = serde_json::from_slice(outcome.canonical_json())?;
        let response_id = operation_response_object_id(&journal, &first_continue_operation_id)?;
        let receipt_id = wow_store::ObjectId::new(
            value["next_receipt_object_id"]
                .as_str()
                .ok_or("round two receipt object ID")?
                .to_owned(),
        )?;
        let chain_ids = history_receipt_object_ids(
            &journal,
            issued_envelope["request_id"]
                .as_str()
                .ok_or("original request ID")?,
        )?;
        assert_eq!(chain_ids.len(), 2);
        let gc = journal.collect_garbage(wow_store::LogicalEpoch::new(0), 10_000)?;
        assert!(!gc.deleted().contains(&initial_response_object_id));
        assert!(!gc.deleted().contains(&initial_receipt_object_id));
        assert!(!gc.deleted().contains(&response_id));
        assert!(!gc.deleted().contains(&receipt_id));
        assert!(journal.object(&response_id)?.is_some());
        for receipt_id in &chain_ids {
            assert!(!gc.deleted().contains(receipt_id));
            assert!(journal.object(receipt_id)?.is_some());
        }
        (receipt_id, chain_ids)
    };
    assert!(receipt_chain_object_ids.contains(&initial_receipt_object_id));
    assert!(receipt_chain_object_ids.contains(&round_two_receipt_object_id));
    assert_eq!(
        execute_context_build(&host, &request, &stop)?.canonical_bytes(),
        issued.canonical_bytes(),
        "GC between rounds must preserve the original exact build response"
    );
    assert_eq!(
        execute_context_continue(&host, &continuation, &stop)?.canonical_bytes(),
        resumed.canonical_bytes()
    );

    // A stricter selected owner cap admits the original delivery, then refuses
    // when historical spend, this request's replay and the next native prefix
    // are all charged together. The cap is fixed test input, not a saved scalar.
    let boundary_cap = 3 * 1024 * 1024;
    let mut boundary_host_json = host_json.clone();
    boundary_host_json["limits"]["max_metadata_bytes"] = json!(boundary_cap);
    let boundary_host = decode_host(&boundary_host_json, &stop)?;
    let boundary_request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        None,
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[3].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[4].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
    let roots_before_boundary = {
        let store = LiveProjectStore::open(&store_root)?;
        let roots = store.retention_roots(&stop)?;
        store.close()?;
        roots
    };
    let boundary_issue = execute_context_build(&boundary_host, &boundary_request, &stop)?;
    assert_eq!(boundary_issue.failure_code(), None);
    let boundary_envelope: Value = serde_json::from_slice(boundary_issue.canonical_bytes())?;
    let boundary_token = boundary_envelope["payload"]["continuation_id"]
        .as_str()
        .ok_or("boundary continuation token")?;
    let boundary_pack_id = boundary_envelope["payload"]["semantic_pack"]["semantic_pack_id"]
        .as_str()
        .ok_or("boundary semantic pack")?;
    let boundary_historical_bytes =
        boundary_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
            .as_u64()
            .ok_or("boundary original reservation")?
            .checked_add(u64::try_from(boundary_issue.canonical_bytes().len())?)
            .ok_or("boundary delivered reservation")?;
    assert!(boundary_historical_bytes <= u64::try_from(boundary_cap)?);
    let boundary_continue = ContextContinueRequest::new(
        boundary_token,
        boundary_envelope["request_id"].as_str(),
        Some(boundary_pack_id),
    )?;
    let boundary_refusal = execute_context_continue(&boundary_host, &boundary_continue, &stop)?;
    assert_eq!(
        boundary_refusal.failure_code(),
        Some(crate::ServiceErrorCode::BudgetExceeded)
    );
    assert_eq!(
        execute_context_continue(&boundary_host, &boundary_continue, &stop)?.canonical_bytes(),
        boundary_refusal.canonical_bytes()
    );
    let mut store = LiveProjectStore::open(&store_root)?;
    let roots_after_boundary = store.retention_roots(&stop)?;
    let boundary_root = roots_after_boundary
        .iter()
        .find(|root| !roots_before_boundary.contains(root))
        .ok_or("boundary refusal preserved its exact hold")?;
    assert!(store.remove_retention_root(
        boundary_root.root_id(),
        boundary_root.pin_digest(),
        &stop,
    )?);
    store.close()?;

    // A completed retry is bound to the original exact host limits and Reference.
    let mut changed_limits_json = host_json.clone();
    changed_limits_json["limits"]["max_context_work"] = json!(31_999_999);
    let changed_limits_host = decode_host(&changed_limits_json, &stop)?;
    let changed_limits_retry =
        execute_context_continue(&changed_limits_host, &continuation, &stop)?;
    assert_eq!(
        changed_limits_retry.failure_code(),
        Some(crate::ServiceErrorCode::IdentityMismatch)
    );

    let changed_reference = wow_reference::ReferenceView::new(
        format!("{}-retry", reference.generation_id()),
        reference.partitions().to_vec(),
        reference.conflicts().to_vec(),
    )?;
    let mut changed_reference_json = host_json.clone();
    changed_reference_json["reference"] = serde_json::to_value(changed_reference)?;
    let changed_reference_host = decode_host(&changed_reference_json, &stop)?;
    let changed_reference_retry =
        execute_context_continue(&changed_reference_host, &continuation, &stop)?;
    assert_eq!(
        changed_reference_retry.failure_code(),
        Some(crate::ServiceErrorCode::IdentityMismatch)
    );

    let round_two_request = ContextContinueRequest::new(
        round_two_token,
        issued_envelope["request_id"].as_str(),
        Some(round_two_pack_id),
    )?;
    let mut highest_successful_round = 2_u8;
    let mut budget_refusal_round = None;
    let mut budget_stopped = false;
    let resumed_round_three = execute_context_continue(&host, &round_two_request, &stop)?;
    if resumed_round_three.failure_code() == Some(crate::ServiceErrorCode::BudgetExceeded) {
        let failed: Value = serde_json::from_slice(resumed_round_three.canonical_bytes())?;
        assert!(failed.get("payload").is_none_or(Value::is_null));
        let historical_work =
            resumed_envelope["payload"]["operation_budget_usage"]["work_reserved"]
                .as_u64()
                .ok_or("round two cumulative work")?
                .checked_add(u64::try_from(resumed.canonical_bytes().len())?)
                .ok_or("round two delivered work")?;
        let historical_bytes =
            resumed_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
                .as_u64()
                .ok_or("round two cumulative bytes")?
                .checked_add(u64::try_from(resumed.canonical_bytes().len())?)
                .ok_or("round two delivered bytes")?;
        assert!(historical_work <= 32_000_000);
        assert!(historical_bytes <= 16 * 1024 * 1024);
        assert_eq!(
            execute_context_continue(&host, &round_two_request, &stop)?.canonical_bytes(),
            resumed_round_three.canonical_bytes(),
            "historical cumulative budget plus two verified-prefix replays must refuse deterministically"
        );
        budget_refusal_round = Some(3_u8);
        budget_stopped = true;
    } else {
        assert_eq!(resumed_round_three.status(), ContextMapStatus::Partial);
        highest_successful_round = 3;
        let resumed_round_three_envelope: Value =
            serde_json::from_slice(resumed_round_three.canonical_bytes())?;
        assert_eq!(
            resumed_round_three_envelope["payload"]["semantic_pack"]["core"]["expansion"]["core"]["progression"]
                ["round"],
            3
        );
        let round_three_token = resumed_round_three_envelope["payload"]["continuation_id"]
            .as_str()
            .ok_or("round three continuation token")?;
        let round_three_pack_id =
            resumed_round_three_envelope["payload"]["semantic_pack"]["semantic_pack_id"]
                .as_str()
                .ok_or("round three semantic pack")?;
        assert!(
            resumed_round_three_envelope["payload"]["operation_budget_usage"]["work_reserved"]
                .as_u64()
                .ok_or("round three cumulative work")?
                > resumed_envelope["payload"]["operation_budget_usage"]["work_reserved"]
                    .as_u64()
                    .ok_or("round two cumulative work")?
        );
        assert!(
            resumed_round_three_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
                .as_u64()
                .ok_or("round three cumulative bytes")?
                > resumed_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
                    .as_u64()
                    .ok_or("round two cumulative bytes")?
        );
        assert_eq!(
            execute_context_continue(&host, &round_two_request, &stop)?.canonical_bytes(),
            resumed_round_three.canonical_bytes()
        );

        let round_three_request = ContextContinueRequest::new(
            round_three_token,
            issued_envelope["request_id"].as_str(),
            Some(round_three_pack_id),
        )?;
        let fourth = execute_context_continue(&host, &round_three_request, &stop)?;
        if fourth.failure_code() == Some(crate::ServiceErrorCode::BudgetExceeded) {
            let historical_work = resumed_round_three_envelope["payload"]["operation_budget_usage"]
                ["work_reserved"]
                .as_u64()
                .ok_or("round three cumulative work")?
                .checked_add(u64::try_from(resumed_round_three.canonical_bytes().len())?)
                .ok_or("round three delivered work")?;
            let historical_bytes =
                resumed_round_three_envelope["payload"]["operation_budget_usage"]["bytes_reserved"]
                    .as_u64()
                    .ok_or("round three cumulative bytes")?
                    .checked_add(u64::try_from(resumed_round_three.canonical_bytes().len())?)
                    .ok_or("round three delivered bytes")?;
            assert!(historical_work <= 32_000_000);
            assert!(historical_bytes <= 16 * 1024 * 1024);
            assert_eq!(
                execute_context_continue(&host, &round_three_request, &stop)?.canonical_bytes(),
                fourth.canonical_bytes(),
                "historical cumulative budget plus all prefix replays must refuse deterministically"
            );
            budget_refusal_round = Some(4_u8);
            budget_stopped = true;
        } else {
            assert_eq!(fourth.failure_code(), None);
            let fourth_envelope: Value = serde_json::from_slice(fourth.canonical_bytes())?;
            assert_eq!(
                fourth_envelope["payload"]["semantic_pack"]["core"]["expansion"]["core"]["progression"]
                    ["round"],
                4
            );
            assert!(fourth_envelope["payload"]["continuation_id"].is_null());
            assert_eq!(
                execute_context_continue(&host, &round_three_request, &stop)?.canonical_bytes(),
                fourth.canonical_bytes()
            );
            highest_successful_round = 4;
        }
    }

    let mut store = LiveProjectStore::open(&store_root)?;
    let roots = store.retention_roots(&stop)?;
    assert_eq!(roots.len(), if budget_stopped { 2 } else { 1 });
    assert!(roots.contains(&orphan_root));
    if budget_stopped {
        let active_root = crate::context::continuation::retention_root(
            &store,
            issued_envelope["request_id"]
                .as_str()
                .ok_or("initial request ID")?,
            publication.generation_id.clone(),
        )?;
        assert!(roots.contains(&active_root));
        for root in roots {
            assert!(store.remove_retention_root(root.root_id(), root.pin_digest(), &stop)?);
        }
    } else {
        assert_eq!(roots, vec![orphan_root.clone()]);
        assert!(store.remove_retention_root(
            orphan_root.root_id(),
            orphan_root.pin_digest(),
            &stop
        )?);
    }
    store.close()?;
    eprintln!(
        "native four-round lifecycle: highest_successful_round={highest_successful_round}, budget_refusal_round={budget_refusal_round:?}, selected_caps=32M_work/16MiB_bytes"
    );

    let cancelled = AtomicBool::new(true);
    let cancelled_result = execute_context_continue(&host, &continuation, &cancelled);
    assert!(matches!(
        cancelled_result,
        Err(error) if error.code() == crate::ServiceErrorCode::Cancelled
    ));
    let unknown = ContextContinueRequest::new(
        "context-continuation:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        None,
        None,
    )?;
    let unknown_result = execute_context_continue(&host, &unknown, &stop)?;
    assert_eq!(unknown_result.status(), ContextMapStatus::Failed);
    assert_eq!(
        unknown_result.failure_code(),
        Some(crate::ServiceErrorCode::InvalidRequest)
    );

    // Cancellation after completion must still return the stored response. Its
    // exact retry is byte-identical, and the persisted JSON prefix resumes.
    let mut rendered_request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::Json,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
    let build_operation =
        crate::context::continuation::build_operation_identity(&rendered_request)?.1;
    let cancelled_build_stop = std::sync::Arc::new(AtomicBool::new(false));
    let build_observer = cancel_when_operation_completes(
        scratch.root.join("continuation-store.sqlite"),
        build_operation,
        std::sync::Arc::clone(&cancelled_build_stop),
    );
    let completed_build = execute_context_build(&host, &rendered_request, &cancelled_build_stop);
    assert!(
        build_observer
            .join()
            .map_err(|_| std::io::Error::other("build observer panicked"))?
            .map_err(std::io::Error::other)?
    );
    let completed_build = completed_build
        .map_err(|error| std::io::Error::other(format!("completed build response: {error:?}")))?;
    assert_eq!(completed_build.failure_code(), None);
    let json_issue = execute_context_build(&host, &rendered_request, &stop)?;
    assert_eq!(
        completed_build.canonical_bytes(),
        json_issue.canonical_bytes(),
        "exact build retry must return the already committed response"
    );
    let json_envelope: Value = serde_json::from_slice(json_issue.canonical_bytes())?;
    let json_token = json_envelope["payload"]["continuation_id"]
        .as_str()
        .ok_or("persisted JSON continuation token")?;
    let json_build_operation_id =
        crate::context::continuation::build_operation_identity(&rendered_request)?.1;
    let json_history_prune = crate::context::ContextContinuationHistoryPruneRequest::new(
        json_envelope["request_id"]
            .as_str()
            .ok_or("JSON original request ID")?,
    )?;
    let active_prune = crate::context::execute_context_continuation_history_prune(
        &host,
        &json_history_prune,
        &stop,
    )?;
    assert_eq!(
        active_prune.failure_code(),
        Some(crate::ServiceErrorCode::OperationConflict),
        "a live token history cannot be pruned"
    );
    let (json_initial_response_id, json_initial_receipt_id, json_initial_outcome_id) = {
        let journal = open_continuation_store(&continuation_store_path)?;
        let response_id = operation_response_object_id(&journal, &json_build_operation_id)?;
        let operation = journal
            .operation(&json_build_operation_id)?
            .ok_or("JSON initial operation")?;
        let outcome_id = operation
            .result_object_id()
            .ok_or("JSON initial outcome")?
            .clone();
        let outcome = journal
            .object(&outcome_id)?
            .ok_or("JSON initial outcome bytes")?;
        let value: Value = serde_json::from_slice(outcome.canonical_json())?;
        (
            response_id,
            wow_store::ObjectId::new(
                value["next_receipt_object_id"]
                    .as_str()
                    .ok_or("JSON initial receipt object ID")?
                    .to_owned(),
            )?,
            outcome_id,
        )
    };
    let gc_after_json_issue = collect_continuation_garbage(&continuation_store_path)?;
    assert!(!gc_after_json_issue.contains(&json_initial_response_id));
    assert!(!gc_after_json_issue.contains(&json_initial_receipt_id));
    assert_eq!(
        execute_context_build(&host, &rendered_request, &stop)?.canonical_bytes(),
        json_issue.canonical_bytes(),
        "generic GC preserves the selected JSON response retry"
    );
    let json_continue = ContextContinueRequest::new(
        json_token,
        json_envelope["request_id"].as_str(),
        json_envelope["payload"]["semantic_pack"]["semantic_pack_id"].as_str(),
    )?;
    let json_resume = execute_context_continue(&host, &json_continue, &stop)?;
    assert_eq!(json_resume.failure_code(), None);
    let json_resume_envelope: Value = serde_json::from_slice(json_resume.canonical_bytes())?;
    assert_eq!(
        json_resume_envelope["payload"]["semantic_pack"]["core"]["expansion"]["core"]["progression"]
            ["round"],
        2
    );
    assert_eq!(
        json_resume_envelope["payload"]["rendered_artifacts"]
            .as_array()
            .ok_or("persisted JSON renderer artifacts")?
            .len(),
        1
    );
    assert!(json_resume_envelope["payload"]["continuation_id"].is_null());
    assert_eq!(
        execute_context_continue(&host, &json_continue, &stop)?.canonical_bytes(),
        json_resume.canonical_bytes()
    );
    let json_continue_operation_id =
        crate::context::continuation::continue_operation_identity(&json_continue)?.1;
    let (
        json_terminal_response_id,
        json_receipt_ids,
        json_terminal_outcome_id,
        json_history_index_id,
    ) = {
        let journal = open_continuation_store(&continuation_store_path)?;
        (
            operation_response_object_id(&journal, &json_continue_operation_id)?,
            history_receipt_object_ids(
                &journal,
                json_envelope["request_id"]
                    .as_str()
                    .ok_or("JSON original request ID")?,
            )?,
            operation_result_object_id(&journal, &json_continue_operation_id)?,
            history_index_object_id(
                &journal,
                json_envelope["request_id"]
                    .as_str()
                    .ok_or("JSON original request ID")?,
            )?,
        )
    };
    assert_eq!(json_receipt_ids.len(), 1);
    let gc_before_prune = collect_continuation_garbage(&continuation_store_path)?;
    assert!(!gc_before_prune.contains(&json_initial_response_id));
    assert!(!gc_before_prune.contains(&json_terminal_response_id));
    for receipt_id in &json_receipt_ids {
        assert!(!gc_before_prune.contains(receipt_id));
    }
    assert_eq!(
        execute_context_build(&host, &rendered_request, &stop)?.canonical_bytes(),
        json_issue.canonical_bytes()
    );
    assert_eq!(
        execute_context_continue(&host, &json_continue, &stop)?.canonical_bytes(),
        json_resume.canonical_bytes(),
        "terminal retries remain exact until explicit prune"
    );
    let pruned = crate::context::execute_context_continuation_history_prune(
        &host,
        &json_history_prune,
        &stop,
    )?;
    assert_eq!(pruned.status(), ContextMapStatus::Complete);
    assert_eq!(pruned.failure_code(), None);
    assert_eq!(
        crate::context::execute_context_continuation_history_prune(
            &host,
            &json_history_prune,
            &stop,
        )?
        .canonical_bytes(),
        pruned.canonical_bytes(),
        "the terminal prune operation has an exact durable retry"
    );
    let gc_after_prune = collect_continuation_garbage(&continuation_store_path)?;
    assert!(gc_after_prune.contains(&json_initial_response_id));
    assert!(gc_after_prune.contains(&json_terminal_response_id));
    assert!(gc_after_prune.contains(&json_initial_outcome_id));
    assert!(gc_after_prune.contains(&json_terminal_outcome_id));
    assert!(gc_after_prune.contains(&json_history_index_id));
    for receipt_id in &json_receipt_ids {
        assert!(gc_after_prune.contains(receipt_id));
    }
    assert_eq!(
        crate::context::execute_context_continuation_history_prune(
            &host,
            &json_history_prune,
            &stop,
        )?
        .canonical_bytes(),
        pruned.canonical_bytes(),
        "the prune result remains exact after generic GC"
    );
    let mut changed_prune_limits_json = host_json.clone();
    changed_prune_limits_json["limits"]["max_context_work"] = json!(31_999_999);
    let changed_prune_limits_host = decode_host(&changed_prune_limits_json, &stop)?;
    assert_eq!(
        crate::context::execute_context_continuation_history_prune(
            &changed_prune_limits_host,
            &json_history_prune,
            &stop,
        )?
        .failure_code(),
        Some(crate::ServiceErrorCode::OperationConflict),
        "a pruned retry with changed owner limits is refused after history GC"
    );
    let changed_prune_reference = wow_reference::ReferenceView::new(
        format!("{}-prune-retry", reference.generation_id()),
        reference.partitions().to_vec(),
        reference.conflicts().to_vec(),
    )?;
    let mut changed_prune_reference_json = host_json.clone();
    changed_prune_reference_json["reference"] = serde_json::to_value(changed_prune_reference)?;
    let changed_prune_reference_host = decode_host(&changed_prune_reference_json, &stop)?;
    assert_eq!(
        crate::context::execute_context_continuation_history_prune(
            &changed_prune_reference_host,
            &json_history_prune,
            &stop,
        )?
        .failure_code(),
        Some(crate::ServiceErrorCode::OperationConflict),
        "a pruned retry with changed Reference identity is refused after history GC"
    );
    assert_eq!(
        execute_context_build(&host, &rendered_request, &stop)?.failure_code(),
        Some(crate::ServiceErrorCode::OperationReleased)
    );
    assert_eq!(
        execute_context_continue(&host, &json_continue, &stop)?.failure_code(),
        Some(crate::ServiceErrorCode::OperationReleased)
    );
    let store = LiveProjectStore::open(&store_root)?;
    assert!(store.retention_roots(&stop)?.is_empty());
    store.close()?;

    // A separate token loses its exact hold and refuses rather than using Current.
    let stale_build = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
    let stale_issue = execute_context_build(&host, &stale_build, &stop)?;
    let stale_envelope: Value = serde_json::from_slice(stale_issue.canonical_bytes())?;
    let stale_token = stale_envelope["payload"]["continuation_id"]
        .as_str()
        .ok_or("stale continuation token")?;
    let stale_request = ContextContinueRequest::new(
        stale_token,
        stale_envelope["request_id"].as_str(),
        stale_envelope["payload"]["semantic_pack"]["semantic_pack_id"].as_str(),
    )?;
    let mut store = LiveProjectStore::open(&store_root)?;
    let roots = store.retention_roots(&stop)?;
    assert_eq!(roots.len(), 1);
    let root = roots.first().ok_or("issued generation hold")?;
    assert!(store.remove_retention_root(root.root_id(), root.pin_digest(), &stop)?);
    store.close()?;
    let stale = execute_context_continue(&host, &stale_request, &stop)
        .map_err(|error| std::io::Error::other(format!("stale-token resume: {error:?}")))?;
    assert_eq!(stale.status(), ContextMapStatus::Failed);
    assert_eq!(
        stale.failure_code(),
        Some(crate::ServiceErrorCode::IdentityMismatch)
    );

    // Delivery itself is charged; a pre-delivery fit cannot mint a token.
    let mut budget_host = host_value(&store_root, &profile, &reference);
    budget_host["schema"] = json!(CONTEXT_MAP_CONTINUATION_HOST_SCHEMA);
    budget_host["continuation_store_file"] = json!(
        scratch
            .root
            .join("continuation-budget-store.sqlite")
            .to_string_lossy()
            .to_string()
    );
    budget_host["limits"]["max_context_work"] = json!(5892);
    budget_host["limits"]["max_metadata_bytes"] = json!(16 * 1024 * 1024);
    let budget_host = decode_host(&budget_host, &stop)?;
    rendered_request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![
        vec![ContextMapRoot::graph_entity(selected[1].as_str())?],
        vec![ContextMapRoot::graph_entity(selected[2].as_str())?],
    ])?
    .with_continuation_retention_profile(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID)?;
    let refused = execute_context_build(&budget_host, &rendered_request, &stop)
        .map_err(|error| std::io::Error::other(format!("delivery budget refusal: {error:?}")))?;
    assert_eq!(refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        refused.failure_code(),
        Some(crate::ServiceErrorCode::BudgetExceeded)
    );
    let store = LiveProjectStore::open(&store_root)?;
    assert!(store.retention_roots(&stop)?.is_empty());
    store.close()?;
    Ok(())
}
