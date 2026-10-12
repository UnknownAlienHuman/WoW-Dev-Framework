use std::sync::atomic::AtomicBool;

use serde::{Serialize, Serializer, ser::SerializeSeq};

use crate::{GraphResult, GraphSnapshot, partition::check_cancelled};

/// Fresh full-build proof. No raw constructor, mutation, decoding or cloning.
#[derive(Debug)]
pub(crate) struct ValidatedGraphSnapshot {
    snapshot: GraphSnapshot,
}

impl ValidatedGraphSnapshot {
    pub(super) fn new(snapshot: GraphSnapshot) -> Self {
        Self { snapshot }
    }

    pub(crate) fn as_snapshot(&self) -> &GraphSnapshot {
        &self.snapshot
    }

    pub(crate) fn into_raw(self) -> GraphSnapshot {
        self.snapshot
    }
}

pub(super) fn clone_records<T: Clone>(values: &[T], stop: &AtomicBool) -> GraphResult<Vec<T>> {
    check_cancelled(stop)?;
    let mut copies = Vec::with_capacity(values.len());
    for value in values {
        check_cancelled(stop)?;
        copies.push(value.clone());
    }
    Ok(copies)
}

pub(super) struct CheckedSlice<'a, T> {
    values: &'a [T],
    stop: &'a AtomicBool,
}

pub(super) fn checked_slice<'a, T>(values: &'a [T], stop: &'a AtomicBool) -> CheckedSlice<'a, T> {
    CheckedSlice { values, stop }
}

impl<T: Serialize> Serialize for CheckedSlice<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.values.len()))?;
        for value in self.values {
            check_cancelled(self.stop).map_err(|_| {
                <S::Error as serde::ser::Error>::custom("graph snapshot identity cancelled")
            })?;
            sequence.serialize_element(value)?;
        }
        sequence.end()
    }
}

// Nonsemantic observations used by the selected owner witness only. No runtime
// counters, serialized fields or profiling subsystem are added to production.
#[cfg(test)]
thread_local! {
    static SNAPSHOT_WORK: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

#[cfg(test)]
pub(super) fn record_build() {
    SNAPSHOT_WORK.with(|work| {
        let (builds, validations) = work.get();
        work.set((builds + 1, validations));
    });
}

#[cfg(test)]
pub(super) fn record_validation() {
    SNAPSHOT_WORK.with(|work| {
        let (builds, validations) = work.get();
        work.set((builds, validations + 1));
    });
}

#[cfg(test)]
pub(crate) fn snapshot_work_counts() -> (usize, usize) {
    SNAPSHOT_WORK.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    struct CancellingValue<'a>(&'a AtomicBool);

    impl Clone for CancellingValue<'_> {
        fn clone(&self) -> Self {
            self.0.store(true, Ordering::Relaxed);
            Self(self.0)
        }
    }

    impl Serialize for CancellingValue<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let result = serializer.serialize_u8(1)?;
            self.0.store(true, Ordering::Relaxed);
            Ok(result)
        }
    }

    #[test]
    fn copies_and_identity_admission_stop_between_records() -> Result<(), Box<dyn std::error::Error>>
    {
        let stop = AtomicBool::new(false);
        let values = [CancellingValue(&stop), CancellingValue(&stop)];
        assert_eq!(
            clone_records(&values, &stop)
                .err()
                .ok_or("cancelled copy accepted")?
                .code(),
            crate::GraphErrorCode::Cancelled
        );
        stop.store(false, Ordering::Relaxed);
        assert!(wow_core::canonical_json_bytes(&checked_slice(&values, &stop)).is_err());
        assert!(stop.load(Ordering::Relaxed));
        Ok(())
    }
}
