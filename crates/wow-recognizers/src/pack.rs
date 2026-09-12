use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{RecognizerError, RecognizerErrorCode, RecognizerResult};

pub const RECOGNIZER_PACK_SCHEMA_VERSION: u32 = 1;
pub const MAX_RECOGNIZER_PACK_BYTES: usize = 1024 * 1024;
const HARD_MAX_RULES: u32 = 256;
const HARD_MAX_CLAUSES_PER_RULE: u32 = 256;
const HARD_MAX_CLAUSE_DEPTH: u32 = 16;
const HARD_MAX_JOIN_EXPANSIONS: u64 = 1_000_000;
const HARD_MAX_MATCHES: u32 = 100_000;
const HARD_MAX_PROPOSALS: u32 = 200_000;
const HARD_MAX_EXPLANATION_BYTES: u32 = 8 * 1024 * 1024;
const MAX_TEXT: usize = 1024;
const MAX_LIST: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerPackTrustClass {
    Core,
    Calibration,
    Experimental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerPackRollout {
    Shadow,
    Default,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerPackBudgets {
    pub max_rules: u32,
    pub max_clauses_per_rule: u32,
    pub max_clause_depth: u32,
    pub max_join_expansions_per_rule: u64,
    pub max_matches_per_rule_partition: u32,
    pub max_proposals_per_rule_partition: u32,
    pub max_explanation_bytes: u32,
}

impl RecognizerPackBudgets {
    fn validate(&self) -> RecognizerResult<()> {
        if self.max_rules == 0
            || self.max_rules > HARD_MAX_RULES
            || self.max_clauses_per_rule == 0
            || self.max_clauses_per_rule > HARD_MAX_CLAUSES_PER_RULE
            || self.max_clause_depth == 0
            || self.max_clause_depth > HARD_MAX_CLAUSE_DEPTH
            || self.max_join_expansions_per_rule == 0
            || self.max_join_expansions_per_rule > HARD_MAX_JOIN_EXPANSIONS
            || self.max_matches_per_rule_partition == 0
            || self.max_matches_per_rule_partition > HARD_MAX_MATCHES
            || self.max_proposals_per_rule_partition == 0
            || self.max_proposals_per_rule_partition > HARD_MAX_PROPOSALS
            || self.max_explanation_bytes == 0
            || self.max_explanation_bytes > HARD_MAX_EXPLANATION_BYTES
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackBudgetInvalid,
                "recognizer pack budgets are zero or exceed hard limits",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecognizerPackLiteral {
    Boolean(bool),
    Integer(i64),
    String(Box<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecognizerClause {
    Fact {
        #[serde(rename = "as")]
        alias: Box<str>,
        kind: Box<str>,
    },
    Join {
        left: Box<str>,
        right: Box<str>,
    },
    FieldEq {
        field: Box<str>,
        value: RecognizerPackLiteral,
    },
    FieldIn {
        field: Box<str>,
        values: Vec<RecognizerPackLiteral>,
    },
    SameScope {
        left: Box<str>,
        right: Box<str>,
        scope: Box<str>,
    },
    Exists {
        clauses: Vec<RecognizerClause>,
    },
    NotExists {
        clauses: Vec<RecognizerClause>,
        required_complete_capability: Box<str>,
    },
    OrderedRelation {
        left: Box<str>,
        right: Box<str>,
        relation: Box<str>,
    },
    ControlFlowRelation {
        left: Box<str>,
        right: Box<str>,
        relation: Box<str>,
    },
    AllOf {
        clauses: Vec<RecognizerClause>,
    },
    AnyOf {
        clauses: Vec<RecognizerClause>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerCaptureCardinality {
    One,
    Optional,
    BoundedMany,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerCapture {
    pub name: Box<str>,
    pub value_type: Box<str>,
    pub source: Box<str>,
    pub cardinality: RecognizerCaptureCardinality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerOutputConfidence {
    Derived,
    Possible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecognizerOutput {
    EntityAssertion {
        output_id: Box<str>,
        entity_kind_id: Box<str>,
        semantic_key: BTreeMap<Box<str>, Box<str>>,
        confidence: RecognizerOutputConfidence,
    },
    RelationAssertion {
        output_id: Box<str>,
        relation_kind_id: Box<str>,
        source: Box<str>,
        target: Box<str>,
        confidence: RecognizerOutputConfidence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerRule {
    pub rule_id: Box<str>,
    pub version: u32,
    pub required_capabilities: Vec<Box<str>>,
    pub scope: Box<str>,
    pub clauses: Vec<RecognizerClause>,
    pub captures: Vec<RecognizerCapture>,
    pub outputs: Vec<RecognizerOutput>,
    pub positive_fixture_ids: Vec<Box<str>>,
    pub near_negative_fixture_ids: Vec<Box<str>>,
    pub partial_fixture_ids: Vec<Box<str>>,
    pub mutation_fixture_ids: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerPack {
    pub pack_id: Box<str>,
    pub version: Box<str>,
    pub trust_class: RecognizerPackTrustClass,
    pub fact_schema_profile_id: Box<str>,
    pub graph_registry_bundle_id: Box<str>,
    pub evaluation_profile_id: Box<str>,
    pub rollout: RecognizerPackRollout,
    pub budgets: RecognizerPackBudgets,
    pub rules: Vec<RecognizerRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerPackDocument {
    pub schema_version: u32,
    pub pack: RecognizerPack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledRecognizerPack {
    schema: Box<str>,
    pack_digest: Box<str>,
    document: RecognizerPackDocument,
}

impl CompiledRecognizerPack {
    #[must_use]
    pub fn pack_digest(&self) -> &str {
        &self.pack_digest
    }

    #[must_use]
    pub const fn document(&self) -> &RecognizerPackDocument {
        &self.document
    }

    pub fn validate(&self) -> RecognizerResult<()> {
        if self.schema.as_ref() != "wow-recognizers/compiled-pack/e2-b/1" {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackIdentityMismatch,
                "compiled recognizer pack schema is unsupported",
            ));
        }
        validate_document(&self.document)?;
        let canonical = canonical_document(&self.document)?;
        let expected = digest(&canonical);
        if expected != self.pack_digest.as_ref() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackIdentityMismatch,
                "compiled recognizer pack digest does not match",
            ));
        }
        Ok(())
    }
}

/// Parses only exact canonical JSON and compiles a bounded, non-executable pack value.
pub fn parse_recognizer_pack(input: &[u8]) -> RecognizerResult<CompiledRecognizerPack> {
    if input.is_empty() || input.len() > MAX_RECOGNIZER_PACK_BYTES || input.contains(&0) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackTooLarge,
            "recognizer pack bytes are empty, contain NUL, or exceed the hard limit",
        ));
    }
    let document: RecognizerPackDocument = serde_json::from_slice(input).map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::PackSyntaxInvalid,
            "recognizer pack is not valid strict JSON for schema v1",
        )
    })?;
    validate_document(&document)?;
    let canonical = canonical_document(&document)?;
    if input != canonical {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackNonCanonical,
            "recognizer pack input is not canonical JSON",
        ));
    }
    let compiled = CompiledRecognizerPack {
        schema: "wow-recognizers/compiled-pack/e2-b/1".into(),
        pack_digest: digest(&canonical).into(),
        document,
    };
    compiled.validate()?;
    Ok(compiled)
}

fn validate_document(document: &RecognizerPackDocument) -> RecognizerResult<()> {
    if document.schema_version != RECOGNIZER_PACK_SCHEMA_VERSION {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackSchemaUnsupported,
            "recognizer pack schema version is unsupported",
        ));
    }
    let pack = &document.pack;
    for value in [
        pack.pack_id.as_ref(),
        pack.version.as_ref(),
        pack.fact_schema_profile_id.as_ref(),
        pack.graph_registry_bundle_id.as_ref(),
        pack.evaluation_profile_id.as_ref(),
    ] {
        validate_component(value, RecognizerErrorCode::PackInvalid)?;
    }
    pack.budgets.validate()?;
    if pack.rules.is_empty() || pack.rules.len() > pack.budgets.max_rules as usize {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackBudgetInvalid,
            "recognizer pack rule count is empty or exceeds its budget",
        ));
    }
    if pack.trust_class != RecognizerPackTrustClass::Core
        && pack.rollout == RecognizerPackRollout::Default
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackInvalid,
            "only a core pack may request default rollout",
        ));
    }
    let mut previous = None;
    for rule in &pack.rules {
        let key = (rule.rule_id.as_ref(), rule.version);
        if previous.is_some_and(|previous| previous >= key) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackRuleDuplicate,
                "recognizer rules must be unique and canonically ordered",
            ));
        }
        previous = Some(key);
        validate_rule(rule, &pack.budgets)?;
    }
    Ok(())
}

fn validate_rule(rule: &RecognizerRule, budgets: &RecognizerPackBudgets) -> RecognizerResult<()> {
    validate_component(&rule.rule_id, RecognizerErrorCode::PackInvalid)?;
    validate_component(&rule.scope, RecognizerErrorCode::PackInvalid)?;
    if rule.version == 0
        || rule.clauses.is_empty()
        || rule.outputs.is_empty()
        || rule.clauses.len() > budgets.max_clauses_per_rule as usize
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackInvalid,
            "recognizer rule version, clauses, or outputs are invalid",
        ));
    }
    validate_sorted_ids(&rule.required_capabilities, true)?;
    validate_sorted_ids(&rule.positive_fixture_ids, true)?;
    validate_sorted_ids(&rule.near_negative_fixture_ids, true)?;
    validate_sorted_ids(&rule.partial_fixture_ids, true)?;
    validate_sorted_ids(&rule.mutation_fixture_ids, true)?;

    let mut aliases = BTreeSet::new();
    let mut clause_count = 0usize;
    collect_aliases(&rule.clauses, 1, budgets, &mut aliases, &mut clause_count)?;
    validate_clauses(
        &rule.clauses,
        1,
        budgets,
        &aliases,
        &rule.required_capabilities,
    )?;

    let mut captures = BTreeSet::new();
    for capture in &rule.captures {
        validate_component(&capture.name, RecognizerErrorCode::PackInvalid)?;
        validate_component(&capture.value_type, RecognizerErrorCode::PackInvalid)?;
        validate_field_reference(&capture.source, &aliases)?;
        if !captures.insert(capture.name.as_ref()) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackInvalid,
                "recognizer rule contains duplicate captures",
            ));
        }
    }
    if rule.captures.len() > MAX_LIST || rule.outputs.len() > MAX_LIST {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackBudgetInvalid,
            "recognizer captures or outputs exceed the hard list limit",
        ));
    }
    let mut outputs = BTreeSet::new();
    for output in &rule.outputs {
        match output {
            RecognizerOutput::EntityAssertion {
                output_id,
                entity_kind_id,
                semantic_key,
                ..
            } => {
                validate_output_id(output_id, &mut outputs)?;
                validate_component(entity_kind_id, RecognizerErrorCode::PackOutputInvalid)?;
                if semantic_key.is_empty() || semantic_key.len() > 64 {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::PackOutputInvalid,
                        "entity output semantic key is empty or too large",
                    ));
                }
                for (key, value) in semantic_key {
                    validate_component(key, RecognizerErrorCode::PackOutputInvalid)?;
                    validate_output_reference(value, &captures, &aliases)?;
                }
            }
            RecognizerOutput::RelationAssertion {
                output_id,
                relation_kind_id,
                source,
                target,
                ..
            } => {
                validate_output_id(output_id, &mut outputs)?;
                validate_component(relation_kind_id, RecognizerErrorCode::PackOutputInvalid)?;
                validate_output_reference(source, &captures, &aliases)?;
                validate_output_reference(target, &captures, &aliases)?;
            }
        }
    }
    Ok(())
}

fn collect_aliases<'a>(
    clauses: &'a [RecognizerClause],
    depth: u32,
    budgets: &RecognizerPackBudgets,
    aliases: &mut BTreeSet<&'a str>,
    count: &mut usize,
) -> RecognizerResult<()> {
    if depth > budgets.max_clause_depth {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackClauseInvalid,
            "recognizer clause nesting exceeds the rule budget",
        ));
    }
    for clause in clauses {
        *count += 1;
        if *count > budgets.max_clauses_per_rule as usize {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackBudgetInvalid,
                "recognizer recursive clause count exceeds the rule budget",
            ));
        }
        match clause {
            RecognizerClause::Fact { alias, kind } => {
                validate_component(alias, RecognizerErrorCode::PackClauseInvalid)?;
                validate_component(kind, RecognizerErrorCode::PackClauseInvalid)?;
                if !aliases.insert(alias) {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::PackClauseInvalid,
                        "recognizer rule contains a duplicate fact alias",
                    ));
                }
            }
            RecognizerClause::Exists { clauses }
            | RecognizerClause::NotExists { clauses, .. }
            | RecognizerClause::AllOf { clauses }
            | RecognizerClause::AnyOf { clauses } => {
                if clauses.is_empty() {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::PackClauseInvalid,
                        "recognizer composite clause is empty",
                    ));
                }
                collect_aliases(clauses, depth + 1, budgets, aliases, count)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_clauses(
    clauses: &[RecognizerClause],
    depth: u32,
    budgets: &RecognizerPackBudgets,
    aliases: &BTreeSet<&str>,
    capabilities: &[Box<str>],
) -> RecognizerResult<()> {
    if depth > budgets.max_clause_depth {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackClauseInvalid,
            "recognizer clause nesting exceeds the rule budget",
        ));
    }
    for clause in clauses {
        match clause {
            RecognizerClause::Fact { .. } => {}
            RecognizerClause::Join { left, right }
            | RecognizerClause::SameScope { left, right, .. }
            | RecognizerClause::OrderedRelation { left, right, .. }
            | RecognizerClause::ControlFlowRelation { left, right, .. } => {
                validate_field_reference(left, aliases)?;
                validate_field_reference(right, aliases)?;
            }
            RecognizerClause::FieldEq { field, value } => {
                validate_field_reference(field, aliases)?;
                validate_literal(value)?;
            }
            RecognizerClause::FieldIn { field, values } => {
                validate_field_reference(field, aliases)?;
                if values.is_empty() || values.len() > 256 {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::PackClauseInvalid,
                        "field_in literal set is empty or too large",
                    ));
                }
                for value in values {
                    validate_literal(value)?;
                }
            }
            RecognizerClause::Exists { clauses }
            | RecognizerClause::AllOf { clauses }
            | RecognizerClause::AnyOf { clauses } => {
                validate_clauses(clauses, depth + 1, budgets, aliases, capabilities)?;
            }
            RecognizerClause::NotExists {
                clauses,
                required_complete_capability,
            } => {
                if capabilities
                    .binary_search_by(|value| value.as_ref().cmp(required_complete_capability))
                    .is_err()
                {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::PackNegativeCoverageMissing,
                        "not_exists lacks its declared complete-coverage capability",
                    ));
                }
                validate_clauses(clauses, depth + 1, budgets, aliases, capabilities)?;
            }
        }
        match clause {
            RecognizerClause::SameScope { scope, .. }
            | RecognizerClause::OrderedRelation {
                relation: scope, ..
            }
            | RecognizerClause::ControlFlowRelation {
                relation: scope, ..
            } => validate_component(scope, RecognizerErrorCode::PackClauseInvalid)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_literal(value: &RecognizerPackLiteral) -> RecognizerResult<()> {
    if let RecognizerPackLiteral::String(value) = value {
        validate_text(value, RecognizerErrorCode::PackClauseInvalid)?;
    }
    Ok(())
}

fn validate_field_reference(value: &str, aliases: &BTreeSet<&str>) -> RecognizerResult<()> {
    validate_text(value, RecognizerErrorCode::PackClauseInvalid)?;
    let Some((alias, field)) = value.split_once('.') else {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackClauseInvalid,
            "recognizer field reference must be alias.field",
        ));
    };
    validate_component(alias, RecognizerErrorCode::PackClauseInvalid)?;
    validate_component(field, RecognizerErrorCode::PackClauseInvalid)?;
    if !aliases.contains(alias) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackClauseInvalid,
            "recognizer field reference uses an undeclared fact alias",
        ));
    }
    Ok(())
}

fn validate_output_reference(
    value: &str,
    captures: &BTreeSet<&str>,
    aliases: &BTreeSet<&str>,
) -> RecognizerResult<()> {
    if captures.contains(value) {
        return Ok(());
    }
    validate_field_reference(value, aliases).map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::PackOutputInvalid,
            "recognizer output references neither a capture nor a fact field",
        )
    })
}

fn validate_output_id<'a>(
    output_id: &'a str,
    outputs: &mut BTreeSet<&'a str>,
) -> RecognizerResult<()> {
    validate_component(output_id, RecognizerErrorCode::PackOutputInvalid)?;
    if !outputs.insert(output_id) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackOutputInvalid,
            "recognizer rule contains duplicate output IDs",
        ));
    }
    Ok(())
}

fn validate_sorted_ids(values: &[Box<str>], require_nonempty: bool) -> RecognizerResult<()> {
    if (require_nonempty && values.is_empty()) || values.len() > MAX_LIST {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PackInvalid,
            "recognizer ID list is empty or exceeds the hard limit",
        ));
    }
    let mut previous = None;
    for value in values {
        validate_component(value, RecognizerErrorCode::PackInvalid)?;
        if previous.is_some_and(|previous| previous >= value.as_ref()) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PackInvalid,
                "recognizer ID list is not unique and canonically ordered",
            ));
        }
        previous = Some(value.as_ref());
    }
    Ok(())
}

fn validate_component(value: &str, code: RecognizerErrorCode) -> RecognizerResult<()> {
    if value.is_empty()
        || value.len() > MAX_TEXT
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
    {
        return Err(RecognizerError::new(
            code,
            "recognizer pack component is invalid",
        ));
    }
    Ok(())
}

fn validate_text(value: &str, code: RecognizerErrorCode) -> RecognizerResult<()> {
    if value.is_empty() || value.len() > MAX_TEXT || value.chars().any(char::is_control) {
        return Err(RecognizerError::new(
            code,
            "recognizer pack text is invalid",
        ));
    }
    Ok(())
}

fn canonical_document(document: &RecognizerPackDocument) -> RecognizerResult<Vec<u8>> {
    canonical_json_bytes(document).map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::PackNonCanonical,
            "recognizer pack cannot be represented as canonical JSON",
        )
    })
}

fn digest(canonical: &[u8]) -> String {
    format!("recognizer-pack:sha256:{}", hex(&Sha256::digest(canonical)))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
