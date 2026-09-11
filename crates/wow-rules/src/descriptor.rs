use serde::Serialize;
use wow_core::{
    CapabilityId, MessageCode, RemediationClass, RolloutPolicy, RuleId, Severity, ToolVersion,
};

use crate::identity::canonical_id;
use crate::{RuleError, RuleErrorCode, RuleResult};

pub const API_EXISTS_RULE: &str = "wow.api.exists";
pub const SECRET_LOCAL_RULE: &str = "wow.secret.local_operation";
pub const FIXTURE_PROFILE_ID: &str = "profile:fixture:retail-120100-e0-v1";
pub const FIXTURE_POLICY_ID: &str = "wow-rules-e0-fixture-policy/1";
pub const RULE_VERSION: &str = "1.0.0";

const API_REQUIRED: &[&str] = &[
    "project.generation.coherent",
    "project.source.registry.complete",
    "project.analyzer.snapshot.available",
    "emmy.library.loaded",
    "emmy.file.parsed",
    "emmy.fact.references",
    "emmy.fact.calls",
    "emmy.source_coordinates.exact",
    "reference.fixture.profile.valid",
    "reference.symbol.exact_lookup",
];

const SECRET_REQUIRED: &[&str] = &[
    "project.generation.coherent",
    "project.source.registry.complete",
    "project.analyzer.snapshot.available",
    "emmy.library.loaded",
    "emmy.file.parsed",
    "emmy.fact.references",
    "emmy.fact.calls",
    "emmy.fact.local_bindings",
    "emmy.fact.local_flow",
    "emmy.fact.operations",
    "emmy.fact.guards",
    "emmy.fact.control_flow",
    "emmy.source_coordinates.exact",
    "reference.fixture.profile.valid",
    "reference.symbol.exact_lookup",
    "reference.restriction.facets",
    "reference.source_handle.resolve",
];

/// One immutable active rule contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleDescriptor {
    rule_id: RuleId,
    rule_version: ToolVersion,
    semantic_category: MessageCode,
    technical_severity: Severity,
    rollout_policy: RolloutPolicy,
    remediation_class: RemediationClass,
    supported_profile_id: Box<str>,
    required_capabilities: Vec<CapabilityId>,
    fixture_case_set_id: Box<str>,
    max_evaluations: u64,
    max_findings: u64,
}

impl RuleDescriptor {
    fn api_exists() -> RuleResult<Self> {
        Self::build(
            API_EXISTS_RULE,
            "wow.api.missing",
            API_REQUIRED,
            "wow-rules-e0-api-exists-v1",
        )
    }

    fn secret_local() -> RuleResult<Self> {
        Self::build(
            SECRET_LOCAL_RULE,
            "wow.secret.unsafe_local_operation",
            SECRET_REQUIRED,
            "wow-rules-e0-secret-local-v1",
        )
    }

    fn build(
        rule_id: &str,
        category: &str,
        capabilities: &[&str],
        fixture_case_set_id: &str,
    ) -> RuleResult<Self> {
        let mut required_capabilities = capabilities
            .iter()
            .map(|value| value.parse::<CapabilityId>().map_err(core_error))
            .collect::<RuleResult<Vec<_>>>()?;
        required_capabilities.sort();
        required_capabilities.dedup();
        let value = Self {
            rule_id: rule_id.parse().map_err(core_error)?,
            rule_version: RULE_VERSION.parse().map_err(core_error)?,
            semantic_category: category.parse().map_err(core_error)?,
            technical_severity: Severity::Error,
            rollout_policy: RolloutPolicy::Advisory,
            remediation_class: RemediationClass::PlanOnly,
            supported_profile_id: FIXTURE_PROFILE_ID.into(),
            required_capabilities,
            fixture_case_set_id: fixture_case_set_id.into(),
            max_evaluations: 1_024,
            max_findings: 256,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> RuleResult<()> {
        let supported = matches!(self.rule_id.as_str(), API_EXISTS_RULE | SECRET_LOCAL_RULE)
            && self.rule_version == RULE_VERSION.parse().map_err(core_error)?
            && self.technical_severity == Severity::Error
            && self.rollout_policy == RolloutPolicy::Advisory
            && self.remediation_class == RemediationClass::PlanOnly
            && self.supported_profile_id.as_ref() == FIXTURE_PROFILE_ID
            && !self.required_capabilities.is_empty()
            && !self.fixture_case_set_id.is_empty()
            && self.max_evaluations > 0
            && self.max_findings > 0;
        if supported
            && self
                .required_capabilities
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleDescriptorInvalid,
                "rule descriptor violates the closed E0-E contract",
            )
            .with_rule(self.rule_id.as_str()))
        }
    }

    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    #[must_use]
    pub const fn rule_version(&self) -> &ToolVersion {
        &self.rule_version
    }

    #[must_use]
    pub const fn semantic_category(&self) -> &MessageCode {
        &self.semantic_category
    }

    #[must_use]
    pub const fn technical_severity(&self) -> Severity {
        self.technical_severity
    }

    #[must_use]
    pub const fn rollout_policy(&self) -> RolloutPolicy {
        self.rollout_policy
    }

    #[must_use]
    pub const fn remediation_class(&self) -> RemediationClass {
        self.remediation_class
    }

    #[must_use]
    pub fn supported_profile_id(&self) -> &str {
        &self.supported_profile_id
    }

    #[must_use]
    pub fn required_capabilities(&self) -> &[CapabilityId] {
        &self.required_capabilities
    }

    #[must_use]
    pub fn fixture_case_set_id(&self) -> &str {
        &self.fixture_case_set_id
    }

    #[must_use]
    pub const fn max_evaluations(&self) -> u64 {
        self.max_evaluations
    }

    #[must_use]
    pub const fn max_findings(&self) -> u64 {
        self.max_findings
    }
}

/// Canonical closed registry containing exactly the two E0-E providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRegistry {
    schema: &'static str,
    registry_id: Box<str>,
    descriptors: Vec<RuleDescriptor>,
}

impl RuleRegistry {
    pub fn e0() -> RuleResult<Self> {
        let mut descriptors = vec![
            RuleDescriptor::api_exists()?,
            RuleDescriptor::secret_local()?,
        ];
        descriptors.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            descriptors: &'a [RuleDescriptor],
        }
        let registry_id = canonical_id(
            "rule-registry:sha256:",
            "wow-rules/registry/e0-e/1",
            &Identity {
                schema: "wow-rules/registry/1",
                descriptors: &descriptors,
            },
        )?;
        let registry = Self {
            schema: "wow-rules/registry/1",
            registry_id,
            descriptors,
        };
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> RuleResult<()> {
        if self.schema != "wow-rules/registry/1" || self.descriptors.len() != 2 {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "E0-E registry must contain exactly two descriptors",
            ));
        }
        for descriptor in &self.descriptors {
            descriptor.validate()?;
        }
        if self.descriptors[0].rule_id.as_str() != API_EXISTS_RULE
            || self.descriptors[1].rule_id.as_str() != SECRET_LOCAL_RULE
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "E0-E registry rule set/order is invalid",
            ));
        }
        let rebuilt = Self::e0_without_validation()?;
        if self.registry_id == rebuilt.registry_id {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "rule registry identity does not match its descriptors",
            ))
        }
    }

    fn e0_without_validation() -> RuleResult<Self> {
        let mut descriptors = vec![
            RuleDescriptor::api_exists()?,
            RuleDescriptor::secret_local()?,
        ];
        descriptors.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            descriptors: &'a [RuleDescriptor],
        }
        let registry_id = canonical_id(
            "rule-registry:sha256:",
            "wow-rules/registry/e0-e/1",
            &Identity {
                schema: "wow-rules/registry/1",
                descriptors: &descriptors,
            },
        )?;
        Ok(Self {
            schema: "wow-rules/registry/1",
            registry_id,
            descriptors,
        })
    }

    #[must_use]
    pub fn registry_id(&self) -> &str {
        &self.registry_id
    }

    #[must_use]
    pub fn descriptors(&self) -> &[RuleDescriptor] {
        &self.descriptors
    }

    #[must_use]
    pub fn descriptor(&self, rule_id: &str) -> Option<&RuleDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.rule_id.as_str() == rule_id)
    }
}

fn core_error(error: wow_core::CoreError) -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleDescriptorInvalid,
        format!("rule descriptor contains an invalid core value: {error}"),
    )
}
