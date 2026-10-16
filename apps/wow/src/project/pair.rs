//! One strict JSON-only projection of the native generation-pair operation.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
};
use wow_service::{
    ServiceErrorCode,
    live_project::{
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES, ProjectGenerationPairRequest,
        read_live_project_generation_pair,
    },
};

pub(super) fn run(values: Vec<OsString>) -> u8 {
    if values.len() == 3 && values[2] == "--help" {
        return if io::stdout()
            .lock()
            .write_all(super::HELP.as_bytes())
            .is_ok()
        {
            0
        } else {
            4
        };
    }
    let (root, request) = match parse(values) {
        Ok(parsed) => parsed,
        Err(()) => return crate::usage("invalid exact project generation-pair command"),
    };
    let Some(stop) = crate::cancellation_flag() else {
        return 64;
    };
    // Exactly one Service operation; no owner probes or root/current discovery.
    match read_live_project_generation_pair(&root, &request, &stop) {
        Ok(result) => {
            let mut output = io::stdout().lock();
            if output
                .write_all(result.canonical_bytes())
                .and_then(|()| output.write_all(b"\n"))
                .is_err()
            {
                4
            } else {
                2
            }
        }
        Err(error) => {
            let _ = writeln!(
                io::stderr().lock(),
                "wow: project generation pair failed ({:?})",
                error.code()
            );
            match error.code() {
                ServiceErrorCode::Cancelled => 130,
                ServiceErrorCode::InvalidRequest
                | ServiceErrorCode::IdentityMismatch
                | ServiceErrorCode::ExactGenerationUnavailable
                | ServiceErrorCode::BudgetExceeded => 3,
                _ => 4,
            }
        }
    }
}

fn parse(values: Vec<OsString>) -> Result<(PathBuf, ProjectGenerationPairRequest), ()> {
    if values.len() < 8
        || values.len() > 12
        || values[0] != "project"
        || values[1] != "pair"
        || values
            .iter()
            .try_fold(0usize, |n, v| n.checked_add(v.len()))
            .is_none_or(|n| n > 65536)
    {
        return Err(());
    }
    let mut options = BTreeMap::new();
    let mut rest = values.into_iter().skip(2);
    while let Some(key) = rest.next() {
        if !matches!(
            key.to_str(),
            Some(
                "--store-root"
                    | "--before-generation"
                    | "--after-generation"
                    | "--max-output-bytes"
                    | "--format"
            )
        ) {
            return Err(());
        }
        if options.insert(key, rest.next().ok_or(())?).is_some() {
            return Err(());
        }
    }
    let root = PathBuf::from(options.get(&OsString::from("--store-root")).ok_or(())?);
    if !root.is_absolute() || root.as_os_str().len() > 32768 {
        return Err(());
    }
    let text = |key: &str| {
        options
            .get(&OsString::from(key))
            .and_then(|v| v.to_str())
            .ok_or(())
    };
    if options
        .get(&OsString::from("--format"))
        .is_some_and(|v| v != "json")
    {
        return Err(());
    }
    let limit = match options.get(&OsString::from("--max-output-bytes")) {
        Some(value) => value.to_str().ok_or(())?.parse::<usize>().map_err(|_| ())?,
        None => PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    };
    let request = ProjectGenerationPairRequest::new(
        text("--before-generation")?,
        text("--after-generation")?,
        limit,
    )
    .map_err(|_| ())?;
    Ok((root, request))
}
