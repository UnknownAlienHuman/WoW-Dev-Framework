//! Complete bounded ProjectEntity projection from one real coherent Project pair.
//! Graph owner keys remain opaque identity metadata, never canonical names.
use super::*;
use wow_graph::{GraphLimits, GraphNode, GraphNodeId};
mod discovery;
mod query;
pub use query::{
    EntityPage, EntityPageCursor, EntityPagePolicy, EntityQueryRequest, EntityQueryResult,
};
const PROFILE_SCHEMA: &str = "wow-search/project-graph-entity-profile/1";
const MANIFEST_SCHEMA: &str = "wow-search/project-graph-entity-shard/1";

#[derive(Debug, Clone)]
pub struct EntitySearchProfile {
    bounds: SearchBounds,
    digest: Box<str>,
}
impl EntitySearchProfile {
    pub fn new(bounds: SearchBounds) -> SearchResult<Self> {
        bounds.physical()?;
        Ok(Self {
            bounds,
            digest: digest(
                "",
                &(
                    PROFILE_SCHEMA,
                    MANIFEST_SCHEMA,
                    "GraphNode.node_id()/kind()/owner_key()/evidence_ids()",
                    "opaque-owner-key-metadata-only",
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
    pub fn bounds(&self) -> SearchBounds {
        self.bounds
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySearchDocument {
    entity: GraphNode,
}
impl EntitySearchDocument {
    pub fn entity_kind(&self) -> &str {
        self.entity.kind()
    }
    pub fn entity_id(&self) -> &GraphNodeId {
        self.entity.node_id()
    }
    pub fn owner_metadata_key(&self) -> &str {
        self.entity.owner_key()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    profile_digest: Box<str>,
    owner: OwnerManifestEvidence,
    graph_limits: GraphLimits,
    graph_registry_digest: Box<str>,
    documents: Vec<EntitySearchDocument>,
}
pub struct PreparedEntityShard {
    manifest: Manifest,
    physical: PreparedSearchShard,
}
impl PreparedEntityShard {
    pub fn logical_id(&self) -> &str {
        self.physical.search_shard_id()
    }
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    pub fn documents(&self) -> &[EntitySearchDocument] {
        &self.manifest.documents
    }
    pub fn into_object(self) -> PendingObject {
        self.physical.into_object()
    }
}

pub fn prepare_project_entity_shard(
    read: &ReadSnapshot,
    generation: &StoreGenerationId,
    profile: &EntitySearchProfile,
    permission: LocalMetadataPermission,
    object_limits: wow_store::StoreLimits,
    stop: &AtomicBool,
) -> SearchResult<PreparedEntityShard> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    if &read.manifest().generation_id != generation {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let pair = AcquiredProjectPair::read(read, stop)?;
    let graph = pair.graph().snapshot();
    if graph.nodes().is_empty() || graph.nodes().len() > profile.bounds.max_documents.get() as usize
    {
        return Err(SearchError::FileSetOverCapacity);
    }
    let owner = owner_manifest_evidence(read, &pair, profile.bounds, stop)?;
    let mut manifest = Manifest {
        schema: MANIFEST_SCHEMA.into(),
        profile_digest: profile.digest.clone(),
        owner,
        graph_limits: graph.limits(),
        graph_registry_digest: digest("", pair.graph().registry())?,
        documents: Vec::new(),
    };
    let mut charged = size(&manifest)?;
    if charged > profile.bounds.max_owner_manifest_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    for node in graph.nodes() {
        check(stop)?;
        if node.node_id().as_str().len() > profile.bounds.max_document_id_bytes.get() as usize
            || node.owner_key().len() > profile.bounds.max_owner_metadata_label_bytes.get() as usize
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let doc = EntitySearchDocument {
            entity: node.clone(),
        };
        charged = charged
            .checked_add(size(&doc)? + u64::from(!manifest.documents.is_empty()))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        if charged > profile.bounds.max_owner_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        manifest.documents.push(doc);
    }
    let bytes =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 != charged {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let logical_id = format!("search-shard:sha256:{}", hex(&Sha256::digest(&bytes)));
    let rows = manifest
        .documents
        .iter()
        .map(|d| SearchStoreDocument::new(d.entity_id().as_str(), d.owner_metadata_key()))
        .collect::<Vec<_>>();
    let physical = SearchStore::prepare(
        SearchStoreProfile::new(profile.digest.to_string(), profile.bounds.physical()?)?,
        logical_id,
        &rows,
        &bytes,
        object_limits,
        stop,
    )?;
    Ok(PreparedEntityShard { manifest, physical })
}

pub struct EntitySearchShard {
    manifest: Manifest,
    physical: PublishedSearchShard,
    bounds: SearchBounds,
}
impl EntitySearchShard {
    pub fn logical_id(&self) -> &str {
        self.physical.search_shard_id()
    }
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    pub fn documents(&self) -> &[EntitySearchDocument] {
        &self.manifest.documents
    }
    pub fn release(self, store: &mut Store) -> SearchResult<()> {
        self.physical.release(store).map_err(SearchError::from)
    }
}
#[allow(clippy::too_many_arguments)]
pub fn open_project_entity_shard(
    store: &mut Store,
    logical_id: &str,
    object_id: &ObjectId,
    generation: &StoreGenerationId,
    profile: &EntitySearchProfile,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
    stop: &AtomicBool,
) -> SearchResult<EntitySearchShard> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    let physical = SearchStore::open_exact(
        store,
        SearchStoreProfile::new(profile.digest.to_string(), profile.bounds.physical()?)?,
        logical_id.to_owned(),
        object_id,
        lease,
        stop,
    )?;
    let result = (|| {
        let bytes = physical.reader().owner_manifest_bytes();
        let manifest: Manifest =
            serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if manifest.schema != MANIFEST_SCHEMA
            || manifest.profile_digest != profile.digest
            || manifest.owner.source.store_generation_id.as_ref() != generation.as_str()
            || wow_core::canonical_json_bytes(&manifest)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != bytes
            || format!("search-shard:sha256:{}", hex(&Sha256::digest(bytes))) != logical_id
            || manifest.documents.len() != physical.reader().document_count() as usize
            || !manifest
                .documents
                .windows(2)
                .all(|p| p[0].entity_id() < p[1].entity_id())
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        for doc in &manifest.documents {
            check(stop)?;
            doc.entity
                .validate(manifest.graph_limits)
                .map_err(graph_error)?;
            if doc.entity.universe().as_str() != manifest.owner.source.graph_universe_id.as_ref()
                || doc.entity.generation().as_str()
                    != manifest.owner.source.graph_generation_id.as_ref()
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            let row = physical
                .reader()
                .exact_id(doc.entity_id().as_str())?
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if row.owner_metadata_label() != doc.owner_metadata_key() {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }
        Ok(manifest)
    })();
    match result {
        Ok(manifest) => Ok(EntitySearchShard {
            manifest,
            physical,
            bounds: profile.bounds,
        }),
        Err(error) => Err(release_after_owner_error(physical, store, error)),
    }
}
fn graph_error(error: wow_graph::GraphError) -> SearchError {
    match error.code() {
        wow_graph::GraphErrorCode::Cancelled => SearchError::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded | wow_graph::GraphErrorCode::LimitsInvalid => {
            SearchError::FileSetBudgetExceeded
        }
        _ => SearchError::OwnerEvidenceInvalid,
    }
}
fn check(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}
fn digest<T: Serialize + ?Sized>(prefix: &str, value: &T) -> SearchResult<Box<str>> {
    let bytes =
        wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    Ok(format!("{prefix}sha256:{}", hex(&Sha256::digest(bytes))).into())
}
fn size<T: Serialize + ?Sized>(value: &T) -> SearchResult<u64> {
    wow_core::canonical_json_bytes(value)
        .map(|v| v.len() as u64)
        .map_err(|_| SearchError::OwnerEvidenceInvalid)
}
