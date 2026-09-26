use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{ProfileId, ProfileKind};
use wow_project::{ProjectFileId, ProjectView};
use wow_reference::ReferenceView;

use crate::descriptor::{
    API_EXISTS_RULE, FIXTURE_POLICY_ID, FIXTURE_PROFILE_ID, RULE_VERSION, SECRET_LOCAL_RULE,
};
use crate::identity::canonical_id;
use crate::{RuleError, RuleErrorCode, RuleRegistry, RuleResult};

/// Closed E0 fixture semantics; never production Secret authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleFixturePolicy {
    policy_id: &'static str,
    profile_id: &'static str,
    rule_versions: [&'static str; 2],
    accepted_guard_callee: &'static str,
    accepted_guard_kind: &'static str,
    supported_operation: &'static str,
    supported_facet: &'static str,
    return_position: u64,
    applicability: &'static str,
    policy_digest: Box<str>,
}

impl RuleFixturePolicy {
    pub fn e0() -> RuleResult<Self> {
        #[derive(Serialize)]
        struct Material<'a> {
            policy_id: &'a str,
            profile_id: &'a str,
            rule_versions: [&'a str; 2],
            accepted_guard_callee: &'a str,
            accepted_guard_kind: &'a str,
            supported_operation: &'a str,
            supported_facet: &'a str,
            return_position: u64,
            applicability: &'a str,
        }
        let material = Material {
            policy_id: FIXTURE_POLICY_ID,
            profile_id: FIXTURE_PROFILE_ID,
            rule_versions: ["wow.api.exists@1", "wow.secret.local_operation@1"],
            accepted_guard_callee: "canaccessvalue",
            accepted_guard_kind: "access_single",
            supported_operation: "concatenation",
            supported_facet: "secret.return",
            return_position: 1,
            applicability: "unconditional_fixture",
        };
        let policy = Self {
            policy_id: material.policy_id,
            profile_id: material.profile_id,
            rule_versions: material.rule_versions,
            accepted_guard_callee: material.accepted_guard_callee,
            accepted_guard_kind: material.accepted_guard_kind,
            supported_operation: material.supported_operation,
            supported_facet: material.supported_facet,
            return_position: material.return_position,
            applicability: material.applicability,
            policy_digest: canonical_id(
                "rule-fixture-policy:sha256:",
                "wow-rules/fixture-policy/e0-e/1",
                &material,
            )?,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> RuleResult<()> {
        let expected = Self::e0_without_validation()?;
        if self == &expected {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleFixturePolicyInvalid,
                "fixture rule policy differs from the closed E0-E policy",
            ))
        }
    }

    fn e0_without_validation() -> RuleResult<Self> {
        #[derive(Serialize)]
        struct Material<'a> {
            policy_id: &'a str,
            profile_id: &'a str,
            rule_versions: [&'a str; 2],
            accepted_guard_callee: &'a str,
            accepted_guard_kind: &'a str,
            supported_operation: &'a str,
            supported_facet: &'a str,
            return_position: u64,
            applicability: &'a str,
        }
        let material = Material {
            policy_id: FIXTURE_POLICY_ID,
            profile_id: FIXTURE_PROFILE_ID,
            rule_versions: ["wow.api.exists@1", "wow.secret.local_operation@1"],
            accepted_guard_callee: "canaccessvalue",
            accepted_guard_kind: "access_single",
            supported_operation: "concatenation",
            supported_facet: "secret.return",
            return_position: 1,
            applicability: "unconditional_fixture",
        };
        Ok(Self {
            policy_id: material.policy_id,
            profile_id: material.profile_id,
            rule_versions: material.rule_versions,
            accepted_guard_callee: material.accepted_guard_callee,
            accepted_guard_kind: material.accepted_guard_kind,
            supported_operation: material.supported_operation,
            supported_facet: material.supported_facet,
            return_position: material.return_position,
            applicability: material.applicability,
            policy_digest: canonical_id(
                "rule-fixture-policy:sha256:",
                "wow-rules/fixture-policy/e0-e/1",
                &material,
            )?,
        })
    }

    #[must_use]
    pub const fn policy_id(&self) -> &str {
        self.policy_id
    }

    #[must_use]
    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }
}

/// Closed production policy for exact native API presence and the first
/// source-backed Secret slice. Only exact `SecretReturns=true` first-return
/// facets and the exact global `canaccessvalue(value)` predicate are admitted.
/// Conditional/aspect/runtime semantics remain unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleProductionPolicy {
    policy_id: Box<str>,
    profile_id: Box<str>,
    rule_versions: [&'static str; 2],
    api_partition_id: &'static str,
    api_fact_scope: &'static str,
    api_presence_authority: &'static str,
    api_absence_authority: &'static str,
    secret_partition_id: &'static str,
    secret_return_payload: &'static str,
    guard_entity: &'static str,
    guard_payload: &'static str,
    guard_callee: &'static str,
    secret_policy: &'static str,
    policy_digest: Box<str>,
}

impl RuleProductionPolicy {
    pub fn native_api(profile_id: &str) -> RuleResult<Self> {
        let policy = Self::build(profile_id)?;
        policy.validate()?;
        Ok(policy)
    }

    fn build(profile_id: &str) -> RuleResult<Self> {
        let parsed = ProfileId::parse(profile_id).map_err(|error| {
            RuleError::new(
                RuleErrorCode::RuleFixturePolicyInvalid,
                format!("production policy profile is invalid: {error}"),
            )
        })?;
        if parsed.value().as_str() != profile_id || profile_id == FIXTURE_PROFILE_ID {
            return Err(RuleError::new(
                RuleErrorCode::RuleFixturePolicyInvalid,
                "production policy requires a canonical non-fixture profile",
            ));
        }
        #[derive(Serialize)]
        struct Material<'a> {
            schema: &'static str,
            profile_id: &'a str,
            rule_versions: [&'static str; 2],
            api_partition_id: &'static str,
            api_fact_scope: &'static str,
            api_presence_authority: &'static str,
            api_absence_authority: &'static str,
            secret_partition_id: &'static str,
            secret_return_payload: &'static str,
            guard_entity: &'static str,
            guard_payload: &'static str,
            guard_callee: &'static str,
            secret_policy: &'static str,
        }
        let material = Material {
            schema: "wow-rules/production-policy/native-api-secret/2",
            profile_id,
            rule_versions: ["wow.api.exists@1", "wow.secret.local_operation@1"],
            api_partition_id: wow_reference::native_view::NATIVE_API_PARTITION,
            api_fact_scope: "direct_static_namespace_member",
            api_presence_authority: "exact_record",
            api_absence_authority: "partition_coverage_only",
            secret_partition_id: wow_reference::native_view::NATIVE_RESTRICTION_PARTITION,
            secret_return_payload: wow_reference::native_view::NATIVE_SECRET_RETURN_PAYLOAD,
            guard_entity: wow_reference::native_view::NATIVE_ACCESS_PREDICATE_ENTITY,
            guard_payload: wow_reference::native_view::NATIVE_ACCESS_PREDICATE_PAYLOAD,
            guard_callee: "canaccessvalue",
            secret_policy: "exact_unconditional_first_return_local_concat_v1",
        };
        let policy_digest =
            canonical_id("rule-production-policy:sha256:", material.schema, &material)?;
        Ok(Self {
            policy_id: policy_digest.clone(),
            profile_id: profile_id.into(),
            rule_versions: material.rule_versions,
            api_partition_id: material.api_partition_id,
            api_fact_scope: material.api_fact_scope,
            api_presence_authority: material.api_presence_authority,
            api_absence_authority: material.api_absence_authority,
            secret_partition_id: material.secret_partition_id,
            secret_return_payload: material.secret_return_payload,
            guard_entity: material.guard_entity,
            guard_payload: material.guard_payload,
            guard_callee: material.guard_callee,
            secret_policy: material.secret_policy,
            policy_digest,
        })
    }

    pub fn validate(&self) -> RuleResult<()> {
        let expected = Self::build(&self.profile_id)?;
        if self == &expected {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleFixturePolicyInvalid,
                "production rule policy differs from the closed native API policy",
            ))
        }
    }

    #[must_use]
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }

    #[must_use]
    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn api_partition_id(&self) -> &str {
        self.api_partition_id
    }

    #[must_use]
    pub const fn secret_partition_id(&self) -> &str {
        self.secret_partition_id
    }

    #[must_use]
    pub const fn secret_return_payload(&self) -> &str {
        self.secret_return_payload
    }

    #[must_use]
    pub const fn guard_entity(&self) -> &str {
        self.guard_entity
    }

    #[must_use]
    pub const fn guard_payload(&self) -> &str {
        self.guard_payload
    }

    #[must_use]
    pub const fn guard_callee(&self) -> &str {
        self.guard_callee
    }
}

/// Bounded synchronous E0 provider budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleExecutionBudget {
    max_evaluations: u64,
    max_findings: u64,
    max_source_handles: u64,
    max_evidence_records: u64,
    max_serialized_output_bytes: u64,
}

impl RuleExecutionBudget {
    pub fn new(
        max_evaluations: u64,
        max_findings: u64,
        max_source_handles: u64,
        max_evidence_records: u64,
        max_serialized_output_bytes: u64,
    ) -> RuleResult<Self> {
        if [
            max_evaluations,
            max_findings,
            max_source_handles,
            max_evidence_records,
            max_serialized_output_bytes,
        ]
        .contains(&0)
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleExecutionBudgetExceeded,
                "rule budget contains a zero limit",
            ));
        }
        Ok(Self {
            max_evaluations,
            max_findings,
            max_source_handles,
            max_evidence_records,
            max_serialized_output_bytes,
        })
    }

    pub fn fixture_e0() -> RuleResult<Self> {
        Self::new(1_024, 256, 4_096, 4_096, 8 * 1024 * 1024)
    }

    #[must_use]
    pub const fn max_evaluations(self) -> u64 {
        self.max_evaluations
    }

    #[must_use]
    pub const fn max_findings(self) -> u64 {
        self.max_findings
    }

    #[must_use]
    pub const fn max_source_handles(self) -> u64 {
        self.max_source_handles
    }

    #[must_use]
    pub const fn max_evidence_records(self) -> u64 {
        self.max_evidence_records
    }

    #[must_use]
    pub const fn max_serialized_output_bytes(self) -> u64 {
        self.max_serialized_output_bytes
    }
}

/// Canonical set of project files selected for rule evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleScope {
    file_ids: Vec<ProjectFileId>,
}

impl RuleScope {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            file_ids: Vec::new(),
        }
    }

    #[must_use]
    pub fn files(mut file_ids: Vec<ProjectFileId>) -> Self {
        file_ids.sort();
        file_ids.dedup();
        Self { file_ids }
    }

    #[must_use]
    pub fn file_ids(&self) -> &[ProjectFileId] {
        &self.file_ids
    }

    pub(crate) fn contains(&self, file_id: &ProjectFileId) -> bool {
        self.file_ids.is_empty() || self.file_ids.binary_search(file_id).is_ok()
    }
}

/// Policy selected for one immutable execution context.
#[derive(Clone, Copy)]
enum RulePolicyRef<'a> {
    Fixture(&'a RuleFixturePolicy),
    Production(&'a RuleProductionPolicy),
}

/// Immutable execution context assembled by a service or a fixture test.
pub struct RuleExecutionContext<'a> {
    registry: &'a RuleRegistry,
    policy: RulePolicyRef<'a>,
    project: &'a ProjectView,
    reference: &'a ReferenceView,
    budget: RuleExecutionBudget,
    cancelled: &'a AtomicBool,
}

impl<'a> RuleExecutionContext<'a> {
    #[must_use]
    pub const fn new(
        registry: &'a RuleRegistry,
        fixture_policy: &'a RuleFixturePolicy,
        project: &'a ProjectView,
        reference: &'a ReferenceView,
        budget: RuleExecutionBudget,
        cancelled: &'a AtomicBool,
    ) -> Self {
        Self {
            registry,
            policy: RulePolicyRef::Fixture(fixture_policy),
            project,
            reference,
            budget,
            cancelled,
        }
    }

    #[must_use]
    pub const fn production(
        registry: &'a RuleRegistry,
        production_policy: &'a RuleProductionPolicy,
        project: &'a ProjectView,
        reference: &'a ReferenceView,
        budget: RuleExecutionBudget,
        cancelled: &'a AtomicBool,
    ) -> Self {
        Self {
            registry,
            policy: RulePolicyRef::Production(production_policy),
            project,
            reference,
            budget,
            cancelled,
        }
    }

    pub fn validate(&self, scope: &RuleScope) -> RuleResult<()> {
        self.registry.validate()?;
        match self.policy {
            RulePolicyRef::Fixture(policy) => policy.validate()?,
            RulePolicyRef::Production(policy) => policy.validate()?,
        }
        self.project.snapshot().validate().map_err(|error| {
            RuleError::new(
                RuleErrorCode::RuleExecutionContextInvalid,
                format!("project snapshot validation failed: {error}"),
            )
        })?;
        self.reference.validate().map_err(|error| {
            RuleError::new(
                RuleErrorCode::RuleExecutionContextInvalid,
                format!("reference view validation failed: {error}"),
            )
        })?;
        let configuration = self.project.configuration();
        let selected_profile = configuration.selected_profile();
        if selected_profile.profile_id().as_str() != self.policy_profile_id() {
            return Err(RuleError::new(
                RuleErrorCode::RuleProfileMismatch,
                "rule policy profile differs from the selected project profile",
            ));
        }
        match self.policy {
            RulePolicyRef::Fixture(_)
                if selected_profile.profile_kind() != ProfileKind::Fixture =>
            {
                return Err(RuleError::new(
                    RuleErrorCode::RuleProfileMismatch,
                    "fixture rule policy requires a fixture profile",
                ));
            }
            RulePolicyRef::Production(_)
                if selected_profile.profile_kind() != ProfileKind::Release =>
            {
                return Err(RuleError::new(
                    RuleErrorCode::RuleProfileMismatch,
                    "production rule policy requires a release profile",
                ));
            }
            _ => {}
        }
        if self
            .registry
            .descriptors()
            .iter()
            .any(|descriptor| descriptor.supported_profile_id() != self.policy_profile_id())
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "rule registry profile differs from the execution policy",
            ));
        }
        let reference_generation = self
            .reference
            .generation_id()
            .parse::<wow_core::ReferenceGenerationId>()
            .map_err(|error| {
                RuleError::new(
                    RuleErrorCode::RuleReferenceGenerationMismatch,
                    format!("reference generation is not canonical: {error}"),
                )
            })?;
        if reference_generation != configuration.reference_generation() {
            return Err(RuleError::new(
                RuleErrorCode::RuleReferenceGenerationMismatch,
                "reference view generation differs from project configuration",
            ));
        }
        if self
            .project
            .snapshot()
            .generation_context()
            .project_generation()
            != Some(self.project.project_generation())
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleProjectGenerationMismatch,
                "project generation context is incoherent",
            ));
        }
        if self.project.analyzer_snapshot_id().is_empty()
            || self.project.member_call_report().main_snapshot_id()
                != self.project.local_flow_report().main_snapshot_id()
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleAnalyzerSnapshotMismatch,
                "project analyzer report identities are incoherent",
            ));
        }
        for file_id in scope.file_ids() {
            if self.project.file_by_id(file_id).is_none() {
                return Err(RuleError::new(
                    RuleErrorCode::RuleExecutionContextInvalid,
                    "rule scope contains a file outside the project snapshot",
                )
                .with_scope(file_id.as_str()));
            }
        }
        let expected_version = RULE_VERSION
            .parse::<wow_core::ToolVersion>()
            .map_err(|error| {
                RuleError::new(
                    RuleErrorCode::RuleRegistryInvalid,
                    format!("closed rule version is invalid: {error}"),
                )
            })?;
        if self.registry.descriptor(API_EXISTS_RULE).is_none()
            || self.registry.descriptor(SECRET_LOCAL_RULE).is_none()
            || self
                .registry
                .descriptors()
                .iter()
                .any(|descriptor| descriptor.rule_version() != &expected_version)
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "registry is missing a required active rule version",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn registry(&self) -> &RuleRegistry {
        self.registry
    }

    #[must_use]
    pub const fn fixture_policy(&self) -> Option<&RuleFixturePolicy> {
        match self.policy {
            RulePolicyRef::Fixture(policy) => Some(policy),
            RulePolicyRef::Production(_) => None,
        }
    }

    #[must_use]
    pub fn policy_id(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(policy) => policy.policy_id(),
            RulePolicyRef::Production(policy) => policy.policy_id(),
        }
    }

    #[must_use]
    pub fn policy_digest(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(policy) => policy.policy_digest(),
            RulePolicyRef::Production(policy) => policy.policy_digest(),
        }
    }

    #[must_use]
    pub fn policy_profile_id(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => FIXTURE_PROFILE_ID,
            RulePolicyRef::Production(policy) => policy.profile_id(),
        }
    }

    #[must_use]
    pub fn api_partition_id(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => "reference.fixture.apidoc.system:C_E0Fixture",
            RulePolicyRef::Production(policy) => policy.api_partition_id(),
        }
    }

    #[must_use]
    pub fn secret_partition_id(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => "reference.fixture.restriction:C_E0Fixture.SecretText",
            RulePolicyRef::Production(policy) => policy.secret_partition_id(),
        }
    }

    #[must_use]
    pub fn secret_return_payload(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => "return_position:1;applicability:unconditional_fixture",
            RulePolicyRef::Production(policy) => policy.secret_return_payload(),
        }
    }

    #[must_use]
    pub fn guard_entity(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => "function:canaccessvalue",
            RulePolicyRef::Production(policy) => policy.guard_entity(),
        }
    }

    #[must_use]
    pub fn guard_payload(&self) -> Option<&str> {
        match self.policy {
            RulePolicyRef::Fixture(_) => None,
            RulePolicyRef::Production(policy) => Some(policy.guard_payload()),
        }
    }

    #[must_use]
    pub fn guard_callee(&self) -> &str {
        match self.policy {
            RulePolicyRef::Fixture(_) => "canaccessvalue",
            RulePolicyRef::Production(policy) => policy.guard_callee(),
        }
    }

    #[must_use]
    pub const fn is_fixture_policy(&self) -> bool {
        matches!(self.policy, RulePolicyRef::Fixture(_))
    }

    #[must_use]
    pub const fn supports_secret_policy(&self) -> bool {
        true
    }

    #[must_use]
    pub const fn project(&self) -> &ProjectView {
        self.project
    }

    #[must_use]
    pub const fn reference(&self) -> &ReferenceView {
        self.reference
    }

    #[must_use]
    pub const fn budget(&self) -> RuleExecutionBudget {
        self.budget
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
