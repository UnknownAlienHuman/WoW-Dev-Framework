//! Focused external-reader/render witnesses over the retained native pack fixture.
//!
//! Intended as a child module of `wow-service::context::tests::build`, where it
//! reuses that module's established fixture constructors and Service harness.
//! The JSON renderer's `content` is the native owner's exact canonical pack
//! byte slice; this test never rebuilds or blesses the positive artifact by
//! serializing a decoded JSON value.

use super::*;
use std::sync::atomic::AtomicBool;

const READER_MAX_BYTES: u64 = 4 * 1024 * 1024;
const READER_MAX_WORK: u64 = 8_000_000;
const READER_MAX_TOKENS: u64 = 2_000_000;

struct PackScratch {
    root: PathBuf,
    preserve: bool,
}

impl PackScratch {
    fn new() -> TestResult<Self> {
        let export = std::env::var_os("WOW_CONTEXT_VALIDATE_EXPORT");
        let preserve = export.is_some();
        let root = export.map_or_else(
            || {
                std::env::temp_dir().join(format!(
                    "wow-context-external-pack-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos(),
                ))
            },
            PathBuf::from,
        );
        if !root.is_absolute() || root.as_os_str().len() > 32768 {
            return Err("pack witness export requires a bounded absolute fresh path".into());
        }
        std::fs::create_dir(&root)?;
        Ok(Self { root, preserve })
    }
}

impl Drop for PackScratch {
    fn drop(&mut self) {
        if !self.preserve {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn reader_limits(max_input_bytes: u64) -> TestResult<wow_context::PrimaryPackValidationLimits> {
    Ok(wow_context::PrimaryPackValidationLimits::new(
        max_input_bytes,
        READER_MAX_BYTES,
        READER_MAX_WORK,
        128,
        READER_MAX_TOKENS,
    )?)
}

fn native_rendered_artifact<'a>(envelope: &'a Value, profile_id: &str) -> TestResult<&'a Value> {
    let payload = envelope.get("payload").ok_or("native build payload")?;
    let artifacts = rows(&payload["rendered_artifacts"])?;
    let mut matching = artifacts
        .iter()
        .filter(|artifact| artifact["core"]["profile"]["profile_id"].as_str() == Some(profile_id));
    let artifact = matching.next().ok_or("native renderer artifact")?;
    if matching.next().is_some() {
        return Err("duplicate native renderer artifact".into());
    }
    Ok(artifact)
}

fn rendered_native_pack_bytes(result: &ContextMapResult) -> TestResult<Vec<u8>> {
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    let artifact = native_rendered_artifact(&envelope, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID)?;
    let core = &artifact["core"];
    let content = core["content"]
        .as_str()
        .ok_or("native canonical pack content")?;
    if core["byte_length"].as_u64() != Some(content.len() as u64) {
        return Err("native renderer byte length mismatch".into());
    }
    Ok(content.as_bytes().to_vec())
}

fn native_l0_and_l1_bytes() -> TestResult<(PackScratch, Vec<u8>, Vec<u8>, ContextMapResult)> {
    let stop = AtomicBool::new(false);
    let scratch = PackScratch::new()?;
    let store_root = scratch.root.join("registered-store");

    // Reuse the canonical two-file native fixture and its real graph/project
    // publishers from the existing retained-native-pack test.
    let bundle = input_bundle_files(vec![
        ProjectInputFile::new(ADMITTED_PATH, LUA)?,
        ProjectInputFile::new("other.lua", PROJECT_OTHER_LUA)?,
    ])?;
    let (publisher, graph, reference, profile) = owners_from_bundle(bundle, &stop)?;
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(
        &publisher,
        &graph,
        "fixture-context-external-pack-witness",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);

    let expected = retained_build(&store_root, &publication.generation_id, &stop)?;
    let selected = rows(&expected["files"])?
        .iter()
        .find(|file| file["file"]["path"] == ADMITTED_PATH)
        .ok_or("selected native source file")?;
    let root: GraphNodeId = serde_json::from_value(selected["node"]["node_id"].clone())?;
    let mut host_value = host_value(&store_root, &profile, &reference);
    host_value["limits"]["max_context_work"] = json!(FIXTURE_WORK);
    host_value["limits"]["max_metadata_bytes"] = json!(FIXTURE_METADATA);
    let host = decode_host(&host_value, &stop)?;
    let publication = ContextMapPublication::store_generation(publication.generation_id.as_str())?;
    let guard = Some(reference_guard(&profile, &reference)?);

    let l0_request = ContextBuildRequest::primary(
        STORE_ID,
        publication.clone(),
        ContextInspectLevel::L0,
        ContextMapRoot::graph_entity(root.as_str())?,
        ContextRenderSelection::Both,
        None,
        guard.clone(),
    )?;
    let l0_result = execute_context_build(&host, &l0_request, &stop)?;
    let l0_bytes = rendered_native_pack_bytes(&l0_result)?;

    // Build a second genuine owner artifact for the explicit unsupported-level
    // case; do not synthesize an L1 header around L0 bytes.
    let l1_request = ContextBuildRequest::primary(
        STORE_ID,
        publication,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(root.as_str())?,
        ContextRenderSelection::Json,
        None,
        guard,
    )?;
    let l1_result = execute_context_build(&host, &l1_request, &stop)?;
    let l1_bytes = rendered_native_pack_bytes(&l1_result)?;
    if scratch.preserve {
        for (name, bytes) in [
            ("host.json", wow_core::canonical_json_bytes(&host_value)?),
            (
                "build-request.json",
                wow_core::canonical_json_bytes(&l0_request)?,
            ),
            ("native-l0.semantic-pack.json", l0_bytes.clone()),
            ("native-l1.semantic-pack.json", l1_bytes.clone()),
        ] {
            std::fs::write(scratch.root.join(name), bytes)?;
        }
    }
    Ok((scratch, l0_bytes, l1_bytes, l0_result))
}

fn tamper_pack_identity(bytes: &[u8]) -> TestResult<Vec<u8>> {
    const MARKER: &[u8] = b"\"semantic_pack_id\":\"context-pack:sha256:";
    let marker = bytes
        .windows(MARKER.len())
        .position(|window| window == MARKER)
        .ok_or("native semantic pack identity field")?;
    let mut tampered = bytes.to_vec();
    let digit = marker + MARKER.len();
    let current = *tampered.get(digit).ok_or("semantic pack digest byte")?;
    if !current.is_ascii_hexdigit() {
        return Err("semantic pack identity digest is malformed".into());
    }
    tampered[digit] = if current == b'0' { b'1' } else { b'0' };
    Ok(tampered)
}

fn domain_identity(core: &Value, prefix: &str) -> TestResult<String> {
    let domain = core["schema"].as_str().ok_or("identity schema")?;
    let digest = wow_core::domain_separated_digest(domain, core)?;
    let text = wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(digest).to_string();
    let hex = text
        .strip_prefix("sha256:")
        .ok_or("canonical digest prefix")?;
    Ok(format!("{prefix}{hex}"))
}

fn project_map_mut(pack: &mut Value) -> &mut Value {
    &mut pack["core"]["skeletons"]["l0"]["core"]["context"]["origins"]
}

fn reseal_native_pack(pack: &mut Value) -> TestResult<()> {
    let map = project_map_mut(pack);
    map["project_map_id"] = json!(domain_identity(&map["core"], "project-map:sha256:")?);
    let map_id = map["project_map_id"]
        .as_str()
        .ok_or("recomputed ProjectMap ID")?
        .to_owned();

    let l0 = &mut pack["core"]["skeletons"]["l0"];
    l0["core"]["context"]["origin_map_id"] = json!(map_id);
    l0["l0_skeleton_id"] = json!(domain_identity(&l0["core"], "l0-skeleton:sha256:")?);
    let l0_id = l0["l0_skeleton_id"]
        .as_str()
        .ok_or("recomputed L0 skeleton ID")?
        .to_owned();

    let items = pack["core"]["items"]
        .as_array_mut()
        .ok_or("native context items")?;
    for item in items {
        item["core"]["origin_map_id"] = json!(map_id);
        item["context_item_id"] = json!(domain_identity(&item["core"], "context-item:sha256:")?);
    }
    let item_links = pack["core"]["items"]
        .as_array()
        .ok_or("reidentified context items")?
        .iter()
        .map(|item| {
            Ok((
                item["context_item_id"]
                    .as_str()
                    .ok_or("recomputed context item ID")?
                    .to_owned(),
                item["core"]["origin_map_id"]
                    .as_str()
                    .ok_or("rebound context item map ID")?
                    .to_owned(),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let trace = pack["core"]["selection_trace"]
        .as_array_mut()
        .ok_or("native selection trace")?;
    if trace.len() != item_links.len() {
        return Err("native item and trace cardinality changed".into());
    }
    for (selection, (item_id, origin_map_id)) in trace.iter_mut().zip(item_links) {
        selection["item_id"] = json!(item_id);
        selection["origin_map_id"] = json!(origin_map_id);
    }

    let upstream = pack["core"]["omissions"]["upstream"]
        .as_array_mut()
        .ok_or("native upstream omissions")?;
    for omission in upstream {
        match omission["producer"].as_str() {
            Some("project_map") => omission["artifact_id"] = json!(map_id),
            Some("l0_skeleton") => omission["artifact_id"] = json!(l0_id),
            _ => {}
        }
    }
    pack["semantic_pack_id"] = json!(domain_identity(&pack["core"], "context-pack:sha256:")?);
    Ok(())
}

fn mutate_and_reseal_native_pack(
    bytes: &[u8],
    mutate: impl FnOnce(&mut Value) -> TestResult<()>,
) -> TestResult<Vec<u8>> {
    let mut pack: Value = serde_json::from_slice(bytes)?;
    mutate(&mut pack)?;
    reseal_native_pack(&mut pack)?;
    Ok(wow_core::canonical_json_bytes(&pack)?)
}

fn assert_resealed_owner_mutation_is_rejected(
    bytes: &[u8],
    stop: &AtomicBool,
    label: &str,
    mutate: impl FnOnce(&mut Value) -> TestResult<()>,
) -> TestResult<()> {
    let mutated = mutate_and_reseal_native_pack(bytes, mutate)?;
    let validation =
        wow_context::validate_primary_l0_pack(&mutated, reader_limits(READER_MAX_BYTES)?, stop)?;
    assert_eq!(
        validation.digest_and_identity(),
        wow_context::ValidationState::Valid,
        "{label} must retain valid enclosing canonical identities: {validation:?}"
    );
    assert_ne!(
        validation.overall_state(),
        wow_context::ValidationState::Valid,
        "{label} must fail owner-record admission: {validation:?}"
    );
    Ok(())
}

fn forge_digest_record_id(record: &mut Value, field: &str) -> TestResult<()> {
    let current = record[field]
        .as_str()
        .ok_or("owner record identity string")?;
    let (prefix, digest) = current
        .rsplit_once(':')
        .ok_or("owner record identity domain")?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("owner record identity must end in a 64-digit digest".into());
    }
    let replacement = if current.ends_with('0') { '1' } else { '0' };
    record[field] = json!(format!("{prefix}:{}{replacement}", &digest[..63]));
    Ok(())
}

fn remove_native_derivation_chain(bytes: &[u8], complete: bool) -> TestResult<Vec<u8>> {
    let mut pack: Value = serde_json::from_slice(bytes)?;
    let map = &mut pack["core"]["skeletons"]["l0"]["core"]["context"]["origins"];
    let supports = map["core"]["supports"]
        .as_array_mut()
        .ok_or("native ProjectMap support receipts")?;
    let mut removed_records = 0usize;
    for support in supports {
        let explanation = &mut support["explanation"];
        removed_records = removed_records
            .checked_add(
                explanation["assertion_supports"]
                    .as_array()
                    .ok_or("native assertion support chain")?
                    .len(),
            )
            .and_then(|count| count.checked_add(explanation["derivations"].as_array()?.len()))
            .ok_or("removed assertion count overflow")?;
        explanation["assertion_supports"] = json!([]);
        explanation["derivations"] = json!([]);
        explanation["derivation_complete"] = json!(complete);
    }
    if removed_records == 0 {
        return Err("genuine L0 witness had no native local chain to remove".into());
    }
    reseal_native_pack(&mut pack)?;
    Ok(wow_core::canonical_json_bytes(&pack)?)
}

#[test]
fn external_primary_pack_reader_accepts_one_native_l0_and_keeps_limits_and_authority_typed()
-> TestResult {
    let (_scratch, l0_bytes, l1_bytes, _native_result) = native_l0_and_l1_bytes()?;
    let stop = AtomicBool::new(false);

    let valid =
        wow_context::validate_primary_l0_pack(&l0_bytes, reader_limits(READER_MAX_BYTES)?, &stop)?;
    assert_eq!(
        valid.overall_state(),
        wow_context::ValidationState::Valid,
        "{valid:?}"
    );
    assert_eq!(valid.canonical_bytes(), wow_context::ValidationState::Valid);
    assert_eq!(valid.schema(), wow_context::ValidationState::Valid);
    assert_eq!(
        valid.compiled_profile(),
        wow_context::ValidationState::Valid
    );
    assert_eq!(
        valid.digest_and_identity(),
        wow_context::ValidationState::Valid
    );
    assert_eq!(
        valid.internal_references(),
        wow_context::ValidationState::Valid
    );
    assert_eq!(
        valid.owner_generation_closure(),
        wow_context::ValidationState::NotEvaluated
    );
    assert_eq!(
        valid.source_or_runtime_authority(),
        wow_context::ValidationState::NotEvaluated
    );
    assert_eq!(
        valid.origin_closure(),
        wow_context::PrimaryPackOriginClosure::StructuralOnly
    );
    assert!(valid.nonrepairing());
    assert!(valid.semantic_pack_id().is_some());
    assert_eq!(valid.byte_length(), l0_bytes.len() as u64);
    assert_eq!(valid.artifact_digest().len(), "sha256:".len() + 64);
    assert_eq!(
        valid.selection().map(|selection| selection.level()),
        Some(wow_context::PrimaryPackLevel::L0)
    );

    let incomplete_chain = remove_native_derivation_chain(&l0_bytes, false)?;
    let incomplete = wow_context::validate_primary_l0_pack(
        &incomplete_chain,
        reader_limits(READER_MAX_BYTES)?,
        &stop,
    )?;
    assert_eq!(
        incomplete.overall_state(),
        wow_context::ValidationState::NotEvaluated,
        "a native receipt that declares its local derivation chain incomplete is never Valid"
    );
    assert_eq!(
        incomplete.digest_and_identity(),
        wow_context::ValidationState::Valid,
        "the negative retains correctly recomputed enclosing native identities"
    );

    let missing_chain = remove_native_derivation_chain(&l0_bytes, true)?;
    let missing = wow_context::validate_primary_l0_pack(
        &missing_chain,
        reader_limits(READER_MAX_BYTES)?,
        &stop,
    )?;
    assert_eq!(
        missing.overall_state(),
        wow_context::ValidationState::NotEvaluated,
        "claiming completeness cannot replace removed non-Proven derivation records"
    );
    assert_eq!(
        missing.digest_and_identity(),
        wow_context::ValidationState::Valid,
        "the missing-chain negative retains correctly recomputed enclosing identities"
    );

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "partial graph coverage with negative authority",
        |pack| {
            let coverage = project_map_mut(pack)["core"]["navigation"]["coverage"]
                .as_array_mut()
                .ok_or("native graph coverage rows")?;
            let row = coverage
                .first_mut()
                .ok_or("native graph coverage row for mutation")?;
            row["state"] = json!("partial");
            row["negative_authority"] = json!(true);
            Ok(())
        },
    )?;

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "forged project CoverageRecord ID",
        |pack| {
            let coverage = project_map_mut(pack)["core"]["project_coverage"]
                .as_array_mut()
                .ok_or("native project coverage rows")?;
            let row = coverage
                .first_mut()
                .ok_or("native project CoverageRecord for mutation")?;
            forge_digest_record_id(row, "coverage_id")
        },
    )?;

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "forged NotEvaluatedRecord ID",
        |pack| {
            let deferred = project_map_mut(pack)["core"]["deferred_project_capabilities"]
                .as_array_mut()
                .ok_or("native deferred project capabilities")?;
            let row = deferred
                .first_mut()
                .ok_or("native NotEvaluatedRecord for mutation")?;
            forge_digest_record_id(&mut row["record"], "not_evaluated_id")
        },
    )?;

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "duplicate NotEvaluatedRecord blocker",
        |pack| {
            let deferred = project_map_mut(pack)["core"]["deferred_project_capabilities"]
                .as_array_mut()
                .ok_or("native deferred project capabilities")?;
            let row = deferred
                .first_mut()
                .ok_or("native NotEvaluatedRecord for blocker mutation")?;
            let blockers = row["record"]["blocking_capability_ids"]
                .as_array_mut()
                .ok_or("NotEvaluatedRecord blocker IDs")?;
            let blocker = blockers
                .first()
                .cloned()
                .ok_or("NotEvaluatedRecord blocker for duplication")?;
            blockers.push(blocker);
            let record: wow_core::NotEvaluatedRecord =
                serde_json::from_value(row["record"].clone())?;
            row["record"]["not_evaluated_id"] =
                serde_json::to_value(wow_core::derive_not_evaluated_id(&record)?)?;
            Ok(())
        },
    )?;

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "duplicate Reference coverage partition",
        |pack| {
            let rows = project_map_mut(pack)["core"]["reference_coverage"]
                .as_array_mut()
                .ok_or("native Reference coverage rows")?;
            let row = match rows.first() {
                Some(row) => row.clone(),
                None => {
                    let row = json!({"partition_id": "witness-partition", "state": "partial"});
                    rows.push(row.clone());
                    row
                }
            };
            rows.push(row);
            Ok(())
        },
    )?;

    assert_resealed_owner_mutation_is_rejected(
        &l0_bytes,
        &stop,
        "malformed ReferenceConflict candidate population",
        |pack| {
            let map = project_map_mut(pack);
            let coverage = map["core"]["reference_coverage"]
                .as_array_mut()
                .ok_or("native Reference coverage rows")?;
            if coverage.is_empty() {
                coverage.push(json!({"partition_id": "witness-partition", "state": "partial"}));
            }
            let partition_id = map["core"]["reference_coverage"]
                .as_array()
                .and_then(|rows| rows.first())
                .and_then(|row| row["partition_id"].as_str())
                .ok_or("native Reference partition for conflict mutation")?
                .to_owned();
            let conflicts = map["core"]["reference_conflicts"]
                .as_array_mut()
                .ok_or("native Reference conflict rows")?;
            let candidate = format!("sha256:{}", "0".repeat(64));
            conflicts.push(json!({
                "partition_id": partition_id,
                "key": "witness-conflict",
                "candidate_digests": [candidate.clone(), candidate],
                "source_ids": ["witness-source-a", "witness-source-b"]
            }));
            Ok(())
        },
    )?;

    let service_request = crate::context::ContextValidateRequest::primary_semantic_pack_structural(
        READER_MAX_BYTES,
        READER_MAX_BYTES,
        READER_MAX_WORK,
        128,
        READER_MAX_TOKENS,
    )?;
    let service = crate::context::execute_context_validate(&service_request, &l0_bytes, &stop)?;
    assert_eq!(
        service.operation_status(),
        ContextMapStatus::Complete,
        "service failure: {:?}",
        service.failure_code()
    );
    assert_eq!(
        service.validation_state(),
        Some(crate::context::ValidationState::Valid)
    );
    let envelope: Value = serde_json::from_slice(service.canonical_bytes())?;
    assert_eq!(
        envelope["schema"],
        crate::context::CONTEXT_VALIDATE_RESULT_SCHEMA
    );
    assert_eq!(
        envelope["payload"]["semantic_pack_id"].as_str(),
        valid.semantic_pack_id()
    );
    assert_eq!(
        envelope["payload"]["owner_generation_closure"],
        "not_evaluated"
    );
    assert_eq!(
        envelope["payload"]["source_or_runtime_authority"],
        "not_evaluated"
    );
    assert!(service.canonical_bytes().len() <= READER_MAX_BYTES as usize);

    let changed_identity = tamper_pack_identity(&l0_bytes)?;
    let invalid = wow_context::validate_primary_l0_pack(
        &changed_identity,
        reader_limits(READER_MAX_BYTES)?,
        &stop,
    )?;
    assert_eq!(
        invalid.overall_state(),
        wow_context::ValidationState::Invalid
    );
    assert_eq!(
        invalid.digest_and_identity(),
        wow_context::ValidationState::Invalid
    );
    assert_eq!(invalid.semantic_pack_id(), None);

    let unsupported =
        wow_context::validate_primary_l0_pack(&l1_bytes, reader_limits(READER_MAX_BYTES)?, &stop)?;
    assert_eq!(
        unsupported.overall_state(),
        wow_context::ValidationState::NotEvaluated
    );
    assert_eq!(unsupported.schema(), wow_context::ValidationState::Valid);
    assert_eq!(
        unsupported.digest_and_identity(),
        wow_context::ValidationState::NotEvaluated
    );
    assert_eq!(
        unsupported.selection().map(|selection| selection.level()),
        Some(wow_context::PrimaryPackLevel::L1)
    );

    let cancelled = AtomicBool::new(true);
    let error = wow_context::validate_primary_l0_pack(
        &l0_bytes,
        reader_limits(READER_MAX_BYTES)?,
        &cancelled,
    )
    .err()
    .ok_or("pre-cancelled validation must stop before parsing")?;
    assert_eq!(error.code(), wow_context::ContextErrorCode::Cancelled);

    let too_small = reader_limits((l0_bytes.len() as u64).saturating_sub(1))?;
    let error = wow_context::validate_primary_l0_pack(&l0_bytes, too_small, &stop)
        .err()
        .ok_or("caller byte cap must refuse the complete native artifact")?;
    assert_eq!(
        error.code(),
        wow_context::ContextErrorCode::MinimumRequiredContentExceedsBudget
    );

    let metadata_too_small = wow_context::PrimaryPackValidationLimits::new(
        READER_MAX_BYTES,
        (l0_bytes.len() as u64).saturating_sub(1),
        READER_MAX_WORK,
        128,
        READER_MAX_TOKENS,
    )?;
    let error = wow_context::validate_primary_l0_pack(&l0_bytes, metadata_too_small, &stop)
        .err()
        .ok_or("caller metadata cap must refuse the complete native artifact")?;
    assert_eq!(
        error.code(),
        wow_context::ContextErrorCode::MinimumRequiredContentExceedsBudget
    );

    let work_too_small = wow_context::PrimaryPackValidationLimits::new(
        READER_MAX_BYTES,
        READER_MAX_BYTES,
        1,
        128,
        READER_MAX_TOKENS,
    )?;
    let error = wow_context::validate_primary_l0_pack(&l0_bytes, work_too_small, &stop)
        .err()
        .ok_or("raw hashing and scanning must consume the caller work allowance")?;
    assert_eq!(
        error.code(),
        wow_context::ContextErrorCode::MinimumRequiredContentExceedsBudget
    );
    Ok(())
}

#[test]
fn external_primary_l0_render_matches_native_and_service_artifacts_without_custody() -> TestResult {
    use crate::context::{ContextRenderFormat, ContextRenderRequest, execute_context_render};
    use wow_context::{PrimaryPackRenderFormat, validate_and_render_primary_l0_pack};

    let (_scratch, l0_bytes, l1_bytes, native_result) = native_l0_and_l1_bytes()?;
    let native: Value = serde_json::from_slice(native_result.canonical_bytes())?;
    assert_eq!(rows(&native["payload"]["rendered_artifacts"])?.len(), 2);
    let stop = AtomicBool::new(false);
    let render_request = |format, work| {
        ContextRenderRequest::primary_semantic_pack_structural(
            format,
            READER_MAX_BYTES,
            READER_MAX_BYTES,
            work,
            128,
            READER_MAX_TOKENS,
        )
    };
    let mut markdown_work = 0;

    for (format, service_format, profile_id) in [
        (
            PrimaryPackRenderFormat::Json,
            ContextRenderFormat::Json,
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        ),
        (
            PrimaryPackRenderFormat::Markdown,
            ContextRenderFormat::Markdown,
            PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        ),
    ] {
        // Expected artifacts are borrowed from the one real native build. No
        // decoded input is reconstructed as a native pack or source custody.
        let expected = native_rendered_artifact(&native, profile_id)?;
        let expected_bytes = wow_core::canonical_json_bytes(expected)?;
        let rendered = validate_and_render_primary_l0_pack(
            &l0_bytes,
            reader_limits(READER_MAX_BYTES)?,
            format,
            &stop,
        )?;
        let validation = rendered.validation();
        assert_eq!(
            validation.overall_state(),
            wow_context::ValidationState::Valid
        );
        assert_eq!(
            validation.digest_and_identity(),
            wow_context::ValidationState::Valid
        );
        assert_eq!(
            validation.origin_closure(),
            wow_context::PrimaryPackOriginClosure::StructuralOnly
        );
        assert_eq!(
            validation.owner_generation_closure(),
            wow_context::ValidationState::NotEvaluated
        );
        assert_eq!(
            validation.source_or_runtime_authority(),
            wow_context::ValidationState::NotEvaluated
        );
        let artifact = rendered
            .artifact()
            .ok_or("valid external render artifact")?;
        assert_eq!(wow_core::canonical_json_bytes(artifact)?, expected_bytes);
        assert_eq!(
            artifact.content().as_bytes(),
            expected["core"]["content"]
                .as_str()
                .ok_or("native rendered content")?
                .as_bytes()
        );
        assert_eq!(
            artifact.content_digest().to_string(),
            expected["core"]["content_digest"]
                .as_str()
                .ok_or("native rendered digest")?
        );
        assert_eq!(
            serde_json::to_value(artifact.trace())?,
            expected["core"]["trace"]
        );
        if format == PrimaryPackRenderFormat::Markdown {
            markdown_work = rendered.total_work();
        }

        let service = execute_context_render(
            &render_request(service_format, READER_MAX_WORK)?,
            &l0_bytes,
            &stop,
        )?;
        assert_eq!(service.operation_status(), ContextMapStatus::Complete);
        assert_eq!(
            service.validation_state(),
            Some(wow_context::ValidationState::Valid)
        );
        assert_eq!(service.failure_code(), None);
        let envelope: Value = serde_json::from_slice(service.canonical_bytes())?;
        assert_eq!(
            envelope["schema"],
            crate::context::CONTEXT_RENDER_RESULT_SCHEMA
        );
        let payload = envelope.get("payload").ok_or("Service render payload")?;
        let work_usage = &payload["work_usage"];
        let input_hash_work = l0_bytes.len().div_ceil(8192) as u64;
        assert_eq!(
            work_usage["service_input_hash_work"].as_u64(),
            Some(input_hash_work)
        );
        assert_eq!(
            work_usage["owner_validation_and_render_work"].as_u64(),
            Some(rendered.total_work())
        );
        assert_eq!(
            work_usage["total_context_work"].as_u64(),
            Some(input_hash_work + rendered.total_work())
        );
        assert!(input_hash_work + rendered.total_work() <= READER_MAX_WORK);
        let service_artifact = payload
            .get("rendered_artifact")
            .ok_or("Service borrowed render artifact")?;
        assert_eq!(
            wow_core::canonical_json_bytes(service_artifact)?,
            expected_bytes
        );
        assert_eq!(
            payload["validation"]["owner_generation_closure"],
            "not_evaluated"
        );
        assert_eq!(
            payload["validation"]["source_or_runtime_authority"],
            "not_evaluated"
        );
    }

    let invalid_coverage = mutate_and_reseal_native_pack(&l0_bytes, |pack| {
        let coverage = project_map_mut(pack)["core"]["navigation"]["coverage"]
            .as_array_mut()
            .ok_or("native graph coverage rows")?;
        let row = coverage
            .first_mut()
            .ok_or("native graph coverage mutation")?;
        row["state"] = json!("partial");
        row["negative_authority"] = json!(true);
        Ok(())
    })?;
    let json_request = render_request(ContextRenderFormat::Json, READER_MAX_WORK)?;
    for (bytes, expected_state) in [
        (
            invalid_coverage.as_slice(),
            wow_context::ValidationState::Invalid,
        ),
        (
            l1_bytes.as_slice(),
            wow_context::ValidationState::NotEvaluated,
        ),
    ] {
        let rendered = validate_and_render_primary_l0_pack(
            bytes,
            reader_limits(READER_MAX_BYTES)?,
            PrimaryPackRenderFormat::Json,
            &stop,
        )?;
        assert_eq!(rendered.validation().overall_state(), expected_state);
        assert!(rendered.artifact().is_none());
        if expected_state == wow_context::ValidationState::Invalid {
            assert_eq!(
                rendered.validation().digest_and_identity(),
                wow_context::ValidationState::Valid,
                "#132 coverage mutation keeps its enclosing identities valid"
            );
        }
        let service = execute_context_render(&json_request, bytes, &stop)?;
        assert_eq!(service.operation_status(), ContextMapStatus::Complete);
        assert_eq!(service.validation_state(), Some(expected_state));
        assert_eq!(service.failure_code(), None);
        let envelope: Value = serde_json::from_slice(service.canonical_bytes())?;
        let payload = envelope
            .get("payload")
            .ok_or("Service refusal validation payload")?;
        assert!(payload.get("validation").is_some());
        assert!(
            payload.get("rendered_artifact").is_none(),
            "no artifact or null placeholder"
        );
    }

    let cancelled = AtomicBool::new(true);
    let error = validate_and_render_primary_l0_pack(
        &l0_bytes,
        reader_limits(READER_MAX_BYTES)?,
        PrimaryPackRenderFormat::Markdown,
        &cancelled,
    )
    .err()
    .ok_or("cancelled owner render must refuse an artifact")?;
    assert_eq!(error.code(), wow_context::ContextErrorCode::Cancelled);
    let error = execute_context_render(&json_request, &l0_bytes, &cancelled)
        .err()
        .ok_or("cancelled Service render must refuse an artifact")?;
    assert_eq!(error.code(), crate::ServiceErrorCode::Cancelled);

    // One unit below the observed shared admission/render ledger must refuse
    // the same Markdown operation, even though its input was already Valid.
    let insufficient_work = markdown_work
        .checked_sub(1)
        .filter(|work| *work > 0)
        .ok_or("positive Markdown render work")?;
    let limits = wow_context::PrimaryPackValidationLimits::new(
        READER_MAX_BYTES,
        READER_MAX_BYTES,
        insufficient_work,
        128,
        READER_MAX_TOKENS,
    )?;
    let error = validate_and_render_primary_l0_pack(
        &l0_bytes,
        limits,
        PrimaryPackRenderFormat::Markdown,
        &stop,
    )
    .err()
    .ok_or("insufficient shared render work must refuse an artifact")?;
    assert_eq!(
        error.code(),
        wow_context::ContextErrorCode::MinimumRequiredContentExceedsBudget
    );
    let error = execute_context_render(
        &render_request(ContextRenderFormat::Markdown, 1)?,
        &l0_bytes,
        &stop,
    )
    .err()
    .ok_or("insufficient Service hashing work must refuse an artifact")?;
    assert_eq!(error.code(), crate::ServiceErrorCode::BudgetExceeded);
    Ok(())
}
