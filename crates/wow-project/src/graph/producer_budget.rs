//! Shared metadata accounting; copies are scratch until their stage succeeds.
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::{MAX_TEXT_BYTES, ProjectResult, raw_inventory, resource_exhausted};
use crate::{ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectPhase};

#[derive(Clone, Copy)]
pub(super) struct ProducerBudget {
    used: usize,
    limit: usize,
    refusal: Option<MetadataRefusal>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct MetadataRefusal {
    counter_before_charge_bytes: u64,
    attempted_bytes: u128,
    configured_limit_bytes: u64,
}

impl ProducerBudget {
    pub(super) fn for_configuration(
        configuration: &ProjectConfiguration,
        used: usize,
    ) -> ProjectResult<Self> {
        let limit = match configuration.budget_policy().source_graph_budget() {
            Some(budget) => usize::try_from(budget.max_metadata_bytes()).map_err(|_| {
                ProjectError::new(
                    ProjectErrorCode::InvalidBudgetPolicy,
                    ProjectPhase::Configuration,
                    "source graph metadata byte limit exceeds this target's address space",
                )
            })?,
            None => MAX_TEXT_BYTES,
        };
        Self::new(used, limit)
    }

    pub(super) fn new(used: usize, limit: usize) -> ProjectResult<Self> {
        if used > limit {
            return Err(resource_exhausted(
                "producer metadata bytes",
                used as u128,
                limit,
            ));
        }
        Ok(Self {
            used,
            limit,
            refusal: None,
        })
    }

    pub(super) const fn used(self) -> usize {
        self.used
    }

    pub(super) const fn limit(self) -> usize {
        self.limit
    }

    pub(super) fn charge(&mut self, bytes: usize) -> ProjectResult<()> {
        let attempted = self.used as u128 + bytes as u128;
        if attempted > self.limit as u128 {
            self.note_refusal(attempted);
        }
        self.used = self
            .used
            .checked_add(bytes)
            .ok_or_else(|| resource_exhausted("metadata bytes", attempted, self.limit))?;
        if self.used > self.limit {
            return Err(resource_exhausted("metadata bytes", attempted, self.limit));
        }
        Ok(())
    }

    pub(super) fn note_refusal(&mut self, attempted: u128) {
        self.refusal.get_or_insert(MetadataRefusal {
            counter_before_charge_bytes: self.used as u64,
            attempted_bytes: attempted,
            configured_limit_bytes: self.limit as u64,
        });
    }

    pub(super) const fn refusal(self) -> Option<MetadataRefusal> {
        self.refusal
    }

    pub(super) fn charge_serialized<T: Serialize>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        raw_inventory::charge_serialized(self, value, stop)
    }
}
