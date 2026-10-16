//! Closed document selection for the existing durable Search lifecycle.
use super::*;
use wow_search::entities::{
    EntitySearchProfile, EntitySearchShard, PreparedEntityShard, open_project_entity_shard,
    prepare_project_entity_shard,
};
use wow_search::source_observations::{
    ObservationSearchProfile, ObservationSearchShard, PreparedObservationShard,
    open_source_observation_shard, prepare_source_observation_shard,
};
use wow_search::{
    PreparedProjectFileSetShard, ProjectFileSetSelection, SearchFileSetBuildProfile,
    SearchFileSetOpenRequest, SearchFileSetPrepareRequest, SearchFileSetShard,
    open_project_file_set_shard, prepare_project_file_set_shard,
};
pub(super) const OBSERVATION_COMPONENT: &str = "project_source_observations_v1";
pub(super) const ENTITY_COMPONENT: &str = "project_graph_entities_v1";

pub(super) const FILE_SET_COMPONENT: &str = "project_file_set_v1";

impl SearchIndexBuildRequest {
    /// Build the explicitly selected set; no representative query file is required.
    #[allow(clippy::too_many_arguments)]
    pub fn new_file_set(
        operation_id: OperationId,
        project_store_root: PathBuf,
        search_store_path: PathBuf,
        store_generation_id: impl Into<Box<str>>,
        ids: Vec<String>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        if !project_store_root.is_absolute() || !search_store_path.is_absolute() {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search requires explicit absolute source and SearchStore paths",
            ));
        }
        Ok(Self {
            operation_id,
            project_store_root,
            search_store_path,
            store_generation_id: store_generation_id.into(),
            selection: Selection::file_set(ids, bounds, stop)?,
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}

impl SearchShardRetireRequest {
    /// Retire the exact complete set bound by the original source hold.
    #[allow(clippy::too_many_arguments)]
    pub fn new_file_set(
        operation_id: OperationId,
        search_store_path: PathBuf,
        source_hold: SearchSourceHoldBinding,
        store_generation_id: impl Into<Box<str>>,
        ids: Vec<String>,
        logical_shard_id: impl Into<Box<str>>,
        object_id: impl Into<Box<str>>,
        profile_digest: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        if !search_store_path.is_absolute() || lease_expires_after == 0 {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search retirement requires an absolute path and positive lease epoch",
            ));
        }
        Ok(Self {
            operation_id,
            search_store_path,
            source_hold,
            store_generation_id: store_generation_id.into(),
            selection: Selection::file_set(ids, bounds, stop)?,
            logical_shard_id: logical_shard_id.into(),
            object_id: object_id.into(),
            profile_digest: profile_digest.into(),
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}

pub(super) enum Selection {
    Single(Box<str>),
    FileSet(ProjectFileSetSelection),
    GraphEntities,
    SourceObservations,
}

#[derive(Serialize)]
pub(super) struct Identity<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    file_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_ids: Option<&'a [ProjectFileId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_component: Option<&'static str>,
}

impl Selection {
    pub(super) fn file_set(
        ids: Vec<String>,
        bounds: SearchBounds,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if ids.len() > bounds.max_documents() as usize {
            return Err(failure(
                ServiceErrorCode::BudgetExceeded,
                "ProjectFile set exceeds document cap",
            ));
        }
        let profile = SearchFileSetBuildProfile::new(bounds).map_err(map_search_error)?;
        let ids = ids
            .into_iter()
            .map(|id| {
                check_cancelled(stop)?;
                ProjectFileId::parse(&id).map_err(map_project_error)
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        Ok(Self::FileSet(
            ProjectFileSetSelection::new(ids, &profile, stop).map_err(map_search_error)?,
        ))
    }

    pub(super) fn is_file_set(&self) -> bool {
        matches!(self, Self::FileSet(_))
    }

    pub(super) fn identity(&self) -> Identity<'_> {
        match self {
            Self::Single(id) => Identity {
                file_id: Some(id),
                file_ids: None,
                document_component: None,
            },
            Self::FileSet(selection) => Identity {
                file_id: None,
                file_ids: Some(selection.ids()),
                document_component: Some(FILE_SET_COMPONENT),
            },
            Self::SourceObservations => Identity {
                file_id: None,
                file_ids: None,
                document_component: Some(OBSERVATION_COMPONENT),
            },
            Self::GraphEntities => Identity {
                file_id: None,
                file_ids: None,
                document_component: Some(ENTITY_COMPONENT),
            },
        }
    }

    pub(super) fn normalize(&self) -> ServiceResult<Documents<'_>> {
        match self {
            Self::Single(id) => Ok(Documents::Single(
                ProjectFileId::parse(id).map_err(map_project_error)?,
            )),
            Self::FileSet(selection) => Ok(Documents::FileSet(selection)),
            Self::GraphEntities => Ok(Documents::GraphEntities),
            Self::SourceObservations => Ok(Documents::SourceObservations),
        }
    }
}

pub(super) enum Documents<'a> {
    Single(ProjectFileId),
    FileSet(&'a ProjectFileSetSelection),
    GraphEntities,
    SourceObservations,
}

impl Documents<'_> {
    pub(super) fn identity(&self) -> Identity<'_> {
        match self {
            Self::Single(id) => Identity {
                file_id: Some(id.as_str()),
                file_ids: None,
                document_component: None,
            },
            Self::FileSet(selection) => Identity {
                file_id: None,
                file_ids: Some(selection.ids()),
                document_component: Some(FILE_SET_COMPONENT),
            },
            Self::SourceObservations => Identity {
                file_id: None,
                file_ids: None,
                document_component: Some(OBSERVATION_COMPONENT),
            },
            Self::GraphEntities => Identity {
                file_id: None,
                file_ids: None,
                document_component: Some(ENTITY_COMPONENT),
            },
        }
    }
    pub(super) fn single_id(&self) -> Option<&ProjectFileId> {
        match self {
            Self::Single(id) => Some(id),
            Self::FileSet(_) => None,
            Self::GraphEntities => None,
            Self::SourceObservations => None,
        }
    }
}

pub(super) enum Profile {
    Single(SearchBuildProfile),
    FileSet(SearchFileSetBuildProfile),
    GraphEntities(EntitySearchProfile),
    SourceObservations(ObservationSearchProfile),
}

impl Profile {
    pub(super) fn for_selection(
        bounds: SearchBounds,
        selection: &Selection,
    ) -> ServiceResult<Self> {
        if matches!(selection, Selection::SourceObservations) {
            Ok(Self::SourceObservations(
                ObservationSearchProfile::new(bounds).map_err(map_search_error)?,
            ))
        } else if matches!(selection, Selection::GraphEntities) {
            Ok(Self::GraphEntities(
                EntitySearchProfile::new(bounds).map_err(map_search_error)?,
            ))
        } else {
            Self::new(bounds, selection.is_file_set())
        }
    }
    pub(super) fn new(bounds: SearchBounds, file_set: bool) -> ServiceResult<Self> {
        if file_set {
            Ok(Self::FileSet(
                SearchFileSetBuildProfile::new(bounds).map_err(map_search_error)?,
            ))
        } else {
            Ok(Self::Single(
                SearchBuildProfile::new(bounds).map_err(map_search_error)?,
            ))
        }
    }
    pub(super) fn digest(&self) -> &str {
        match self {
            Self::Single(profile) => profile.digest(),
            Self::FileSet(profile) => profile.digest(),
            Self::GraphEntities(profile) => profile.digest(),
            Self::SourceObservations(profile) => profile.digest(),
        }
    }
    pub(super) fn component(&self) -> Option<&'static str> {
        match self {
            Self::Single(_) => None,
            Self::FileSet(_) => Some(FILE_SET_COMPONENT),
            Self::GraphEntities(_) => Some(ENTITY_COMPONENT),
            Self::SourceObservations(_) => Some(OBSERVATION_COMPONENT),
        }
    }
    pub(super) fn prepare(
        &self,
        read: &wow_store::project::ReadSnapshot,
        generation: &StoreGenerationId,
        documents: &Documents<'_>,
        permission: LocalMetadataPermission,
        object_limits: SearchStoreLimits,
        stop: &AtomicBool,
    ) -> ServiceResult<Prepared> {
        match (self, documents) {
            (Self::SourceObservations(profile), Documents::SourceObservations) => {
                Ok(Prepared::SourceObservations(
                    prepare_source_observation_shard(
                        read,
                        generation,
                        profile,
                        permission,
                        object_limits,
                        stop,
                    )
                    .map_err(map_search_error)?,
                ))
            }
            (Self::GraphEntities(profile), Documents::GraphEntities) => {
                Ok(Prepared::GraphEntities(
                    prepare_project_entity_shard(
                        read,
                        generation,
                        profile,
                        permission,
                        object_limits,
                        stop,
                    )
                    .map_err(map_search_error)?,
                ))
            }
            (Self::Single(profile), Documents::Single(id)) => Ok(Prepared::Single(
                prepare_project_file_shard(
                    read,
                    SearchShardPrepareRequest::new(
                        generation.clone(),
                        id.clone(),
                        profile.clone(),
                        permission,
                    ),
                    object_limits,
                    stop,
                )
                .map_err(map_search_error)?,
            )),
            (Self::FileSet(profile), Documents::FileSet(selection)) => Ok(Prepared::FileSet(
                prepare_project_file_set_shard(
                    read,
                    SearchFileSetPrepareRequest::new(
                        generation.clone(),
                        (*selection).clone(),
                        profile.clone(),
                        permission,
                    ),
                    object_limits,
                    stop,
                )
                .map_err(map_search_error)?,
            )),
            _ => Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "Search component and document selection disagree",
            )),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn open(
        &self,
        store: &mut Store,
        generation: StoreGenerationId,
        file_id: Option<&ProjectFileId>,
        expected: Option<&Documents<'_>>,
        logical_id: &str,
        object_id: wow_store::ObjectId,
        lease: SearchStoreLeaseRequest,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Opened> {
        let opened = match self {
            Self::SourceObservations(profile) => Ok(Opened::SourceObservations(
                open_source_observation_shard(
                    store,
                    logical_id,
                    &object_id,
                    &generation,
                    profile,
                    permission,
                    lease,
                    stop,
                )
                .map_err(map_search_error)?,
            )),
            Self::GraphEntities(profile) => Ok(Opened::GraphEntities(
                open_project_entity_shard(
                    store,
                    logical_id,
                    &object_id,
                    &generation,
                    profile,
                    permission,
                    lease,
                    stop,
                )
                .map_err(map_search_error)?,
            )),
            Self::Single(profile) => Ok(Opened::Single(
                open_project_file_shard(
                    store,
                    SearchShardOpenRequest::new(
                        generation,
                        file_id.cloned().ok_or_else(|| {
                            failure(
                                ServiceErrorCode::InvalidRequest,
                                "single-file reader requires its exact file",
                            )
                        })?,
                        profile.clone(),
                        logical_id.to_owned(),
                        object_id,
                        lease,
                        permission,
                    ),
                    stop,
                )
                .map_err(map_search_error)?,
            )),
            Self::FileSet(profile) => Ok(Opened::FileSet(
                open_project_file_set_shard(
                    store,
                    SearchFileSetOpenRequest::new(
                        logical_id.into(),
                        object_id,
                        generation,
                        profile.clone(),
                        permission,
                        lease,
                    ),
                    stop,
                )
                .map_err(map_search_error)?,
            )),
        }?;
        if let Some(expected) = expected {
            let matches = match (&opened, expected) {
                (Opened::Single(shard), Documents::Single(id)) => {
                    shard.document().entity_id() == id
                }
                (Opened::FileSet(shard), Documents::FileSet(selection)) => shard
                    .documents()
                    .iter()
                    .map(|document| document.entity_id())
                    .eq(selection.ids().iter()),
                (Opened::GraphEntities(_), Documents::GraphEntities) => true,
                (Opened::SourceObservations(_), Documents::SourceObservations) => true,
                _ => false,
            };
            if !matches {
                opened.release(store).map_err(map_search_error)?;
                return Err(failure(
                    ServiceErrorCode::IdentityMismatch,
                    "retiring selection differs from the exact stored document set",
                ));
            }
        }
        Ok(opened)
    }
}

pub(super) enum Prepared {
    Single(PreparedProjectFileShard),
    FileSet(PreparedProjectFileSetShard),
    GraphEntities(PreparedEntityShard),
    SourceObservations(PreparedObservationShard),
}
impl Prepared {
    pub(super) fn logical_id(&self) -> &str {
        match self {
            Self::Single(value) => value.logical_id(),
            Self::FileSet(value) => value.logical_id(),
            Self::GraphEntities(value) => value.logical_id(),
            Self::SourceObservations(value) => value.logical_id(),
        }
    }
    pub(super) fn object_id(&self) -> &wow_store::ObjectId {
        match self {
            Self::Single(value) => value.object_id(),
            Self::FileSet(value) => value.object_id(),
            Self::GraphEntities(value) => value.object_id(),
            Self::SourceObservations(value) => value.object_id(),
        }
    }
    pub(super) fn source_generation(&self) -> &str {
        match self {
            Self::Single(value) => value.source_generation(),
            Self::FileSet(value) => value.source_generation(),
            Self::GraphEntities(value) => value.source_generation(),
            Self::SourceObservations(value) => value.source_generation(),
        }
    }
    pub(super) fn profile_digest(&self) -> &str {
        match self {
            Self::Single(value) => value.profile_digest(),
            Self::FileSet(value) => value.profile_digest(),
            Self::GraphEntities(value) => value.profile_digest(),
            Self::SourceObservations(value) => value.profile_digest(),
        }
    }
    pub(super) fn transport_documents(&self) -> Vec<DurableDocument<'_>> {
        match self {
            Self::Single(value) => vec![DurableDocument {
                entity_id: value.document().entity_id().as_str(),
                owner_metadata_label: value.document().owner_metadata_label().as_str(),
                entity_kind: None,
            }],
            Self::FileSet(value) => value
                .documents()
                .iter()
                .map(|d| DurableDocument {
                    entity_id: d.entity_id().as_str(),
                    owner_metadata_label: d.owner_metadata_label().as_str(),
                    entity_kind: None,
                })
                .collect(),
            Self::SourceObservations(value) => value
                .documents()
                .iter()
                .map(|d| DurableDocument {
                    entity_id: d.fact_id(),
                    owner_metadata_label: d.recorded_name(),
                    entity_kind: Some(d.kind()),
                })
                .collect(),
            Self::GraphEntities(value) => value
                .documents()
                .iter()
                .map(|d| DurableDocument {
                    entity_id: d.entity_id().as_str(),
                    owner_metadata_label: d.owner_metadata_key(),
                    entity_kind: Some(d.entity_kind()),
                })
                .collect(),
        }
    }
    pub(super) fn into_object(self) -> PendingObject {
        match self {
            Self::Single(value) => value.into_object(),
            Self::FileSet(value) => value.into_object(),
            Self::GraphEntities(value) => value.into_object(),
            Self::SourceObservations(value) => value.into_object(),
        }
    }
}

pub(super) enum Opened {
    Single(wow_search::SearchShard),
    FileSet(SearchFileSetShard),
    GraphEntities(EntitySearchShard),
    SourceObservations(ObservationSearchShard),
}
impl Opened {
    pub(super) fn logical_id(&self) -> &str {
        match self {
            Self::Single(value) => value.logical_id(),
            Self::FileSet(value) => value.logical_id(),
            Self::GraphEntities(value) => value.logical_id(),
            Self::SourceObservations(value) => value.logical_id(),
        }
    }
    pub(super) fn object_id(&self) -> &wow_store::ObjectId {
        match self {
            Self::Single(value) => value.object_id(),
            Self::FileSet(value) => value.object_id(),
            Self::GraphEntities(value) => value.object_id(),
            Self::SourceObservations(value) => value.object_id(),
        }
    }
    pub(super) fn source_generation(&self) -> &str {
        match self {
            Self::Single(value) => value.source_generation(),
            Self::FileSet(value) => value.source_generation(),
            Self::GraphEntities(value) => value.source_generation(),
            Self::SourceObservations(value) => value.source_generation(),
        }
    }
    pub(super) fn profile_digest(&self) -> &str {
        match self {
            Self::Single(value) => value.profile_digest(),
            Self::FileSet(value) => value.profile_digest(),
            Self::GraphEntities(value) => value.profile_digest(),
            Self::SourceObservations(value) => value.profile_digest(),
        }
    }
    pub(super) fn is_file_set(&self) -> bool {
        matches!(self, Self::FileSet(_))
    }
    pub(super) fn exact_id_lookup(
        &self,
        id: &ProjectFileId,
    ) -> wow_search::SearchResult<Option<wow_search::SearchExactHit>> {
        match self {
            Self::Single(value) => value.exact_id_lookup(id),
            Self::FileSet(value) => value.exact_id_lookup(id),
            Self::GraphEntities(_) | Self::SourceObservations(_) => {
                Err(wow_search::SearchError::InvalidProfile)
            }
        }
    }
    pub(super) fn exact_label_lookup(
        &self,
        label: &str,
    ) -> wow_search::SearchResult<Vec<wow_search::SearchExactHit>> {
        match self {
            Self::Single(value) => value.exact_label_lookup(label),
            Self::FileSet(value) => value.exact_label_lookup(label),
            Self::GraphEntities(_) | Self::SourceObservations(_) => {
                Err(wow_search::SearchError::InvalidProfile)
            }
        }
    }
    pub(super) fn literal_candidates(
        &self,
        query: &str,
    ) -> wow_search::SearchResult<Vec<wow_search::SearchCandidate>> {
        match self {
            Self::Single(value) => value.literal_candidates(query),
            Self::FileSet(value) => value.literal_candidates(query),
            Self::GraphEntities(_) | Self::SourceObservations(_) => {
                Err(wow_search::SearchError::InvalidProfile)
            }
        }
    }
    pub(super) fn release(self, store: &mut Store) -> wow_search::SearchResult<()> {
        match self {
            Self::Single(value) => value.release(store),
            Self::FileSet(value) => value.release(store),
            Self::GraphEntities(value) => value.release(store),
            Self::SourceObservations(value) => value.release(store),
        }
    }
}
