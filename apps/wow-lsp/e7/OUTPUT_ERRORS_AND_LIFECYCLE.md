# E7-A LSP output, errors, cancellation, progress, and lifecycle

**Status:** normative.

## Output stream

Protocol stdout contains only valid LSP/JSON-RPC frames. Logs and bounded diagnostics use stderr or a configured telemetry sink. Locale, terminal width, color, and client name do not alter protocol bytes.

## Result mapping

The adapter maps one exact service result to one LSP result under the frozen projection profile:

- diagnostics retain finding identity, severity mapping, evidence/coverage/conflict state, and result ID;
- hover uses typed static content and source-data boundaries;
- definition/references/symbols preserve cardinality, ordering, partial/truncated state, and exact locations;
- code actions are advisory and contain no edit/command;
- unsupported service fields produce explicit projection-loss records and cannot remove mandatory trust metadata.

A service `Complete` status means operation completion, not “no diagnostics.” Empty findings under partial coverage are not a clean authoritative result.

## Error mapping

Protocol-layer errors are used for malformed framing/JSON/request/method/initialization/version/session issues. Valid domain outcomes—including findings, multiple definitions, Candidate results, authorization denial, partial coverage, conflicts, `NotEvaluated`, and truncation—remain normal typed results when the LSP profile supports structured data.

The stable service error code is retained in a bounded `data` field where permitted. Raw source, private paths, credentials, owner handles, and stack traces are not included.

## Cancellation

`$/cancelRequest` maps the exact LSP request ID to one service operation and invokes `operation_cancel` once. The adapter reports actual state:

```text
cancelled before dispatch
cancelled at owner safe point
completed before cancellation
effect committed
OutcomeUnknown
unsupported/stale cancellation
```

No cancellation result is guessed. No automatic second cancel or retry.

## Progress and partial results

Client-provided progress tokens are validated and mapped to service progress streams. Progress is bounded and nonsemantic. Partial result chunks retain exact page/continuation IDs, stable ordering, cumulative budgets, and previous omissions.

A final response remains mandatory unless transport is lost. Progress completion is not a substitute for a response.

## Initialization

- one initialize request per process/session;
- exact profile/version/capability negotiation;
- no normal request before initialization completes;
- initialization failure closes acquired service/session resources;
- initialized notification changes adapter state only and does not call session initialization twice.

## Shutdown and exit

- `shutdown` invokes `session_shutdown` once and returns a response after mandatory closure;
- after shutdown, normal requests are rejected;
- `exit` closes transport and process state without a second shutdown effect;
- exit before shutdown follows the exact profile’s abnormal-exit behavior;
- abrupt EOF/broken pipe invokes bounded disconnect handling and never saves documents or repeats effects.

## Server-initiated messages

The initial profile emits only protocol messages required for progress, diagnostic compatibility, or capability/lifecycle behavior. It does not request arbitrary client commands, workspace edits, configuration discovery, registration, or source changes.

## Stale responses

A request captures exact `SessionViewSetId` and overlay generation. Even if the session rebinds later, its result is either returned with original exact identity or cancelled under policy. It is never relabeled as the new view.

Clients can discard stale responses based on document version/result data, but server semantic identity remains accurate.

## Broken pipe/output failure

- stop writing;
- do not invoke service again;
- record projection/output failure;
- initiate session disconnect cancellation/reconciliation/closure;
- retain exact service artifacts/effects under real state;
- do not report successful delivery.

## Exit behavior

Process exit codes are operational and profile-specific. They do not replace JSON-RPC/LSP errors. Clean shutdown/exit differs from initialization failure, protocol violation, authorization failure, owner failure, output failure, and internal panic boundary.

## Panic boundary

A panic/internal fault:

- is caught at the application boundary where possible;
- emits no malformed mixed protocol/log output;
- cancels/reconciles active operations;
- closes resources;
- returns/terminates according to the frozen platform profile;
- does not expose source/credentials/stack data by default;
- does not restart or retry automatically.
