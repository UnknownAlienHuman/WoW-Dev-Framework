//! Strict transport for separately retained Candidate snapshots.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{self, Read, Write},
    path::PathBuf,
};
use wow_service::ServiceErrorCode;
use wow_service::lineage::{
    LineageBuildRequest, LineageReadRequest, abandon_lineage_snapshot, build_lineage_snapshot,
    compare_lineage_snapshot, retire_lineage_snapshot,
};

const HELP: &str = "wow lineage build|compare|retire|abandon --store-root <EXISTING-ORDINARY-PROJECT-STORE> --lineage-store <ABSOLUTE-STORE-FILE> --request-json <ABSOLUTE-JSON-FILE|-> [--operation-id <RETIRE-OR-ABANDON-ID>] [--max-output-bytes <ABANDON-BOUND>]\nBuild/abandon: wow-service/lineage-build/1. Compare/retire: wow-service/lineage-read/1, exact original build operation/digest/snapshot and Store limits. Candidate-only source-span discovery; no accepted lineage, absence, change, migration or runtime conclusion. Build/compare exit2. Retire releases exact snapshot/source roots; retained journal bytes remain. Abandon fences an unfinished original build, removes any inactive target, then releases its source holds. Abandon requires its own --max-output-bytes independent of the failed build bound.\n";
pub(super) fn run(values: Vec<OsString>) -> u8 {
    if values.len() == 2 && values[1] == "--help" {
        return if io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let command = match parse(values) {
        Ok(value) => value,
        Err(()) => return crate::usage("invalid exact lineage command; use wow lineage --help"),
    };
    let Some(stop) = crate::cancellation_flag() else {
        return 64;
    };
    let result = match command.request {
        Request::Build(request) => {
            build_lineage_snapshot(&command.source, &command.store, &request, &stop)
        }
        Request::Compare(request) => {
            compare_lineage_snapshot(&command.source, &command.store, &request, &stop)
        }
        Request::Retire(request, operation) => {
            retire_lineage_snapshot(&command.source, &command.store, &request, &operation, &stop)
        }
        Request::Abandon(request, operation, limit) => abandon_lineage_snapshot(
            &command.source,
            &command.store,
            &request,
            &operation,
            limit,
            &stop,
        ),
    };
    match result {
        Ok(bytes) => {
            let mut output = io::stdout().lock();
            if output
                .write_all(&bytes)
                .and_then(|()| output.write_all(b"\n"))
                .and_then(|()| output.flush())
                .is_err()
            {
                4
            } else if command.retire {
                0
            } else {
                2
            }
        }
        Err(error) => {
            crate::diagnostic(&format!("lineage failed ({:?})", error.code()));
            match error.code() {
                ServiceErrorCode::Cancelled => 130,
                ServiceErrorCode::InvalidRequest => 64,
                ServiceErrorCode::ComponentUnavailable => 2,
                _ => 4,
            }
        }
    }
}
enum Request {
    Build(LineageBuildRequest),
    Compare(LineageReadRequest),
    Retire(LineageReadRequest, String),
    Abandon(LineageBuildRequest, String, usize),
}
struct Command {
    source: PathBuf,
    store: PathBuf,
    request: Request,
    retire: bool,
}
fn parse(values: Vec<OsString>) -> Result<Command, ()> {
    if values.len() < 8
        || values.len() > 12
        || values
            .iter()
            .try_fold(0usize, |n, v| n.checked_add(v.len()))
            .is_none_or(|n| n > 65536)
    {
        return Err(());
    }
    let verb = values[1].to_str().ok_or(())?.to_owned();
    if !matches!(verb.as_str(), "build" | "compare" | "retire" | "abandon") {
        return Err(());
    }
    let mut fields = BTreeMap::new();
    let mut rest = values.into_iter().skip(2);
    while let Some(key) = rest.next() {
        if !matches!(
            key.to_str(),
            Some(
                "--store-root"
                    | "--lineage-store"
                    | "--request-json"
                    | "--operation-id"
                    | "--max-output-bytes"
            )
        ) || fields.insert(key, rest.next().ok_or(())?).is_some()
        {
            return Err(());
        }
    }
    let field = |key: &str| fields.get(&OsString::from(key)).ok_or(());
    let source = PathBuf::from(field("--store-root")?);
    let store = PathBuf::from(field("--lineage-store")?);
    if !source.is_absolute() || !store.is_absolute() {
        return Err(());
    }
    let input = field("--request-json")?;
    let mut bytes = Vec::new();
    if input == "-" {
        io::stdin()
            .lock()
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| ())?;
    } else {
        let path = PathBuf::from(input);
        if !path.is_absolute() {
            return Err(());
        }
        std::fs::File::open(path)
            .map_err(|_| ())?
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| ())?;
    }
    if bytes.is_empty() || bytes.len() > 65536 {
        return Err(());
    }
    let retire = matches!(verb.as_str(), "retire" | "abandon");
    if verb != "abandon" && fields.contains_key(&OsString::from("--max-output-bytes")) {
        return Err(());
    }
    if !retire && fields.contains_key(&OsString::from("--operation-id")) {
        return Err(());
    }
    let request = match verb.as_str() {
        "build" => Request::Build(serde_json::from_slice(&bytes).map_err(|_| ())?),
        "compare" => Request::Compare(serde_json::from_slice(&bytes).map_err(|_| ())?),
        "retire" => Request::Retire(
            serde_json::from_slice(&bytes).map_err(|_| ())?,
            field("--operation-id")?.to_str().ok_or(())?.to_owned(),
        ),
        "abandon" => Request::Abandon(
            serde_json::from_slice(&bytes).map_err(|_| ())?,
            field("--operation-id")?.to_str().ok_or(())?.to_owned(),
            field("--max-output-bytes")?
                .to_str()
                .ok_or(())?
                .parse()
                .map_err(|_| ())?,
        ),
        _ => return Err(()),
    };
    Ok(Command {
        source,
        store,
        request,
        retire,
    })
}
