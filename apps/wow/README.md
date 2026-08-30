# `apps/wow` E0 CLI contract

**Status:** E0-F implementation-ready transport contract; no Rust code yet.

## Mission

`apps/wow` is a thin command-line projection over `wow-service`. It parses a small explicit command/option surface, constructs transport-neutral service requests, serializes the returned service result, and maps service operation status to process exit codes.

It does not inspect source, coordinate lower crates, run diagnostics/rules directly, alter semantic records, or implement future commands.

## Required reading

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../README.md`](../README.md)
3. [`../../crates/AGENTS.md`](../../crates/AGENTS.md)
4. [`../../crates/wow-service/README.md`](../../crates/wow-service/README.md)
5. [`../../crates/wow-service/CONTRACT.json`](../../crates/wow-service/CONTRACT.json)
6. [`../../crates/wow-service/RESULT_ENVELOPE.md`](../../crates/wow-service/RESULT_ENVELOPE.md)
7. [`AGENTS.md`](AGENTS.md)
8. [`CONTRACT.json`](CONTRACT.json)

## Direct dependency boundary

Framework dependency:

```text
wow-service
```

Allowed nonframework dependencies are limited to argument parsing, canonical JSON serialization required by the service contract, and safe terminal text projection.

Forbidden direct framework dependencies:

```text
wow-core
wow-reference
wow-emmy
wow-project
wow-rules
all later crates
```

The application receives public service result types through `wow-service`; it does not rebuild them from lower records.

## E0 commands

```text
wow status
wow check
```

Every other command is typed unavailable or a CLI usage error according to whether it reached the service deferred-operation surface.

## `wow status`

Conceptual syntax:

```text
wow status [--project <ProjectId>] [--detail summary|capabilities] [--format json|text]
```

Behavior:

- construct `StatusRequest`;
- invoke `wow-service status`;
- write canonical JSON or noncanonical text projection;
- map operation state to exit code;
- do not run check/diagnostics;
- do not add pass/clean/test/runtime claims.

## `wow check`

Conceptual syntax:

```text
wow check \
  --project <ProjectId> \
  [--generation current|<ProjectGenerationId>] \
  [--file <ProjectFileId> ...] \
  [--rule <RuleId>@<version> ...] \
  [--format json|text]
```

E0 defaults:

```text
generation = current (explicit CurrentPublished selector scoped to project)
scope = AllProjectFiles
rules = E0 service default registry
include generic findings = true
presentation = raw_and_roots
format = json
```

For golden tests, pass an exact frozen generation.

Rules:

- `current` is sent as `CurrentPublished(ProjectId)`; service resolves exact generation once;
- CLI does not resolve project current pointer itself;
- `--file` accepts canonical ProjectFileId, not filesystem path/glob;
- `--rule` accepts only active E0 rule IDs/versions;
- no source stdin/file path scan;
- no autofix/apply/write flag;
- no search/replacement/runtime option.

## Output formats

### `json`

Canonical E0 format.

- stdout contains exactly the canonical service result JSON bytes plus one final newline;
- no banners/progress/logs on stdout;
- field/order/content identical to service canonical serialization;
- CLI does not add hostname/time/path/command prose;
- structured ServiceFailureResult/ServiceCancelledResult also go to stdout for syntactically valid commands reaching service;
- stderr remains empty unless a CLI/startup/serialization fault occurs.

### `text`

Noncanonical human projection.

- derives only from the service result;
- may show status, exact generations, display roots/children, raw counts, blockers, and deferred state;
- must not omit the existence of raw findings/NotEvaluated records merely because folded;
- never changes exit code/service semantics;
- no text golden is a semantic identity oracle.

## Exit codes

### `status`

```text
0   available
2   partial
3   structured request/context/configuration/unavailable/deferred failure
4   internal component/result-contract/serialization failure
64  CLI syntax/argument/config-loading error before service invocation
130 cancelled
```

### `check`

```text
0   semantic clean
1   semantic findings
2   semantic partial
3   structured request/context/generation/deferred/unavailable failure
4   internal component/provider/presentation/envelope/serialization failure
64  CLI syntax/argument/config-loading error before service invocation
130 cancelled
```

Exit code is a projection of service/CLI failure classification, not a hidden release policy.

E0 advisory rule findings still map semantic `findings` to exit 1. The records retain rollout `advisory`; future policy modes require an explicit contract and cannot silently change this default.

## Failure classification

Exit 3 examples:

```text
invalid service request after parsing
unknown project
exact generation unavailable
profile/context mismatch
deferred operation requested
```

Exit 4 examples:

```text
component/provider contract failure
invalid presentation graph/envelope
canonical serialization failure
internal invariant violation
```

CLI parser errors that cannot create a service request use exit 64 and a minimal usage error on stderr.

## Stdout/stderr discipline

- JSON operation results: stdout only.
- Parser/startup error: stderr only, no fabricated service JSON.
- Text operation result: stdout; operational logging remains disabled by default or stderr.
- Never write source text, Secret-capable values, absolute local paths, credentials, or private URLs.
- Broken-pipe handling must not produce a second conflicting semantic result.

## Deferred commands

If CLI syntax recognizes a roadmap command solely to route a typed service deferred error, output the ServiceFailureResult and mapped exit 3.

Alternatively, E0 parser may reject unknown commands with exit 64. The implementation must freeze one explicit command-discovery policy before code and keep `status` deferred registry authoritative.

It must never return empty success.

## Determinism

Given the same canonical service result and format:

- JSON stdout byte-identical;
- exit code identical;
- no timestamp/color/terminal width/temp path effect;
- ordering exactly service ordering;
- text projection may vary only in nonsemantic prose under documented tests, not omit state.

Color is disabled in JSON and excluded from canonical behavior.

## Cancellation and signals

- map service cancellation to exit 130;
- no late result after cancellation;
- no partial JSON followed by cancellation JSON;
- process signal handling cannot mutate lower component/project/source state;
- no background daemon/work after process exit.

## Security

- no arbitrary shell/process command;
- no plugin loading from current directory/source repo;
- no source/repository hook execution;
- no implicit editor/client config discovery;
- no credential/token echo;
- arguments are data, not shell fragments;
- output escaping/encoding follows canonical serializer;
- source comments cannot affect CLI policy.

## Required tests

- status/check JSON exact bytes;
- current selector delegated to service and exact generation visible;
- exact selector preserved;
- clean/findings/partial/failure/cancelled exit mapping;
- status available/partial exit mapping;
- CLI syntax error exit 64 without service invocation;
- stdout contains only operation JSON in JSON mode;
- text result derives from same service records;
- raw counts/blockers remain visible in text;
- no lower framework dependency/import;
- no source path/glob/stdin/update/autofix flags;
- deferred/unknown command behavior frozen and non-successful;
- broken pipe/cancellation no double output;
- no private/source/Secret payload leak;
- temp directory/terminal width/color/time does not alter JSON bytes/exit.

## Definition of done

`apps/wow` E0 is complete when it can expose exact `status` and `check` service results in canonical JSON, provide a faithful text projection, map every service state/failure class to the frozen exit codes, and contains no domain/source/orchestration logic beyond service request construction.
