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
        "wow-graph.header.v2",
        "wow-graph.producer.v2",
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
            let schema = if p.batch().assertion_records().is_some() {
                "wow-graph.producer.v2"
            } else {
                "wow-graph.producer.v1"
            };
            records.push(PartitionRecord::new(&key, schema, p)?);
            producers.push(key);
        }
        records.push(PartitionRecord::new(
            "graph.header",
            if self.schema.as_ref() == GRAPH_PARTITION_SNAPSHOT_SCHEMA_V2 {
                "wow-graph.header.v2"
            } else {
                "wow-graph.header.v1"
            },
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
        let header_schema = read
            .manifest()
            .members
            .iter()
            .find(|member| member.key == "graph.header")
            .map(|member| member.schema.as_str())
            .ok_or_else(|| invalid("stored graph header missing"))?;
        if !["wow-graph.header.v1", "wow-graph.header.v2"].contains(&header_schema) {
            return Err(invalid("unsupported stored graph header schema"));
        }
        let header: Header = load(read, "graph.header", header_schema, stop)?;
        if (header.schema.as_ref() == GRAPH_PARTITION_SNAPSHOT_SCHEMA_V2)
            != (header_schema == "wow-graph.header.v2")
        {
            return Err(invalid(
                "stored graph header version disagrees with snapshot",
            ));
        }
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
            let producer_schema = read
                .manifest()
                .members
                .iter()
                .find(|member| member.key == key)
                .map(|member| member.schema.as_str())
                .ok_or_else(|| invalid("stored producer missing"))?;
            if !["wow-graph.producer.v1", "wow-graph.producer.v2"].contains(&producer_schema) {
                return Err(invalid("unsupported stored graph producer schema"));
            }
            let p: GraphProducerPartition = load(read, &key, producer_schema, stop)?;
            if p.batch().assertion_records().is_some()
                != (producer_schema == "wow-graph.producer.v2")
            {
                return Err(invalid(
                    "stored producer version disagrees with its assertion records",
                ));
            }
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
