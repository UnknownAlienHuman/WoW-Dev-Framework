# E7-A standalone LSP stdio server

**Status:** normative baseline LSP mapping.

## Invocation

```text
wow lsp serve --stdio --config <PATH>
```

The server hosts one `wow-service` runtime in-process for one stdio client. It does not connect to or auto-start the local daemon and does not listen on a socket/network endpoint.

The exact LSP protocol version, position-encoding profile, capability set, schema mappings and limits are pinned by the build compatibility manifest. A floating `latest` profile is invalid.

## Initialization

`initialize` is transport lifecycle, not a semantic project registration or service operation. The server validates:

```text
client protocol/capabilities
server compatibility manifest and operation registry
requested/available position encoding
workspace-folder/root URI as untrusted registration candidate
client work-done/diagnostic capabilities
host resource and exposure profile
```

The result advertises only the exact intersection whose service/owner implementations and mappings are frozen. Unsupported capabilities are absent, not advertised as stubs.

After `initialized`, the host opens one `EphemeralStdioSession`. If a workspace/root was supplied and the selected profile supports automatic registration prompting, the host constructs one explicit bounded `session_project_bind` request; service/project owners decide whether it is valid. The URI itself is not authority.

Baseline supports one bound project per LSP session. Multi-root project composition is not advertised unless a later exact profile defines it.

## Position encoding

Baseline position profile is UTF-16 code units because LSP editor ranges commonly use that representation. The exact negotiated encoding is recorded in every document/position request.

`wow-project` owns conversion to UTF-8 byte/source positions. The app validates numeric shape and bounds but never patches or resolves source coordinates itself.

An unsupported client/server position encoding intersection rejects initialization; no guessed conversion.

## Text synchronization

Advertised baseline:

```text
openClose = true
change = Full
save = supported
willSave = false
willSaveWaitUntil = false
```

Mappings:

```text
textDocument/didOpen   -> session_overlay_open
textDocument/didChange -> session_overlay_change
textDocument/didSave   -> session_overlay_save
textDocument/didClose  -> session_overlay_close
```

Each notification invokes exactly one service operation. Document URI, language ID, monotonic version, full text and exact protocol metadata are transported as data.

Optional incremental sync can be advertised only when the exact E7-A/project-owner capability manifest enables it. The app passes ordered content changes and versions unchanged to service; it does not apply them locally.

Save does not reindex or drop the overlay. Close reverts to the exact bound base generation, not floating disk/current state.

## Diagnostic model

Baseline prefers pull diagnostics:

```text
textDocument/diagnostic -> editor_diagnostics(document)
workspace/diagnostic    -> editor_diagnostics(workspace)
```

Result IDs bind exact session/document/project/profile/overlay generations. `previousResultId` is honored only if it identifies the same exact snapshot/profile and service validates reuse. A changed document/session generation produces a new result.

A zero diagnostic list is clean only when the service result carries complete relevant capability/coverage. Partial, conflict, truncated and `NotEvaluated` state is preserved through result data/related information and never silently displayed as clean.

Push diagnostics are disabled in the baseline profile to avoid stale unsolicited results. A later profile requires exact snapshot/version and invalidation rules.

## Completion and signature help

```text
textDocument/completion   -> editor_completion
textDocument/signatureHelp -> editor_signature_help
```

Completion items and signatures preserve service authority, deprecation/restriction state, exact source identities and replacement ranges. Search/provider rank is not translated to framework confidence.

Baseline does not advertise `completionItem/resolve` or lazy arbitrary callbacks. Items contain stable bounded data needed by the client.

## Hover and navigation

```text
textDocument/hover       -> editor_hover
textDocument/definition  -> editor_definition
textDocument/references  -> editor_references
```

Only service-tagged exact locations enter the normal LSP location fields. Possible/fuzzy/external candidates, when the selected profile permits their display, remain explicitly tagged in auxiliary data and are never encoded as exact definitions/references.

A zero result is returned as exact absence only with owner negative authority; otherwise auxiliary status states partial/`NotEvaluated` or the request returns an explicit service-derived error/result.

## Symbols

```text
textDocument/documentSymbol -> editor_document_symbols
workspace/symbol             -> editor_workspace_symbols
```

Document symbol hierarchy uses the exact requested/advertised parent axis. Baseline selects lexical hierarchy for the standard tree and may expose other axes only as explicit extension data. It never merges ownership/load/object/inheritance axes into one parent.

Workspace symbols preserve search lane/score/authority data in bounded extension fields. `workspaceSymbol/resolve` is not advertised in baseline.

## Call hierarchy

```text
textDocument/prepareCallHierarchy -> editor_prepare_call_hierarchy
callHierarchy/incomingCalls        -> editor_incoming_calls
callHierarchy/outgoingCalls        -> editor_outgoing_calls
```

Call hierarchy item data binds exact session snapshot and owner entity ID. Follow-up requests cannot switch generations. Event registration, hooks, callbacks, XML handlers, ownership and load relations remain distinct from calls and may appear only in explicitly typed auxiliary relation data.

## Code actions

```text
textDocument/codeAction -> editor_code_actions
```

The server returns:

- a protocol-native edit only for a `MechanicallySafeEditCandidate` with exact document version/base/session/source-digest guards;
- a disabled code action with reason for stale/inapplicable candidates;
- a plan-only action as nonapplying descriptive data.

The server does not advertise `executeCommand`, does not send `workspace/applyEdit`, does not save files, and does not run commands. The editor/user chooses whether to apply a returned edit. A subsequent stale version invalidates it.

## Unsupported baseline capabilities

Not advertised:

```text
rename
formatting and range formatting
semantic tokens
inlay hints
folding ranges
notebook documents
workspace file operations
executeCommand
server-initiated applyEdit
implicit file watching/project refresh
dynamic capability registration
```

## Cancellation and progress

`$/cancelRequest` maps to exact transport/service cancellation for the identified request. It does not prove no effect. Response loss and `OutcomeUnknown` remain visible through stable error/result data.

Work-done progress uses bounded `$/progress` notifications derived from nonsemantic service progress. Progress completion is not operation success. The final LSP response controls.

Partial-result streaming is enabled only for operations whose registry descriptor and LSP mapping define exact bounded chunk semantics. No source disclosure widening.

## Errors

The server maps closed transport/service errors to protocol error categories and places stable operation/session/status/recovery IDs in bounded `data` fields. It never reflects raw input, document text, secrets, private paths, owner handles or stack traces.

Unknown or unadvertised methods return method-not-supported. They never route to a generic service dispatcher.

## Shutdown

```text
shutdown -> stop semantic request admission, close session through service, return response
exit     -> terminate only after the shutdown/forced-exit profile is applied
```

Abrupt stdio loss records cancellation/session-close intent and reconciles possible effects; it is not automatically a clean shutdown. No background service work continues after process exit.

## Conformance

Required tests cover initialization negotiation, exact capability advertisement, UTF-16/multibyte positions, full and optional incremental sync, stale versions, diagnostics result IDs, every method mapping, exact-versus-Candidate locations, code-action guards, cancellation/progress, malformed framing/JSON, stdout purity, shutdown/abrupt disconnect, and canonical equivalence with direct service/CLI requests.