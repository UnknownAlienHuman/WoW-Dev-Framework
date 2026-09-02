# E7-A LSP 3.18 method mapping

**Status:** normative.

Every supported semantic LSP method or notification maps to exactly one E7-A service operation. Pure protocol acknowledgements do not call domain owners.

| LSP method | Service operation | Notes |
|---|---|---|
| `initialize` | `frontend_session_open` | protocol, client, registry and capability negotiation |
| `shutdown` | `frontend_session_close` | drain/reconcile/close; response precedes `exit` |
| `workspace/didChangeConfiguration` | `frontend_session_configuration_change` | one strict closed `wow` settings object; semantic changes create new exact bindings |
| `workspace/didChangeWorkspaceFolders` | `frontend_workspace_folders_change` | atomically validates added and removed folders in one operation |
| `workspace/didChangeWatchedFiles` | `frontend_workspace_files_changed` | bounded hint batch only; owners reacquire exact bytes |
| `textDocument/didOpen` | `frontend_document_open` | full exact content/version |
| `textDocument/didChange` | `frontend_document_change` | strict incremental/full replacement |
| `textDocument/didSave` | `frontend_document_save` | save notification is not disk proof |
| `textDocument/didClose` | `frontend_document_close` | releases overlay ownership |
| `textDocument/diagnostic` | `frontend_document_diagnostics` | canonical pull diagnostics |
| `workspace/diagnostic` | `frontend_workspace_diagnostics` | bounded partial-result profile |
| `textDocument/hover` | `frontend_hover` | exact evidence and source binding |
| `textDocument/definition` | `frontend_definition` | Candidate state preserved |
| `textDocument/references` | `frontend_references` | coverage/negative-authority state preserved |
| `textDocument/documentSymbol` | `frontend_document_symbols` | exact overlay or saved generation |
| `workspace/symbol` | `frontend_workspace_symbols` | exact search lanes/explanation refs |
| `textDocument/completion` | `frontend_completion` | no automatic fuzzy/external insertion |
| `textDocument/signatureHelp` | `frontend_signature_help` | exact ReferenceView/project profile |
| `textDocument/codeAction` | `frontend_code_actions` | exact guarded edits or plan/disabled |
| `textDocument/prepareCallHierarchy` | `frontend_call_hierarchy_prepare` | exact graph evidence |
| `callHierarchy/incomingCalls` | `frontend_call_hierarchy_incoming` | coverage/conflicts explicit |
| `callHierarchy/outgoingCalls` | `frontend_call_hierarchy_outgoing` | coverage/conflicts explicit |
| `$/cancelRequest` | `frontend_operation_cancel` | targets exact operation ticket |

`initialized`, `exit`, work-done progress creation acknowledgements and protocol bookkeeping do not create semantic service operations.

## Push diagnostic compatibility

When negotiated, `textDocument/publishDiagnostics` is emitted from the exact retained result of `frontend_document_diagnostics`; it is not a second analysis. The notification includes the exact document version and stable result reference in `data` where the client permits.

## Workspace change atomicity

A single `workspace/didChangeWorkspaceFolders` notification can contain both additions and removals. `frontend_workspace_folders_change` validates the entire change set against one prior session/workspace state and either publishes one coherent successor or rejects it. It does not issue multiple independently visible service calls.

A watched-file notification is similarly one bounded hint-batch operation. It may schedule/recommend exact owner reacquisition within the same documented service operation, but the notification cannot be returned as proof that disk bytes changed or a project generation was successfully published.

## Method preconditions

All document requests require a ready session, registered workspace/document, exact overlay snapshot/client version, negotiated position encoding, advertised method capability, bounded request and cancellation token.

Workspace operations require exact session registration/profile guards. Stale document versions trigger resynchronization rather than analysis against guessed content.

## Unadvertised methods

Rename, formatting, on-type formatting, semantic tokens, inlay hints, folding, selection ranges, code lens, executeCommand, type hierarchy, notebook documents and mutating file operations are rejected as unsupported initially.

## Projection rules

- Convert positions only against exact overlay bytes/line index.
- Preserve diagnostic codes, owner source, evidence/result IDs and partial/`NotEvaluated` state.
- Use `data` or exact resource/result references when standard LSP fields cannot carry canonical meaning.
- Never expose a Candidate as an exact definition/reference/completion.
- Never produce a `WorkspaceEdit` without exact version/content/range/old-byte guards.
- Never ask the editor to run shell commands or modify settings.