# AGENTS.md — `wow-service` E7-A

## Scope

Implement transport-neutral session, operation-registry, workspace/document-overlay orchestration, cancellation/reconciliation and exact frontend projections only. Do not reproduce analyzer, project, reference, graph, rules, search, context, external-provider, calibration, publication or storage algorithms.

## Required reading

1. repository, crate, service, application, launch-gate and E7-A package instructions;
2. the complete owner contract for every advertised operation;
3. E7-A document-overlay/store seams;
4. pinned official LSP 3.18 and MCP 2025-11-25 specifications;
5. current external WoW engineering KB routes before any patch-sensitive profile claim.

## Registry discipline

- Registry entries are immutable reviewed data bound to exact service request/result/error schemas.
- No reflection, plugin discovery, arbitrary operation name, raw RPC forwarding or generic `call_service` entry.
- Advertise a capability only when the exact owner implementation and required profile are available.
- Transport aliases never create a second semantic operation.
- Effect class, authorization, privacy/license, budgets, cancellation, continuation and output profiles remain explicit.

## Session discipline

- Session IDs, client identities, protocol versions, capabilities, workspace registrations, overlays, leases and operation tickets are exact and isolated.
- Transport request IDs are not durable operation IDs.
- A session cannot inherit another client's workspace, overlays, authorization, output or continuation.
- Shutdown/close is explicit and synchronous; no unowned background work.

## Workspace and document discipline

- Workspace roots are explicitly registered and normalized under a platform path profile.
- Never search parents, inspect Git/editor/WoW state or auto-register repositories.
- Overlays bind URI, workspace, language, version, content digest, position encoding and exact prior snapshot.
- Reject stale/out-of-order versions and invalid ranges; require full resynchronization instead of guessing.
- Unsaved content is private session data and is not persisted by default.

## LSP discipline

- Pin LSP 3.18.
- Support only listed methods and advertise only implemented capabilities.
- Prefer pull diagnostics; push is a negotiated projection of the same result.
- Preserve exact document version/result ID.
- No rename/formatting/semantic-token/inlay-hint capability without a later owner contract.
- Code actions return exact version-guarded edits only from authorized service results; otherwise plans/disabled actions.

## MCP discipline

- Pin revision 2025-11-25.
- Default transport is stdio; local Streamable HTTP is explicit and disabled by default.
- Fixed tools/resources only; no prompts, sampling, elicitation, tasks, arbitrary roots or generic proxy.
- Structured output preserves the exact service result; text is a faithful compatibility projection.
- The default tool set is non-source-mutating, not uniformly read-only: each descriptor declares its actual `PureRead` or `DurableLocalEffect` class and correct MCP annotations. User-source, provider, calibration, publication, activation, release and external effects are absent.
- Tool annotations are descriptive hints, never enforcement or authorization.

## Lifecycle discipline

- Disconnect is not cancellation.
- Progress is bounded/non-authoritative and never substitutes for a final receipt.
- `OutcomeUnknown` remains unsafe to retry and is reconciled through the exact durable operation ID.
- Backpressure may coalesce/drop progress but never the final result, error, cancellation or state transition.
- Required resources close in reverse order before success.

## Security

No shell, arbitrary process launch, raw SQL/MCP/RPC, plugin/script/model prompt, provider database, unrestricted source, secret material, remote listener or cross-client data enters the default path.

## Completion report

```text
transport/profile/protocol version
registry entry and service operation
session/workspace/document/overlay identities
request and durable operation IDs
owner capabilities and advertised frontend capability
actual effect class and tool annotations
progress/cancellation/reconnect/backpressure result
final service/transport status and exact bytes
privacy/license/isolation/close state
all tests and every skipped/NotEvaluated gate
```