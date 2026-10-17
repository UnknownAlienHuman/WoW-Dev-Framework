//! Strict transport for separately retained Candidate snapshots.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{self, Read, Write},
    path::PathBuf,
};
use wow_service::ServiceErrorCode;
use wow_service::lineage::{
    LineageBuildRequest, LineageCandidateQuery, LineageQueryRequest, LineageReadRequest,
    ReferenceLineageRoots, abandon_lineage_snapshot, abandon_reference_lineage_snapshot,
    build_lineage_snapshot, build_reference_lineage_snapshot, compare_lineage_snapshot,
    compare_reference_lineage_snapshot, query_lineage_snapshot, query_reference_lineage_snapshot,
    retire_lineage_snapshot, retire_reference_lineage_snapshot,
};

const HELP: &str = "wow lineage build|compare|retire|abandon|status|validate|explain --store-root <EXISTING-ORDINARY-PROJECT-STORE> --lineage-store <ABSOLUTE-STORE-FILE> --request-json <ABSOLUTE-JSON-FILE|-> [--operation-id <RETIRE-OR-ABANDON-ID>] [--max-output-bytes <ABANDON-BOUND>]\nReference requires --after-store-root <EXISTING-AFTER-STORE>, reference-lineage-build/read/query/1 and reference_selection, genuine native target/C0 and a separate Reference Store; see apps/wow/REFERENCE_LINEAGE.md.\nOrdinary build/abandon: wow-service/lineage-build/1. Source uses explicit source-lineage-build/1, source-lineage-read/1 or source-lineage-query/1 plus source_selection with a stored composed candidate; see apps/wow/SOURCE_LINEAGE.md. Each Source Store has its own fixed profile. Compare/retire: wow-service/lineage-read/1, exact original build operation/digest/snapshot and Store limits. Status/validate/explain and exact entity compare: wow-service/lineage-query/1, original build/digest/snapshot guards, exact proposal/component or role-bound node IDs, finite read limits and logical lease expiry. Same query operation replays its original capture; use a new operation ID for a fresh read. Candidate-only source-span discovery; no accepted lineage, absence, change, migration or runtime conclusion. Build/compare/status/validate/explain exit2. Retire releases exact snapshot/source roots; retained journal bytes remain. Abandon fences an unfinished original build, removes any inactive target, then releases its source holds. Abandon requires its own --max-output-bytes independent of the failed build bound.\n";
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
    let roots = command
        .after_source
        .as_deref()
        .map(|after| ReferenceLineageRoots {
            before: &command.source,
            after,
        });
    let result = match command.request {
        Request::Build(request) => match roots {
            Some(roots) => build_reference_lineage_snapshot(roots, &command.store, &request, &stop),
            None => build_lineage_snapshot(&command.source, &command.store, &request, &stop),
        },
        Request::Compare(request) => match roots {
            Some(roots) => {
                compare_reference_lineage_snapshot(roots, &command.store, &request, &stop)
            }
            None => compare_lineage_snapshot(&command.source, &command.store, &request, &stop),
        },
        Request::Retire(request, operation) => match roots {
            Some(roots) => retire_reference_lineage_snapshot(
                roots,
                &command.store,
                &request,
                &operation,
                &stop,
            ),
            None => retire_lineage_snapshot(
                &command.source,
                &command.store,
                &request,
                &operation,
                &stop,
            ),
        },
        Request::Abandon(request, operation, limit) => {
            if let Some(roots) = roots {
                abandon_reference_lineage_snapshot(
                    roots,
                    &command.store,
                    &request,
                    &operation,
                    limit,
                    &stop,
                )
            } else {
                abandon_lineage_snapshot(
                    &command.source,
                    &command.store,
                    &request,
                    &operation,
                    limit,
                    &stop,
                )
            }
        }
        Request::Query(request) => match roots {
            Some(roots) => query_reference_lineage_snapshot(roots, &command.store, &request, &stop),
            None => query_lineage_snapshot(&command.source, &command.store, &request, &stop),
        },
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
    Query(LineageQueryRequest),
}
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CompareInput {
    Legacy(LineageReadRequest),
    Query(LineageQueryRequest),
}
struct Command {
    source: PathBuf,
    after_source: Option<PathBuf>,
    store: PathBuf,
    request: Request,
    retire: bool,
}
fn parse(values: Vec<OsString>) -> Result<Command, ()> {
    if values.len() < 8
        || values.len() > 14
        || values
            .iter()
            .try_fold(0usize, |n, v| n.checked_add(v.len()))
            .is_none_or(|n| n > 65536)
    {
        return Err(());
    }
    let verb = values[1].to_str().ok_or(())?.to_owned();
    if !matches!(
        verb.as_str(),
        "build" | "compare" | "retire" | "abandon" | "status" | "validate" | "explain"
    ) {
        return Err(());
    }
    let mut fields = BTreeMap::new();
    let mut rest = values.into_iter().skip(2);
    while let Some(key) = rest.next() {
        if !matches!(
            key.to_str(),
            Some(
                "--store-root"
                    | "--after-store-root"
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
    let after_source = fields
        .get(&OsString::from("--after-store-root"))
        .map(PathBuf::from);
    let store = PathBuf::from(field("--lineage-store")?);
    if !source.is_absolute()
        || !store.is_absolute()
        || after_source
            .as_ref()
            .is_some_and(|path| !path.is_absolute())
    {
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
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| ())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err(());
            }
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
        "compare" => match serde_json::from_slice::<CompareInput>(&bytes).map_err(|_| ())? {
            CompareInput::Legacy(request) => Request::Compare(request),
            CompareInput::Query(request) => {
                validate_verb_matches_query(&verb, &request.query)?;
                Request::Query(request)
            }
        },
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
        "status" | "validate" | "explain" => {
            let query_request: LineageQueryRequest =
                serde_json::from_slice(&bytes).map_err(|_| ())?;
            validate_verb_matches_query(&verb, &query_request.query)?;
            Request::Query(query_request)
        }
        _ => return Err(()),
    };
    let reference = match &request {
        Request::Build(r) | Request::Abandon(r, ..) => r.reference_selection.is_some(),
        Request::Compare(r) | Request::Retire(r, ..) => r.reference_selection.is_some(),
        Request::Query(r) => r.reference_selection.is_some(),
    };
    if reference != after_source.is_some() {
        return Err(());
    }
    Ok(Command {
        source,
        after_source,
        store,
        request,
        retire,
    })
}
fn validate_verb_matches_query(verb: &str, query: &LineageCandidateQuery) -> Result<(), ()> {
    if matches!(
        (verb, query),
        ("status", LineageCandidateQuery::Status {})
            | ("validate", LineageCandidateQuery::Validate {})
            | ("explain", LineageCandidateQuery::Explain { .. })
            | ("compare", LineageCandidateQuery::Compare { .. })
    ) {
        Ok(())
    } else {
        Err(())
    }
}
