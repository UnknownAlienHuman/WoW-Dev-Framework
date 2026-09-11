use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::identity::{validate_digest, validate_identifier};
use crate::{OperationId, ServiceError, ServiceErrorCode, ServiceResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentHealth {
    Ready,
    Degraded,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    Partial,
    Deferred,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSnapshot {
    component_id: Box<str>,
    producer_version: Box<str>,
    exact_identity: Box<str>,
    health: ComponentHealth,
    capabilities: BTreeMap<Box<str>, CapabilityState>,
    last_known_good_identity: Option<Box<str>>,
    failed_target_identity: Option<Box<str>>,
}

impl ComponentSnapshot {
    pub fn new(
        component_id: impl Into<Box<str>>,
        producer_version: impl Into<Box<str>>,
        exact_identity: impl Into<Box<str>>,
        health: ComponentHealth,
    ) -> ServiceResult<Self> {
        let component_id = component_id.into();
        let producer_version = producer_version.into();
        let exact_identity = exact_identity.into();
        validate_identifier(&component_id, "component_id")?;
        validate_identifier(&producer_version, "producer_version")?;
        validate_identifier(&exact_identity, "component exact identity")?;
        Ok(Self {
            component_id,
            producer_version,
            exact_identity,
            health,
            capabilities: BTreeMap::new(),
            last_known_good_identity: None,
            failed_target_identity: None,
        })
    }

    pub fn with_capability(
        mut self,
        capability_id: impl Into<Box<str>>,
        state: CapabilityState,
    ) -> ServiceResult<Self> {
        let capability_id = capability_id.into();
        validate_identifier(&capability_id, "capability_id")?;
        if self.capabilities.insert(capability_id, state).is_some() {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "duplicate component capability",
            ));
        }
        Ok(self)
    }

    pub fn with_last_known_good(
        mut self,
        identity: impl Into<Box<str>>,
    ) -> ServiceResult<Self> {
        let identity = identity.into();
        validate_identifier(&identity, "last-known-good identity")?;
        self.last_known_good_identity = Some(identity);
        Ok(self)
    }

    pub fn with_failed_target(
        mut self,
        identity: impl Into<Box<str>>,
    ) -> ServiceResult<Self> {
        let identity = identity.into();
        validate_identifier(&identity, "failed target identity")?;
        self.failed_target_identity = Some(identity);
        Ok(self)
    }

    #[must_use]
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
    #[must_use]
    pub fn exact_identity(&self) -> &str {
        &self.exact_identity
    }
    #[must_use]
    pub const fn health(&self) -> ComponentHealth {
        self.health
    }
    #[must_use]
    pub fn capabilities(&self) -> &BTreeMap<Box<str>, CapabilityState> {
        &self.capabilities
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum GenerationSelector {
    Exact(Box<str>),
    CurrentPublished { project_id: Box<str> },
}

impl GenerationSelector {
    pub fn exact(value: impl Into<Box<str>>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier(&value, "exact project generation")?;
        Ok(Self::Exact(value))
    }

    pub fn current_published(project_id: impl Into<Box<str>>) -> ServiceResult<Self> {
        let project_id = project_id.into();
        validate_identifier(&project_id, "project_id selector")?;
        Ok(Self::CurrentPublished { project_id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "paths", rename_all = "snake_case")]
pub enum CheckScope {
    WholeProject,
    Files(Vec<Box<str>>),
}

impl CheckScope {
    pub fn files<I, S>(paths: I) -> ServiceResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        let mut paths = paths.into_iter().map(Into::into).collect::<Vec<_>>();
        for path in &paths {
            validate_path(path)?;
        }
        paths.sort();
        if paths.is_empty() || paths.iter().collect::<BTreeSet<_>>().len() != paths.len() {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidRequest,
                "file scope must be non-empty and unique",
            ));
        }
        Ok(Self::Files(paths))
    }

    #[must_use]
    pub fn file_count(&self) -> Option<usize> {
        match self {
            Self::WholeProject => None,
            Self::Files(paths) => Some(paths.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatusRequest {
    operation_id: OperationId,
}

impl StatusRequest {
    #[must_use]
    pub const fn new(operation_id: OperationId) -> Self {
        Self { operation_id }
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRequest {
    operation_id: OperationId,
    selector: GenerationSelector,
    scope: CheckScope,
}

impl CheckRequest {
    #[must_use]
    pub const fn new(
        operation_id: OperationId,
        selector: GenerationSelector,
        scope: CheckScope,
    ) -> Self {
        Self {
            operation_id,
            selector,
            scope,
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    #[must_use]
    pub const fn selector(&self) -> &GenerationSelector {
        &self.selector
    }
    #[must_use]
    pub const fn scope(&self) -> &CheckScope {
        &self.scope
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextIdentity {
    configuration_id: Box<str>,
    project_id: Box<str>,
    profile_id: Box<str>,
    reference_generation_id: Box<str>,
    project_generation_id: Box<str>,
    project_snapshot_id: Box<str>,
    analyzer_snapshot_id: Box<str>,
    analyzer_pin_id: Box<str>,
    rule_registry_id: Box<str>,
}

impl ContextIdentity {
    pub fn builder() -> ContextIdentityBuilder {
        ContextIdentityBuilder::default()
    }

    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }
    #[must_use]
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }
    #[must_use]
    pub fn reference_generation_id(&self) -> &str {
        &self.reference_generation_id
    }
    #[must_use]
    pub fn project_generation_id(&self) -> &str {
        &self.project_generation_id
    }
    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.project_snapshot_id
    }
    #[must_use]
    pub fn analyzer_snapshot_id(&self) -> &str {
        &self.analyzer_snapshot_id
    }
    #[must_use]
    pub fn analyzer_pin_id(&self) -> &str {
        &self.analyzer_pin_id
    }
    #[must_use]
    pub fn rule_registry_id(&self) -> &str {
        &self.rule_registry_id
    }
}

#[derive(Debug, Default)]
pub struct ContextIdentityBuilder {
    configuration_id: Option<Box<str>>,
    project_id: Option<Box<str>>,
    profile_id: Option<Box<str>>,
    reference_generation_id: Option<Box<str>>,
    project_generation_id: Option<Box<str>>,
    project_snapshot_id: Option<Box<str>>,
    analyzer_snapshot_id: Option<Box<str>>,
    analyzer_pin_id: Option<Box<str>>,
    rule_registry_id: Option<Box<str>>,
}

macro_rules! context_builder_setter {
    ($name:ident) => {
        #[must_use]
        pub fn $name(mut self, value: impl Into<Box<str>>) -> Self {
            self.$name = Some(value.into());
            self
        }
    };
}

impl ContextIdentityBuilder {
    context_builder_setter!(configuration_id);
    context_builder_setter!(project_id);
    context_builder_setter!(profile_id);
    context_builder_setter!(reference_generation_id);
    context_builder_setter!(project_generation_id);
    context_builder_setter!(project_snapshot_id);
    context_builder_setter!(analyzer_snapshot_id);
    context_builder_setter!(analyzer_pin_id);
    context_builder_setter!(rule_registry_id);

    pub fn build(self) -> ServiceResult<ContextIdentity> {
        Ok(ContextIdentity {
            configuration_id: context_field(self.configuration_id, "configuration_id")?,
            project_id: context_field(self.project_id, "project_id")?,
            profile_id: context_field(self.profile_id, "profile_id")?,
            reference_generation_id: context_field(
                self.reference_generation_id,
                "reference_generation_id",
            )?,
            project_generation_id: context_field(
                self.project_generation_id,
                "project_generation_id",
            )?,
            project_snapshot_id: context_field(self.project_snapshot_id, "project_snapshot_id")?,
            analyzer_snapshot_id: context_field(
                self.analyzer_snapshot_id,
                "analyzer_snapshot_id",
            )?,
            analyzer_pin_id: context_field(self.analyzer_pin_id, "analyzer_pin_id")?,
            rule_registry_id: context_field(self.rule_registry_id, "rule_registry_id")?,
        })
    }
}

fn context_field(value: Option<Box<str>>, field: &str) -> ServiceResult<Box<str>> {
    let value = value.ok_or_else(|| {
        ServiceError::new(
            ServiceErrorCode::InvalidContext,
            format!("missing {field}"),
        )
    })?;
    validate_identifier(&value, field)?;
    Ok(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingOrigin {
    GenericAnalyzer,
    WowRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceSemanticStatus {
    Clean,
    Findings,
    Partial,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExactSourceLocation {
    path: Box<str>,
    content_sha256: Box<str>,
    byte_start: u64,
    byte_end: u64,
}

impl ExactSourceLocation {
    pub fn new(
        path: impl Into<Box<str>>,
        content_sha256: impl Into<Box<str>>,
        byte_start: u64,
        byte_end: u64,
    ) -> ServiceResult<Self> {
        let path = path.into();
        let content_sha256 = content_sha256.into();
        validate_path(&path)?;
        validate_digest(&content_sha256, "source content digest")?;
        if byte_start > byte_end {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "source span is reversed",
            ));
        }
        Ok(Self {
            path,
            content_sha256,
            byte_start,
            byte_end,
        })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }
    #[must_use]
    pub const fn byte_start(&self) -> u64 {
        self.byte_start
    }
    #[must_use]
    pub const fn byte_end(&self) -> u64 {
        self.byte_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GenericFinding {
    finding_id: Box<str>,
    category: Box<str>,
    upstream_code: Box<str>,
    severity: Box<str>,
    location: ExactSourceLocation,
}

impl GenericFinding {
    pub fn new(
        finding_id: impl Into<Box<str>>,
        category: impl Into<Box<str>>,
        upstream_code: impl Into<Box<str>>,
        severity: impl Into<Box<str>>,
        location: ExactSourceLocation,
    ) -> ServiceResult<Self> {
        let finding_id = finding_id.into();
        let category = category.into();
        let upstream_code = upstream_code.into();
        let severity = severity.into();
        validate_identifier(&finding_id, "generic finding_id")?;
        validate_identifier(&category, "generic category")?;
        validate_identifier(&upstream_code, "generic upstream code")?;
        validate_identifier(&severity, "generic severity")?;
        Ok(Self {
            finding_id,
            category,
            upstream_code,
            severity,
            location,
        })
    }

    #[must_use]
    pub fn finding_id(&self) -> &str {
        &self.finding_id
    }
    #[must_use]
    pub const fn location(&self) -> &ExactSourceLocation {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleFinding {
    finding_id: Box<str>,
    rule_id: Box<str>,
    category: Box<str>,
    severity: Box<str>,
    location: ExactSourceLocation,
    source_handle_id: Box<str>,
    evidence_ids: Vec<Box<str>>,
}

impl RuleFinding {
    pub fn new(
        finding_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        category: impl Into<Box<str>>,
        severity: impl Into<Box<str>>,
        location: ExactSourceLocation,
        source_handle_id: impl Into<Box<str>>,
        mut evidence_ids: Vec<Box<str>>,
    ) -> ServiceResult<Self> {
        let finding_id = finding_id.into();
        let rule_id = rule_id.into();
        let category = category.into();
        let severity = severity.into();
        let source_handle_id = source_handle_id.into();
        for (value, field) in [
            (&finding_id, "rule finding_id"),
            (&rule_id, "rule_id"),
            (&category, "rule category"),
            (&severity, "rule severity"),
            (&source_handle_id, "source_handle_id"),
        ] {
            validate_identifier(value, field)?;
        }
        evidence_ids.sort();
        if evidence_ids.is_empty()
            || evidence_ids.iter().collect::<BTreeSet<_>>().len() != evidence_ids.len()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "rule finding evidence must be non-empty and unique",
            ));
        }
        for evidence_id in &evidence_ids {
            validate_identifier(evidence_id, "evidence_id")?;
        }
        Ok(Self {
            finding_id,
            rule_id,
            category,
            severity,
            location,
            source_handle_id,
            evidence_ids,
        })
    }

    #[must_use]
    pub fn finding_id(&self) -> &str {
        &self.finding_id
    }
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }
    #[must_use]
    pub const fn location(&self) -> &ExactSourceLocation {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(tag = "origin", content = "finding", rename_all = "snake_case")]
pub enum RawFinding {
    Generic(GenericFinding),
    Rule(RuleFinding),
}

impl RawFinding {
    #[must_use]
    pub fn finding_id(&self) -> &str {
        match self {
            Self::Generic(finding) => finding.finding_id(),
            Self::Rule(finding) => finding.finding_id(),
        }
    }

    #[must_use]
    pub const fn origin(&self) -> FindingOrigin {
        match self {
            Self::Generic(_) => FindingOrigin::GenericAnalyzer,
            Self::Rule(_) => FindingOrigin::WowRule,
        }
    }

    #[must_use]
    pub const fn location(&self) -> &ExactSourceLocation {
        match self {
            Self::Generic(finding) => finding.location(),
            Self::Rule(finding) => finding.location(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanEvaluation {
    record_id: Box<str>,
    rule_id: Box<str>,
    scope_id: Box<str>,
    claim: Box<str>,
    coverage_ids: Vec<Box<str>>,
}

impl CleanEvaluation {
    pub fn new(
        record_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        claim: impl Into<Box<str>>,
        mut coverage_ids: Vec<Box<str>>,
    ) -> ServiceResult<Self> {
        let record_id = record_id.into();
        let rule_id = rule_id.into();
        let scope_id = scope_id.into();
        let claim = claim.into();
        for (value, field) in [
            (&record_id, "clean record_id"),
            (&rule_id, "clean rule_id"),
            (&scope_id, "clean scope_id"),
            (&claim, "clean claim"),
        ] {
            validate_identifier(value, field)?;
        }
        coverage_ids.sort();
        if coverage_ids.is_empty()
            || coverage_ids.iter().collect::<BTreeSet<_>>().len() != coverage_ids.len()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "clean coverage must be non-empty and unique",
            ));
        }
        Ok(Self {
            record_id,
            rule_id,
            scope_id,
            claim,
            coverage_ids,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockerKind {
    MissingCapability,
    IncompleteCoverage,
    ConflictingEvidence,
    ComponentFailed,
    BudgetTruncated,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleBlocker {
    blocker_id: Box<str>,
    kind: BlockerKind,
    capability_id: Option<Box<str>>,
}

impl RuleBlocker {
    pub fn new(
        blocker_id: impl Into<Box<str>>,
        kind: BlockerKind,
        capability_id: Option<Box<str>>,
    ) -> ServiceResult<Self> {
        let blocker_id = blocker_id.into();
        validate_identifier(&blocker_id, "blocker_id")?;
        if let Some(capability_id) = &capability_id {
            validate_identifier(capability_id, "blocked capability_id")?;
        }
        Ok(Self {
            blocker_id,
            kind,
            capability_id,
        })
    }

    #[must_use]
    pub fn blocker_id(&self) -> &str {
        &self.blocker_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleEvaluationState {
    Findings,
    EvaluatedClean,
    NotEvaluated,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluation {
    evaluation_id: Box<str>,
    rule_id: Box<str>,
    scope_id: Box<str>,
    state: RuleEvaluationState,
    findings: Vec<RuleFinding>,
    clean_records: Vec<CleanEvaluation>,
    blocker: Option<RuleBlocker>,
    failure_code: Option<Box<str>>,
    degradable: bool,
}

impl RuleEvaluation {
    pub fn findings(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        findings: Vec<RuleFinding>,
    ) -> ServiceResult<Self> {
        if findings.is_empty() {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "findings evaluation is empty",
            ));
        }
        Self::build(
            evaluation_id,
            rule_id,
            scope_id,
            RuleEvaluationState::Findings,
            findings,
            Vec::new(),
            None,
            None,
            false,
        )
    }

    pub fn clean(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        clean_records: Vec<CleanEvaluation>,
    ) -> ServiceResult<Self> {
        if clean_records.is_empty() {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "clean evaluation has no authority record",
            ));
        }
        Self::build(
            evaluation_id,
            rule_id,
            scope_id,
            RuleEvaluationState::EvaluatedClean,
            Vec::new(),
            clean_records,
            None,
            None,
            false,
        )
    }

    pub fn not_evaluated(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        blocker: RuleBlocker,
    ) -> ServiceResult<Self> {
        Self::build(
            evaluation_id,
            rule_id,
            scope_id,
            RuleEvaluationState::NotEvaluated,
            Vec::new(),
            Vec::new(),
            Some(blocker),
            None,
            true,
        )
    }

    pub fn failed(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        failure_code: impl Into<Box<str>>,
        degradable: bool,
    ) -> ServiceResult<Self> {
        Self::build(
            evaluation_id,
            rule_id,
            scope_id,
            RuleEvaluationState::Failed,
            Vec::new(),
            Vec::new(),
            None,
            Some(failure_code.into()),
            degradable,
        )
    }

    pub fn cancelled(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
    ) -> ServiceResult<Self> {
        Self::build(
            evaluation_id,
            rule_id,
            scope_id,
            RuleEvaluationState::Cancelled,
            Vec::new(),
            Vec::new(),
            None,
            None,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        evaluation_id: impl Into<Box<str>>,
        rule_id: impl Into<Box<str>>,
        scope_id: impl Into<Box<str>>,
        state: RuleEvaluationState,
        mut findings: Vec<RuleFinding>,
        mut clean_records: Vec<CleanEvaluation>,
        blocker: Option<RuleBlocker>,
        failure_code: Option<Box<str>>,
        degradable: bool,
    ) -> ServiceResult<Self> {
        let evaluation_id = evaluation_id.into();
        let rule_id = rule_id.into();
        let scope_id = scope_id.into();
        for (value, field) in [
            (&evaluation_id, "evaluation_id"),
            (&rule_id, "rule_id"),
            (&scope_id, "rule scope_id"),
        ] {
            validate_identifier(value, field)?;
        }
        if let Some(failure_code) = &failure_code {
            validate_identifier(failure_code, "rule failure_code")?;
        }
        findings.sort();
        clean_records.sort();
        Ok(Self {
            evaluation_id,
            rule_id,
            scope_id,
            state,
            findings,
            clean_records,
            blocker,
            failure_code,
            degradable,
        })
    }

    #[must_use]
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }
    #[must_use]
    pub const fn state(&self) -> RuleEvaluationState {
        self.state
    }
    #[must_use]
    pub fn findings_slice(&self) -> &[RuleFinding] {
        &self.findings
    }
    #[must_use]
    pub fn clean_records(&self) -> &[CleanEvaluation] {
        &self.clean_records
    }
    #[must_use]
    pub const fn blocker(&self) -> Option<&RuleBlocker> {
        self.blocker.as_ref()
    }
    #[must_use]
    pub const fn degradable(&self) -> bool {
        self.degradable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationRelationKind {
    CausesOrExplains,
    BlockedBy,
    ExactDuplicateOf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CausalRelation {
    parent_id: Box<str>,
    child_id: Box<str>,
    kind: PresentationRelationKind,
}

impl CausalRelation {
    pub fn new(
        parent_id: impl Into<Box<str>>,
        child_id: impl Into<Box<str>>,
        kind: PresentationRelationKind,
    ) -> ServiceResult<Self> {
        let parent_id = parent_id.into();
        let child_id = child_id.into();
        validate_identifier(&parent_id, "causal parent_id")?;
        validate_identifier(&child_id, "causal child_id")?;
        if parent_id == child_id {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "causal relation is self-referential",
            ));
        }
        Ok(Self {
            parent_id,
            child_id,
            kind,
        })
    }

    #[must_use]
    pub fn parent_id(&self) -> &str {
        &self.parent_id
    }
    #[must_use]
    pub fn child_id(&self) -> &str {
        &self.child_id
    }
    #[must_use]
    pub const fn kind(&self) -> PresentationRelationKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationNodeKind {
    Finding,
    Blocker,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationNode {
    node_id: Box<str>,
    kind: PresentationNodeKind,
}

impl PresentationNode {
    pub(crate) fn new(node_id: impl Into<Box<str>>, kind: PresentationNodeKind) -> Self {
        Self {
            node_id: node_id.into(),
            kind,
        }
    }

    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationRelation {
    parent_id: Box<str>,
    child_id: Box<str>,
    kind: PresentationRelationKind,
}

impl PresentationRelation {
    pub(crate) fn from_relation(relation: &CausalRelation) -> Self {
        Self {
            parent_id: relation.parent_id.clone(),
            child_id: relation.child_id.clone(),
            kind: relation.kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationGraph {
    nodes: Vec<PresentationNode>,
    display_root_ids: Vec<Box<str>>,
    relations: Vec<PresentationRelation>,
}

impl PresentationGraph {
    pub(crate) const fn new(
        nodes: Vec<PresentationNode>,
        display_root_ids: Vec<Box<str>>,
        relations: Vec<PresentationRelation>,
    ) -> Self {
        Self {
            nodes,
            display_root_ids,
            relations,
        }
    }

    #[must_use]
    pub fn nodes(&self) -> &[PresentationNode] {
        &self.nodes
    }
    #[must_use]
    pub fn display_root_ids(&self) -> &[Box<str>] {
        &self.display_root_ids
    }
    #[must_use]
    pub fn relations(&self) -> &[PresentationRelation] {
        &self.relations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckContext {
    identity: ContextIdentity,
    selected_scope: CheckScope,
    components: Vec<ComponentSnapshot>,
    generic_findings: Vec<GenericFinding>,
    rule_evaluations: Vec<RuleEvaluation>,
    causal_relations: Vec<CausalRelation>,
}

impl CheckContext {
    #[must_use]
    pub const fn new(
        identity: ContextIdentity,
        selected_scope: CheckScope,
        components: Vec<ComponentSnapshot>,
        generic_findings: Vec<GenericFinding>,
        rule_evaluations: Vec<RuleEvaluation>,
        causal_relations: Vec<CausalRelation>,
    ) -> Self {
        Self {
            identity,
            selected_scope,
            components,
            generic_findings,
            rule_evaluations,
            causal_relations,
        }
    }

    #[must_use]
    pub const fn identity(&self) -> &ContextIdentity {
        &self.identity
    }
    #[must_use]
    pub const fn selected_scope(&self) -> &CheckScope {
        &self.selected_scope
    }
    #[must_use]
    pub fn components(&self) -> &[ComponentSnapshot] {
        &self.components
    }
    #[must_use]
    pub fn generic_findings(&self) -> &[GenericFinding] {
        &self.generic_findings
    }
    #[must_use]
    pub fn rule_evaluations(&self) -> &[RuleEvaluation] {
        &self.rule_evaluations
    }
    #[must_use]
    pub fn causal_relations(&self) -> &[CausalRelation] {
        &self.causal_relations
    }
}

fn validate_path(path: &str) -> ServiceResult<()> {
    if path.is_empty()
        || path.len() > 4096
        || path.starts_with('/')
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidRequest,
            "invalid normalized source path",
        ));
    }
    Ok(())
}
