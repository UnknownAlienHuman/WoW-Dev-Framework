//! Read-only metadata for the two existing positive native restriction families.
use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeRestrictionKind {
    SecretReturn,
    AccessPredicate,
}
impl NativeRestrictionKind {
    pub(super) fn source_kind(self) -> &'static str {
        match self {
            Self::SecretReturn => "secret-return",
            Self::AccessPredicate => "access-predicate",
        }
    }
    pub fn facet_id(self) -> &'static str {
        match self {
            Self::SecretReturn => "secret.return",
            Self::AccessPredicate => "secret.predicate",
        }
    }
    fn expected_state(self) -> RestrictionState {
        match self {
            Self::SecretReturn => RestrictionState::Restricted,
            Self::AccessPredicate => RestrictionState::Allowed,
        }
    }
    fn expected_payload(self) -> &'static str {
        match self {
            Self::SecretReturn => NATIVE_SECRET_RETURN_PAYLOAD,
            Self::AccessPredicate => NATIVE_ACCESS_PREDICATE_PAYLOAD,
        }
    }
}
fn validate_record(
    record: &ReferenceRecord,
    kind: NativeRestrictionKind,
) -> Result<(), NativeError> {
    if record.kind() != crate::ReferenceRecordKind::Restriction
        || record.source_ids().len() != 1
        || record.restrictions().len() != 1
        || record.payload() != kind.expected_payload()
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let facet = &record.restrictions()[0];
    if facet.id() != kind.facet_id()
        || facet.state() != kind.expected_state()
        || facet.evidence_ids() != record.source_ids()
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RestrictionRow {
    key: String,
    digest: String,
    source: String,
    kind: NativeRestrictionKind,
    facets: Vec<RestrictionFacet>,
    name: String,
    namespace: Option<String>,
    entity_id: String,
    subject_record_digest: Option<String>,
    subject_conflicted: bool,
}
impl RestrictionRow {
    pub(super) fn capture(
        record: &ReferenceRecord,
        kind: NativeRestrictionKind,
        namespace: Option<&str>,
        name: &str,
        stop: &AtomicBool,
    ) -> Result<Self, NativeError> {
        checkpoint(stop)?;
        validate_record(record, kind)?;
        Ok(Self {
            key: record.key().into(),
            digest: record
                .digest()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
            source: record.source_ids()[0].to_string(),
            kind,
            facets: record.restrictions().to_vec(),
            name: name.into(),
            namespace: namespace.map(str::to_owned),
            entity_id: String::new(),
            subject_record_digest: None,
            subject_conflicted: false,
        })
    }
    fn check(&self, record: &ReferenceRecord) -> Result<(), NativeError> {
        validate_record(record, self.kind)?;
        if self.key != record.key()
            || self.source != record.source_ids()[0].as_ref()
            || self.facets != record.restrictions()
            || self.digest
                != record
                    .digest()
                    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?
        {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RestrictionCatalog {
    generation: String,
    view_digest: String,
    rows: BTreeMap<String, RestrictionRow>,
}
impl RestrictionCatalog {
    pub(super) fn finish(
        view: &ReferenceView,
        mut pending: BTreeMap<String, Vec<RestrictionRow>>,
        stop: &AtomicBool,
    ) -> Result<Self, NativeError> {
        checkpoint(stop)?;
        let mut catalog = Self {
            generation: view.generation_id().into(),
            view_digest: view.self_digest().into(),
            rows: BTreeMap::new(),
        };
        let Some(partition) = view
            .partitions()
            .iter()
            .find(|part| part.id() == NATIVE_RESTRICTION_PARTITION)
        else {
            return Ok(catalog);
        };
        let mut subjects = BTreeMap::new();
        if let Some(api) = view
            .partitions()
            .iter()
            .find(|part| part.id() == NATIVE_API_PARTITION)
        {
            for record in api.records() {
                checkpoint(stop)?;
                subjects.insert(record.key(), record);
            }
        }
        let mut conflicts = BTreeSet::new();
        for conflict in view.conflicts() {
            checkpoint(stop)?;
            if conflict.partition_id() == NATIVE_API_PARTITION {
                conflicts.insert(conflict.key());
            }
        }
        for record in partition.records() {
            checkpoint(stop)?;
            let mut group = pending
                .remove(record.key())
                .filter(|rows| rows.len() == 1)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            let mut row = group
                .pop()
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            row.check(record)?;
            row.entity_id = restriction_entity_id(view.generation_id(), record.key())?;
            row.subject_conflicted = conflicts.contains(record.key());
            row.subject_record_digest = subjects
                .get(record.key())
                .map(|subject| {
                    if row.subject_conflicted || subject.kind() != crate::ReferenceRecordKind::Api {
                        return Err(error(NativeErrorCode::InvalidIdentity));
                    }
                    subject
                        .digest()
                        .map_err(|_| error(NativeErrorCode::InvalidIdentity))
                })
                .transpose()?;
            catalog.rows.insert(record.key().into(), row);
        }
        // Pending rows belonging to conflicts were not materialized by the owner.
        Ok(catalog)
    }
    pub(super) fn read<'a>(
        &'a self,
        record: &ReferenceRecord,
        stop: &AtomicBool,
    ) -> Result<NativeRestrictionSignature<'a>, NativeError> {
        checkpoint(stop)?;
        let row = self
            .rows
            .get(record.key())
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        row.check(record)?;
        Ok(NativeRestrictionSignature {
            generation: &self.generation,
            view_digest: &self.view_digest,
            row,
        })
    }
}

/// Borrowed original native metadata; no public constructor or deserializer.
#[derive(Debug, Clone, Copy)]
pub struct NativeRestrictionSignature<'a> {
    generation: &'a str,
    view_digest: &'a str,
    row: &'a RestrictionRow,
}
impl NativeRestrictionSignature<'_> {
    pub fn kind(&self) -> NativeRestrictionKind {
        self.row.kind
    }
    pub fn entity_id(&self) -> &str {
        &self.row.entity_id
    }
    pub fn reference_generation(&self) -> &str {
        self.generation
    }
    pub fn reference_view_digest(&self) -> &str {
        self.view_digest
    }
    pub fn record_key(&self) -> &str {
        &self.row.key
    }
    pub fn record_digest(&self) -> &str {
        &self.row.digest
    }
    pub fn source_id(&self) -> &str {
        &self.row.source
    }
    pub fn subject_key(&self) -> &str {
        &self.row.key
    }
    pub fn subject_name(&self) -> &str {
        &self.row.name
    }
    pub fn subject_namespace(&self) -> Option<&str> {
        self.row.namespace.as_deref()
    }
    pub fn subject_record_digest(&self) -> Option<&str> {
        self.row.subject_record_digest.as_deref()
    }
    pub fn subject_conflicted(&self) -> bool {
        self.row.subject_conflicted
    }
    pub fn facets(&self) -> &[RestrictionFacet] {
        &self.row.facets
    }
}

/// Computes an identity only. The caller separately validates actual owner membership.
pub fn restriction_entity_id(generation: &str, key: &str) -> Result<String, NativeError> {
    if generation.is_empty() || generation.len() > 4096 || generation.chars().any(char::is_control)
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    ReferenceRecord::validate_key(key).map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let material = crate::wire_json::canonical_json_bytes(&(
        "wow-reference/native-restriction-entity/1",
        generation,
        NATIVE_RESTRICTION_PARTITION,
        key,
    ))
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    Ok(format!(
        "reference-restriction:{}",
        source_digest(&material)
    ))
}
