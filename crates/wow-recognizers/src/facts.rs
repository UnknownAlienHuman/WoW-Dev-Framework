use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{
    CoverageId, EvidenceId, GenerationContext, GenerationContextId, StableHandleId,
    canonical_json_bytes,
};
use wow_graph::GraphConfidence;

use crate::{
    RecognizerError, RecognizerErrorCode, RecognizerFactBundleId, RecognizerFactId,
    RecognizerResult,
};

pub const RECOGNIZER_FACT_BUNDLE_SCHEMA: &str = "wow-recognizers/fact-bundle/e2-b/1";
const MAX_TEXT_BYTES: usize = 4 * 1024;
const MAX_COMPONENT_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactLimits {
    pub max_facts: u32,
    pub max_coverage_records: u32,
    pub max_dependency_partitions: u32,
    pub max_fields_per_fact: u32,
    pub max_source_handles_per_fact: u32,
    pub max_evidence_per_fact: u32,
    pub max_blockers_per_coverage: u32,
}

impl RecognizerFactLimits {
    pub fn new(
        max_facts: u32,
        max_coverage_records: u32,
        max_dependency_partitions: u32,
        max_fields_per_fact: u32,
        max_source_handles_per_fact: u32,
        max_evidence_per_fact: u32,
        max_blockers_per_coverage: u32,
    ) -> RecognizerResult<Self> {
        let limits = Self {
            max_facts,
            max_coverage_records,
            max_dependency_partitions,
            max_fields_per_fact,
            max_source_handles_per_fact,
            max_evidence_per_fact,
            max_blockers_per_coverage,
        };
        limits.validate()?;
        Ok(limits)
    }

    pub(crate) fn validate(self) -> RecognizerResult<()> {
        if self.max_facts == 0
            || self.max_facts > 1_000_000
            || self.max_coverage_records == 0
            || self.max_coverage_records > 16_384
            || self.max_dependency_partitions > 1_024
            || self.max_fields_per_fact == 0
            || self.max_fields_per_fact > 512
            || self.max_source_handles_per_fact == 0
            || self.max_source_handles_per_fact > 256
            || self.max_evidence_per_fact == 0
            || self.max_evidence_per_fact > 256
            || self.max_blockers_per_coverage == 0
            || self.max_blockers_per_coverage > 256
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBudgetExceeded,
                "recognizer fact limits are outside the reviewed E2-B profile",
            ));
        }
        Ok(())
    }
}

impl Default for RecognizerFactLimits {
    fn default() -> Self {
        Self {
            max_facts: 100_000,
            max_coverage_records: 4_096,
            max_dependency_partitions: 128,
            max_fields_per_fact: 128,
            max_source_handles_per_fact: 64,
            max_evidence_per_fact: 64,
            max_blockers_per_coverage: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecognizerFactValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    String(Box<str>),
    Identifier(Box<str>),
    Tag(Box<str>),
    Reference(Box<str>),
}

impl RecognizerFactValue {
    fn validate(&self) -> RecognizerResult<()> {
        match self {
            Self::Nil | Self::Boolean(_) | Self::Integer(_) => Ok(()),
            Self::String(value) => {
                if value.len() > MAX_TEXT_BYTES || value.contains('\0') {
                    Err(RecognizerError::new(
                        RecognizerErrorCode::FactInvalid,
                        "recognizer fact string is too large or contains NUL",
                    ))
                } else {
                    Ok(())
                }
            }
            Self::Identifier(value) | Self::Tag(value) | Self::Reference(value) => {
                validate_component(value, RecognizerErrorCode::FactInvalid)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerFactScopeKind {
    Partition,
    File,
    Function,
    Package,
    XmlDocument,
    LoadUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactScope {
    kind: RecognizerFactScopeKind,
    id: Box<str>,
}

impl RecognizerFactScope {
    pub fn new(kind: RecognizerFactScopeKind, id: impl Into<Box<str>>) -> RecognizerResult<Self> {
        let id = id.into();
        validate_bounded_text(&id, RecognizerErrorCode::FactInvalid)?;
        Ok(Self { kind, id })
    }

    #[must_use]
    pub const fn kind(&self) -> RecognizerFactScopeKind {
        self.kind
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    fn validate(&self) -> RecognizerResult<()> {
        validate_bounded_text(&self.id, RecognizerErrorCode::FactInvalid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognizerFactInput {
    pub kind: Box<str>,
    pub partition_id: Box<str>,
    pub scope: RecognizerFactScope,
    pub producer_id: Box<str>,
    pub producer_version: Box<str>,
    pub confidence: GraphConfidence,
    pub fields: BTreeMap<Box<str>, RecognizerFactValue>,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFact {
    fact_id: RecognizerFactId,
    context_id: GenerationContextId,
    kind: Box<str>,
    partition_id: Box<str>,
    scope: RecognizerFactScope,
    producer_id: Box<str>,
    producer_version: Box<str>,
    confidence: GraphConfidence,
    fields: BTreeMap<Box<str>, RecognizerFactValue>,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
}

impl RecognizerFact {
    pub fn new(
        context_id: GenerationContextId,
        input: RecognizerFactInput,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let RecognizerFactInput {
            kind,
            partition_id,
            scope,
            producer_id,
            producer_version,
            confidence,
            fields,
            source_handle_ids,
            evidence_ids,
        } = input;
        validate_component(&kind, RecognizerErrorCode::FactInvalid)?;
        validate_bounded_text(&partition_id, RecognizerErrorCode::FactInvalid)?;
        scope.validate()?;
        validate_component(&producer_id, RecognizerErrorCode::FactInvalid)?;
        validate_bounded_text(&producer_version, RecognizerErrorCode::FactInvalid)?;
        if fields.is_empty() || fields.len() > limits.max_fields_per_fact as usize {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBudgetExceeded,
                "recognizer fact fields are empty or exceed the configured limit",
            ));
        }
        for (name, value) in &fields {
            validate_component(name, RecognizerErrorCode::FactInvalid)?;
            value.validate()?;
        }
        let source_handle_ids = normalize_nonempty(
            source_handle_ids,
            limits.max_source_handles_per_fact as usize,
            RecognizerErrorCode::FactInvalid,
            "recognizer fact source handles are empty or exceed the configured limit",
        )?;
        let evidence_ids = normalize_nonempty(
            evidence_ids,
            limits.max_evidence_per_fact as usize,
            RecognizerErrorCode::FactInvalid,
            "recognizer fact evidence is empty or exceeds the configured limit",
        )?;
        let fact_id = derive_fact_id(
            context_id,
            &kind,
            &partition_id,
            &scope,
            &producer_id,
            &producer_version,
            confidence,
            &fields,
        )?;
        Ok(Self {
            fact_id,
            context_id,
            kind,
            partition_id,
            scope,
            producer_id,
            producer_version,
            confidence,
            fields,
            source_handle_ids,
            evidence_ids,
        })
    }

    pub fn validate(&self, limits: RecognizerFactLimits) -> RecognizerResult<()> {
        let rebuilt = Self::new(
            self.context_id,
            RecognizerFactInput {
                kind: self.kind.clone(),
                partition_id: self.partition_id.clone(),
                scope: self.scope.clone(),
                producer_id: self.producer_id.clone(),
                producer_version: self.producer_version.clone(),
                confidence: self.confidence,
                fields: self.fields.clone(),
                source_handle_ids: self.source_handle_ids.clone(),
                evidence_ids: self.evidence_ids.clone(),
            },
            limits,
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactInvalid,
                "recognizer fact identity or canonical support does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn fact_id(&self) -> &RecognizerFactId {
        &self.fact_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    #[must_use]
    pub const fn scope(&self) -> &RecognizerFactScope {
        &self.scope
    }

    #[must_use]
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }

    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }

    #[must_use]
    pub const fn confidence(&self) -> GraphConfidence {
        self.confidence
    }

    #[must_use]
    pub fn fields(&self) -> &BTreeMap<Box<str>, RecognizerFactValue> {
        &self.fields
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&RecognizerFactValue> {
        self.fields.get(name)
    }

    #[must_use]
    pub fn source_handle_ids(&self) -> &[StableHandleId] {
        &self.source_handle_ids
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[EvidenceId] {
        &self.evidence_ids
    }

    fn same_semantics(&self, other: &Self) -> bool {
        self.fact_id == other.fact_id
            && self.context_id == other.context_id
            && self.kind == other.kind
            && self.partition_id == other.partition_id
            && self.scope == other.scope
            && self.producer_id == other.producer_id
            && self.producer_version == other.producer_version
            && self.confidence == other.confidence
            && self.fields == other.fields
    }

    fn merge_support(
        &mut self,
        other: &Self,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<()> {
        if !self.same_semantics(other) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactDuplicate,
                "recognizer facts share an identity but disagree semantically",
            ));
        }
        self.source_handle_ids
            .extend_from_slice(&other.source_handle_ids);
        self.source_handle_ids.sort();
        self.source_handle_ids.dedup();
        self.evidence_ids.extend_from_slice(&other.evidence_ids);
        self.evidence_ids.sort();
        self.evidence_ids.dedup();
        if self.source_handle_ids.len() > limits.max_source_handles_per_fact as usize
            || self.evidence_ids.len() > limits.max_evidence_per_fact as usize
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBudgetExceeded,
                "merged recognizer fact support exceeds the configured limit",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerFactCoverageState {
    Complete,
    Partial,
    NotEvaluated,
    Failed,
    Truncated,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactCoverage {
    coverage_id: CoverageId,
    context_id: GenerationContextId,
    partition_id: Box<str>,
    capability_id: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    state: RecognizerFactCoverageState,
    blocker_ids: Vec<Box<str>>,
}

impl RecognizerFactCoverage {
    pub fn new(
        context_id: GenerationContextId,
        partition_id: impl Into<Box<str>>,
        capability_id: impl Into<Box<str>>,
        producer_id: impl Into<Box<str>>,
        producer_version: impl Into<Box<str>>,
        state: RecognizerFactCoverageState,
        blocker_ids: Vec<Box<str>>,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let partition_id = partition_id.into();
        let capability_id = capability_id.into();
        let producer_id = producer_id.into();
        let producer_version = producer_version.into();
        validate_bounded_text(&partition_id, RecognizerErrorCode::FactCoverageInvalid)?;
        validate_component(&capability_id, RecognizerErrorCode::FactCoverageInvalid)?;
        validate_component(&producer_id, RecognizerErrorCode::FactCoverageInvalid)?;
        validate_bounded_text(
            &producer_version,
            RecognizerErrorCode::FactCoverageInvalid,
        )?;
        let blocker_ids = normalize_optional_text_ids(
            blocker_ids,
            limits.max_blockers_per_coverage as usize,
            RecognizerErrorCode::FactCoverageInvalid,
        )?;
        if state == RecognizerFactCoverageState::Complete && !blocker_ids.is_empty() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactCoverageInvalid,
                "complete recognizer fact coverage cannot carry blockers",
            ));
        }
        if state != RecognizerFactCoverageState::Complete && blocker_ids.is_empty() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactCoverageInvalid,
                "non-complete recognizer fact coverage requires a blocker",
            ));
        }
        let coverage_id = derive_coverage_id(
            context_id,
            &partition_id,
            &capability_id,
            &producer_id,
            &producer_version,
            state,
            &blocker_ids,
        )?;
        Ok(Self {
            coverage_id,
            context_id,
            partition_id,
            capability_id,
            producer_id,
            producer_version,
            state,
            blocker_ids,
        })
    }

    pub fn validate(&self, limits: RecognizerFactLimits) -> RecognizerResult<()> {
        let rebuilt = Self::new(
            self.context_id,
            self.partition_id.clone(),
            self.capability_id.clone(),
            self.producer_id.clone(),
            self.producer_version.clone(),
            self.state,
            self.blocker_ids.clone(),
            limits,
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactCoverageInvalid,
                "recognizer fact coverage identity or canonical blockers do not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn coverage_id(&self) -> CoverageId {
        self.coverage_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub const fn state(&self) -> RecognizerFactCoverageState {
        self.state
    }

    #[must_use]
    pub fn blocker_ids(&self) -> &[Box<str>] {
        &self.blocker_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerFactBundle {
    schema: Box<str>,
    bundle_id: RecognizerFactBundleId,
    context_id: GenerationContextId,
    primary_partition_id: Box<str>,
    dependency_partition_ids: Vec<Box<str>>,
    facts: Vec<RecognizerFact>,
    coverage: Vec<RecognizerFactCoverage>,
}

impl RecognizerFactBundle {
    pub fn build(
        context: &GenerationContext,
        primary_partition_id: impl Into<Box<str>>,
        mut dependency_partition_ids: Vec<Box<str>>,
        mut facts: Vec<RecognizerFact>,
        mut coverage: Vec<RecognizerFactCoverage>,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        context.validate().map_err(|_| {
            RecognizerError::new(
                RecognizerErrorCode::FactBundleInvalid,
                "recognizer fact bundle generation context is invalid",
            )
        })?;
        let context_id = context.context_id();
        let primary_partition_id = primary_partition_id.into();
        validate_bounded_text(
            &primary_partition_id,
            RecognizerErrorCode::FactBundleInvalid,
        )?;
        if dependency_partition_ids.len() > limits.max_dependency_partitions as usize {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBudgetExceeded,
                "recognizer dependency partitions exceed the configured limit",
            ));
        }
        for partition in &dependency_partition_ids {
            validate_bounded_text(partition, RecognizerErrorCode::FactBundleInvalid)?;
        }
        dependency_partition_ids.sort();
        if dependency_partition_ids
            .windows(2)
            .any(|pair| pair[0] == pair[1])
            || dependency_partition_ids
                .binary_search(&primary_partition_id)
                .is_ok()
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBundleInvalid,
                "recognizer dependency partitions are duplicate or include the primary partition",
            ));
        }
        if facts.len() > limits.max_facts as usize
            || coverage.len() > limits.max_coverage_records as usize
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBudgetExceeded,
                "recognizer fact bundle exceeds configured record limits",
            ));
        }
        for fact in &facts {
            fact.validate(limits)?;
            require_context(context_id, fact.context_id())?;
            require_partition(
                fact.partition_id(),
                &primary_partition_id,
                &dependency_partition_ids,
            )?;
        }
        facts.sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
        let mut merged_facts: Vec<RecognizerFact> = Vec::with_capacity(facts.len());
        for fact in facts {
            if let Some(previous) = merged_facts.last_mut()
                && previous.fact_id == fact.fact_id
            {
                previous.merge_support(&fact, limits)?;
                continue;
            }
            merged_facts.push(fact);
        }
        for record in &coverage {
            record.validate(limits)?;
            require_context(context_id, record.context_id())?;
            require_partition(
                record.partition_id(),
                &primary_partition_id,
                &dependency_partition_ids,
            )?;
        }
        coverage.sort_by(|left, right| {
            (left.partition_id.as_ref(), left.capability_id.as_ref())
                .cmp(&(right.partition_id.as_ref(), right.capability_id.as_ref()))
        });
        let mut canonical_coverage = Vec::with_capacity(coverage.len());
        for record in coverage {
            if let Some(previous) = canonical_coverage.last()
                && previous.partition_id == record.partition_id
                && previous.capability_id == record.capability_id
            {
                if previous == &record {
                    continue;
                }
                return Err(RecognizerError::new(
                    RecognizerErrorCode::FactCoverageInvalid,
                    "recognizer fact coverage conflicts for one capability partition",
                ));
            }
            canonical_coverage.push(record);
        }
        let bundle_id = derive_bundle_id(
            context_id,
            &primary_partition_id,
            &dependency_partition_ids,
            &merged_facts,
            &canonical_coverage,
        )?;
        Ok(Self {
            schema: RECOGNIZER_FACT_BUNDLE_SCHEMA.into(),
            bundle_id,
            context_id,
            primary_partition_id,
            dependency_partition_ids,
            facts: merged_facts,
            coverage: canonical_coverage,
        })
    }

    pub fn validate(
        &self,
        context: &GenerationContext,
        limits: RecognizerFactLimits,
    ) -> RecognizerResult<()> {
        if self.schema.as_ref() != RECOGNIZER_FACT_BUNDLE_SCHEMA {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBundleInvalid,
                "recognizer fact bundle schema is unsupported",
            ));
        }
        let rebuilt = Self::build(
            context,
            self.primary_partition_id.clone(),
            self.dependency_partition_ids.clone(),
            self.facts.clone(),
            self.coverage.clone(),
            limits,
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::FactBundleIdentityMismatch,
                "recognizer fact bundle identity or canonical records do not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn bundle_id(&self) -> &RecognizerFactBundleId {
        &self.bundle_id
    }

    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    #[must_use]
    pub fn primary_partition_id(&self) -> &str {
        &self.primary_partition_id
    }

    #[must_use]
    pub fn dependency_partition_ids(&self) -> &[Box<str>] {
        &self.dependency_partition_ids
    }

    #[must_use]
    pub fn facts(&self) -> &[RecognizerFact] {
        &self.facts
    }

    #[must_use]
    pub fn coverage(&self) -> &[RecognizerFactCoverage] {
        &self.coverage
    }

    #[must_use]
    pub fn fact_by_id(&self, fact_id: &RecognizerFactId) -> Option<&RecognizerFact> {
        self.facts
            .binary_search_by(|fact| fact.fact_id.cmp(fact_id))
            .ok()
            .map(|index| &self.facts[index])
    }

    #[must_use]
    pub fn facts_by_kind<'a>(
        &'a self,
        kind: &'a str,
    ) -> impl Iterator<Item = &'a RecognizerFact> + 'a {
        self.facts.iter().filter(move |fact| fact.kind() == kind)
    }

    #[must_use]
    pub fn coverage_for(
        &self,
        partition_id: &str,
        capability_id: &str,
    ) -> Option<&RecognizerFactCoverage> {
        self.coverage
            .binary_search_by(|record| {
                (record.partition_id(), record.capability_id())
                    .cmp(&(partition_id, capability_id))
            })
            .ok()
            .map(|index| &self.coverage[index])
    }

    #[must_use]
    pub fn has_complete_capability(&self, partition_id: &str, capability_id: &str) -> bool {
        self.coverage_for(partition_id, capability_id)
            .is_some_and(|record| record.state() == RecognizerFactCoverageState::Complete)
    }

    #[must_use]
    pub fn visible_partition_ids(&self) -> BTreeSet<&str> {
        let mut partitions = BTreeSet::new();
        partitions.insert(self.primary_partition_id());
        partitions.extend(
            self.dependency_partition_ids
                .iter()
                .map(|partition| partition.as_ref()),
        );
        partitions
    }
}

#[derive(Serialize)]
struct FactIdentity<'a> {
    schema: &'static str,
    context_id: GenerationContextId,
    kind: &'a str,
    partition_id: &'a str,
    scope: &'a RecognizerFactScope,
    producer_id: &'a str,
    producer_version: &'a str,
    confidence: GraphConfidence,
    fields: &'a BTreeMap<Box<str>, RecognizerFactValue>,
}

fn derive_fact_id(
    context_id: GenerationContextId,
    kind: &str,
    partition_id: &str,
    scope: &RecognizerFactScope,
    producer_id: &str,
    producer_version: &str,
    confidence: GraphConfidence,
    fields: &BTreeMap<Box<str>, RecognizerFactValue>,
) -> RecognizerResult<RecognizerFactId> {
    let bytes = canonical_json_bytes(&FactIdentity {
        schema: "wow-recognizers/fact/e2-b/1",
        context_id,
        kind,
        partition_id,
        scope,
        producer_id,
        producer_version,
        confidence,
        fields,
    })
    .map_err(|_| identity_error("recognizer fact identity cannot be canonicalized"))?;
    RecognizerFactId::new(format!(
        "recognizer-fact:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

#[derive(Serialize)]
struct CoverageIdentity<'a> {
    schema: &'static str,
    context_id: GenerationContextId,
    partition_id: &'a str,
    capability_id: &'a str,
    producer_id: &'a str,
    producer_version: &'a str,
    state: RecognizerFactCoverageState,
    blocker_ids: &'a [Box<str>],
}

fn derive_coverage_id(
    context_id: GenerationContextId,
    partition_id: &str,
    capability_id: &str,
    producer_id: &str,
    producer_version: &str,
    state: RecognizerFactCoverageState,
    blocker_ids: &[Box<str>],
) -> RecognizerResult<CoverageId> {
    CoverageId::derive(&CoverageIdentity {
        schema: "wow-recognizers/fact-coverage/e2-b/1",
        context_id,
        partition_id,
        capability_id,
        producer_id,
        producer_version,
        state,
        blocker_ids,
    })
    .map_err(|_| identity_error("recognizer fact coverage identity cannot be canonicalized"))
}

#[derive(Serialize)]
struct BundleIdentity<'a> {
    schema: &'static str,
    context_id: GenerationContextId,
    primary_partition_id: &'a str,
    dependency_partition_ids: &'a [Box<str>],
    facts: &'a [RecognizerFact],
    coverage: &'a [RecognizerFactCoverage],
}

fn derive_bundle_id(
    context_id: GenerationContextId,
    primary_partition_id: &str,
    dependency_partition_ids: &[Box<str>],
    facts: &[RecognizerFact],
    coverage: &[RecognizerFactCoverage],
) -> RecognizerResult<RecognizerFactBundleId> {
    let bytes = canonical_json_bytes(&BundleIdentity {
        schema: RECOGNIZER_FACT_BUNDLE_SCHEMA,
        context_id,
        primary_partition_id,
        dependency_partition_ids,
        facts,
        coverage,
    })
    .map_err(|_| identity_error("recognizer fact bundle identity cannot be canonicalized"))?;
    RecognizerFactBundleId::new(format!(
        "recognizer-fact-bundle:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

fn require_context(
    expected: GenerationContextId,
    actual: GenerationContextId,
) -> RecognizerResult<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(RecognizerError::new(
            RecognizerErrorCode::FactContextMismatch,
            "recognizer fact record belongs to a different generation context",
        ))
    }
}

fn require_partition(
    partition_id: &str,
    primary: &str,
    dependencies: &[Box<str>],
) -> RecognizerResult<()> {
    if partition_id == primary
        || dependencies
            .binary_search_by(|candidate| candidate.as_ref().cmp(partition_id))
            .is_ok()
    {
        Ok(())
    } else {
        Err(RecognizerError::new(
            RecognizerErrorCode::FactPartitionUnknown,
            "recognizer fact record belongs to an undeclared partition",
        ))
    }
}

fn normalize_nonempty<T: Ord>(
    mut values: Vec<T>,
    max: usize,
    code: RecognizerErrorCode,
    message: &'static str,
) -> RecognizerResult<Vec<T>> {
    if values.is_empty() || values.len() > max {
        return Err(RecognizerError::new(code, message));
    }
    values.sort();
    values.dedup();
    Ok(values)
}

fn normalize_optional_text_ids(
    mut values: Vec<Box<str>>,
    max: usize,
    code: RecognizerErrorCode,
) -> RecognizerResult<Vec<Box<str>>> {
    if values.len() > max {
        return Err(RecognizerError::new(
            code,
            "recognizer fact blocker IDs exceed the configured limit",
        ));
    }
    for value in &values {
        validate_component(value, code)?;
    }
    values.sort();
    values.dedup();
    Ok(values)
}

fn validate_component(value: &str, code: RecognizerErrorCode) -> RecognizerResult<()> {
    if value.is_empty()
        || value.len() > MAX_COMPONENT_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@' | b'/' | b'#')
        })
    {
        return Err(RecognizerError::new(
            code,
            "recognizer fact component is invalid",
        ));
    }
    Ok(())
}

fn validate_bounded_text(value: &str, code: RecognizerErrorCode) -> RecognizerResult<()> {
    if value.is_empty()
        || value.len() > MAX_TEXT_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(RecognizerError::new(
            code,
            "recognizer fact text is invalid",
        ));
    }
    Ok(())
}

fn identity_error(message: &'static str) -> RecognizerError {
    RecognizerError::new(RecognizerErrorCode::FactBundleIdentityMismatch, message)
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
