use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_graph::{GraphExplainLimits, GraphNeighborReadLimits};

use crate::{ContextError, ContextErrorCode, ContextResult};

/// Finite caller selections inside the existing native graph query bounds.
/// No default, deserialization, token limit, or execution-cap increase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PrimaryMapLimits {
    pub(crate) neighbors: GraphNeighborReadLimits,
    pub(crate) explain: GraphExplainLimits,
    pub(crate) max_direct_edges: u32,
    pub(crate) max_input_bytes: u64,
    pub(crate) max_metadata_bytes: u64,
    pub(crate) max_context_work: u64,
}
impl PrimaryMapLimits {
    pub fn new(
        neighbors: GraphNeighborReadLimits,
        explain: GraphExplainLimits,
        max_direct_edges: u32,
        max_input_bytes: u64,
        max_metadata_bytes: u64,
        max_context_work: u64,
    ) -> ContextResult<Self> {
        neighbors.validate().map_err(|_| invalid())?;
        explain.validate().map_err(|_| invalid())?;
        // Positive finite representable values; no new arbitrary resource ceiling.
        for value in [max_input_bytes, max_metadata_bytes, max_context_work] {
            if value == 0 || usize::try_from(value).is_err() {
                return Err(invalid());
            }
        }
        if max_direct_edges == 0 || max_direct_edges > 100_000 {
            return Err(invalid());
        }
        Ok(Self {
            neighbors,
            explain,
            max_direct_edges,
            max_input_bytes,
            max_metadata_bytes,
            max_context_work,
        })
    }
    #[must_use]
    pub const fn max_metadata_bytes(self) -> u64 {
        self.max_metadata_bytes
    }
    #[must_use]
    pub const fn max_context_work(self) -> u64 {
        self.max_context_work
    }
    #[must_use]
    pub const fn explanation_limits(self) -> GraphExplainLimits {
        self.explain
    }
}

pub(crate) fn checkpoint(stop: &AtomicBool) -> ContextResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(ContextError::new(ContextErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn invalid() -> ContextError {
    ContextError::new(ContextErrorCode::InvalidBudget)
}
pub(crate) fn exhausted() -> ContextError {
    ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
}

/// Cumulative context reservations. Work units are conservative loop-iteration
/// reservations, not measurements of CPU time or native analyzer/graph work.
pub(crate) struct Budget {
    pub(crate) work: u64,
    pub(crate) retained_bytes: u64,
    metadata_limit: u64,
    work_limit: u64,
}
impl Budget {
    pub(crate) fn new(limits: PrimaryMapLimits) -> Self {
        Self {
            work: 0,
            retained_bytes: 0,
            metadata_limit: limits.max_metadata_bytes,
            work_limit: limits.max_context_work,
        }
    }

    /// Continue an already charged bounded operation, such as external pack
    /// admission followed by rendering of its validated decoded view.
    pub(crate) fn continue_external(
        metadata_limit: u64,
        work_limit: u64,
        metadata_reserved: u64,
        work_reserved: u64,
    ) -> ContextResult<Self> {
        if metadata_limit == 0 {
            return Err(invalid());
        }
        if metadata_reserved > metadata_limit || work_reserved > work_limit {
            return Err(exhausted());
        }
        Ok(Self {
            work: work_reserved,
            retained_bytes: metadata_reserved,
            metadata_limit,
            work_limit,
        })
    }

    pub(crate) fn remaining_metadata_bytes(&self) -> ContextResult<u64> {
        self.metadata_limit
            .checked_sub(self.retained_bytes)
            .ok_or_else(exhausted)
    }
    pub(crate) fn remaining_context_work(&self) -> u64 {
        self.work_limit - self.work
    }
    pub(crate) fn work(&mut self, amount: usize, stop: &AtomicBool) -> ContextResult<()> {
        checkpoint(stop)?;
        self.work = self
            .work
            .checked_add(u64::try_from(amount).map_err(|_| exhausted())?)
            .filter(|value| *value <= self.work_limit)
            .ok_or_else(exhausted)?;
        Ok(())
    }
    /// Count a borrowed whole row BEFORE adding a retained reference or copy.
    pub(crate) fn retain<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> ContextResult<()> {
        let left = self.remaining_metadata_bytes()?;
        let bytes = encoded_len(value, left, stop)?;
        self.retained_bytes = self
            .retained_bytes
            .checked_add(bytes)
            .ok_or_else(exhausted)?;
        Ok(())
    }
}

/// Compact serde counting matches the byte length of strict canonical JSON for
/// these typed records (sorting object keys changes order, never length).
/// This counter does not admit null/other unsupported canonical types.
pub(crate) fn encoded_len<T: Serialize + ?Sized>(
    value: &T,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<u64> {
    struct Counter<'a> {
        bytes: u64,
        limit: u64,
        stop: &'a AtomicBool,
        exceeded: bool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.stop.load(Ordering::Acquire) {
                return Err(io::Error::other("context cancelled"));
            }
            let length = u64::try_from(bytes.len())
                .map_err(|_| io::Error::other("context byte overflow"))?;
            if length > self.limit.saturating_sub(self.bytes) {
                self.exceeded = true;
                return Err(io::Error::other("context byte bound"));
            }
            self.bytes += length;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    checkpoint(stop)?;
    let mut counter = Counter {
        bytes: 0,
        limit,
        stop,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut counter, value);
    checkpoint(stop)?;
    if counter.exceeded {
        return Err(exhausted());
    }
    result.map_err(|_| ContextError::new(ContextErrorCode::EncodingInvalid))?;
    Ok(counter.bytes)
}

pub(crate) fn canonical<T: Serialize + ?Sized>(
    value: &T,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<Vec<u8>> {
    let length = encoded_len(value, limit, stop)?;
    let encoded = wow_core::canonical_json_bytes(value);
    checkpoint(stop)?;
    let bytes = encoded.map_err(|_| ContextError::new(ContextErrorCode::EncodingInvalid))?;
    if u64::try_from(bytes.len()).map_err(|_| exhausted())? != length {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    Ok(bytes)
}

pub(crate) fn identity<T: Serialize + ?Sized>(
    domain: &'static str,
    prefix: &'static str,
    value: &T,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<Box<str>> {
    #[derive(Serialize)]
    struct Material<'a, T: Serialize + ?Sized> {
        domain: &'static str,
        value: &'a T,
    }
    // Precharge the actual domain envelope before core canonicalization/hash.
    encoded_len(&Material { domain, value }, limit, stop)?;
    let hashed = wow_core::domain_separated_digest(domain, value);
    checkpoint(stop)?;
    let digest = hashed.map_err(|_| ContextError::new(ContextErrorCode::EncodingInvalid))?;
    let mut id = String::from(prefix);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        id.push(char::from(HEX[usize::from(byte >> 4)]));
        id.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(id.into_boxed_str())
}
