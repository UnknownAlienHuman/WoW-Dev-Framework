//! Separate Candidate-only cross-generation overlay; local E2 records stay intact.
mod model;
pub use model::*;
mod source_model;
pub use source_model::*;
mod candidates;
pub use candidates::*;
mod snapshot;
pub use snapshot::*;
mod query;
pub use query::*;
mod read_view;
pub use read_view::*;

use crate::{GraphError, GraphErrorCode, GraphResult};
use serde::Serialize;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageUsage {
    pub work: u64,
    pub metadata_bytes: usize,
}

/// Continuing validation/construction ledger, never reset between stages.
pub(super) struct OperationBudget<'a> {
    pub(super) limits: LineageLimits,
    pub(super) usage: LineageUsage,
    stop: &'a AtomicBool,
}
impl<'a> OperationBudget<'a> {
    pub(super) fn resume(
        limits: LineageLimits,
        usage: LineageUsage,
        stop: &'a AtomicBool,
    ) -> GraphResult<Self> {
        limits.validate()?;
        checkpoint(stop)?;
        if usage.work > limits.max_work || usage.metadata_bytes > limits.max_metadata_bytes {
            return Err(budget());
        }
        Ok(Self {
            limits,
            usage,
            stop,
        })
    }
    pub(super) fn work(&mut self, amount: usize) -> GraphResult<()> {
        checkpoint(self.stop)?;
        self.usage.work = self
            .usage
            .work
            .checked_add(u64::try_from(amount).map_err(|_| budget())?)
            .ok_or_else(budget)?;
        if self.usage.work > self.limits.max_work {
            return Err(budget());
        }
        Ok(())
    }
    pub(super) fn metadata<T: Serialize>(&mut self, value: &T) -> GraphResult<usize> {
        let remaining = self
            .limits
            .max_metadata_bytes
            .checked_sub(self.usage.metadata_bytes)
            .ok_or_else(budget)?;
        let mut sink = Count {
            bytes: 0,
            limit: remaining,
            stop: self.stop,
        };
        if serde_json::to_writer(&mut sink, value).is_err() {
            checkpoint(self.stop)?;
            return Err(budget());
        }
        self.work(sink.bytes.checked_mul(2).ok_or_else(budget)?)?;
        self.usage.metadata_bytes = self
            .usage
            .metadata_bytes
            .checked_add(sink.bytes)
            .ok_or_else(budget)?;
        Ok(sink.bytes)
    }
}
struct Count<'a> {
    bytes: usize,
    limit: usize,
    stop: &'a AtomicBool,
}
impl Write for Count<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("lineage metadata bound exceeded"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn checkpoint(stop: &AtomicBool) -> GraphResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "lineage operation cancelled",
        ))
    } else {
        Ok(())
    }
}
pub(super) fn invalid() -> GraphError {
    GraphError::new(
        GraphErrorCode::LineageInputInvalid,
        "lineage input binding or proof is invalid",
    )
}
pub(super) fn budget() -> GraphError {
    GraphError::new(
        GraphErrorCode::BudgetExceeded,
        "lineage operation exceeds its selected bound",
    )
}
pub(super) fn identity<T: Serialize>(domain: &str, value: &T) -> GraphResult<Box<str>> {
    let digest = wow_core::domain_separated_digest(domain, value).map_err(|_| invalid())?;
    Ok(format!(
        "{domain}:{}",
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(digest)
    )
    .into())
}
