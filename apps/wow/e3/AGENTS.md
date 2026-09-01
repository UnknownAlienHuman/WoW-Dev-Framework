# AGENTS.md — `apps/wow` E3-C

## Scope

Implement only the seven context CLI command projections, bounded explicit config/artifact input, service invocation, output modes, exit codes, and signal/broken-pipe handling.

## Before coding

1. Read repository/app/service E3-C contracts.
2. Freeze service request/result schemas and exact canonical bytes.
3. Freeze command grammar, root token encoding, selector/profile options, config schema, input limits, output modes, exit codes, and test vectors.
4. Verify app has exactly one framework dependency: `wow-service`.
5. Verify E0 status/check behavior remains unchanged.

## Request construction

- Preserve typed selectors; never resolve current.
- Decode exact root tokens mechanically; never search/guess.
- Pass profile aliases/IDs to service without resolving semantic targets.
- Read artifact bytes only for explicit validate/render input.
- Do not send host input/config paths in semantic requests.
- Reject unknown/duplicate/conflicting flags and excessive input before service invocation.

## Invocation

- Exactly one service call per parsed command.
- No automatic retry on current change, acquisition failure, continuation expiry, output error, or cancellation.
- Pass one cancellation token/source through the adapter.
- No hidden preflight call that changes selection semantics; `context status` is an explicit user operation.

## Output

### `envelope-json`

Write exact canonical service JSON plus one final LF. No other stdout bytes.

### `text`

Use service result fields only. Always show operation/status, exact resolved generations, artifact IDs, partial/truncated/NotEvaluated/conflict/omission counts, continuation availability, and validation state where relevant.

### `artifact`

Write exact returned validated artifact bytes unchanged. Require exactly one eligible artifact. Do not append LF, headings, status, or source boundary text.

## Exit codes

Use [`OUTPUT_EXIT_AND_STREAMS.md`](OUTPUT_EXIT_AND_STREAMS.md) exactly. Invalid context artifact is exit 1 only for `context validate`; service/internal failure remains 3/4 as classified.

## Security

- No source directory/file/glob/stdin scan; only explicit artifact input is allowed.
- No implicit config file/env/editor/WoW client/repository lookup.
- No shell/plugin/template execution.
- No lower crate imports.
- No source/private path/token in parser errors or text output.
- Treat returned source-bearing artifacts as opaque exact bytes in artifact mode.

## Completion report

```text
commands and exact flag grammar
service requests/results consumed
root token encode/decode vectors
explicit config/artifact input behavior
output bytes and exit mapping
stdout/stderr/broken-pipe/cancellation behavior
dependency/import audit
security tests and missing freezes
```
