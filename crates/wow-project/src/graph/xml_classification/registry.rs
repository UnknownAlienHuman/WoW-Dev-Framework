//! Exact schema-class definition; source occurrences keep their existing identity.
use super::*;

pub(in crate::graph) const CLASS_KIND: &str = "xml_schema_class";
pub(in crate::graph) const CLASSIFICATION_RELATION: &str = "xml_occurrence_schema_class";

pub(in crate::graph) fn extend(
    entities: &mut Vec<GraphEntityKindDefinition>,
    relations: &mut Vec<GraphRelationKindDefinition>,
) -> ProjectResult<()> {
    entities.push(
        GraphEntityKindDefinition::new(
            CLASS_KIND,
            vec!["blizzard_ui_source".into()],
            vec![
                "catalog_digest".into(),
                "profile_digest".into(),
                "declaration".into(),
                "class".into(),
                "proof_digest".into(),
            ],
            vec![GraphConfidence::Derived],
        )
        .map_err(|_| invalid())?,
    );
    relations.push(
        GraphRelationKindDefinition::new(
            CLASSIFICATION_RELATION,
            GraphRelationKind::HasSchemaClass,
            vec!["xml_source_occurrence".into()],
            vec![CLASS_KIND.into()],
            vec![GraphConfidence::Derived],
        )
        .map_err(|_| invalid())?,
    );
    Ok(())
}
