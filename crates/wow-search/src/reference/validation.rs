//! The same owner validator serves query admission and explicit inspection.
use super::*;
#[cfg(test)]
mod tests;

/// A verdict only for the native immutable shard open checks, not full E4 acceptance.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReferenceShardValidation {
    Valid { shard: Box<ReferenceShardMetadata> },
    Invalid { check: &'static str },
}

#[derive(Debug, Serialize)]
pub struct ReferenceShardMetadata {
    validation_scope: &'static str,
    full_e4_validation: &'static str,
    logical_shard_id: Box<str>,
    object_id: ObjectId,
    search_profile_digest: Box<str>,
    reference_profile: ProfileIdentity,
    source_revision: String,
    source_acquisition_partial: bool,
    source_acquisition_failure_count: usize,
    reference_generation: String,
    reference_view_digest: String,
    reference_view_object_id: ObjectId,
    native_projection_profile: String,
    document_count: usize,
    source_count: usize,
    partitions: Vec<PartitionStatus>,
    conflict_count: usize,
    projection_issue_count: usize,
    manifest_owner_negative_authority: bool,
    capabilities: Capabilities,
    physical_read_only_validated: bool,
    privacy_scope: String,
    license_state: String,
}
#[derive(Debug, Serialize)]
struct PartitionStatus {
    id: String,
    coverage: CoverageStatus,
    owner_record_count: u64,
}
#[derive(Debug, Serialize)]
struct Capabilities {
    callables: bool,
    events: bool,
    source_declarations: bool,
    documentation: bool,
    restrictions: bool,
    compatibility_aliases: bool,
    global_declarations: bool,
}

/// Reuses query admission without executing a retrieval lane or repairing data.
/// Only definite contradictions in the owner validator produce `Invalid`.
/// Acquisition, ambiguous bounded evidence, cancellation and cleanup remain errors.
#[allow(clippy::too_many_arguments)]
pub fn validate_reference_callable_shard(
    store: &mut Store,
    logical_id: &str,
    object_id: &ObjectId,
    profile: &ReferenceSearchProfile,
    reference_object_id: &ObjectId,
    reference: &ReferenceView,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
    stop: &AtomicBool,
) -> SearchResult<ReferenceShardValidation> {
    let shard = match open(
        store,
        logical_id,
        object_id,
        profile,
        reference_object_id,
        reference,
        permission,
        lease,
        stop,
    ) {
        Ok(shard) => shard,
        Err(ValidationFailure::Invalid(check)) => {
            return Ok(ReferenceShardValidation::Invalid { check });
        }
        Err(ValidationFailure::Failed(error)) => return Err(error),
    };
    let manifest = &shard.manifest;
    let metadata = ReferenceShardMetadata {
        validation_scope: "exact_native_reference_owner_and_physical_open",
        full_e4_validation: "not_evaluated",
        logical_shard_id: shard.physical.search_shard_id().into(),
        object_id: shard.physical.object_id().clone(),
        search_profile_digest: manifest.profile_digest.clone(),
        reference_profile: manifest.binding.profile.clone(),
        source_revision: manifest.binding.source_revision.clone(),
        source_acquisition_partial: manifest.binding.acquisition_partial,
        source_acquisition_failure_count: manifest.binding.acquisition_failure_receipts.len(),
        reference_generation: manifest.reference_generation.clone(),
        reference_view_digest: manifest.reference_view_digest.clone(),
        reference_view_object_id: manifest.binding.reference_view_object_id.clone(),
        native_projection_profile: manifest.native_projection_profile.clone(),
        document_count: manifest.documents.len(),
        source_count: manifest.source_origins.len(),
        partitions: manifest
            .partitions
            .iter()
            .map(|part| PartitionStatus {
                id: part.id.clone(),
                coverage: part.coverage,
                owner_record_count: part.record_count,
            })
            .collect(),
        conflict_count: manifest.conflicts.len(),
        projection_issue_count: manifest.issues.len(),
        manifest_owner_negative_authority: manifest.owner_negative_authority,
        capabilities: Capabilities {
            callables: true,
            events: manifest.native_documents.is_some(),
            source_declarations: manifest.native_documents
                == Some(NativeReferenceDocuments::SourceDeclarationsV1),
            documentation: manifest.include_documentation == Some(true),
            restrictions: manifest.include_restrictions == Some(true),
            compatibility_aliases: manifest.compatibility_aliases.is_some(),
            global_declarations: false,
        },
        physical_read_only_validated: true,
        privacy_scope: manifest.privacy_scope.clone(),
        license_state: manifest.license_state.clone(),
    };
    release_physical(shard.physical, store)?;
    Ok(ReferenceShardValidation::Valid {
        shard: Box::new(metadata),
    })
}

pub(super) enum ValidationFailure {
    Invalid(&'static str),
    Failed(SearchError),
}
impl From<SearchError> for ValidationFailure {
    fn from(error: SearchError) -> Self {
        Self::Failed(error)
    }
}
impl From<StoreError> for ValidationFailure {
    fn from(error: StoreError) -> Self {
        Self::Failed(error.into())
    }
}
impl ValidationFailure {
    pub(super) fn into_search_error(self) -> SearchError {
        match self {
            Self::Invalid(_) => SearchError::OwnerEvidenceInvalid,
            Self::Failed(error) => error,
        }
    }
    fn after_release(self, physical: PublishedSearchShard, store: &mut Store) -> Self {
        match release_physical(physical, store) {
            Ok(()) => self,
            Err(error) => Self::Failed(error),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn open(
    store: &mut Store,
    logical_id: &str,
    object_id: &ObjectId,
    profile: &ReferenceSearchProfile,
    reference_object_id: &ObjectId,
    reference: &ReferenceView,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
    stop: &AtomicBool,
) -> Result<ReferenceSearchShard, ValidationFailure> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired.into());
    }
    let physical = SearchStore::open_exact(
        store,
        SearchStoreProfile::new(profile.digest.to_string(), profile.bounds.physical()?)?,
        logical_id.to_owned(),
        object_id,
        lease,
        stop,
    )?;
    let result: Result<Manifest, ValidationFailure> = (|| {
        reference
            .validate()
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let bytes = physical.reader().owner_manifest_bytes();
        let manifest: Manifest = serde_json::from_slice(bytes)
            .map_err(|_| ValidationFailure::Invalid("manifest_encoding"))?;
        if manifest.schema != profile.manifest_schema()
            || manifest.compatibility_aliases.is_some() != profile.include_compatibility_aliases
            || manifest.native_documents != profile.native_documents
            || manifest.include_documentation != profile.include_documentation.then_some(true)
            || manifest.include_restrictions != profile.include_restrictions.then_some(true)
            || manifest.profile_digest != profile.digest
            || manifest.reference_generation != reference.generation_id()
            || manifest.reference_view_digest != reference.self_digest()
            || &manifest.binding.reference_view_object_id != reference_object_id
            || manifest.binding.reference_store_instance_id.as_ref()
                != store.store_instance_id()?.as_str()
            || manifest.binding.reference_store_configuration_id.as_ref()
                != store.configuration().configuration_id()
            || wow_core::canonical_json_bytes(&manifest)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != bytes
            || format!("search-shard:sha256:{}", hex(&Sha256::digest(bytes))) != logical_id
            || manifest.documents.len() != physical.reader().document_count() as usize
            || !manifest
                .documents
                .windows(2)
                .all(|p| p[0].entity_key < p[1].entity_key)
            || !manifest
                .source_origins
                .windows(2)
                .all(|p| p[0].source_id < p[1].source_id)
        {
            return Err(ValidationFailure::Invalid("owner_manifest_binding"));
        }
        if profile.include_compatibility_aliases {
            let catalog = wow_reference::compatibility_aliases::read(
                reference,
                &manifest.binding.profile,
                stop,
            )
            .map_err(native_error)?
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if manifest.compatibility_aliases.as_ref()
                != Some(&compatibility_aliases::AliasInventory::capture(
                    &catalog,
                    profile.bounds,
                    stop,
                )?)
            {
                return Err(ValidationFailure::Invalid("owner_manifest_binding"));
            }
        }
        let partition = reference
            .partitions()
            .iter()
            .find(|p| p.id() == NATIVE_API_PARTITION)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let mut records = partition
            .records()
            .iter()
            .map(|record| (record.key().to_owned(), record))
            .collect::<Vec<_>>();
        if profile.native_documents.is_some() {
            let events = reference
                .partitions()
                .iter()
                .find(|part| part.id() == NATIVE_EVENT_PARTITION)
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if events.coverage() != CoverageStatus::Partial {
                return Err(ValidationFailure::Invalid("owner_manifest_binding"));
            }
            records.extend(
                events
                    .records()
                    .iter()
                    .map(|record| (record.key().to_owned(), record)),
            );
        }
        if profile.native_documents == Some(NativeReferenceDocuments::SourceDeclarationsV1) {
            let declarations = reference
                .partitions()
                .iter()
                .find(|part| part.id() == NATIVE_DECLARATION_PARTITION)
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if declarations.coverage() != CoverageStatus::Partial {
                return Err(ValidationFailure::Invalid("owner_manifest_binding"));
            }
            records.extend(
                declarations
                    .records()
                    .iter()
                    .map(|record| (record.key().to_owned(), record)),
            );
        }
        if profile.include_restrictions {
            let partition = reference
                .partitions()
                .iter()
                .find(|part| part.id() == wow_reference::native_view::NATIVE_RESTRICTION_PARTITION)
                .filter(|part| part.coverage() == CoverageStatus::Partial)
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            for record in partition.records() {
                check(stop)?;
                records.push((
                    wow_reference::native_view::restriction_entity_id(
                        reference.generation_id(),
                        record.key(),
                    )
                    .map_err(native_error)?,
                    record,
                ));
            }
        }
        records.sort_by(|a, b| a.0.cmp(&b.0));
        if records.len() != manifest.documents.len() {
            return Err(ValidationFailure::Invalid("owner_manifest_binding"));
        }
        manifest
            .binding
            .profile
            .validate()
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if manifest.privacy_scope != "local_metadata_only"
            || manifest.license_state != "not_evaluated"
            || manifest.conflicts != reference.conflicts()
            || manifest.partitions.len() != reference.partitions().len()
            || manifest
                .partitions
                .iter()
                .zip(reference.partitions())
                .any(|(a, b)| {
                    a.id != b.id()
                        || a.coverage != b.coverage()
                        || a.record_count != b.records().len() as u64
                })
        {
            return Err(ValidationFailure::Invalid("owner_manifest_binding"));
        }
        let subject_index = if profile.include_restrictions {
            Some(restrictions::SubjectIndex::new(reference, stop)?)
        } else {
            None
        };
        for (doc, (entity_id, record)) in manifest.documents.iter().zip(records) {
            check(stop)?;
            let origin_id = if doc.declaration.is_some() {
                wow_reference::native_view::declaration_origin_id
            } else if doc.event.is_some() {
                wow_reference::native_view::event_origin_id
            } else {
                wow_reference::native_view::callable_origin_id
            };
            let row = physical
                .reader()
                .exact_id(&doc.entity_key)?
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if doc.entity_key != entity_id
                || record.kind()
                    != if doc.restriction.is_some() {
                        ReferenceRecordKind::Restriction
                    } else if let Some(declaration) = &doc.declaration {
                        declaration.kind.record_kind()
                    } else if doc.event.is_some() {
                        ReferenceRecordKind::Event
                    } else {
                        ReferenceRecordKind::Api
                    }
                || (doc.event.is_some() && profile.native_documents.is_none())
                || !declarations::valid_document(doc, profile.native_documents)
                || (doc.restriction.is_some() && !profile.include_restrictions)
                || !restrictions::valid(doc, record, reference, subject_index.as_ref())?
                || doc.event.as_ref().is_some_and(|event| {
                    event.literal_name != doc.canonical_short_name
                        || !doc.arguments.is_empty()
                        || !doc.returns.is_empty()
                        || doc.namespace.is_some()
                })
                || doc.owner_record_digest
                    != record
                        .digest()
                        .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                || record.source_ids().len() != 1
                || record.source_ids()[0].as_ref() != doc.source.source_id
                || row.owner_metadata_label() != documentation::row_text(doc, profile.bounds)?
                || doc.documentation.is_some() != profile.include_documentation
                || doc.type_authority
                    != if doc.restriction.is_some() {
                        "original_source_restriction_facets_only"
                    } else {
                        "original_source_type_identities_unresolved"
                    }
                || doc.canonical_qualified_name
                    != doc.namespace.as_ref().map_or_else(
                        || doc.canonical_short_name.clone(),
                        |n| format!("{n}.{}", doc.canonical_short_name),
                    )
                || manifest
                    .source_origins
                    .binary_search_by(|source| source.source_id.cmp(&doc.source.source_id))
                    .ok()
                    .is_none_or(|index| manifest.source_origins[index] != doc.source)
                || (if let Some(restriction) = &doc.restriction {
                    wow_reference::native_view::restriction_origin_id(
                        &manifest.native_projection_profile,
                        restriction.kind,
                        &manifest.binding.source_revision,
                        &doc.source.path,
                        &doc.source.sha256,
                        wow_reference::native::Span {
                            start: doc.source.byte_range[0],
                            end: doc.source.byte_range[1],
                        },
                    )
                } else {
                    origin_id(
                        &manifest.native_projection_profile,
                        &manifest.binding.source_revision,
                        &doc.source.path,
                        &doc.source.sha256,
                        wow_reference::native::Span {
                            start: doc.source.byte_range[0],
                            end: doc.source.byte_range[1],
                        },
                    )
                })
                .map_err(native_error)?
                    != doc.source.source_id
            {
                return Err(ValidationFailure::Invalid("owner_manifest_binding"));
            }
        }
        Ok(manifest)
    })();
    match result {
        Ok(manifest) => Ok(ReferenceSearchShard {
            manifest,
            physical,
            bounds: profile.bounds,
        }),
        Err(error) => Err(error.after_release(physical, store)),
    }
}
