//! Project-owned retained source/fact partition encoding. Does not rehydrate a
//! live analyzer or claim a complete E2 ProjectSnapshot from a JSON receipt.
use super::{RetainedProjectGraphEvidence, SOURCE_GRAPH_PROFILE};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_graph::GraphPartitionSnapshot;
use wow_store::project::{PartitionRecord, ReadSnapshot};

pub const STORAGE_SCHEMAS: &[&str] = &[
    "wow-project.retained-field.v1",
    "wow-project.retained-header.v1",
];
pub const STORAGE_CHECK: &str = "wow-project.retained-source-evidence-closure.v1";

pub fn records(
    provenance: &Value,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<Vec<PartitionRecord>> {
    validate(provenance, graph, stop)?;
    let root = provenance.as_object().ok_or_else(invalid)?;
    if root.is_empty() || root.len() > 64 {
        return Err(invalid());
    }
    let mut keys: Vec<&str> = root.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut records = Vec::new();
    for key in &keys {
        crate::analyzer::checkpoint(stop)?;
        if !field(key) {
            return Err(invalid());
        }
        let value = root.get(*key).ok_or_else(invalid)?;
        records.push(
            PartitionRecord::new(
                format!("project.provenance.{key}"),
                "wow-project.retained-field.v1",
                value,
            )
            .map_err(store_error)?,
        );
    }
    records.push(
        PartitionRecord::new("project.header", "wow-project.retained-header.v1", &keys)
            .map_err(store_error)?,
    );
    Ok(records)
}
pub fn read_provenance(
    read: &ReadSnapshot,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<Value> {
    let keys: Vec<String> = load(
        read,
        "project.header",
        "wow-project.retained-header.v1",
        stop,
    )?;
    if keys.is_empty() || keys.len() > 64 || keys.windows(2).any(|w| w[0] >= w[1]) {
        return Err(invalid());
    }
    let mut expected = BTreeSet::from(["project.header".to_owned()]);
    let mut object = Map::new();
    for key in keys {
        if !field(&key) {
            return Err(invalid());
        }
        let record_key = format!("project.provenance.{key}");
        expected.insert(record_key.clone());
        object.insert(
            key,
            load(read, &record_key, "wow-project.retained-field.v1", stop)?,
        );
    }
    let actual: BTreeSet<_> = read
        .manifest()
        .members
        .iter()
        .filter(|m| m.key.starts_with("project."))
        .map(|m| m.key.clone())
        .collect();
    if actual != expected {
        return Err(invalid());
    }
    let value = Value::Object(object);
    validate(&value, graph, stop)?;
    Ok(value)
}
/// Exact source/analyzer/graph identity labels, never storage row IDs. These
/// retained labels remain metadata rather than a new live ProjectView.
pub fn bindings(
    provenance: &Value,
    graph: &GraphPartitionSnapshot,
) -> ProjectResult<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for key in ["project_snapshot_id", "analyzer_snapshot_id"] {
        let value = provenance
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        if value.is_empty() || value.len() > 256 {
            return Err(invalid());
        }
        result.insert(key.to_owned(), value.to_owned());
    }
    result.insert(
        "graph_snapshot_id".into(),
        graph.snapshot().snapshot_id().as_str().into(),
    );
    result.insert(
        "graph_generation_id".into(),
        graph.snapshot().generation().as_str().into(),
    );
    result.insert(
        "source_context_id".into(),
        graph.source_context_id().to_string(),
    );
    Ok(result)
}
fn validate(value: &Value, graph: &GraphPartitionSnapshot, stop: &AtomicBool) -> ProjectResult<()> {
    if value.get("profile").and_then(Value::as_str) != Some(SOURCE_GRAPH_PROFILE) {
        return Err(invalid());
    }
    let retained: RetainedProjectGraphEvidence =
        serde_json::from_value(value.clone()).map_err(|_| invalid())?;
    retained.admit(graph, stop)?;
    Ok(())
}
fn field(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn load<T: serde::de::DeserializeOwned>(
    read: &ReadSnapshot,
    key: &str,
    schema: &str,
    stop: &AtomicBool,
) -> ProjectResult<T> {
    if !read
        .manifest()
        .members
        .iter()
        .any(|m| m.key == key && m.schema == schema)
    {
        return Err(invalid());
    }
    read.record(key, stop)
        .map_err(store_error)?
        .ok_or_else(invalid)?
        .decode()
        .map_err(store_error)
}
fn store_error(e: wow_store::StoreError) -> ProjectError {
    ProjectError::new(
        match e.code() {
            wow_store::StoreErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
            wow_store::StoreErrorCode::BudgetExceeded => ProjectErrorCode::SourceBudgetExceeded,
            _ => ProjectErrorCode::SourceRegistryInvalid,
        },
        ProjectPhase::View,
        "retained project storage validation failed",
    )
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::View,
        "retained project publication is inconsistent",
    )
}
