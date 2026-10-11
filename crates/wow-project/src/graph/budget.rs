//! Explicit metadata capacity; count, traversal and downstream limits stay separate.
use serde::{Deserialize, Serialize};

use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SourceGraphBudgetInput", deny_unknown_fields)]
pub struct SourceGraphBudget {
    max_metadata_bytes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceGraphBudgetInput {
    max_metadata_bytes: u64,
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
        Ok(Self { max_metadata_bytes })
    }

    #[must_use]
    pub const fn max_metadata_bytes(self) -> u64 {
        self.max_metadata_bytes
    }
}

impl TryFrom<SourceGraphBudgetInput> for SourceGraphBudget {
    type Error = ProjectError;

    fn try_from(input: SourceGraphBudgetInput) -> ProjectResult<Self> {
        Self::new(input.max_metadata_bytes)
    }
}
