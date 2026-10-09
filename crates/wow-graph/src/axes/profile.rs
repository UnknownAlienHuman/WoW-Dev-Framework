use super::{digest, error};
use crate::{GraphDirection, GraphErrorCode, GraphRegistryBundle, GraphRelationKind, GraphResult};
use serde::{Deserialize, Serialize};

pub const GRAPH_AXIS_PROFILE_SCHEMA: &str = "wow-graph/axis-profile/e2-a/1";

/// Explicit second Load recipe, selected only by registry content. It never
/// changes any v1 profile, digest or stored assertion identity.
pub const GRAPH_AXIS_PROFILE_SCHEMA_V2: &str = "wow-graph/axis-profile/e2-a/2";

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
    pub fn relation_id(&self) -> &str {
        &self.relation_id
    }
    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }
    #[must_use]
    pub const fn forward_direction(&self) -> GraphDirection {
        self.forward_direction
    }
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
        let (shape, families) = families(axis, registry)?;
        let mut relations = Vec::new();
        for (relation, direction) in families {
            let mut matches = registry
                .relation_kinds()
                .iter()
                .filter(|d| d.relation() == relation);
            let definition = matches
                .next()
                .ok_or_else(|| error(GraphErrorCode::AxisUnsupported))?;
            if matches.next().is_some() {
                return Err(error(GraphErrorCode::AxisProfileInvalid));
            }
            relations.push(GraphAxisRelation {
                relation_id: definition.relation_id().into(),
                relation,
                forward_direction: direction,
            });
        }
        // The recipe follows the registry alone. Binding and reconstruction
        // therefore derive the same schema and families, and neither a request
        // nor a deserialized profile can choose a different Load review.
        relations.sort_by_key(|s| s.relation);
        let cycle_policy: Box<str> = "preserve_edges_with_visited_nodes".into();
        let ordering: Box<str> = "multi_root_bfs_node_then_edge_id/1".into();
        let schema = if axis == GraphAxis::Load && matches!(load_recipe(registry), LoadRecipe::V2) {
            GRAPH_AXIS_PROFILE_SCHEMA_V2
        } else {
            GRAPH_AXIS_PROFILE_SCHEMA
        };
        let digest = digest(
            "graph-axis-profile:sha256:",
            &(
                schema,
                axis,
                registry.registry_digest(),
                shape,
                &cycle_policy,
                &ordering,
                &relations,
            ),
        )?;
        Ok(Self {
            schema: schema.into(),
            axis,
            registry_digest: registry.registry_digest().into(),
            shape,
            cycle_policy,
            ordering,
            relations,
            digest,
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
    pub const fn axis(&self) -> GraphAxis {
        self.axis
    }
    #[must_use]
    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    #[must_use]
    pub const fn shape(&self) -> GraphAxisShape {
        self.shape
    }
    #[must_use]
    pub fn relations(&self) -> &[GraphAxisRelation] {
        &self.relations
    }
}

/// Forward hierarchies run from owner/base to owned/derived. Load forward runs
/// from loader/prerequisite to consumer; it is not an executable load schedule.
/// Network axes retain stored directions and never acquire "parent" semantics.
fn families(
    axis: GraphAxis,
    registry: &GraphRegistryBundle,
) -> GraphResult<(GraphAxisShape, Vec<(GraphRelationKind, GraphDirection)>)> {
    use GraphDirection::{Incoming as Reverse, Outgoing as Forward};
    use GraphRelationKind::*;
    // The Load family set is derived from the exact registry, so binding and
    // validation always agree and no caller can select a recipe.
    let shape = match axis {
        GraphAxis::Ownership | GraphAxis::Inheritance => GraphAxisShape::MultiParent,
        _ => GraphAxisShape::DirectedNetwork,
    };
    let families = match axis {
        // Neither lexical contains/declares nor object parent_of is representable
        // in the current closed GraphRelationKind schema. Do not alias owns or
        // factory_creates into those distinct semantics.
        GraphAxis::Lexical | GraphAxis::Object => {
            return Err(error(GraphErrorCode::AxisUnsupported));
        }
        GraphAxis::Ownership => vec![(Owns, Forward)],
        GraphAxis::Load => match load_recipe(registry) {
            LoadRecipe::V1 => vec![(Loads, Forward), (DependsOn, Reverse)],
            LoadRecipe::V2 => vec![
                (Loads, Forward),
                (DependsOn, Reverse),
                (LoadsBefore, Forward),
                (OptionalDependsOn, Reverse),
            ],
        },
        GraphAxis::Inheritance => vec![(Inherits, Reverse), (MixesIn, Reverse)],
        GraphAxis::Registration => vec![
            (RegistersNativeEvent, Forward),
            (HandlesNativeEvent, Forward),
            (BridgesNativeEvent, Forward),
            (EmitsCustomSignal, Forward),
            (HandlesCustomSignal, Forward),
            (RegistersCvarCallback, Forward),
            (SetsScript, Forward),
            (HooksScript, Forward),
            (SecureHooksFunction, Forward),
        ],
        GraphAxis::Lifecycle => vec![(FactoryCreates, Forward), (Instantiates, Forward)],
        GraphAxis::State => vec![(ReadsState, Forward), (WritesState, Forward)],
        // UsesApi is not necessarily a call. Possible calls keep the original
        // Calls edge's confidence; no Candidate/possible-to-Proven conversion.
        GraphAxis::Call => vec![(Calls, Forward)],
    };
    Ok((shape, families))
}

/// Which Load relation families one reviewed profile covers. Derived from the
/// exact registry, never from a request, a deserialized profile or a default.
enum LoadRecipe {
    /// The original family set. Its schema, families, directions, ordering and
    /// digest inputs stay byte-identical to the pre-extension profile.
    V1,
    /// Selected only when the registry admits `LoadsBefore` or `OptionalDependsOn`.
    /// It then requires all four Load kinds to be registered.
    V2,
}

fn load_recipe(registry: &GraphRegistryBundle) -> LoadRecipe {
    let admits_extended = registry.relation_kinds().iter().any(|definition| {
        matches!(
            definition.relation(),
            GraphRelationKind::LoadsBefore | GraphRelationKind::OptionalDependsOn
        )
    });
    if admits_extended {
        LoadRecipe::V2
    } else {
        LoadRecipe::V1
    }
}
