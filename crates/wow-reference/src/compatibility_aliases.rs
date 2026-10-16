//! Explicit configured compatibility declarations. These are not runtime or Blizzard aliases.
use crate::native::{NativeError, NativeErrorCode, source_digest};
use crate::native_view::{
    NATIVE_API_PARTITION, NATIVE_DECLARATION_PARTITION, NATIVE_EVENT_PARTITION, NativeViewSource,
};
use crate::{
    CoverageStatus, LookupResult, ReferencePartition, ReferenceRecord, ReferenceRecordKind,
    ReferenceView,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{ProfileIdentity, ReferenceGenerationId};

mod input;
use input::{Input, Target};
pub const PARTITION: &str = "reference.configured.compatibility-alias";
pub const MAX_INPUT_BYTES: usize = 65_536;
const MAX_DECLARATIONS: usize = 128;
const HEADER_KEY: &str = "configured-compatibility-alias-catalog";
const HEADER_SCHEMA: &str = "wow-reference/configured-compatibility-alias-catalog/1";
const RECORD_SCHEMA: &str = "wow-reference/configured-compatibility-alias-record/1";

fn error(code: NativeErrorCode) -> NativeError {
    NativeError { code, span: None }
}
fn checkpoint(stop: &AtomicBool) -> Result<(), NativeError> {
    if stop.load(Ordering::Acquire) {
        Err(error(NativeErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, NativeError> {
    crate::wire_json::canonical_json_bytes(value)
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))
}
pub fn profile_digest(profile: &ProfileIdentity) -> Result<String, NativeError> {
    profile
        .validate()
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    Ok(source_digest(
        &wow_core::canonical_json_bytes(profile)
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AliasReceipt {
    pub source_path: String,
    pub source_digest: String,
    pub source_bytes: usize,
    pub profile_id: String,
    pub profile_digest: String,
    pub base_generation: String,
    pub base_view_digest: String,
    pub declared_records: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AliasState {
    Resolved,
    TargetMissing,
    TargetConflict,
    TargetDigestMismatch,
    AliasConflict,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    schema: String,
    receipt: AliasReceipt,
    resource_json: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    schema: String,
    value: String,
    target: Target,
    state: AliasState,
    declaration_indices: Vec<usize>,
}

/// An owned validated view of original metadata records from a real ReferenceView.
/// No public constructor/deserialization path; Search may copy its data only.
#[derive(Debug, Clone)]
pub struct AliasCatalog {
    receipt: AliasReceipt,
    generation: String,
    view_digest: String,
    records: Vec<AliasRecord>,
    pub(crate) partition: ReferencePartition,
    pub(crate) sources: Vec<NativeViewSource>,
}
#[derive(Debug, Clone)]
pub struct AliasRecord {
    record_key: String,
    record_digest: String,
    sources: Vec<String>,
    data: Payload,
}
impl AliasRecord {
    pub fn record_key(&self) -> &str {
        &self.record_key
    }
    pub fn record_digest(&self) -> &str {
        &self.record_digest
    }
    pub fn source_ids(&self) -> &[String] {
        &self.sources
    }
    pub fn value(&self) -> &str {
        &self.data.value
    }
    pub fn target_partition(&self) -> &str {
        &self.data.target.partition_id
    }
    pub fn target_key(&self) -> &str {
        &self.data.target.record_key
    }
    pub fn target_digest(&self) -> &str {
        &self.data.target.record_digest
    }
    pub fn state(&self) -> AliasState {
        self.data.state
    }
    pub fn declaration_indices(&self) -> &[usize] {
        &self.data.declaration_indices
    }
}
impl AliasCatalog {
    pub fn receipt(&self) -> &AliasReceipt {
        &self.receipt
    }
    pub fn generation(&self) -> &str {
        &self.generation
    }
    pub fn view_digest(&self) -> &str {
        &self.view_digest
    }
    pub fn records(&self) -> &[AliasRecord] {
        &self.records
    }
}
fn source_id(receipt: &AliasReceipt, index: Option<usize>) -> Result<String, NativeError> {
    Ok(format!(
        "configured-compatibility:{}",
        source_digest(&encode(&(receipt, index))?)
    ))
}
fn allowed_partition(part: &str) -> bool {
    matches!(
        part,
        NATIVE_API_PARTITION | NATIVE_EVENT_PARTITION | NATIVE_DECLARATION_PARTITION
    )
}
pub(crate) fn admit(
    profile: &ProfileIdentity,
    base: &ReferenceView,
    path: &str,
    bytes: &[u8],
    stop: &AtomicBool,
) -> Result<AliasCatalog, NativeError> {
    checkpoint(stop)?;
    base.validate()
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    if base.partitions().iter().any(|part| part.id() == PARTITION)
        || path.is_empty()
        || path.len() > 4096
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let parsed = Input::parse(bytes, stop)?;
    let digest = profile_digest(profile)?;
    if parsed.profile_id != profile.profile_id().as_str()
        || parsed.profile_digest != digest
        || parsed.base_generation != base.generation_id()
        || parsed.base_view_digest != base.self_digest()
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let receipt = AliasReceipt {
        source_path: path.into(),
        source_digest: source_digest(bytes),
        source_bytes: bytes.len(),
        profile_id: parsed.profile_id,
        profile_digest: digest,
        base_generation: parsed.base_generation,
        base_view_digest: parsed.base_view_digest,
        declared_records: parsed.declarations.len(),
    };
    let header = Header {
        schema: HEADER_SCHEMA.into(),
        receipt: receipt.clone(),
        resource_json: std::str::from_utf8(bytes)
            .map_err(|_| error(NativeErrorCode::InvalidEncoding))?
            .into(),
    };
    let header_payload = encode(&header)?;
    if header_payload.len() > 65_536 {
        return Err(error(NativeErrorCode::Limit));
    }
    let header_source = source_id(&receipt, None)?;
    let mut original_sources = vec![NativeViewSource {
        id: header_source.clone(),
        path: path.into(),
        sha256: receipt.source_digest.clone(),
        span: crate::native::Span {
            start: 0,
            end: bytes.len(),
        },
    }];
    let mut records = vec![
        ReferenceRecord::new(
            HEADER_KEY,
            ReferenceRecordKind::Other,
            String::from_utf8(header_payload)
                .map_err(|_| error(NativeErrorCode::InvalidEncoding))?,
            vec![header_source],
            Vec::new(),
        )
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
    ];
    let mut values = BTreeMap::<String, BTreeSet<Target>>::new();
    let mut groups = BTreeMap::<(String, Target), Vec<usize>>::new();
    for (index, declaration) in parsed.declarations.into_iter().enumerate() {
        checkpoint(stop)?;
        if !allowed_partition(&declaration.target.partition_id) {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        values
            .entry(declaration.value.clone())
            .or_default()
            .insert(declaration.target.clone());
        groups
            .entry((declaration.value, declaration.target))
            .or_default()
            .push(index);
    }
    let mut projected = Vec::new();
    for ((value, target), indices) in groups {
        checkpoint(stop)?;
        let state = if values.get(&value).is_some_and(|targets| targets.len() > 1) {
            AliasState::AliasConflict
        } else {
            match base.lookup(&target.partition_id, &target.record_key) {
                LookupResult::Found(record) => {
                    if record
                        .digest()
                        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?
                        == target.record_digest
                    {
                        AliasState::Resolved
                    } else {
                        AliasState::TargetDigestMismatch
                    }
                }
                LookupResult::Conflict(_) => AliasState::TargetConflict,
                _ => AliasState::TargetMissing,
            }
        };
        let key = format!(
            "configured-compatibility-alias:{}",
            source_digest(&encode(&(&receipt, &value, &target))?)
        );
        let ids = indices
            .iter()
            .map(|index| source_id(&receipt, Some(*index)))
            .collect::<Result<Vec<_>, _>>()?;
        for id in &ids {
            original_sources.push(NativeViewSource {
                id: id.clone(),
                path: path.into(),
                sha256: receipt.source_digest.clone(),
                span: crate::native::Span {
                    start: 0,
                    end: bytes.len(),
                },
            });
        }
        let data = Payload {
            schema: RECORD_SCHEMA.into(),
            value,
            target,
            state,
            declaration_indices: indices,
        };
        let payload = encode(&data)?;
        if payload.len() > 65_536 {
            return Err(error(NativeErrorCode::Limit));
        }
        let record = ReferenceRecord::new(
            &key,
            ReferenceRecordKind::Other,
            String::from_utf8(payload).map_err(|_| error(NativeErrorCode::InvalidEncoding))?,
            ids.clone(),
            Vec::new(),
        )
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        projected.push(AliasRecord {
            record_key: key,
            record_digest: record
                .digest()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
            sources: record
                .source_ids()
                .iter()
                .map(ToString::to_string)
                .collect(),
            data,
        });
        records.push(record);
    }
    let partition = ReferencePartition::new(PARTITION, CoverageStatus::Partial, records)
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    Ok(AliasCatalog {
        receipt,
        generation: String::new(),
        view_digest: String::new(),
        records: projected,
        partition,
        sources: original_sources,
    })
}
pub(crate) fn compose(
    base: &ReferenceView,
    catalog: &AliasCatalog,
) -> Result<ReferenceView, NativeError> {
    let generation = ReferenceGenerationId::derive(&(
        "wow-reference/configured-compatibility-alias-view/1",
        base.generation_id(),
        base.self_digest(),
        &catalog.partition,
    ))
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let mut partitions = base.partitions().to_vec();
    partitions.push(catalog.partition.clone());
    ReferenceView::new(
        generation.to_string(),
        partitions,
        base.conflicts().to_vec(),
    )
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))
}
/// Read and revalidate the owner's original metadata, never a Search projection.
pub fn read(
    view: &ReferenceView,
    profile: &ProfileIdentity,
    stop: &AtomicBool,
) -> Result<Option<AliasCatalog>, NativeError> {
    checkpoint(stop)?;
    let Some(partition) = view.partitions().iter().find(|part| part.id() == PARTITION) else {
        return Ok(None);
    };
    view.validate()
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let header_record = partition
        .records()
        .iter()
        .find(|record| record.key() == HEADER_KEY)
        .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
    let header: Header = serde_json::from_str(header_record.payload())
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    if header.schema != HEADER_SCHEMA || header.resource_json.len() > MAX_INPUT_BYTES {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let base = ReferenceView::new(
        &header.receipt.base_generation,
        view.partitions()
            .iter()
            .filter(|part| part.id() != PARTITION)
            .cloned()
            .collect(),
        view.conflicts().to_vec(),
    )
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let mut catalog = admit(
        profile,
        &base,
        &header.receipt.source_path,
        header.resource_json.as_bytes(),
        stop,
    )?;
    if catalog.receipt != header.receipt
        || &catalog.partition != partition
        || compose(&base, &catalog)? != *view
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    catalog.generation = view.generation_id().into();
    catalog.view_digest = view.self_digest().into();
    Ok(Some(catalog))
}
