//! Shared metadata accounting; copies are scratch until their stage succeeds.
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::{MAX_TEXT_BYTES, ProjectResult, raw_inventory, resource_exhausted};

#[derive(Clone, Copy)]
pub(super) struct ProducerBudget {
    used: usize,
}

impl ProducerBudget {
    pub(super) fn new(used: usize) -> ProjectResult<Self> {
        if used > MAX_TEXT_BYTES {
            return Err(resource_exhausted(
                "producer metadata bytes",
                used as u128,
                MAX_TEXT_BYTES,
            ));
        }
        Ok(Self { used })
    }

    pub(super) fn charge_serialized<T: Serialize>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        raw_inventory::charge_serialized(&mut self.used, value, stop)
    }
}
