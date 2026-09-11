use std::collections::BTreeSet;

use serde::Serialize;

use crate::identity::{canonical_digest, validate_identifier};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub(crate) const SERVICE_SCHEMA: &str = "wow-service/e0-f/1";
pub(crate) const RESULT_SCHEMA: &str = "wow-service/result-envelope/1";
pub(crate) const SERVICE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Operations intentionally unavailable during the E0 milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferredOperation {
    Lookup,
    Search,
    Tree,
    Skeleton,
    Plan,
    PatchImpact,
    IndexRepo,
    RuntimeReview,
    Lsp,
    Mcp,
    Release,
    Pack,
}

impl DeferredOperation {
    pub(crate) const ALL: [Self; 12] = [
        Self::Lookup,
        Self::Search,
        Self::Tree,
        Self::Skeleton,
        Self::Plan,
        Self::PatchImpact,
        Self::IndexRepo,
        Self::RuntimeReview,
        Self::Lsp,
        Self::Mcp,
        Self::Release,
        Self::Pack,
    ];
}

/// Closed request/result limits for one synchronous service operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceBudgets {
    pub max_scope_files: u32,
    pub max_components: u32,
    pub max_generic_findings: u32,
    pub max_rule_evaluations: u32,
    pub max_raw_findings: u32,
    pub max_presentation_relations: u32,
}

impl ServiceBudgets {
    pub fn new(
        max_scope_files: u32,
        max_components: u32,
        max_generic_findings: u32,
        max_rule_evaluations: u32,
        max_raw_findings: u32,
        max_presentation_relations: u32,
    ) -> ServiceResult<Self> {
        let value = Self {
            max_scope_files,
            max_components,
            max_generic_findings,
            max_rule_evaluations,
            max_raw_findings,
            max_presentation_relations,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(self) -> ServiceResult<()> {
        let values = [
            self.max_scope_files,
            self.max_components,
            self.max_generic_findings,
            self.max_rule_evaluations,
            self.max_raw_findings,
            self.max_presentation_relations,
        ];
        if values.contains(&0) || values.into_iter().any(|value| value > 1_048_576) {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidConfiguration,
                "service budgets must be non-zero and bounded",
            ));
        }
        Ok(())
    }
}

impl Default for ServiceBudgets {
    fn default() -> Self {
        Self {
            max_scope_files: 65_536,
            max_components: 64,
            max_generic_findings: 65_536,
            max_rule_evaluations: 65_536,
            max_raw_findings: 131_072,
            max_presentation_relations: 131_072,
        }
    }
}

/// Immutable, content-addressed E0 service configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfiguration {
    schema: &'static str,
    service_version: &'static str,
    configuration_id: Box<str>,
    project_id: Box<str>,
    profile_id: Box<str>,
    reference_generation_id: Box<str>,
    analyzer_pin_id: Box<str>,
    rule_registry_id: Box<str>,
    result_schema: &'static str,
    budgets: ServiceBudgets,
    deferred_operations: Vec<DeferredOperation>,
}

impl ServiceConfiguration {
    #[must_use]
    pub fn builder() -> ServiceConfigurationBuilder {
        ServiceConfigurationBuilder::default()
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
    pub fn analyzer_pin_id(&self) -> &str {
        &self.analyzer_pin_id
    }
    #[must_use]
    pub fn rule_registry_id(&self) -> &str {
        &self.rule_registry_id
    }
    #[must_use]
    pub const fn budgets(&self) -> ServiceBudgets {
        self.budgets
    }
    #[must_use]
    pub fn deferred_operations(&self) -> &[DeferredOperation] {
        &self.deferred_operations
    }
}

#[derive(Debug, Default)]
pub struct ServiceConfigurationBuilder {
    project_id: Option<Box<str>>,
    profile_id: Option<Box<str>>,
    reference_generation_id: Option<Box<str>>,
    analyzer_pin_id: Option<Box<str>>,
    rule_registry_id: Option<Box<str>>,
    budgets: Option<ServiceBudgets>,
    deferred_operations: Option<Vec<DeferredOperation>>,
}

impl ServiceConfigurationBuilder {
    #[must_use]
    pub fn project_id(mut self, value: impl Into<Box<str>>) -> Self {
        self.project_id = Some(value.into());
        self
    }
    #[must_use]
    pub fn profile_id(mut self, value: impl Into<Box<str>>) -> Self {
        self.profile_id = Some(value.into());
        self
    }
    #[must_use]
    pub fn reference_generation_id(mut self, value: impl Into<Box<str>>) -> Self {
        self.reference_generation_id = Some(value.into());
        self
    }
    #[must_use]
    pub fn analyzer_pin_id(mut self, value: impl Into<Box<str>>) -> Self {
        self.analyzer_pin_id = Some(value.into());
        self
    }
    #[must_use]
    pub fn rule_registry_id(mut self, value: impl Into<Box<str>>) -> Self {
        self.rule_registry_id = Some(value.into());
        self
    }
    #[must_use]
    pub const fn budgets(mut self, value: ServiceBudgets) -> Self {
        self.budgets = Some(value);
        self
    }
    #[must_use]
    pub fn deferred_operations(mut self, value: Vec<DeferredOperation>) -> Self {
        self.deferred_operations = Some(value);
        self
    }

    pub fn build(self) -> ServiceResult<ServiceConfiguration> {
        let project_id = required(self.project_id, "project_id")?;
        let profile_id = required(self.profile_id, "profile_id")?;
        let reference_generation_id = required(
            self.reference_generation_id,
            "reference_generation_id",
        )?;
        let analyzer_pin_id = required(self.analyzer_pin_id, "analyzer_pin_id")?;
        let rule_registry_id = required(self.rule_registry_id, "rule_registry_id")?;
        let budgets = self.budgets.unwrap_or_default();
        budgets.validate()?;
        let mut deferred_operations = self
            .deferred_operations
            .unwrap_or_else(|| DeferredOperation::ALL.to_vec());
        deferred_operations.sort_unstable();
        if deferred_operations.iter().copied().collect::<BTreeSet<_>>().len()
            != deferred_operations.len()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidConfiguration,
                "duplicate deferred operation",
            ));
        }

        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            service_version: &'static str,
            project_id: &'a str,
            profile_id: &'a str,
            reference_generation_id: &'a str,
            analyzer_pin_id: &'a str,
            rule_registry_id: &'a str,
            result_schema: &'static str,
            budgets: ServiceBudgets,
            deferred_operations: &'a [DeferredOperation],
        }
        let identity = Identity {
            schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            project_id: &project_id,
            profile_id: &profile_id,
            reference_generation_id: &reference_generation_id,
            analyzer_pin_id: &analyzer_pin_id,
            rule_registry_id: &rule_registry_id,
            result_schema: RESULT_SCHEMA,
            budgets,
            deferred_operations: &deferred_operations,
        };
        let configuration_id = canonical_digest("service-configuration:sha256:", &identity)?;
        Ok(ServiceConfiguration {
            schema: SERVICE_SCHEMA,
            service_version: SERVICE_VERSION,
            configuration_id,
            project_id,
            profile_id,
            reference_generation_id,
            analyzer_pin_id,
            rule_registry_id,
            result_schema: RESULT_SCHEMA,
            budgets,
            deferred_operations,
        })
    }
}

fn required(value: Option<Box<str>>, field: &str) -> ServiceResult<Box<str>> {
    let value = value.ok_or_else(|| {
        ServiceError::new(
            ServiceErrorCode::InvalidConfiguration,
            format!("missing {field}"),
        )
    })?;
    validate_identifier(&value, field)?;
    Ok(value)
}
