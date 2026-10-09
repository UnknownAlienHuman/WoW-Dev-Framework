//! Producer-owned graph derivations and unresolved conflicts. Exact assertion
//! resolution belongs to partition publication; these types own canonical data.
use crate::{GraphError, GraphErrorCode, GraphGenerationId, GraphResult, GraphUniverseId};
use serde::{Deserialize, Serialize};
use wow_core::{GenerationContextId, canonical_json_bytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphAssertionKind {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphLocalAssertion {
    pub kind: GraphAssertionKind,
    pub proposal_id: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "snake_case", deny_unknown_fields)]
pub enum GraphAssertionRef {
    Local {
        assertion: GraphLocalAssertion,
    },
    Producer {
        partition_id: Box<str>,
        batch_id: Box<str>,
        assertion: GraphLocalAssertion,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAssertionRecordScope {
    pub universe: GraphUniverseId,
    pub generation: GraphGenerationId,
    pub source_context_id: GenerationContextId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphDerivationRecord {
    pub output: GraphLocalAssertion,
    pub rule_id: Box<str>,
    pub rule_version: u32,
    pub inputs: Vec<GraphAssertionRef>,
    pub rebuttals: Vec<GraphAssertionRef>,
    pub missing: Vec<Box<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphConflictKind {
    ExclusiveAttributeDisagreement,
    RelationMultiplicityViolation,
    ForbiddenCycle,
    EndpointKindDisagreement,
    CrossScopeGenerationConflict,
    ProducerIdentityOrSchemaConflict,
    EvidenceOrSourceHandleConflict,
    CoverageVersusAssertionConflict,
}

/// A producer reports this unresolved conflict; admission validates its exact
/// participants, not the authenticity or completeness of a producer's judgment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphConflictRecord {
    pub kind: GraphConflictKind,
    pub subject: GraphAssertionRef,
    pub assertions: Vec<GraphAssertionRef>,
    pub affected_capabilities: Vec<Box<str>>,
    pub affected_axes: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAssertionRecords {
    pub scope: GraphAssertionRecordScope,
    pub derivations: Vec<GraphDerivationRecord>,
    pub conflicts: Vec<GraphConflictRecord>,
}
impl GraphAssertionRecords {
    pub fn build(
        scope: GraphAssertionRecordScope,
        mut derivations: Vec<GraphDerivationRecord>,
        mut conflicts: Vec<GraphConflictRecord>,
    ) -> GraphResult<Self> {
        if derivations.len().saturating_add(conflicts.len()) > 200_000 {
            return Err(budget());
        }
        GraphUniverseId::new(scope.universe.as_str())?;
        GraphGenerationId::new(scope.generation.as_str())?;
        for record in &mut derivations {
            local(&record.output)?;
            crate::registry::validate_component(&record.rule_id, "derivation rule")?;
            if record.rule_version == 0
                || record.inputs.len().saturating_add(record.rebuttals.len()) > 64
                || record.missing.len() > 64
            {
                return Err(invalid());
            }
            normalize_refs(&mut record.inputs)?;
            normalize_refs(&mut record.rebuttals)?;
            normalize_text(&mut record.missing, 64)?;
            if record.inputs.is_empty() && record.missing.is_empty() {
                return Err(invalid());
            }
            if record
                .inputs
                .iter()
                .any(|input| record.rebuttals.binary_search(input).is_ok())
            {
                return Err(invalid());
            }
        }
        derivations.sort_by(|a, b| a.output.cmp(&b.output));
        if derivations
            .windows(2)
            .any(|pair| pair[0].output == pair[1].output)
        {
            return Err(invalid());
        }
        for record in &mut conflicts {
            reference(&record.subject)?;
            normalize_refs(&mut record.assertions)?;
            normalize_text(&mut record.affected_capabilities, 32)?;
            normalize_text(&mut record.affected_axes, 32)?;
            if record.assertions.len() < 2
                || record.assertions.binary_search(&record.subject).is_err()
                || (record.affected_capabilities.is_empty() && record.affected_axes.is_empty())
            {
                return Err(invalid());
            }
        }
        conflicts.sort();
        if conflicts.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid());
        }
        let records = Self {
            scope,
            derivations,
            conflicts,
        };
        if canonical_json_bytes(&records).map_err(|_| invalid())?.len() > 64 * 1024 * 1024 {
            return Err(budget());
        }
        Ok(records)
    }
    pub fn validate(&self) -> GraphResult<()> {
        if Self::build(
            self.scope.clone(),
            self.derivations.clone(),
            self.conflicts.clone(),
        )? != *self
        {
            return Err(invalid());
        }
        Ok(())
    }
}
fn local(assertion: &GraphLocalAssertion) -> GraphResult<()> {
    // Same external-ID domain as graph proposals; paths/Unicode are not narrowed.
    if assertion.proposal_id.is_empty()
        || assertion.proposal_id.len() > 1024
        || assertion.proposal_id.chars().any(char::is_control)
    {
        return Err(invalid());
    }
    Ok(())
}
fn reference(value: &GraphAssertionRef) -> GraphResult<()> {
    match value {
        GraphAssertionRef::Local { assertion } => local(assertion),
        GraphAssertionRef::Producer {
            partition_id,
            batch_id,
            assertion,
        } => {
            crate::registry::validate_component(partition_id, "assertion producer")?;
            let Some(hex) = batch_id.strip_prefix("graph-proposal-batch:sha256:") else {
                return Err(invalid());
            };
            if hex.len() != 64
                || !hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(invalid());
            }
            local(assertion)
        }
    }
}
fn normalize_refs(values: &mut [GraphAssertionRef]) -> GraphResult<()> {
    if values.len() > 64 {
        return Err(budget());
    }
    for value in values.iter() {
        reference(value)?;
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid());
    }
    Ok(())
}
fn normalize_text(values: &mut [Box<str>], limit: usize) -> GraphResult<()> {
    if values.len() > limit {
        return Err(budget());
    }
    for value in values.iter() {
        crate::registry::validate_component(value, "record scope identifier")?;
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid());
    }
    Ok(())
}
fn invalid() -> GraphError {
    GraphError::new(
        GraphErrorCode::ProposalBatchInvalid,
        "invalid or noncanonical assertion records",
    )
}
fn budget() -> GraphError {
    GraphError::new(
        GraphErrorCode::BudgetExceeded,
        "assertion record budget exceeded",
    )
}
