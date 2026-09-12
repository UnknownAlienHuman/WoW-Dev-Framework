use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    CompiledRecognizerPack, RecognizerClause, RecognizerError, RecognizerErrorCode,
    RecognizerOutput, RecognizerPlanId, RecognizerResult,
};

pub const RECOGNIZER_PLAN_SCHEMA: &str = "wow-recognizers/semantic-plan/e2-b/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerPlanStepKind {
    Fact,
    Join,
    FieldEq,
    FieldIn,
    SameScope,
    Exists,
    NotExists,
    OrderedRelation,
    ControlFlowRelation,
    AllOf,
    AnyOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerPlanCostClass {
    FactScan,
    ExactPredicate,
    ScopePredicate,
    OrderedRelation,
    ControlFlowRelation,
    PositiveExistence,
    NegativeExistence,
    EqualityJoin,
    BooleanComposite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerPlanStep {
    step_id: Box<str>,
    clause_path: Vec<u32>,
    kind: RecognizerPlanStepKind,
    cost_class: RecognizerPlanCostClass,
    required_aliases: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    produced_alias: Option<Box<str>>,
    required_capabilities: Vec<Box<str>>,
}

impl RecognizerPlanStep {
    #[must_use]
    pub fn step_id(&self) -> &str {
        &self.step_id
    }

    #[must_use]
    pub fn clause_path(&self) -> &[u32] {
        &self.clause_path
    }

    #[must_use]
    pub const fn kind(&self) -> RecognizerPlanStepKind {
        self.kind
    }

    #[must_use]
    pub const fn cost_class(&self) -> RecognizerPlanCostClass {
        self.cost_class
    }

    #[must_use]
    pub fn required_aliases(&self) -> &[Box<str>] {
        &self.required_aliases
    }

    #[must_use]
    pub fn produced_alias(&self) -> Option<&str> {
        self.produced_alias.as_deref()
    }

    #[must_use]
    pub fn required_capabilities(&self) -> &[Box<str>] {
        &self.required_capabilities
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerRulePlanBounds {
    pub clause_steps: u32,
    pub fact_scans: u32,
    pub joins: u32,
    pub predicates: u32,
    pub composites: u32,
    pub captures: u32,
    pub outputs: u32,
    pub max_join_expansions: u64,
    pub max_matches: u32,
    pub max_proposals: u32,
    pub max_explanation_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledRecognizerRulePlan {
    rule_id: Box<str>,
    rule_version: u32,
    semantic_step_count: u32,
    evaluation_order: Vec<RecognizerPlanStep>,
    capture_names: Vec<Box<str>>,
    output_ids: Vec<Box<str>>,
    bounds: RecognizerRulePlanBounds,
}

impl CompiledRecognizerRulePlan {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub const fn rule_version(&self) -> u32 {
        self.rule_version
    }

    #[must_use]
    pub fn evaluation_order(&self) -> &[RecognizerPlanStep] {
        &self.evaluation_order
    }

    #[must_use]
    pub fn capture_names(&self) -> &[Box<str>] {
        &self.capture_names
    }

    #[must_use]
    pub fn output_ids(&self) -> &[Box<str>] {
        &self.output_ids
    }

    #[must_use]
    pub const fn bounds(&self) -> RecognizerRulePlanBounds {
        self.bounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerPlanBounds {
    pub rules: u32,
    pub clause_steps: u32,
    pub fact_scans: u32,
    pub joins: u32,
    pub predicates: u32,
    pub composites: u32,
    pub captures: u32,
    pub outputs: u32,
    pub max_join_expansions: u64,
    pub max_matches: u64,
    pub max_proposals: u64,
    pub max_explanation_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledRecognizerPlan {
    schema: Box<str>,
    plan_id: RecognizerPlanId,
    source_pack_digest: Box<str>,
    rules: Vec<CompiledRecognizerRulePlan>,
    bounds: RecognizerPlanBounds,
}

impl CompiledRecognizerPlan {
    #[must_use]
    pub fn plan_id(&self) -> &RecognizerPlanId {
        &self.plan_id
    }

    #[must_use]
    pub fn source_pack_digest(&self) -> &str {
        &self.source_pack_digest
    }

    #[must_use]
    pub fn rules(&self) -> &[CompiledRecognizerRulePlan] {
        &self.rules
    }

    #[must_use]
    pub const fn bounds(&self) -> RecognizerPlanBounds {
        self.bounds
    }

    pub fn validate(&self) -> RecognizerResult<()> {
        if self.schema.as_ref() != RECOGNIZER_PLAN_SCHEMA
            || !self.source_pack_digest.starts_with("recognizer-pack:sha256:")
            || self.rules.is_empty()
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer semantic plan header is invalid",
            ));
        }
        let mut previous_rule = None;
        let mut aggregate = RecognizerPlanBounds::zero();
        for rule in &self.rules {
            let key = (rule.rule_id.as_ref(), rule.rule_version);
            if previous_rule.is_some_and(|previous| previous >= key) {
                return Err(RecognizerError::new(
                    RecognizerErrorCode::PlanInvalid,
                    "recognizer semantic rule plans are not canonically ordered",
                ));
            }
            previous_rule = Some(key);
            validate_rule_plan(rule)?;
            aggregate.add_rule(rule.bounds)?;
        }
        if aggregate != self.bounds {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer semantic plan bounds do not match its rules",
            ));
        }
        let expected = plan_id(&self.source_pack_digest, &self.rules, self.bounds)?;
        if expected != self.plan_id {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanIdentityMismatch,
                "recognizer semantic plan identity does not match",
            ));
        }
        Ok(())
    }
}

impl RecognizerPlanBounds {
    const fn zero() -> Self {
        Self {
            rules: 0,
            clause_steps: 0,
            fact_scans: 0,
            joins: 0,
            predicates: 0,
            composites: 0,
            captures: 0,
            outputs: 0,
            max_join_expansions: 0,
            max_matches: 0,
            max_proposals: 0,
            max_explanation_bytes: 0,
        }
    }

    fn add_rule(&mut self, rule: RecognizerRulePlanBounds) -> RecognizerResult<()> {
        self.rules = self.rules.checked_add(1).ok_or_else(plan_overflow)?;
        self.clause_steps = self
            .clause_steps
            .checked_add(rule.clause_steps)
            .ok_or_else(plan_overflow)?;
        self.fact_scans = self
            .fact_scans
            .checked_add(rule.fact_scans)
            .ok_or_else(plan_overflow)?;
        self.joins = self.joins.checked_add(rule.joins).ok_or_else(plan_overflow)?;
        self.predicates = self
            .predicates
            .checked_add(rule.predicates)
            .ok_or_else(plan_overflow)?;
        self.composites = self
            .composites
            .checked_add(rule.composites)
            .ok_or_else(plan_overflow)?;
        self.captures = self
            .captures
            .checked_add(rule.captures)
            .ok_or_else(plan_overflow)?;
        self.outputs = self
            .outputs
            .checked_add(rule.outputs)
            .ok_or_else(plan_overflow)?;
        self.max_join_expansions = self
            .max_join_expansions
            .checked_add(rule.max_join_expansions)
            .ok_or_else(plan_overflow)?;
        self.max_matches = self
            .max_matches
            .checked_add(u64::from(rule.max_matches))
            .ok_or_else(plan_overflow)?;
        self.max_proposals = self
            .max_proposals
            .checked_add(u64::from(rule.max_proposals))
            .ok_or_else(plan_overflow)?;
        self.max_explanation_bytes = self
            .max_explanation_bytes
            .checked_add(u64::from(rule.max_explanation_bytes))
            .ok_or_else(plan_overflow)?;
        Ok(())
    }
}

/// Compiles a validated pack into a deterministic semantic evaluation plan.
///
/// The plan contains no runtime statistics or physical indexes. Cost-class ordering is fixed by
/// this schema and ties are resolved by the original clause path, so equivalent pack bytes produce
/// the same plan independently of machine, worker count, or observed data distribution.
pub fn compile_recognizer_plan(
    pack: &CompiledRecognizerPack,
) -> RecognizerResult<CompiledRecognizerPlan> {
    pack.validate()?;
    let document = pack.document();
    let mut rules = Vec::with_capacity(document.pack.rules.len());
    let mut aggregate = RecognizerPlanBounds::zero();
    for rule in &document.pack.rules {
        let mut steps = Vec::new();
        flatten_clauses(
            &rule.rule_id,
            rule.version,
            &rule.clauses,
            &[],
            &mut steps,
        )?;
        steps.sort_by(|left, right| {
            (left.cost_class, left.clause_path.as_slice())
                .cmp(&(right.cost_class, right.clause_path.as_slice()))
        });
        let mut captures = rule
            .captures
            .iter()
            .map(|capture| capture.name.clone())
            .collect::<Vec<_>>();
        captures.sort();
        if captures.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer plan contains duplicate capture names",
            ));
        }
        let mut output_ids = rule.outputs.iter().map(output_id).collect::<Vec<_>>();
        output_ids.sort();
        if output_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer plan contains duplicate output IDs",
            ));
        }
        let bounds = rule_bounds(
            &steps,
            captures.len(),
            output_ids.len(),
            &document.pack.budgets,
        )?;
        let plan = CompiledRecognizerRulePlan {
            rule_id: rule.rule_id.clone(),
            rule_version: rule.version,
            semantic_step_count: u32::try_from(steps.len()).map_err(|_| plan_overflow())?,
            evaluation_order: steps,
            capture_names: captures,
            output_ids,
            bounds,
        };
        validate_rule_plan(&plan)?;
        aggregate.add_rule(bounds)?;
        rules.push(plan);
    }
    let plan_id = plan_id(pack.pack_digest(), &rules, aggregate)?;
    let plan = CompiledRecognizerPlan {
        schema: RECOGNIZER_PLAN_SCHEMA.into(),
        plan_id,
        source_pack_digest: pack.pack_digest().into(),
        rules,
        bounds: aggregate,
    };
    plan.validate()?;
    Ok(plan)
}

fn flatten_clauses(
    rule_id: &str,
    rule_version: u32,
    clauses: &[RecognizerClause],
    parent: &[u32],
    output: &mut Vec<RecognizerPlanStep>,
) -> RecognizerResult<()> {
    for (index, clause) in clauses.iter().enumerate() {
        let mut path = parent.to_vec();
        path.push(u32::try_from(index).map_err(|_| plan_overflow())?);
        let (kind, cost_class, mut required_aliases, produced_alias, required_capabilities) =
            step_fields(clause)?;
        required_aliases.sort();
        required_aliases.dedup();
        let step_id = step_id(rule_id, rule_version, &path, kind)?;
        output.push(RecognizerPlanStep {
            step_id,
            clause_path: path.clone(),
            kind,
            cost_class,
            required_aliases,
            produced_alias,
            required_capabilities,
        });
        match clause {
            RecognizerClause::Exists { clauses }
            | RecognizerClause::NotExists { clauses, .. }
            | RecognizerClause::AllOf { clauses }
            | RecognizerClause::AnyOf { clauses } => {
                flatten_clauses(rule_id, rule_version, clauses, &path, output)?;
            }
            _ => {}
        }
    }
    Ok(())
}

type StepFields = (
    RecognizerPlanStepKind,
    RecognizerPlanCostClass,
    Vec<Box<str>>,
    Option<Box<str>>,
    Vec<Box<str>>,
);

fn step_fields(clause: &RecognizerClause) -> RecognizerResult<StepFields> {
    let fields = match clause {
        RecognizerClause::Fact { alias, .. } => (
            RecognizerPlanStepKind::Fact,
            RecognizerPlanCostClass::FactScan,
            Vec::new(),
            Some(alias.clone()),
            Vec::new(),
        ),
        RecognizerClause::Join { left, right } => (
            RecognizerPlanStepKind::Join,
            RecognizerPlanCostClass::EqualityJoin,
            aliases([left.as_ref(), right.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::FieldEq { field, .. } => (
            RecognizerPlanStepKind::FieldEq,
            RecognizerPlanCostClass::ExactPredicate,
            aliases([field.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::FieldIn { field, .. } => (
            RecognizerPlanStepKind::FieldIn,
            RecognizerPlanCostClass::ExactPredicate,
            aliases([field.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::SameScope { left, right, .. } => (
            RecognizerPlanStepKind::SameScope,
            RecognizerPlanCostClass::ScopePredicate,
            aliases([left.as_ref(), right.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::Exists { .. } => (
            RecognizerPlanStepKind::Exists,
            RecognizerPlanCostClass::PositiveExistence,
            Vec::new(),
            None,
            Vec::new(),
        ),
        RecognizerClause::NotExists {
            required_complete_capability,
            ..
        } => (
            RecognizerPlanStepKind::NotExists,
            RecognizerPlanCostClass::NegativeExistence,
            Vec::new(),
            None,
            vec![required_complete_capability.clone()],
        ),
        RecognizerClause::OrderedRelation { left, right, .. } => (
            RecognizerPlanStepKind::OrderedRelation,
            RecognizerPlanCostClass::OrderedRelation,
            aliases([left.as_ref(), right.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::ControlFlowRelation { left, right, .. } => (
            RecognizerPlanStepKind::ControlFlowRelation,
            RecognizerPlanCostClass::ControlFlowRelation,
            aliases([left.as_ref(), right.as_ref()])?,
            None,
            Vec::new(),
        ),
        RecognizerClause::AllOf { .. } => (
            RecognizerPlanStepKind::AllOf,
            RecognizerPlanCostClass::BooleanComposite,
            Vec::new(),
            None,
            Vec::new(),
        ),
        RecognizerClause::AnyOf { .. } => (
            RecognizerPlanStepKind::AnyOf,
            RecognizerPlanCostClass::BooleanComposite,
            Vec::new(),
            None,
            Vec::new(),
        ),
    };
    Ok(fields)
}

fn aliases<const N: usize>(fields: [&str; N]) -> RecognizerResult<Vec<Box<str>>> {
    fields
        .into_iter()
        .map(|field| {
            field
                .split_once('.')
                .map(|(alias, _)| alias.into())
                .ok_or_else(|| {
                    RecognizerError::new(
                        RecognizerErrorCode::PlanInvalid,
                        "recognizer plan field reference has no alias",
                    )
                })
        })
        .collect()
}

fn output_id(output: &RecognizerOutput) -> Box<str> {
    match output {
        RecognizerOutput::EntityAssertion { output_id, .. }
        | RecognizerOutput::RelationAssertion { output_id, .. } => output_id.clone(),
    }
}

fn rule_bounds(
    steps: &[RecognizerPlanStep],
    captures: usize,
    outputs: usize,
    budgets: &crate::RecognizerPackBudgets,
) -> RecognizerResult<RecognizerRulePlanBounds> {
    let mut bounds = RecognizerRulePlanBounds {
        clause_steps: u32::try_from(steps.len()).map_err(|_| plan_overflow())?,
        fact_scans: 0,
        joins: 0,
        predicates: 0,
        composites: 0,
        captures: u32::try_from(captures).map_err(|_| plan_overflow())?,
        outputs: u32::try_from(outputs).map_err(|_| plan_overflow())?,
        max_join_expansions: budgets.max_join_expansions_per_rule,
        max_matches: budgets.max_matches_per_rule_partition,
        max_proposals: budgets.max_proposals_per_rule_partition,
        max_explanation_bytes: budgets.max_explanation_bytes,
    };
    if bounds.outputs > bounds.max_proposals {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PlanInvalid,
            "recognizer rule outputs exceed the declared proposal budget",
        ));
    }
    for step in steps {
        match step.cost_class {
            RecognizerPlanCostClass::FactScan => {
                bounds.fact_scans = bounds.fact_scans.checked_add(1).ok_or_else(plan_overflow)?;
            }
            RecognizerPlanCostClass::EqualityJoin => {
                bounds.joins = bounds.joins.checked_add(1).ok_or_else(plan_overflow)?;
            }
            RecognizerPlanCostClass::BooleanComposite
            | RecognizerPlanCostClass::PositiveExistence
            | RecognizerPlanCostClass::NegativeExistence => {
                bounds.composites = bounds
                    .composites
                    .checked_add(1)
                    .ok_or_else(plan_overflow)?;
            }
            _ => {
                bounds.predicates = bounds
                    .predicates
                    .checked_add(1)
                    .ok_or_else(plan_overflow)?;
            }
        }
    }
    Ok(bounds)
}

fn validate_rule_plan(rule: &CompiledRecognizerRulePlan) -> RecognizerResult<()> {
    if rule.rule_id.is_empty()
        || rule.rule_version == 0
        || rule.evaluation_order.is_empty()
        || rule.semantic_step_count as usize != rule.evaluation_order.len()
        || rule.bounds.clause_steps != rule.semantic_step_count
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PlanInvalid,
            "compiled recognizer rule plan header or counts are invalid",
        ));
    }
    let mut paths = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut previous = None;
    for step in &rule.evaluation_order {
        if step.clause_path.is_empty()
            || !paths.insert(step.clause_path.clone())
            || !ids.insert(step.step_id.as_ref())
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer plan contains duplicate or empty step identities",
            ));
        }
        let key = (step.cost_class, step.clause_path.as_slice());
        if previous.is_some_and(|previous| previous >= key) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer plan steps are not in deterministic evaluation order",
            ));
        }
        previous = Some(key);
        if step.required_aliases.windows(2).any(|pair| pair[0] >= pair[1])
            || step
                .required_capabilities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanInvalid,
                "recognizer plan dependencies are not unique and ordered",
            ));
        }
        let expected = step_id(
            &rule.rule_id,
            rule.rule_version,
            &step.clause_path,
            step.kind,
        )?;
        if expected.as_ref() != step.step_id.as_ref() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::PlanIdentityMismatch,
                "recognizer plan step identity does not match",
            ));
        }
    }
    if rule.capture_names.windows(2).any(|pair| pair[0] >= pair[1])
        || rule.output_ids.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::PlanInvalid,
            "recognizer plan captures or outputs are not unique and ordered",
        ));
    }
    Ok(())
}

fn step_id(
    rule_id: &str,
    rule_version: u32,
    path: &[u32],
    kind: RecognizerPlanStepKind,
) -> RecognizerResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        rule_id: &'a str,
        rule_version: u32,
        path: &'a [u32],
        kind: RecognizerPlanStepKind,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: "wow-recognizers/plan-step/e2-b/1",
        rule_id,
        rule_version,
        path,
        kind,
    })
    .map_err(|_| plan_identity_error())?;
    Ok(format!("recognizer-plan-step:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

fn plan_id(
    source_pack_digest: &str,
    rules: &[CompiledRecognizerRulePlan],
    bounds: RecognizerPlanBounds,
) -> RecognizerResult<RecognizerPlanId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        source_pack_digest: &'a str,
        rules: &'a [CompiledRecognizerRulePlan],
        bounds: RecognizerPlanBounds,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: RECOGNIZER_PLAN_SCHEMA,
        source_pack_digest,
        rules,
        bounds,
    })
    .map_err(|_| plan_identity_error())?;
    RecognizerPlanId::new(format!(
        "recognizer-plan:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn plan_overflow() -> RecognizerError {
    RecognizerError::new(
        RecognizerErrorCode::PlanInvalid,
        "recognizer semantic plan bounds overflow",
    )
}

fn plan_identity_error() -> RecognizerError {
    RecognizerError::new(
        RecognizerErrorCode::PlanIdentityMismatch,
        "recognizer semantic plan identity cannot be canonicalized",
    )
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
