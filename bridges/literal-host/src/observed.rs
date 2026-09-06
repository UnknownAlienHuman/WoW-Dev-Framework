//! Local observations of one retained operation. No source text, IO, global
//! state, authority change or wire-schema extension is involved.
use crate::{BridgeError, Limits, LiteralBridge, LiteralError, Request, SelectedModule, Snapshot};
use std::cell::{Cell, RefCell};

/// Only successful receipts contribute fuel/memory observations. Failed calls
/// may consume resources too; these counters are not an operation-wide budget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub successful_fuel: u64,
    pub peak_successful_fuel: u64,
    pub peak_successful_memory_bytes: usize,
}

/// Preserves host failure details while the stable bridge still returns its
/// closed generic BridgeFailure. One instance belongs to one source operation.
/// A fatal failure is sticky and never retries the guest or falls back to native.
pub struct ObservedSnapshot {
    snapshot: Snapshot,
    usage: Cell<Usage>,
    failure: RefCell<Option<BridgeError>>,
}
impl ObservedSnapshot {
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            usage: Cell::new(Usage::default()),
            failure: RefCell::new(None),
        }
    }
    pub fn limits(&self) -> Limits {
        self.snapshot.limits()
    }
    pub fn usage(&self) -> Usage {
        self.usage.get()
    }
    pub fn failure(&self) -> Option<BridgeError> {
        self.failure.borrow().clone()
    }
}
impl LiteralBridge for ObservedSnapshot {
    fn selected_module(&self) -> Option<SelectedModule> {
        self.snapshot.selected_module()
    }
    fn render(&self, request: &Request) -> std::result::Result<String, LiteralError> {
        if self.failure.borrow().is_some() {
            return Err(LiteralError::BridgeFailure);
        }
        let mut usage = self.usage.get();
        match self.snapshot.render(request) {
            Ok(receipt) => {
                usage.successful_calls = usage.successful_calls.saturating_add(1);
                usage.successful_fuel = usage.successful_fuel.saturating_add(receipt.fuel_consumed);
                usage.peak_successful_fuel = usage.peak_successful_fuel.max(receipt.fuel_consumed);
                usage.peak_successful_memory_bytes =
                    usage.peak_successful_memory_bytes.max(receipt.memory_bytes);
                self.usage.set(usage);
                Ok(receipt.text)
            }
            Err(error) => {
                usage.failed_calls = usage.failed_calls.saturating_add(1);
                self.usage.set(usage);
                match error {
                    BridgeError::Render(
                        error @ (LiteralError::InvalidIdentifier
                        | LiteralError::UnsafeDocumentation
                        | LiteralError::DuplicateName
                        | LiteralError::InputLimit
                        | LiteralError::OutputLimit
                        | LiteralError::UnsupportedLiteral),
                    ) => Err(error),
                    error => {
                        *self.failure.borrow_mut() = Some(error);
                        Err(LiteralError::BridgeFailure)
                    }
                }
            }
        }
    }
}
