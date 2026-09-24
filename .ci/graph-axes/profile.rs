use serde::{Deserialize, Serialize};
use crate::{GraphDirection, GraphErrorCode, GraphRegistryBundle, GraphRelationKind, GraphResult};
use super::{digest, error};

pub const GRAPH_AXIS_PROFILE_SCHEMA: &str = "wow-graph/axis-profile/e2-a/1";

/// Repository-owned query meanings, not executable or source-defined predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphAxis {
    Lexical,
    Ownership,
    Load,
    Object,
    Inheritance,
    Registration,
    Lifecycle,
    State,
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphAxisShape {
    DirectedNetwork,
    /// Parents/children are a many-valued presentation. A tree, unique parent or
    /// acyclic runtime structure is not inferred from these source relations.
    MultiParent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAxisRelation {
    relation_id: Box<str>,
    relation: GraphRelationKind,
    forward_direction: GraphDirection,
}
impl GraphAxisRelation {
    #[must_use]
    pub fn relation_id(&self) -> &str { &self.relation_id }
    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind { self.relation }
    #[must_use]
    pub const fn forward_direction(&self) -> GraphDirection { self.forward_direction }
}

/// Immutable sidecar query profile tied to one exact graph registry. It does not
/// change that registry's existing bytes or any stored assertion/snapshot ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAxisProfile {
    schema: Box<str>,
    axis: GraphAxis,
    registry_digest: Box<str>,
    shape: GraphAxisShape,
    cycle_policy: Box<str>,
    ordering: Box<str>,
    relations: Vec<GraphAxisRelation>,
    digest: Box<str>,
}
impl GraphAxisProfile {
    /// Bind every required family to its exact registered definition. A missing
    /// family is unsupported, not an empty/complete axis. Multiple definition IDs
    /// mapping to one stored enum cannot be distinguished by materialized edges;
    /// reject instead of arbitrarily choosing or conflating their meanings.
    pub fn bind(registry: &GraphRegistryBundle, axis: GraphAxis) -> GraphResult<Self> {
        registry.validate()?;
        let (shape, families) = families(axis)?;
        let mut relations = Vec::new();
        for (relation, direction) in families {
            let mut matches = registry.relation_kinds().iter().filter(|d| d.relation() == relation);
            let definition = matches.next().ok_or_else(|| error(GraphErrorCode::AxisUnsupported))?;
            if matches.next().is_some() { return Err(error(GraphErrorCode::AxisProfileInvalid)); }
            relations.push(GraphAxisRelation {
                relation_id: definition.relation_id().into(),
                relation,
                forward_direction: direction,
            });
        }
        relations.sort_by_key(|s| s.relation);
        let cycle_policy: Box<str> = "preserve_edges_with_visited_nodes".into();
        let ordering: Box<str> = "multi_root_bfs_node_then_edge_id/1".into();
        let digest = digest("graph-axis-profile:sha256:", &(
            GRAPH_AXIS_PROFILE_SCHEMA, axis, registry.registry_digest(), shape,
            &cycle_policy, &ordering, &relations,
        ))?;
        Ok(Self {
            schema: GRAPH_AXIS_PROFILE_SCHEMA.into(), axis,
            registry_digest: registry.registry_digest().into(), shape,
            cycle_policy, ordering, relations, digest,
        })
    }

    /// Rebuild from the reviewed recipe, not merely from a self-consistent hash
    /// on deserialized fields. A recomputed digest cannot redefine an axis.
    pub fn validate(&self, registry: &GraphRegistryBundle) -> GraphResult<()> {
        if self.registry_digest.as_ref() != registry.registry_digest() {
            return Err(error(GraphErrorCode::RegistryIdentityMismatch));
        }
        if *self != Self::bind(registry, self.axis)? {
            return Err(error(GraphErrorCode::AxisProfileIdentityMismatch));
        }
        Ok(())
    }
    #[must_use]
    pub const fn axis(&self) -> GraphAxis { self.axis }
    #[must_use]
    pub fn registry_digest(&self) -> &str { &self.registry_digest }
    #[must_use]
    pub fn digest(&self) -> &str { &self.digest }
    #[must_use]
    pub const fn shape(&self) -> GraphAxisShape { self.shape }
    #[must_use]
    pub fn relations(&self) -> &[GraphAxisRelation] { &self.relations }
}

/// Forward hierarchies run from owner/base to owned/derived. Load forward runs
/// from loader/prerequisite to consumer; it is not an executable load schedule.
/// Network axes retain stored directions and never acquire "parent" semantics.
fn families(axis: GraphAxis) -> GraphResult<(GraphAxisShape, Vec<(GraphRelationKind, GraphDirection)>)> {
    use GraphDirection::{Incoming as Reverse, Outgoing as Forward};
    use GraphRelationKind::*;
    let shape = match axis {
        GraphAxis::Ownership | GraphAxis::Inheritance => GraphAxisShape::MultiParent,
        _ => GraphAxisShape::DirectedNetwork,
    };
    let families = match axis {
        // Neither lexical contains/declares nor object parent_of is representable
        // in the current closed GraphRelationKind schema. Do not alias owns or
        // factory_creates into those distinct semantics.
        GraphAxis::Lexical | GraphAxis::Object => return Err(error(GraphErrorCode::AxisUnsupported)),
        GraphAxis::Ownership => vec![(Owns, Forward)],
        GraphAxis::Load => vec![(Loads, Forward), (DependsOn, Reverse)],
        GraphAxis::Inheritance => vec![(Inherits, Reverse), (MixesIn, Reverse)],
        GraphAxis::Registration => vec![
            (RegistersNativeEvent, Forward), (HandlesNativeEvent, Forward),
            (BridgesNativeEvent, Forward), (EmitsCustomSignal, Forward),
            (HandlesCustomSignal, Forward), (RegistersCvarCallback, Forward),
            (SetsScript, Forward), (HooksScript, Forward), (SecureHooksFunction, Forward),
        ],
        GraphAxis::Lifecycle => vec![(FactoryCreates, Forward)],
        GraphAxis::State => vec![(ReadsState, Forward), (WritesState, Forward)],
        // UsesApi is not necessarily a call. Possible calls keep the original
        // Calls edge's confidence; no Candidate/possible-to-Proven conversion.
        GraphAxis::Call => vec![(Calls, Forward)],
    };
    Ok((shape, families))
}
