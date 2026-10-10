//! Registry19 native XML-to-virtual-Lua source associations.
use super::*;

pub(super) fn extend(
    entities: &mut Vec<GraphEntityKindDefinition>,
    relations: &mut Vec<GraphRelationKindDefinition>,
) -> ProjectResult<()> {
    for (kind, fields) in [
        (
            "xml_source_virtual_lua_unit",
            vec![
                "scope",
                "document",
                "document_digest",
                "occurrence",
                "unit_id",
                "extracted_unit_id",
                "virtual_path",
                "content_digest",
                "byte_length",
                "analysis_id",
                "semantic_state",
                "semantic_context",
                "mapped_observations_digest",
            ],
        ),
        (
            "xml_source_virtual_lua_map_piece",
            vec![
                "unit_id",
                "ordinal",
                "virtual_byte_start",
                "virtual_byte_end",
                "mapping_kind",
                "xml_span",
                "source_handle",
            ],
        ),
    ] {
        entities.push(
            GraphEntityKindDefinition::new(
                kind,
                vec!["blizzard_ui_source".into()],
                fields.into_iter().map(Into::into).collect(),
                vec![GraphConfidence::Proven],
            )
            .map_err(|_| invalid())?,
        );
    }
    for (id, kind, source, target) in [
        (
            "xml_script_site_owns_virtual_lua",
            GraphRelationKind::Owns,
            "xml_source_script_site",
            "xml_source_virtual_lua_unit",
        ),
        (
            "xml_virtual_lua_contains_map_piece",
            GraphRelationKind::Contains,
            "xml_source_virtual_lua_unit",
            "xml_source_virtual_lua_map_piece",
        ),
        (
            "xml_map_piece_source_span",
            GraphRelationKind::Owns,
            "xml_source_virtual_lua_map_piece",
            "source_span",
        ),
    ] {
        relations.push(
            GraphRelationKindDefinition::new(
                id,
                kind,
                vec![source.into()],
                vec![target.into()],
                vec![GraphConfidence::Proven],
            )
            .map_err(|_| invalid())?,
        );
    }
    Ok(())
}
