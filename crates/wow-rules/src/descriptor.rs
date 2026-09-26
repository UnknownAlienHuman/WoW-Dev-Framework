use serde::Serialize;
use wow_core::{
    CapabilityId, MessageCode, ProfileId, RemediationClass, RolloutPolicy, RuleId, Severity,
    ToolVersion,
};

use crate::identity::canonical_id;
use crate::{RuleError, RuleErrorCode, RuleResult};

pub const API_EXISTS_RULE: &str = "wow.api.exists";
pub const SECRET_LOCAL_RULE: &str = "wow.secret.local_operation";
pub const FIXTURE_PROFILE_ID: &str = "profile:fixture:retail-120100-e0-v1";
pub const FIXTURE_POLICY_ID: &str = "wow-rules-e0-fixture-policy/1";
pub const RULE_VERSION: &str = "1.0.0";

const API_FIXTURE_REQUIRED: &[&str] = &[
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

const SECRET_FIXTURE_REQUIRED: &[&str] = &[
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

const API_PRODUCTION_REQUIRED: &[&str] = &[
    "project.generation.coherent",
    "project.source.registry.complete",
    "project.analyzer.snapshot.available",
    "emmy.library.loaded",
    "emmy.file.parsed",
    "emmy.fact.references",
    "emmy.fact.calls",
    "emmy.source_coordinates.exact",
    "reference.native.profile.valid",
    "reference.symbol.exact_lookup",
];

const SECRET_PRODUCTION_REQUIRED: &[&str] = &[
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
    "reference.native.profile.valid",
    "reference.symbol.exact_lookup",
    "reference.restriction.facets",
    "reference.source_handle.resolve",
];

const API_FIXTURE_CASES: &str = "wow-rules-e0-api-exists-v1";
const SECRET_FIXTURE_CASES: &str = "wow-rules-e0-secret-local-v1";
const API_PRODUCTION_CASES: &str = "wow-rules-production-native-api-v1";
const SECRET_PRODUCTION_CASES: &str = "wow-rules-production-secret-policy-v1";

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
    fn api_exists(profile_id: &str) -> RuleResult<Self> {
        let production = profile_id != FIXTURE_PROFILE_ID;
        Self::build(
            API_EXISTS_RULE,
            "wow.api.missing",
            profile_id,
            if production {
                API_PRODUCTION_REQUIRED
            } else {
                API_FIXTURE_REQUIRED
            },
            if production {
                API_PRODUCTION_CASES
            } else {
                API_FIXTURE_CASES
            },
        )
    }

    fn secret_local(profile_id: &str) -> RuleResult<Self> {
        let production = profile_id != FIXTURE_PROFILE_ID;
        Self::build(
            SECRET_LOCAL_RULE,
            "wow.secret.unsafe_local_operation",
            profile_id,
            if production {
                SECRET_PRODUCTION_REQUIRED
            } else {
                SECRET_FIXTURE_REQUIRED
            },
            if production {
                SECRET_PRODUCTION_CASES
            } else {
                SECRET_FIXTURE_CASES
            },
        )
    }

    fn build(
        rule_id: &str,
        category: &str,
        profile_id: &str,
        capabilities: &[&str],
        fixture_case_set_id: &str,
    ) -> RuleResult<Self> {
        canonical_profile_id(profile_id)?;
        let required_capabilities = capability_ids(capabilities)?;
        let value = Self {
            rule_id: rule_id.parse().map_err(core_error)?,
            rule_version: RULE_VERSION.parse().map_err(core_error)?,
            semantic_category: category.parse().map_err(core_error)?,
            technical_severity: Severity::Error,
            rollout_policy: RolloutPolicy::Advisory,
            remediation_class: RemediationClass::PlanOnly,
            supported_profile_id: profile_id.into(),
            required_capabilities,
            fixture_case_set_id: fixture_case_set_id.into(),
            max_evaluations: 1_024,
            max_findings: 256,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> RuleResult<()> {
        canonical_profile_id(&self.supported_profile_id)?;
        let production = self.supported_profile_id.as_ref() != FIXTURE_PROFILE_ID;
        let (expected_capabilities, expected_cases) = match self.rule_id.as_str() {
            API_EXISTS_RULE => (
                if production {
                    API_PRODUCTION_REQUIRED
                } else {
                    API_FIXTURE_REQUIRED
                },
                if production {
                    API_PRODUCTION_CASES
                } else {
                    API_FIXTURE_CASES
                },
            ),
            SECRET_LOCAL_RULE => (
                if production {
                    SECRET_PRODUCTION_REQUIRED
                } else {
                    SECRET_FIXTURE_REQUIRED
                },
                if production {
                    SECRET_PRODUCTION_CASES
                } else {
                    SECRET_FIXTURE_CASES
                },
            ),
            _ => {
                return Err(RuleError::new(
                    RuleErrorCode::RuleDescriptorInvalid,
                    "rule descriptor contains an unsupported rule",
                )
                .with_rule(self.rule_id.as_str()));
            }
        };
        let expected_capabilities = capability_ids(expected_capabilities)?;
        let supported = self.rule_version == RULE_VERSION.parse().map_err(core_error)?
            && self.technical_severity == Severity::Error
            && self.rollout_policy == RolloutPolicy::Advisory
            && self.remediation_class == RemediationClass::PlanOnly
            && self.required_capabilities == expected_capabilities
            && self.fixture_case_set_id.as_ref() == expected_cases
            && self.max_evaluations > 0
            && self.max_findings > 0;
        if supported {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleDescriptorInvalid,
                "rule descriptor violates the closed fixture/production contract",
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

/// Canonical closed registry containing the two active providers for one profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRegistry {
    schema: &'static str,
    registry_id: Box<str>,
    descriptors: Vec<RuleDescriptor>,
}

impl RuleRegistry {
    pub fn e0() -> RuleResult<Self> {
        Self::build(FIXTURE_PROFILE_ID)
    }

    pub fn production(profile_id: &str) -> RuleResult<Self> {
        if profile_id == FIXTURE_PROFILE_ID {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "production registry cannot select the E0 fixture profile",
            ));
        }
        Self::build(profile_id)
    }

    pub fn for_profile(profile_id: &str) -> RuleResult<Self> {
        if profile_id == FIXTURE_PROFILE_ID {
            Self::e0()
        } else {
            Self::production(profile_id)
        }
    }

    fn build(profile_id: &str) -> RuleResult<Self> {
        canonical_profile_id(profile_id)?;
        let mut descriptors = vec![
            RuleDescriptor::api_exists(profile_id)?,
            RuleDescriptor::secret_local(profile_id)?,
        ];
        descriptors.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            descriptors: &'a [RuleDescriptor],
        }
        let registry_id = canonical_id(
            "rule-registry:sha256:",
            if profile_id == FIXTURE_PROFILE_ID {
                "wow-rules/registry/e0-e/1"
            } else {
                "wow-rules/registry/production/1"
            },
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
                "rule registry must contain exactly two descriptors",
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
                "rule registry rule set/order is invalid",
            ));
        }
        let profile_id = self.descriptors[0].supported_profile_id();
        if self
            .descriptors
            .iter()
            .any(|descriptor| descriptor.supported_profile_id() != profile_id)
        {
            return Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "rule registry mixes profile policies",
            ));
        }
        let rebuilt = Self::build_without_validation(profile_id)?;
        if self.registry_id == rebuilt.registry_id {
            Ok(())
        } else {
            Err(RuleError::new(
                RuleErrorCode::RuleRegistryInvalid,
                "rule registry identity does not match its descriptors",
            ))
        }
    }

    fn build_without_validation(profile_id: &str) -> RuleResult<Self> {
        canonical_profile_id(profile_id)?;
        let mut descriptors = vec![
            RuleDescriptor::api_exists(profile_id)?,
            RuleDescriptor::secret_local(profile_id)?,
        ];
        descriptors.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            descriptors: &'a [RuleDescriptor],
        }
        let registry_id = canonical_id(
            "rule-registry:sha256:",
            if profile_id == FIXTURE_PROFILE_ID {
                "wow-rules/registry/e0-e/1"
            } else {
                "wow-rules/registry/production/1"
            },
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

fn capability_ids(values: &[&str]) -> RuleResult<Vec<CapabilityId>> {
    let mut capabilities = values
        .iter()
        .map(|value| value.parse::<CapabilityId>().map_err(core_error))
        .collect::<RuleResult<Vec<_>>>()?;
    capabilities.sort();
    capabilities.dedup();
    Ok(capabilities)
}

fn canonical_profile_id(value: &str) -> RuleResult<()> {
    let parsed = ProfileId::parse(value).map_err(core_error)?;
    if parsed.value().as_str() == value {
        Ok(())
    } else {
        Err(RuleError::new(
            RuleErrorCode::RuleDescriptorInvalid,
            "rule profile ID is not canonical",
        ))
    }
}

fn core_error(error: wow_core::CoreError) -> RuleError {
    RuleError::new(
        RuleErrorCode::RuleDescriptorInvalid,
        format!("rule descriptor contains an invalid core value: {error}"),
    )
}
