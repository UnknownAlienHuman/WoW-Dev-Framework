//! Explicit graph metadata and XML preflight limits; downstream limits stay separate.
use serde::{Deserialize, Serialize};

use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SourceGraphBudgetInput", deny_unknown_fields)]
pub struct SourceGraphBudget {
    max_metadata_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_counts: Option<SourceGraphXmlCountBudget>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceGraphBudgetInput {
    max_metadata_bytes: u64,
    #[serde(default, deserialize_with = "non_null_xml_counts")]
    xml_counts: Option<SourceGraphXmlCountBudget>,
}

fn non_null_xml_counts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SourceGraphXmlCountBudget>, D::Error> {
    SourceGraphXmlCountBudget::deserialize(deserializer).map(Some)
}

/// Complete retained declaration and Inherits populations, including unresolved
/// records. These limits do not select fact, containment or global graph budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SourceGraphXmlCountBudgetInput", deny_unknown_fields)]
pub struct SourceGraphXmlCountBudget {
    max_declarations: u64,
    max_inheritance_references: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceGraphXmlCountBudgetInput {
    max_declarations: u64,
    max_inheritance_references: u64,
}

impl SourceGraphXmlCountBudget {
    pub fn new(max_declarations: u64, max_inheritance_references: u64) -> ProjectResult<Self> {
        if max_declarations == 0 || max_inheritance_references == 0 {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "source graph XML count limits must be positive",
            ));
        }
        Ok(Self {
            max_declarations,
            max_inheritance_references,
        })
    }

    #[must_use]
    pub const fn max_declarations(self) -> u64 {
        self.max_declarations
    }

    #[must_use]
    pub const fn max_inheritance_references(self) -> u64 {
        self.max_inheritance_references
    }
}

impl TryFrom<SourceGraphXmlCountBudgetInput> for SourceGraphXmlCountBudget {
    type Error = ProjectError;

    fn try_from(input: SourceGraphXmlCountBudgetInput) -> ProjectResult<Self> {
        Self::new(input.max_declarations, input.max_inheritance_references)
    }
}

impl SourceGraphBudget {
    pub fn new(max_metadata_bytes: u64) -> ProjectResult<Self> {
        if max_metadata_bytes == 0 {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidBudgetPolicy,
                ProjectPhase::Configuration,
                "source graph metadata byte limit must be positive",
            ));
        }
        Ok(Self {
            max_metadata_bytes,
            xml_counts: None,
        })
    }

    #[must_use]
    pub const fn max_metadata_bytes(self) -> u64 {
        self.max_metadata_bytes
    }

    #[must_use]
    pub const fn with_xml_counts(mut self, budget: SourceGraphXmlCountBudget) -> Self {
        self.xml_counts = Some(budget);
        self
    }

    #[must_use]
    pub const fn xml_counts(self) -> Option<SourceGraphXmlCountBudget> {
        self.xml_counts
    }
}

impl TryFrom<SourceGraphBudgetInput> for SourceGraphBudget {
    type Error = ProjectError;

    fn try_from(input: SourceGraphBudgetInput) -> ProjectResult<Self> {
        let mut budget = Self::new(input.max_metadata_bytes)?;
        budget.xml_counts = input.xml_counts;
        Ok(budget)
    }
}
