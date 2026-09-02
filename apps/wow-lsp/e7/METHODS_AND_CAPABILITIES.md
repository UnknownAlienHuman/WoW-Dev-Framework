# E7-A LSP methods and capability mapping

**Status:** normative. The exact official specification revision and compatibility profile freeze before implementation.

## Lifecycle

| LSP method/notification | Service mapping | Notes |
|---|---|---|
| `initialize` | `session_initialize` | Includes exact protocol profile, authorization context reference, and optional explicit workspace bind. |
| `initialized` | application state transition only | No second service initialization. |
| `shutdown` | `session_shutdown` | One service call; response precedes protocol exit. |
| `exit` | application transport close | No second shutdown effect. |
| `$/cancelRequest` | `operation_cancel` | Exact request-to-operation mapping required. |

## Document synchronization

| LSP notification | Service mapping | Notes |
|---|---|---|
| `textDocument/didOpen` | `document_open` | Full text, explicit version, exact URI/language profile. |
| `textDocument/didChange` | `document_change` | Monotonic version and validated ordered changes. |
| `textDocument/didSave` | `document_save` | Observation only; never disk/publication proof. |
| `textDocument/didClose` | `document_close` | Removes session overlay; never saves. |

The active profile freezes synchronization kind, save-text behavior, position encoding, and compatibility with clients that omit or misuse versions. Unsupported behavior is rejected or requires full resynchronization; it is not guessed.

## Diagnostics

| LSP method | Service mapping | Notes |
|---|---|---|
| `textDocument/diagnostic` | `analysis_diagnostics` | Canonical exact result with previous-result guard. |
| `workspace/diagnostic` | `analysis_diagnostics` | Available only under an exact bounded workspace profile. |

Pull diagnostics are canonical. A push-diagnostic compatibility projection, if enabled, is derived from the same immutable `DiagnosticResultSet` and does not create another analysis result.

Diagnostic `resultId` maps to an exact retained service result ID/profile and cannot be reused across session views, overlays, projects, or rule/reference generations.

## Navigation and information

| LSP method | Service mapping |
|---|---|
| `textDocument/hover` | `analysis_hover` |
| `textDocument/definition` | `analysis_definition` |
| `textDocument/references` | `analysis_references` |
| `textDocument/documentSymbol` | `analysis_symbols` with exact document scope |
| `workspace/symbol` | `analysis_symbols` with exact workspace/search scope |

Multiple definitions/references remain multiple. Candidate workspace-symbol search remains labeled Candidate. The adapter never picks top-1.

## Advisory code actions

| LSP method | Service mapping |
|---|---|
| `textDocument/codeAction` | `analysis_code_actions` |
| `codeAction/resolve` | `analysis_resolve_action` |

E7-A code actions:

- bind exact finding/action/session/overlay IDs;
- expose title/kind/diagnostic references and typed advisory data;
- preserve remediation tier, blockers, required validation, and `NotEvaluated`;
- contain no `WorkspaceEdit` and no executable command;
- do not apply source changes.

A client cannot request edit inclusion through a capability flag.

## Workspace/status extension

Standard LSP methods remain standard. Framework-specific status/context/search operations are not invented as arbitrary LSP methods in the initial profile. They are exposed through MCP/CLI, or through a later explicitly namespaced and versioned LSP extension profile.

Initialization options can select only repository-owned named profiles and explicit workspace bindings. They cannot provide executable rules or raw owner requests.

## Progress and partial results

Work-done progress and partial-result tokens are validated according to the frozen LSP profile. The adapter maps service progress/result pages mechanically and preserves cumulative budgets and exact result identities.

A client-provided token is transport metadata, not an artifact ID or authorization grant.

## Capability advertisement

The server advertises only the intersection of:

```text
implemented LSP adapter methods
frozen protocol profile
available wow-service operations
bound owner capabilities
authorization/privacy policy
session/workspace state
```

Dynamic registration is not used in the initial profile. Client declarations cannot make an unimplemented/denied capability available.

## Unsupported methods

Initial profile returns the exact protocol-defined unsupported behavior for:

```text
completion and resolve
signature help
implementation/type definition/declaration (unless separately activated)
rename/prepareRename
formatting/range/on-type formatting
executeCommand
workspace/applyEdit
workspace file operations
semantic tokens
inlay hints
code lens
call hierarchy/type hierarchy
inline values/completion
notebook documents
```

No unsupported method falls back to raw source, fuzzy search, arbitrary service calls, shell, model, or editor mutation.

## Result projection rules

- Source positions convert from exact owner coordinates to negotiated LSP encoding.
- Evidence/coverage/conflicts/nonclaims are retained in stable `data` fields or explicit related metadata under the profile.
- Partial/truncated reference/symbol/diagnostic results are not rendered as complete.
- Hover Markdown/plaintext uses static templates and source-data boundaries.
- Protocol severity/tags map from owner findings under a frozen loss profile; no severity inflation.
- URI locations are authorized stable protocol projections, not leaked private paths.

## One-call rule

Each supported request invokes exactly one service operation. Notifications invoke at most their declared one operation. The adapter does not compose search+selection+context, validate+apply, or diagnostic+fix workflows locally.
