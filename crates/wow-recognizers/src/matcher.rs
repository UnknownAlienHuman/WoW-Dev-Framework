use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{
    CoverageId, EvidenceId, GenerationContext, GenerationContextId, StableHandleId,
    canonical_json_bytes,
};
use wow_graph::GraphConfidence;

use crate::{
    CompiledRecognizerPack, CompiledRecognizerPlan, CompiledRecognizerRulePlan, RecognizerCapture,
    RecognizerCaptureCardinality, RecognizerClause, RecognizerError, RecognizerErrorCode,
    RecognizerFact, RecognizerFactBundle, RecognizerFactCoverageState, RecognizerFactId,
    RecognizerFactLimits, RecognizerFactScopeKind, RecognizerFactValue, RecognizerMatchId,
    RecognizerOutput, RecognizerOutputConfidence, RecognizerOutputPartitionId, RecognizerPlanId,
    RecognizerProposalId, RecognizerResult, RecognizerRule,
};

pub const RECOGNIZER_OUTPUT_PARTITION_SCHEMA: &str = "wow-recognizers/output-partition/e2-b/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerRuleOutcomeState {
    Matched,
    EvaluatedNoMatch,
    NotEvaluated,
    Partial,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "cardinality", content = "value", rename_all = "snake_case")]
pub enum RecognizerCapturedValue {
    One(RecognizerFactValue),
    Optional(Option<RecognizerFactValue>),
    BoundedMany(Vec<RecognizerFactValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerMatch {
    match_id: RecognizerMatchId,
    source_pack_digest: Box<str>,
    plan_id: RecognizerPlanId,
    fact_bundle_id: crate::RecognizerFactBundleId,
    rule_id: Box<str>,
    rule_version: u32,
    partition_id: Box<str>,
    bindings: BTreeMap<Box<str>, RecognizerFactId>,
    captures: BTreeMap<Box<str>, RecognizerCapturedValue>,
    decisive_fact_ids: Vec<RecognizerFactId>,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<CoverageId>,
    confidence: RecognizerOutputConfidence,
}

impl RecognizerMatch {
    #[must_use]
    pub fn match_id(&self) -> &RecognizerMatchId {
        &self.match_id
    }

    #[must_use]
    pub fn bindings(&self) -> &BTreeMap<Box<str>, RecognizerFactId> {
        &self.bindings
    }

    #[must_use]
    pub fn captures(&self) -> &BTreeMap<Box<str>, RecognizerCapturedValue> {
        &self.captures
    }

    #[must_use]
    pub fn decisive_fact_ids(&self) -> &[RecognizerFactId] {
        &self.decisive_fact_ids
    }

    #[must_use]
    pub const fn confidence(&self) -> RecognizerOutputConfidence {
        self.confidence
    }

    fn validate(&self) -> RecognizerResult<()> {
        if self.rule_id.is_empty()
            || self.rule_version == 0
            || self.partition_id.is_empty()
            || self.bindings.is_empty()
            || self.decisive_fact_ids.is_empty()
            || self.source_handle_ids.is_empty()
            || self.evidence_ids.is_empty()
            || !strictly_sorted(&self.decisive_fact_ids)
            || !strictly_sorted(&self.source_handle_ids)
            || !strictly_sorted(&self.evidence_ids)
            || !strictly_sorted(&self.coverage_ids)
        {
            return Err(matcher_error(
                RecognizerErrorCode::MatchInvalid,
                "recognizer match is empty or not canonical",
            ));
        }
        let expected = derive_match_id(MatchIdentity {
            source_pack_digest: &self.source_pack_digest,
            plan_id: &self.plan_id,
            fact_bundle_id: &self.fact_bundle_id,
            rule_id: &self.rule_id,
            rule_version: self.rule_version,
            partition_id: &self.partition_id,
            bindings: &self.bindings,
            captures: &self.captures,
            decisive_fact_ids: &self.decisive_fact_ids,
            source_handle_ids: &self.source_handle_ids,
            evidence_ids: &self.evidence_ids,
            coverage_ids: &self.coverage_ids,
            confidence: self.confidence,
        })?;
        if expected != self.match_id {
            return Err(matcher_error(
                RecognizerErrorCode::MatchIdentityMismatch,
                "recognizer match identity does not match",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecognizerProposedAssertion {
    Entity {
        proposal_id: RecognizerProposalId,
        output_id: Box<str>,
        entity_kind_id: Box<str>,
        semantic_key: BTreeMap<Box<str>, RecognizerFactValue>,
        confidence: RecognizerOutputConfidence,
        match_id: RecognizerMatchId,
        decisive_fact_ids: Vec<RecognizerFactId>,
        source_handle_ids: Vec<StableHandleId>,
        evidence_ids: Vec<EvidenceId>,
        coverage_ids: Vec<CoverageId>,
    },
    Relation {
        proposal_id: RecognizerProposalId,
        output_id: Box<str>,
        relation_kind_id: Box<str>,
        source: RecognizerFactValue,
        target: RecognizerFactValue,
        confidence: RecognizerOutputConfidence,
        match_id: RecognizerMatchId,
        decisive_fact_ids: Vec<RecognizerFactId>,
        source_handle_ids: Vec<StableHandleId>,
        evidence_ids: Vec<EvidenceId>,
        coverage_ids: Vec<CoverageId>,
    },
}

impl RecognizerProposedAssertion {
    #[must_use]
    pub fn proposal_id(&self) -> &RecognizerProposalId {
        match self {
            Self::Entity { proposal_id, .. } | Self::Relation { proposal_id, .. } => proposal_id,
        }
    }

    #[must_use]
    pub fn output_id(&self) -> &str {
        match self {
            Self::Entity { output_id, .. } | Self::Relation { output_id, .. } => output_id,
        }
    }

    #[must_use]
    pub const fn confidence(&self) -> RecognizerOutputConfidence {
        match self {
            Self::Entity { confidence, .. } | Self::Relation { confidence, .. } => *confidence,
        }
    }

    fn validate(&self) -> RecognizerResult<()> {
        let expected = match self {
            Self::Entity {
                output_id,
                entity_kind_id,
                semantic_key,
                confidence,
                match_id,
                decisive_fact_ids,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } => {
                if output_id.is_empty()
                    || entity_kind_id.is_empty()
                    || semantic_key.is_empty()
                    || !strictly_sorted(decisive_fact_ids)
                    || !strictly_sorted(source_handle_ids)
                    || !strictly_sorted(evidence_ids)
                    || !strictly_sorted(coverage_ids)
                {
                    return Err(matcher_error(
                        RecognizerErrorCode::ProposalInvalid,
                        "recognizer entity proposal is empty or not canonical",
                    ));
                }
                derive_proposal_id(&ProposalIdentity::Entity {
                    output_id,
                    entity_kind_id,
                    semantic_key,
                    confidence: *confidence,
                    match_id,
                    decisive_fact_ids,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                })?
            }
            Self::Relation {
                output_id,
                relation_kind_id,
                source,
                target,
                confidence,
                match_id,
                decisive_fact_ids,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } => {
                if output_id.is_empty()
                    || relation_kind_id.is_empty()
                    || !strictly_sorted(decisive_fact_ids)
                    || !strictly_sorted(source_handle_ids)
                    || !strictly_sorted(evidence_ids)
                    || !strictly_sorted(coverage_ids)
                {
                    return Err(matcher_error(
                        RecognizerErrorCode::ProposalInvalid,
                        "recognizer relation proposal is empty or not canonical",
                    ));
                }
                derive_proposal_id(&ProposalIdentity::Relation {
                    output_id,
                    relation_kind_id,
                    source,
                    target,
                    confidence: *confidence,
                    match_id,
                    decisive_fact_ids,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                })?
            }
        };
        if expected != *self.proposal_id() {
            return Err(matcher_error(
                RecognizerErrorCode::ProposalIdentityMismatch,
                "recognizer proposal identity does not match",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerRuleOutcome {
    rule_id: Box<str>,
    rule_version: u32,
    state: RecognizerRuleOutcomeState,
    truncated: bool,
    blocker_ids: Vec<Box<str>>,
    matches: Vec<RecognizerMatch>,
    proposals: Vec<RecognizerProposedAssertion>,
}

impl RecognizerRuleOutcome {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub const fn rule_version(&self) -> u32 {
        self.rule_version
    }

    #[must_use]
    pub const fn state(&self) -> RecognizerRuleOutcomeState {
        self.state
    }

    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    #[must_use]
    pub fn blocker_ids(&self) -> &[Box<str>] {
        &self.blocker_ids
    }

    #[must_use]
    pub fn matches(&self) -> &[RecognizerMatch] {
        &self.matches
    }

    #[must_use]
    pub fn proposals(&self) -> &[RecognizerProposedAssertion] {
        &self.proposals
    }

    fn validate(&self) -> RecognizerResult<()> {
        if self.rule_id.is_empty()
            || self.rule_version == 0
            || !strictly_sorted(&self.blocker_ids)
            || self
                .matches
                .windows(2)
                .any(|pair| pair[0].match_id >= pair[1].match_id)
            || self
                .proposals
                .windows(2)
                .any(|pair| pair[0].proposal_id() >= pair[1].proposal_id())
        {
            return Err(matcher_error(
                RecognizerErrorCode::MatchInvalid,
                "recognizer rule outcome is not canonical",
            ));
        }
        for item in &self.matches {
            item.validate()?;
        }
        for item in &self.proposals {
            item.validate()?;
        }
        let has_output = !self.matches.is_empty() || !self.proposals.is_empty();
        let valid_state = match self.state {
            RecognizerRuleOutcomeState::Matched => has_output && !self.truncated,
            RecognizerRuleOutcomeState::EvaluatedNoMatch => !has_output && !self.truncated,
            RecognizerRuleOutcomeState::NotEvaluated => !has_output && !self.blocker_ids.is_empty(),
            RecognizerRuleOutcomeState::Partial => self.truncated || !self.blocker_ids.is_empty(),
            RecognizerRuleOutcomeState::Cancelled => !has_output && !self.blocker_ids.is_empty(),
        };
        if !valid_state {
            return Err(matcher_error(
                RecognizerErrorCode::MatchInvalid,
                "recognizer rule outcome state disagrees with its records",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerOutputPartition {
    schema: Box<str>,
    partition_id: RecognizerOutputPartitionId,
    context_id: GenerationContextId,
    source_pack_digest: Box<str>,
    plan_id: RecognizerPlanId,
    fact_bundle_id: crate::RecognizerFactBundleId,
    producer_partition_id: Box<str>,
    outcomes: Vec<RecognizerRuleOutcome>,
}

impl RecognizerOutputPartition {
    #[must_use]
    pub fn partition_id(&self) -> &RecognizerOutputPartitionId {
        &self.partition_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub fn source_pack_digest(&self) -> &str {
        &self.source_pack_digest
    }

    #[must_use]
    pub fn plan_id(&self) -> &RecognizerPlanId {
        &self.plan_id
    }

    #[must_use]
    pub fn fact_bundle_id(&self) -> &crate::RecognizerFactBundleId {
        &self.fact_bundle_id
    }

    #[must_use]
    pub fn producer_partition_id(&self) -> &str {
        &self.producer_partition_id
    }

    #[must_use]
    pub fn outcomes(&self) -> &[RecognizerRuleOutcome] {
        &self.outcomes
    }

    pub fn validate(&self) -> RecognizerResult<()> {
        if self.schema.as_ref() != RECOGNIZER_OUTPUT_PARTITION_SCHEMA
            || self.source_pack_digest.is_empty()
            || self.producer_partition_id.is_empty()
            || self.outcomes.is_empty()
            || self.outcomes.windows(2).any(|pair| {
                (pair[0].rule_id.as_ref(), pair[0].rule_version)
                    >= (pair[1].rule_id.as_ref(), pair[1].rule_version)
            })
        {
            return Err(matcher_error(
                RecognizerErrorCode::OutputPartitionInvalid,
                "recognizer output partition header or order is invalid",
            ));
        }
        for outcome in &self.outcomes {
            outcome.validate()?;
        }
        let expected = derive_partition_id(PartitionIdentity {
            context_id: self.context_id,
            source_pack_digest: &self.source_pack_digest,
            plan_id: &self.plan_id,
            fact_bundle_id: &self.fact_bundle_id,
            producer_partition_id: &self.producer_partition_id,
            outcomes: &self.outcomes,
        })?;
        if expected != self.partition_id {
            return Err(matcher_error(
                RecognizerErrorCode::OutputPartitionIdentityMismatch,
                "recognizer output partition identity does not match",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Binding {
    aliases: BTreeMap<Box<str>, RecognizerFactId>,
    // Includes branch-local facts so their evidence survives alias projection.
    witness_fact_ids: BTreeSet<RecognizerFactId>,
    // Union of facts participating in at least one fully exact proof path.
    exact_proof_fact_ids: BTreeSet<RecognizerFactId>,
    // Proof alternatives combine with OR; required fact clauses combine with AND.
    has_exact_proof: bool,
}

impl Default for Binding {
    fn default() -> Self {
        Self {
            aliases: BTreeMap::new(),
            witness_fact_ids: BTreeSet::new(),
            exact_proof_fact_ids: BTreeSet::new(),
            has_exact_proof: true,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SemanticOutputIdentity<'a> {
    Entity {
        output_id: &'a str,
        entity_kind_id: &'a str,
        semantic_key: BTreeMap<Box<str>, RecognizerFactValue>,
    },
    Relation {
        output_id: &'a str,
        relation_kind_id: &'a str,
        source: RecognizerFactValue,
        target: RecognizerFactValue,
    },
}

#[derive(Default)]
struct EvaluationBudget {
    expansions: u64,
    proof_copies: u64,
    truncated: bool,
}

struct ClauseExecution<'a> {
    bundle: &'a RecognizerFactBundle,
    plan: &'a CompiledRecognizerRulePlan,
    budget: &'a mut EvaluationBudget,
    cancelled: &'a AtomicBool,
    order_by_path: BTreeMap<Vec<u32>, usize>,
}

#[derive(Debug)]
enum EvaluationAbort {
    NotEvaluated(Vec<Box<str>>),
    Cancelled,
}

/// Executes one validated pack and semantic plan over one immutable fact bundle.
pub fn execute_recognizer_plan(
    context: &GenerationContext,
    pack: &CompiledRecognizerPack,
    plan: &CompiledRecognizerPlan,
    bundle: &RecognizerFactBundle,
    fact_limits: RecognizerFactLimits,
    cancelled: &AtomicBool,
) -> RecognizerResult<RecognizerOutputPartition> {
    pack.validate()?;
    plan.validate()?;
    bundle.validate(context, fact_limits)?;
    if plan.source_pack_digest() != pack.pack_digest() {
        return Err(matcher_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "recognizer plan was compiled from another pack",
        ));
    }
    let rules = &pack.document().pack.rules;
    if rules.len() != plan.rules().len()
        || rules.iter().zip(plan.rules()).any(|(rule, planned)| {
            rule.rule_id.as_ref() != planned.rule_id() || rule.version != planned.rule_version()
        })
    {
        return Err(matcher_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "recognizer pack and semantic plan rule sets differ",
        ));
    }

    let mut outcomes = Vec::with_capacity(rules.len());
    for (rule, planned) in rules.iter().zip(plan.rules()) {
        outcomes.push(evaluate_rule(
            rule,
            planned,
            pack.pack_digest(),
            plan.plan_id(),
            bundle,
            cancelled,
        )?);
    }
    outcomes.sort_by(|left, right| {
        (left.rule_id.as_ref(), left.rule_version)
            .cmp(&(right.rule_id.as_ref(), right.rule_version))
    });
    let partition_id = derive_partition_id(PartitionIdentity {
        context_id: bundle.context_id(),
        source_pack_digest: pack.pack_digest(),
        plan_id: plan.plan_id(),
        fact_bundle_id: bundle.bundle_id(),
        producer_partition_id: bundle.primary_partition_id(),
        outcomes: &outcomes,
    })?;
    let partition = RecognizerOutputPartition {
        schema: RECOGNIZER_OUTPUT_PARTITION_SCHEMA.into(),
        partition_id,
        context_id: bundle.context_id(),
        source_pack_digest: pack.pack_digest().into(),
        plan_id: plan.plan_id().clone(),
        fact_bundle_id: bundle.bundle_id().clone(),
        producer_partition_id: bundle.primary_partition_id().into(),
        outcomes,
    };
    partition.validate()?;
    Ok(partition)
}

fn evaluate_rule(
    rule: &RecognizerRule,
    plan: &CompiledRecognizerRulePlan,
    source_pack_digest: &str,
    plan_id: &RecognizerPlanId,
    bundle: &RecognizerFactBundle,
    cancelled: &AtomicBool,
) -> RecognizerResult<RecognizerRuleOutcome> {
    if cancelled.load(Ordering::Relaxed) {
        return Ok(terminal_outcome(
            rule,
            RecognizerRuleOutcomeState::Cancelled,
            vec!["matcher.cancelled".into()],
        ));
    }
    let coverage = rule_coverage(rule, bundle);
    if coverage.not_evaluated {
        return Ok(terminal_outcome(
            rule,
            RecognizerRuleOutcomeState::NotEvaluated,
            coverage.blockers,
        ));
    }

    let mut budget = EvaluationBudget::default();
    let order_by_path = plan
        .evaluation_order()
        .iter()
        .enumerate()
        .map(|(order, step)| (step.clause_path().to_vec(), order))
        .collect::<BTreeMap<_, _>>();
    let rows = {
        let mut execution = ClauseExecution {
            bundle,
            plan,
            budget: &mut budget,
            cancelled,
            order_by_path,
        };
        evaluate_all(&rule.clauses, vec![Binding::default()], &[], &mut execution)
    };
    let rows = match rows {
        Ok(rows) => rows,
        Err(EvaluationAbort::NotEvaluated(mut blockers)) => {
            blockers.extend(coverage.blockers);
            normalize(&mut blockers);
            let mut outcome = terminal_outcome(
                rule,
                if coverage.partial || budget.truncated {
                    RecognizerRuleOutcomeState::Partial
                } else {
                    RecognizerRuleOutcomeState::NotEvaluated
                },
                blockers,
            );
            outcome.truncated = budget.truncated;
            return Ok(outcome);
        }
        Err(EvaluationAbort::Cancelled) => {
            return Ok(terminal_outcome(
                rule,
                RecognizerRuleOutcomeState::Cancelled,
                vec!["matcher.cancelled".into()],
            ));
        }
    };

    let mut rows = canonical_rows(rows);
    let mut semantic_keys_by_row = Vec::with_capacity(rows.len());
    let mut semantic_key_counts = BTreeMap::<Vec<u8>, usize>::new();
    for binding in &rows {
        if cancelled.load(Ordering::Relaxed) {
            return Ok(terminal_outcome(
                rule,
                RecognizerRuleOutcomeState::Cancelled,
                vec!["matcher.cancelled".into()],
            ));
        }
        let Some(keys) = semantic_output_keys(rule, binding, bundle)? else {
            semantic_keys_by_row.push(None);
            continue;
        };
        for key in &keys {
            let count = semantic_key_counts.entry(key.clone()).or_default();
            *count = count.checked_add(1).ok_or_else(|| {
                matcher_error(
                    RecognizerErrorCode::MatchBudgetExceeded,
                    "semantic competition group count overflowed",
                )
            })?;
        }
        semantic_keys_by_row.push(Some(keys));
    }
    let ambiguous_rows = semantic_keys_by_row
        .iter()
        .map(|keys| {
            keys.as_ref().is_none_or(|keys| {
                keys.iter()
                    .any(|key| semantic_key_counts.get(key).is_some_and(|count| *count > 1))
            })
        })
        .collect::<Vec<_>>();
    let maximum_matches = usize::try_from(plan.bounds().max_matches).map_err(|_| {
        matcher_error(
            RecognizerErrorCode::MatchBudgetExceeded,
            "recognizer match limit does not fit usize",
        )
    })?;
    if rows.len() > maximum_matches {
        rows.truncate(maximum_matches);
        budget.truncated = true;
    }

    let mut matches = Vec::new();
    let mut proposals = Vec::new();
    let maximum_proposals = usize::try_from(plan.bounds().max_proposals).map_err(|_| {
        matcher_error(
            RecognizerErrorCode::ProposalBudgetExceeded,
            "recognizer proposal limit does not fit usize",
        )
    })?;
    let mut explanation_bytes = 0usize;
    for (row_index, binding) in rows.into_iter().enumerate() {
        if cancelled.load(Ordering::Relaxed) {
            return Ok(terminal_outcome(
                rule,
                RecognizerRuleOutcomeState::Cancelled,
                vec!["matcher.cancelled".into()],
            ));
        }
        let item = build_match(
            rule,
            source_pack_digest,
            plan_id,
            binding,
            bundle,
            &coverage.coverage_ids,
            coverage.partial || ambiguous_rows[row_index],
        )?;
        explanation_bytes = explanation_bytes
            .checked_add(
                canonical_json_bytes(&item)
                    .map_err(|_| {
                        matcher_error(
                            RecognizerErrorCode::MatchIdentityMismatch,
                            "recognizer match cannot be canonicalized",
                        )
                    })?
                    .len(),
            )
            .ok_or_else(|| {
                matcher_error(
                    RecognizerErrorCode::MatchBudgetExceeded,
                    "recognizer explanation byte count overflowed",
                )
            })?;
        if explanation_bytes > plan.bounds().max_explanation_bytes as usize {
            budget.truncated = true;
            break;
        }
        for output in &rule.outputs {
            if proposals.len() >= maximum_proposals {
                budget.truncated = true;
                break;
            }
            proposals.push(build_proposal(output, &item, bundle)?);
        }
        matches.push(item);
        if budget.truncated {
            break;
        }
    }
    matches.sort_by(|left, right| left.match_id.cmp(&right.match_id));
    proposals.sort_by(|left, right| left.proposal_id().cmp(right.proposal_id()));
    proposals.dedup_by(|left, right| left.proposal_id() == right.proposal_id());

    let mut blockers = coverage.blockers;
    if budget.truncated {
        blockers.push("matcher.output.truncated".into());
    }
    normalize(&mut blockers);
    let state = if budget.truncated || coverage.partial {
        RecognizerRuleOutcomeState::Partial
    } else if matches.is_empty() {
        RecognizerRuleOutcomeState::EvaluatedNoMatch
    } else {
        RecognizerRuleOutcomeState::Matched
    };
    let outcome = RecognizerRuleOutcome {
        rule_id: rule.rule_id.clone(),
        rule_version: rule.version,
        state,
        truncated: budget.truncated,
        blocker_ids: blockers,
        matches,
        proposals,
    };
    outcome.validate()?;
    Ok(outcome)
}

fn terminal_outcome(
    rule: &RecognizerRule,
    state: RecognizerRuleOutcomeState,
    mut blockers: Vec<Box<str>>,
) -> RecognizerRuleOutcome {
    normalize(&mut blockers);
    RecognizerRuleOutcome {
        rule_id: rule.rule_id.clone(),
        rule_version: rule.version,
        state,
        truncated: false,
        blocker_ids: blockers,
        matches: Vec::new(),
        proposals: Vec::new(),
    }
}

struct RuleCoverage {
    coverage_ids: Vec<CoverageId>,
    blockers: Vec<Box<str>>,
    partial: bool,
    not_evaluated: bool,
}

fn rule_coverage(rule: &RecognizerRule, bundle: &RecognizerFactBundle) -> RuleCoverage {
    let mut coverage_ids = Vec::new();
    let mut blockers = Vec::new();
    let mut partial = false;
    let mut not_evaluated = false;
    for capability in &rule.required_capabilities {
        let records = bundle
            .coverage()
            .iter()
            .filter(|record| record.capability_id() == capability.as_ref())
            .collect::<Vec<_>>();
        if records.is_empty() {
            blockers.push(format!("coverage.missing:{}", capability).into_boxed_str());
            not_evaluated = true;
            continue;
        }
        for record in records {
            coverage_ids.push(record.coverage_id());
            match record.state() {
                RecognizerFactCoverageState::Complete => {}
                RecognizerFactCoverageState::Partial | RecognizerFactCoverageState::Truncated => {
                    partial = true;
                    blockers.extend(record.blocker_ids().iter().cloned());
                }
                RecognizerFactCoverageState::NotEvaluated
                | RecognizerFactCoverageState::Failed
                | RecognizerFactCoverageState::Cancelled => {
                    not_evaluated = true;
                    blockers.extend(record.blocker_ids().iter().cloned());
                }
            }
        }
    }
    normalize(&mut coverage_ids);
    normalize(&mut blockers);
    RuleCoverage {
        coverage_ids,
        blockers,
        partial,
        not_evaluated,
    }
}

fn evaluate_all(
    clauses: &[RecognizerClause],
    rows: Vec<Binding>,
    scope_path: &[u32],
    execution: &mut ClauseExecution<'_>,
) -> Result<Vec<Binding>, EvaluationAbort> {
    let mut current = rows;
    for index in planned_clause_indices(clauses, scope_path, &execution.order_by_path)? {
        let clause_path = append_clause_path(scope_path, index)?;
        current = evaluate_clause(&clauses[index], current, &clause_path, execution)?;
        if current.is_empty() {
            break;
        }
    }
    Ok(current)
}

fn planned_clause_indices(
    clauses: &[RecognizerClause],
    scope_path: &[u32],
    order_by_path: &BTreeMap<Vec<u32>, usize>,
) -> Result<Vec<usize>, EvaluationAbort> {
    let mut ordered = Vec::with_capacity(clauses.len());
    for index in 0..clauses.len() {
        let path = append_clause_path(scope_path, index)?;
        let Some(order) = order_by_path.get(&path).copied() else {
            return Err(EvaluationAbort::NotEvaluated(vec![
                "matcher.plan.path.missing".into(),
            ]));
        };
        ordered.push((order, index));
    }
    ordered.sort_unstable();
    Ok(ordered.into_iter().map(|(_, index)| index).collect())
}

fn append_clause_path(scope_path: &[u32], index: usize) -> Result<Vec<u32>, EvaluationAbort> {
    let index = u32::try_from(index)
        .map_err(|_| EvaluationAbort::NotEvaluated(vec!["matcher.plan.path.invalid".into()]))?;
    let mut path = scope_path.to_vec();
    path.push(index);
    Ok(path)
}

fn evaluate_clause(
    clause: &RecognizerClause,
    rows: Vec<Binding>,
    clause_path: &[u32],
    execution: &mut ClauseExecution<'_>,
) -> Result<Vec<Binding>, EvaluationAbort> {
    if execution.cancelled.load(Ordering::Relaxed) {
        return Err(EvaluationAbort::Cancelled);
    }
    match clause {
        RecognizerClause::Fact { alias, kind } => {
            let facts = execution.bundle.facts_by_kind(kind).collect::<Vec<_>>();
            let mut output = Vec::new();
            for row in rows {
                if let Some(bound) = row.aliases.get(alias) {
                    if facts.iter().any(|fact| fact.fact_id() == bound) {
                        output.push(row);
                    }
                    continue;
                }
                for fact in &facts {
                    consume_expansion(execution.budget, execution.plan)?;
                    consume_binding_copy(&row, execution)?;
                    let mut expanded = row.clone();
                    let fact_id = fact.fact_id().clone();
                    expanded.aliases.insert(alias.clone(), fact_id.clone());
                    expanded.witness_fact_ids.insert(fact_id.clone());
                    if matches!(
                        fact.confidence(),
                        GraphConfidence::Possible | GraphConfidence::Candidate
                    ) {
                        expanded.has_exact_proof = false;
                        expanded.exact_proof_fact_ids.clear();
                    } else if expanded.has_exact_proof {
                        expanded.exact_proof_fact_ids.insert(fact_id);
                    }
                    output.push(expanded);
                }
            }
            Ok(canonical_rows(output))
        }
        RecognizerClause::Join { left, right } => Ok(filter_rows(rows, |row| {
            resolve_field(row, execution.bundle, left)
                == resolve_field(row, execution.bundle, right)
                && resolve_field(row, execution.bundle, left).is_some()
        })),
        RecognizerClause::FieldEq { field, value } => {
            let expected = literal(value);
            Ok(filter_rows(rows, |row| {
                resolve_field(row, execution.bundle, field) == Some(&expected)
            }))
        }
        RecognizerClause::FieldIn { field, values } => {
            let expected = values.iter().map(literal).collect::<BTreeSet<_>>();
            Ok(filter_rows(rows, |row| {
                resolve_field(row, execution.bundle, field)
                    .is_some_and(|value| expected.contains(value))
            }))
        }
        RecognizerClause::SameScope { left, right, scope } => Ok(filter_rows(rows, |row| {
            same_scope(row, execution.bundle, left, right, scope)
        })),
        RecognizerClause::Exists { clauses } => evaluate_all(clauses, rows, clause_path, execution),
        RecognizerClause::NotExists {
            clauses,
            required_complete_capability,
        } => {
            let records = execution
                .bundle
                .coverage()
                .iter()
                .filter(|record| record.capability_id() == required_complete_capability.as_ref())
                .collect::<Vec<_>>();
            if records.is_empty()
                || records
                    .iter()
                    .any(|record| record.state() != RecognizerFactCoverageState::Complete)
            {
                return Err(EvaluationAbort::NotEvaluated(vec![
                    format!("coverage.incomplete:{}", required_complete_capability)
                        .into_boxed_str(),
                ]));
            }
            let mut output = Vec::new();
            for row in rows {
                consume_binding_copy(&row, execution)?;
                let matches = evaluate_all(clauses, vec![row.clone()], clause_path, execution)?;
                if matches.is_empty() {
                    output.push(row);
                }
            }
            Ok(output)
        }
        RecognizerClause::OrderedRelation {
            left,
            right,
            relation,
        } => Ok(filter_rows(rows, |row| {
            ordered_relation(row, execution.bundle, left, right, relation)
        })),
        RecognizerClause::ControlFlowRelation {
            left,
            right,
            relation,
        } => Ok(filter_rows(rows, |row| {
            supplied_relation(row, execution.bundle, left, right, relation)
        })),
        RecognizerClause::AllOf { clauses } => evaluate_all(clauses, rows, clause_path, execution),
        RecognizerClause::AnyOf { clauses } => {
            let mut inherited_aliases = rows
                .first()
                .map(|row| row.aliases.keys().cloned().collect::<BTreeSet<_>>())
                .unwrap_or_default();
            for row in rows.iter().skip(1) {
                inherited_aliases.retain(|alias| row.aliases.contains_key(alias));
            }
            let mut escaping_aliases = common_clause_output_aliases(clauses);
            escaping_aliases.extend(inherited_aliases);
            let mut output = Vec::new();
            for index in planned_clause_indices(clauses, clause_path, &execution.order_by_path)? {
                let branch_path = append_clause_path(clause_path, index)?;
                for row in &rows {
                    consume_binding_copy(row, execution)?;
                }
                let mut branch_rows =
                    evaluate_clause(&clauses[index], rows.clone(), &branch_path, execution)?;
                for branch_row in &mut branch_rows {
                    branch_row
                        .aliases
                        .retain(|alias, _| escaping_aliases.contains(alias));
                }
                output.extend(branch_rows);
            }
            Ok(canonical_rows(output))
        }
    }
}

fn consume_expansion(
    budget: &mut EvaluationBudget,
    plan: &CompiledRecognizerRulePlan,
) -> Result<(), EvaluationAbort> {
    budget.expansions = budget.expansions.saturating_add(1);
    if budget.expansions > plan.bounds().max_join_expansions {
        budget.truncated = true;
        Err(EvaluationAbort::NotEvaluated(vec![
            "matcher.join-expansion.truncated".into(),
        ]))
    } else {
        Ok(())
    }
}

fn consume_binding_copy(
    row: &Binding,
    execution: &mut ClauseExecution<'_>,
) -> Result<(), EvaluationAbort> {
    if execution.cancelled.load(Ordering::Relaxed) {
        return Err(EvaluationAbort::Cancelled);
    }
    // A projected row can retain many branch-local witnesses. Bound those
    // copies before cloning, independently of the number of expanded rows.
    let copies = row.aliases.len() as u64
        + row.witness_fact_ids.len() as u64
        + row.exact_proof_fact_ids.len() as u64;
    execution.budget.proof_copies = execution.budget.proof_copies.saturating_add(copies);
    if execution.budget.proof_copies > execution.plan.bounds().max_join_expansions {
        execution.budget.truncated = true;
        return Err(EvaluationAbort::NotEvaluated(vec![
            "matcher.proof-copy.truncated".into(),
        ]));
    }
    Ok(())
}

fn canonical_rows(mut rows: Vec<Binding>) -> Vec<Binding> {
    rows.sort_by(|left, right| left.aliases.cmp(&right.aliases));
    let mut canonical = Vec::<Binding>::with_capacity(rows.len());
    for row in rows {
        match canonical.last_mut() {
            Some(existing) if existing.aliases == row.aliases => {
                // One visible environment may have several corroborating proof paths.
                existing.witness_fact_ids.extend(row.witness_fact_ids);
                if row.has_exact_proof {
                    existing.has_exact_proof = true;
                    existing
                        .exact_proof_fact_ids
                        .extend(row.exact_proof_fact_ids);
                }
            }
            _ => canonical.push(row),
        }
    }
    canonical
}

fn common_clause_output_aliases(clauses: &[RecognizerClause]) -> BTreeSet<Box<str>> {
    let Some((first, rest)) = clauses.split_first() else {
        return BTreeSet::new();
    };
    let mut common = clause_output_aliases(first);
    for clause in rest {
        let output = clause_output_aliases(clause);
        common.retain(|alias| output.contains(alias));
    }
    common
}

fn clause_output_aliases(clause: &RecognizerClause) -> BTreeSet<Box<str>> {
    match clause {
        RecognizerClause::Fact { alias, .. } => BTreeSet::from([alias.clone()]),
        RecognizerClause::Exists { clauses } | RecognizerClause::AllOf { clauses } => {
            clauses.iter().flat_map(clause_output_aliases).collect()
        }
        RecognizerClause::AnyOf { clauses } => common_clause_output_aliases(clauses),
        RecognizerClause::NotExists { .. }
        | RecognizerClause::Join { .. }
        | RecognizerClause::FieldEq { .. }
        | RecognizerClause::FieldIn { .. }
        | RecognizerClause::SameScope { .. }
        | RecognizerClause::OrderedRelation { .. }
        | RecognizerClause::ControlFlowRelation { .. } => BTreeSet::new(),
    }
}

fn filter_rows(rows: Vec<Binding>, predicate: impl Fn(&Binding) -> bool) -> Vec<Binding> {
    rows.into_iter().filter(predicate).collect()
}

fn resolve_field<'a>(
    row: &'a Binding,
    bundle: &'a RecognizerFactBundle,
    reference: &str,
) -> Option<&'a RecognizerFactValue> {
    let (alias, field) = reference.split_once('.')?;
    let fact_id = row.aliases.get(alias)?;
    bundle.fact_by_id(fact_id)?.field(field)
}

fn fact_for_reference<'a>(
    row: &'a Binding,
    bundle: &'a RecognizerFactBundle,
    reference: &str,
) -> Option<&'a RecognizerFact> {
    let (alias, _) = reference.split_once('.')?;
    bundle.fact_by_id(row.aliases.get(alias)?)
}

fn same_scope(
    row: &Binding,
    bundle: &RecognizerFactBundle,
    left: &str,
    right: &str,
    scope: &str,
) -> bool {
    let Some(left) = fact_for_reference(row, bundle, left) else {
        return false;
    };
    let Some(right) = fact_for_reference(row, bundle, right) else {
        return false;
    };
    let expected = match scope {
        "partition" => RecognizerFactScopeKind::Partition,
        "file" => RecognizerFactScopeKind::File,
        "function" => RecognizerFactScopeKind::Function,
        "package" => RecognizerFactScopeKind::Package,
        "xml_document" => RecognizerFactScopeKind::XmlDocument,
        "load_unit" => RecognizerFactScopeKind::LoadUnit,
        _ => return false,
    };
    left.scope().kind() == expected
        && right.scope().kind() == expected
        && left.scope().id() == right.scope().id()
}

fn ordered_relation(
    row: &Binding,
    bundle: &RecognizerFactBundle,
    left: &str,
    right: &str,
    relation: &str,
) -> bool {
    let Some(RecognizerFactValue::Integer(left)) = resolve_field(row, bundle, left) else {
        return false;
    };
    let Some(RecognizerFactValue::Integer(right)) = resolve_field(row, bundle, right) else {
        return false;
    };
    match relation {
        "before" => left < right,
        "after" => left > right,
        "equal" => left == right,
        _ => false,
    }
}

fn supplied_relation(
    row: &Binding,
    bundle: &RecognizerFactBundle,
    left: &str,
    right: &str,
    relation: &str,
) -> bool {
    let Some(left) = resolve_field(row, bundle, left) else {
        return false;
    };
    let Some(right) = resolve_field(row, bundle, right) else {
        return false;
    };
    bundle.facts().iter().any(|fact| {
        matches!(
            fact.kind(),
            "lua.control_flow" | "emmy.control_flow_relation"
        ) && fact.field("left") == Some(left)
            && fact.field("right") == Some(right)
            && fact
                .field("relation")
                .is_some_and(|value| fact_text(value) == Some(relation))
    })
}

fn literal(value: &crate::RecognizerPackLiteral) -> RecognizerFactValue {
    match value {
        crate::RecognizerPackLiteral::Boolean(value) => RecognizerFactValue::Boolean(*value),
        crate::RecognizerPackLiteral::Integer(value) => RecognizerFactValue::Integer(*value),
        crate::RecognizerPackLiteral::String(value) => RecognizerFactValue::String(value.clone()),
    }
}

fn build_match(
    rule: &RecognizerRule,
    source_pack_digest: &str,
    plan_id: &RecognizerPlanId,
    binding: Binding,
    bundle: &RecognizerFactBundle,
    coverage_ids: &[CoverageId],
    force_possible: bool,
) -> RecognizerResult<RecognizerMatch> {
    let captures = captures_for_binding(rule, &binding, bundle)?;
    let mut decisive_fact_ids = if binding.has_exact_proof {
        binding
            .exact_proof_fact_ids
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    } else {
        binding.witness_fact_ids.iter().cloned().collect::<Vec<_>>()
    };
    normalize(&mut decisive_fact_ids);
    let mut source_handle_ids = Vec::new();
    let mut evidence_ids = Vec::new();
    for fact_id in &binding.witness_fact_ids {
        let fact = bundle.fact_by_id(fact_id).ok_or_else(|| {
            matcher_error(
                RecognizerErrorCode::MatchInvalid,
                "recognizer match references an unknown fact",
            )
        })?;
        source_handle_ids.extend_from_slice(fact.source_handle_ids());
        evidence_ids.extend_from_slice(fact.evidence_ids());
    }
    normalize(&mut source_handle_ids);
    normalize(&mut evidence_ids);
    let mut coverage_ids = coverage_ids.to_vec();
    normalize(&mut coverage_ids);
    // An exact proof path is sufficient for an AnyOf even if another branch is Possible.
    let confidence = if force_possible || !binding.has_exact_proof {
        RecognizerOutputConfidence::Possible
    } else {
        RecognizerOutputConfidence::Derived
    };
    let match_id = derive_match_id(MatchIdentity {
        source_pack_digest,
        plan_id,
        fact_bundle_id: bundle.bundle_id(),
        rule_id: &rule.rule_id,
        rule_version: rule.version,
        partition_id: bundle.primary_partition_id(),
        bindings: &binding.aliases,
        captures: &captures,
        decisive_fact_ids: &decisive_fact_ids,
        source_handle_ids: &source_handle_ids,
        evidence_ids: &evidence_ids,
        coverage_ids: &coverage_ids,
        confidence,
    })?;
    let item = RecognizerMatch {
        match_id,
        source_pack_digest: source_pack_digest.into(),
        plan_id: plan_id.clone(),
        fact_bundle_id: bundle.bundle_id().clone(),
        rule_id: rule.rule_id.clone(),
        rule_version: rule.version,
        partition_id: bundle.primary_partition_id().into(),
        bindings: binding.aliases,
        captures,
        decisive_fact_ids,
        source_handle_ids,
        evidence_ids,
        coverage_ids,
        confidence,
    };
    item.validate()?;
    Ok(item)
}

fn semantic_output_keys(
    rule: &RecognizerRule,
    binding: &Binding,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<Option<BTreeSet<Vec<u8>>>> {
    let captures = match captures_for_binding(rule, binding, bundle) {
        Ok(captures) => captures,
        Err(error) if error.code() == RecognizerErrorCode::MatchInvalid => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut keys = BTreeSet::new();
    for output in &rule.outputs {
        let identity = match output {
            RecognizerOutput::EntityAssertion {
                output_id,
                entity_kind_id,
                semantic_key,
                ..
            } => {
                let mut resolved = BTreeMap::new();
                for (key, reference) in semantic_key {
                    let value = match resolve_output_from_bindings(
                        reference,
                        &captures,
                        &binding.aliases,
                        bundle,
                    ) {
                        Ok(value) => value,
                        Err(error) if error.code() == RecognizerErrorCode::ProposalInvalid => {
                            return Ok(None);
                        }
                        Err(error) => return Err(error),
                    };
                    resolved.insert(key.clone(), value);
                }
                SemanticOutputIdentity::Entity {
                    output_id,
                    entity_kind_id,
                    semantic_key: resolved,
                }
            }
            RecognizerOutput::RelationAssertion {
                output_id,
                relation_kind_id,
                source,
                target,
                ..
            } => {
                let source =
                    match resolve_output_from_bindings(source, &captures, &binding.aliases, bundle)
                    {
                        Ok(value) => value,
                        Err(error) if error.code() == RecognizerErrorCode::ProposalInvalid => {
                            return Ok(None);
                        }
                        Err(error) => return Err(error),
                    };
                let target =
                    match resolve_output_from_bindings(target, &captures, &binding.aliases, bundle)
                    {
                        Ok(value) => value,
                        Err(error) if error.code() == RecognizerErrorCode::ProposalInvalid => {
                            return Ok(None);
                        }
                        Err(error) => return Err(error),
                    };
                SemanticOutputIdentity::Relation {
                    output_id,
                    relation_kind_id,
                    source,
                    target,
                }
            }
        };
        let bytes = canonical_json_bytes(&identity).map_err(|_| {
            matcher_error(
                RecognizerErrorCode::MatchIdentityMismatch,
                "semantic output identity cannot be canonicalized",
            )
        })?;
        keys.insert(bytes);
    }
    Ok(Some(keys))
}

fn captures_for_binding(
    rule: &RecognizerRule,
    binding: &Binding,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<BTreeMap<Box<str>, RecognizerCapturedValue>> {
    let mut captures = BTreeMap::new();
    for capture in &rule.captures {
        if let Some(value) = capture_value(capture, binding, bundle)? {
            captures.insert(capture.name.clone(), value);
        }
    }
    Ok(captures)
}

fn capture_value(
    capture: &RecognizerCapture,
    binding: &Binding,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<Option<RecognizerCapturedValue>> {
    let value = resolve_field(binding, bundle, &capture.source).cloned();
    match capture.cardinality {
        RecognizerCaptureCardinality::One => value
            .map(RecognizerCapturedValue::One)
            .map(Some)
            .ok_or_else(|| {
                matcher_error(
                    RecognizerErrorCode::MatchInvalid,
                    "required recognizer capture is unavailable",
                )
            }),
        RecognizerCaptureCardinality::Optional => {
            Ok(Some(RecognizerCapturedValue::Optional(value)))
        }
        RecognizerCaptureCardinality::BoundedMany => Ok(Some(
            RecognizerCapturedValue::BoundedMany(value.into_iter().collect()),
        )),
    }
}

fn build_proposal(
    output: &RecognizerOutput,
    item: &RecognizerMatch,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<RecognizerProposedAssertion> {
    let confidence = effective_confidence(requested_confidence(output), item.confidence);
    let proposal = match output {
        RecognizerOutput::EntityAssertion {
            output_id,
            entity_kind_id,
            semantic_key,
            ..
        } => {
            let semantic_key = semantic_key
                .iter()
                .map(|(key, value)| Ok((key.clone(), resolve_output_value(value, item, bundle)?)))
                .collect::<RecognizerResult<BTreeMap<_, _>>>()?;
            let proposal_id = derive_proposal_id(&ProposalIdentity::Entity {
                output_id,
                entity_kind_id,
                semantic_key: &semantic_key,
                confidence,
                match_id: &item.match_id,
                decisive_fact_ids: &item.decisive_fact_ids,
                source_handle_ids: &item.source_handle_ids,
                evidence_ids: &item.evidence_ids,
                coverage_ids: &item.coverage_ids,
            })?;
            RecognizerProposedAssertion::Entity {
                proposal_id,
                output_id: output_id.clone(),
                entity_kind_id: entity_kind_id.clone(),
                semantic_key,
                confidence,
                match_id: item.match_id.clone(),
                decisive_fact_ids: item.decisive_fact_ids.clone(),
                source_handle_ids: item.source_handle_ids.clone(),
                evidence_ids: item.evidence_ids.clone(),
                coverage_ids: item.coverage_ids.clone(),
            }
        }
        RecognizerOutput::RelationAssertion {
            output_id,
            relation_kind_id,
            source,
            target,
            ..
        } => {
            let source = resolve_output_value(source, item, bundle)?;
            let target = resolve_output_value(target, item, bundle)?;
            let proposal_id = derive_proposal_id(&ProposalIdentity::Relation {
                output_id,
                relation_kind_id,
                source: &source,
                target: &target,
                confidence,
                match_id: &item.match_id,
                decisive_fact_ids: &item.decisive_fact_ids,
                source_handle_ids: &item.source_handle_ids,
                evidence_ids: &item.evidence_ids,
                coverage_ids: &item.coverage_ids,
            })?;
            RecognizerProposedAssertion::Relation {
                proposal_id,
                output_id: output_id.clone(),
                relation_kind_id: relation_kind_id.clone(),
                source,
                target,
                confidence,
                match_id: item.match_id.clone(),
                decisive_fact_ids: item.decisive_fact_ids.clone(),
                source_handle_ids: item.source_handle_ids.clone(),
                evidence_ids: item.evidence_ids.clone(),
                coverage_ids: item.coverage_ids.clone(),
            }
        }
    };
    proposal.validate()?;
    Ok(proposal)
}

fn requested_confidence(output: &RecognizerOutput) -> RecognizerOutputConfidence {
    match output {
        RecognizerOutput::EntityAssertion { confidence, .. }
        | RecognizerOutput::RelationAssertion { confidence, .. } => *confidence,
    }
}

fn effective_confidence(
    requested: RecognizerOutputConfidence,
    observed: RecognizerOutputConfidence,
) -> RecognizerOutputConfidence {
    if requested == RecognizerOutputConfidence::Possible
        || observed == RecognizerOutputConfidence::Possible
    {
        RecognizerOutputConfidence::Possible
    } else {
        RecognizerOutputConfidence::Derived
    }
}

fn resolve_output_value(
    reference: &str,
    item: &RecognizerMatch,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<RecognizerFactValue> {
    resolve_output_from_bindings(reference, &item.captures, &item.bindings, bundle)
}

fn resolve_output_from_bindings(
    reference: &str,
    captures: &BTreeMap<Box<str>, RecognizerCapturedValue>,
    bindings: &BTreeMap<Box<str>, RecognizerFactId>,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<RecognizerFactValue> {
    if let Some(value) = captures.get(reference) {
        return match value {
            RecognizerCapturedValue::One(value)
            | RecognizerCapturedValue::Optional(Some(value)) => Ok(value.clone()),
            RecognizerCapturedValue::BoundedMany(values) if values.len() == 1 => {
                Ok(values[0].clone())
            }
            RecognizerCapturedValue::Optional(None) | RecognizerCapturedValue::BoundedMany(_) => {
                Err(matcher_error(
                    RecognizerErrorCode::ProposalInvalid,
                    "recognizer output requires a single available capture value",
                ))
            }
        };
    }
    let (alias, field) = reference.split_once('.').ok_or_else(|| {
        matcher_error(
            RecognizerErrorCode::ProposalInvalid,
            "recognizer output reference is not a capture or alias.field",
        )
    })?;
    let fact_id = bindings.get(alias).ok_or_else(|| {
        matcher_error(
            RecognizerErrorCode::ProposalInvalid,
            "recognizer output alias is not bound",
        )
    })?;
    bundle
        .fact_by_id(fact_id)
        .and_then(|fact| fact.field(field))
        .cloned()
        .ok_or_else(|| {
            matcher_error(
                RecognizerErrorCode::ProposalInvalid,
                "recognizer output fact field is unavailable",
            )
        })
}

fn fact_text(value: &RecognizerFactValue) -> Option<&str> {
    match value {
        RecognizerFactValue::String(value)
        | RecognizerFactValue::Identifier(value)
        | RecognizerFactValue::Tag(value)
        | RecognizerFactValue::Reference(value) => Some(value),
        _ => None,
    }
}

#[derive(Serialize)]
struct MatchIdentity<'a> {
    source_pack_digest: &'a str,
    plan_id: &'a RecognizerPlanId,
    fact_bundle_id: &'a crate::RecognizerFactBundleId,
    rule_id: &'a str,
    rule_version: u32,
    partition_id: &'a str,
    bindings: &'a BTreeMap<Box<str>, RecognizerFactId>,
    captures: &'a BTreeMap<Box<str>, RecognizerCapturedValue>,
    decisive_fact_ids: &'a [RecognizerFactId],
    source_handle_ids: &'a [StableHandleId],
    evidence_ids: &'a [EvidenceId],
    coverage_ids: &'a [CoverageId],
    confidence: RecognizerOutputConfidence,
}

fn derive_match_id(identity: MatchIdentity<'_>) -> RecognizerResult<RecognizerMatchId> {
    let bytes = canonical_json_bytes(&identity).map_err(|_| {
        matcher_error(
            RecognizerErrorCode::MatchIdentityMismatch,
            "recognizer match identity cannot be canonicalized",
        )
    })?;
    RecognizerMatchId::new(format!(
        "recognizer-match:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProposalIdentity<'a> {
    Entity {
        output_id: &'a str,
        entity_kind_id: &'a str,
        semantic_key: &'a BTreeMap<Box<str>, RecognizerFactValue>,
        confidence: RecognizerOutputConfidence,
        match_id: &'a RecognizerMatchId,
        decisive_fact_ids: &'a [RecognizerFactId],
        source_handle_ids: &'a [StableHandleId],
        evidence_ids: &'a [EvidenceId],
        coverage_ids: &'a [CoverageId],
    },
    Relation {
        output_id: &'a str,
        relation_kind_id: &'a str,
        source: &'a RecognizerFactValue,
        target: &'a RecognizerFactValue,
        confidence: RecognizerOutputConfidence,
        match_id: &'a RecognizerMatchId,
        decisive_fact_ids: &'a [RecognizerFactId],
        source_handle_ids: &'a [StableHandleId],
        evidence_ids: &'a [EvidenceId],
        coverage_ids: &'a [CoverageId],
    },
}

fn derive_proposal_id(identity: &ProposalIdentity<'_>) -> RecognizerResult<RecognizerProposalId> {
    let bytes = canonical_json_bytes(identity).map_err(|_| {
        matcher_error(
            RecognizerErrorCode::ProposalIdentityMismatch,
            "recognizer proposal identity cannot be canonicalized",
        )
    })?;
    RecognizerProposalId::new(format!(
        "recognizer-proposal:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

#[derive(Serialize)]
struct PartitionIdentity<'a> {
    context_id: GenerationContextId,
    source_pack_digest: &'a str,
    plan_id: &'a RecognizerPlanId,
    fact_bundle_id: &'a crate::RecognizerFactBundleId,
    producer_partition_id: &'a str,
    outcomes: &'a [RecognizerRuleOutcome],
}

fn derive_partition_id(
    identity: PartitionIdentity<'_>,
) -> RecognizerResult<RecognizerOutputPartitionId> {
    let bytes = canonical_json_bytes(&identity).map_err(|_| {
        matcher_error(
            RecognizerErrorCode::OutputPartitionIdentityMismatch,
            "recognizer output partition identity cannot be canonicalized",
        )
    })?;
    RecognizerOutputPartitionId::new(format!(
        "recognizer-output-partition:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

fn normalize<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn matcher_error(code: RecognizerErrorCode, message: &'static str) -> RecognizerError {
    RecognizerError::new(code, message)
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
