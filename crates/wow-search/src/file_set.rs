//! Explicit same-generation ProjectFile sets over the existing native Store.
use std::collections::BTreeSet;

use super::*;

const PROFILE_SCHEMA: &str = "wow-search/project-file-set-local-metadata-profile/1";
const MANIFEST_SCHEMA: &str = "wow-search/project-file-set-shard-manifest/1";

/// Separately selected component; the original single-file profile stays exact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFileSetBuildProfile {
    base: SearchBuildProfile,
    digest: Box<str>,
}

impl SearchFileSetBuildProfile {
    pub fn new(bounds: SearchBounds) -> SearchResult<Self> {
        let base = SearchBuildProfile::new(bounds)?;
        let bytes =
            wow_core::canonical_json_bytes(&(PROFILE_SCHEMA, MANIFEST_SCHEMA, base.digest()))
                .map_err(|_| SearchError::InvalidProfile)?;
        Ok(Self {
            base,
            digest: format!("sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str(),
        })
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    #[must_use]
    pub const fn bounds(&self) -> SearchBounds {
        self.base.bounds
    }
}

/// Exact caller selection; construction never discovers files or deduplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFileSetSelection {
    ids: Box<[ProjectFileId]>,
}

impl ProjectFileSetSelection {
    pub fn new(
        ids: Vec<ProjectFileId>,
        profile: &SearchFileSetBuildProfile,
        stop: &AtomicBool,
    ) -> SearchResult<Self> {
        check(stop)?;
        validate_selection_bounds(&ids, profile.bounds(), stop)?;
        let mut ordered = BTreeSet::new();
        for id in ids {
            check(stop)?;
            if !ordered.insert(id) {
                return Err(SearchError::FileSetDuplicate);
            }
        }
        let ids = ordered
            .into_iter()
            .map(|id| {
                check(stop)?;
                Ok(id)
            })
            .collect::<SearchResult<Vec<_>>>()?
            .into_boxed_slice();
        Ok(Self { ids })
    }

    #[must_use]
    pub fn ids(&self) -> &[ProjectFileId] {
        &self.ids
    }
}

fn check(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}

fn validate_selection_bounds(
    ids: &[ProjectFileId],
    bounds: SearchBounds,
    stop: &AtomicBool,
) -> SearchResult<()> {
    if ids.is_empty() {
        return Err(SearchError::FileSetEmpty);
    }
    // Refuse the raw count before any ordered-set allocation or sorting work.
    if ids.len() > bounds.max_documents.get() as usize {
        return Err(SearchError::FileSetOverCapacity);
    }
    let mut remaining = bounds.max_owner_manifest_bytes.get();
    for id in ids {
        check(stop)?;
        if id.as_str().len() > bounds.max_document_id_bytes.get() as usize {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        remaining = remaining
            .checked_sub(id.as_str().len() as u64)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
    }
    Ok(())
}

pub struct SearchFileSetPrepareRequest {
    generation: StoreGenerationId,
    selection: ProjectFileSetSelection,
    profile: SearchFileSetBuildProfile,
    permission: LocalMetadataPermission,
}

impl SearchFileSetPrepareRequest {
    #[must_use]
    pub fn new(
        generation: StoreGenerationId,
        selection: ProjectFileSetSelection,
        profile: SearchFileSetBuildProfile,
        permission: LocalMetadataPermission,
    ) -> Self {
        Self {
            generation,
            selection,
            profile,
            permission,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSetManifest {
    schema: Box<str>,
    profile_digest: Box<str>,
    documents: Vec<ProjectFileSearchDocument>,
    owner: OwnerManifestEvidence,
}

pub struct PreparedProjectFileSetShard {
    logical_id: Box<str>,
    manifest: FileSetManifest,
    physical: PreparedSearchShard,
}

impl PreparedProjectFileSetShard {
    #[must_use]
    pub fn logical_id(&self) -> &str {
        &self.logical_id
    }
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    #[must_use]
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    #[must_use]
    pub fn documents(&self) -> &[ProjectFileSearchDocument] {
        &self.manifest.documents
    }
    #[must_use]
    pub fn into_object(self) -> PendingObject {
        self.physical.into_object()
    }
}

pub fn prepare_project_file_set_shard(
    read: &ReadSnapshot,
    request: SearchFileSetPrepareRequest,
    object_limits: wow_store::StoreLimits,
    stop: &AtomicBool,
) -> SearchResult<PreparedProjectFileSetShard> {
    check(stop)?;
    if !request.permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    if read.manifest().generation_id != request.generation {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let bounds = request.profile.bounds();
    validate_selection_bounds(request.selection.ids(), bounds, stop)?;
    let pair = AcquiredProjectPair::read(read, stop)?;
    check(stop)?;
    let owner = owner_manifest_evidence(read, &pair, bounds, stop)?;
    let mut manifest = FileSetManifest {
        schema: MANIFEST_SCHEMA.into(),
        profile_digest: request.profile.digest.clone(),
        documents: Vec::new(),
        owner,
    };
    let header_bytes = wow_core::canonical_json_bytes(&manifest)
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?
        .len() as u64;
    let mut remaining = bounds
        .max_owner_manifest_bytes
        .get()
        .checked_sub(header_bytes)
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    for id in request.selection.ids() {
        check(stop)?;
        let file = pair
            .project()
            .file_by_id(id)
            .ok_or(SearchError::SelectedFileUnavailable)?;
        let document = project_file_document(file, bounds)?;
        let bytes = wow_core::canonical_json_bytes(&document)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let charge = (bytes.len() as u64)
            .checked_add(u64::from(!manifest.documents.is_empty()))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        remaining = remaining
            .checked_sub(charge)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        manifest.documents.push(document);
    }
    let bytes =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 != bounds.max_owner_manifest_bytes.get() - remaining {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    check(stop)?;
    let logical_id =
        format!("search-shard:sha256:{}", hex(&Sha256::digest(&bytes))).into_boxed_str();
    let documents = manifest
        .documents
        .iter()
        .map(|document| {
            check(stop)?;
            Ok(SearchStoreDocument::new(
                document.entity_id.as_str(),
                document.owner_metadata_label.as_str(),
            ))
        })
        .collect::<SearchResult<Vec<_>>>()?;
    let physical = SearchStore::prepare(
        SearchStoreProfile::new(request.profile.digest.to_string(), bounds.physical()?)?,
        logical_id.to_string(),
        &documents,
        &bytes,
        object_limits,
        stop,
    )?;
    Ok(PreparedProjectFileSetShard {
        logical_id,
        manifest,
        physical,
    })
}

pub struct SearchFileSetOpenRequest {
    logical_id: Box<str>,
    object_id: ObjectId,
    generation: StoreGenerationId,
    profile: SearchFileSetBuildProfile,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
}

impl SearchFileSetOpenRequest {
    #[must_use]
    pub fn new(
        logical_id: Box<str>,
        object_id: ObjectId,
        generation: StoreGenerationId,
        profile: SearchFileSetBuildProfile,
        permission: LocalMetadataPermission,
        lease: SearchStoreLeaseRequest,
    ) -> Self {
        Self {
            logical_id,
            object_id,
            generation,
            profile,
            permission,
            lease,
        }
    }
}

pub struct SearchFileSetShard {
    logical_id: Box<str>,
    manifest: FileSetManifest,
    physical: PublishedSearchShard,
}

pub fn open_project_file_set_shard(
    store: &mut Store,
    request: SearchFileSetOpenRequest,
    stop: &AtomicBool,
) -> SearchResult<SearchFileSetShard> {
    check(stop)?;
    if !request.permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    let physical = SearchStore::open_exact(
        store,
        SearchStoreProfile::new(
            request.profile.digest.to_string(),
            request.profile.bounds().physical()?,
        )?,
        request.logical_id.clone(),
        &request.object_id,
        request.lease,
        stop,
    )?;
    let manifest = match validate_manifest(
        &physical,
        &request.logical_id,
        &request.generation,
        &request.profile,
        stop,
    ) {
        Ok(manifest) => manifest,
        Err(error) => return Err(release_after_owner_error(physical, store, error)),
    };
    Ok(SearchFileSetShard {
        logical_id: request.logical_id,
        manifest,
        physical,
    })
}

fn validate_manifest(
    physical: &PublishedSearchShard,
    logical_id: &str,
    generation: &StoreGenerationId,
    profile: &SearchFileSetBuildProfile,
    stop: &AtomicBool,
) -> SearchResult<FileSetManifest> {
    check(stop)?;
    let bytes = physical.reader().owner_manifest_bytes();
    let manifest: FileSetManifest =
        serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let canonical =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let count =
        u32::try_from(manifest.documents.len()).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes != canonical
        || manifest.schema.as_ref() != MANIFEST_SCHEMA
        || manifest.profile_digest.as_ref() != profile.digest()
        || physical.reader().profile_digest() != profile.digest()
        || physical.search_shard_id() != logical_id
        || format!("search-shard:sha256:{}", hex(&Sha256::digest(bytes))) != logical_id
        || manifest.owner.source.store_generation_id.as_ref() != generation.as_str()
        || count == 0
        || count > profile.bounds().max_documents.get()
        || physical.reader().document_count() != count
        || !manifest
            .documents
            .windows(2)
            .all(|pair| pair[0].entity_id < pair[1].entity_id)
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    for document in &manifest.documents {
        check(stop)?;
        if document.schema.as_ref() != SEARCH_DOCUMENT_SCHEMA {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let row = physical
            .reader()
            .exact_id(document.entity_id.as_str())?
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if row.owner_metadata_label() != document.owner_metadata_label.as_str() {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
    }
    Ok(manifest)
}

impl SearchFileSetShard {
    #[must_use]
    pub fn logical_id(&self) -> &str {
        &self.logical_id
    }
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    #[must_use]
    pub fn lease(&self) -> &LeaseRecord {
        self.physical.lease()
    }
    #[must_use]
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    #[must_use]
    pub fn documents(&self) -> &[ProjectFileSearchDocument] {
        &self.manifest.documents
    }

    fn document(&self, id: &str, label: &str) -> SearchResult<&ProjectFileSearchDocument> {
        let index = self
            .manifest
            .documents
            .binary_search_by(|document| document.entity_id.as_str().cmp(id))
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let document = &self.manifest.documents[index];
        if document.owner_metadata_label.as_str() != label {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        Ok(document)
    }

    pub fn exact_id_lookup(&self, id: &ProjectFileId) -> SearchResult<Option<SearchExactHit>> {
        self.physical
            .reader()
            .exact_id(id.as_str())?
            .map(|row| {
                Ok(SearchExactHit {
                    document: self
                        .document(row.document_id(), row.owner_metadata_label())?
                        .clone(),
                    evidence: SearchEvidenceClass::ExactEntityIdentity,
                })
            })
            .transpose()
    }

    pub fn exact_label_lookup(&self, label: &str) -> SearchResult<Vec<SearchExactHit>> {
        self.physical
            .reader()
            .exact_label(label)?
            .into_iter()
            .map(|row| {
                Ok(SearchExactHit {
                    document: self
                        .document(row.document_id(), row.owner_metadata_label())?
                        .clone(),
                    evidence: SearchEvidenceClass::ExactOwnerMetadataLabel,
                })
            })
            .collect()
    }

    pub fn literal_candidates(&self, literal: &str) -> SearchResult<Vec<SearchCandidate>> {
        self.physical
            .reader()
            .literal_candidates(literal)?
            .into_iter()
            .map(|row| {
                let document = self.document(row.document_id(), row.owner_metadata_label())?;
                Ok(SearchCandidate {
                    entity_id: document.entity_id.clone(),
                    owner_metadata_label: document.owner_metadata_label.clone(),
                    evidence: SearchEvidenceClass::Candidate,
                })
            })
            .collect()
    }

    pub fn release(self, store: &mut Store) -> SearchResult<()> {
        release_physical(self.physical, store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn profile(max_documents: u32) -> SearchResult<SearchFileSetBuildProfile> {
        let mut bounds = SearchBuildProfile::new(SearchBounds::new(
            NonZeroU32::new(2).ok_or(SearchError::InvalidProfile)?,
            NonZeroU32::new(256).ok_or(SearchError::InvalidProfile)?,
            NonZeroU32::new(256).ok_or(SearchError::InvalidProfile)?,
            NonZeroU32::new(64).ok_or(SearchError::InvalidProfile)?,
            NonZeroU32::new(8).ok_or(SearchError::InvalidProfile)?,
            NonZeroU32::new(64).ok_or(SearchError::InvalidProfile)?,
            NonZeroU64::new(16384).ok_or(SearchError::InvalidProfile)?,
            NonZeroU64::new(262144).ok_or(SearchError::InvalidProfile)?,
            NonZeroU64::new(1048576).ok_or(SearchError::InvalidProfile)?,
        )?)?
        .bounds();
        bounds.max_documents = NonZeroU32::new(max_documents).ok_or(SearchError::InvalidProfile)?;
        SearchFileSetBuildProfile::new(bounds)
    }

    #[test]
    fn explicit_set_is_ordered_and_never_silently_deduplicated() -> TestResult {
        let stop = AtomicBool::new(false);
        let profile = profile(2)?;
        let alpha = ProjectFileId::parse("project-file:group/alpha.lua")?;
        let beta = ProjectFileId::parse("project-file:group/beta.lua")?;
        let selection =
            ProjectFileSetSelection::new(vec![beta.clone(), alpha.clone()], &profile, &stop)?;
        assert_eq!(selection.ids(), &[alpha.clone(), beta]);
        assert!(matches!(
            ProjectFileSetSelection::new(vec![alpha.clone(), alpha], &profile, &stop),
            Err(SearchError::FileSetDuplicate)
        ));
        assert!(matches!(
            ProjectFileSetSelection::new(Vec::new(), &profile, &stop),
            Err(SearchError::FileSetEmpty)
        ));
        Ok(())
    }

    #[test]
    fn raw_cap_and_cancellation_precede_set_allocation() -> TestResult {
        let alpha = ProjectFileId::parse("project-file:alpha.lua")?;
        let selected = vec![alpha.clone(), alpha];
        assert!(matches!(
            ProjectFileSetSelection::new(selected.clone(), &profile(1)?, &AtomicBool::new(false)),
            Err(SearchError::FileSetOverCapacity)
        ));
        assert!(matches!(
            ProjectFileSetSelection::new(selected, &profile(2)?, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}
