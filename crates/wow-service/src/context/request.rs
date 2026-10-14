use serde::{Deserialize, Serialize};
use wow_core::{ProfileId, ReferenceGenerationId};
use wow_graph::GraphNodeId;
use wow_project::ProjectId;
use wow_store::project::{CurrentRecordId, StoreGenerationId};

use super::{
    CONTEXT_MAP_ID_MAX_BYTES, PRIMARY_NAVIGATION_MAP_PROFILE_ID,
    PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID, failure,
};
use crate::{ServiceErrorCode, ServiceResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ContextMapPublication {
    Current,
    StoreGeneration(StoreGenerationId),
}
impl ContextMapPublication {
    pub fn store_generation(id: &str) -> ServiceResult<Self> {
        StoreGenerationId::parse(id)
            .map(Self::StoreGeneration)
            .map_err(|_| invalid())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ContextMapRoot {
    Project(ProjectId),
    GraphEntity(GraphNodeId),
}
impl ContextMapRoot {
    pub fn project(id: &str) -> ServiceResult<Self> {
        identifier(id)?;
        ProjectId::new(id).map(Self::Project).map_err(|_| invalid())
    }
    pub fn graph_entity(id: &str) -> ServiceResult<Self> {
        identifier(id)?;
        GraphNodeId::new(id)
            .map(Self::GraphEntity)
            .map_err(|_| invalid())
    }
}

/// Inert bounded selection; native owner admission proves each round's membership.
#[derive(Debug, Clone, Serialize)]
pub(super) struct ProgressiveExpansion {
    pub(super) profile_id: &'static str,
    pub(super) rounds: Vec<Vec<GraphNodeId>>,
}
impl ProgressiveExpansion {
    pub(super) fn new(
        level: super::ContextInspectLevel,
        root: &ContextMapRoot,
        rounds: Vec<Vec<ContextMapRoot>>,
    ) -> ServiceResult<Self> {
        let ContextMapRoot::GraphEntity(root) = root else {
            return Err(invalid());
        };
        if level != super::ContextInspectLevel::L1
            || rounds.is_empty()
            || rounds.len() > 4
            || rounds
                .iter()
                .any(|round| round.is_empty() || round.len() > 32)
        {
            return Err(invalid());
        }
        identifier(root.as_str())?;
        GraphNodeId::new(root.as_str()).map_err(|_| invalid())?;
        let mut normalized: Vec<Vec<GraphNodeId>> = Vec::with_capacity(rounds.len());
        for round in rounds {
            let mut ids = round
                .into_iter()
                .map(|root| match root {
                    ContextMapRoot::GraphEntity(id) => {
                        identifier(id.as_str())?;
                        GraphNodeId::new(id.as_str()).map_err(|_| invalid())
                    }
                    ContextMapRoot::Project(_) => Err(invalid()),
                })
                .collect::<ServiceResult<Vec<_>>>()?;
            ids.sort();
            if ids.windows(2).any(|pair| pair[0] == pair[1])
                || ids.binary_search(root).is_ok()
                || ids.iter().any(|id| {
                    normalized
                        .iter()
                        .any(|prior| prior.binary_search(id).is_ok())
                })
            {
                return Err(invalid());
            }
            normalized.push(ids);
        }
        Ok(Self {
            profile_id: super::PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
            rounds: normalized,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextMapCurrentGuard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) record: Option<CurrentRecordId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) digest: Option<Box<str>>,
}
impl ContextMapCurrentGuard {
    pub fn new(record: Option<&str>, digest: Option<&str>) -> ServiceResult<Self> {
        if record.is_none() && digest.is_none() {
            return Err(invalid());
        }
        let record = record
            .map(CurrentRecordId::parse)
            .transpose()
            .map_err(|_| invalid())?;
        let digest = digest
            .map(|id| {
                crate::identity::validate_digest(id, "current record digest")?;
                Ok::<_, crate::ServiceError>(Box::<str>::from(id))
            })
            .transpose()?;
        Ok(Self { record, digest })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMapReferenceGuard {
    pub(super) profile: ProfileId,
    pub(super) generation: ReferenceGenerationId,
    pub(super) view: Box<str>,
}
impl ContextMapReferenceGuard {
    pub fn new(profile: &str, generation: &str, view: &str) -> ServiceResult<Self> {
        identifier(profile)?;
        crate::identity::validate_digest(view, "reference view digest")?;
        Ok(Self {
            profile: profile.parse().map_err(|_| invalid())?,
            generation: generation.parse().map_err(|_| invalid())?,
            view: view.into(),
        })
    }
}

/// Registration key is configured explicitly; no key is inferred from a host path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextMapRequest {
    schema: &'static str,
    pub(super) project_store_id: Box<str>,
    pub(super) publication: ContextMapPublication,
    pub(super) map_profile_id: &'static str,
    platform: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) root: Option<ContextMapRoot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) current: Option<ContextMapCurrentGuard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reference: Option<ContextMapReferenceGuard>,
}
impl ContextMapRequest {
    pub fn primary(
        store: &str,
        publication: ContextMapPublication,
        profile: &str,
        root: Option<ContextMapRoot>,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> ServiceResult<Self> {
        registration_id(store)?;
        let map_profile_id = match profile {
            PRIMARY_NAVIGATION_MAP_PROFILE_ID => PRIMARY_NAVIGATION_MAP_PROFILE_ID,
            PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID => PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
            _ => return Err(invalid()),
        };
        if (map_profile_id == PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID
            && matches!(root, Some(ContextMapRoot::GraphEntity(_))))
            || (current.is_some() && !matches!(publication, ContextMapPublication::Current))
        {
            return Err(invalid());
        }
        Ok(Self {
            schema: "wow-service/context-map-request/1",
            project_store_id: store.into(),
            publication,
            map_profile_id,
            platform: "omitted",
            root,
            current,
            reference,
        })
    }
}

pub(super) fn identifier(value: &str) -> ServiceResult<()> {
    if value.is_empty()
        || value.len() > CONTEXT_MAP_ID_MAX_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn registration_id(value: &str) -> ServiceResult<()> {
    identifier(value)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    }) {
        return Err(invalid());
    }
    Ok(())
}
fn invalid() -> crate::ServiceError {
    failure(ServiceErrorCode::InvalidRequest)
}
