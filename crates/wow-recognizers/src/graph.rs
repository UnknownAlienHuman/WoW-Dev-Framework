//! Bounded handoff from the typed E2-B matcher into the graph proposal owner.
//!
//! This module never resolves semantic names or publishes graph state. Relation
//! endpoints must be supplied as exact existing graph node IDs by the caller;
//! `wow-graph` revalidates the entire proposal batch against that exact input.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};
use wow_core::{GenerationContext, canonical_json_bytes};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEntityProposal, GraphError,
    GraphLimits, GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalValidationReport, GraphProposalValue, GraphRegistryBundle, GraphRelationKind,
    GraphRelationProposal, GraphRelationProposalInput, GraphSnapshot,
    validate_graph_proposal_batch,
};

use crate::{
    CompiledRecognizerPack, CompiledRecognizerPlan, RecognizerError, RecognizerErrorCode,
    RecognizerFactBundle, RecognizerFactLimits, RecognizerFactValue, RecognizerMatchId,
    RecognizerOutputConfidence, RecognizerOutputPartition, RecognizerProposalId,
    RecognizerProposedAssertion, RecognizerResult, RecognizerRuleOutcomeState,
};

// Keep the adapter's preflight no larger than wow-graph's current immutable
// proposal-batch ceiling. The graph owner independently enforces its own bound.
const MAX_GRAPH_PROPOSALS: usize = 200_000;
const MAX_GRAPH_ENDPOINT_BINDINGS: usize = MAX_GRAPH_PROPOSALS * 2;
const MAX_GRAPH_ENDPOINT_BINDING_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognizerGraphHandoffDisposition {
    /// The matcher output is complete and every graph proposal was accepted.
    Ready,
    /// Partial or unevaluated matcher evidence is retained without a complete batch.
    Deferred,
    /// Cancellation occurred; the matcher output remains attached to the result.
    Cancelled,
    /// Inputs or proposal construction failed after matcher output was produced.
    Rejected,
    /// Graph returned explicit proposal rejections; no publication batch is exposed.
    GraphRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecognizerGraphEndpointSide {
    Source,
    Target,
}

/// A caller-resolved relation endpoint. No name, semantic-key, or source lookup
/// is performed by the recognizer adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognizerGraphEndpointBinding {
    relation_kind_id: Box<str>,
    side: RecognizerGraphEndpointSide,
    value: RecognizerFactValue,
    match_id: Option<RecognizerMatchId>,
    endpoint: GraphProposalEndpoint,
}

impl RecognizerGraphEndpointBinding {
    pub fn existing(
        relation_kind_id: impl Into<Box<str>>,
        side: RecognizerGraphEndpointSide,
        value: RecognizerFactValue,
        node_id: wow_graph::GraphNodeId,
    ) -> RecognizerResult<Self> {
        let relation_kind_id = relation_kind_id.into();
        if relation_kind_id.is_empty()
            || relation_kind_id.len() > 256
            || relation_kind_id.chars().any(char::is_control)
        {
            return Err(handoff_error(
                RecognizerErrorCode::AdapterBindingInvalid,
                "graph endpoint binding has an invalid relation kind",
            ));
        }
        Ok(Self {
            relation_kind_id,
            side,
            value,
            match_id: None,
            endpoint: GraphProposalEndpoint::Existing(node_id),
        })
    }

    /// Binds one match-scoped endpoint to an entity proposal in the same output.
    /// The graph owner validates that the proposal exists and has an admitted kind.
    pub fn proposed(
        relation_kind_id: impl Into<Box<str>>,
        side: RecognizerGraphEndpointSide,
        value: RecognizerFactValue,
        match_id: RecognizerMatchId,
        proposal_id: RecognizerProposalId,
    ) -> RecognizerResult<Self> {
        let relation_kind_id = relation_kind_id.into();
        if relation_kind_id.is_empty()
            || relation_kind_id.len() > 256
            || relation_kind_id.chars().any(char::is_control)
        {
            return Err(handoff_error(
                RecognizerErrorCode::AdapterBindingInvalid,
                "graph endpoint binding has an invalid relation kind",
            ));
        }
        Ok(Self {
            relation_kind_id,
            side,
            value,
            match_id: Some(match_id),
            endpoint: GraphProposalEndpoint::Proposed(proposal_id.as_str().into()),
        })
    }
}

/// Exact inputs for one matcher-to-graph operation. The graph snapshot and
/// endpoint bindings are supplied by the caller; this request owns no publisher.
pub struct RecognizerGraphHandoffRequest<'a> {
    pub context: &'a GenerationContext,
    pub pack: &'a CompiledRecognizerPack,
    pub plan: &'a CompiledRecognizerPlan,
    pub facts: &'a RecognizerFactBundle,
    pub output: &'a RecognizerOutputPartition,
    pub fact_limits: RecognizerFactLimits,
    pub graph: &'a GraphPartitionSnapshot,
    pub graph_limits: GraphLimits,
    pub endpoint_bindings: &'a [RecognizerGraphEndpointBinding],
    pub cancelled: &'a AtomicBool,
}

/// The exact matcher output stays attached to every disposition, including
/// deferred and rejected outcomes. Only `Ready` exposes a publishable batch.
#[derive(Debug)]
pub struct RecognizerGraphHandoff<'a> {
    disposition: RecognizerGraphHandoffDisposition,
    output: &'a RecognizerOutputPartition,
    producer_partition_id: Box<str>,
    producer_version: Box<str>,
    batch: Option<GraphProposalBatch>,
    validation: Option<GraphProposalValidationReport>,
    coverage: Option<Vec<GraphCoverageRecord>>,
    failure: Option<RecognizerError>,
}

/// Move-only payload for the caller that will prepare the graph replacement.
/// Taking these parts avoids cloning a potentially large proposal batch.
pub struct RecognizerGraphHandoffParts<'a> {
    pub source_output: &'a RecognizerOutputPartition,
    pub producer_partition_id: Box<str>,
    pub producer_version: Box<str>,
    pub batch: GraphProposalBatch,
    pub validation_report: GraphProposalValidationReport,
    pub coverage_records: Vec<GraphCoverageRecord>,
}

impl<'a> RecognizerGraphHandoff<'a> {
    #[must_use]
    pub const fn disposition(&self) -> RecognizerGraphHandoffDisposition {
        self.disposition
    }

    #[must_use]
    pub fn output(&self) -> &RecognizerOutputPartition {
        self.output
    }

    #[must_use]
    pub fn producer_partition_id(&self) -> &str {
        &self.producer_partition_id
    }

    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }

    #[must_use]
    pub fn batch(&self) -> Option<&GraphProposalBatch> {
        self.batch.as_ref()
    }

    #[must_use]
    pub fn validation_report(&self) -> Option<&GraphProposalValidationReport> {
        self.validation.as_ref()
    }

    #[must_use]
    pub fn coverage_records(&self) -> Option<&[GraphCoverageRecord]> {
        self.coverage.as_deref()
    }

    #[must_use]
    pub fn failure(&self) -> Option<&RecognizerError> {
        self.failure.as_ref()
    }

    #[must_use]
    pub fn publication_ready(&self) -> bool {
        self.disposition == RecognizerGraphHandoffDisposition::Ready
            && self.batch.is_some()
            && self
                .validation
                .as_ref()
                .is_some_and(GraphProposalValidationReport::ready_for_publication)
            && self.coverage.is_some()
            && self.failure.is_none()
    }

    /// Consume a ready handoff without cloning its batch. Non-ready evidence is
    /// returned intact so callers cannot accidentally publish deferred output.
    pub fn into_ready_parts(self) -> Result<RecognizerGraphHandoffParts<'a>, Box<Self>> {
        if !self.publication_ready() {
            return Err(Box::new(self));
        }
        let Self {
            disposition,
            output,
            producer_partition_id,
            producer_version,
            batch,
            validation,
            coverage,
            failure,
        } = self;
        match (batch, validation, coverage) {
            (Some(batch), Some(validation_report), Some(coverage_records))
                if disposition == RecognizerGraphHandoffDisposition::Ready && failure.is_none() =>
            {
                Ok(RecognizerGraphHandoffParts {
                    source_output: output,
                    producer_partition_id,
                    producer_version,
                    batch,
                    validation_report,
                    coverage_records,
                })
            }
            (batch, validation, coverage) => Err(Box::new(Self {
                disposition,
                output,
                producer_partition_id,
                producer_version,
                batch,
                validation,
                coverage,
                failure,
            })),
        }
    }
}

/// Adapts one existing matcher output after binding its four producer identities
/// to the supplied pack, plan, fact bundle, and primary input partition. Only a
/// complete output can yield a batch; every result retains the original output,
/// so partial/unknown evidence is never turned into a complete replacement.
pub fn adapt_recognizer_output_to_graph<'a>(
    request: RecognizerGraphHandoffRequest<'a>,
) -> RecognizerResult<RecognizerGraphHandoff<'a>> {
    let RecognizerGraphHandoffRequest {
        context,
        pack,
        plan,
        facts,
        output,
        fact_limits,
        graph,
        graph_limits,
        endpoint_bindings,
        cancelled,
    } = request;

    pack.validate()?;
    plan.validate()?;
    facts.validate(context, fact_limits)?;
    output.validate()?;
    if plan.source_pack_digest() != pack.pack_digest()
        || output.source_pack_digest() != pack.pack_digest()
        || output.plan_id() != plan.plan_id()
        || output.fact_bundle_id() != facts.bundle_id()
        || output.producer_partition_id() != facts.primary_partition_id()
        || output.context_id() != context.context_id()
        || facts.context_id() != context.context_id()
    {
        return Err(handoff_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "matcher output is not bound to the supplied pack, plan, facts, and context",
        ));
    }
    if plan.rules().len() != pack.document().pack.rules.len()
        || output.outcomes().len() != pack.document().pack.rules.len()
    {
        return Err(handoff_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "matcher output rule set differs from the exact compiled pack",
        ));
    }
    for rule in &pack.document().pack.rules {
        let expected = (rule.rule_id.as_ref(), rule.version);
        if !plan
            .rules()
            .iter()
            .any(|planned| (planned.rule_id(), planned.rule_version()) == expected)
            || !output
                .outcomes()
                .iter()
                .any(|outcome| (outcome.rule_id(), outcome.rule_version()) == expected)
        {
            return Err(handoff_error(
                RecognizerErrorCode::MatcherInputMismatch,
                "matcher output contains another rule identity",
            ));
        }
    }
    let producer_partition_id =
        stable_producer_partition_id(&pack.document().pack.pack_id, facts.primary_partition_id())?;
    let producer_version = pack.document().pack.version.clone();

    let mut handoff = RecognizerGraphHandoff {
        disposition: RecognizerGraphHandoffDisposition::Rejected,
        output,
        producer_partition_id,
        producer_version,
        batch: None,
        validation: None,
        coverage: None,
        failure: None,
    };

    if cancelled.load(Ordering::Relaxed)
        || handoff
            .output
            .outcomes()
            .iter()
            .any(|outcome| outcome.state() == RecognizerRuleOutcomeState::Cancelled)
    {
        handoff.disposition = RecognizerGraphHandoffDisposition::Cancelled;
        return Ok(handoff);
    }

    if handoff.output.outcomes().iter().any(|outcome| {
        matches!(
            outcome.state(),
            RecognizerRuleOutcomeState::Partial | RecognizerRuleOutcomeState::NotEvaluated
        )
    }) {
        handoff.disposition = RecognizerGraphHandoffDisposition::Deferred;
        return Ok(handoff);
    }

    if handoff.output.context_id() != facts.context_id()
        || graph.source_context_id() != context.context_id()
        || handoff.output.context_id() != context.context_id()
    {
        handoff.failure = Some(handoff_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "recognizer and graph source contexts differ",
        ));
        return Ok(handoff);
    }
    if pack.document().pack.graph_registry_bundle_id.as_ref() != graph.registry().bundle_id() {
        handoff.failure = Some(handoff_error(
            RecognizerErrorCode::MatcherInputMismatch,
            "compiled pack names another graph registry bundle",
        ));
        return Ok(handoff);
    }

    let input = match graph.input_view(cancelled) {
        Ok(input) => input,
        Err(_) if cancelled.load(Ordering::Relaxed) => {
            handoff.disposition = RecognizerGraphHandoffDisposition::Cancelled;
            return Ok(handoff);
        }
        Err(error) => {
            handoff.failure = Some(graph_error(error));
            return Ok(handoff);
        }
    };

    match build_and_validate_batch(BatchBuildRequest {
        output: handoff.output,
        pack,
        producer_partition_id: &handoff.producer_partition_id,
        graph,
        input: &input,
        graph_limits,
        endpoint_bindings,
        cancelled,
    }) {
        Ok(BatchValidation::Cancelled) => {
            handoff.disposition = RecognizerGraphHandoffDisposition::Cancelled;
        }
        Ok(BatchValidation::Rejected(error)) => {
            handoff.failure = Some(error);
        }
        Ok(BatchValidation::GraphRejected(report)) => {
            handoff.disposition = RecognizerGraphHandoffDisposition::GraphRejected;
            handoff.validation = Some(report);
        }
        Ok(BatchValidation::Ready(batch, report, coverage)) => {
            if cancelled.load(Ordering::Relaxed) {
                handoff.disposition = RecognizerGraphHandoffDisposition::Cancelled;
            } else {
                handoff.disposition = RecognizerGraphHandoffDisposition::Ready;
                handoff.batch = Some(*batch);
                handoff.validation = Some(report);
                handoff.coverage = Some(coverage);
            }
        }
        Err(error) => handoff.failure = Some(error),
    }
    Ok(handoff)
}

enum BatchValidation {
    Cancelled,
    Rejected(RecognizerError),
    GraphRejected(GraphProposalValidationReport),
    Ready(
        Box<GraphProposalBatch>,
        GraphProposalValidationReport,
        Vec<GraphCoverageRecord>,
    ),
}

struct BatchBuildRequest<'a> {
    output: &'a RecognizerOutputPartition,
    pack: &'a CompiledRecognizerPack,
    producer_partition_id: &'a str,
    graph: &'a GraphPartitionSnapshot,
    input: &'a GraphSnapshot,
    graph_limits: GraphLimits,
    endpoint_bindings: &'a [RecognizerGraphEndpointBinding],
    cancelled: &'a AtomicBool,
}

type EndpointBindingKey = (
    Box<str>,
    RecognizerGraphEndpointSide,
    RecognizerFactValue,
    Option<RecognizerMatchId>,
);

fn build_and_validate_batch(request: BatchBuildRequest<'_>) -> RecognizerResult<BatchValidation> {
    let BatchBuildRequest {
        output,
        pack,
        producer_partition_id,
        graph,
        input,
        graph_limits,
        endpoint_bindings,
        cancelled,
    } = request;
    if endpoint_bindings.len() > MAX_GRAPH_ENDPOINT_BINDINGS {
        return Ok(BatchValidation::Rejected(handoff_error(
            RecognizerErrorCode::AdapterBindingInvalid,
            "graph endpoint bindings exceed the handoff bound",
        )));
    }

    let mut bindings = BTreeMap::<EndpointBindingKey, GraphProposalEndpoint>::new();
    let mut binding_bytes = 0_usize;
    for binding in endpoint_bindings {
        if cancelled.load(Ordering::Relaxed) {
            return Ok(BatchValidation::Cancelled);
        }
        let endpoint_bytes = match &binding.endpoint {
            GraphProposalEndpoint::Existing(node_id) => {
                if input.node(node_id).is_none() {
                    return Ok(BatchValidation::Rejected(handoff_error(
                        RecognizerErrorCode::AdapterBindingMissing,
                        "bound graph node is absent from the exact input view",
                    )));
                }
                node_id.as_str().len()
            }
            GraphProposalEndpoint::Proposed(proposal_id) => proposal_id.len(),
        };
        let value_bytes = match &binding.value {
            RecognizerFactValue::Nil => 0,
            RecognizerFactValue::Boolean(_) => std::mem::size_of::<bool>(),
            RecognizerFactValue::Integer(_) => std::mem::size_of::<i64>(),
            RecognizerFactValue::String(value)
            | RecognizerFactValue::Identifier(value)
            | RecognizerFactValue::Tag(value)
            | RecognizerFactValue::Reference(value) => value.len(),
        };
        let Some(next_binding_bytes) = binding_bytes
            .checked_add(binding.relation_kind_id.len())
            .and_then(|bytes| bytes.checked_add(value_bytes))
            .and_then(|bytes| bytes.checked_add(endpoint_bytes))
            .and_then(|bytes| {
                bytes.checked_add(binding.match_id.as_ref().map_or(0, |id| id.as_str().len()))
            })
        else {
            return Ok(BatchValidation::Rejected(handoff_error(
                RecognizerErrorCode::ProposalBudgetExceeded,
                "graph endpoint binding bytes overflowed",
            )));
        };
        if next_binding_bytes > MAX_GRAPH_ENDPOINT_BINDING_BYTES {
            return Ok(BatchValidation::Rejected(handoff_error(
                RecognizerErrorCode::ProposalBudgetExceeded,
                "graph endpoint bindings exceed the handoff byte bound",
            )));
        }
        binding_bytes = next_binding_bytes;
        let key = (
            binding.relation_kind_id.clone(),
            binding.side,
            binding.value.clone(),
            binding.match_id.clone(),
        );
        if bindings.insert(key, binding.endpoint.clone()).is_some() {
            return Ok(BatchValidation::Rejected(handoff_error(
                RecognizerErrorCode::AdapterBindingDuplicate,
                "duplicate graph relation endpoint binding",
            )));
        }
    }

    let proposal_count = output
        .outcomes()
        .iter()
        .try_fold(0_usize, |count, outcome| {
            count.checked_add(outcome.proposals().len())
        });
    let Some(proposal_count) = proposal_count else {
        return Ok(BatchValidation::Rejected(handoff_error(
            RecognizerErrorCode::ProposalBudgetExceeded,
            "graph proposal count overflowed",
        )));
    };
    if proposal_count > MAX_GRAPH_PROPOSALS {
        return Ok(BatchValidation::Rejected(handoff_error(
            RecognizerErrorCode::ProposalBudgetExceeded,
            "graph proposal count exceeds the handoff bound",
        )));
    }

    let mut used_bindings = BTreeSet::new();
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    for outcome in output.outcomes() {
        if cancelled.load(Ordering::Relaxed) {
            return Ok(BatchValidation::Cancelled);
        }
        for proposal in outcome.proposals() {
            if cancelled.load(Ordering::Relaxed) {
                return Ok(BatchValidation::Cancelled);
            }
            match proposal {
                RecognizerProposedAssertion::Entity {
                    proposal_id,
                    entity_kind_id,
                    semantic_key,
                    confidence,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } => {
                    let semantic_key = semantic_key
                        .iter()
                        .map(|(field, value)| Ok((field.clone(), graph_value(value)?)))
                        .collect::<RecognizerResult<BTreeMap<_, _>>>();
                    let semantic_key = match semantic_key {
                        Ok(value) => value,
                        Err(error) => return Ok(BatchValidation::Rejected(error)),
                    };
                    match GraphEntityProposal::new(
                        proposal_id.as_str(),
                        entity_kind_id.clone(),
                        semantic_key,
                        graph_confidence(*confidence),
                        source_handle_ids.clone(),
                        evidence_ids.clone(),
                        coverage_ids.clone(),
                    ) {
                        Ok(entity) => entities.push(entity),
                        Err(error) => return Ok(BatchValidation::Rejected(graph_error(error))),
                    }
                }
                RecognizerProposedAssertion::Relation {
                    proposal_id,
                    relation_kind_id,
                    source,
                    target,
                    confidence,
                    match_id,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } => {
                    let source_key = (
                        relation_kind_id.clone(),
                        RecognizerGraphEndpointSide::Source,
                        source.clone(),
                        Some(match_id.clone()),
                    );
                    let target_key = (
                        relation_kind_id.clone(),
                        RecognizerGraphEndpointSide::Target,
                        target.clone(),
                        Some(match_id.clone()),
                    );
                    let (
                        Some((source_binding_key, source_endpoint)),
                        Some((target_binding_key, target_endpoint)),
                    ) = (
                        resolve_endpoint_binding(&bindings, source_key),
                        resolve_endpoint_binding(&bindings, target_key),
                    )
                    else {
                        return Ok(BatchValidation::Rejected(handoff_error(
                            RecognizerErrorCode::AdapterBindingMissing,
                            "relation proposal lacks an exact existing-node endpoint binding",
                        )));
                    };
                    used_bindings.insert(source_binding_key);
                    used_bindings.insert(target_binding_key);
                    let relation = GraphRelationProposal::new(
                        proposal_id.as_str(),
                        relation_kind_id.clone(),
                        GraphRelationProposalInput {
                            source: source_endpoint.clone(),
                            target: target_endpoint.clone(),
                            confidence: graph_confidence(*confidence),
                            source_handle_ids: source_handle_ids.clone(),
                            evidence_ids: evidence_ids.clone(),
                            coverage_ids: coverage_ids.clone(),
                        },
                    );
                    match relation {
                        Ok(relation) => relations.push(relation),
                        Err(error) => return Ok(BatchValidation::Rejected(graph_error(error))),
                    }
                }
            }
        }
    }
    if used_bindings.len() != bindings.len() {
        return Ok(BatchValidation::Rejected(handoff_error(
            RecognizerErrorCode::AdapterBindingUnknown,
            "an exact graph endpoint binding was not consumed by this output",
        )));
    }

    let batch = match GraphProposalBatch::build(
        graph.registry().bundle_id(),
        graph.registry().registry_digest(),
        input.universe().clone(),
        input.generation().clone(),
        graph.source_context_id(),
        producer_partition_id,
        entities,
        relations,
    ) {
        Ok(batch) => batch,
        Err(error) => return Ok(BatchValidation::Rejected(graph_error(error))),
    };
    if cancelled.load(Ordering::Relaxed) {
        return Ok(BatchValidation::Cancelled);
    }
    let report =
        match validate_graph_proposal_batch(graph.registry(), Some(input), &batch, graph_limits) {
            Ok(report) => report,
            Err(_) if cancelled.load(Ordering::Relaxed) => return Ok(BatchValidation::Cancelled),
            Err(error) => return Ok(BatchValidation::Rejected(graph_error(error))),
        };
    if cancelled.load(Ordering::Relaxed) {
        return Ok(BatchValidation::Cancelled);
    }
    if !report.ready_for_publication() {
        return Ok(BatchValidation::GraphRejected(report));
    }
    let coverage = match graph_coverage_records(pack, graph.registry(), graph_limits) {
        Ok(coverage) => coverage,
        Err(error) => return Ok(BatchValidation::Rejected(error)),
    };
    Ok(BatchValidation::Ready(Box::new(batch), report, coverage))
}

fn resolve_endpoint_binding(
    bindings: &BTreeMap<EndpointBindingKey, GraphProposalEndpoint>,
    match_scoped_key: EndpointBindingKey,
) -> Option<(EndpointBindingKey, &GraphProposalEndpoint)> {
    if let Some(endpoint) = bindings.get(&match_scoped_key) {
        return Some((match_scoped_key, endpoint));
    }
    let (relation_kind_id, side, value, _) = match_scoped_key;
    let unscoped_key = (relation_kind_id, side, value, None);
    bindings
        .get(&unscoped_key)
        .map(|endpoint| (unscoped_key, endpoint))
}

fn graph_coverage_records(
    pack: &CompiledRecognizerPack,
    registry: &GraphRegistryBundle,
    limits: GraphLimits,
) -> RecognizerResult<Vec<GraphCoverageRecord>> {
    let mut relations = BTreeSet::new();
    for rule in &pack.document().pack.rules {
        for output in &rule.outputs {
            if let crate::RecognizerOutput::RelationAssertion {
                relation_kind_id, ..
            } = output
            {
                let Some(definition) = registry.relation_kind(relation_kind_id) else {
                    return Err(handoff_error(
                        RecognizerErrorCode::ProposalInvalid,
                        "complete recognizer output names an unregistered graph relation",
                    ));
                };
                relations.insert(definition.relation());
            }
        }
    }
    relations
        .into_iter()
        .map(|relation: GraphRelationKind| {
            GraphCoverageRecord::new(
                relation,
                GraphCoverageState::Complete,
                false,
                Vec::new(),
                limits,
            )
            .map_err(graph_error)
        })
        .collect()
}

fn graph_value(value: &RecognizerFactValue) -> RecognizerResult<GraphProposalValue> {
    match value {
        RecognizerFactValue::Boolean(value) => Ok(GraphProposalValue::Boolean(*value)),
        RecognizerFactValue::Integer(value) => Ok(GraphProposalValue::Integer(*value)),
        RecognizerFactValue::String(value) => Ok(GraphProposalValue::String(value.clone())),
        RecognizerFactValue::Identifier(value) => Ok(GraphProposalValue::Identifier(value.clone())),
        RecognizerFactValue::Reference(value) => Ok(GraphProposalValue::Reference(value.clone())),
        RecognizerFactValue::Nil | RecognizerFactValue::Tag(_) => Err(handoff_error(
            RecognizerErrorCode::ProposalInvalid,
            "graph identity schema has no exact value variant for this recognizer value",
        )),
    }
}

const fn graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn stable_producer_partition_id(
    pack_id: &str,
    input_partition_id: &str,
) -> RecognizerResult<Box<str>> {
    // Exclude the pack version so a version update replaces this exact pack/input
    // partition. The version is carried separately for GraphPartitionReplacement.
    let bytes = canonical_json_bytes(&(
        "wow-recognizers/graph-producer-partition/e2-b/1",
        pack_id,
        input_partition_id,
    ))
    .map_err(|_| {
        handoff_error(
            RecognizerErrorCode::ProposalIdentityMismatch,
            "recognizer producer partition identity cannot be canonicalized",
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut id = String::with_capacity("recognizer-producer-partition:sha256:".len() + 64);
    id.push_str("recognizer-producer-partition:sha256:");
    for byte in digest {
        let _ = write!(id, "{byte:02x}");
    }
    Ok(id.into_boxed_str())
}

fn graph_error(error: GraphError) -> RecognizerError {
    handoff_error(
        RecognizerErrorCode::GraphProjectionFailed,
        format!("graph handoff validation failed: {error}"),
    )
}

fn handoff_error(code: RecognizerErrorCode, message: impl Into<Box<str>>) -> RecognizerError {
    RecognizerError::new(code, message)
}
