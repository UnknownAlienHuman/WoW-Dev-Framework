use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
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

/// Immutable execution context assembled by a service or a fixture test.
pub struct RuleExecutionContext<'a> {
    registry: &'a RuleRegistry,
    fixture_policy: &'a RuleFixturePolicy,
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
            fixture_policy,
            project,
            reference,
            budget,
            cancelled,
        }
    }

    pub fn validate(&self, scope: &RuleScope) -> RuleResult<()> {
        self.registry.validate()?;
        self.fixture_policy.validate()?;
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
        if configuration.selected_profile().profile_id().as_str() != FIXTURE_PROFILE_ID {
            return Err(RuleError::new(
                RuleErrorCode::RuleProfileMismatch,
                "selected profile is outside the closed E0-E fixture scope",
            ));
        }
        if self.fixture_policy.profile_id != FIXTURE_PROFILE_ID {
            return Err(RuleError::new(
                RuleErrorCode::RuleFixturePolicyInvalid,
                "fixture policy profile differs from the project profile",
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
                "registry is missing a required E0-E rule version",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn registry(&self) -> &RuleRegistry {
        self.registry
    }

    #[must_use]
    pub const fn fixture_policy(&self) -> &RuleFixturePolicy {
        self.fixture_policy
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
