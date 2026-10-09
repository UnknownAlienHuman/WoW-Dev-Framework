//! Focused Load-axis recipe compatibility.
//!
//! The Load relation family set is chosen from the exact registered relation
//! definitions. A registry that admits neither `loads_before` nor
//! `optional_depends_on` keeps the original two-family recipe, so its profile
//! schema, families, relation IDs and directions are unchanged. Admitting either
//! extended kind selects the four-family recipe, which then requires all four
//! kinds to be registered exactly once.

use std::error::Error;

use serde_json::Value;
use wow_graph::{
    GraphAxis, GraphAxisProfile, GraphConfidence, GraphEntityKindDefinition, GraphErrorCode,
    GraphRegistryBundle, GraphRelationKind, GraphRelationKindDefinition,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const V1_SCHEMA: &str = "wow-graph/axis-profile/e2-a/1";
const V2_SCHEMA: &str = "wow-graph/axis-profile/e2-a/2";

fn packages() -> TestResult<GraphEntityKindDefinition> {
    Ok(GraphEntityKindDefinition::new(
        "addon_package",
        vec!["project".into()],
        vec!["package".into()],
        vec![GraphConfidence::Proven, GraphConfidence::Possible],
    )?)
}

fn functions() -> TestResult<GraphEntityKindDefinition> {
    Ok(GraphEntityKindDefinition::new(
        "function",
        vec!["project".into()],
        vec!["symbol".into()],
        vec![GraphConfidence::Proven, GraphConfidence::Derived],
    )?)
}

fn relation(
    relation_id: &str,
    relation: GraphRelationKind,
) -> TestResult<GraphRelationKindDefinition> {
    let source = if matches!(relation, GraphRelationKind::Calls) {
        "function"
    } else {
        "addon_package"
    };
    Ok(GraphRelationKindDefinition::new(
        relation_id,
        relation,
        vec![source.into()],
        vec![source.into()],
        vec![GraphConfidence::Proven],
    )?)
}

/// One registry that also carries a Call family, so a non-Load axis can be bound
/// on the very same registry and compared against the Load recipe choice.
fn registry(extended: bool) -> TestResult<GraphRegistryBundle> {
    let mut relations = vec![
        relation("source_calls", GraphRelationKind::Calls)?,
        relation("source_loads", GraphRelationKind::Loads)?,
        relation("source_depends_on", GraphRelationKind::DependsOn)?,
    ];
    if extended {
        relations.push(relation(
            "source_loads_before",
            GraphRelationKind::LoadsBefore,
        )?);
        relations.push(relation(
            "source_optional_depends_on",
            GraphRelationKind::OptionalDependsOn,
        )?);
    }
    Ok(GraphRegistryBundle::build(
        if extended {
            "graph-registry:load-axis:extended"
        } else {
            "graph-registry:load-axis:original"
        },
        "1.0.0",
        vec![packages()?, functions()?],
        relations,
    )?)
}

fn kinds(profile: &GraphAxisProfile) -> Vec<GraphRelationKind> {
    profile
        .relations()
        .iter()
        .map(|entry| entry.relation())
        .collect()
}

fn schema_of(profile: &GraphAxisProfile) -> TestResult<String> {
    let encoded = serde_json::to_value(profile)?;
    Ok(encoded
        .get("schema")
        .and_then(Value::as_str)
        .ok_or("serialized axis profile has no schema field")?
        .to_owned())
}

/// A registry admitting no extended Load kind keeps the original schema,
/// families, relation IDs
/// and the same directions as before the new relation enums existed.
#[test]
fn original_registry_keeps_the_two_family_load_recipe() -> TestResult {
    let registry = registry(false)?;
    let profile = GraphAxisProfile::bind(&registry, GraphAxis::Load)?;
    assert_eq!(schema_of(&profile)?, V1_SCHEMA);
    // Canonical order sorts by the stored relation enum, not by registry or
    // declaration order. DependsOn precedes Loads there.
    assert_eq!(
        kinds(&profile),
        vec![GraphRelationKind::DependsOn, GraphRelationKind::Loads],
    );
    assert!(
        !kinds(&profile).iter().any(|kind| matches!(
            kind,
            GraphRelationKind::LoadsBefore | GraphRelationKind::OptionalDependsOn
        )),
        "an extended family cannot leak into the original recipe",
    );
    assert_eq!(
        profile
            .relations()
            .iter()
            .map(|entry| entry.forward_direction())
            .collect::<Vec<_>>(),
        vec![
            wow_graph::GraphDirection::Incoming,
            wow_graph::GraphDirection::Outgoing,
        ],
    );
    // Reconstruction and repeated binding reproduce the exact same profile.
    profile.validate(&registry)?;
    assert_eq!(GraphAxisProfile::bind(&registry, GraphAxis::Load)?, profile);
    Ok(())
}

/// Admitting an extended Load kind selects the four-family recipe, recorded by
/// the explicit second schema version rather than by a rewritten v1 profile.
#[test]
fn extended_registry_binds_all_four_load_families() -> TestResult {
    let registry = registry(true)?;
    let profile = GraphAxisProfile::bind(&registry, GraphAxis::Load)?;
    assert_eq!(schema_of(&profile)?, V2_SCHEMA);
    assert_eq!(
        profile
            .relations()
            .iter()
            .map(|entry| entry.relation_id())
            .collect::<Vec<_>>(),
        vec![
            "source_depends_on",
            "source_optional_depends_on",
            "source_loads",
            "source_loads_before",
        ],
    );
    assert_eq!(
        kinds(&profile),
        vec![
            GraphRelationKind::DependsOn,
            GraphRelationKind::OptionalDependsOn,
            GraphRelationKind::Loads,
            GraphRelationKind::LoadsBefore,
        ],
    );
    assert_eq!(
        profile
            .relations()
            .iter()
            .map(|entry| entry.forward_direction())
            .collect::<Vec<_>>(),
        vec![
            wow_graph::GraphDirection::Incoming,
            wow_graph::GraphDirection::Incoming,
            wow_graph::GraphDirection::Outgoing,
            wow_graph::GraphDirection::Outgoing,
        ],
    );
    profile.validate(&registry)?;
    assert_eq!(GraphAxisProfile::bind(&registry, GraphAxis::Load)?, profile);
    Ok(())
}

/// Admitting either extended kind alone selects the four-family recipe. A family
/// that is then missing from the registry is unsupported, never a silently
/// narrowed traversal whose empty result would read as a clean negative.
#[test]
fn extended_registry_without_a_family_is_unsupported() -> TestResult {
    let registry = GraphRegistryBundle::build(
        "graph-registry:load-axis:partial",
        "1.0.0",
        vec![packages()?, functions()?],
        vec![
            relation("source_calls", GraphRelationKind::Calls)?,
            relation("source_loads", GraphRelationKind::Loads)?,
            relation("source_depends_on", GraphRelationKind::DependsOn)?,
            relation("source_loads_before", GraphRelationKind::LoadsBefore)?,
        ],
    )?;
    let error = GraphAxisProfile::bind(&registry, GraphAxis::Load)
        .err()
        .ok_or("expected a bounding error for the missing Load family")?;
    assert_eq!(error.code(), GraphErrorCode::AxisUnsupported);
    Ok(())
}

/// Two definitions sharing one stored relation enum cannot be distinguished by
/// materialized edges, so the recipe rejects the ambiguity instead of choosing.
#[test]
fn ambiguous_registered_family_is_rejected() -> TestResult {
    let registry = GraphRegistryBundle::build(
        "graph-registry:load-axis:ambiguous",
        "1.0.0",
        vec![packages()?, functions()?],
        vec![
            relation("source_calls", GraphRelationKind::Calls)?,
            relation("source_loads", GraphRelationKind::Loads)?,
            relation("source_package_loads", GraphRelationKind::Loads)?,
            relation("source_depends_on", GraphRelationKind::DependsOn)?,
            relation(
                "source_optional_depends_on",
                GraphRelationKind::OptionalDependsOn,
            )?,
        ],
    )?;
    let error = GraphAxisProfile::bind(&registry, GraphAxis::Load)
        .err()
        .ok_or("expected a rejection for the ambiguous registered family")?;
    assert_eq!(error.code(), GraphErrorCode::AxisProfileInvalid);
    Ok(())
}

/// The second Load recipe applies to the Load axis only. The same registry keeps
/// every other axis on its original recipe and schema.
#[test]
fn other_axes_keep_their_original_recipe() -> TestResult {
    let registry = registry(true)?;
    let profile = GraphAxisProfile::bind(&registry, GraphAxis::Call)?;
    assert_eq!(schema_of(&profile)?, V1_SCHEMA);
    assert_eq!(kinds(&profile), vec![GraphRelationKind::Calls]);
    profile.validate(&registry)?;
    Ok(())
}
