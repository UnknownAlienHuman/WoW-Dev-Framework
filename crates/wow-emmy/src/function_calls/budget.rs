//! Explicit independent work/count/output limits for one function-call sidecar.
use serde::{Deserialize, Serialize};

use crate::global_access::GlobalAccessBudget;
use crate::references::EmmyMemberCallResult;

/// Positive finite limits. Alias work/text, scope, argument and literal bounds
/// remain independent. The default reproduces the original sidecar admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "FunctionCallBudgetInput")]
pub struct FunctionCallBudget {
    max_ast_visits: u64,
    max_functions: u64,
    max_calls: u64,
    max_global_accesses: u64,
    max_named_targets: u64,
    max_report_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_access: Option<GlobalAccessBudget>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionCallBudgetInput {
    max_ast_visits: u64,
    max_functions: u64,
    max_calls: u64,
    max_global_accesses: u64,
    max_named_targets: u64,
    max_report_bytes: u64,
    #[serde(default, deserialize_with = "non_null_global_access")]
    global_access: Option<GlobalAccessBudget>,
}

fn non_null_global_access<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<GlobalAccessBudget>, D::Error> {
    GlobalAccessBudget::deserialize(deserializer).map(Some)
}

impl TryFrom<FunctionCallBudgetInput> for FunctionCallBudget {
    type Error = crate::references::EmmyMemberCallError;

    fn try_from(input: FunctionCallBudgetInput) -> EmmyMemberCallResult<Self> {
        let mut budget = Self::new(
            input.max_ast_visits,
            input.max_functions,
            input.max_calls,
            input.max_global_accesses,
            input.max_named_targets,
            input.max_report_bytes,
        )?;
        budget.global_access = input.global_access;
        Ok(budget)
    }
}

impl Default for FunctionCallBudget {
    fn default() -> Self {
        Self {
            max_ast_visits: super::MAX_AST_VISITS as u64,
            max_functions: super::MAX_FUNCTIONS as u64,
            max_calls: super::MAX_CALLS as u64,
            max_global_accesses: crate::global_access::MAX_ACCESSES as u64,
            max_named_targets: crate::bindings::MAX_QUERIES as u64,
            max_report_bytes: super::MAX_REPORT_BYTES as u64,
            global_access: None,
        }
    }
}

impl FunctionCallBudget {
    pub fn new(
        max_ast_visits: u64,
        max_functions: u64,
        max_calls: u64,
        max_global_accesses: u64,
        max_named_targets: u64,
        max_report_bytes: u64,
    ) -> EmmyMemberCallResult<Self> {
        if [
            max_ast_visits,
            max_functions,
            max_calls,
            max_global_accesses,
            max_named_targets,
            max_report_bytes,
        ]
        .contains(&0)
        {
            return Err(super::budget());
        }
        Ok(Self {
            max_ast_visits,
            max_functions,
            max_calls,
            max_global_accesses,
            max_named_targets,
            max_report_bytes,
            global_access: None,
        })
    }

    #[must_use]
    pub fn with_global_access_budget(mut self, budget: GlobalAccessBudget) -> Self {
        self.global_access = Some(budget);
        self
    }
    #[must_use]
    pub const fn global_access_budget(self) -> Option<GlobalAccessBudget> {
        self.global_access
    }

    #[must_use]
    pub const fn max_ast_visits(self) -> u64 {
        self.max_ast_visits
    }
    #[must_use]
    pub const fn max_functions(self) -> u64 {
        self.max_functions
    }
    #[must_use]
    pub const fn max_calls(self) -> u64 {
        self.max_calls
    }
    #[must_use]
    pub const fn max_global_accesses(self) -> u64 {
        self.max_global_accesses
    }
    #[must_use]
    pub const fn max_named_targets(self) -> u64 {
        self.max_named_targets
    }
    #[must_use]
    pub const fn max_report_bytes(self) -> u64 {
        self.max_report_bytes
    }
}
