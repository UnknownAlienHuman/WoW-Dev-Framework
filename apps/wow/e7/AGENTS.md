# AGENTS.md — `apps/wow` E7-A

## Dependency

```text
apps/wow -> wow-service
```

No direct lower-crate, provider SDK, editor semantic owner, database or source-reader dependency.

## Host-mode discipline

- Select one explicit mode.
- Use only frozen protocol profiles and registry generations.
- Unknown flags/config fields/methods/tools/resources fail.
- No implicit daemon, provider/tool discovery, workspace root, editor state, project profile or current generation.
- Exactly one service operation per semantic request except protocol acknowledgements.

## LSP discipline

- stdio initially;
- stdout contains LSP frames only; stderr is bounded redacted logging;
- advertise only exact negotiated implemented capabilities;
- preserve document versions/position encoding;
- never apply edits automatically or mutate editor settings;
- never advertise unsupported rename/formatting/semantic-token/inlay-hint/generic commands.

## MCP discipline

- pin revision 2025-11-25;
- stdio default; local Streamable HTTP explicit and disabled by default;
- fixed non-source-mutating tool registry and exact resources;
- each tool preserves its actual pure-read or durable-analysis effect class and annotations;
- no user-source/provider/calibration/publication/activation/release/external mutation in default tools;
- no prompts, sampling, elicitation, tasks, arbitrary roots, generic tools or provider proxy;
- `structuredContent` preserves exact status/evidence; text cannot override it;
- model invocation and tool annotations are not authorization.

## Daemon discipline

- foreground process;
- current-user Windows named pipe or Unix-domain socket;
- no TCP/remote listener;
- bounded framing/queues/sessions;
- peer identity does not authorize effects;
- disconnect does not cancel;
- graceful shutdown drains/reconciles/closes synchronously.

## Security

Never expose source, overlays, private paths, credentials, session secrets, provider cursors, hidden holdout/cohort data or raw owner handles beyond exact service policy. No shell/process launch/raw SQL/RPC/plugin/script/model execution/arbitrary file access.

## Completion report

```text
mode/protocol/profile/registry generation
service operation mapping and actual effect class
session/workspace/document/overlay IDs
wire request and durable operation IDs
input/output bytes and limits
progress/cancellation/reconnect/backpressure
exit/close state
privacy/license/isolation/redaction
all tests and skipped/NotEvaluated gates
```