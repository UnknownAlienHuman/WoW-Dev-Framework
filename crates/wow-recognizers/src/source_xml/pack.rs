//! Data-only shadow packs. Missing acceptance fixture categories stay empty;
//! default rollout continues to require the complete fixture declarations.
use super::*;
use crate::{
    CompiledRecognizerPack, RecognizerClause, RecognizerOutput, RecognizerOutputConfidence,
    RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackLiteral,
    RecognizerPackRollout, RecognizerPackTrustClass, RecognizerRule, parse_recognizer_pack,
};

pub(super) fn compile(recipe: Recipe, registry: &str) -> RecognizerResult<CompiledRecognizerPack> {
    let outputs = match recipe {
        Recipe::TemplateDeclared => vec![
            entity(
                "template",
                "xml_template",
                &[("document", "document"), ("occurrence", "occurrence")],
            ),
            relation("xml_defines_template", "file", "entity"),
            relation("xml_owns_template", "owner", "entity"),
        ],
        Recipe::ObjectDeclared => vec![
            entity(
                "object",
                "xml_object",
                &[("document", "document"), ("occurrence", "occurrence")],
            ),
            relation("xml_defines_object", "file", "entity"),
            relation("xml_owns_object", "owner", "entity"),
        ],
        Recipe::ParentOf => vec![relation("xml_parent_of", "source", "target")],
        Recipe::InheritsTemplate => vec![relation("xml_inherits", "source", "target")],
        Recipe::ReferencesTemplate => vec![relation("xml_references_template", "source", "target")],
        // The site entity is declared in both recipes so one binding match can
        // reference the same-match site produced by a ScriptSite match.
        Recipe::ScriptSite => vec![
            entity(
                "site",
                "xml_script_site",
                &[("document", "document"), ("occurrence", "occurrence")],
            ),
            relation("xml_owns_script", "owner", "site"),
        ],
        Recipe::ScriptBinding => vec![
            entity(
                "site",
                "xml_script_site",
                &[("document", "document"), ("occurrence", "occurrence")],
            ),
            relation("xml_owns_script", "owner", "site"),
            relation("xml_script_handler", "site", "handler"),
            relation("xml_sets_script", "owner", "handler"),
        ],
    };
    let positive = match recipe {
        Recipe::TemplateDeclared => "RECOG-XML-001",
        Recipe::ObjectDeclared => "RECOG-XML-002",
        Recipe::ParentOf => "RECOG-XML-003",
        Recipe::InheritsTemplate | Recipe::ReferencesTemplate => "RECOG-XML-004",
        Recipe::ScriptSite | Recipe::ScriptBinding => "RECOG-XML-006",
    };
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: format!("wow-core-{}", recipe.name()).into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: FACT_PROFILE.into(),
            graph_registry_bundle_id: registry.into(),
            evaluation_profile_id: "wow-recognizers-xml-structural-1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 1,
                max_clauses_per_rule: 4,
                max_clause_depth: 2,
                max_join_expansions_per_rule: 65_536,
                max_matches_per_rule_partition: MAX_FACTS as u32,
                max_proposals_per_rule_partition: 163_840,
                max_explanation_bytes: 8 * 1024 * 1024,
            },
            rules: vec![RecognizerRule {
                rule_id: recipe.family().rule_id().into(),
                version: 1,
                required_capabilities: vec![recipe.family().capability_id().into()],
                scope: "package".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "f".into(),
                        kind: recipe.name().into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "f.admitted".into(),
                        value: RecognizerPackLiteral::Boolean(true),
                    },
                ],
                captures: Vec::new(),
                outputs,
                positive_fixture_ids: vec![positive.into()],
                near_negative_fixture_ids: Vec::new(),
                partial_fixture_ids: Vec::new(),
                mutation_fixture_ids: Vec::new(),
            }],
        },
    };
    let bytes = wow_core::canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

fn entity(id: &str, kind: &str, fields: &[(&str, &str)]) -> RecognizerOutput {
    RecognizerOutput::EntityAssertion {
        output_id: id.into(),
        entity_kind_id: kind.into(),
        semantic_key: fields
            .iter()
            .map(|(key, field)| ((*key).into(), format!("f.{field}").into()))
            .collect(),
        confidence: RecognizerOutputConfidence::Derived,
    }
}
fn relation(kind: &str, source: &str, target: &str) -> RecognizerOutput {
    RecognizerOutput::RelationAssertion {
        output_id: kind.into(),
        relation_kind_id: kind.into(),
        source: format!("f.{source}").into(),
        target: format!("f.{target}").into(),
        confidence: RecognizerOutputConfidence::Derived,
    }
}
