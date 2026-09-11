use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_graph::{
    GraphConfidence, GraphGenerationId, GraphNodeId, GraphRelationKind, GraphSnapshotId,
    GraphUniverseId,
};

use crate::{
    RecognitionAssertionId, RecognitionReportId, RecognizerError, RecognizerErrorCode,
    RecognizerId, RecognizerRegistryId, RecognizerResult, RecognizerVersion,
    StructuredObservationId,
};

pub const RECOGNITION_REPORT_SCHEMA: &str = "wow-recognizers/report/e2-b/1";
pub const RECOGNIZER_REGISTRY_SCHEMA: &str = "wow-recognizers/registry/e2-b/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationFamily {
    DirectCall,
    StateRead,
    StateWrite,
    NativeEventRegistration,
    NativeEventHandler,
    NativeEventBridge,
    CustomSignalEmit,
    CustomSignalHandle,
    CvarCallbackRegistration,
    ScriptAssignment,
    ScriptHook,
    SecureFunctionHook,
    Dependency,
    Ownership,
    Load,
    Inheritance,
    Mixin,
    FactoryCreation,
    ApiUse,
}

impl ObservationFamily {
    pub const ALL: [Self; 19] = [
        Self::DirectCall,
        Self::StateRead,
        Self::StateWrite,
        Self::NativeEventRegistration,
        Self::NativeEventHandler,
        Self::NativeEventBridge,
        Self::CustomSignalEmit,
        Self::CustomSignalHandle,
        Self::CvarCallbackRegistration,
        Self::ScriptAssignment,
        Self::ScriptHook,
        Self::SecureFunctionHook,
        Self::Dependency,
        Self::Ownership,
        Self::Load,
        Self::Inheritance,
        Self::Mixin,
        Self::FactoryCreation,
        Self::ApiUse,
    ];

    #[must_use]
    pub const fn relation(self) -> GraphRelationKind {
        match self {
            Self::DirectCall => GraphRelationKind::Calls,
            Self::StateRead => GraphRelationKind::ReadsState,
            Self::StateWrite => GraphRelationKind::WritesState,
            Self::NativeEventRegistration => GraphRelationKind::RegistersNativeEvent,
            Self::NativeEventHandler => GraphRelationKind::HandlesNativeEvent,
            Self::NativeEventBridge => GraphRelationKind::BridgesNativeEvent,
            Self::CustomSignalEmit => GraphRelationKind::EmitsCustomSignal,
            Self::CustomSignalHandle => GraphRelationKind::HandlesCustomSignal,
            Self::CvarCallbackRegistration => GraphRelationKind::RegistersCvarCallback,
            Self::ScriptAssignment => GraphRelationKind::SetsScript,
            Self::ScriptHook => GraphRelationKind::HooksScript,
            Self::SecureFunctionHook => GraphRelationKind::SecureHooksFunction,
            Self::Dependency => GraphRelationKind::DependsOn,
            Self::Ownership => GraphRelationKind::Owns,
            Self::Load => GraphRelationKind::Loads,
            Self::Inheritance => GraphRelationKind::Inherits,
            Self::Mixin => GraphRelationKind::MixesIn,
            Self::FactoryCreation => GraphRelationKind::FactoryCreates,
            Self::ApiUse => GraphRelationKind::UsesApi,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::DirectCall => "direct-call",
            Self::StateRead => "state-read",
            Self::StateWrite => "state-write",
            Self::NativeEventRegistration => "native-event-registration",
            Self::NativeEventHandler => "native-event-handler",
            Self::NativeEventBridge => "native-event-bridge",
            Self::CustomSignalEmit => "custom-signal-emit",
            Self::CustomSignalHandle => "custom-signal-handle",
            Self::CvarCallbackRegistration => "cvar-callback-registration",
            Self::ScriptAssignment => "script-assignment",
            Self::ScriptHook => "script-hook",
            Self::SecureFunctionHook => "secure-function-hook",
            Self::Dependency => "dependency",
            Self::Ownership => "ownership",
            Self::Load => "load",
            Self::Inheritance => "inheritance",
            Self::Mixin => "mixin",
            Self::FactoryCreation => "factory-creation",
            Self::ApiUse => "api-use",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOrigin {
    AnalyzerFact,
    ProjectFact,
    ReferenceFact,
    ReviewedFixture,
    ExternalCandidate,
}

impl ObservationOrigin {
    const fn confidence_ceiling(self) -> GraphConfidence {
        match self {
            Self::AnalyzerFact | Self::ProjectFact | Self::ReferenceFact => {
                GraphConfidence::Derived
            }
            Self::ReviewedFixture => GraphConfidence::Derived,
            Self::ExternalCandidate => GraphConfidence::Candidate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerLimits {
    pub max_observations: u32,
    pub max_assertions: u32,
    pub max_coverage_records: u32,
    pub max_evidence_per_observation: u32,
}

impl RecognizerLimits {
    pub fn new(
        max_observations: u32,
        max_assertions: u32,
        max_coverage_records: u32,
        max_evidence_per_observation: u32,
    ) -> RecognizerResult<Self> {
        let limits = Self {
            max_observations,
            max_assertions,
            max_coverage_records,
            max_evidence_per_observation,
        };
        limits.validate()?;
        Ok(limits)
    }

    pub(crate) fn validate(self) -> RecognizerResult<()> {
        if self.max_observations == 0
            || self.max_observations > 1_000_000
            || self.max_assertions == 0
            || self.max_assertions > 1_000_000
            || self.max_coverage_records < ObservationFamily::ALL.len() as u32
            || self.max_coverage_records > 1024
            || self.max_evidence_per_observation == 0
            || self.max_evidence_per_observation > 256
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::LimitsInvalid,
                "recognizer limits are outside the reviewed E2-B profile",
            ));
        }
        Ok(())
    }
}

impl Default for RecognizerLimits {
    fn default() -> Self {
        Self {
            max_observations: 100_000,
            max_assertions: 100_000,
            max_coverage_records: 128,
            max_evidence_per_observation: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredObservation {
    observation_id: StructuredObservationId,
    source_snapshot_id: GraphSnapshotId,
    family: ObservationFamily,
    from: GraphNodeId,
    to: GraphNodeId,
    origin: ObservationOrigin,
    confidence: GraphConfidence,
    evidence_ids: Vec<Box<str>>,
}

impl StructuredObservation {
    pub fn new(
        source_snapshot_id: GraphSnapshotId,
        family: ObservationFamily,
        from: GraphNodeId,
        to: GraphNodeId,
        origin: ObservationOrigin,
        confidence: GraphConfidence,
        evidence_ids: Vec<Box<str>>,
        limits: RecognizerLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        if from == to {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationInvalid,
                "recognizer observations cannot be self-relations",
            ));
        }
        let evidence_ids = normalize_ids(
            evidence_ids,
            limits.max_evidence_per_observation as usize,
            RecognizerErrorCode::ObservationInvalid,
        )?;
        let observation_id = observation_id(
            &source_snapshot_id,
            family,
            &from,
            &to,
            origin,
            confidence,
            &evidence_ids,
        )?;
        Ok(Self {
            observation_id,
            source_snapshot_id,
            family,
            from,
            to,
            origin,
            confidence,
            evidence_ids,
        })
    }

    pub fn validate(&self, limits: RecognizerLimits) -> RecognizerResult<()> {
        let rebuilt = Self::new(
            self.source_snapshot_id.clone(),
            self.family,
            self.from.clone(),
            self.to.clone(),
            self.origin,
            self.confidence,
            self.evidence_ids.clone(),
            limits,
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationInvalid,
                "structured observation identity or canonical evidence does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn observation_id(&self) -> &StructuredObservationId {
        &self.observation_id
    }

    #[must_use]
    pub fn source_snapshot_id(&self) -> &GraphSnapshotId {
        &self.source_snapshot_id
    }

    #[must_use]
    pub const fn family(&self) -> ObservationFamily {
        self.family
    }

    #[must_use]
    pub fn from(&self) -> &GraphNodeId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &GraphNodeId {
        &self.to
    }

    #[must_use]
    pub const fn origin(&self) -> ObservationOrigin {
        self.origin
    }

    #[must_use]
    pub const fn confidence(&self) -> GraphConfidence {
        self.confidence
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerDescriptor {
    recognizer_id: RecognizerId,
    version: RecognizerVersion,
    family: ObservationFamily,
    relation: GraphRelationKind,
    confidence_ceiling: GraphConfidence,
}

impl RecognizerDescriptor {
    pub fn new(
        recognizer_id: RecognizerId,
        version: RecognizerVersion,
        family: ObservationFamily,
        relation: GraphRelationKind,
        confidence_ceiling: GraphConfidence,
    ) -> RecognizerResult<Self> {
        if relation != family.relation() || confidence_ceiling == GraphConfidence::Proven {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryIncomplete,
                "recognizer descriptor relation or confidence ceiling is invalid",
            ));
        }
        Ok(Self {
            recognizer_id,
            version,
            family,
            relation,
            confidence_ceiling,
        })
    }

    #[must_use]
    pub fn recognizer_id(&self) -> &RecognizerId {
        &self.recognizer_id
    }

    #[must_use]
    pub fn version(&self) -> &RecognizerVersion {
        &self.version
    }

    #[must_use]
    pub const fn family(&self) -> ObservationFamily {
        self.family
    }

    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }

    #[must_use]
    pub const fn confidence_ceiling(&self) -> GraphConfidence {
        self.confidence_ceiling
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognizerRegistry {
    schema: Box<str>,
    registry_id: RecognizerRegistryId,
    descriptors: Vec<RecognizerDescriptor>,
}

impl RecognizerRegistry {
    pub fn e2_default() -> RecognizerResult<Self> {
        let descriptors = ObservationFamily::ALL
            .into_iter()
            .map(|family| {
                RecognizerDescriptor::new(
                    RecognizerId::new(format!("wow.{0}", family.slug()))?,
                    RecognizerVersion::new("1")?,
                    family,
                    family.relation(),
                    GraphConfidence::Derived,
                )
            })
            .collect::<RecognizerResult<Vec<_>>>()?;
        Self::build(descriptors)
    }

    pub fn build(mut descriptors: Vec<RecognizerDescriptor>) -> RecognizerResult<Self> {
        if descriptors.is_empty() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryEmpty,
                "recognizer registry cannot be empty",
            ));
        }
        descriptors.sort_by_key(RecognizerDescriptor::family);
        if descriptors
            .windows(2)
            .any(|pair| pair[0].family() == pair[1].family())
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryDuplicate,
                "recognizer registry contains duplicate observation families",
            ));
        }
        let families = descriptors
            .iter()
            .map(RecognizerDescriptor::family)
            .collect::<Vec<_>>();
        if families != ObservationFamily::ALL {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryIncomplete,
                "recognizer registry must cover every E2-B observation family",
            ));
        }
        let registry_id = registry_id(&descriptors)?;
        Ok(Self {
            schema: RECOGNIZER_REGISTRY_SCHEMA.into(),
            registry_id,
            descriptors,
        })
    }

    pub fn validate(&self) -> RecognizerResult<()> {
        if self.schema.as_ref() != RECOGNIZER_REGISTRY_SCHEMA {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryIdentityMismatch,
                "recognizer registry schema is unsupported",
            ));
        }
        let rebuilt = Self::build(self.descriptors.clone())?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::RegistryIdentityMismatch,
                "recognizer registry identity or canonical order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn registry_id(&self) -> &RecognizerRegistryId {
        &self.registry_id
    }

    #[must_use]
    pub fn descriptors(&self) -> &[RecognizerDescriptor] {
        &self.descriptors
    }

    #[must_use]
    pub(crate) fn descriptor(
        &self,
        family: ObservationFamily,
    ) -> Option<&RecognizerDescriptor> {
        self.descriptors
            .binary_search_by_key(&family, RecognizerDescriptor::family)
            .ok()
            .map(|index| &self.descriptors[index])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognitionCoverageState {
    Complete,
    Partial,
    NotEvaluated,
    Failed,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognitionCoverage {
    family: ObservationFamily,
    state: RecognitionCoverageState,
    blocker_ids: Vec<Box<str>>,
}

impl RecognitionCoverage {
    pub fn new(
        family: ObservationFamily,
        state: RecognitionCoverageState,
        blocker_ids: Vec<Box<str>>,
        limits: RecognizerLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        let blocker_ids = normalize_ids(
            blocker_ids,
            limits.max_evidence_per_observation as usize,
            RecognizerErrorCode::CoverageInvalid,
        )?;
        if state == RecognitionCoverageState::Failed && blocker_ids.is_empty() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::CoverageInvalid,
                "failed recognizer coverage requires a blocker",
            ));
        }
        if state == RecognitionCoverageState::Complete && !blocker_ids.is_empty() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::CoverageInvalid,
                "complete recognizer coverage cannot retain blockers",
            ));
        }
        Ok(Self {
            family,
            state,
            blocker_ids,
        })
    }

    pub fn validate(&self, limits: RecognizerLimits) -> RecognizerResult<()> {
        if Self::new(self.family, self.state, self.blocker_ids.clone(), limits)? != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::CoverageInvalid,
                "recognizer coverage is not canonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn family(&self) -> ObservationFamily {
        self.family
    }

    #[must_use]
    pub const fn state(&self) -> RecognitionCoverageState {
        self.state
    }

    #[must_use]
    pub fn blocker_ids(&self) -> &[Box<str>] {
        &self.blocker_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognitionAssertion {
    assertion_id: RecognitionAssertionId,
    recognizer_id: RecognizerId,
    recognizer_version: RecognizerVersion,
    observation_id: StructuredObservationId,
    source_snapshot_id: GraphSnapshotId,
    from: GraphNodeId,
    to: GraphNodeId,
    relation: GraphRelationKind,
    confidence: GraphConfidence,
    evidence_ids: Vec<Box<str>>,
}

impl RecognitionAssertion {
    pub(crate) fn from_observation(
        descriptor: &RecognizerDescriptor,
        observation: &StructuredObservation,
    ) -> RecognizerResult<Self> {
        let confidence = weakest_confidence([
            observation.confidence(),
            observation.origin().confidence_ceiling(),
            descriptor.confidence_ceiling(),
        ]);
        let mut evidence_ids = observation.evidence_ids().to_vec();
        evidence_ids.push(observation.observation_id().as_str().into());
        evidence_ids.sort();
        evidence_ids.dedup();
        let assertion_id = assertion_id(descriptor, observation, confidence, &evidence_ids)?;
        Ok(Self {
            assertion_id,
            recognizer_id: descriptor.recognizer_id().clone(),
            recognizer_version: descriptor.version().clone(),
            observation_id: observation.observation_id().clone(),
            source_snapshot_id: observation.source_snapshot_id().clone(),
            from: observation.from().clone(),
            to: observation.to().clone(),
            relation: descriptor.relation(),
            confidence,
            evidence_ids,
        })
    }

    #[must_use]
    pub fn assertion_id(&self) -> &RecognitionAssertionId {
        &self.assertion_id
    }

    #[must_use]
    pub fn recognizer_id(&self) -> &RecognizerId {
        &self.recognizer_id
    }

    #[must_use]
    pub fn recognizer_version(&self) -> &RecognizerVersion {
        &self.recognizer_version
    }

    #[must_use]
    pub fn observation_id(&self) -> &StructuredObservationId {
        &self.observation_id
    }

    #[must_use]
    pub fn source_snapshot_id(&self) -> &GraphSnapshotId {
        &self.source_snapshot_id
    }

    #[must_use]
    pub fn from(&self) -> &GraphNodeId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &GraphNodeId {
        &self.to
    }

    #[must_use]
    pub const fn relation(&self) -> GraphRelationKind {
        self.relation
    }

    #[must_use]
    pub const fn confidence(&self) -> GraphConfidence {
        self.confidence
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognitionReport {
    schema: Box<str>,
    report_id: RecognitionReportId,
    registry_id: RecognizerRegistryId,
    source_snapshot_id: GraphSnapshotId,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    limits: RecognizerLimits,
    assertions: Vec<RecognitionAssertion>,
    coverage: Vec<RecognitionCoverage>,
}

impl RecognitionReport {
    pub(crate) fn build(
        registry_id: RecognizerRegistryId,
        source_snapshot_id: GraphSnapshotId,
        universe: GraphUniverseId,
        generation: GraphGenerationId,
        limits: RecognizerLimits,
        mut assertions: Vec<RecognitionAssertion>,
        mut coverage: Vec<RecognitionCoverage>,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
        assertions.sort_by(|left, right| left.assertion_id().cmp(right.assertion_id()));
        if assertions
            .windows(2)
            .any(|pair| pair[0].assertion_id() == pair[1].assertion_id())
        {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AssertionDuplicate,
                "recognition report contains duplicate assertions",
            ));
        }
        coverage.sort_by_key(RecognitionCoverage::family);
        let report_id = report_id(
            &registry_id,
            &source_snapshot_id,
            &universe,
            &generation,
            limits,
            &assertions,
            &coverage,
        )?;
        Ok(Self {
            schema: RECOGNITION_REPORT_SCHEMA.into(),
            report_id,
            registry_id,
            source_snapshot_id,
            universe,
            generation,
            limits,
            assertions,
            coverage,
        })
    }

    pub fn validate(&self) -> RecognizerResult<()> {
        if self.schema.as_ref() != RECOGNITION_REPORT_SCHEMA {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ReportIdentityMismatch,
                "recognition report schema is unsupported",
            ));
        }
        let rebuilt = Self::build(
            self.registry_id.clone(),
            self.source_snapshot_id.clone(),
            self.universe.clone(),
            self.generation.clone(),
            self.limits,
            self.assertions.clone(),
            self.coverage.clone(),
        )?;
        if rebuilt != *self {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ReportIdentityMismatch,
                "recognition report identity or canonical order does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn report_id(&self) -> &RecognitionReportId {
        &self.report_id
    }

    #[must_use]
    pub fn registry_id(&self) -> &RecognizerRegistryId {
        &self.registry_id
    }

    #[must_use]
    pub fn source_snapshot_id(&self) -> &GraphSnapshotId {
        &self.source_snapshot_id
    }

    #[must_use]
    pub fn universe(&self) -> &GraphUniverseId {
        &self.universe
    }

    #[must_use]
    pub fn generation(&self) -> &GraphGenerationId {
        &self.generation
    }

    #[must_use]
    pub const fn limits(&self) -> RecognizerLimits {
        self.limits
    }

    #[must_use]
    pub fn assertions(&self) -> &[RecognitionAssertion] {
        &self.assertions
    }

    #[must_use]
    pub fn coverage(&self) -> &[RecognitionCoverage] {
        &self.coverage
    }
}

fn observation_id(
    source_snapshot_id: &GraphSnapshotId,
    family: ObservationFamily,
    from: &GraphNodeId,
    to: &GraphNodeId,
    origin: ObservationOrigin,
    confidence: GraphConfidence,
    evidence_ids: &[Box<str>],
) -> RecognizerResult<StructuredObservationId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        source_snapshot_id: &'a GraphSnapshotId,
        family: ObservationFamily,
        from: &'a GraphNodeId,
        to: &'a GraphNodeId,
        origin: ObservationOrigin,
        confidence: GraphConfidence,
        evidence_ids: &'a [Box<str>],
    }
    let bytes = canonical_json_bytes(&Identity {
        source_snapshot_id,
        family,
        from,
        to,
        origin,
        confidence,
        evidence_ids,
    })
    .map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::ObservationInvalid,
            "structured observation identity cannot be canonicalized",
        )
    })?;
    StructuredObservationId::new(format!(
        "recognizer-observation:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn assertion_id(
    descriptor: &RecognizerDescriptor,
    observation: &StructuredObservation,
    confidence: GraphConfidence,
    evidence_ids: &[Box<str>],
) -> RecognizerResult<RecognitionAssertionId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        recognizer_id: &'a RecognizerId,
        recognizer_version: &'a RecognizerVersion,
        observation_id: &'a StructuredObservationId,
        source_snapshot_id: &'a GraphSnapshotId,
        from: &'a GraphNodeId,
        to: &'a GraphNodeId,
        relation: GraphRelationKind,
        confidence: GraphConfidence,
        evidence_ids: &'a [Box<str>],
    }
    let bytes = canonical_json_bytes(&Identity {
        recognizer_id: descriptor.recognizer_id(),
        recognizer_version: descriptor.version(),
        observation_id: observation.observation_id(),
        source_snapshot_id: observation.source_snapshot_id(),
        from: observation.from(),
        to: observation.to(),
        relation: descriptor.relation(),
        confidence,
        evidence_ids,
    })
    .map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::AssertionInvalid,
            "recognition assertion identity cannot be canonicalized",
        )
    })?;
    RecognitionAssertionId::new(format!(
        "recognizer-assertion:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn registry_id(descriptors: &[RecognizerDescriptor]) -> RecognizerResult<RecognizerRegistryId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        descriptors: &'a [RecognizerDescriptor],
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: RECOGNIZER_REGISTRY_SCHEMA,
        descriptors,
    })
    .map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::RegistryIdentityMismatch,
            "recognizer registry identity cannot be canonicalized",
        )
    })?;
    RecognizerRegistryId::new(format!(
        "recognizer-registry:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn report_id(
    registry_id: &RecognizerRegistryId,
    source_snapshot_id: &GraphSnapshotId,
    universe: &GraphUniverseId,
    generation: &GraphGenerationId,
    limits: RecognizerLimits,
    assertions: &[RecognitionAssertion],
    coverage: &[RecognitionCoverage],
) -> RecognizerResult<RecognitionReportId> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        registry_id: &'a RecognizerRegistryId,
        source_snapshot_id: &'a GraphSnapshotId,
        universe: &'a GraphUniverseId,
        generation: &'a GraphGenerationId,
        limits: RecognizerLimits,
        assertions: &'a [RecognitionAssertion],
        coverage: &'a [RecognitionCoverage],
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: RECOGNITION_REPORT_SCHEMA,
        registry_id,
        source_snapshot_id,
        universe,
        generation,
        limits,
        assertions,
        coverage,
    })
    .map_err(|_| {
        RecognizerError::new(
            RecognizerErrorCode::ReportIdentityMismatch,
            "recognition report identity cannot be canonicalized",
        )
    })?;
    RecognitionReportId::new(format!(
        "recognition-report:sha256:{}",
        hex(&Sha256::digest(bytes))
    ))
}

fn normalize_ids(
    mut values: Vec<Box<str>>,
    max: usize,
    code: RecognizerErrorCode,
) -> RecognizerResult<Vec<Box<str>>> {
    if values.len() > max {
        return Err(RecognizerError::new(
            RecognizerErrorCode::BudgetExceeded,
            "recognizer evidence count exceeds configured limits",
        ));
    }
    if values.iter().any(|value| {
        value.is_empty()
            || value.len() > 512
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
            })
    }) {
        return Err(RecognizerError::new(
            code,
            "recognizer evidence or blocker identity is invalid",
        ));
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(RecognizerError::new(
            code,
            "recognizer evidence or blocker identities must be unique",
        ));
    }
    Ok(values)
}

pub(crate) fn weakest_confidence<const N: usize>(
    values: [GraphConfidence; N],
) -> GraphConfidence {
    values
        .into_iter()
        .max_by_key(|confidence| confidence_rank(*confidence))
        .unwrap_or(GraphConfidence::Candidate)
}

const fn confidence_rank(confidence: GraphConfidence) -> u8 {
    match confidence {
        GraphConfidence::Proven => 0,
        GraphConfidence::Derived => 1,
        GraphConfidence::Possible => 2,
        GraphConfidence::Candidate => 3,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_ceiling_never_strengthens_an_observation() {
        assert_eq!(
            weakest_confidence([
                GraphConfidence::Proven,
                GraphConfidence::Derived,
                GraphConfidence::Possible,
            ]),
            GraphConfidence::Possible
        );
        assert_eq!(
            weakest_confidence([
                GraphConfidence::Proven,
                GraphConfidence::Candidate,
                GraphConfidence::Derived,
            ]),
            GraphConfidence::Candidate
        );
    }
}
