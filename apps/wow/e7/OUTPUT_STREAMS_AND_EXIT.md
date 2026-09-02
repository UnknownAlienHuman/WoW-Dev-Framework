# E7-A output streams, process exit, and protocol shutdown

**Status:** normative.

## One-shot CLI

Existing CLI commands retain their own output/exit contracts. `wow transport capabilities` supports:

```text
envelope-json  exact canonical service envelope + one LF
text           faithful bounded projection
```

stdout contains requested output only. stderr contains bounded redacted diagnostics. Output failure after a service call never causes a second call.

## LSP stdio

- stdin/stdout carry standard LSP framed messages only;
- no banner, ANSI, log line, panic text, or ordinary JSON outside framing;
- stderr carries bounded redacted operational logs;
- graceful sequence is `shutdown` request/response, then `exit` notification;
- normal graceful process exit is 0;
- `exit` without `shutdown`, framing failure, unrecoverable session failure, or stdout write failure is nonzero under the release profile;
- owner/domain invalidity is returned as an LSP response/result and does not crash the process.

## MCP stdio

- stdin/stdout carry individual MCP JSON-RPC messages only under revision `2025-11-25`;
- stderr carries bounded redacted logs;
- protocol/domain errors are represented according to the MCP profile;
- EOF/disconnect closes transport but is not cancellation of possible durable effects;
- normal initialized shutdown/EOF handling exits 0 when required close/reconciliation succeeds;
- malformed framing, unrecoverable session failure, or stdout failure exits nonzero.

## Local daemon

`wow daemon run` is a foreground long-running host. It writes no protocol bytes to ordinary stdout unless the selected administrative output mode requests one startup/shutdown envelope. Operational logs use stderr or an explicit protected log sink.

Exit classes:

```text
0    graceful requested shutdown and complete mandatory close
1    startup configuration/profile/endpoint failure
2    protocol/session failure requiring operator attention
3    internal owner/store/recovery failure
4    unresolved OutcomeUnknown or mandatory close/flush failure
130  process-level cancellation/interrupt after exact shutdown attempt
```

The daemon does not report graceful success when forcibly terminated or when unresolved mandatory effects remain.

`wow daemon status` and `wow daemon shutdown` are one-shot administrative clients. Their stdout is one canonical envelope/text result; they do not start a fallback embedded service if the daemon is unavailable.

## MCP local Streamable HTTP

HTTP status and MCP JSON-RPC status remain distinct. Invalid Origin or unauthenticated/cross-session requests are rejected before service dispatch. A dropped HTTP/SSE connection does not alter the service result or trigger recomputation.

## Progress and logs

Progress travels only through the active protocol:

```text
LSP: $/progress
MCP: progress notifications under negotiated profile
daemon: bounded progress notification frames
CLI: stderr progress only when explicitly enabled
```

Progress/log failure or coalescing cannot change final status. Sensitive source, overlays, credentials, session secrets, private endpoints, provider cursors, hidden holdout/cohort data, and raw owner handles are excluded.

## Panic and crash behavior

A panic boundary converts recoverable request-local failures into exact internal errors, marks possible effects for reconciliation, closes request resources, and keeps another isolated client/session usable when safe. Process-wide corruption or invariant failure terminates nonzero after best-effort synchronous journal/audit flush; it is never advertised as success.

## Determinism

Given identical exact service result and transport profile, machine response bytes are deterministic except protocol request IDs and explicitly operational fields. Locale, terminal width, editor name, connection ordering, progress timing, and log level do not alter semantic output.