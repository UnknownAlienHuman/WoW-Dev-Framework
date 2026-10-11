//! Cumulative alias-index and global-path work for one native sidecar.
use serde::{Deserialize, Serialize};

use crate::references::EmmyMemberCallResult;

/// Independent positive finite work and retained-text accounting ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "GlobalAccessBudgetInput")]
pub struct GlobalAccessBudget {
    max_work: u64,
    max_text_bytes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GlobalAccessBudgetInput {
    max_work: u64,
    max_text_bytes: u64,
}

impl TryFrom<GlobalAccessBudgetInput> for GlobalAccessBudget {
    type Error = crate::references::EmmyMemberCallError;
    fn try_from(input: GlobalAccessBudgetInput) -> EmmyMemberCallResult<Self> {
        Self::new(input.max_work, input.max_text_bytes)
    }
}

impl Default for GlobalAccessBudget {
    fn default() -> Self {
        Self {
            max_work: super::MAX_ACCESS_WORK as u64,
            max_text_bytes: super::MAX_TEXT_BYTES as u64,
        }
    }
}

impl GlobalAccessBudget {
    pub fn new(max_work: u64, max_text_bytes: u64) -> EmmyMemberCallResult<Self> {
        if max_work == 0 || max_text_bytes == 0 {
            return Err(super::budget());
        }
        Ok(Self {
            max_work,
            max_text_bytes,
        })
    }
    #[must_use]
    pub const fn max_work(self) -> u64 {
        self.max_work
    }
    #[must_use]
    pub const fn max_text_bytes(self) -> u64 {
        self.max_text_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_access::AccessBudget;

    #[test]
    fn selected_global_budget_exact_ceiling_and_checked_overflow() -> EmmyMemberCallResult<()> {
        let mut budget = AccessBudget::new(GlobalAccessBudget::new(1, 1)?);
        budget.step()?;
        budget.text(1)?;
        assert!(budget.step().is_err());
        assert!(budget.text(1).is_err());
        let mut budget = AccessBudget::new(GlobalAccessBudget::new(u64::MAX, u64::MAX)?);
        budget.work = u64::MAX;
        budget.text = u64::MAX;
        assert!(budget.step().is_err());
        assert!(budget.text(1).is_err());
        Ok(())
    }
}
