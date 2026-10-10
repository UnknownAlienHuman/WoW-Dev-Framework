//! Registry entries for exact selected-TOC structure. These definitions grant
//! no client loading, initialization or persistent-state authority.
use super::*;

pub(super) fn extend(
    universe_class: &str,
    entities: &mut Vec<GraphEntityKindDefinition>,
    relations: &mut Vec<GraphRelationKindDefinition>,
) -> ProjectResult<()> {
    for (kind, fields) in [
        ("addon_package", vec!["package"]),
        ("toc_manifest", vec!["document"]),
        ("toc_variant", vec!["document", "flavor"]),
        ("toc_load_policy", vec!["document", "state"]),
    ] {
        entities.push(
            GraphEntityKindDefinition::new(
                kind,
                vec![universe_class.into()],
                fields.into_iter().map(Into::into).collect(),
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )
            .map_err(|_| invalid())?,
        );
    }
    for (id, kind, sources, targets) in [
        (
            "toc_contains_manifest",
            GraphRelationKind::Contains,
            vec!["addon_package"],
            vec!["toc_manifest"],
        ),
        (
            "toc_defines_variant",
            GraphRelationKind::Defines,
            vec!["toc_manifest"],
            vec!["toc_variant"],
        ),
        (
            "toc_loads",
            GraphRelationKind::Loads,
            vec!["toc_variant"],
            vec!["source_file"],
        ),
        (
            "toc_loads_before",
            GraphRelationKind::LoadsBefore,
            vec!["source_file"],
            vec!["source_file"],
        ),
        (
            "toc_depends_on",
            GraphRelationKind::DependsOn,
            vec!["addon_package"],
            vec!["addon_package"],
        ),
        (
            "toc_optional_depends_on",
            GraphRelationKind::OptionalDependsOn,
            vec!["addon_package"],
            vec!["addon_package"],
        ),
        (
            "toc_owns_state",
            GraphRelationKind::Owns,
            vec!["addon_package", "toc_variant"],
            vec!["state_root"],
        ),
        (
            "toc_defines_load_policy",
            GraphRelationKind::Defines,
            vec!["toc_variant"],
            vec!["toc_load_policy"],
        ),
    ] {
        relations.push(
            GraphRelationKindDefinition::new(
                id,
                kind,
                sources.into_iter().map(Into::into).collect(),
                targets.into_iter().map(Into::into).collect(),
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )
            .map_err(|_| invalid())?,
        );
    }
    Ok(())
}
