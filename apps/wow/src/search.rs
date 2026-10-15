//! Strict one-shot search build/query CLI route for the W20 first slice.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::Write,
    num::{NonZeroU32, NonZeroU64},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use wow_service::{
    OperationId, ServiceErrorCode,
    search::{
        LocalMetadataPermission, SearchBounds, SearchBuildQueryRequest, SearchOperationService,
        SearchStoreLimits,
    },
};

const HELP: &str = "wow search build-query --store-root <ABSOLUTE-PATH> --generation <ExactStoreGenerationId> --file-id <ProjectFileId> --literal-query <UTF-8-LITERAL> --operation-id <ID> --lease-expires-after <positive-epoch> --allow-local-metadata-only --max-documents <positive-u32> --max-document-id-bytes <positive-u32> --max-owner-metadata-label-bytes <positive-u32> --max-literal-query-bytes <positive-u32> --max-candidate-results <positive-u32> --max-owner-evidence-records <positive-u32> --max-owner-manifest-bytes <positive-u64> --max-database-bytes <positive-u64> --max-search-object-bytes <positive-u64> --max-output-bytes <positive-u64> --store-max-object-bytes <positive-u64> --store-max-batch-objects <positive-u32> --store-max-catalog-mutations <positive-u32> --store-max-manifest-records <positive-u32> --store-max-gc-deletes <positive-u32>\n";
const MAX_ARGUMENTS: usize = 64;
const MAX_ARGUMENT_BYTES: usize = 128 * 1024;
const MAX_VALUE_BYTES: usize = 32 * 1024;

const VALUE_OPTIONS: &[&str] = &[
    "--store-root",
    "--generation",
    "--file-id",
    "--literal-query",
    "--operation-id",
    "--lease-expires-after",
    "--max-documents",
    "--max-document-id-bytes",
    "--max-owner-metadata-label-bytes",
    "--max-literal-query-bytes",
    "--max-candidate-results",
    "--max-owner-evidence-records",
    "--max-owner-manifest-bytes",
    "--max-database-bytes",
    "--max-search-object-bytes",
    "--max-output-bytes",
    "--store-max-object-bytes",
    "--store-max-batch-objects",
    "--store-max-catalog-mutations",
    "--store-max-manifest-records",
    "--store-max-gc-deletes",
];

struct ParsedSearch {
    request: SearchBuildQueryRequest,
    max_output_bytes: u64,
}

/// Entry used by the root dispatcher after it adds the search branch.
pub fn run(arguments: Vec<OsString>) -> u8 {
    let stop = Arc::new(AtomicBool::new(false));
    let signal_stop = Arc::clone(&stop);
    if ctrlc::set_handler(move || signal_stop.store(true, Ordering::Release)).is_err() {
        diagnostic(
            &mut std::io::stderr().lock(),
            "could not initialize cancellation handling",
        );
        return 64;
    }
    run_with_io(
        arguments,
        &stop,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    )
}

fn run_with_io<W: Write, E: Write>(
    arguments: Vec<OsString>,
    stop: &AtomicBool,
    stdout: &mut W,
    stderr: &mut E,
) -> u8 {
    if is_help(&arguments) {
        return match stdout
            .write_all(HELP.as_bytes())
            .and_then(|()| stdout.flush())
        {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    let parsed = match parse(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            diagnostic(stderr, message);
            return 64;
        }
    };
    if stop.load(Ordering::Acquire) {
        diagnostic(stderr, "search operation cancelled");
        return 130;
    }
    let service = SearchOperationService::new();
    let result = match service.build_query(&parsed.request, stop) {
        Ok(result) => result,
        Err(error) => {
            diagnostic(
                stderr,
                &format!("search {:?}: {}", error.code(), error.message()),
            );
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    if stop.load(Ordering::Acquire) {
        diagnostic(stderr, "search operation cancelled before output");
        return 130;
    }
    let framed_bytes = match result.canonical_bytes().len().checked_add(1) {
        Some(size) if u64::try_from(size).is_ok_and(|size| size <= parsed.max_output_bytes) => size,
        _ => {
            diagnostic(stderr, "search output exceeds the caller byte bound");
            return 4;
        }
    };
    let mut output = Vec::with_capacity(framed_bytes);
    output.extend_from_slice(result.canonical_bytes());
    output.push(b'\n');
    match stdout.write_all(&output).and_then(|()| stdout.flush()) {
        Ok(()) => 0,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::BrokenPipe {
                diagnostic(stderr, "search output failed");
            }
            4
        }
    }
}

fn is_help(arguments: &[OsString]) -> bool {
    (arguments.len() == 2 && arguments[0] == "search" && arguments[1] == "--help")
        || (arguments.len() == 3
            && arguments[0] == "search"
            && arguments[1] == "build-query"
            && (arguments[2] == "--help" || arguments[2] == "-h"))
}

fn parse(arguments: Vec<OsString>) -> Result<ParsedSearch, &'static str> {
    let total_bytes = arguments
        .iter()
        .try_fold(0usize, |total, value| total.checked_add(value.len()))
        .ok_or("argument size overflow")?;
    if arguments.len() > MAX_ARGUMENTS || total_bytes > MAX_ARGUMENT_BYTES {
        return Err("search argument limit exceeded");
    }
    let mut values = arguments.into_iter();
    if values.next().as_deref() != Some(std::ffi::OsStr::new("search"))
        || values.next().as_deref() != Some(std::ffi::OsStr::new("build-query"))
    {
        return Err("expected search build-query");
    }

    let mut options = BTreeMap::<&'static str, OsString>::new();
    let mut allow_local_metadata = false;
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid option encoding")?;
        if option == "--allow-local-metadata-only" {
            if allow_local_metadata {
                return Err("duplicate local metadata permission");
            }
            allow_local_metadata = true;
            continue;
        }
        let key = VALUE_OPTIONS
            .iter()
            .copied()
            .find(|candidate| *candidate == option)
            .ok_or("unknown search option")?;
        let value = values.next().ok_or("missing search option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid search option value length");
        }
        if options.insert(key, value).is_some() {
            return Err("duplicate search option");
        }
    }
    if !allow_local_metadata {
        return Err("search requires --allow-local-metadata-only");
    }

    let store_root = PathBuf::from(required(&options, "--store-root")?);
    if !store_root.is_absolute() {
        return Err("--store-root must be absolute");
    }
    let generation = required_text(&options, "--generation")?;
    let file_id = required_text(&options, "--file-id")?;
    let literal_query = required_text(&options, "--literal-query")?;
    if literal_query.trim().is_empty() {
        return Err("--literal-query must be nonempty");
    }
    let operation_id = OperationId::new(required_text(&options, "--operation-id")?)
        .map_err(|_| "invalid --operation-id")?;

    let bounds = SearchBounds::new(
        positive_u32(&options, "--max-documents")?,
        positive_u32(&options, "--max-document-id-bytes")?,
        positive_u32(&options, "--max-owner-metadata-label-bytes")?,
        positive_u32(&options, "--max-literal-query-bytes")?,
        positive_u32(&options, "--max-candidate-results")?,
        positive_u32(&options, "--max-owner-evidence-records")?,
        positive_u64(&options, "--max-owner-manifest-bytes")?,
        positive_u64(&options, "--max-database-bytes")?,
        positive_u64(&options, "--max-search-object-bytes")?,
    )
    .map_err(|_| "invalid or inconsistent search bounds")?;
    let store_limits = SearchStoreLimits::new(
        positive_u64(&options, "--store-max-object-bytes")?.get(),
        positive_u32(&options, "--store-max-batch-objects")?.get(),
        positive_u32(&options, "--store-max-catalog-mutations")?.get(),
        positive_u32(&options, "--store-max-manifest-records")?.get(),
        positive_u32(&options, "--store-max-gc-deletes")?.get(),
    )
    .map_err(|_| "invalid Store limits")?;
    let max_output_bytes = positive_u64(&options, "--max-output-bytes")?;
    let lease_expires_after = positive_u64(&options, "--lease-expires-after")?.get();
    let request = SearchBuildQueryRequest::new(
        operation_id,
        store_root,
        generation,
        file_id,
        literal_query,
        bounds,
        store_limits,
        max_output_bytes,
        LocalMetadataPermission::authorize_local_metadata_only(),
        lease_expires_after,
    )
    .map_err(|_| "invalid exact search request")?;
    Ok(ParsedSearch {
        request,
        max_output_bytes: max_output_bytes.get(),
    })
}

fn required<'a>(
    options: &'a BTreeMap<&'static str, OsString>,
    key: &str,
) -> Result<&'a OsString, &'static str> {
    options.get(key).ok_or("missing required search option")
}

fn required_text<'a>(
    options: &'a BTreeMap<&'static str, OsString>,
    key: &str,
) -> Result<&'a str, &'static str> {
    let text = required(options, key)?
        .to_str()
        .ok_or("search option must be UTF-8")?;
    if text.is_empty() || text.len() > MAX_VALUE_BYTES || text.chars().any(char::is_control) {
        return Err("invalid search option text");
    }
    Ok(text)
}

fn positive_u32(
    options: &BTreeMap<&'static str, OsString>,
    key: &str,
) -> Result<NonZeroU32, &'static str> {
    let value = required_text(options, key)?;
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("numeric search limits must be unsigned decimal integers");
    }
    NonZeroU32::new(value.parse().map_err(|_| "search limit is out of range")?)
        .ok_or("search limits must be positive")
}

fn positive_u64(
    options: &BTreeMap<&'static str, OsString>,
    key: &str,
) -> Result<NonZeroU64, &'static str> {
    let value = required_text(options, key)?;
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("numeric search limits must be unsigned decimal integers");
    }
    NonZeroU64::new(value.parse().map_err(|_| "search limit is out of range")?)
        .ok_or("search limits must be positive")
}

fn diagnostic(stderr: &mut impl Write, message: &str) {
    let _ = writeln!(stderr, "wow: {message}");
}

#[cfg(test)]
mod tests {
    use std::{error::Error, ffi::OsString, sync::atomic::AtomicBool};

    use serde_json::Value;

    use super::run_with_io;

    const GENERATION: &str = "project-store-generation:sha256:8bb56e1a142df7bf917ad4a9e204dacb55ba2924c195940697731906014dcf51";
    const FILE_ID: &str = "project-file:state.lua";

    #[test]
    #[ignore = "requires the retained W20 fixture; set WOW_W20_RETAINED_STORE_ROOT and run with --ignored"]
    fn retained_exact_generation_build_query_route_returns_exact_and_candidate_evidence()
    -> Result<(), Box<dyn Error>> {
        let store_root = std::env::var_os("WOW_W20_RETAINED_STORE_ROOT")
            .ok_or("set WOW_W20_RETAINED_STORE_ROOT to the retained registered-store directory")?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let stop = AtomicBool::new(false);
        let exit = run_with_io(args(store_root), &stop, &mut stdout, &mut stderr);
        assert_eq!(
            exit,
            0,
            "unexpected route failure: {}",
            String::from_utf8_lossy(&stderr)
        );
        assert!(
            stderr.is_empty(),
            "unexpected stderr: {}",
            String::from_utf8_lossy(&stderr)
        );
        assert_eq!(stdout.last(), Some(&b'\n'));
        let result: Value = serde_json::from_slice(&stdout[..stdout.len() - 1])?;
        assert_eq!(result["schema"], "wow-service/search-result-envelope/1");
        assert_eq!(result["operation"], "search_build_query");
        assert_eq!(result["payload"]["source_store_generation_id"], GENERATION);
        assert_eq!(result["payload"]["entity_id"], FILE_ID);
        assert_eq!(result["payload"]["owner_metadata_label"], "state.lua");
        assert_eq!(result["payload"]["exact_id_match"], true);
        assert_eq!(result["payload"]["exact_owner_metadata_label_matches"], 1);
        assert_eq!(
            result["payload"]["literal_fts_candidates"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["payload"]["literal_fts_candidates"][0]["entity_id"],
            FILE_ID
        );
        assert_eq!(
            result["payload"]["literal_fts_candidates"][0]["owner_metadata_label"],
            "state.lua"
        );
        assert_eq!(
            result["payload"]["literal_fts_candidates"][0]["evidence"],
            "candidate"
        );
        assert_eq!(result["payload"]["resource_closure"], "closed");
        assert_eq!(result["payload"]["privacy_scope"], "local_metadata_only");
        assert_eq!(result["payload"]["license_state"], "not_evaluated");
        assert!(result.get("continuation").is_none());
        assert!(!String::from_utf8_lossy(&stdout).contains("C:/Development"));
        Ok(())
    }

    fn args(store_root: OsString) -> Vec<OsString> {
        [
            "search",
            "build-query",
            "--generation",
            GENERATION,
            "--file-id",
            FILE_ID,
            "--literal-query",
            "state",
            "--operation-id",
            "w20-retained-route-20261014",
            "--lease-expires-after",
            "1",
            "--allow-local-metadata-only",
            "--max-documents",
            "1",
            "--max-document-id-bytes",
            "256",
            "--max-owner-metadata-label-bytes",
            "1024",
            "--max-literal-query-bytes",
            "128",
            "--max-candidate-results",
            "32",
            "--max-owner-evidence-records",
            "128",
            "--max-owner-manifest-bytes",
            "1048576",
            "--max-database-bytes",
            "1048576",
            "--max-search-object-bytes",
            "3145728",
            "--max-output-bytes",
            "65536",
            "--store-max-object-bytes",
            "4194304",
            "--store-max-batch-objects",
            "8",
            "--store-max-catalog-mutations",
            "8",
            "--store-max-manifest-records",
            "256",
            "--store-max-gc-deletes",
            "16",
        ]
        .into_iter()
        .map(OsString::from)
        .chain([OsString::from("--store-root"), store_root])
        .collect()
    }
}
