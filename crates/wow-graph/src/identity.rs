use std::fmt;

use serde::{Deserialize, Serialize};
use wow_store::CatalogPath;

use crate::{GraphError, GraphErrorCode, GraphResult};

macro_rules! text_id {
    ($name:ident, $label:literal, $max:expr, $validator:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> GraphResult<Self> {
                let value = value.into();
                if value.is_empty() || value.len() > $max || !$validator(&value) {
                    return Err(GraphError::new(
                        GraphErrorCode::IdentifierInvalid,
                        concat!("invalid ", $label),
                    ));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

fn component(value: &str) -> bool {
    value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    })
}

fn node_id(value: &str) -> bool {
    digest_id(value, "graph-node:sha256:")
}

fn edge_id(value: &str) -> bool {
    digest_id(value, "graph-edge:sha256:")
}

fn snapshot_id(value: &str) -> bool {
    digest_id(value, "graph-snapshot:sha256:")
}

fn digest_id(value: &str, prefix: &str) -> bool {
    let Some(hex) = value.strip_prefix(prefix) else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

text_id!(GraphUniverseId, "graph universe id", 256, component);
text_id!(GraphGenerationId, "graph generation id", 256, component);
text_id!(GraphNodeId, "graph node id", 96, node_id);
text_id!(GraphEdgeId, "graph edge id", 96, edge_id);
text_id!(GraphSnapshotId, "graph snapshot id", 100, snapshot_id);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPublicationKey {
    universe: GraphUniverseId,
    profile: Box<str>,
    catalog_path: CatalogPath,
}

impl GraphPublicationKey {
    pub fn new(universe: GraphUniverseId, profile: impl Into<Box<str>>) -> GraphResult<Self> {
        let profile = profile.into();
        if profile.is_empty() || profile.len() > 256 || !component(&profile) {
            return Err(GraphError::new(
                GraphErrorCode::PublicationKeyInvalid,
                "invalid graph publication profile",
            ));
        }
        let catalog_path = CatalogPath::new(format!("{}/{profile}", universe.as_str()))?;
        Ok(Self {
            universe,
            profile,
            catalog_path,
        })
    }

    #[must_use]
    pub fn universe(&self) -> &GraphUniverseId {
        &self.universe
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn catalog_path(&self) -> &CatalogPath {
        &self.catalog_path
    }
}
