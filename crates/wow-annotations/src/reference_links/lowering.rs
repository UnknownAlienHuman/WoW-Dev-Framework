//! Exact record read-back with no payload copy or successful-read allocation.
use super::NativeCallableBindingError;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::ReferenceRecord;

pub(super) fn digest_bytes(value: &str) -> Result<[u8; 32], NativeCallableBindingError> {
    let parsed = wow_core::ContentDigest::<wow_core::CanonicalResult>::parse(value)
        .map_err(|_| NativeCallableBindingError::IncompatibleInput)?;
    if !parsed.was_canonical() {
        return Err(NativeCallableBindingError::IncompatibleInput);
    }
    Ok(*parsed.value().as_bytes())
}

pub(super) fn verify_record(
    record: &ReferenceRecord,
    expected: &[u8; 32],
    max_bytes: usize,
    max_steps: usize,
    stop: &AtomicBool,
) -> Result<(), NativeCallableBindingError> {
    struct Sink<'a> {
        hash: Sha256,
        bytes: usize,
        steps: usize,
        max_bytes: usize,
        max_steps: usize,
        stop: &'a AtomicBool,
        failure: Option<NativeCallableBindingError>,
    }
    impl Write for Sink<'_> {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            let bytes = self.bytes.checked_add(data.len());
            let steps = self.steps.checked_add(1);
            self.failure = if self.stop.load(Ordering::Acquire) {
                Some(NativeCallableBindingError::Cancelled)
            } else if bytes.is_none_or(|value| value > self.max_bytes)
                || steps.is_none_or(|value| value > self.max_steps)
            {
                Some(NativeCallableBindingError::IncompatibleInput)
            } else {
                None
            };
            if self.failure.is_some() {
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.bytes = bytes.ok_or(io::ErrorKind::InvalidInput)?;
            self.steps = steps.ok_or(io::ErrorKind::InvalidInput)?;
            self.hash.update(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    if stop.load(Ordering::Acquire) {
        return Err(NativeCallableBindingError::Cancelled);
    }
    let mut sink = Sink {
        hash: Sha256::new(),
        bytes: 0,
        steps: 0,
        max_bytes,
        max_steps,
        stop,
        failure: None,
    };
    // Same fixed ReferenceRecord Serialize implementation used by digest_bounded.
    serde_json::to_writer(&mut sink, record).map_err(|_| {
        sink.failure
            .unwrap_or(NativeCallableBindingError::IncompatibleInput)
    })?;
    if stop.load(Ordering::Acquire) {
        return Err(NativeCallableBindingError::Cancelled);
    }
    let actual: [u8; 32] = sink.hash.finalize().into();
    if actual != *expected {
        return Err(NativeCallableBindingError::IncompatibleInput);
    }
    Ok(())
}
