//! One conservative allowance for metadata retained by a selected service build.
use super::{MAX_BUNDLE_BYTES, checkpoint, error};
use crate::{ServiceErrorCode, ServiceResult};
use serde::{Serialize, ser::SerializeMap};
use std::{
    cell::Cell,
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(test)]
mod tests;

pub(super) struct AssemblyBudget {
    used: Cell<usize>,
}

impl AssemblyBudget {
    pub(super) const fn new() -> Self {
        Self { used: Cell::new(0) }
    }

    pub(super) fn reserve(
        &self,
        field: &'static str,
        value: &impl Serialize,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        struct Field<'a, T> {
            field: &'static str,
            value: &'a T,
        }
        impl<T: Serialize> Serialize for Field<'_, T> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(self.field, self.value)?;
                map.end()
            }
        }
        struct Counter<'a> {
            used: usize,
            stop: &'a AtomicBool,
            failure: Option<ServiceErrorCode>,
        }
        impl Write for Counter<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.stop.load(Ordering::Acquire) {
                    self.failure = Some(ServiceErrorCode::Cancelled);
                    return Err(std::io::Error::other("service assembly cancelled"));
                }
                if bytes.len() > MAX_BUNDLE_BYTES.saturating_sub(self.used) {
                    self.failure = Some(ServiceErrorCode::BudgetExceeded);
                    return Err(std::io::Error::other("service assembly metadata limit"));
                }
                self.used += bytes.len();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        checkpoint(stop)?;
        let mut counter = Counter {
            used: self.used.get(),
            stop,
            failure: None,
        };
        // Separate one-field frames conservatively include field names and JSON
        // punctuation. They count real components, not an evidence wire owner.
        let encoded = serde_json::to_writer(&mut counter, &Field { field, value });
        checkpoint(stop)?;
        encoded.map_err(|_| {
            error(
                counter
                    .failure
                    .unwrap_or(ServiceErrorCode::CanonicalizationFailed),
            )
        })?;
        self.used.set(counter.used);
        Ok(())
    }
}
