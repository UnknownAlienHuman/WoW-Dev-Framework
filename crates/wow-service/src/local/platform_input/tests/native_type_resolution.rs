//! Focused selected-corrections -> Reference type overlay -> Primary ReferenceFacts witness.
//! All corpus and correction data here is synthetic fixture input; this test makes no
//! live-client, donor, current-source, or runtime claim.
use super::*;
use std::io::Write;
use wow_store::project::ReadSelector;

const TYPE_PROFILE: &str = "wow-reference/native-callable-type-resolution/1";
const CONTEXT_STORE_ID: &str = "project-store:fixture-native-type-resolution";
const CORRECTION_ID: &str = "fixture-correct-return-type";

fn add_main_and_manifest(root: &FixtureRoot, config: &mut Value) -> TestResult {
    let main = "function FixtureFunction() return C_Transport.Read() end\n";
    std::fs::write(root.0.join("UI/main.lua"), main)?;
    let api = r#"APIDocumentation:AddDocumentationTable({Name="Transport",Type="System",Namespace="C_Transport",Environment="All",Functions={{Name="Read",Type="Function",Arguments={{Name="flag",Type="bool",Nilable=false,Default=false}},Returns={{Name="value",Type="number",Nilable=true}}}},Tables={{Name="FixtureRecord",Type="Structure",Fields={{Name="label",Type="string",Nilable=false}}}}})"#;
    let api_path = "Blizzard_APIDocumentationGenerated/API.lua";
    std::fs::write(root.0.join(api_path), api)?;
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"]
        .as_array_mut()
        .ok_or("fixture manifest files")?;
    for (path, source) in [("UI/main.lua", main), (api_path, api)] {
        let member = files
            .iter_mut()
            .find(|file| file["path"] == path)
            .ok_or("fixture manifest row")?;
        member["bytes"] = json!(source.len());
        member["content_sha256"] = json!(format!("{:x}", Sha256::digest(source.as_bytes())));
    }
    let total = files.iter().try_fold(0_u64, |sum, row| {
        sum.checked_add(row["bytes"].as_u64().ok_or("manifest row bytes")?)
            .ok_or("manifest byte sum")
    })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let descriptor = write_json(root, "manifest.json", &manifest)?;
    config["native_reference"]["native_source"]["manifest"] = serde_json::to_value(descriptor)?;
    Ok(())
}

// Build one exact guarded correction from the small synthetic API.lua fixture.
// Hashes and `before` come from its parsed raw field; no source is executed.
fn correction_pack(root: &FixtureRoot, revision: &str) -> TestResult<Value> {
    let path = "Blizzard_APIDocumentationGenerated/API.lua";
    let source = std::fs::read_to_string(root.0.join(path))?;
    let source_sha = wow_reference::native::source_digest(source.as_bytes());
    let document = wow_reference::native::ingest_document(
        revision,
        path,
        &source,
        &source_sha,
        &AtomicBool::new(false),
    )?;
    let normalized = wow_reference::native_model::normalize_document(&document);
    let system = normalized
        .systems
        .iter()
        .find_map(|entry| entry.as_ref().ok())
        .ok_or("synthetic API registration")?;
    let function = system
        .functions
        .iter()
        .find(|function| function.name == "Read")
        .ok_or("synthetic Read function")?;
    let returned = function
        .returns
        .iter()
        .find(|field| field.name == "value")
        .ok_or("synthetic return field")?;
    let raw_digest = wow_reference::native_corrections::raw_digest(returned.raw)?;
    Ok(json!({
        "schema": "wow-native-corrections/1", "version": 1,
        "revision": revision, "environment": "Mainline", "normalizer": "native-model/1",
        "records": [{
            "id": CORRECTION_ID,
            "target": {"path": path, "registration": system.registration_ordinal,
                "projection": {"kind": "callable_field", "function": "Read",
                    "lane": "returns", "member": "value", "property": "type"}},
            "expected_source_sha256": source_sha, "expected_raw_sha256": raw_digest,
            "before": {"kind": "text", "value": returned.type_name},
            "after": {"kind": "text", "value": "FixtureRecord"},
            "reviewer": "synthetic-fixture", "rationale": "Exercise one exact Type overlay",
            "evidence": [{"revision": revision, "path": path, "sha256": source_sha}]
        }]
    }))
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> TestResult {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)?;
    Ok(())
}

fn find_profile<'a>(value: &'a Value, profile: &str, found: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object) => {
            if object.get("schema").and_then(Value::as_str) == Some(profile) {
                found.push(value);
            }
            for child in object.values() {
                find_profile(child, profile, found);
            }
        }
        Value::Array(values) => {
            for child in values {
                find_profile(child, profile, found);
            }
        }
        _ => {}
    }
}

fn has_applied_correction(value: &Value, id: &str) -> bool {
    match value {
        Value::Object(object) => {
            (object.get("correction_id").and_then(Value::as_str) == Some(id)
                && object.get("status").and_then(Value::as_str) == Some("applied"))
                || object
                    .values()
                    .any(|child| has_applied_correction(child, id))
        }
        Value::Array(values) => values.iter().any(|child| has_applied_correction(child, id)),
        _ => false,
    }
}

fn contains_value(value: &Value, expected: &Value) -> bool {
    value == expected
        || match value {
            Value::Object(object) => object.values().any(|child| contains_value(child, expected)),
            Value::Array(values) => values.iter().any(|child| contains_value(child, expected)),
            _ => false,
        }
}

fn require_one_profile<'a>(value: &'a Value, profile: &str) -> TestResult<&'a Value> {
    let mut found = Vec::new();
    find_profile(value, profile, &mut found);
    let [one] = found.as_slice() else {
        return Err(format!("expected one {profile} observation").into());
    };
    Ok(one)
}

// Deliberately ignored by ordinary test runs. Root supplies one fresh export path
// to retain the exact CLI host/requests and observed JSON/Markdown envelopes.
#[test]
#[ignore = "focused native type-resolution witness; requires a fresh export directory"]
fn selected_corrections_reopen_into_primary_reference_facts() -> TestResult {
    use crate::{
        context::{
            CONTEXT_MAP_HOST_SCHEMA, ContextBuildRequest, ContextInspectLevel,
            ContextInspectRequest, ContextL1Profile, ContextMapHostInput, ContextMapPublication,
            ContextMapReferenceGuard, ContextMapRoot, ContextMapStatus, ContextRenderSelection,
            execute_context_build, execute_context_inspect,
        },
        graph::GraphBuildRequest,
        live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project},
    };
    use wow_core::CallableLoweringLane;
    use wow_project::disk::ProjectDiskFile;

    let export = PathBuf::from(
        std::env::var_os("WOW_NATIVE_TYPE_RESOLUTION_EXPORT")
            .ok_or("set WOW_NATIVE_TYPE_RESOLUTION_EXPORT to a fresh absolute directory")?,
    );
    if !export.is_absolute() || export.exists() {
        return Err("export must be fresh and absolute".into());
    }
    std::fs::create_dir(&export)?;
    let (root, mut config) = native_fixture()?;
    add_main_and_manifest(&root, &mut config)?;
    let revision = config["native_reference"]["profile"]["revision"]
        .as_str()
        .ok_or("fixture revision")?;
    let corrections = correction_pack(&root, revision)?;
    let correction_file = write_json(&root, "corrections.json", &corrections)?;
    let native_manifest = json!({
        "schema": crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id": config["project_id"], "workspace_id": config["workspace_id"],
        "source_origin_id": config["source_origin_id"], "logical_root": config["logical_root"],
        "profile": config["native_reference"]["profile"], "analyzer": config["analyzer"],
        "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
        "native_source": config["native_reference"]["native_source"],
        "native_api_uses": "reference_v1",
        "annotation_inputs": {"corrections": correction_file}
    });
    write_json(&root, "native-project.json", &native_manifest)?;
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("native-project.json"), &stop)?;
    let retained_input = input
        .bundle
        .configuration()
        .native_library_input()
        .ok_or("captured native producer input")?;
    let retained_wire: Value = serde_json::from_slice(retained_input.bytes())?;
    assert_eq!(
        retained_wire["schema"],
        "wow-service/native-library-producer-input/2"
    );
    let retained_corrections: Value = serde_json::from_str(
        retained_wire["annotation_inputs"]["selected_resources"]["correction_json"]
            .as_str()
            .ok_or("retained selected corrections")?,
    )?;
    assert_eq!(retained_corrections, corrections);
    let request = GraphBuildRequest::new(
        input.bundle.configuration().project_id().as_str().into(),
        "current".into(),
    )?
    .with_input_profile(&input);
    let store_path = export.join("registered-store");
    publish_local_project(
        &root.0.join("native-project.json"),
        &request,
        &store_path,
        &LiveProjectPublishRequest::new("fixture-native-type-resolution", "absent", true, true)?,
        &stop,
    )?;
    drop(input);
    std::fs::remove_dir_all(&root.0)?;
    assert!(
        !root.0.exists(),
        "all selected source/correction inputs removed before reopen"
    );

    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    let rebuilt = read.requalify_native_library(&stop)?;
    assert_eq!(
        rebuilt.input_failure_count(),
        1,
        "fixture's malformed API file remains visible"
    );
    let reference = &rebuilt.reference().view;
    let wow_reference::LookupResult::Found(record) = reference.lookup(
        wow_reference::native_view::NATIVE_API_PARTITION,
        "function:C_Transport.Read",
    ) else {
        return Err("retained native callable missing".into());
    };
    let original_signature =
        serde_json::to_value(rebuilt.reference().callable_signature(record, &stop)?)?;
    let lowering = rebuilt
        .callable_bindings()
        .callable_lowering(record, &stop)
        .map_err(|error| format!("owner lowering acquisition failed: {error:?}"))?
        .ok_or("retained native callable lowering missing")?;
    let owner_lowering: Value = serde_json::to_value(lowering)?;
    assert_eq!(
        owner_lowering["schema"],
        "wow-core/callable-lowering-observation/1"
    );
    assert_eq!(
        lowering.profile_id(),
        rebuilt.profile().profile_id().as_str()
    );
    assert_eq!(lowering.reference_generation(), reference.generation_id());
    assert_eq!(lowering.reference_view_digest(), reference.self_digest());
    assert_eq!(lowering.record_key(), record.key());
    assert_eq!(
        lowering.record_digest(),
        original_signature["record_digest"]
    );
    assert_eq!(lowering.lowering_syntax(), "emitted_analysis_only");
    assert_eq!(lowering.consumer_semantics(), "NotEvaluated");
    let annotation_file = rebuilt
        .annotation_files()
        .iter()
        .find(|file| file.path == lowering.generated_path())
        .ok_or("lowering generated annotation file")?;
    assert_eq!(annotation_file.sha256, lowering.generated_sha256());
    let binding = rebuilt.library_binding();
    let verified_file = binding.file_view(&annotation_file.path, &annotation_file.text, &stop)?;
    assert_eq!(verified_file.path(), lowering.generated_path());
    assert_eq!(verified_file.sha256(), lowering.generated_sha256());
    assert_eq!(lowering.fields().len(), 2);
    let byte_range = |span: wow_core::SourceSpan| -> TestResult<std::ops::Range<usize>> {
        Ok(
            usize::try_from(span.byte_start().ok_or("missing byte start")?)?
                ..usize::try_from(span.byte_end().ok_or("missing byte end")?)?,
        )
    };
    let parameter = &lowering.fields()[0];
    assert_eq!(parameter.lane(), CallableLoweringLane::Parameter);
    assert_eq!(parameter.ordinal(), 0);
    assert_eq!(parameter.lowered_type(), "boolean");
    assert!(parameter.optional());
    let parameter_optional_marker = byte_range(
        parameter
            .optional_marker_span()
            .ok_or("parameter optional marker span")?,
    )?;
    assert_eq!(
        verified_file.slice(
            parameter_optional_marker.start,
            parameter_optional_marker.end,
            &stop,
        )?,
        "?"
    );
    let parameter_token = byte_range(parameter.type_token_span())?;
    assert_eq!(
        verified_file.slice(parameter_token.start, parameter_token.end, &stop)?,
        "boolean"
    );
    let returned_lowering = &lowering.fields()[1];
    assert_eq!(returned_lowering.lane(), CallableLoweringLane::Return);
    assert_eq!(returned_lowering.ordinal(), 0);
    // The fixture explicitly corrects number -> FixtureRecord before emission;
    // the original Reference source signature intentionally keeps number.
    assert_eq!(returned_lowering.lowered_type(), "FixtureRecord");
    assert!(returned_lowering.optional());
    let return_optional_marker = byte_range(
        returned_lowering
            .optional_marker_span()
            .ok_or("return optional marker span")?,
    )?;
    assert_eq!(
        verified_file.slice(
            return_optional_marker.start,
            return_optional_marker.end,
            &stop,
        )?,
        "?"
    );
    let return_token = byte_range(returned_lowering.type_token_span())?;
    assert_eq!(
        verified_file.slice(return_token.start, return_token.end, &stop)?,
        "FixtureRecord"
    );
    let mut corrupted_annotation = annotation_file.text.clone();
    corrupted_annotation.replace_range(parameter_token.clone(), &"x".repeat(parameter_token.len()));
    assert!(matches!(
        binding.file_view(&annotation_file.path, &corrupted_annotation, &stop),
        Err(error) if error.code() == wow_project::ProjectErrorCode::FileDigestMismatch
    ));
    assert!(matches!(
        binding.file_view(&annotation_file.path, &annotation_file.text, &AtomicBool::new(true)),
        Err(error) if error.code() == wow_project::ProjectErrorCode::AnalysisCancelled
    ));
    assert!(matches!(
        rebuilt
            .callable_bindings()
            .callable_lowering(record, &AtomicBool::new(true)),
        Err(wow_annotations::reference_links::NativeCallableBindingError::Cancelled)
    ));
    assert_eq!(
        assert_transport_signature(rebuilt.reference().callable_signature(record, &stop)?)?,
        original_signature
    );
    let owner_observation = original_signature.clone();
    let type_signature = require_one_profile(&owner_observation, TYPE_PROFILE)?.clone();
    let fields = type_signature["fields"]
        .as_array()
        .ok_or("owner type fields")?;
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0]["original_type"], "bool");
    assert_eq!(fields[0]["status"], "unresolved");
    let returned = &fields[1];
    assert_eq!(returned["lane"], "returns");
    assert_eq!(returned["ordinal"], 0);
    assert_eq!(returned["original_type"], "number");
    assert_eq!(returned["effective_type"], "FixtureRecord");
    assert_eq!(returned["status"], "resolved");
    assert_eq!(returned["candidate_count"], 1);
    assert_eq!(returned["candidates_truncated"], false);
    let target = &returned["candidates"][0];
    assert_eq!(target["kind"], "structure");
    assert_eq!(target["original_name"], "FixtureRecord");
    assert_eq!(target["effective_name"], "FixtureRecord");
    assert_eq!(target["path"], "Blizzard_APIDocumentationGenerated/API.lua");
    assert_eq!(
        target["sha256"],
        corrections["records"][0]["expected_source_sha256"]
    );
    assert!(
        contains_value(&type_signature, &json!("FixtureRecord")),
        "fixture Type correction is reflected by the Reference owner"
    );
    assert!(
        has_applied_correction(&owner_observation, CORRECTION_ID),
        "exact Type correction application is retained"
    );

    let graph_read = read.pair().graph_read(&stop)?;
    let producer = graph_read
        .owner()
        .partition("wow-service.native-api-callables")
        .ok_or("original native API partition")?;
    let [accepted] = producer.report().accepted_entities() else {
        return Err("fixture requires one original accepted native callable".into());
    };
    let node =
        graph_read.published_entity(producer.partition_id(), accepted.proposal_id(), &stop)?;
    let guard = ContextMapReferenceGuard::new(
        rebuilt.profile().profile_id().as_str(),
        reference.generation_id(),
        reference.self_digest(),
    )?;
    let publication = ContextMapPublication::store_generation(read.store_generation_id().as_str())?;
    let inspect_request = ContextInspectRequest::primary(
        CONTEXT_STORE_ID,
        publication.clone(),
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(node.node_id().as_str())?,
        None,
        Some(guard.clone()),
    )?
    .with_l1_profile(ContextL1Profile::ReferenceFactsV1)?;
    let build_request = ContextBuildRequest::primary(
        CONTEXT_STORE_ID,
        publication,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(node.node_id().as_str())?,
        ContextRenderSelection::Both,
        None,
        Some(guard),
    )?
    .with_l1_profile(ContextL1Profile::ReferenceFactsV1)?;
    let host_value = json!({"schema":CONTEXT_MAP_HOST_SCHEMA, "project_store_id":CONTEXT_STORE_ID,
        "project_store_root":store_path, "project_id":read.project().configuration().project_id(),
        "reference_profile":rebuilt.profile(), "reference":reference,
        "limits":{"neighbors":wow_graph::GraphNeighborReadLimits::default(),
            "explain":wow_graph::GraphExplainLimits::default(), "evidence":wow_graph::GraphEvidenceResolveLimits::default(),
            "max_direct_edges":16,"max_input_bytes":4*1024*1024,"max_metadata_bytes":4*1024*1024,"max_context_work":8_000_000}});
    let host_bytes = wow_core::canonical_json_bytes(&host_value)?;
    let inspect_bytes = wow_core::canonical_json_bytes(&serde_json::to_value(&inspect_request)?)?;
    let build_bytes = wow_core::canonical_json_bytes(&serde_json::to_value(&build_request)?)?;
    let host = ContextMapHostInput::from_json_slice(&host_bytes, &stop)?;
    drop(rebuilt);
    read.close()?;
    store.close()?;
    assert!(!root.0.exists());

    let inspected = execute_context_inspect(&host, &inspect_request, &stop)?;
    let built = execute_context_build(&host, &build_request, &stop)?;
    assert_eq!(inspected.status(), ContextMapStatus::Partial);
    assert_eq!(built.status(), ContextMapStatus::Partial);
    let inspect_json: Value = serde_json::from_slice(inspected.canonical_bytes())?;
    let build_json: Value = serde_json::from_slice(built.canonical_bytes())?;
    for envelope in [&inspect_json, &build_json] {
        assert_eq!(envelope["resource_closure"], "closed");
        assert!(envelope.get("failure_code").is_none());
        assert_eq!(
            require_one_profile(envelope, TYPE_PROFILE)?,
            &type_signature
        );
        assert_eq!(
            require_one_profile(envelope, "wow-core/callable-lowering-observation/1")?,
            &owner_lowering,
            "Primary ReferenceFacts preserves the exact owner lowering observation"
        );
        assert!(
            contains_value(envelope, &original_signature["signature"]),
            "original source signature remains unchanged"
        );
    }
    assert_eq!(
        inspect_json["payload"]["skeletons"]["l1"]["core"]["schema"],
        "wow-context/l1-skeleton/primary-reference-facts/1"
    );
    let renderers = build_json["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("Primary renderers")?;
    assert_eq!(renderers.len(), 2);
    assert_eq!(
        renderers[0]["core"]["profile"]["profile_id"],
        crate::context::PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID
    );
    assert_eq!(
        renderers[1]["core"]["profile"]["profile_id"],
        crate::context::PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID
    );
    let pack = &build_json["payload"]["semantic_pack"];
    assert_eq!(
        renderers[0]["core"]["content"]
            .as_str()
            .ok_or("JSON content")?
            .as_bytes(),
        wow_core::canonical_json_bytes(pack)?
    );
    let markdown = renderers[1]["core"]["content"]
        .as_str()
        .ok_or("Markdown content")?;
    let skeleton_record = renderers[1]["core"]["trace"]
        .as_array()
        .ok_or("Markdown trace")?
        .iter()
        .find(|row| row["template_id"] == "native_skeletons")
        .ok_or("native skeleton record")?;
    let start = usize::try_from(
        skeleton_record["byte_start"]
            .as_u64()
            .ok_or("record start")?,
    )?;
    let end = usize::try_from(skeleton_record["byte_end"].as_u64().ok_or("record end")?)?;
    let quoted = markdown
        .get(start..end)
        .ok_or("UTF-8 record")?
        .strip_prefix("SKELETONS ")
        .and_then(|line| line.strip_suffix('\n'))
        .ok_or("quoted skeleton record")?;
    let decoded: String = serde_json::from_str(quoted)?;
    let skeletons: Value = serde_json::from_str(&decoded)?;
    assert_eq!(
        skeletons, pack["core"]["skeletons"],
        "Markdown preserves the full typed ReferenceFacts data"
    );
    assert_eq!(
        require_one_profile(&skeletons, "wow-core/callable-lowering-observation/1")?,
        &owner_lowering,
        "Markdown preserves the exact owner lowering observation"
    );

    let work = build_json["payload"]["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("Build work")?;
    assert!(work > 0, "the selected Build path must charge work");
    let mut low_host = host_value.clone();
    low_host["limits"]["max_context_work"] = json!(work / 2);
    let low_host =
        ContextMapHostInput::from_json_slice(&wow_core::canonical_json_bytes(&low_host)?, &stop)?;
    let refused = execute_context_build(&low_host, &build_request, &stop)?;
    assert_eq!(refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        refused.failure_code(),
        Some(crate::ServiceErrorCode::BudgetExceeded)
    );
    let refused_json: Value = serde_json::from_slice(refused.canonical_bytes())?;
    assert_eq!(refused_json["resource_closure"], "closed");
    assert!(refused_json.get("payload").is_none());
    let cancelled = execute_context_build(&host, &build_request, &AtomicBool::new(true))?;
    assert_eq!(cancelled.status(), ContextMapStatus::Cancelled);
    assert_eq!(
        cancelled.failure_code(),
        Some(crate::ServiceErrorCode::Cancelled)
    );

    for (name, bytes) in [
        (
            "native-project.json",
            wow_core::canonical_json_bytes(&native_manifest)?,
        ),
        (
            "corrections.json",
            wow_core::canonical_json_bytes(&corrections)?,
        ),
        ("host.json", host_bytes),
        ("inspect-request.json", inspect_bytes),
        ("build-request.json", build_bytes),
        (
            "expected-inspect-envelope.json",
            inspected.canonical_bytes().to_vec(),
        ),
        (
            "expected-build-envelope.json",
            built.canonical_bytes().to_vec(),
        ),
        (
            "expected-work-refusal-envelope.json",
            refused.canonical_bytes().to_vec(),
        ),
        (
            "expected-cancelled-envelope.json",
            cancelled.canonical_bytes().to_vec(),
        ),
        (
            "owner-type-signature.json",
            wow_core::canonical_json_bytes(&type_signature)?,
        ),
        (
            "owner-callable-lowering.json",
            wow_core::canonical_json_bytes(&owner_lowering)?,
        ),
        (
            "owner-reference-observation.json",
            wow_core::canonical_json_bytes(&owner_observation)?,
        ),
        (
            "original-signature.json",
            wow_core::canonical_json_bytes(&original_signature)?,
        ),
        (
            "source-absence.json",
            wow_core::canonical_json_bytes(
                &json!({"source_and_corrections_removed_before_reopen":true,"fixture_claim":"synthetic only"}),
            )?,
        ),
    ] {
        write_new(&export.join(name), &bytes)?;
    }
    Ok(())
}
