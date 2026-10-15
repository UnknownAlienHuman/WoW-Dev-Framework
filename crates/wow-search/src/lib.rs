#![forbid(unsafe_code)]

//! Exact-generation, metadata-only search for retained project files.
//!
//! Single-file and explicit-set projections use leased immutable `wow-store`
//! SQLite/FTS5 objects. The explicit-set component also provides ranked retrieval
//! and whole-candidate pages; service secures continuation retention. These
//! metadata profiles establish neither corpus completeness nor negative authority.

use std::{
    fmt,
    num::{NonZeroU32, NonZeroU64},
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_project::{
    ProjectError, ProjectFileId, ProjectFileRecord, replay::publication::AcquiredProjectPair,
};
use wow_store::{
    LeaseId, LeaseRecord, LogicalEpoch, ObjectId, PendingObject, Store, StoreError, StoreErrorCode,
    project::{ReadSnapshot, StoreGenerationId},
    search::{
        PreparedSearchShard, PublishedSearchShard, SearchStore, SearchStoreDocument,
        SearchStoreLimits, SearchStoreProfile,
    },
};

pub use wow_store::search::SearchStoreLeaseRequest;

mod file_set;
pub mod reference;
pub use file_set::{
    PreparedProjectFileSetShard, ProjectFileSetSelection, RetrievalCursor, RetrievalPage,
    RetrievalPagePolicy, RetrievalRequest, RetrievalResult, RetrievalState,
    SearchFileSetBuildProfile, SearchFileSetOpenRequest, SearchFileSetPrepareRequest,
    SearchFileSetShard, SearchQuery, SearchRetrievalLane, open_project_file_set_shard,
    prepare_project_file_set_shard,
};

const SEARCH_PROFILE_SCHEMA: &str = "wow-search/project-file-local-metadata-profile/1";
const SEARCH_MANIFEST_SCHEMA: &str = "wow-search/project-file-shard-manifest/1";
const SEARCH_DOCUMENT_SCHEMA: &str = "wow-search/project-file-document/1";
const FTS_CANDIDATE_PROFILE: &str = "unicode61-literal-phrase-candidate/1";

pub type SearchResult<T> = Result<T, SearchError>;

#[derive(Debug)]
pub enum SearchError {
    Store(StoreError),
    Project(ProjectError),
    InvalidProfile,
    OwnerEvidenceInvalid,
    ShardLeaseCleanupOutcomeUnknown {
        lease_id: LeaseId,
        holder: Box<str>,
        object_id: ObjectId,
        shard_id: Box<str>,
        expires_after: LogicalEpoch,
        release_error: StoreErrorCode,
    },
    SelectedFileUnavailable,
    FileSetEmpty,
    FileSetDuplicate,
    FileSetOverCapacity,
    FileSetBudgetExceeded,
    Cancelled,
    LocalMetadataPermissionRequired,
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Store(_) => "search store operation failed",
            Self::Project(_) => "retained project owner read failed",
            Self::InvalidProfile => "search profile is invalid",
            Self::OwnerEvidenceInvalid => "typed owner evidence is invalid or exceeds its bound",
            Self::ShardLeaseCleanupOutcomeUnknown { .. } => {
                "search shard lease cleanup outcome is unknown"
            }
            Self::SelectedFileUnavailable => {
                "selected ProjectFile is absent from the exact owner generation"
            }
            Self::FileSetEmpty => "explicit ProjectFile set is empty",
            Self::FileSetDuplicate => "explicit ProjectFile set contains a duplicate ID",
            Self::FileSetOverCapacity => "explicit ProjectFile set exceeds the document cap",
            Self::FileSetBudgetExceeded => "ProjectFile set exceeds its continuing metadata bound",
            Self::Cancelled => "search shard operation was cancelled",
            Self::LocalMetadataPermissionRequired => {
                "explicit local metadata permission is required"
            }
        })
    }
}

impl std::error::Error for SearchError {}

impl From<StoreError> for SearchError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ProjectError> for SearchError {
    fn from(value: ProjectError) -> Self {
        Self::Project(value)
    }
}

/// Every first-slice resource bound is explicit, finite, and nonzero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchBounds {
    max_documents: NonZeroU32,
    max_document_id_bytes: NonZeroU32,
    max_owner_metadata_label_bytes: NonZeroU32,
    max_literal_query_bytes: NonZeroU32,
    max_candidate_results: NonZeroU32,
    max_owner_evidence_records: NonZeroU32,
    max_owner_manifest_bytes: NonZeroU64,
    max_database_bytes: NonZeroU64,
    max_store_object_bytes: NonZeroU64,
}

impl SearchBounds {
    #[must_use]
    pub const fn max_documents(&self) -> u32 {
        self.max_documents.get()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_documents: NonZeroU32,
        max_document_id_bytes: NonZeroU32,
        max_owner_metadata_label_bytes: NonZeroU32,
        max_literal_query_bytes: NonZeroU32,
        max_candidate_results: NonZeroU32,
        max_owner_evidence_records: NonZeroU32,
        max_owner_manifest_bytes: NonZeroU64,
        max_database_bytes: NonZeroU64,
        max_store_object_bytes: NonZeroU64,
    ) -> SearchResult<Self> {
        if max_database_bytes > max_store_object_bytes {
            return Err(SearchError::InvalidProfile);
        }
        Ok(Self {
            max_documents,
            max_document_id_bytes,
            max_owner_metadata_label_bytes,
            max_literal_query_bytes,
            max_candidate_results,
            max_owner_evidence_records,
            max_owner_manifest_bytes,
            max_database_bytes,
            max_store_object_bytes,
        })
    }

    fn physical(self) -> SearchResult<SearchStoreLimits> {
        Ok(SearchStoreLimits::new(
            self.max_documents.get(),
            self.max_document_id_bytes.get(),
            self.max_owner_metadata_label_bytes.get(),
            self.max_literal_query_bytes.get(),
            self.max_candidate_results.get(),
            self.max_owner_manifest_bytes.get(),
            self.max_database_bytes.get(),
            self.max_store_object_bytes.get(),
        )?)
    }
}

/// Candidate implementation profile; its digest binds the field mapping,
/// candidate-only tokenizer semantics, local-only privacy scope, and all bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBuildProfile {
    bounds: SearchBounds,
    digest: Box<str>,
}

impl SearchBuildProfile {
    pub fn new(bounds: SearchBounds) -> SearchResult<Self> {
        #[derive(Serialize)]
        struct Identity {
            schema: &'static str,
            document_schema: &'static str,
            exact_entity_owner_field: &'static str,
            label_owner_field: &'static str,
            label_authority: &'static str,
            fts_candidate_profile: &'static str,
            privacy_scope: &'static str,
            bounds: SearchBounds,
        }
        let bytes = wow_core::canonical_json_bytes(&Identity {
            schema: SEARCH_PROFILE_SCHEMA,
            document_schema: SEARCH_DOCUMENT_SCHEMA,
            exact_entity_owner_field: "ProjectFileRecord.file_id()",
            label_owner_field: "ProjectFileRecord.relative_path()",
            label_authority: "OwnerMetadataLabel",
            fts_candidate_profile: FTS_CANDIDATE_PROFILE,
            privacy_scope: "explicit-local-metadata-only",
            bounds,
        })
        .map_err(|_| SearchError::InvalidProfile)?;
        let digest = format!("sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str();
        let _ = bounds.physical()?;
        Ok(Self { bounds, digest })
    }

    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    #[must_use]
    pub const fn bounds(&self) -> SearchBounds {
        self.bounds
    }
}

/// Required capability token for a caller that explicitly authorizes indexing
/// the allowlisted local owner metadata fields in a local shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalMetadataPermission {
    authorized: bool,
}

impl LocalMetadataPermission {
    #[must_use]
    pub const fn authorize_local_metadata_only() -> Self {
        Self { authorized: true }
    }
}

#[derive(Debug, Clone)]
pub struct SearchShardBuildRequest {
    store_generation_id: StoreGenerationId,
    file_id: ProjectFileId,
    profile: SearchBuildProfile,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
}

impl SearchShardBuildRequest {
    #[must_use]
    pub fn new(
        store_generation_id: StoreGenerationId,
        file_id: ProjectFileId,
        profile: SearchBuildProfile,
        permission: LocalMetadataPermission,
        lease: SearchStoreLeaseRequest,
    ) -> Self {
        Self {
            store_generation_id,
            file_id,
            profile,
            permission,
            lease,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchShardPrepareRequest {
    store_generation_id: StoreGenerationId,
    file_id: ProjectFileId,
    profile: SearchBuildProfile,
    permission: LocalMetadataPermission,
}

impl SearchShardPrepareRequest {
    #[must_use]
    pub fn new(
        store_generation_id: StoreGenerationId,
        file_id: ProjectFileId,
        profile: SearchBuildProfile,
        permission: LocalMetadataPermission,
    ) -> Self {
        Self {
            store_generation_id,
            file_id,
            profile,
            permission,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchShardOpenRequest {
    store_generation_id: StoreGenerationId,
    file_id: ProjectFileId,
    profile: SearchBuildProfile,
    logical_id: Box<str>,
    object_id: ObjectId,
    lease: SearchStoreLeaseRequest,
    permission: LocalMetadataPermission,
}

impl SearchShardOpenRequest {
    #[must_use]
    pub fn new(
        store_generation_id: StoreGenerationId,
        file_id: ProjectFileId,
        profile: SearchBuildProfile,
        logical_id: impl Into<Box<str>>,
        object_id: ObjectId,
        lease: SearchStoreLeaseRequest,
        permission: LocalMetadataPermission,
    ) -> Self {
        Self {
            store_generation_id,
            file_id,
            profile,
            logical_id: logical_id.into(),
            object_id,
            lease,
            permission,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerMetadataLabel(Box<str>);

impl OwnerMetadataLabel {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileSearchDocument {
    schema: Box<str>,
    entity_id: ProjectFileId,
    owner_metadata_label: OwnerMetadataLabel,
    content_digest: serde_json::Value,
    byte_length: u64,
    language_kind: serde_json::Value,
    role: serde_json::Value,
    source_origin_id: Box<str>,
    workspace_id: Box<str>,
}

impl ProjectFileSearchDocument {
    #[must_use]
    pub fn entity_id(&self) -> &ProjectFileId {
        &self.entity_id
    }

    #[must_use]
    pub fn owner_metadata_label(&self) -> &OwnerMetadataLabel {
        &self.owner_metadata_label
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchSourceBinding {
    store_generation_id: Box<str>,
    project_id: Box<str>,
    project_profile: serde_json::Value,
    reference_generation_id: Box<str>,
    project_snapshot_id: Box<str>,
    project_generation_id: Box<str>,
    analyzer_snapshot_id: Box<str>,
    publication_set_id: Box<str>,
    graph_snapshot_id: Box<str>,
    graph_generation_id: Box<str>,
    graph_universe_id: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphPartitionEvidence {
    partition_id: Box<str>,
    partition_digest: Box<str>,
    producer_version: Box<str>,
    coverage: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GraphConflictManifestState {
    NotExposedByTypedOwnerRead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchShardManifest {
    schema: Box<str>,
    profile_digest: Box<str>,
    source: SearchSourceBinding,
    document: ProjectFileSearchDocument,
    project_coverage_and_conflicts: Vec<serde_json::Value>,
    graph_coverage: Vec<serde_json::Value>,
    graph_partitions: Vec<GraphPartitionEvidence>,
    graph_conflict_manifest: GraphConflictManifestState,
    source_provenance_present: bool,
}

pub struct PreparedProjectFileShard {
    logical_id: Box<str>,
    manifest: SearchShardManifest,
    manifest_bytes: Box<[u8]>,
    physical: PreparedSearchShard,
}

impl PreparedProjectFileShard {
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
        &self.manifest.source.store_generation_id
    }

    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }

    #[must_use]
    pub fn document(&self) -> &ProjectFileSearchDocument {
        &self.manifest.document
    }

    #[must_use]
    pub fn into_object(self) -> PendingObject {
        self.physical.into_object()
    }
}

pub struct SearchShard {
    logical_id: Box<str>,
    manifest: SearchShardManifest,
    physical: PublishedSearchShard,
}

impl SearchShard {
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
        &self.manifest.source.store_generation_id
    }

    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }

    #[must_use]
    pub fn document(&self) -> &ProjectFileSearchDocument {
        &self.manifest.document
    }

    /// A miss is scoped to this exact one-file shard and is never a corpus
    /// absence claim. The returned match proves only the exact owner ID relation.
    pub fn exact_id_lookup(&self, file_id: &ProjectFileId) -> SearchResult<Option<SearchExactHit>> {
        let row = self.physical.reader().exact_id(file_id.as_str())?;
        row.map(|row| {
            self.document_for(
                row.document_id(),
                row.owner_metadata_label(),
                SearchEvidenceClass::ExactEntityIdentity,
            )
        })
        .transpose()
    }

    /// Exact, case-sensitive string relation; this does not imply user intent.
    pub fn exact_label_lookup(&self, label: &str) -> SearchResult<Vec<SearchExactHit>> {
        self.physical
            .reader()
            .exact_label(label)?
            .into_iter()
            .map(|row| {
                self.document_for(
                    row.document_id(),
                    row.owner_metadata_label(),
                    SearchEvidenceClass::ExactOwnerMetadataLabel,
                )
            })
            .collect()
    }

    /// FTS text is returned only as candidate evidence; it never creates an
    /// alias, canonical name, or authoritative miss.
    pub fn literal_candidates(&self, literal: &str) -> SearchResult<Vec<SearchCandidate>> {
        self.physical
            .reader()
            .literal_candidates(literal)?
            .into_iter()
            .map(|row| {
                Ok(SearchCandidate {
                    entity_id: ProjectFileId::parse(row.document_id())?,
                    owner_metadata_label: OwnerMetadataLabel(row.owner_metadata_label().into()),
                    evidence: SearchEvidenceClass::Candidate,
                })
            })
            .collect()
    }

    pub fn release(self, store: &mut Store) -> SearchResult<()> {
        release_physical(self.physical, store)
    }

    fn document_for(
        &self,
        document_id: &str,
        label: &str,
        evidence: SearchEvidenceClass,
    ) -> SearchResult<SearchExactHit> {
        if document_id != self.manifest.document.entity_id.as_str()
            || label != self.manifest.document.owner_metadata_label.as_str()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        Ok(SearchExactHit {
            document: self.manifest.document.clone(),
            evidence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEvidenceClass {
    ExactEntityIdentity,
    ExactOwnerMetadataLabel,
    Candidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchExactHit {
    document: ProjectFileSearchDocument,
    evidence: SearchEvidenceClass,
}

impl SearchExactHit {
    #[must_use]
    pub fn document(&self) -> &ProjectFileSearchDocument {
        &self.document
    }

    #[must_use]
    pub const fn evidence(&self) -> SearchEvidenceClass {
        self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCandidate {
    entity_id: ProjectFileId,
    owner_metadata_label: OwnerMetadataLabel,
    evidence: SearchEvidenceClass,
}

impl SearchCandidate {
    #[must_use]
    pub fn entity_id(&self) -> &ProjectFileId {
        &self.entity_id
    }

    #[must_use]
    pub fn owner_metadata_label(&self) -> &OwnerMetadataLabel {
        &self.owner_metadata_label
    }

    #[must_use]
    pub const fn evidence(&self) -> SearchEvidenceClass {
        self.evidence
    }
}

pub fn build_project_file_shard(
    read: &ReadSnapshot,
    request: SearchShardBuildRequest,
    store: &mut Store,
    stop: &AtomicBool,
) -> SearchResult<SearchShard> {
    let SearchShardBuildRequest {
        store_generation_id,
        file_id,
        profile,
        permission,
        lease,
    } = request;
    let prepared = prepare_project_file_shard(
        read,
        SearchShardPrepareRequest::new(store_generation_id, file_id, profile.clone(), permission),
        store.configuration().limits(),
        stop,
    )?;
    let PreparedProjectFileShard {
        logical_id,
        manifest,
        manifest_bytes,
        physical,
    } = prepared;
    let object_id = physical.object_id().clone();
    let mut batch = wow_store::WriteBatch::new();
    batch.add_object(physical.into_object())?;
    store.commit(batch)?;
    let physical_profile =
        SearchStoreProfile::new(profile.digest().to_owned(), profile.bounds().physical()?)?;
    let physical = SearchStore::open_exact(
        store,
        physical_profile,
        logical_id.clone(),
        &object_id,
        lease,
        stop,
    )?;
    let physical = verify_reopened_owner_manifest(physical, &manifest_bytes, store)?;
    Ok(SearchShard {
        logical_id,
        manifest,
        physical,
    })
}

pub fn prepare_project_file_shard(
    read: &ReadSnapshot,
    request: SearchShardPrepareRequest,
    object_limits: wow_store::StoreLimits,
    stop: &AtomicBool,
) -> SearchResult<PreparedProjectFileShard> {
    if stop.load(Ordering::Acquire) {
        return Err(SearchError::Cancelled);
    }
    if read.manifest().generation_id != request.store_generation_id {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    if !request.permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    let pair = AcquiredProjectPair::read(read, stop)?;
    if stop.load(Ordering::Acquire) {
        return Err(SearchError::Cancelled);
    }
    let project = pair.project();
    let file = project
        .file_by_id(&request.file_id)
        .ok_or(SearchError::SelectedFileUnavailable)?;
    let document = project_file_document(file, request.profile.bounds)?;
    let evidence = owner_manifest_evidence(read, &pair, request.profile.bounds, stop)?;
    let manifest = SearchShardManifest {
        schema: SEARCH_MANIFEST_SCHEMA.into(),
        profile_digest: request.profile.digest.clone(),
        source: evidence.source,
        document: document.clone(),
        project_coverage_and_conflicts: evidence.project_coverage_and_conflicts,
        graph_coverage: evidence.graph_coverage,
        graph_partitions: evidence.graph_partitions,
        graph_conflict_manifest: evidence.graph_conflict_manifest,
        source_provenance_present: evidence.source_provenance_present,
    };
    let manifest_bytes =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if manifest_bytes.len() as u64 > request.profile.bounds.max_owner_manifest_bytes.get() {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let logical_id = format!(
        "search-shard:sha256:{}",
        hex(&Sha256::digest(&manifest_bytes))
    )
    .into_boxed_str();
    let physical_profile = SearchStoreProfile::new(
        request.profile.digest.to_string(),
        request.profile.bounds.physical()?,
    )?;
    let physical = SearchStore::prepare(
        physical_profile,
        logical_id.to_string(),
        &[SearchStoreDocument::new(
            document.entity_id.as_str(),
            document.owner_metadata_label.as_str(),
        )],
        &manifest_bytes,
        object_limits,
        stop,
    )?;
    Ok(PreparedProjectFileShard {
        logical_id,
        manifest,
        manifest_bytes: manifest_bytes.into_boxed_slice(),
        physical,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerManifestEvidence {
    source: SearchSourceBinding,
    project_coverage_and_conflicts: Vec<serde_json::Value>,
    graph_coverage: Vec<serde_json::Value>,
    graph_partitions: Vec<GraphPartitionEvidence>,
    graph_conflict_manifest: GraphConflictManifestState,
    source_provenance_present: bool,
}

fn owner_manifest_evidence(
    read: &ReadSnapshot,
    pair: &AcquiredProjectPair,
    bounds: SearchBounds,
    stop: &AtomicBool,
) -> SearchResult<OwnerManifestEvidence> {
    let project = pair.project();
    let project_coverage = project.project_coverage_records();
    let graph = pair.graph();
    let graph_coverage = graph.snapshot().coverage();
    let graph_partitions = graph.partitions();
    let mut evidence_count = project_coverage.len();
    evidence_count = evidence_count
        .checked_add(graph_coverage.len())
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    for partition in graph_partitions {
        evidence_count = evidence_count
            .checked_add(partition.coverage().len())
            .and_then(|value| value.checked_add(1))
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
    }
    if evidence_count > bounds.max_owner_evidence_records.get() as usize
        || graph_partitions.len() > bounds.max_owner_evidence_records.get() as usize
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }

    let source = SearchSourceBinding {
        store_generation_id: read.manifest().generation_id.as_str().into(),
        project_id: project
            .configuration()
            .project_id()
            .to_string()
            .into_boxed_str(),
        project_profile: serde_json::to_value(project.configuration().selected_profile())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
        reference_generation_id: project
            .configuration()
            .reference_generation()
            .to_string()
            .into_boxed_str(),
        project_snapshot_id: project.snapshot_id().into(),
        project_generation_id: project.project_generation().to_string().into_boxed_str(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        publication_set_id: pair.publication_set_id().into(),
        graph_snapshot_id: graph.snapshot().snapshot_id().to_string().into_boxed_str(),
        graph_generation_id: graph.snapshot().generation().to_string().into_boxed_str(),
        graph_universe_id: graph.snapshot().universe().to_string().into_boxed_str(),
    };
    let project_coverage_and_conflicts = serialize_owner_records(project_coverage)?;
    let graph_coverage = serialize_owner_records(graph_coverage)?;
    let mut partition_evidence = Vec::with_capacity(graph_partitions.len());
    for partition in graph_partitions {
        if stop.load(Ordering::Acquire) {
            return Err(SearchError::Cancelled);
        }
        partition_evidence.push(GraphPartitionEvidence {
            partition_id: partition.partition_id().into(),
            partition_digest: partition.partition_digest().into(),
            producer_version: partition.producer_version().into(),
            coverage: serialize_owner_records(partition.coverage())?,
        });
    }
    Ok(OwnerManifestEvidence {
        source,
        project_coverage_and_conflicts,
        graph_coverage,
        graph_partitions: partition_evidence,
        graph_conflict_manifest: GraphConflictManifestState::NotExposedByTypedOwnerRead,
        source_provenance_present: pair.source_provenance().is_some(),
    })
}

pub fn open_project_file_shard(
    store: &mut Store,
    request: SearchShardOpenRequest,
    stop: &AtomicBool,
) -> SearchResult<SearchShard> {
    if stop.load(Ordering::Acquire) {
        return Err(SearchError::Cancelled);
    }
    if !request.permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    let physical_profile = SearchStoreProfile::new(
        request.profile.digest().to_owned(),
        request.profile.bounds().physical()?,
    )?;
    let logical_id = request.logical_id;
    let object_id = request.object_id;
    let store_generation_id = request.store_generation_id;
    let file_id = request.file_id;
    let profile = request.profile;
    let physical = SearchStore::open_exact(
        store,
        physical_profile,
        logical_id.clone(),
        &object_id,
        request.lease,
        stop,
    )?;
    let manifest = match validate_opened_manifest(
        &physical,
        &logical_id,
        &object_id,
        &store_generation_id,
        &file_id,
        &profile,
    ) {
        Ok(manifest) => manifest,
        Err(error) => return Err(release_after_owner_error(physical, store, error)),
    };
    Ok(SearchShard {
        logical_id,
        manifest,
        physical,
    })
}

fn validate_opened_manifest(
    physical: &PublishedSearchShard,
    logical_id: &str,
    object_id: &ObjectId,
    store_generation_id: &StoreGenerationId,
    file_id: &ProjectFileId,
    profile: &SearchBuildProfile,
) -> SearchResult<SearchShardManifest> {
    let bytes = physical.reader().owner_manifest_bytes();
    let manifest: SearchShardManifest =
        serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let canonical =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let expected_logical_id = format!("search-shard:sha256:{}", hex(&Sha256::digest(bytes)));
    if canonical.as_slice() != bytes
        || physical.object_id() != object_id
        || physical.search_shard_id() != logical_id
        || expected_logical_id != logical_id
        || physical.reader().profile_digest() != profile.digest()
        || physical.reader().document_count() != 1
        || manifest.schema.as_ref() != SEARCH_MANIFEST_SCHEMA
        || manifest.profile_digest.as_ref() != profile.digest()
        || manifest.source.store_generation_id.as_ref() != store_generation_id.as_str()
        || &manifest.document.entity_id != file_id
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let physical_document = physical
        .reader()
        .exact_id(file_id.as_str())?
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    if physical_document.owner_metadata_label() != manifest.document.owner_metadata_label.as_str() {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let labels = physical
        .reader()
        .exact_label(manifest.document.owner_metadata_label.as_str())?;
    if labels.len() != 1
        || labels[0].document_id() != file_id.as_str()
        || labels[0].owner_metadata_label() != manifest.document.owner_metadata_label.as_str()
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    Ok(manifest)
}

fn verify_reopened_owner_manifest(
    physical: PublishedSearchShard,
    expected_manifest: &[u8],
    store: &mut Store,
) -> SearchResult<PublishedSearchShard> {
    if physical.reader().owner_manifest_bytes() == expected_manifest {
        return Ok(physical);
    }
    Err(release_after_owner_error(
        physical,
        store,
        SearchError::OwnerEvidenceInvalid,
    ))
}

fn release_physical(physical: PublishedSearchShard, store: &mut Store) -> SearchResult<()> {
    let lease_id = physical.lease().lease_id().clone();
    let holder = physical.lease().holder().into();
    let object_id = physical.object_id().clone();
    let shard_id = physical.search_shard_id().into();
    let expires_after = physical.lease().expires_after();
    match physical.release(store) {
        Ok(()) => Ok(()),
        Err(error) => Err(SearchError::ShardLeaseCleanupOutcomeUnknown {
            lease_id,
            holder,
            object_id,
            shard_id,
            expires_after,
            release_error: error.code(),
        }),
    }
}

fn release_after_owner_error(
    physical: PublishedSearchShard,
    store: &mut Store,
    owner_error: SearchError,
) -> SearchError {
    match release_physical(physical, store) {
        Ok(()) => owner_error,
        Err(cleanup_error) => cleanup_error,
    }
}

fn project_file_document(
    file: &ProjectFileRecord,
    bounds: SearchBounds,
) -> SearchResult<ProjectFileSearchDocument> {
    if file.file_id().as_str().len() > bounds.max_document_id_bytes.get() as usize
        || file.relative_path().as_str().len()
            > bounds.max_owner_metadata_label_bytes.get() as usize
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let manifest = file.manifest();
    Ok(ProjectFileSearchDocument {
        schema: SEARCH_DOCUMENT_SCHEMA.into(),
        entity_id: file.file_id().clone(),
        owner_metadata_label: OwnerMetadataLabel(file.relative_path().as_str().into()),
        content_digest: serde_json::to_value(file.content_digest())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
        byte_length: file.byte_length(),
        language_kind: serde_json::to_value(manifest.language_kind())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
        role: serde_json::to_value(manifest.role())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
        source_origin_id: file.source_origin_id().to_string().into_boxed_str(),
        workspace_id: file.workspace_id().to_string().into_boxed_str(),
    })
}

fn serialize_owner_records<T: Serialize>(records: &[T]) -> SearchResult<Vec<serde_json::Value>> {
    records
        .iter()
        .map(|record| serde_json::to_value(record).map_err(|_| SearchError::OwnerEvidenceInvalid))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        num::{NonZeroU32, NonZeroU64},
        sync::atomic::AtomicBool,
    };

    use wow_store::{
        LeaseId, LogicalEpoch, Store, StoreConfiguration, StoreErrorCode, StoreLimits,
    };

    use super::{
        PublishedSearchShard, SearchBounds, SearchBuildProfile, SearchError, SearchStore,
        SearchStoreDocument, SearchStoreLeaseRequest, SearchStoreLimits, SearchStoreProfile,
        verify_reopened_owner_manifest,
    };

    type TestResult<T = ()> = Result<T, Box<dyn Error>>;

    fn profile(candidate_cap: u32) -> TestResult<SearchBuildProfile> {
        let bounds = SearchBounds::new(
            NonZeroU32::new(1).ok_or("nonzero document cap")?,
            NonZeroU32::new(128).ok_or("nonzero ID cap")?,
            NonZeroU32::new(256).ok_or("nonzero label cap")?,
            NonZeroU32::new(64).ok_or("nonzero query cap")?,
            NonZeroU32::new(candidate_cap).ok_or("nonzero candidate cap")?,
            NonZeroU32::new(32).ok_or("nonzero evidence cap")?,
            NonZeroU64::new(16_384).ok_or("nonzero manifest cap")?,
            NonZeroU64::new(256_000).ok_or("nonzero database cap")?,
            NonZeroU64::new(800_000).ok_or("nonzero object cap")?,
        )?;
        Ok(SearchBuildProfile::new(bounds)?)
    }

    fn physical_profile() -> TestResult<SearchStoreProfile> {
        let limits = SearchStoreLimits::new(1, 128, 256, 64, 1, 16_384, 256_000, 800_000)?;
        Ok(SearchStoreProfile::new(
            format!("sha256:{}", "a".repeat(64)),
            limits,
        )?)
    }

    fn test_store() -> TestResult<Store> {
        let limits = StoreLimits::new(1_000_000, 8, 8, 128, 8)?;
        Ok(Store::open_in_memory(StoreConfiguration::new(
            "search-owner-cleanup-test-v1",
            limits,
        )?)?)
    }

    fn physical_shard(
        store: &mut Store,
        shard_id: &str,
        lease_id: &str,
        owner_manifest: &[u8],
    ) -> TestResult<PublishedSearchShard> {
        Ok(SearchStore::build_and_publish(
            store,
            physical_profile()?,
            shard_id,
            &[SearchStoreDocument::new(
                "project-file:state.lua",
                "state.lua",
            )],
            owner_manifest,
            SearchStoreLeaseRequest::new(
                LeaseId::new(lease_id)?,
                "search-cleanup-test",
                LogicalEpoch::new(10),
            )?,
            &AtomicBool::new(false),
        )?)
    }

    #[test]
    fn profile_digest_binds_every_caller_supplied_bound() -> TestResult {
        assert_ne!(profile(1)?.digest(), profile(2)?.digest());
        Ok(())
    }

    #[test]
    fn post_acquire_owner_manifest_mismatch_cleans_lease_or_reports_identity() -> TestResult {
        let mut store = test_store()?;
        let physical = physical_shard(
            &mut store,
            "search-shard:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "search-owner-mismatch-release",
            br#"{"owner":"exact"}"#,
        )?;
        let object_id = physical.object_id().clone();
        assert_eq!(physical.lease().expires_after().get(), 10);

        let mismatch =
            verify_reopened_owner_manifest(physical, br#"{"owner":"different"}"#, &mut store);
        assert!(matches!(mismatch, Err(SearchError::OwnerEvidenceInvalid)));
        let collected = store.collect_garbage(LogicalEpoch::new(1), 8)?;
        assert_eq!(collected.expired_leases_deleted(), 0);
        assert!(collected.deleted().contains(&object_id));
        assert!(store.object(&object_id)?.is_none());

        let physical = physical_shard(
            &mut store,
            "search-shard:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "search-owner-mismatch-unknown",
            br#"{"owner":"exact"}"#,
        )?;
        let lease_id = physical.lease().lease_id().clone();
        let holder = physical.lease().holder().to_owned();
        let expected_object = physical.object_id().clone();
        let expected_shard = physical.search_shard_id().to_owned();
        let expected_expiry = physical.lease().expires_after();
        store.release_lease(&lease_id, &holder)?;

        let error =
            match verify_reopened_owner_manifest(physical, br#"{"owner":"different"}"#, &mut store)
            {
                Err(error) => error,
                Ok(_) => return Err("mismatched owner manifest returned a shard".into()),
            };
        match error {
            SearchError::ShardLeaseCleanupOutcomeUnknown {
                lease_id: actual_lease,
                holder: actual_holder,
                object_id: actual_object,
                shard_id: actual_shard,
                expires_after: actual_expiry,
                release_error,
            } => {
                assert_eq!(actual_lease, lease_id);
                assert_eq!(actual_holder.as_ref(), holder.as_str());
                assert_eq!(actual_object, expected_object);
                assert_eq!(actual_shard.as_ref(), expected_shard.as_str());
                assert_eq!(actual_expiry, expected_expiry);
                assert_eq!(release_error, StoreErrorCode::LeaseConflict);
            }
            other => return Err(format!("unexpected mismatch cleanup error: {other:?}").into()),
        }
        Ok(())
    }
}
