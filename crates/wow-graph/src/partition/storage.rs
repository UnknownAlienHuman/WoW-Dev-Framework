//! Graph-owned encoding over the generic manifested partition catalog. This is
//! retained graph metadata, not acceptance of a full E2 project publication.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};
use wow_store::project::{PartitionRecord, ReadSnapshot};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    schema: Box<str>,
    source_context_id: GenerationContextId,
    producers: Vec<String>,
}
impl GraphPartitionSnapshot {
    pub const STORAGE_SCHEMAS: &'static [&'static str] = &[
        "wow-graph.header.v1",
        "wow-graph.registry.v1",
        "wow-graph.foundation.v1",
        "wow-graph.producer.v1",
        "wow-graph.materialized.v1",
    ];
    pub const STORAGE_CHECK: &'static str = "wow-graph.retained-partition-closure.v1";

    /// Each producer is a separate immutable storage partition. Unchanged
    /// registries and producer records can be reused without rewriting them.
    pub fn storage_records(&self, stop: &AtomicBool) -> GraphResult<Vec<PartitionRecord>> {
        self.validate(stop)?;
        let mut records = vec![
            PartitionRecord::new("graph.registry", "wow-graph.registry.v1", &self.registry)?,
            PartitionRecord::new(
                "graph.foundation",
                "wow-graph.foundation.v1",
                &self.foundation,
            )?,
            PartitionRecord::new(
                "graph.materialized",
                "wow-graph.materialized.v1",
                &self.snapshot,
            )?,
        ];
        let mut producers = Vec::new();
        for p in &self.partitions {
            check_cancelled(stop)?;
            let key = producer_key(p.partition_id())?;
            records.push(PartitionRecord::new(&key, "wow-graph.producer.v1", p)?);
            producers.push(key);
        }
        records.push(PartitionRecord::new(
            "graph.header",
            "wow-graph.header.v1",
            &Header {
                schema: self.schema.clone(),
                source_context_id: self.source_context_id,
                producers,
            },
        )?);
        Ok(records)
    }
    /// Restore only from this exact generation's complete membership, then replay
    /// the existing registry/proposal/materialization validator, not Lua analysis.
    pub fn read_stored(read: &ReadSnapshot, stop: &AtomicBool) -> GraphResult<Self> {
        check_cancelled(stop)?;
        let header: Header = load(read, "graph.header", "wow-graph.header.v1", stop)?;
        if header.producers.len() > MAX_GRAPH_PRODUCER_PARTITIONS {
            return Err(invalid("stored producer count exceeds profile"));
        }
        let mut expected: BTreeSet<String> = [
            "graph.header",
            "graph.registry",
            "graph.foundation",
            "graph.materialized",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let mut partitions = Vec::new();
        for key in header.producers {
            if !expected.insert(key.clone()) {
                return Err(invalid("duplicate stored graph partition"));
            }
            let p: GraphProducerPartition = load(read, &key, "wow-graph.producer.v1", stop)?;
            if producer_key(p.partition_id())? != key {
                return Err(invalid("stored graph partition key mismatch"));
            }
            partitions.push(p);
        }
        let actual: BTreeSet<_> = read
            .manifest()
            .members
            .iter()
            .filter(|m| m.key.starts_with("graph."))
            .map(|m| m.key.clone())
            .collect();
        if actual != expected {
            return Err(invalid("stored graph membership mismatch"));
        }
        let result = Self {
            schema: header.schema,
            source_context_id: header.source_context_id,
            partitions,
            registry: load(read, "graph.registry", "wow-graph.registry.v1", stop)?,
            foundation: load(read, "graph.foundation", "wow-graph.foundation.v1", stop)?,
            snapshot: load(
                read,
                "graph.materialized",
                "wow-graph.materialized.v1",
                stop,
            )?,
        };
        result.validate(stop)?;
        Ok(result)
    }
}
fn producer_key(id: &str) -> GraphResult<String> {
    Ok(digest("graph.producer", &id)?.into())
}
fn load<T: serde::de::DeserializeOwned>(
    read: &ReadSnapshot,
    key: &str,
    schema: &str,
    stop: &AtomicBool,
) -> GraphResult<T> {
    if !read
        .manifest()
        .members
        .iter()
        .any(|m| m.key == key && m.schema == schema)
    {
        return Err(invalid("stored graph schema mismatch"));
    }
    Ok(read
        .record(key, stop)
        .map_err(storage_error)?
        .ok_or_else(|| invalid("stored graph record missing"))?
        .decode()?)
}

fn storage_error(e: wow_store::StoreError) -> GraphError {
    GraphError::new(
        match e.code() {
            wow_store::StoreErrorCode::Cancelled => GraphErrorCode::Cancelled,
            wow_store::StoreErrorCode::BudgetExceeded => GraphErrorCode::BudgetExceeded,
            _ => GraphErrorCode::StoreFailure,
        },
        "stored graph read failed",
    )
}
