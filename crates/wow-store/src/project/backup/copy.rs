use crate::project::{database, model::*};
use crate::{StoreError, StoreErrorCode, StoreResult};
use rusqlite::{
    Connection,
    backup::{Backup, StepResult},
};
use std::{fs::OpenOptions, path::Path, sync::atomic::AtomicBool, time::Duration};

/// Source is already pinned by a real read transaction. Destination is new and
/// exclusively held; no destination API runs while the Backup handle exists.
pub(super) fn copy(
    source: &Connection,
    path: &Path,
    epoch: &EpochManifest,
    stop: &AtomicBool,
) -> StoreResult<()> {
    checkpoint(stop)?;
    let pages: i64 = source
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    if !(1..=262144).contains(&pages) {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    let mut destination = database::connect(path, false)?;
    database::enable_writer(&destination)?;
    let mut done = false;
    {
        let backup = Backup::new(source, &mut destination).map_err(StoreError::database)?;
        let mut contention = 0;
        for _ in 0..2057 {
            checkpoint(stop)?;
            let step = backup.step(128).map_err(StoreError::database)?;
            let progress = backup.progress();
            if progress.pagecount < 0 || progress.pagecount > 262144 {
                return Err(failure(StoreErrorCode::BudgetExceeded));
            }
            match step {
                StepResult::Done => {
                    done = true;
                    break;
                }
                StepResult::More => {}
                StepResult::Busy | StepResult::Locked => {
                    contention += 1;
                    if contention >= 8 {
                        return Err(failure(StoreErrorCode::WriterBusy));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                _ => return Err(failure(StoreErrorCode::DatabaseUnavailable)),
            }
        }
    }
    if !done {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    checkpoint(stop)?;
    database::enable_writer(&destination)?;
    database::validate_header(&destination, epoch)?;
    let (busy, _, _): (i64, i64, i64) = destination
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(StoreError::database)?;
    if busy != 0 {
        return Err(failure(StoreErrorCode::WriterBusy));
    }
    destination
        .close()
        .map_err(|(_, error)| StoreError::database(error))?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
    checkpoint(stop)
}
