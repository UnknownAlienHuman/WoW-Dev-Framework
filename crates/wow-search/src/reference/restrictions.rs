//! Data-only copies of genuine native restriction records and their subject link.
use super::*;
use wow_reference::native_view::{
    NATIVE_RESTRICTION_PARTITION, NativeRestrictionKind, NativeRestrictionScope,
    restriction_entity_id, restriction_origin_id,
};
use wow_reference::{RestrictionFacet, RestrictionState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RestrictionDocument {
    pub kind: NativeRestrictionKind,
    pub scope: NativeRestrictionScope,
    pub owner_partition: String,
    pub record_key: String,
    pub subject_key: String,
    pub subject_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_record_digest: Option<String>,
    pub subject_conflicted: bool,
    pub facets: Vec<RestrictionFacet>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn capture(
    native: &NativeViewProjection,
    partition: &wow_reference::ReferencePartition,
    origins: &BTreeMap<&str, SourceOrigin>,
    manifest: &mut Manifest,
    bounds: SearchBounds,
    charged: &mut u64,
    stop: &AtomicBool,
) -> SearchResult<()> {
    for record in partition.records() {
        check(stop)?;
        let fact = native
            .restriction_signature(record, stop)
            .map_err(native_error)?;
        if fact.reference_generation() != manifest.reference_generation
            || fact.reference_view_digest() != manifest.reference_view_digest
            || fact.record_key() != record.key()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let source = origins
            .get(fact.source_id())
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if restriction_origin_id(
            native.schema,
            fact.kind(),
            &manifest.binding.source_revision,
            &source.path,
            &source.sha256,
            wow_reference::native::Span {
                start: source.byte_range[0],
                end: source.byte_range[1],
            },
        )
        .map_err(native_error)?
            != source.source_id
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let facet = fact
            .facets()
            .first()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if fact.entity_id().len() > bounds.max_document_id_bytes.get() as usize
            || facet.id().len() > bounds.max_owner_metadata_label_bytes.get() as usize
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let document = Document {
            entity_key: fact.entity_id().into(),
            owner_record_digest: fact.record_digest().into(),
            source: source.clone(),
            canonical_short_name: facet.id().into(),
            namespace: None,
            canonical_qualified_name: facet.id().into(),
            arguments: Vec::new(),
            returns: Vec::new(),
            type_authority: "original_source_restriction_facets_only".into(),
            event: None,
            declaration: None,
            documentation: Some(documentation::Documentation::NotStated),
            restriction: Some(RestrictionDocument {
                kind: fact.kind(),
                scope: fact.scope(),
                owner_partition: NATIVE_RESTRICTION_PARTITION.into(),
                record_key: fact.record_key().into(),
                subject_key: fact.subject_key().into(),
                subject_name: fact.subject_name().into(),
                subject_namespace: fact.subject_namespace().map(str::to_owned),
                subject_record_digest: fact.subject_record_digest().map(str::to_owned),
                subject_conflicted: fact.subject_conflicted(),
                facets: fact.facets().to_vec(),
            }),
        };
        *charged = charged
            .checked_add(canonical_size(&document)? + u64::from(!manifest.documents.is_empty()))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        if *charged > bounds.max_owner_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        manifest.documents.push(document);
    }
    Ok(())
}

pub(super) struct SubjectIndex<'a> {
    records: BTreeMap<&'a str, &'a wow_reference::ReferenceRecord>,
    conflicts: std::collections::BTreeSet<&'a str>,
}
impl<'a> SubjectIndex<'a> {
    pub(super) fn new(view: &'a ReferenceView, stop: &AtomicBool) -> SearchResult<Self> {
        let part = view
            .partitions()
            .iter()
            .find(|part| part.id() == NATIVE_API_PARTITION)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let mut records = BTreeMap::new();
        for record in part.records() {
            check(stop)?;
            if record.kind() != ReferenceRecordKind::Api {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            records.insert(record.key(), record);
        }
        let mut conflicts = std::collections::BTreeSet::new();
        for conflict in view.conflicts() {
            check(stop)?;
            if conflict.partition_id() == NATIVE_API_PARTITION {
                conflicts.insert(conflict.key());
            }
        }
        Ok(Self { records, conflicts })
    }
}
pub(super) fn valid(
    document: &Document,
    record: &wow_reference::ReferenceRecord,
    view: &ReferenceView,
    index: Option<&SubjectIndex<'_>>,
) -> SearchResult<bool> {
    let Some(data) = &document.restriction else {
        return Ok(true);
    };
    let index = index.ok_or(SearchError::OwnerEvidenceInvalid)?;
    let subject = index
        .records
        .get(data.subject_key.as_str())
        .map(|record| {
            record
                .digest()
                .map_err(|_| SearchError::OwnerEvidenceInvalid)
        })
        .transpose()?;
    let conflicted = index.conflicts.contains(data.subject_key.as_str());
    let facet = record.restrictions().first();
    Ok(data.owner_partition == NATIVE_RESTRICTION_PARTITION
        && data.record_key == record.key()
        && data.subject_key == record.key()
        && data.facets == record.restrictions()
        && data.subject_record_digest == subject
        && data.subject_conflicted == conflicted
        && data.facets.len() == 1
        && facet.is_some_and(|facet| {
            facet.id() == data.kind.facet_id()
                && facet.state()
                    == match data.kind {
                        NativeRestrictionKind::SecretReturn => RestrictionState::Restricted,
                        NativeRestrictionKind::AccessPredicate => RestrictionState::Allowed,
                    }
        })
        && data.scope
            == match data.kind {
                NativeRestrictionKind::SecretReturn => {
                    NativeRestrictionScope::FirstReturnUnconditionalSource
                }
                NativeRestrictionKind::AccessPredicate => {
                    NativeRestrictionScope::FirstArgumentTrueImmediateCaller
                }
            }
        && document.entity_key
            == restriction_entity_id(view.generation_id(), record.key()).map_err(native_error)?
        && document.canonical_short_name == data.kind.facet_id()
        && document.event.is_none()
        && document.declaration.is_none()
        && document.namespace.is_none()
        && document.arguments.is_empty()
        && document.returns.is_empty()
        && matches!(
            document.documentation,
            Some(documentation::Documentation::NotStated)
        ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubjectBinding {
    Exact,
    Unresolved,
    Conflicted,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RestrictionShape {
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<NativeRestrictionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<NativeRestrictionScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    facet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<RestrictionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_entity_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_binding: Option<SubjectBinding>,
}
impl RestrictionShape {
    pub(super) fn validate(&self) -> SearchResult<()> {
        if self.kind.is_none()
            && self.scope.is_none()
            && self.facet_id.is_none()
            && self.state.is_none()
            && self.subject_entity_key.is_none()
            && self.subject_binding.is_none()
        {
            return Err(SearchError::InvalidProfile);
        }
        if let Some(value) = &self.facet_id {
            query::valid_text(value)?;
        }
        if let Some(value) = &self.subject_entity_key {
            wow_reference::ReferenceRecord::validate_key(value)
                .map_err(|_| SearchError::InvalidProfile)?;
        }
        Ok(())
    }
    pub(super) fn matches(&self, data: &RestrictionDocument) -> bool {
        self.kind.is_none_or(|kind| kind == data.kind)
            && self.scope.is_none_or(|scope| scope == data.scope)
            && self
                .facet_id
                .as_ref()
                .is_none_or(|id| data.facets.iter().any(|facet| facet.id() == id))
            && self
                .state
                .is_none_or(|state| data.facets.iter().any(|facet| facet.state() == state))
            && self
                .subject_entity_key
                .as_ref()
                .is_none_or(|key| key == &data.subject_key)
            && self
                .subject_binding
                .as_ref()
                .is_none_or(|binding| match binding {
                    SubjectBinding::Exact => {
                        data.subject_record_digest.is_some() && !data.subject_conflicted
                    }
                    SubjectBinding::Unresolved => data.subject_record_digest.is_none(),
                    SubjectBinding::Conflicted => data.subject_conflicted,
                })
    }
}
