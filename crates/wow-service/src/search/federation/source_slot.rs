//! Finite source-family dispatch; legacy Graph binding JSON stays unchanged.
use super::*;
use crate::search::source_observations::{
    self, BoundSourceRequest, ObservationComponent, ObservationRetainedBinding,
};
use wow_search::federation::RetainedSourceShard;

pub(super) enum SourceRequest {
    Entities(EntityRetrievalRequest),
    Observations(BoundSourceRequest),
}
pub(super) enum SourceResult {
    Entities(wow_search::entities::EntityQueryResult),
    Observations(wow_search::source_observations::BoundObservationQueryResult),
}
impl SourceResult {
    pub(super) fn entity(&self) -> ServiceResult<&wow_search::entities::EntityQueryResult> {
        match self {
            Self::Entities(v) => Ok(v),
            _ => Err(invalid()),
        }
    }
    pub(super) fn observation(
        &self,
    ) -> ServiceResult<&wow_search::source_observations::BoundObservationQueryResult> {
        match self {
            Self::Observations(v) => Ok(v),
            _ => Err(invalid()),
        }
    }
}
impl SourceRequest {
    pub(super) fn input(&self) -> serde_json::Value {
        match self {
            Self::Entities(r) => entity::federation_input(r),
            Self::Observations(r) => source_observations::federation_input(r),
        }
    }
    pub(super) fn fence(&self, batch: &mut WriteBatch) -> ServiceResult<()> {
        match self {
            Self::Entities(r) => entity::fence_result(r, batch),
            Self::Observations(r) => source_observations::fence_result(r, batch),
        }
    }
    pub(super) fn retained(&self) -> SourceRetainedBinding {
        match self {
            Self::Entities(r) => SourceRetainedBinding::Entities(entity::retained_binding(r)),
            Self::Observations(r) => {
                SourceRetainedBinding::Observations(source_observations::retained_binding(r))
            }
        }
    }
    pub(super) fn acquire(
        &self,
        store: &mut Store,
        digest: &str,
        purpose: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<SourceResult> {
        match self {
            Self::Entities(r) => Ok(SourceResult::Entities(entity::acquire_result_as(
                r, store, digest, purpose, stop,
            )?)),
            Self::Observations(r) => Ok(SourceResult::Observations(
                source_observations::acquire_bound(r, store, digest, purpose, stop)?,
            )),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum SourceRetainedBinding {
    Entities(entity::EntityRetainedBinding),
    Observations(ObservationRetainedBinding),
}
impl SourceRetainedBinding {
    pub(super) fn object_id(&self) -> &wow_store::ObjectId {
        match self {
            Self::Entities(v) => &v.object_id,
            Self::Observations(v) => &v.object_id,
        }
    }
    pub(super) fn uses_observations(&self) -> bool {
        matches!(self, Self::Observations(_))
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn open(
        &self,
        store: &mut Store,
        digest: &str,
        purpose: &str,
        expires: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<OpenedSource> {
        match self {
            Self::Entities(v) => Ok(OpenedSource::Entities(Box::new(
                if purpose == "federated-page-primary" {
                    entity::open_retained_shard(v, store, digest, expires, permission, stop)?
                } else {
                    entity::open_retained_shard_as(
                        v, store, digest, purpose, expires, permission, stop,
                    )?
                },
            ))),
            Self::Observations(v) => Ok(OpenedSource::Observations(Box::new(
                source_observations::open_retained(
                    v, store, digest, purpose, expires, permission, stop,
                )?,
            ))),
        }
    }
}
pub(super) enum OpenedSource {
    Entities(Box<wow_search::entities::EntitySearchShard>),
    Observations(Box<wow_search::source_observations::ObservationSearchShard>),
}
impl OpenedSource {
    pub(super) fn view(&self) -> RetainedSourceShard<'_> {
        match self {
            Self::Entities(v) => RetainedSourceShard::Entities(v),
            Self::Observations(v) => RetainedSourceShard::Observations(v),
        }
    }
    pub(super) fn release(self, store: &mut Store) -> wow_search::SearchResult<()> {
        match self {
            Self::Entities(v) => v.release(store),
            Self::Observations(v) => v.release(store),
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn from_input(
    p: Primary,
    observed: bool,
    operation: &OperationId,
    store: &std::path::Path,
    limits: SearchStoreLimits,
    max_bytes: NonZeroU64,
    expires: u64,
    permission: LocalMetadataPermission,
    stop: &AtomicBool,
) -> ServiceResult<SourceRequest> {
    if !observed {
        if p.source_component.is_some() {
            return Err(invalid());
        }
        return Ok(SourceRequest::Entities(source_request(
            p, operation, store, limits, max_bytes, expires, permission, stop,
        )?));
    }
    let component = p.source_component.ok_or_else(invalid)?;
    let hold = SearchSourceHoldBinding::new(
        p.project_store_root,
        OperationId::new(p.build_operation_id).map_err(|_| invalid())?,
        p.source_store_epoch_id,
        p.source_generation_hold_id,
        p.source_generation_hold_digest,
    )?;
    let query = SourceObservationRetrievalRequest::new_selected(
        operation.clone(),
        store.to_path_buf(),
        &p.generation,
        &p.logical_shard_id,
        &p.object_id,
        &p.profile_digest,
        &encode(&p.query)?,
        p.bounds,
        limits,
        max_bytes,
        expires,
        permission,
        component.declarations(),
        stop,
    )?;
    Ok(SourceRequest::Observations(BoundSourceRequest {
        query,
        hold,
    }))
}
pub(super) fn present_component<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ObservationComponent>, D::Error> {
    ObservationComponent::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_source_discriminator_preserves_legacy_bytes_and_rejects_ambiguous_forms()
    -> ServiceResult<()> {
        let legacy = serde_json::json!({"generation":format!("project-store-generation:sha256:{}","1".repeat(64)),
            "logical_id":format!("search-shard:sha256:{}","2".repeat(64)),"object_id":format!("store-object:sha256:{}","3".repeat(64)),
            "profile_digest":format!("sha256:{}","4".repeat(64)),"bounds":{"max_documents":8,"max_document_id_bytes":256,"max_owner_metadata_label_bytes":4096,
                "max_literal_query_bytes":64,"max_candidate_results":8,"max_owner_evidence_records":128,"max_owner_manifest_bytes":65536,"max_database_bytes":65536,"max_store_object_bytes":131072}});
        let parse = |v: &serde_json::Value| {
            serde_json::from_value::<SourceRetainedBinding>(v.clone()).map_err(|_| invalid())
        };
        let old = parse(&legacy)?;
        if old.uses_observations() || serde_json::to_value(old).map_err(|_| invalid())? != legacy {
            return Err(invalid());
        }
        let mut observed = legacy.clone();
        observed["component"] = "source_declarations".into();
        let named = parse(&observed)?;
        if !named.uses_observations()
            || serde_json::to_value(named).map_err(|_| invalid())? != observed
        {
            return Err(invalid());
        }
        for bad in [
            serde_json::Value::Null,
            "source_aliases".into(),
            "graph_entities".into(),
        ] {
            observed["component"] = bad;
            if parse(&observed).is_ok() {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
