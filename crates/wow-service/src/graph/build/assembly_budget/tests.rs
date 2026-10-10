use super::AssemblyBudget;
use crate::ServiceErrorCode;
use serde::{Serialize, Serializer, ser::SerializeSeq};
use std::{
    cell::Cell,
    error::Error,
    sync::atomic::{AtomicBool, Ordering},
};

struct RepeatedStrings<'a> {
    value: &'a str,
    repetitions: usize,
}

impl Serialize for RepeatedStrings<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.repetitions))?;
        for _ in 0..self.repetitions {
            sequence.serialize_element(self.value)?;
        }
        sequence.end()
    }
}

#[test]
fn assembly_budget_rejects_combined_individually_valid_components() -> Result<(), Box<dyn Error>> {
    let stop = AtomicBool::new(false);
    // One borrowed MiB is streamed seventeen times; no encoded JSON is retained.
    let text = "x".repeat(1024 * 1024);
    let component = RepeatedStrings {
        value: &text,
        repetitions: 17,
    };
    let budget = AssemblyBudget::new();
    budget.reserve("first", &component, &stop)?;
    AssemblyBudget::new().reserve("second", &component, &stop)?;
    let error = budget
        .reserve("second", &component, &stop)
        .err()
        .ok_or("combined components exceeded 32 MiB without refusal")?;
    assert_eq!(error.code(), ServiceErrorCode::BudgetExceeded);
    assert!(!stop.load(Ordering::Acquire));
    Ok(())
}

struct CancelOnCanonicalPass<'a> {
    stop: &'a AtomicBool,
    serializations: Cell<usize>,
}

impl Serialize for CancelOnCanonicalPass<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.serializations.get() + 1;
        self.serializations.set(count);
        if count == 2 {
            self.stop.store(true, Ordering::Release);
        }
        serializer.serialize_str("bounded")
    }
}

#[test]
fn bounded_cancellable_observes_stop_during_canonical_serialize() -> Result<(), Box<dyn Error>> {
    let stop = AtomicBool::new(false);
    let value = CancelOnCanonicalPass {
        stop: &stop,
        serializations: Cell::new(0),
    };
    let error = super::super::bounded_cancellable(&value, 256, &stop)
        .err()
        .ok_or("canonical serialization returned output after cancellation")?;
    assert_eq!(error.code(), ServiceErrorCode::Cancelled);
    assert_eq!(value.serializations.get(), 2);
    assert!(stop.load(Ordering::Acquire));
    Ok(())
}
