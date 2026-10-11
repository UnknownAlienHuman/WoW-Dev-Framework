//! Count the complete borrowed page before canonical identity allocation.
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde::Serialize;
use std::{
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

pub(super) fn encoded_size(
    value: &impl Serialize,
    limit: u32,
    stop: &AtomicBool,
) -> ProjectResult<usize> {
    struct Counter<'a> {
        used: usize,
        limit: usize,
        exhausted: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(Ordering::Acquire) {
                return Err(std::io::Error::other("Skeleton serialization cancelled"));
            }
            let attempted = self.used.checked_add(bytes.len());
            if attempted.is_none_or(|n| n > self.limit) {
                self.exhausted = true;
                return Err(std::io::Error::other("Skeleton output budget exceeded"));
            }
            self.used = attempted.ok_or_else(|| std::io::Error::other("Skeleton size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        used: 0,
        limit: usize::try_from(limit).map_err(|_| exhausted())?,
        exhausted: false,
        stop,
    };
    crate::analyzer::checkpoint(stop)?;
    let result = serde_json::to_writer(&mut counter, value);
    crate::analyzer::checkpoint(stop)?;
    if counter.exhausted {
        return Err(exhausted());
    }
    result.map_err(|_| {
        ProjectError::new(
            ProjectErrorCode::CanonicalizationFailed,
            ProjectPhase::View,
            "Skeleton serialization failed",
        )
    })?;
    Ok(counter.used)
}

pub(super) fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        "native Skeleton read exceeds its selected bounded profile",
    )
}
