use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{GraphConfidence, GraphError, GraphErrorCode, GraphRelationKind, GraphResult};

pub const GRAPH_REGISTRY_SCHEMA: &str = "wow-graph/registry/e2-a/1";
const MAX_DEFINITIONS: usize = 512;
const MAX_FIELDS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEntityKindDefinition {
    kind_id: Box<str>,
    allowed_universe_classes: Vec<Box<str>>,
    identity_fields: Vec<Box<str>>,
    allowed_confidences: Vec<GraphConfidence>,
}

impl GraphEntityKindDefinition {
    pub fn new(
        kind_id: impl Into<Box<str>>,
        mut allowed_universe_classes: Vec<Box<str>>,
        mut identity_fields: Vec<Box<str>>,
        mut allowed_confidences: Vec<GraphConfidence>,
    ) -> GraphResult<Self> {
        let kind_id = kind_id.into();
        validate_component(&kind_id, "entity kind id")?;
        normalize_components(&mut allowed_universe_classes, "universe class")?;
        normalize_components(&mut identity_fields, "identity field")?;
        allowed_confidences.sort();
        if allowed_universe_classes.is_empty()
            || identity_fields.is_empty()
            || identity_fields.len() > MAX_FIELDS
            || allowed_confidences.is_empty()
            || allowed_confidences
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph entity kind definition is empty, duplicated, or too large",
            ));
        }
        Ok(Self {
            kind_id,
            allowed_universe_classes,
            identity_fields,
            allowed_confidences,
        })
    }

    pub fn validate(&self) -> GraphResult<()> {
        if Self::new(
            self.kind_id.clone(),
            self.allowed_universe_classes.clone(),
            self.identity_fields.clone(),
            self.allowed_confidences.clone(),
        )? != *self
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph entity kind definition is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn kind_id(&self) -> &str {
        &self.kind_id
    }

    #[must_use]
    pub fn allowed_universe_classes(&self) -> &[Box<str>] {
        &self.allowed_universe_classes
    }

    #[must_use]
    pub fn identity_fields(&self) -> &[Box<str>] {
        &self.identity_fields
    }

    #[must_use]
    pub fn allows_confidence(&self, confidence: GraphConfidence) -> bool {
        self.allowed_confidences.binary_search(&confidence).is_ok()
    }

    #[must_use]
    pub fn allows_universe(&self, universe: &crate::GraphUniverseId) -> bool {
        let class = universe
            .as_str()
            .split_once(':')
            .map_or(universe.as_str(), |(class, _)| class);
        self.allowed_universe_classes
            .binary_search_by(|value| value.as_ref().cmp(class))
            .is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRelationKindDefinition {
    relation_id: Box<str>,
    relation: GraphRelationKind,
    source_kinds: Vec<Box<str>>,
    target_kinds: Vec<Box<str>>,
    allowed_confidences: Vec<GraphConfidence>,
}

impl GraphRelationKindDefinition {
    pub fn new(
        relation_id: impl Into<Box<str>>,
        relation: GraphRelationKind,
        mut source_kinds: Vec<Box<str>>,
        mut target_kinds: Vec<Box<str>>,
        mut allowed_confidences: Vec<GraphConfidence>,
    ) -> GraphResult<Self> {
        let relation_id = relation_id.into();
        validate_component(&relation_id, "relation kind id")?;
        normalize_components(&mut source_kinds, "source kind")?;
        normalize_components(&mut target_kinds, "target kind")?;
        allowed_confidences.sort();
        if source_kinds.is_empty()
            || target_kinds.is_empty()
            || source_kinds.len() > MAX_FIELDS
            || target_kinds.len() > MAX_FIELDS
            || allowed_confidences.is_empty()
            || allowed_confidences
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph relation definition is empty, duplicated, or too large",
            ));
        }
        Ok(Self {
            relation_id,
            relation,
            source_kinds,
            target_kinds,
            allowed_confidences,
        })
    }

    pub fn validate(&self) -> GraphResult<()> {
        if Self::new(
            self.relation_id.clone(),
            self.relation,
            self.source_kinds.clone(),
            self.target_kinds.clone(),
            self.allowed_confidences.clone(),
        )? != *self
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph relation definition is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn relation_id(&self) -> &str {
        &self.relation_id
    }

    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }

    #[must_use]
    pub fn allows_source_kind(&self, kind: &str) -> bool {
        self.source_kinds
            .binary_search_by(|value| value.as_ref().cmp(kind))
            .is_ok()
    }

    #[must_use]
    pub fn allows_target_kind(&self, kind: &str) -> bool {
        self.target_kinds
            .binary_search_by(|value| value.as_ref().cmp(kind))
            .is_ok()
    }

    #[must_use]
    pub fn allows_confidence(&self, confidence: GraphConfidence) -> bool {
        self.allowed_confidences.binary_search(&confidence).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRegistryBundle {
    schema: Box<str>,
    bundle_id: Box<str>,
    version: Box<str>,
    registry_digest: Box<str>,
    entity_kinds: Vec<GraphEntityKindDefinition>,
    relation_kinds: Vec<GraphRelationKindDefinition>,
}

impl GraphRegistryBundle {
    pub fn build(
        bundle_id: impl Into<Box<str>>,
        version: impl Into<Box<str>>,
        mut entity_kinds: Vec<GraphEntityKindDefinition>,
        mut relation_kinds: Vec<GraphRelationKindDefinition>,
    ) -> GraphResult<Self> {
        let bundle_id = bundle_id.into();
        let version = version.into();
        validate_component(&bundle_id, "registry bundle id")?;
        validate_component(&version, "registry version")?;
        if entity_kinds.is_empty()
            || entity_kinds.len() > MAX_DEFINITIONS
            || relation_kinds.len() > MAX_DEFINITIONS
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph registry definition count is invalid",
            ));
        }
        for definition in &entity_kinds {
            definition.validate()?;
        }
        entity_kinds.sort_by(|left, right| left.kind_id.cmp(&right.kind_id));
        if entity_kinds
            .windows(2)
            .any(|pair| pair[0].kind_id == pair[1].kind_id)
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph registry contains duplicate entity kinds",
            ));
        }
        let entity_ids = entity_kinds
            .iter()
            .map(|definition| definition.kind_id.as_ref())
            .collect::<BTreeSet<_>>();
        for definition in &relation_kinds {
            definition.validate()?;
            if definition
                .source_kinds
                .iter()
                .chain(&definition.target_kinds)
                .any(|kind| !entity_ids.contains(kind.as_ref()))
            {
                return Err(GraphError::new(
                    GraphErrorCode::RegistryInvalid,
                    "graph relation definition references an unknown entity kind",
                ));
            }
        }
        relation_kinds.sort_by(|left, right| left.relation_id.cmp(&right.relation_id));
        if relation_kinds
            .windows(2)
            .any(|pair| pair[0].relation_id == pair[1].relation_id)
        {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph registry contains duplicate relation kinds",
            ));
        }
        let registry_digest =
            derive_registry_digest(&bundle_id, &version, &entity_kinds, &relation_kinds)?;
        Ok(Self {
            schema: GRAPH_REGISTRY_SCHEMA.into(),
            bundle_id,
            version,
            registry_digest,
            entity_kinds,
            relation_kinds,
        })
    }

    pub fn validate(&self) -> GraphResult<()> {
        if self.schema.as_ref() != GRAPH_REGISTRY_SCHEMA {
            return Err(GraphError::new(
                GraphErrorCode::RegistryInvalid,
                "graph registry schema is unsupported",
            ));
        }
        let rebuilt = Self::build(
            self.bundle_id.clone(),
            self.version.clone(),
            self.entity_kinds.clone(),
            self.relation_kinds.clone(),
        )?;
        if rebuilt != *self {
            return Err(GraphError::new(
                GraphErrorCode::RegistryIdentityMismatch,
                "graph registry identity or canonical order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn bundle_id(&self) -> &str {
        &self.bundle_id
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[must_use]
    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }

    #[must_use]
    pub fn entity_kind(&self, kind_id: &str) -> Option<&GraphEntityKindDefinition> {
        self.entity_kinds
            .binary_search_by(|definition| definition.kind_id.as_ref().cmp(kind_id))
            .ok()
            .map(|index| &self.entity_kinds[index])
    }

    #[must_use]
    pub fn relation_kind(&self, relation_id: &str) -> Option<&GraphRelationKindDefinition> {
        self.relation_kinds
            .binary_search_by(|definition| definition.relation_id.as_ref().cmp(relation_id))
            .ok()
            .map(|index| &self.relation_kinds[index])
    }
}

fn derive_registry_digest(
    bundle_id: &str,
    version: &str,
    entity_kinds: &[GraphEntityKindDefinition],
    relation_kinds: &[GraphRelationKindDefinition],
) -> GraphResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        bundle_id: &'a str,
        version: &'a str,
        entity_kinds: &'a [GraphEntityKindDefinition],
        relation_kinds: &'a [GraphRelationKindDefinition],
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: GRAPH_REGISTRY_SCHEMA,
        bundle_id,
        version,
        entity_kinds,
        relation_kinds,
    })
    .map_err(|_| {
        GraphError::new(
            GraphErrorCode::RegistryIdentityMismatch,
            "graph registry identity cannot be canonicalized",
        )
    })?;
    Ok(format!("graph-registry:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

pub(crate) fn validate_component(value: &str, label: &str) -> GraphResult<()> {
    if value.is_empty()
        || value.len() > 512
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
    {
        return Err(GraphError::new(
            GraphErrorCode::RegistryInvalid,
            format!("invalid graph {label}"),
        ));
    }
    Ok(())
}

fn normalize_components(values: &mut [Box<str>], label: &str) -> GraphResult<()> {
    for value in values.iter() {
        validate_component(value, label)?;
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GraphError::new(
            GraphErrorCode::RegistryInvalid,
            format!("graph {label} values must be unique"),
        ));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
