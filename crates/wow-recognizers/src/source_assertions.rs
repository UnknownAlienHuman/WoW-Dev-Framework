//! Exact native endpoint admission and bounded retained-reference metadata.
use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphProducerLookup,
    GraphResolvedEntity,
};

use crate::{RecognizerError, RecognizerErrorCode, RecognizerResult};

const MAX_METADATA_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn entity<'a>(
    lookup: &GraphProducerLookup<'a>,
    scope: &GraphAssertionRecordScope,
    reference: &GraphAssertionRef,
    expected_proposal_id: &str,
    stop: &AtomicBool,
) -> RecognizerResult<GraphResolvedEntity<'a>> {
    checkpoint(stop)?;
    if lookup.scope() != scope {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let GraphAssertionRef::Producer { assertion, .. } = reference else {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    };
    if assertion.kind != GraphAssertionKind::Entity {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }
    if assertion.proposal_id.as_ref() != expected_proposal_id {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let resolved = lookup.entity(scope, reference, stop).map_err(|error| {
        failure(match error.code() {
            wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
            wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
            _ => RecognizerErrorCode::AdapterBindingInvalid,
        })
    })?;
    checkpoint(stop)?;
    Ok(resolved)
}

/// Count the complete borrowed envelope before retaining its owned copies.
/// This writer counts bytes without allocating an encoded JSON buffer.
pub(crate) fn preflight<T: Serialize>(value: &T, stop: &AtomicBool) -> RecognizerResult<()> {
    checkpoint(stop)?;
    let mut counter = Counter {
        used: 0,
        stop,
        failure: None,
    };
    if serde_json::to_writer(&mut counter, value).is_err() {
        return Err(failure(
            counter
                .failure
                .unwrap_or(RecognizerErrorCode::AdapterBindingInvalid),
        ));
    }
    checkpoint(stop)
}

struct Counter<'a> {
    used: usize,
    stop: &'a AtomicBool,
    failure: Option<RecognizerErrorCode>,
}
impl Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            self.failure = Some(RecognizerErrorCode::Cancelled);
            return Err(io::Error::other("source assertion metadata cancelled"));
        }
        let Some(next) = self.used.checked_add(bytes.len()) else {
            self.failure = Some(RecognizerErrorCode::BudgetExceeded);
            return Err(io::Error::other(
                "source assertion metadata exceeds its limit",
            ));
        };
        if next > MAX_METADATA_BYTES {
            self.failure = Some(RecognizerErrorCode::BudgetExceeded);
            return Err(io::Error::other(
                "source assertion metadata exceeds its limit",
            ));
        }
        self.used = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.stop.load(Ordering::Acquire) {
            self.failure = Some(RecognizerErrorCode::Cancelled);
            Err(io::Error::other("source assertion metadata cancelled"))
        } else {
            Ok(())
        }
    }
}

fn checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(code, "native source assertion binding rejected")
}
