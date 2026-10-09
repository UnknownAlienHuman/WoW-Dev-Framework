//! One small declarative pack per core.state literal-path rule.
//!
//! Fact construction and support validation stay in the parent adapter; this
//! file only declares which normalized fields a rule requires and which graph
//! assertion it may propose. The matcher, not this module, decides Derived or
//! Possible, so a Partial coverage record downgrades every match consistently.
use super::*;
use crate::{
    CompiledRecognizerPack, RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerClause, RecognizerError,
    RecognizerErrorCode, RecognizerOutput, RecognizerOutputConfidence, RecognizerPack,
    RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackLiteral, RecognizerPackRollout,
    RecognizerPackTrustClass, RecognizerResult, RecognizerRule, parse_recognizer_pack,
};
use std::collections::BTreeMap;

/// Fact kind and field names are contracted with the parent adapter and its
/// projector; they are restated here only where the pack must name them.
const FACT_KIND: &str = "state_access";
const EVALUATION_PROFILE: &str = "wow-recognizers-state-structural-1";
/// Entity role token of the emitted path, matching the projector's endpoint
/// protocol. A source endpoint is a plain retained node reference instead.
const PATH_ROLE: &str = "path";

/// Compiles the read or write rule for one exact target shape.
///
/// `has_path` selects the route through an emitted `state_path` entity; the
/// root route proposes only the relation against a retained root node.
/// `registry` binds the pack to one exact registry bundle identity.
pub(super) fn compile(
    family: SourceStateCoreFamily,
    has_path: bool,
    registry: &str,
) -> RecognizerResult<CompiledRecognizerPack> {
    let relation_id = family.definition_id();
    let rule_id = family.rule_id();
    let capability_id = family.capability_id();
    let family_name = family.name();
    // RECOG-STATE-002 is a documentation-only example row, not a frozen or
    // verified fixture: the examples set still sits behind its freeze gate.
    // The ID stays for declarative continuity, while the write chain has no case
    // at all and keeps every category empty rather than borrowing an unrelated one.
    let positive: Vec<Box<str>> = match family {
        SourceStateCoreFamily::Read if has_path => vec!["RECOG-STATE-002".into()],
        _ => Vec::new(),
    };
    let mut outputs = Vec::new();
    if has_path {
        outputs.push(RecognizerOutput::EntityAssertion {
            output_id: PATH_ROLE.into(),
            entity_kind_id: "state_path".into(),
            // Typed references, not names: the identifier root and the literal
            // document path keep their fact value types through projection.
            semantic_key: BTreeMap::from([
                ("root".into(), "f.root".into()),
                ("path".into(), "f.path".into()),
            ]),
            confidence: RecognizerOutputConfidence::Derived,
        });
    }
    outputs.push(RecognizerOutput::RelationAssertion {
        output_id: relation_id.into(),
        relation_kind_id: relation_id.into(),
        source: "f.source".into(),
        target: "f.target".into(),
        confidence: RecognizerOutputConfidence::Derived,
    });
    let document = RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: format!(
                "wow-core-state-{}-{}",
                family_name,
                if has_path { "path" } else { "root" }
            )
            .into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: FACT_PROFILE.into(),
            graph_registry_bundle_id: registry.into(),
            evaluation_profile_id: EVALUATION_PROFILE.into(),
            // Shadow keeps this executable without any acceptance claim; no
            // fixture category is borrowed to justify a default rollout.
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 1,
                max_clauses_per_rule: 4,
                max_clause_depth: 2,
                max_join_expansions_per_rule: 65_536,
                max_matches_per_rule_partition: MAX_BINDINGS as u32,
                max_proposals_per_rule_partition: 16_384,
                max_explanation_bytes: 8 * 1024 * 1024,
            },
            rules: vec![RecognizerRule {
                rule_id: rule_id.into(),
                version: 1,
                required_capabilities: vec![capability_id.into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "f".into(),
                        kind: FACT_KIND.into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "f.has_path".into(),
                        value: RecognizerPackLiteral::Boolean(has_path),
                    },
                ],
                captures: Vec::new(),
                outputs,
                positive_fixture_ids: positive,
                near_negative_fixture_ids: Vec::new(),
                partial_fixture_ids: Vec::new(),
                mutation_fixture_ids: Vec::new(),
            }],
        },
    };
    let bytes = wow_core::canonical_json_bytes(&document).map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::PackIdentityMismatch,
            "core state pack cannot be canonicalized",
        )
    })?;
    parse_recognizer_pack(&bytes)
}
