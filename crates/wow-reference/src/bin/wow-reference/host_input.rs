//! Bounded host artifact acquisition shared by reference maintenance binaries.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const CHUNK_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub(super) enum ReadError {
    Inspect(io::Error),
    Open(io::Error),
    Metadata(io::Error),
    Read(io::Error),
    TooLarge,
    ChangedLength,
    NotRegularFile,
    Allocation,
}

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inspect(error)
            | Self::Open(error)
            | Self::Metadata(error)
            | Self::Read(error) => {
                write!(formatter, "{error}")
            }
            Self::TooLarge => formatter.write_str("input exceeds its byte limit"),
            Self::ChangedLength => formatter.write_str("input length changed during acquisition"),
            Self::NotRegularFile => formatter.write_str("input path is not a regular file"),
            Self::Allocation => formatter.write_str("cannot reserve the bounded input buffer"),
        }
    }
}

pub(super) fn read_bounded_artifact(path: &Path, limit: usize) -> Result<Vec<u8>, ReadError> {
    let path_metadata = fs::symlink_metadata(path).map_err(ReadError::Inspect)?;
    if !path_metadata.is_file() || path_metadata.file_type().is_symlink() {
        return Err(ReadError::NotRegularFile);
    }
    let limit_u64 = u64::try_from(limit).map_err(|_| ReadError::TooLarge)?;
    if path_metadata.len() > limit_u64 {
        return Err(ReadError::TooLarge);
    }

    let mut file = File::open(path).map_err(ReadError::Open)?;
    let opened_metadata = file.metadata().map_err(ReadError::Metadata)?;
    if !opened_metadata.is_file() {
        return Err(ReadError::NotRegularFile);
    }
    read_bounded(&mut file, opened_metadata.len(), limit)
}

pub(super) fn read_bounded<R: Read>(
    reader: &mut R,
    declared_len: u64,
    limit: usize,
) -> Result<Vec<u8>, ReadError> {
    let limit_u64 = u64::try_from(limit).map_err(|_| ReadError::TooLarge)?;
    if declared_len > limit_u64 {
        return Err(ReadError::TooLarge);
    }
    let declared_len = usize::try_from(declared_len).map_err(|_| ReadError::TooLarge)?;
    let read_limit = limit.checked_add(1).ok_or(ReadError::TooLarge)?;
    let read_limit_u64 = u64::try_from(read_limit).map_err(|_| ReadError::TooLarge)?;

    let mut bounded = reader.take(read_limit_u64);
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(declared_len)
        .map_err(|_| ReadError::Allocation)?;
    let mut chunk = [0u8; CHUNK_BYTES];

    loop {
        let available = chunk.len().min(read_limit - bytes.len());
        let count = match bounded.read(&mut chunk[..available]) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ReadError::Read(error)),
        };
        if count == 0 {
            break;
        }
        if count > limit - bytes.len() {
            return Err(ReadError::TooLarge);
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| ReadError::Allocation)?;
        bytes.extend_from_slice(&chunk[..count]);
    }

    if bytes.len() != declared_len {
        return Err(ReadError::ChangedLength);
    }
    Ok(bytes)
}
