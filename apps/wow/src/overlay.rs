//! Thin one-shot transport for explicit session-private overlay checking.
use std::{ffi::OsString, io::Write, path::PathBuf};
use wow_service::{ServiceErrorCode, overlay::check_private_overlay};
const HELP: &str = "wow overlay check --request <EXPLICIT-JSON-FILE>\nProject-owned explicit workspace and private Lua overlay, atomic guarded edits, actual syntax adapter and checked memory-only cleanup. No source publication or implicit saved base. Exit0: no diagnostics in the selected adapter families; exit2: diagnostics;64: invalid request;3: resynchronization/identity refusal;130: cancelled;4: other refusal. See apps/wow/OVERLAY.md.\n";
pub(super) fn run(values: Vec<OsString>) -> u8 {
    if (values.len() == 2 && values[1] == "--help")
        || (values.len() == 3 && values[1] == "check" && values[2] == "--help")
    {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let request = match parse(values) {
        Ok(request) => request,
        Err(()) => return super::usage("invalid overlay command; use wow overlay --help"),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let result = check_private_overlay(&request, &stop)
        .and_then(|result| Ok((result.diagnostic_count(), result.canonical_bytes(&stop)?)));
    match result {
        Ok((count, bytes)) => {
            if stop.load(std::sync::atomic::Ordering::Acquire) {
                return 130;
            }
            let mut output = std::io::stdout().lock();
            if output
                .write_all(&bytes)
                .and_then(|()| output.write_all(b"\n"))
                .and_then(|()| output.flush())
                .is_err()
            {
                4
            } else if count == 0 {
                0
            } else {
                2
            }
        }
        Err(error) => {
            super::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
            match error.code() {
                ServiceErrorCode::Cancelled => 130,
                ServiceErrorCode::InvalidConfiguration | ServiceErrorCode::InvalidRequest => 64,
                ServiceErrorCode::IdentityMismatch => 3,
                _ => 4,
            }
        }
    }
}
fn parse(values: Vec<OsString>) -> Result<PathBuf, ()> {
    if values.len() != 4
        || values.iter().map(|value| value.len()).sum::<usize>() > 65536
        || values[0] != "overlay"
        || values[1] != "check"
        || values[2] != "--request"
        || values[3].is_empty()
    {
        return Err(());
    }
    Ok(PathBuf::from(&values[3]))
}
