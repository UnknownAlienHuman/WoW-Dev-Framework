# AGENTS.md — `apps/wow` E7-A

## Dependency

```text
apps/wow -> wow-service
```

No direct lower-crate, provider SDK, editor SDK semantic owner, database, or source-reader dependency.

## Host-mode discipline

- Select exactly one explicit mode.
- Use only frozen protocol profiles and operation registry generations.
- Unknown flags/config fields/methods/tools/resources fail.
- No implicit daemon connection, provider/tool discovery, workspace root, editor state, project profile, or current generation.
- Exactly one service operation per valid command/request/tool call except protocol-only lifecycle acknowledgements.

## LSP discipline

- stdio framing only in the initial profile;
- stdout contains LSP frames only; stderr is bounded redacted logging;
- advertise only service capabilities returned by exact negotiation;
- preserve document versions and negotiated position encoding;
- never apply source edits automatically or mutate editor settings;
- never advertise rename/formatting/semantic tokens/inlay hints or generic commands without a later contract.

## MCP discipline

- pin revision `2025-11-25`;
- stdio is default; local Streamable HTTP is explicit and disabled by default;
- fixed read-only tool/resource registry only;
- no prompts, sampling, elicitation, tasks, generic roots, arbitrary tools, or provider proxy;
- `structuredContent` preserves exact service status/evidence; text cannot override it;
- a model tool call is not user authorization.

## Daemon discipline

- foreground process;
- Windows named pipe or Unix-domain socket under exact current-user policy;
- no TCP or remote listener;
- bounded framing/queues/sessions;
- peer identity does not authorize service effects;
- disconnect does not cancel;
- graceful shutdown drains/reconciles and closes synchronously.

## Security

Never expose source, unsaved overlays, private paths, credentials, session secrets, provider cursors, hidden holdout/cohort data, or raw owner handles beyond exact service output policy. No shell, process launch, raw SQL/RPC, plugin/script/model execution, or arbitrary file read/write.

## Completion report

```text
mode/protocol/profile/registry generation
service operation mapping
session/workspace/document/overlay IDs
wire request and durable operation IDs
input/output bytes and limits
progress/cancellation/reconnect/backpressure
exit/close state
privacy/license/isolation/redaction
all tests and skipped/NotEvaluated gates
```