//! Transport admission of a graph-build v7 receipt. The graph and project owners
//! independently validate the snapshot and source/evidence projection; remaining
//! build sidecars are integrity-bound data, not reconstructed semantic owners.
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use wow_graph::{GraphEvidenceCatalog, GraphPartitionSnapshot};
use wow_project::graph::RetainedProjectGraphEvidence;

use super::{GraphReadFailure, GraphReadStage, checkpoint, hash, input};
use crate::ServiceErrorCode;

pub const GRAPH_BUNDLE_MAX_BYTES: usize = 32 * 1024 * 1024;
const BUILD_SCHEMA: &str = "wow-service/graph-build-result/7";

pub(super) struct AdmittedBundle {
    pub owner: GraphPartitionSnapshot,
    pub evidence: GraphEvidenceCatalog,
    pub sources: wow_project::graph::RetainedProjectSourceManifest,
    pub result_digest: Box<str>,
    pub snapshot_digest: Box<str>,
    pub boundaries: Vec<Box<str>>,
}

pub(super) fn admit(bytes: &[u8], stop: &AtomicBool) -> Result<AdmittedBundle, GraphReadFailure> {
    let value: Value = input::decode_bundle(bytes, GRAPH_BUNDLE_MAX_BYTES, stop)?;
    let Value::Object(mut root) = value else {
        return Err(invalid());
    };
    if root.get("schema").and_then(Value::as_str) != Some(BUILD_SCHEMA)
        || root.get("status").and_then(Value::as_str) != Some("partial")
        || root.contains_key("failure")
    {
        return Err(invalid());
    }
    let expected_result = take_string(&mut root, "result_digest")?;
    // Hash every sidecar as well, including fields not consumed by this read.
    // These hashes protect integrity, not authenticity: callers can recompute
    // them. Independent typed owner validation below is mandatory either way.
    checkpoint(stop)?;
    let canonical = wow_core::canonical_json_bytes(&root).map_err(|_| invalid())?;
    if hash(&canonical).as_ref() != expected_result.as_ref() {
        return Err(mismatch());
    }
    drop(canonical);
    let expected_request = take_string(&mut root, "request_digest")?;
    let request = root.remove("request").ok_or_else(invalid)?;
    let request_bytes = wow_core::canonical_json_bytes(&request).map_err(|_| invalid())?;
    if request_bytes.len() > super::GRAPH_REQUEST_MAX_BYTES
        || hash(&request_bytes).as_ref() != expected_request.as_ref()
        || request.get("schema").and_then(Value::as_str)
            != Some("wow-service/graph-build-request/7")
        || request.get("projection").and_then(Value::as_str)
            != Some(wow_project::graph::SOURCE_GRAPH_PROFILE)
    {
        return Err(mismatch());
    }
    let expected_snapshot = take_string(&mut root, "snapshot_input_digest")?;
    let snapshot_value = root.remove("snapshot").ok_or_else(invalid)?;
    let snapshot_bytes = wow_core::canonical_json_bytes(&snapshot_value).map_err(|_| invalid())?;
    if hash(&snapshot_bytes).as_ref() != expected_snapshot.as_ref() {
        return Err(mismatch());
    }
    drop(snapshot_value);
    // Retain the established bare-snapshot decoder's smaller limits, including
    // its token/string/depth profile, even though the containing receipt is larger.
    let owner: GraphPartitionSnapshot = input::decode(
        &snapshot_bytes,
        super::GRAPH_INPUT_MAX_BYTES,
        1_000_000,
        GraphReadStage::Snapshot,
        stop,
    )?;
    drop(snapshot_bytes);
    let evidence: RetainedProjectGraphEvidence =
        serde_json::from_value(root.remove("provenance").ok_or_else(invalid)?)
            .map_err(|_| invalid())?;
    let (evidence, sources) = evidence.admit_with_sources(&owner, stop).map_err(|error| {
        GraphReadFailure::service(
            GraphReadStage::Evidence,
            match error.code() {
                wow_project::ProjectErrorCode::AnalysisCancelled
                | wow_project::ProjectErrorCode::SourceReadCancelled => ServiceErrorCode::Cancelled,
                wow_project::ProjectErrorCode::SourceBudgetExceeded => {
                    ServiceErrorCode::BudgetExceeded
                }
                _ => ServiceErrorCode::InvalidRequest,
            },
        )
    })?;
    let raw_boundaries = root.remove("boundaries").ok_or_else(invalid)?;
    let boundaries: Vec<Box<str>> =
        serde_json::from_value(raw_boundaries).map_err(|_| invalid())?;
    if boundaries.len() > 64
        || boundaries.iter().any(|text| {
            text.is_empty()
                || text.len() > 512
                || !text
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
    {
        return Err(invalid());
    }
    checkpoint(stop)?;
    Ok(AdmittedBundle {
        owner,
        evidence,
        sources,
        result_digest: expected_result,
        snapshot_digest: expected_snapshot,
        boundaries,
    })
}

fn take_string(
    root: &mut serde_json::Map<String, Value>,
    key: &str,
) -> Result<Box<str>, GraphReadFailure> {
    let Value::String(value) = root.remove(key).ok_or_else(invalid)? else {
        return Err(invalid());
    };
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid());
    }
    Ok(value.into())
}
fn invalid() -> GraphReadFailure {
    GraphReadFailure::service(GraphReadStage::Bundle, ServiceErrorCode::InvalidRequest)
}
fn mismatch() -> GraphReadFailure {
    GraphReadFailure::service(GraphReadStage::Bundle, ServiceErrorCode::IdentityMismatch)
}
