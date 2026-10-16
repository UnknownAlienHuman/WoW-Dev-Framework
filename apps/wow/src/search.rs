//! Strict one-shot and durable exact-generation Search CLI routes.

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
        EntityContinueRequest, EntityResultReleaseRequest, EntityRetrievalRequest,
        FederationContinueRequest, FederationRequest, FederationResultReleaseRequest,
        LocalMetadataPermission, ReferenceContinueRequest, ReferenceIndexBuildRequest,
        ReferenceResultReleaseRequest, ReferenceShardRequest, SearchBounds,
        SearchBuildQueryRequest, SearchContinueRequest, SearchIndexBuildRequest,
        SearchOperationService, SearchResultReleaseRequest, SearchRetrievalRequest,
        SearchShardQueryRequest, SearchShardRetireRequest, SearchSourceHoldBinding,
        SearchStoreGcRequest, SearchStoreLimits, SourceObservationRetrievalRequest,
    },
};

const HELP: &str = "wow search index build --store-root <PROJECT-STORE> --search-store <SEARCH-STORE-FILE> --generation <ExactStoreGenerationId> --file-id <ProjectFileId> --operation-id <ID> --allow-local-metadata-only <finite bounds>\nwow search query --search-store <SEARCH-STORE-FILE> --generation <ExactStoreGenerationId> --file-id <ProjectFileId> --search-shard-id <ExactSearchShardId> --object-id <ExactObjectId> --profile-digest <sha256:...> --literal-query <UTF-8-LITERAL> --operation-id <ID> --lease-expires-after <epoch> --allow-local-metadata-only <finite bounds>\nwow search index retire --store-root <PROJECT-STORE> --search-store <SEARCH-STORE-FILE> --generation <ExactStoreGenerationId> --file-id <ProjectFileId> --search-shard-id <ExactSearchShardId> --object-id <ExactObjectId> --profile-digest <sha256:...> --build-operation-id <BuildOperationId> --source-store-epoch-id <ExactEpochId> --source-generation-hold-id <ExactRetentionRootId> --source-generation-hold-digest <sha256:...> --operation-id <ID> --lease-expires-after <epoch> --allow-local-metadata-only <finite bounds>\nwow search index gc --search-store <SEARCH-STORE-FILE> --target-object-id <ExactObjectId> --operation-id <ID> --gc-now <epoch> --gc-max-deletes <positive-u32> <Store limits>\nwow search build-query --store-root <PROJECT-STORE> <existing one-shot route; it does not claim durable SearchStore persistence>\nAll durable commands require --max-output-bytes <positive-u64> and Store limits: --store-max-object-bytes <positive-u64> --store-max-batch-objects <positive-u32> --store-max-catalog-mutations <positive-u32> --store-max-manifest-records <positive-u32> --store-max-gc-deletes <positive-u32>.\nBuild/query/retire also require Search bounds: --max-documents <positive-u32> --max-document-id-bytes <positive-u32> --max-owner-metadata-label-bytes <positive-u32> --max-literal-query-bytes <positive-u32> --max-candidate-results <positive-u32> --max-owner-evidence-records <positive-u32> --max-owner-manifest-bytes <positive-u64> --max-database-bytes <positive-u64> --max-search-object-bytes <positive-u64>.\n";
const MAX_ARGUMENTS: usize = 64;
const BLIZZARD_FEDERATION_HELP: &str = "Required Blizzard UI uses request schema wow-service/exact-platform-search-federation-input/1 with an exact blizzard source-hold/shard/query record, and query schema wow-search/exact-primary-platform-reference-query/1 with blizzard_ui=required and all3universe priorities. Native ProjectKind/source/package binding required; source grade stays explicit. Captured pages retain5roots; choose original Store catalog-mutation capacity>=9. Excluded2universe schemas/bytes remain exact.\n";
const FEDERATION_HELP: &str = "wow search federated --request-json <wow-service/exact-search-federation-input/1 JSON> --search-store <SAME-STORE> --operation-id <ID> --lease-expires-after <EPOCH> --allow-local-metadata-only <Store/output limits>. Optional --page-json <wow-search/federated-page-policy/1 JSON> retains captured result, primary shard, Reference shard and ReferenceView. wow search federated continue --continuation-json <EXACT> and wow search federated results release --session-object-id <EXACT> use the same Store/output limits. Requires compatible exact native owners; no Current, implicit build or cross-profile fallback.\n";
const ENTITY_PAGES_HELP: &str = "ProjectEntity query accepts optional --page-json <wow-search/project-entity-page-policy/1 JSON>. wow search entities continue --continuation-json <EXACT-TOKEN> and wow search entities results release --session-object-id <EXACT> use the same --search-store and original Store/output limits, --operation-id and --allow-local-metadata-only. Pages retain the original Search capture and shard; continuation does not recompute native Graph owners.\n";
const ENTITIES_HELP: &str = "ProjectEntity index build/retire use --graph-entities instead of --file-id/--file-ids. wow search entities --store-root <ORIGINAL-PROJECT-STORE> --generation <EXACT> --search-store <SEARCH-STORE> --search-shard-id <ID> --object-id <ID> --profile-digest <DIGEST> --build-operation-id <ORIGINAL> --source-store-epoch-id <EXACT> --source-generation-hold-id <EXACT> --source-generation-hold-digest <EXACT> --query-json <CLOSED-ENTITY-QUERY> --operation-id <ID> --lease-expires-after <EPOCH> --allow-local-metadata-only <Search/Store/output bounds>. Actual graph seed expansion uses native Graph owners; opaque owner keys are metadata, not canonical names.\n";
const REFERENCE_PAGES_HELP: &str = "Native Reference query accepts optional --page-json <wow-search/native-reference-page-policy/1 JSON>. wow search reference continue --continuation-json <EXACT-TOKEN> --search-store <SAME-STORE> --operation-id <ID> --allow-local-metadata-only <original Store/output limits>. wow search reference results release --session-object-id <EXACT> --search-store <SAME-STORE> --operation-id <ID> --allow-local-metadata-only <Store/output limits>. Result, shard and original ReferenceView remain rooted until explicit release.\n";
const REFERENCE_HELP: &str = "wow search reference build --config <PINNED-SOURCE-REFERENCE-JSON> --expect-config-sha256 <DIGEST> --search-store <STORE> --operation-id <ID> --allow-local-metadata-only <Search/Store/output bounds>\nwow search reference query --handle-json <EXACT-HANDLE> --query-json <CLOSED-QUERY> --search-store <SAME-STORE> --operation-id <ID> --lease-expires-after <EPOCH> --allow-local-metadata-only <Store/output bounds>\nwow search reference retire --handle-json <EXACT-HANDLE> --search-store <SAME-STORE> --operation-id <ID> --lease-expires-after <EPOCH> --allow-local-metadata-only <Store/output bounds>\n";
const RETRIEVAL_HELP: &str = "wow search retrieve --search-store <STORE> --generation <EXACT> --search-shard-id <ID> --object-id <ID> --profile-digest <DIGEST> --query-json <CLOSED-QUERY-JSON> --operation-id <ID> --lease-expires-after <EPOCH> --allow-local-metadata-only <finite bounds>. This operation selects the exact file-set component and returns an immutable ranked result manifest. Without --page-json it advertises no continuation.\n";
const PAGINATION_HELP: &str = "Optional retrieve --page-json <CLOSED-PAGE-POLICY> retains an immutable result and exact shard before returning page one. wow search continue --continuation-json <EXACT-TOKEN> --search-store <STORE> --operation-id <ID> --allow-local-metadata-only <original Store/output limits>. wow search results release --session-object-id <EXACT> --search-store <STORE> --operation-id <ID> --allow-local-metadata-only <Store/output limits> releases both roots; completed response replay remains retained.\n";
const DURABLE_FILE_SET_HELP: &str = "Durable index build/retire accept either --file-id <ID> or --file-ids <JSON-array-of-exact-IDs>. Durable query uses its real --file-id plus --file-set for that component. Preserve the full original selection and source-hold receipt for retirement.\n";
const FILE_SET_HELP: &str = "For one-shot build-query, optional --file-ids <JSON-array-of-exact-ProjectFileIds> selects a finite set. --file-id must belong to it. Empty/duplicate/absent/over-cap selections refuse; --max-documents caps the raw set. No durable file-set handle is returned.\n";
const OBSERVATIONS_HELP: &str = "Source observation index build/retire use --source-observations instead of file/Graph selection. wow search observations --generation <EXACT> --search-shard-id <EXACT> --object-id <EXACT> --profile-digest <EXACT> --query-json <CLOSED-JSON> uses original Search/Store/output bounds, --search-store, --operation-id, --lease-expires-after and --allow-local-metadata-only. Recorded local names/direct member use sites retain source evidence; they confer no canonical name, alias, runtime target or absence authority.\n";
const DECLARATIONS_HELP: &str = "Optional --source-declarations selects the named-function mode on source index build/retire and search observations, with its exact component/profile and wow-search/source-observation-declaration-query/1 schema. Qualified/short names and recorded dot/colon receiver paths come from native declaration tokens; assignments/aliases/anonymous closures are not inferred. See SEARCH_SOURCE_OBSERVATIONS.md.\n";
const MAX_ARGUMENT_BYTES: usize = 128 * 1024;
const MAX_VALUE_BYTES: usize = 32 * 1024;

const VALUE_OPTIONS: &[&str] = &[
    "--store-root",
    "--generation",
    "--file-id",
    "--file-ids",
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

struct ParsedDurableSearch {
    command: DurableCommand,
    max_output_bytes: u64,
}

enum DurableCommand {
    Observations(SourceObservationRetrievalRequest),
    FederatedContinue(FederationContinueRequest),
    FederatedRelease(FederationResultReleaseRequest),
    Federated(Box<FederationRequest>),
    EntityContinue(EntityContinueRequest),
    EntityRelease(EntityResultReleaseRequest),
    Entities(EntityRetrievalRequest),
    ReferenceContinue(ReferenceContinueRequest),
    ReferenceRelease(ReferenceResultReleaseRequest),
    ReferenceBuild(ReferenceIndexBuildRequest),
    ReferenceQuery(ReferenceShardRequest),
    ReferenceRetire(ReferenceShardRequest),
    Build(SearchIndexBuildRequest),
    Query(SearchShardQueryRequest),
    Retire(SearchShardRetireRequest),
    Gc(SearchStoreGcRequest),
    Retrieve(SearchRetrievalRequest),
    Continue(SearchContinueRequest),
    Release(SearchResultReleaseRequest),
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
            .and_then(|()| stdout.write_all(FILE_SET_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(DURABLE_FILE_SET_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(RETRIEVAL_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(PAGINATION_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(REFERENCE_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(REFERENCE_PAGES_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(ENTITIES_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(ENTITY_PAGES_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(OBSERVATIONS_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(DECLARATIONS_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(FEDERATION_HELP.as_bytes()))
            .and_then(|()| stdout.write_all(BLIZZARD_FEDERATION_HELP.as_bytes()))
            .and_then(|()| stdout.flush())
        {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    match parse_durable(&arguments) {
        Ok(Some(parsed)) => {
            return run_durable(parsed, stop, stdout, stderr);
        }
        Ok(None) => {}
        Err(message) => {
            diagnostic(stderr, message);
            return 64;
        }
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

fn run_durable<W: Write, E: Write>(
    parsed: ParsedDurableSearch,
    stop: &AtomicBool,
    stdout: &mut W,
    stderr: &mut E,
) -> u8 {
    if stop.load(Ordering::Acquire) {
        diagnostic(stderr, "search operation cancelled");
        return 130;
    }
    let service = SearchOperationService::new();
    let result = match parsed.command {
        DurableCommand::FederatedContinue(request) => {
            service.continue_federated_results(&request, stop)
        }
        DurableCommand::FederatedRelease(request) => {
            service.release_federated_results(&request, stop)
        }
        DurableCommand::Federated(request) => service.federate_search(&request, stop),
        DurableCommand::EntityContinue(request) => service.continue_entity_results(&request, stop),
        DurableCommand::EntityRelease(request) => service.release_entity_results(&request, stop),
        DurableCommand::Entities(request) => service.retrieve_entities(&request, stop),
        DurableCommand::Observations(request) => {
            service.retrieve_source_observations(&request, stop)
        }
        DurableCommand::ReferenceContinue(request) => {
            service.continue_reference_results(&request, stop)
        }
        DurableCommand::ReferenceRelease(request) => {
            service.release_reference_results(&request, stop)
        }
        DurableCommand::ReferenceBuild(request) => service.build_reference_shard(&request, stop),
        DurableCommand::ReferenceQuery(request) => service.query_reference_shard(&request, stop),
        DurableCommand::ReferenceRetire(request) => service.retire_reference_shard(&request, stop),
        DurableCommand::Build(request) => service.build_shard(&request, stop),
        DurableCommand::Query(request) => service.query_shard(&request, stop),
        DurableCommand::Retire(request) => service.retire_shard(&request, stop),
        DurableCommand::Gc(request) => service.collect_search_garbage(&request, stop),
        DurableCommand::Retrieve(request) => service.retrieve_shard(&request, stop),
        DurableCommand::Continue(request) => service.continue_results(&request, stop),
        DurableCommand::Release(request) => service.release_results(&request, stop),
    };
    let bytes = match result {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostic(
                stderr,
                &format!("search {:?}: {}", error.code(), error.message()),
            );
            return service_exit(error.code());
        }
    };
    if stop.load(Ordering::Acquire) {
        diagnostic(stderr, "search operation cancelled before output");
        return 130;
    }
    let framed_len = match bytes.len().checked_add(1) {
        Some(length)
            if u64::try_from(length).is_ok_and(|length| length <= parsed.max_output_bytes) =>
        {
            length
        }
        _ => {
            diagnostic(stderr, "search output exceeds the caller byte bound");
            return 4;
        }
    };
    let mut framed = Vec::with_capacity(framed_len);
    let Some(success_exit) = durable_status_exit(&bytes) else {
        diagnostic(stderr, "search result has an unsupported status");
        return 4;
    };
    framed.extend_from_slice(&bytes);
    framed.push(b'\n');
    match stdout.write_all(&framed).and_then(|()| stdout.flush()) {
        Ok(()) => success_exit,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::BrokenPipe {
                diagnostic(stderr, "search output failed");
            }
            4
        }
    }
}

fn durable_status_exit(bytes: &[u8]) -> Option<u8> {
    let envelope: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    match envelope.get("payload")?.get("status")?.as_str()? {
        "candidate_only"
        | "exact_not_found_partial"
        | "no_candidates_under_executed_lanes"
        | "lane_unavailable"
        | "truncated"
        | "conflict_blocked" => Some(2),
        "exact_found" | "exact_not_found_with_authority" => Some(0),
        "retained" | "retired" | "collected" | "released" => Some(0),
        _ => None,
    }
}

fn service_exit(code: ServiceErrorCode) -> u8 {
    match code {
        ServiceErrorCode::Cancelled => 130,
        ServiceErrorCode::InvalidRequest
        | ServiceErrorCode::IdentityMismatch
        | ServiceErrorCode::ExactGenerationUnavailable
        | ServiceErrorCode::OperationConflict
        | ServiceErrorCode::OperationReleased
        | ServiceErrorCode::StoreOutcomeUnknown
        | ServiceErrorCode::BudgetExceeded => 3,
        _ => 4,
    }
}

fn is_help(arguments: &[OsString]) -> bool {
    (arguments.len() == 2 && arguments[0] == "search" && arguments[1] == "--help")
        || (arguments.len() == 3
            && arguments[0] == "search"
            && arguments[1] == "build-query"
            && (arguments[2] == "--help" || arguments[2] == "-h"))
        || (arguments.len() >= 3
            && arguments[0] == "search"
            && arguments
                .iter()
                .any(|value| value == "--help" || value == "-h")
            && (arguments[1] == "query"
                || arguments[1] == "retrieve"
                || arguments[1] == "observations"
                || arguments[1] == "index"
                || arguments[1] == "continue"
                || arguments[1] == "results"))
        || (arguments.len() >= 3
            && arguments[0] == "search"
            && arguments[1] == "federated"
            && arguments.iter().any(|v| v == "--help" || v == "-h"))
        || (arguments.len() >= 3
            && arguments[0] == "search"
            && arguments[1] == "entities"
            && arguments.iter().any(|v| v == "--help" || v == "-h"))
        || (arguments.len() >= 3
            && arguments[0] == "search"
            && arguments[1] == "reference"
            && arguments.iter().any(|v| v == "--help" || v == "-h"))
}

fn parse_durable(arguments: &[OsString]) -> Result<Option<ParsedDurableSearch>, &'static str> {
    if arguments.first().map(OsString::as_os_str) != Some(std::ffi::OsStr::new("search")) {
        return Ok(None);
    }
    let Some(command_word) = arguments.get(1).and_then(|value| value.to_str()) else {
        return Ok(None);
    };
    let (command, prefix_len) = match command_word {
        "federated" => match arguments.get(2).and_then(|value| value.to_str()) {
            Some("continue") => ("federated-continue", 3),
            Some("results") if arguments.get(3).is_some_and(|value| value == "release") => {
                ("federated-release", 4)
            }
            _ => ("federated", 2),
        },
        "entities" => match arguments.get(2).and_then(|v| v.to_str()) {
            Some("continue") => ("entity-continue", 3),
            Some("results") if arguments.get(3).is_some_and(|v| v == "release") => {
                ("entity-release", 4)
            }
            _ => ("entities", 2),
        },
        "reference" => match arguments.get(2).and_then(|v| v.to_str()) {
            Some("build") => ("reference-build", 3),
            Some("query") => ("reference-query", 3),
            Some("retire") => ("reference-retire", 3),
            Some("continue") => ("reference-continue", 3),
            Some("results") if arguments.get(3).is_some_and(|value| value == "release") => {
                ("reference-release", 4)
            }
            _ => return Ok(None),
        },
        "observations" => ("observations", 2),
        "query" => ("query", 2),
        "retrieve" => ("retrieve", 2),
        "continue" => ("continue", 2),
        "results" if arguments.get(2).is_some_and(|value| value == "release") => ("release", 3),
        "index" => match arguments.get(2).and_then(|value| value.to_str()) {
            Some("build") => ("build", 3),
            Some("retire") => ("retire", 3),
            Some("gc") => ("gc", 3),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    let total_bytes = arguments
        .iter()
        .try_fold(0usize, |total, value| total.checked_add(value.len()))
        .ok_or("search argument size overflow")?;
    if arguments.len() > MAX_ARGUMENTS || total_bytes > MAX_ARGUMENT_BYTES {
        return Err("search argument limit exceeded");
    }

    const BOUNDS_OPTIONS: &[&str] = &[
        "--max-documents",
        "--max-document-id-bytes",
        "--max-owner-metadata-label-bytes",
        "--max-literal-query-bytes",
        "--max-candidate-results",
        "--max-owner-evidence-records",
        "--max-owner-manifest-bytes",
        "--max-database-bytes",
        "--max-search-object-bytes",
    ];
    const STORE_OPTIONS: &[&str] = &[
        "--store-max-object-bytes",
        "--store-max-batch-objects",
        "--store-max-catalog-mutations",
        "--store-max-manifest-records",
        "--store-max-gc-deletes",
    ];
    let mut allowed = vec!["--search-store", "--operation-id", "--max-output-bytes"];
    match command {
        "federated" => {
            allowed.extend_from_slice(&["--request-json", "--lease-expires-after", "--page-json"])
        }
        "federated-continue" => allowed.push("--continuation-json"),
        "federated-release" => allowed.push("--session-object-id"),
        "reference-build" => allowed.extend_from_slice(&["--config", "--expect-config-sha256"]),
        "reference-query" => allowed.extend_from_slice(&[
            "--handle-json",
            "--query-json",
            "--lease-expires-after",
            "--page-json",
        ]),
        "reference-retire" => {
            allowed.extend_from_slice(&["--handle-json", "--lease-expires-after"])
        }
        "reference-continue" => allowed.push("--continuation-json"),
        "reference-release" => allowed.push("--session-object-id"),
        "entity-continue" => allowed.push("--continuation-json"),
        "entity-release" => allowed.push("--session-object-id"),
        "build" => allowed.extend_from_slice(&[
            "--store-root",
            "--generation",
            "--file-id",
            "--file-ids",
            "--graph-entities",
            "--source-observations",
            "--source-declarations",
        ]),
        "query" => allowed.extend_from_slice(&[
            "--generation",
            "--file-id",
            "--search-shard-id",
            "--object-id",
            "--profile-digest",
            "--literal-query",
            "--lease-expires-after",
            "--file-set",
        ]),
        "entities" => allowed.extend_from_slice(&[
            "--store-root",
            "--generation",
            "--search-shard-id",
            "--object-id",
            "--profile-digest",
            "--build-operation-id",
            "--source-store-epoch-id",
            "--source-generation-hold-id",
            "--source-generation-hold-digest",
            "--lease-expires-after",
            "--query-json",
            "--page-json",
        ]),
        "retrieve" => allowed.extend_from_slice(&[
            "--generation",
            "--search-shard-id",
            "--object-id",
            "--profile-digest",
            "--query-json",
            "--page-json",
            "--lease-expires-after",
        ]),
        "observations" => allowed.extend_from_slice(&[
            "--generation",
            "--search-shard-id",
            "--object-id",
            "--profile-digest",
            "--query-json",
            "--lease-expires-after",
            "--source-declarations",
        ]),
        "continue" => allowed.push("--continuation-json"),
        "release" => allowed.push("--session-object-id"),
        "retire" => allowed.extend_from_slice(&[
            "--store-root",
            "--generation",
            "--file-id",
            "--file-ids",
            "--graph-entities",
            "--source-observations",
            "--source-declarations",
            "--search-shard-id",
            "--object-id",
            "--profile-digest",
            "--build-operation-id",
            "--source-store-epoch-id",
            "--source-generation-hold-id",
            "--source-generation-hold-digest",
            "--lease-expires-after",
        ]),
        "gc" => allowed.extend_from_slice(&["--target-object-id", "--gc-now", "--gc-max-deletes"]),
        _ => return Err("unknown durable search command"),
    }
    if !matches!(
        command,
        "gc" | "continue"
            | "federated"
            | "federated-continue"
            | "federated-release"
            | "entity-continue"
            | "entity-release"
            | "release"
            | "reference-query"
            | "reference-retire"
            | "reference-continue"
            | "reference-release"
    ) {
        allowed.extend_from_slice(BOUNDS_OPTIONS);
    }
    allowed.extend_from_slice(STORE_OPTIONS);
    let (options, local_permission) =
        parse_durable_options(&arguments[prefix_len..], &allowed, command != "gc")?;
    let store_path = PathBuf::from(required(&options, "--search-store")?);
    if !store_path.is_absolute() {
        return Err("--search-store must be an absolute Store file path");
    }
    let store_limits = parse_store_limits(&options)?;
    let max_output_bytes = positive_u64(&options, "--max-output-bytes")?;
    let command = match command {
        "federated-continue" => DurableCommand::FederatedContinue(
            FederationContinueRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--continuation-json")?.as_bytes(),
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact federated continuation")?,
        ),
        "federated-release" => DurableCommand::FederatedRelease(
            FederationResultReleaseRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--session-object-id")?,
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid exact federated result release")?,
        ),
        "federated" => {
            let mut request = FederationRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--request-json")?.as_bytes(),
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact federation request")?;
            if let Some(policy) = options.get("--page-json") {
                request = request
                    .with_page_policy(
                        policy
                            .to_str()
                            .ok_or("page policy is not UTF-8")?
                            .as_bytes(),
                        &AtomicBool::new(false),
                    )
                    .map_err(|_| "invalid federated page policy")?;
            }
            DurableCommand::Federated(Box::new(request))
        }
        "entity-continue" => DurableCommand::EntityContinue(
            EntityContinueRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--continuation-json")?.as_bytes(),
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact ProjectEntity continuation")?,
        ),
        "entity-release" => DurableCommand::EntityRelease(
            EntityResultReleaseRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--session-object-id")?,
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid exact ProjectEntity session release")?,
        ),
        "observations" => DurableCommand::Observations(
            SourceObservationRetrievalRequest::new_selected(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--generation")?,
                required_text(&options, "--search-shard-id")?,
                required_text(&options, "--object-id")?,
                required_text(&options, "--profile-digest")?,
                required_text(&options, "--query-json")?.as_bytes(),
                parse_search_bounds(&options)?,
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
                options.contains_key("--source-declarations"),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact source observation Search request")?,
        ),
        "entities" => {
            let hold = SearchSourceHoldBinding::new(
                PathBuf::from(required(&options, "--store-root")?),
                OperationId::new(required_text(&options, "--build-operation-id")?)
                    .map_err(|_| "invalid build operation")?,
                required_text(&options, "--source-store-epoch-id")?,
                required_text(&options, "--source-generation-hold-id")?,
                required_text(&options, "--source-generation-hold-digest")?,
            )
            .map_err(|_| "invalid exact source hold")?;
            let mut request = EntityRetrievalRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                hold,
                required_text(&options, "--generation")?,
                required_text(&options, "--search-shard-id")?,
                required_text(&options, "--object-id")?,
                required_text(&options, "--profile-digest")?,
                required_text(&options, "--query-json")?.as_bytes(),
                parse_search_bounds(&options)?,
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid ProjectEntity Search request")?;
            if let Some(policy) = options.get("--page-json") {
                request = request
                    .with_page_policy(
                        policy
                            .to_str()
                            .ok_or("page policy is not UTF-8")?
                            .as_bytes(),
                        &AtomicBool::new(false),
                    )
                    .map_err(|_| "invalid ProjectEntity page policy")?;
            }
            DurableCommand::Entities(request)
        }
        "reference-continue" => DurableCommand::ReferenceContinue(
            ReferenceContinueRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--continuation-json")?.as_bytes(),
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact Reference continuation")?,
        ),
        "reference-release" => DurableCommand::ReferenceRelease(
            ReferenceResultReleaseRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--session-object-id")?,
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid exact Reference session release")?,
        ),
        "reference-build" => DurableCommand::ReferenceBuild(
            ReferenceIndexBuildRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                PathBuf::from(required(&options, "--config")?),
                required_text(&options, "--expect-config-sha256")?,
                store_path,
                parse_search_bounds(&options)?,
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid native Reference Search build request")?,
        ),
        "reference-query" | "reference-retire" => {
            let request = ReferenceShardRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--handle-json")?.as_bytes(),
                if command == "reference-query" {
                    Some(required_text(&options, "--query-json")?.as_bytes())
                } else {
                    None
                },
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact Reference Search handle/query")?;
            if command == "reference-query" {
                let request = if let Some(bytes) = options.get("--page-json") {
                    request
                        .with_pagination_json(
                            bytes
                                .to_str()
                                .ok_or("invalid page JSON encoding")?
                                .as_bytes(),
                            &AtomicBool::new(false),
                        )
                        .map_err(|_| "invalid Reference page policy")?
                } else {
                    request
                };
                DurableCommand::ReferenceQuery(request)
            } else {
                DurableCommand::ReferenceRetire(request)
            }
        }
        "continue" => DurableCommand::Continue(
            SearchContinueRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--continuation-json")?.as_bytes(),
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact continuation request")?,
        ),
        "release" => DurableCommand::Release(
            SearchResultReleaseRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--session-object-id")?,
                store_limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid exact result release request")?,
        ),
        "retrieve" => {
            let request = SearchRetrievalRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--generation")?,
                required_text(&options, "--search-shard-id")?,
                required_text(&options, "--object-id")?,
                required_text(&options, "--profile-digest")?,
                required_text(&options, "--query-json")?.as_bytes(),
                parse_search_bounds(&options)?,
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
                &AtomicBool::new(false),
            )
            .map_err(|_| "invalid exact retrieval request")?;
            let request = if let Some(bytes) = options.get("--page-json") {
                request
                    .with_pagination_json(
                        bytes
                            .to_str()
                            .ok_or("invalid page JSON encoding")?
                            .as_bytes(),
                        &AtomicBool::new(false),
                    )
                    .map_err(|_| "invalid page policy")?
            } else {
                request
            };
            DurableCommand::Retrieve(request)
        }
        "build" => {
            let project_store_root = PathBuf::from(required(&options, "--store-root")?);
            let bounds = parse_search_bounds(&options)?;
            let operation_id = OperationId::new(required_text(&options, "--operation-id")?)
                .map_err(|_| "invalid --operation-id")?;
            let request = if source_observation_selection(&options)? {
                SearchIndexBuildRequest::new_source_observations(
                    operation_id,
                    PathBuf::from(required(&options, "--store-root")?),
                    store_path,
                    required_text(&options, "--generation")?,
                    bounds,
                    store_limits,
                    max_output_bytes,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                )
            } else if graph_entity_selection(&options)? {
                SearchIndexBuildRequest::new_graph_entities(
                    operation_id,
                    project_store_root,
                    store_path,
                    required_text(&options, "--generation")?,
                    bounds,
                    store_limits,
                    max_output_bytes,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                )
            } else {
                match durable_file_ids(&options)? {
                    Some(ids) => SearchIndexBuildRequest::new_file_set(
                        operation_id,
                        project_store_root,
                        store_path,
                        required_text(&options, "--generation")?,
                        ids,
                        bounds,
                        store_limits,
                        max_output_bytes,
                        LocalMetadataPermission::authorize_local_metadata_only(),
                        &AtomicBool::new(false),
                    ),
                    None => SearchIndexBuildRequest::new(
                        operation_id,
                        project_store_root,
                        store_path,
                        required_text(&options, "--generation")?,
                        required_text(&options, "--file-id")?,
                        bounds,
                        store_limits,
                        max_output_bytes,
                        LocalMetadataPermission::authorize_local_metadata_only(),
                    ),
                }
            }
            .map_err(|_| "invalid exact search build request")?;
            let request = if options.contains_key("--source-declarations") {
                request
                    .with_function_declarations()
                    .map_err(|_| "invalid named declaration selection")?
            } else {
                request
            };
            DurableCommand::Build(request)
        }
        "query" => {
            let bounds = parse_search_bounds(&options)?;
            let literal_query = required_text(&options, "--literal-query")?;
            if literal_query.trim().is_empty() {
                return Err("--literal-query must be nonempty");
            }
            let request = SearchShardQueryRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--generation")?,
                required_text(&options, "--file-id")?,
                required_text(&options, "--search-shard-id")?,
                required_text(&options, "--object-id")?,
                required_text(&options, "--profile-digest")?,
                literal_query,
                bounds,
                store_limits,
                max_output_bytes,
                positive_u64(&options, "--lease-expires-after")?.get(),
                LocalMetadataPermission::authorize_local_metadata_only(),
            )
            .map_err(|_| "invalid exact search query request")?;
            DurableCommand::Query(if options.contains_key("--file-set") {
                request.with_file_set_component()
            } else {
                request
            })
        }
        "retire" => {
            let bounds = parse_search_bounds(&options)?;
            let source_hold = SearchSourceHoldBinding::new(
                PathBuf::from(required(&options, "--store-root")?),
                OperationId::new(required_text(&options, "--build-operation-id")?)
                    .map_err(|_| "invalid --build-operation-id")?,
                required_text(&options, "--source-store-epoch-id")?,
                required_text(&options, "--source-generation-hold-id")?,
                required_text(&options, "--source-generation-hold-digest")?,
            )
            .map_err(|_| "invalid exact source hold binding")?;
            let operation_id = OperationId::new(required_text(&options, "--operation-id")?)
                .map_err(|_| "invalid --operation-id")?;
            let permission = if local_permission {
                LocalMetadataPermission::authorize_local_metadata_only()
            } else {
                return Err("search retirement requires --allow-local-metadata-only");
            };
            let request = if source_observation_selection(&options)? {
                SearchShardRetireRequest::new_source_observations(
                    operation_id,
                    store_path,
                    source_hold,
                    required_text(&options, "--generation")?,
                    required_text(&options, "--search-shard-id")?,
                    required_text(&options, "--object-id")?,
                    required_text(&options, "--profile-digest")?,
                    bounds,
                    store_limits,
                    max_output_bytes,
                    positive_u64(&options, "--lease-expires-after")?.get(),
                    permission,
                )
            } else if graph_entity_selection(&options)? {
                SearchShardRetireRequest::new_graph_entities(
                    operation_id,
                    store_path,
                    source_hold,
                    required_text(&options, "--generation")?,
                    required_text(&options, "--search-shard-id")?,
                    required_text(&options, "--object-id")?,
                    required_text(&options, "--profile-digest")?,
                    bounds,
                    store_limits,
                    max_output_bytes,
                    positive_u64(&options, "--lease-expires-after")?.get(),
                    permission,
                )
            } else {
                match durable_file_ids(&options)? {
                    Some(ids) => SearchShardRetireRequest::new_file_set(
                        operation_id,
                        store_path,
                        source_hold,
                        required_text(&options, "--generation")?,
                        ids,
                        required_text(&options, "--search-shard-id")?,
                        required_text(&options, "--object-id")?,
                        required_text(&options, "--profile-digest")?,
                        bounds,
                        store_limits,
                        max_output_bytes,
                        positive_u64(&options, "--lease-expires-after")?.get(),
                        permission,
                        &AtomicBool::new(false),
                    ),
                    None => SearchShardRetireRequest::new(
                        operation_id,
                        store_path,
                        source_hold,
                        required_text(&options, "--generation")?,
                        required_text(&options, "--file-id")?,
                        required_text(&options, "--search-shard-id")?,
                        required_text(&options, "--object-id")?,
                        required_text(&options, "--profile-digest")?,
                        bounds,
                        store_limits,
                        max_output_bytes,
                        positive_u64(&options, "--lease-expires-after")?.get(),
                        permission,
                    ),
                }
            }
            .map_err(|_| "invalid exact search retirement request")?;
            let request = if options.contains_key("--source-declarations") {
                request
                    .with_function_declarations()
                    .map_err(|_| "invalid named declaration retirement selection")?
            } else {
                request
            };
            DurableCommand::Retire(request)
        }
        "gc" => DurableCommand::Gc(
            SearchStoreGcRequest::new(
                OperationId::new(required_text(&options, "--operation-id")?)
                    .map_err(|_| "invalid --operation-id")?,
                store_path,
                required_text(&options, "--target-object-id")?,
                store_limits,
                unsigned_u64(&options, "--gc-now")?,
                positive_u32(&options, "--gc-max-deletes")?.get(),
                max_output_bytes,
            )
            .map_err(|_| "invalid exact search GC request")?,
        ),
        _ => return Err("unknown durable search command"),
    };
    Ok(Some(ParsedDurableSearch {
        command,
        max_output_bytes: max_output_bytes.get(),
    }))
}

fn parse_durable_options(
    values: &[OsString],
    allowed: &[&'static str],
    require_permission: bool,
) -> Result<(BTreeMap<&'static str, OsString>, bool), &'static str> {
    let mut options = BTreeMap::new();
    let mut allow_local_metadata = false;
    let mut values = values.iter();
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid option encoding")?;
        if option == "--file-set"
            || option == "--graph-entities"
            || option == "--source-observations"
            || option == "--source-declarations"
        {
            let key = if option == "--source-declarations" {
                "--source-declarations"
            } else if option == "--source-observations" {
                "--source-observations"
            } else if option == "--file-set" {
                "--file-set"
            } else {
                "--graph-entities"
            };
            if !allowed.contains(&key) {
                return Err("document component flag is not valid for this command");
            }
            if options.insert(key, OsString::from("true")).is_some() {
                return Err("duplicate document component flag");
            }
            continue;
        }
        if option == "--allow-local-metadata-only" {
            if allow_local_metadata {
                return Err("duplicate local metadata permission");
            }
            allow_local_metadata = true;
            continue;
        }
        let key = allowed
            .iter()
            .copied()
            .find(|candidate| *candidate == option)
            .ok_or("unknown durable search option")?;
        let value = values.next().ok_or("missing durable search option value")?;
        if value.is_empty()
            || value.len() > MAX_VALUE_BYTES
            || value.to_str().is_some_and(|value| value.starts_with("--"))
        {
            return Err("invalid durable search option value");
        }
        if options.insert(key, value.clone()).is_some() {
            return Err("duplicate durable search option");
        }
    }
    if require_permission && !allow_local_metadata {
        return Err("search requires --allow-local-metadata-only");
    }
    if !require_permission && allow_local_metadata {
        return Err("GC does not accept metadata permission flags");
    }
    Ok((options, allow_local_metadata))
}

fn durable_file_ids(
    options: &BTreeMap<&'static str, OsString>,
) -> Result<Option<Vec<String>>, &'static str> {
    if options.contains_key("--file-id") && options.contains_key("--file-ids") {
        return Err("choose --file-id or --file-ids, never both");
    }
    options
        .get("--file-ids")
        .map(|value| {
            let text = value.to_str().ok_or("--file-ids must be UTF-8 JSON")?;
            serde_json::from_str(text)
                .map_err(|_| "--file-ids must be an explicit JSON array of exact ProjectFile IDs")
        })
        .transpose()
}

fn source_observation_selection(
    options: &BTreeMap<&'static str, OsString>,
) -> Result<bool, &'static str> {
    let selected = options.contains_key("--source-observations")
        || options.contains_key("--source-declarations");
    if options.contains_key("--source-observations")
        && options.contains_key("--source-declarations")
    {
        return Err("select one source observation mode");
    }
    if selected
        && (options.contains_key("--file-id")
            || options.contains_key("--file-ids")
            || options.contains_key("--graph-entities"))
    {
        return Err("--source-observations conflicts with other document selections");
    }
    Ok(selected)
}

fn graph_entity_selection(
    options: &BTreeMap<&'static str, OsString>,
) -> Result<bool, &'static str> {
    let selected = options.contains_key("--graph-entities");
    if selected && (options.contains_key("--file-id") || options.contains_key("--file-ids")) {
        return Err("--graph-entities conflicts with file selection");
    }
    Ok(selected)
}

fn parse_search_bounds(
    options: &BTreeMap<&'static str, OsString>,
) -> Result<SearchBounds, &'static str> {
    SearchBounds::new(
        positive_u32(options, "--max-documents")?,
        positive_u32(options, "--max-document-id-bytes")?,
        positive_u32(options, "--max-owner-metadata-label-bytes")?,
        positive_u32(options, "--max-literal-query-bytes")?,
        positive_u32(options, "--max-candidate-results")?,
        positive_u32(options, "--max-owner-evidence-records")?,
        positive_u64(options, "--max-owner-manifest-bytes")?,
        positive_u64(options, "--max-database-bytes")?,
        positive_u64(options, "--max-search-object-bytes")?,
    )
    .map_err(|_| "invalid or inconsistent search bounds")
}

fn parse_store_limits(
    options: &BTreeMap<&'static str, OsString>,
) -> Result<SearchStoreLimits, &'static str> {
    SearchStoreLimits::new(
        positive_u64(options, "--store-max-object-bytes")?.get(),
        positive_u32(options, "--store-max-batch-objects")?.get(),
        positive_u32(options, "--store-max-catalog-mutations")?.get(),
        positive_u32(options, "--store-max-manifest-records")?.get(),
        positive_u32(options, "--store-max-gc-deletes")?.get(),
    )
    .map_err(|_| "invalid Store limits")
}

fn unsigned_u64(
    options: &BTreeMap<&'static str, OsString>,
    key: &str,
) -> Result<u64, &'static str> {
    let value = required_text(options, key)?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("numeric search values must be unsigned decimal integers");
    }
    value.parse().map_err(|_| "search value is out of range")
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
    let request = match options.get("--file-ids") {
        Some(value) => {
            let text = value.to_str().ok_or("--file-ids must be UTF-8 JSON")?;
            let ids: Vec<String> = serde_json::from_str(text)
                .map_err(|_| "--file-ids must be an explicit JSON array of ProjectFile IDs")?;
            request
                .with_file_set(ids, &AtomicBool::new(false))
                .map_err(|_| "invalid explicit ProjectFile set")?
        }
        None => request,
    };
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
