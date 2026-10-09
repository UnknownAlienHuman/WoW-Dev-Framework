//! Exact static XML declarations and references; no runtime object authority.
use super::*;

pub(super) fn extend(
    entities: &mut Vec<GraphEntityKindDefinition>,
    relations: &mut Vec<GraphRelationKindDefinition>,
) -> ProjectResult<()> {
    for kind in ["xml_template", "xml_object", "xml_script_site"] {
        entities.push(
            GraphEntityKindDefinition::new(
                kind,
                vec!["project".into()],
                vec!["document".into(), "occurrence".into()],
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )
            .map_err(|_| invalid())?,
        );
    }
    for (id, kind, sources, targets) in [
        (
            "xml_defines_template",
            GraphRelationKind::Defines,
            vec!["source_file"],
            vec!["xml_template"],
        ),
        (
            "xml_defines_object",
            GraphRelationKind::Defines,
            vec!["source_file"],
            vec!["xml_object"],
        ),
        (
            "xml_owns_template",
            GraphRelationKind::Owns,
            vec!["addon_package", "toc_variant"],
            vec!["xml_template"],
        ),
        (
            "xml_owns_object",
            GraphRelationKind::Owns,
            vec!["addon_package", "toc_variant"],
            vec!["xml_object"],
        ),
        (
            "xml_parent_of",
            GraphRelationKind::ParentOf,
            vec!["xml_object"],
            vec!["xml_object"],
        ),
        (
            "xml_inherits",
            GraphRelationKind::Inherits,
            vec!["xml_template", "xml_object"],
            vec!["xml_template"],
        ),
        (
            "xml_references_template",
            GraphRelationKind::ReferencesTemplate,
            vec!["xml_template", "xml_object"],
            vec!["xml_template"],
        ),
        (
            "xml_owns_script",
            GraphRelationKind::Owns,
            vec!["xml_template", "xml_object"],
            vec!["xml_script_site"],
        ),
        (
            "xml_script_handler",
            GraphRelationKind::Defines,
            vec!["xml_script_site"],
            vec!["lua_source_function", "xml_source_handler"],
        ),
        (
            "xml_sets_script",
            GraphRelationKind::SetsScript,
            vec!["xml_template", "xml_object"],
            vec!["lua_source_function", "xml_source_handler"],
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
