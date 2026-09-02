# `wow-service` E7-A LSP/MCP session and transport foundation

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e7-a/lsp-mcp-session-and-transport-foundation`

## Mission

Expose existing exact framework capabilities through one transport-independent session API that can be projected by thin LSP and MCP applications without moving protocol parsing, workspace mutation, owner algorithms, authorization, or model execution into domain crates.

```text
pinned protocol profile
+ authenticated/authorized client session
+ explicit workspace binding
+ exact published project/reference/graph generations
+ optional exact ephemeral document-overlay generation
-> bounded transport-independent service operation
-> exact owner acquisition and invocation
-> evidence/coverage/conflict-preserving result
-> cancellation/progress/retention/closure receipts
-> protocol-specific thin projection
```

E7-A defines session and protocol behavior only. It does not activate the Rust workspace, distribute releases, run an editor, call a model, edit source, or add CI.

## Canonical reading order

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`PROTOCOL_PROFILES_AND_HANDSHAKE.md`](PROTOCOL_PROFILES_AND_HANDSHAKE.md)
5. [`SESSION_WORKSPACE_AND_GENERATIONS.md`](SESSION_WORKSPACE_AND_GENERATIONS.md)
6. [`DOCUMENT_OVERLAYS.md`](DOCUMENT_OVERLAYS.md)
7. [`SERVICE_OPERATIONS.md`](SERVICE_OPERATIONS.md)
8. [`CANCELLATION_PROGRESS_AND_BACKPRESSURE.md`](CANCELLATION_PROGRESS_AND_BACKPRESSURE.md)
9. [`AUTHORIZATION_PRIVACY_AND_TRUST.md`](AUTHORIZATION_PRIVACY_AND_TRUST.md)
10. [`RESULTS_ERRORS_AND_TELEMETRY.md`](RESULTS_ERRORS_AND_TELEMETRY.md)
11. [`SECURITY_AND_BUDGETS.md`](SECURITY_AND_BUDGETS.md)
12. [`TEST_MATRIX.md`](TEST_MATRIX.md)
13. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
14. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
15. [`../../../apps/wow-lsp/e7/`](../../../apps/wow-lsp/e7/README.md)
16. [`../../../apps/wow-mcp/e7/`](../../../apps/wow-mcp/e7/README.md)

Official protocol specifications are external inputs:

- [Language Server Protocol specification](https://microsoft.github.io/language-server-protocol/)
- [Model Context Protocol specification](https://modelcontextprotocol.io/specification/)

Implementation freezes exact protocol versions, transports, message schemas, and compatibility vectors. A floating “latest” specification is not a durable profile.

## Direct dependency slice

E7-A service operations may use only the owner subset required by the operation:

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
wow-search
wow-context
wow-rules
wow-cbm (optional external Candidate lane only)
```

`wow-emmy` and `wow-recognizers` remain behind published project/graph facts for E7-A. `wow-annotations` remains a generated artifact dependency through its owner. Applications depend only on `wow-service`.

## Public operations

```text
protocol_status
protocol_profile_validate
session_initialize
session_capabilities
session_rebind_exact
session_shutdown
workspace_bind
workspace_status
document_open
document_change
document_save
document_close
document_snapshot_validate
analysis_diagnostics
analysis_hover
analysis_definition
analysis_references
analysis_symbols
analysis_code_actions
analysis_resolve_action
context_request
search_request
operation_cancel
operation_status
```

Protocol adapters map their methods/tools/resources to exactly one public service operation. Unsupported protocol features return typed capability-unavailable results; they are never approximated by arbitrary owner calls.

## Exact session view

A session binds:

```text
one exact ReferenceProfile/ReferenceGeneration
one exact published project/store/graph/analyzer generation set
zero or one exact Blizzard UI source generation set
zero or one explicit external Candidate provider generation/session
one exact protocol/authorization/privacy/budget profile set
zero or one immutable ephemeral document-overlay generation
```

A request never refreshes `current` mid-operation. Rebinding is an explicit service operation that creates a new `SessionViewSet` and invalidates incompatible continuations, diagnostics result IDs, overlays, and cached protocol results.

## Unsaved documents

Open or changed documents live in a session-local immutable overlay chain. They never overwrite the published project generation or source files.

```text
published base generation
+ exact document URI/source identity
+ monotonic protocol version
+ canonical full-content digest
+ validated ordered edits
-> DocumentOverlayGeneration
-> bounded ephemeral analyzer/project view
```

A save notification is an observation, not proof that disk content or the published project generation changed. Publication/reindex remains an owner operation outside the LSP/MCP adapter.

## LSP boundary

The LSP app handles JSON-RPC/LSP framing, initialization, capability negotiation, document synchronization, request/notification routing, cancellation, progress, and exact position conversion. It does not implement diagnostics, search, context, source mutation, or owner selection.

E7-A supports a reviewed subset centered on diagnostics, hover, definition, references, symbols, and advisory code-action records. Formatting, rename, execute-command, direct workspace edit application, arbitrary command execution, and editor-setting mutation are deferred or forbidden.

## MCP boundary

The MCP app exposes a fixed allow-listed tool/resource surface backed by exact service operations. It does not expose a generic `call_tool`, raw owner API, arbitrary file/URL read, sampling/model invocation, prompt-controlled authorization, or dynamic tools from source/provider data.

MCP results remain data. A tool result or client approval does not grant edit, shell, network, publication, or runtime authority.

## Completion gate

E7-A implementation is complete only when exact protocol profiles and official-spec vectors are frozen; session/generation rebinding is explicit; document overlays are monotonic and immutable; stale versions and cross-session continuations fail; LSP and MCP surfaces map 1:1 to service operations; cancellation, progress, backpressure, output framing, authorization, privacy, retention, and reverse closure pass under adversarial tests; all results preserve evidence/coverage/conflicts/`NotEvaluated`; no source edit/model/tool escape exists; and 1/2/N-worker execution yields identical semantic results and protocol bytes where the protocol profile requires byte determinism.
