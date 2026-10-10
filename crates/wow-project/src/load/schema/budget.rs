//! One conservative ledger for the complete selected-schema operation.
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::Serialize;

use super::{ProjectResult, exhausted, invalid};
use crate::disk::checkpoint;

pub(super) const MAX_DOCUMENTS: usize = 64;
pub(super) const MAX_COMPONENTS: usize = 32_768;
pub(super) const MAX_ROWS: usize = 65_536;
const MAX_WORK: usize = 1_048_576;
const MAX_METADATA: usize = 16 * 1024 * 1024;

pub(super) struct SchemaBudget {
    work: usize,
    rows: usize,
    metadata: usize,
}

impl SchemaBudget {
    pub(super) const fn new() -> Self {
        Self {
            work: 0,
            rows: 0,
            metadata: 0,
        }
    }

    pub(super) fn visit(&mut self, count: usize, stop: &AtomicBool) -> ProjectResult<()> {
        checkpoint(stop)?;
        self.work = self
            .work
            .checked_add(count)
            .filter(|value| *value <= MAX_WORK)
            .ok_or_else(exhausted)?;
        Ok(())
    }

    pub(super) fn rows(&mut self, count: usize, stop: &AtomicBool) -> ProjectResult<()> {
        self.visit(count, stop)?;
        self.rows = self
            .rows
            .checked_add(count)
            .filter(|value| *value <= MAX_ROWS)
            .ok_or_else(exhausted)?;
        Ok(())
    }

    pub(super) fn charge<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        checkpoint(stop)?;
        let mut counter = Counter {
            used: self.metadata,
            limit: MAX_METADATA,
            overflow: false,
            stop,
        };
        let result = serde_json::to_writer(&mut counter, value);
        checkpoint(stop)?;
        if counter.overflow {
            return Err(exhausted());
        }
        result.map_err(|_| invalid("schema metadata cannot be canonically encoded"))?;
        self.metadata = counter.used;
        Ok(())
    }
}

struct Counter<'a> {
    used: usize,
    limit: usize,
    overflow: bool,
    stop: &'a AtomicBool,
}

impl io::Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
        }
        let Some(next) = self
            .used
            .checked_add(bytes.len())
            .filter(|value| *value <= self.limit)
        else {
            self.overflow = true;
            return Err(io::Error::other("schema metadata limit exceeded"));
        };
        self.used = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
