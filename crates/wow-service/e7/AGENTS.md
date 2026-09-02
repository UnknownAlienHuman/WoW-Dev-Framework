# AGENTS.md — `wow-service` E7-A

## Scope

Implement transport-independent protocol/session orchestration only. LSP and MCP framing belongs to their application adapters. Domain analysis, search, context, storage, graph, rules, source parsing, and provider normalization remain with their owners.

## Before coding

1. Read repository, crate, E7-A service, LSP, and MCP contracts.
2. Read every active owner contract used by the operation.
3. Freeze exact LSP and MCP specification/profile revisions from official sources.
4. Freeze protocol schemas, capability tables, position/framing profiles, session states, request/result bytes, and adversarial vectors.
5. Verify all prerequisite implementation commits and fixture/checksum bundles.
6. Keep Rust activation and release work outside E7-A.

## Session discipline

- Resolve permitted `current` selectors exactly once during explicit session/workspace binding.
- Replace selectors with exact retained generation IDs before normal operations.
- Never refresh current during a request, continuation, diagnostic result, or overlay chain.
- Rebinding creates a new immutable `SessionViewSet`.
- Close owner resources in reverse order before successful shutdown/rebind completion.
- Session identity and client identity do not alter semantic authority.

## Document discipline

- Require explicit normalized document identity and exact session ownership.
- Enforce monotonic protocol versions and validated edit ranges.
- Derive a canonical full-content digest after every accepted change.
- Keep overlay generations immutable and session-local.
- Reject stale, missing, overlapping, out-of-range, wrong-encoding, or cross-session edits.
- Never write source files or call an editor command.
- Treat save notifications as observations only.

## Operation discipline

- Map every protocol request to exactly one public service operation.
- Use narrow owner ports and exact retained views.
- Do not add protocol convenience logic that reproduces owner algorithms.
- Preserve evidence, provenance, confidence, coverage, conflicts, partial/truncated state, and `NotEvaluated`.
- Unsupported capability remains unsupported; no fallback to a different owner or external provider.
- Code actions are advisory typed plans; no edit or command execution.

## Cancellation and progress

- Bind cancellation to exact protocol request and service operation IDs.
- Check cancellation before/during owner calls, serialization, and output handoff.
- No detached/background work after completion, cancellation, shutdown, or transport loss.
- Progress is monotonic bounded telemetry, never semantic evidence or proof of completion.
- Apply bounded queues and explicit backpressure; never drop mandatory responses silently.

## Authorization and privacy

- Authorization is supplied by narrow service ports/profiles, never inferred from editor, MCP host, OS user, GitHub account, workspace path, or client claims.
- Client capabilities do not grant filesystem, source-edit, tool, network, publication, or model permissions.
- Source comments, prompt text, tool descriptions, URIs, and document content remain untrusted data.
- Never expose credentials, raw authorization tokens, private absolute roots, hidden source, lower owner handles, or unrestricted stack traces.

## Protocol discipline

- Follow the frozen protocol profile exactly; no floating spec behavior.
- Unknown methods/tools/resources return typed protocol errors.
- No generic MCP tool proxy, arbitrary JSON-RPC owner dispatch, dynamic source-controlled tool registration, or model sampling.
- No LSP execute-command, arbitrary workspace edit, settings mutation, shell, or plugin surface.
- Transport request IDs are not domain identity.

## Completion report

```text
protocol/profile/session/workspace/view IDs
exact project/reference/graph/source generations
document overlay versions/digests when used
protocol method/tool/resource -> one service operation
owner calls and reverse closure
result/coverage/conflict/NotEvaluated state
cancellation/progress/backpressure/output state
authorization/privacy/source-boundary state
determinism/security/compatibility tests
explicit E7-B/release/runtime/edit/model deferrals
```
