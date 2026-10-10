use super::model::{ProjectStoreId, encode, failure, invalid, named};
use crate::{StoreError, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};

const IDENTITY_SCHEMA: &str = "wow-store/project-store-identity/1";

/// Logical identifiers only; filesystem roots are separate runtime inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectStoreNamespaceRequest {
    pub logical_namespace: String,
    pub owner_project_id: String,
}

/// Inert identity descriptor. Decoding reconstructs its identity from the request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NamespaceWire")]
pub struct ProjectStoreNamespace {
    schema: String,
    request: ProjectStoreNamespaceRequest,
    id: ProjectStoreId,
}

impl ProjectStoreNamespace {
    pub fn new(request: ProjectStoreNamespaceRequest) -> StoreResult<Self> {
        if !named(&request.logical_namespace) || !named(&request.owner_project_id) {
            return Err(failure(StoreErrorCode::ConfigurationInvalid));
        }
        let bytes = encode(
            &(
                IDENTITY_SCHEMA,
                "project",
                &request.logical_namespace,
                &request.owner_project_id,
            ),
            65536,
        )?;
        Ok(Self {
            schema: IDENTITY_SCHEMA.into(),
            request,
            id: ProjectStoreId::derive(&bytes),
        })
    }

    pub fn id(&self) -> &ProjectStoreId {
        &self.id
    }

    pub fn logical_namespace(&self) -> &str {
        &self.request.logical_namespace
    }

    pub fn owner_project_id(&self) -> &str {
        &self.request.owner_project_id
    }

    pub fn validate(&self) -> StoreResult<()> {
        if self != &Self::new(self.request.clone())? {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamespaceWire {
    schema: String,
    request: ProjectStoreNamespaceRequest,
    id: ProjectStoreId,
}

impl TryFrom<NamespaceWire> for ProjectStoreNamespace {
    type Error = StoreError;

    fn try_from(wire: NamespaceWire) -> StoreResult<Self> {
        let expected = Self::new(wire.request)?;
        if wire.schema != expected.schema || wire.id != expected.id {
            return Err(invalid());
        }
        Ok(expected)
    }
}
