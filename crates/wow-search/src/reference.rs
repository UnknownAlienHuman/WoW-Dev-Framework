//! Typed native callable projections in a separate exact Reference universe.
//! Captured names/types remain source observations; no opaque payload decoding.
use super::*;
use std::collections::BTreeMap;
use wow_core::ProfileIdentity;
use wow_reference::{
    CoverageStatus, ReferenceConflict, ReferenceRecordKind, ReferenceView,
    native_view::{NATIVE_API_PARTITION, NativeCallableField, NativeViewProjection},
};
use wow_store::StoreLimits;

mod compatibility_aliases;
mod declarations;
mod documentation;
mod query;
mod restrictions;
mod validation;
pub use query::{
    ReferencePage, ReferencePageCursor, ReferencePagePolicy, ReferenceQueryRequest,
    ReferenceQueryResult,
};
pub use validation::{ReferenceShardValidation, validate_reference_callable_shard};
const PROFILE_SCHEMA: &str = "wow-search/native-reference-callable-profile/1";
const MANIFEST_SCHEMA: &str = "wow-search/native-reference-callable-shard/1";
const DOCUMENT_PROFILE_SCHEMA: &str = "wow-search/native-reference-callable-event-profile/1";
const DOCUMENT_MANIFEST_SCHEMA: &str = "wow-search/native-reference-callable-event-shard/1";
const DECLARATION_PROFILE_SCHEMA: &str =
    "wow-search/native-reference-source-declarations-profile/1";
const DECLARATION_MANIFEST_SCHEMA: &str = "wow-search/native-reference-source-declarations-shard/1";
const DOCUMENTATION_PROFILE_SCHEMA: &str =
    "wow-search/native-reference-documentation-inventory-profile/1";
const DOCUMENTATION_MANIFEST_SCHEMA: &str =
    "wow-search/native-reference-documentation-inventory-shard/1";
const INVENTORY_PROFILE_SCHEMA: &str = "wow-search/native-reference-inventory-profile/1";
const INVENTORY_MANIFEST_SCHEMA: &str = "wow-search/native-reference-inventory-shard/1";
const ALIAS_PROFILE_SCHEMA: &str = "wow-search/native-reference-compatibility-inventory-profile/1";
const ALIAS_MANIFEST_SCHEMA: &str = "wow-search/native-reference-compatibility-inventory-shard/1";
use wow_reference::native_view::{
    NATIVE_DECLARATION_PARTITION, NATIVE_EVENT_PARTITION, NativeDeclarationKind,
    NativeDeclarationScalar, NativeReferenceDocuments,
};

fn profile_schemas(selection: Option<NativeReferenceDocuments>) -> (&'static str, &'static str) {
    match selection {
        None => (PROFILE_SCHEMA, MANIFEST_SCHEMA),
        Some(NativeReferenceDocuments::CallablesAndEventsV1) => {
            (DOCUMENT_PROFILE_SCHEMA, DOCUMENT_MANIFEST_SCHEMA)
        }
        Some(NativeReferenceDocuments::SourceDeclarationsV1) => {
            (DECLARATION_PROFILE_SCHEMA, DECLARATION_MANIFEST_SCHEMA)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceSearchProfile {
    bounds: SearchBounds,
    digest: Box<str>,
    native_documents: Option<NativeReferenceDocuments>,
    include_documentation: bool,
    include_restrictions: bool,
    include_compatibility_aliases: bool,
}
impl ReferenceSearchProfile {
    pub fn new(bounds: SearchBounds) -> SearchResult<Self> {
        Self::new_documents(bounds, None)
    }
    pub fn new_documents(
        bounds: SearchBounds,
        native_documents: Option<NativeReferenceDocuments>,
    ) -> SearchResult<Self> {
        bounds.physical()?;
        Ok(Self {
            bounds,
            native_documents,
            include_documentation: false,
            include_restrictions: false,
            include_compatibility_aliases: false,
            digest: identity(
                "",
                &(
                    profile_schemas(native_documents).0,
                    profile_schemas(native_documents).1,
                    "canonical-name-namespace-source-signature/1",
                    "case-sensitive-identity/1",
                    FTS_CANDIDATE_PROFILE,
                    "local-metadata-only",
                    bounds,
                ),
            )?,
        })
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn documentation_inventory(bounds: SearchBounds) -> SearchResult<Self> {
        let mut profile =
            Self::new_documents(bounds, Some(NativeReferenceDocuments::SourceDeclarationsV1))?;
        profile.digest = identity(
            "",
            &(
                DOCUMENTATION_PROFILE_SCHEMA,
                DOCUMENTATION_MANIFEST_SCHEMA,
                &profile.digest,
                "original-source-documentation-only-fts/1",
                documentation::INDEX_PROFILE,
            ),
        )?;
        profile.include_documentation = true;
        Ok(profile)
    }
    pub fn inventory(bounds: SearchBounds) -> SearchResult<Self> {
        let mut profile = Self::documentation_inventory(bounds)?;
        profile.digest = identity(
            "",
            &(
                INVENTORY_PROFILE_SCHEMA,
                INVENTORY_MANIFEST_SCHEMA,
                &profile.digest,
                "native-positive-restriction-facets-with-scoped-identity/1",
            ),
        )?;
        profile.include_restrictions = true;
        Ok(profile)
    }
    pub fn with_compatibility_aliases(
        bounds: SearchBounds,
        include_restrictions: bool,
    ) -> SearchResult<Self> {
        let mut profile = if include_restrictions {
            Self::inventory(bounds)?
        } else {
            Self::documentation_inventory(bounds)?
        };
        profile.digest = identity(
            "",
            &(
                ALIAS_PROFILE_SCHEMA,
                ALIAS_MANIFEST_SCHEMA,
                &profile.digest,
                "resolved-configured-owner-compatibility-alias/1",
            ),
        )?;
        profile.include_compatibility_aliases = true;
        Ok(profile)
    }
    fn manifest_schema(&self) -> &'static str {
        if self.include_compatibility_aliases {
            ALIAS_MANIFEST_SCHEMA
        } else if self.include_restrictions {
            INVENTORY_MANIFEST_SCHEMA
        } else if self.include_documentation {
            DOCUMENTATION_MANIFEST_SCHEMA
        } else {
            profile_schemas(self.native_documents).1
        }
    }
}

/// Supplied by the genuine SourceReference acquisition and physical Store owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSearchBinding {
    profile: ProfileIdentity,
    reference_store_instance_id: Box<str>,
    reference_store_configuration_id: Box<str>,
    reference_view_object_id: ObjectId,
    acquisition_partial: bool,
    acquisition_failure_receipts: Vec<serde_json::Value>,
    source_revision: String,
}
impl ReferenceSearchBinding {
    pub fn new(
        profile: ProfileIdentity,
        store: &Store,
        reference_view_object_id: ObjectId,
        acquisition_partial: bool,
        source_revision: &str,
        acquisition_failure_receipts: Vec<serde_json::Value>,
    ) -> SearchResult<Self> {
        profile
            .validate()
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if !acquisition_partial && !acquisition_failure_receipts.is_empty() {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        Ok(Self {
            profile,
            reference_store_instance_id: store.store_instance_id()?.as_str().into(),
            reference_store_configuration_id: store.configuration().configuration_id().into(),
            reference_view_object_id,
            acquisition_partial,
            acquisition_failure_receipts,
            source_revision: source_revision.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceOrigin {
    source_id: String,
    path: String,
    sha256: String,
    byte_range: [usize; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    name: String,
    original_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_inner_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nilable: Option<bool>,
    byte_range: [usize; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    entity_key: String,
    owner_record_digest: String,
    source: SourceOrigin,
    canonical_short_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    canonical_qualified_name: String,
    arguments: Vec<Field>,
    returns: Vec<Field>,
    type_authority: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event: Option<EventDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    declaration: Option<declarations::DeclarationDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    documentation: Option<documentation::Documentation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    restriction: Option<restrictions::RestrictionDocument>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventDocument {
    definition_name: String,
    literal_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    definition_namespace: Option<String>,
    payload: Vec<Field>,
}
impl Document {
    fn record_kind(&self) -> ReferenceRecordKind {
        if self.restriction.is_some() {
            ReferenceRecordKind::Restriction
        } else if let Some(declaration) = &self.declaration {
            declaration.kind.record_kind()
        } else if self.event.is_some() {
            ReferenceRecordKind::Event
        } else {
            ReferenceRecordKind::Api
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Partition {
    id: String,
    coverage: CoverageStatus,
    record_count: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionIssue {
    code: String,
    path: String,
    sha256: String,
    byte_range: [usize; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    compatibility_aliases: Option<compatibility_aliases::AliasInventory>,
    schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_documents: Option<NativeReferenceDocuments>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    include_documentation: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    include_restrictions: Option<bool>,
    profile_digest: Box<str>,
    binding: ReferenceSearchBinding,
    reference_generation: String,
    reference_view_digest: String,
    native_projection_profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    available_type_resolution_profile: Option<String>,
    source_origins: Vec<SourceOrigin>,
    partitions: Vec<Partition>,
    conflicts: Vec<ReferenceConflict>,
    issues: Vec<ProjectionIssue>,
    owner_negative_authority: bool,
    documents: Vec<Document>,
    privacy_scope: String,
    license_state: String,
}

pub struct PreparedReferenceShard {
    manifest: Manifest,
    physical: PreparedSearchShard,
}
impl PreparedReferenceShard {
    pub fn logical_id(&self) -> &str {
        self.physical.search_shard_id()
    }
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    pub fn reference_generation(&self) -> &str {
        &self.manifest.reference_generation
    }
    pub fn reference_view_digest(&self) -> &str {
        &self.manifest.reference_view_digest
    }
    pub fn document_count(&self) -> usize {
        self.manifest.documents.len()
    }
    pub fn into_object(self) -> PendingObject {
        self.physical.into_object()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_reference_callable_shard(
    native: &NativeViewProjection,
    binding: ReferenceSearchBinding,
    profile: &ReferenceSearchProfile,
    permission: LocalMetadataPermission,
    object_limits: StoreLimits,
    stop: &AtomicBool,
) -> SearchResult<PreparedReferenceShard> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    native
        .view
        .validate()
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let bounds = profile.bounds;
    if native.native_documents() != profile.native_documents {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let partition = native
        .view
        .partitions()
        .iter()
        .find(|p| p.id() == NATIVE_API_PARTITION)
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let event_partition = if profile.native_documents.is_some() {
        Some(
            native
                .view
                .partitions()
                .iter()
                .find(|part| part.id() == NATIVE_EVENT_PARTITION)
                .ok_or(SearchError::OwnerEvidenceInvalid)?,
        )
    } else {
        None
    };
    let declaration_partition =
        if profile.native_documents == Some(NativeReferenceDocuments::SourceDeclarationsV1) {
            Some(
                native
                    .view
                    .partitions()
                    .iter()
                    .find(|part| part.id() == NATIVE_DECLARATION_PARTITION)
                    .ok_or(SearchError::OwnerEvidenceInvalid)?,
            )
        } else {
            None
        };
    let count = partition
        .records()
        .len()
        .checked_add(event_partition.map_or(0, |part| part.records().len()))
        .and_then(|count| {
            count.checked_add(declaration_partition.map_or(0, |part| part.records().len()))
        })
        .and_then(|count| {
            count.checked_add(if profile.include_restrictions {
                native
                    .view
                    .partitions()
                    .iter()
                    .find(|part| {
                        part.id() == wow_reference::native_view::NATIVE_RESTRICTION_PARTITION
                    })
                    .map_or(0, |part| part.records().len())
            } else {
                0
            })
        })
        .ok_or(SearchError::FileSetOverCapacity)?;
    if count == 0 || count > bounds.max_documents.get() as usize {
        return Err(SearchError::FileSetOverCapacity);
    }
    let evidence = native
        .sources
        .len()
        .checked_add(native.issues.len())
        .and_then(|n| n.checked_add(native.view.conflicts().len()))
        .and_then(|n| n.checked_add(native.view.partitions().len()))
        .and_then(|n| n.checked_add(binding.acquisition_failure_receipts.len()))
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if evidence > bounds.max_owner_evidence_records.get() as usize {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let mut origins = BTreeMap::new();
    for source in &native.sources {
        check(stop)?;
        let origin = SourceOrigin {
            source_id: source.id.clone(),
            path: source.path.clone(),
            sha256: source.sha256.clone(),
            byte_range: [source.span.start, source.span.end],
        };
        if origins
            .insert(source.id.as_str(), origin.clone())
            .is_some_and(|old| old != origin)
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
    }
    let aliases = if profile.include_compatibility_aliases {
        let catalog =
            wow_reference::compatibility_aliases::read(&native.view, &binding.profile, stop)
                .map_err(native_error)?
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
        Some(compatibility_aliases::AliasInventory::capture(
            &catalog, bounds, stop,
        )?)
    } else {
        None
    };
    let mut manifest = Manifest {
        compatibility_aliases: aliases,
        schema: profile.manifest_schema().into(),
        native_documents: profile.native_documents,
        include_documentation: profile.include_documentation.then_some(true),
        include_restrictions: profile.include_restrictions.then_some(true),
        profile_digest: profile.digest.clone(),
        binding,
        reference_generation: native.view.generation_id().into(),
        reference_view_digest: native.view.self_digest().into(),
        native_projection_profile: native.schema.into(),
        available_type_resolution_profile: native.type_resolution_profile().map(str::to_owned),
        source_origins: origins.values().cloned().collect(),
        partitions: native
            .view
            .partitions()
            .iter()
            .map(|p| Partition {
                id: p.id().into(),
                coverage: p.coverage(),
                record_count: p.records().len() as u64,
            })
            .collect(),
        conflicts: native.view.conflicts().to_vec(),
        issues: native
            .issues
            .iter()
            .map(|i| ProjectionIssue {
                code: i.code.into(),
                path: i.path.clone(),
                sha256: i.sha256.clone(),
                byte_range: [i.span.start, i.span.end],
            })
            .collect(),
        owner_negative_authority: native.negative_authority,
        documents: Vec::new(),
        privacy_scope: "local_metadata_only".into(),
        license_state: "not_evaluated".into(),
    };
    let header = canonical_size(&manifest)?;
    if header > bounds.max_owner_manifest_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let mut charged = header;
    for record in partition.records() {
        check(stop)?;
        let signature = native
            .callable_signature(record, stop)
            .map_err(native_error)?;
        if signature.reference_generation() != manifest.reference_generation
            || signature.reference_view_digest() != manifest.reference_view_digest
            || signature.record_key() != record.key()
            || record.kind() != ReferenceRecordKind::Api
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let source = origins
            .get(signature.source_id())
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if wow_reference::native_view::callable_origin_id(
            native.schema,
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
        let short = signature.name();
        let namespace = signature.namespace();
        let qualified_len = short
            .len()
            .checked_add(namespace.map_or(0, |n| n.len() + 1))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        if record.key().len() > bounds.max_document_id_bytes.get() as usize
            || qualified_len > bounds.max_owner_metadata_label_bytes.get() as usize
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        // Frozen projection of explicit owner fields; never parse the record key.
        let qualified = namespace.map_or_else(|| short.to_owned(), |n| format!("{n}.{short}"));
        let fields = |rows: &[NativeCallableField]| -> SearchResult<Vec<Field>> {
            rows.iter()
                .map(|row| {
                    check(stop)?;
                    let span = row.source_span();
                    Ok(Field {
                        name: row.name().into(),
                        original_type: row.type_name().into(),
                        original_inner_type: row.inner_type().map(str::to_owned),
                        nilable: row.nilable(),
                        byte_range: [span.start, span.end],
                    })
                })
                .collect()
        };
        let document = Document {
            restriction: None,
            documentation: profile
                .include_documentation
                .then(|| {
                    documentation::Documentation::capture(signature.documentation(), bounds, stop)
                })
                .transpose()?,
            entity_key: record.key().into(),
            owner_record_digest: signature.record_digest().into(),
            source: origins
                .get(signature.source_id())
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .clone(),
            canonical_short_name: short.into(),
            namespace: namespace.map(str::to_owned),
            canonical_qualified_name: qualified,
            arguments: fields(signature.arguments())?,
            returns: fields(signature.returns())?,
            type_authority: "original_source_type_identities_unresolved".into(),
            event: None,
            declaration: None,
        };
        charged = charged
            .checked_add(canonical_size(&document)? + u64::from(!manifest.documents.is_empty()))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        if charged > bounds.max_owner_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        manifest.documents.push(document);
    }
    if let Some(events) = event_partition {
        for record in events.records() {
            check(stop)?;
            let event = native.event_signature(record, stop).map_err(native_error)?;
            if event.reference_generation() != manifest.reference_generation
                || event.reference_view_digest() != manifest.reference_view_digest
                || event.record_key() != record.key()
                || record.kind() != ReferenceRecordKind::Event
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            let source = origins
                .get(event.source_id())
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if wow_reference::native_view::event_origin_id(
                native.schema,
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
            if record.key().len() > bounds.max_document_id_bytes.get() as usize
                || event.literal_name().len() > bounds.max_owner_metadata_label_bytes.get() as usize
            {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            let mut payload = Vec::new();
            for field in event.payload() {
                check(stop)?;
                let span = field.source_span();
                payload.push(Field {
                    name: field.name().into(),
                    original_type: field.type_name().into(),
                    original_inner_type: field.inner_type().map(str::to_owned),
                    nilable: field.nilable(),
                    byte_range: [span.start, span.end],
                });
            }
            let document = Document {
                restriction: None,
                documentation: profile
                    .include_documentation
                    .then(|| {
                        documentation::Documentation::capture(event.documentation(), bounds, stop)
                    })
                    .transpose()?,
                entity_key: record.key().into(),
                owner_record_digest: event.record_digest().into(),
                source: source.clone(),
                canonical_short_name: event.literal_name().into(),
                namespace: None,
                canonical_qualified_name: event.literal_name().into(),
                arguments: Vec::new(),
                returns: Vec::new(),
                type_authority: "original_source_type_identities_unresolved".into(),
                event: Some(EventDocument {
                    definition_name: event.definition_name().into(),
                    literal_name: event.literal_name().into(),
                    definition_namespace: event.namespace().map(str::to_owned),
                    payload,
                }),
                declaration: None,
            };
            charged = charged
                .checked_add(canonical_size(&document)? + u64::from(!manifest.documents.is_empty()))
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            if charged > bounds.max_owner_manifest_bytes.get() {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            manifest.documents.push(document);
        }
    }
    if let Some(partition) = declaration_partition {
        declarations::capture(
            native,
            partition,
            &origins,
            &mut manifest,
            bounds,
            &mut charged,
            stop,
        )?;
    }
    if profile.include_restrictions {
        let partition = native
            .view
            .partitions()
            .iter()
            .find(|part| part.id() == wow_reference::native_view::NATIVE_RESTRICTION_PARTITION)
            .filter(|part| part.coverage() == CoverageStatus::Partial)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        restrictions::capture(
            native,
            partition,
            &origins,
            &mut manifest,
            bounds,
            &mut charged,
            stop,
        )?;
    }
    manifest
        .documents
        .sort_by(|a, b| a.entity_key.cmp(&b.entity_key));
    let bytes =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 != charged {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let logical_id = format!("search-shard:sha256:{}", hex(&Sha256::digest(&bytes)));
    let rows = manifest
        .documents
        .iter()
        .map(|d| {
            Ok(SearchStoreDocument::new(
                d.entity_key.as_str(),
                documentation::row_text(d, bounds)?,
            ))
        })
        .collect::<SearchResult<Vec<_>>>()?;
    let physical = SearchStore::prepare(
        SearchStoreProfile::new(profile.digest.to_string(), bounds.physical()?)?,
        logical_id,
        &rows,
        &bytes,
        object_limits,
        stop,
    )?;
    Ok(PreparedReferenceShard { manifest, physical })
}

pub struct ReferenceSearchShard {
    manifest: Manifest,
    physical: PublishedSearchShard,
    bounds: SearchBounds,
}
impl ReferenceSearchShard {
    pub fn release(self, store: &mut Store) -> SearchResult<()> {
        let Self { physical, .. } = self;
        physical.release(store).map_err(SearchError::from)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn open_reference_callable_shard(
    store: &mut Store,
    logical_id: &str,
    object_id: &ObjectId,
    profile: &ReferenceSearchProfile,
    reference_object_id: &ObjectId,
    reference: &ReferenceView,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
    stop: &AtomicBool,
) -> SearchResult<ReferenceSearchShard> {
    validation::open(
        store,
        logical_id,
        object_id,
        profile,
        reference_object_id,
        reference,
        permission,
        lease,
        stop,
    )
    .map_err(validation::ValidationFailure::into_search_error)
}

fn native_error(error: wow_reference::native::NativeError) -> SearchError {
    if error.code == wow_reference::native::NativeErrorCode::Cancelled {
        SearchError::Cancelled
    } else {
        SearchError::OwnerEvidenceInvalid
    }
}
fn identity<T: Serialize + ?Sized>(prefix: &str, value: &T) -> SearchResult<Box<str>> {
    let bytes =
        wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    Ok(format!("{prefix}sha256:{}", hex(&Sha256::digest(bytes))).into())
}
fn canonical_size<T: Serialize + ?Sized>(value: &T) -> SearchResult<u64> {
    wow_core::canonical_json_bytes(value)
        .map(|v| v.len() as u64)
        .map_err(|_| SearchError::OwnerEvidenceInvalid)
}
fn check(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}
