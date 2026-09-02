# E7-A Language Server Protocol 3.18 profile

**Status:** normative supported profile; implementation has not started.

Official protocol reference: <https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/>

## Transport and lifecycle

The initial LSP profile uses stdio with standard LSP message framing. `stdout` contains protocol messages only; bounded redacted logs use `stderr`.

Supported lifecycle messages:

```text
initialize
initialized
shutdown
exit
$/cancelRequest
$/progress
```

`initialize` negotiates LSP 3.18, position encoding, workspace folders, diagnostic support, work-done/partial-result progress, client implementation, and the exact server capability projection. `shutdown` drains the frontend session; `exit` closes transport. Exit without shutdown is abnormal and triggers synchronous cleanup/recovery.

## Position encodings

Supported server encodings:

```text
utf-16  mandatory profile
utf-8   supported when the client advertises it
```

The negotiated encoding is fixed for the session. Every request binds the exact document version/overlay snapshot. Invalid ranges or stale versions never fall back to another encoding.

## Text synchronization

```text
textDocument/didOpen
textDocument/didChange   incremental; full-document change allowed as exact replacement
textDocument/didSave
textDocument/didClose
workspace/didChangeWorkspaceFolders
workspace/didChangeWatchedFiles   hint only
```

Every notification maps to one E7-A service operation except pure protocol acknowledgement. Incremental changes require strictly advancing versions and exact prior overlay state. Watched-file notifications do not establish bytes or successful reindexing.

## Diagnostics

Primary projection:

```text
textDocument/diagnostic
workspace/diagnostic
```

Use exact result IDs and unchanged/full reports where supported. A negotiated compatibility profile may also emit `textDocument/publishDiagnostics`, but it must project the same exact owner result and document version. Push delivery is not canonical completion.

Diagnostics preserve severity, code, source owner, exact range, related evidence, capability/coverage/conflict/`NotEvaluated` state, and remediation class. Downstream symptoms are not duplicated as root causes without explicit relationship.

## Language features

Advertise only when the exact underlying service operation is implemented for the registered workspace/profile:

```text
textDocument/hover
textDocument/definition
textDocument/references
textDocument/documentSymbol
workspace/symbol
textDocument/completion
textDocument/signatureHelp
textDocument/codeAction
textDocument/prepareCallHierarchy
callHierarchy/incomingCalls
callHierarchy/outgoingCalls
```

### Feature mapping principles

- hover: exact reference/project/context evidence, bounded and source-linked;
- definition/references: exact owner/graph evidence; Candidate lanes remain labeled;
- symbols: exact project generation or overlay-aware local symbols, with scope declared;
- workspace symbol: E4 search lanes and explanations, no implicit best truth;
- completion/signature help: exact ReferenceView/project facts first; fuzzy/external candidates remain Candidate and are not inserted automatically;
- code actions: exact safe version-guarded edits only when an existing remediation contract authorizes them; otherwise plans/disabled actions;
- call hierarchy: exact graph call evidence with coverage/conflicts; no complete negative without authority.

## Deliberately unadvertised in the initial profile

```text
rename
formatting/on-type formatting
semantic tokens
inlay hints
folding ranges
selection ranges
code lens
executeCommand as a generic operation surface
type hierarchy
notebook documents
file operations that mutate source
```

A future profile can add one only after an owned service contract, exact fixtures, compatibility analysis, and no editor-specific semantic fork.

## Workspace configuration

Only a closed `wow` settings object is accepted. Configuration updates create a new validated session/workspace profile or are rejected; they cannot silently change project/reference generation, privacy/license scope, provider, or authorization.

The server never writes editor settings or asks the editor to execute a shell command.

## Code actions and edits

A `WorkspaceEdit` must bind exact document URI/version/content digest, expected old range bytes, replacement bytes, edit ordering, and owner authorization. If any guard is unavailable, return a plan or disabled action with reason. The server never calls `workspace/applyEdit` autonomously in the initial profile.

## Partial and stale states

Unsaved overlays can make published graph/search/context evidence stale. Results must state overlay-local, overlay-plus-published-partial, or saved-generation-only coverage. The LSP projection never hides this in a tooltip footer or warning.

## Cancellation, progress, and errors

`$/cancelRequest` maps to the exact operation ticket. Cancellation is best effort at the transport level but service effect state remains authoritative. Progress is bounded and nonauthoritative. Protocol errors, invalid params, domain invalidity, owner failures, and `OutcomeUnknown` remain distinct.

## Security

Document text, workspace paths, configuration, client labels, snippets, and diagnostics are untrusted data. They cannot create tools, commands, profiles, authorization, or provider requests. No source body is logged by default.