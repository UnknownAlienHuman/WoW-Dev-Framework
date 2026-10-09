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
        Recipe::PackageNamed => vec![
            entity("package", "addon_package", &[("package", "package")]),
            entity("manifest", "toc_manifest", &[("document", "document")]),
            entity(
                "variant",
                "toc_variant",
                &[("document", "document"), ("flavor", "flavor")],
            ),
            relation("toc_contains_manifest", "package_ref", "manifest_ref"),
            relation("toc_defines_variant", "manifest_ref", "variant_ref"),
        ],
        Recipe::PackageIsolated => vec![
            entity("manifest", "toc_manifest", &[("document", "document")]),
            entity(
                "variant",
                "toc_variant",
                &[("document", "document"), ("flavor", "flavor")],
            ),
            relation("toc_defines_variant", "manifest_ref", "variant_ref"),
        ],
        Recipe::FileLoads => vec![relation("toc_loads", "source", "target")],
        Recipe::FileBefore => vec![relation("toc_loads_before", "source", "target")],
        Recipe::RequiredDependency => vec![relation("toc_depends_on", "source", "target")],
        Recipe::OptionalDependency => vec![relation("toc_optional_depends_on", "source", "target")],
        Recipe::LoadPolicy => vec![
            entity(
                "policy",
                "toc_load_policy",
                &[("document", "document"), ("state", "state")],
            ),
            relation("toc_defines_load_policy", "source", "policy_ref"),
        ],
        Recipe::SavedVariable => vec![
            entity(
                "root",
                "state_root",
                &[
                    ("document", "document"),
                    ("name", "name"),
                    ("scope", "scope"),
                ],
            ),
            relation("toc_owns_state", "source", "root_ref"),
        ],
    };
    let positive = match recipe {
        Recipe::PackageNamed | Recipe::PackageIsolated => "RECOG-TOC-001",
        Recipe::FileLoads | Recipe::FileBefore => "RECOG-TOC-002",
        Recipe::RequiredDependency => "RECOG-TOC-003",
        Recipe::OptionalDependency => "RECOG-TOC-004",
        Recipe::LoadPolicy => "RECOG-TOC-006",
        Recipe::SavedVariable => "RECOG-TOC-008",
    };
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: format!("wow-core-{}", recipe.name()).into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: FACT_PROFILE.into(),
            graph_registry_bundle_id: registry.into(),
            evaluation_profile_id: "wow-recognizers-toc-structural-1".into(),
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
