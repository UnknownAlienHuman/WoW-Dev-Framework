//! Bounded fingerprints for the complete retained generated map inventory.
use super::{LookupError, check_cancelled};
use crate::native::AnnotationFile;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::sync::atomic::AtomicBool;

const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;
pub(super) const MAX_MAPPING_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn charge<const N: usize>(
    current: usize,
    parts: [usize; N],
) -> Result<usize, LookupError> {
    let total = parts
        .into_iter()
        .try_fold(current, usize::checked_add)
        .ok_or(LookupError::InputLimit)?;
    if total > MAX_METADATA_BYTES {
        return Err(LookupError::InputLimit);
    }
    Ok(total)
}

pub(super) fn mapping_digest(
    file: &AnnotationFile,
    max_bytes: usize,
    cancelled: &AtomicBool,
) -> Result<(String, usize), LookupError> {
    check_cancelled(cancelled)?;
    if file.mappings.len() > super::MAX_MAPPINGS {
        return Err(LookupError::InputLimit);
    }
    struct Sink<'a> {
        hash: Sha256,
        bytes: usize,
        max_bytes: usize,
        cancelled: &'a AtomicBool,
        failure: Option<LookupError>,
    }
    impl Write for Sink<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let result = check_cancelled(self.cancelled).and_then(|()| {
                self.bytes = self
                    .bytes
                    .checked_add(bytes.len())
                    .ok_or(LookupError::InputLimit)?;
                if self.bytes > self.max_bytes {
                    return Err(LookupError::InputLimit);
                }
                Ok(())
            });
            if let Err(error) = result {
                self.failure = Some(error);
                return Err(io::Error::other(error));
            }
            self.hash.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink {
        hash: Sha256::new(),
        bytes: 0,
        max_bytes,
        cancelled,
        failure: None,
    };
    sink.hash
        .update(b"wow-annotations/retained-navigation-mappings\0");
    serde_json::to_writer(&mut sink, &file.mappings)
        .map_err(|_| sink.failure.unwrap_or(LookupError::InvalidMapping))?;
    check_cancelled(cancelled)?;
    let digest = sink.hash.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").map_err(|_| LookupError::InvalidMapping)?;
    }
    Ok((hex, sink.bytes))
}
