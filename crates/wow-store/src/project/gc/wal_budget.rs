//! Physical page admission for the existing bounded GC transaction.
use super::{ProjectGcReport, checkpoint, failure, invalid};
use crate::project::database::Database;
use crate::{StoreError, StoreErrorCode, StoreResult};
use rusqlite::{Connection, params};
use std::sync::atomic::AtomicBool;

const PAGE_SIZE: u64 = 4096;
const MAX_PAGES: u64 = 262144;
const MAX_WAL_BYTES: u64 = 128 * 1024 * 1024;
// This is an enforced allocation allowance, not an estimate of all possible
// rebalancing. Exhausting it rolls back the transaction.
const EXTRA_GROWTH_PAGES: u64 = 64;
// WAL header, a final commit frame and FULL-sync sector padding. SQLite bounds
// the VFS sector size at 64 KiB; allow a further complete frame at that boundary.
const WAL_PADDING: u64 = 32 + 65536 + 2 * (PAGE_SIZE + 24);

pub(super) struct Limits {
    page_limit: i64,
    spill: i64,
}

impl Limits {
    pub(super) fn restore(self, connection: &Connection) -> StoreResult<()> {
        let pages = connection.pragma_update(None, "max_page_count", self.page_limit);
        let spill = connection.pragma_update(None, "cache_spill", self.spill);
        if pages.is_err() || spill.is_err() {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let restored = (|| {
            Ok::<_, StoreError>(
                pragma(connection, "max_page_count")? == self.page_limit
                    && pragma(connection, "cache_spill")? == self.spill
                    && connection.is_autocommit(),
            )
        })();
        if !matches!(restored, Ok(true)) {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        Ok(())
    }
}

pub(super) fn admit(
    db: &Database,
    report: &ProjectGcReport,
    receipt_bytes: usize,
    stop: &AtomicBool,
) -> StoreResult<Limits> {
    db.ensure_idle()?;
    checkpoint(stop)?;
    let connection = &db.connection;
    let page_count = u64::try_from(pragma(connection, "page_count")?).map_err(|_| invalid())?;
    let free_pages = u64::try_from(pragma(connection, "freelist_count")?).map_err(|_| invalid())?;
    let limits = Limits {
        page_limit: pragma(connection, "max_page_count")?,
        spill: pragma(connection, "cache_spill")?,
    };
    if pragma(connection, "page_size")? != PAGE_SIZE as i64
        || pragma(connection, "auto_vacuum")? != 0
        || page_count == 0
        || page_count > MAX_PAGES
        || free_pages > page_count
        || limits.page_limit <= 0
        || limits.page_limit as u64 > MAX_PAGES
        || (limits.page_limit as u64) < page_count
    {
        return Err(invalid());
    }
    // Every page in each affected table and all of its compiled indexes is
    // reserved, including retained rows, sparse leaves, internal and overflow
    // pages. Untouched payload/operation trees are not charged as deletion work.
    let affected_pages: i64 = connection
        .query_row(
            "SELECT COALESCE(sum(pageno),0) FROM dbstat('main',1)
             WHERE name IN (
                 SELECT name FROM sqlite_schema WHERE type IN ('table','index')
                 AND (tbl_name='gc_operations'
                     OR (?1 AND tbl_name IN ('publication_history','validations',
                                            'membership','generations'))
                     OR (?2 AND tbl_name='partition_versions'))
             )",
            params![
                !report.delete_generations.is_empty(),
                !report.delete_versions.is_empty()
            ],
            |row| row.get(0),
        )
        .map_err(StoreError::database)?;
    let affected_pages = u64::try_from(affected_pages).map_err(|_| invalid())?;
    if affected_pages == 0 || affected_pages > page_count {
        return Err(invalid());
    }
    let growth_pages = (receipt_bytes as u64)
        .div_ceil(PAGE_SIZE - 4)
        .checked_add(EXTRA_GROWTH_PAGES)
        .ok_or_else(invalid)?;
    let page_limit = page_count
        .checked_add(growth_pages)
        .ok_or_else(invalid)?
        .min(limits.page_limit as u64);
    let growth_pages = page_limit.saturating_sub(page_count);
    // Rebalancing/overflow deletion can reuse any initial freelist page. Freed
    // affected pages are already included. The enforced page limit bounds every
    // newly appended page; auto_vacuum=NONE rules out unrelated pointer-map work.
    let reserved_pages = affected_pages
        .checked_add(free_pages)
        .and_then(|pages| pages.checked_add(growth_pages))
        .and_then(|pages| pages.checked_add(1))
        .ok_or_else(invalid)?;
    let reserve = reserved_pages
        .checked_mul(PAGE_SIZE + 24)
        .and_then(|bytes| bytes.checked_add(WAL_PADDING))
        .filter(|bytes| *bytes <= MAX_WAL_BYTES)
        .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
    checkpoint(stop)?;
    db.reserve_wal_bytes(reserve)?;
    // Keep spilling disabled for this bounded transaction. In the selected
    // SQLite runtime, walFrames also overwrites an already emitted uncommitted
    // page frame; repeated dirtying cannot append an unbounded frame sequence.
    let configured = connection
        .pragma_update(None, "max_page_count", page_limit as i64)
        .and_then(|()| connection.pragma_update(None, "cache_spill", 0));
    if let Err(error) = configured {
        limits.restore(connection)?;
        return Err(StoreError::database(error));
    }
    let observed = (|| {
        Ok::<_, StoreError>(
            pragma(connection, "max_page_count")? == page_limit as i64
                && pragma(connection, "cache_spill")? == 0,
        )
    })();
    if !matches!(observed, Ok(true)) {
        limits.restore(connection)?;
        return Err(observed.err().unwrap_or_else(invalid));
    }
    Ok(limits)
}

fn pragma(connection: &Connection, name: &'static str) -> StoreResult<i64> {
    connection
        .pragma_query_value(None, name, |row| row.get(0))
        .map_err(StoreError::database)
}
