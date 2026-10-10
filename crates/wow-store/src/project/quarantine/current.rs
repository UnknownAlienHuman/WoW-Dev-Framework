use super::model::{CurrentObservation, PointerReadFailure};
use crate::{
    StoreResult,
    project::model::{CurrentRecordId, digest},
};
use rusqlite::{Connection, Row, types::ValueRef};

const MAX_POINTER_BYTES: usize = 4096;
const OBSERVE_CURRENT: &str = "SELECT id, typeof(record_id), length(CAST(record_id AS BLOB)), CASE WHEN typeof(record_id)='text' AND length(CAST(record_id AS BLOB))<=4096 THEN CAST(record_id AS BLOB) END FROM current_publication ORDER BY id LIMIT 3";

pub(super) fn observe(c: &Connection) -> StoreResult<CurrentObservation> {
    let mut statement = match c.prepare(OBSERVE_CURRENT) {
        Ok(statement) => statement,
        Err(_) => return Ok(unreadable(PointerReadFailure::QueryUnavailable)),
    };
    let mut rows = match statement.query([]) {
        Ok(rows) => rows,
        Err(_) => return Ok(unreadable(PointerReadFailure::QueryUnavailable)),
    };
    let observation = match rows.next() {
        Ok(Some(row)) => observe_pointer(row),
        Ok(None) => return Ok(CurrentObservation::Absent),
        Err(_) => return Ok(unreadable(PointerReadFailure::QueryUnavailable)),
    };
    match rows.next() {
        Ok(None) => Ok(observation),
        Ok(Some(_)) => Ok(unreadable(PointerReadFailure::InvalidShape)),
        Err(_) => Ok(unreadable(PointerReadFailure::QueryUnavailable)),
    }
}

fn observe_pointer(row: &Row<'_>) -> CurrentObservation {
    if !matches!(row.get_ref(0), Ok(ValueRef::Integer(1)))
        || !matches!(row.get_ref(1), Ok(ValueRef::Text(b"text")))
    {
        return unreadable(PointerReadFailure::InvalidShape);
    }
    let byte_length = match row.get_ref(2) {
        Ok(ValueRef::Integer(length)) if length >= 0 => length,
        _ => return unreadable(PointerReadFailure::InvalidShape),
    };
    if byte_length > MAX_POINTER_BYTES as i64 {
        return unreadable(PointerReadFailure::BudgetExceeded);
    }
    let bytes = match row.get_ref(3) {
        Ok(ValueRef::Blob(bytes)) if bytes.len() == byte_length as usize => bytes,
        _ => return unreadable(PointerReadFailure::InvalidShape),
    };
    let record_id = std::str::from_utf8(bytes)
        .ok()
        .and_then(|value| CurrentRecordId::parse(value).ok());
    CurrentObservation::Pointer {
        digest: digest("project-current-observation", bytes),
        byte_length: bytes.len(),
        record_id,
    }
}

fn unreadable(reason: PointerReadFailure) -> CurrentObservation {
    CurrentObservation::Unreadable { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_invalid_oversized_and_multiple_pointer_rows_never_become_absence() -> StoreResult<()>
    {
        let c = Connection::open_in_memory().map_err(crate::StoreError::database)?;
        assert_eq!(
            observe(&c)?,
            unreadable(PointerReadFailure::QueryUnavailable)
        );
        c.execute_batch("CREATE TABLE current_publication(id, record_id)")
            .map_err(crate::StoreError::database)?;
        assert_eq!(observe(&c)?, CurrentObservation::Absent);
        c.execute("INSERT INTO current_publication VALUES(1,?1)", ["broken"])
            .map_err(crate::StoreError::database)?;
        assert_eq!(
            observe(&c)?,
            CurrentObservation::Pointer {
                digest: digest("project-current-observation", b"broken"),
                byte_length: 6,
                record_id: None,
            }
        );
        c.execute(
            "UPDATE current_publication SET record_id=?1",
            ["x".repeat(4097)],
        )
        .map_err(crate::StoreError::database)?;
        assert_eq!(observe(&c)?, unreadable(PointerReadFailure::BudgetExceeded));
        c.execute_batch("UPDATE current_publication SET record_id=123")
            .map_err(crate::StoreError::database)?;
        assert_eq!(observe(&c)?, unreadable(PointerReadFailure::InvalidShape));
        c.execute_batch("UPDATE current_publication SET record_id='broken'; INSERT INTO current_publication VALUES(2,'another')")
            .map_err(crate::StoreError::database)?;
        assert_eq!(observe(&c)?, unreadable(PointerReadFailure::InvalidShape));
        Ok(())
    }
}
