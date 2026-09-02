# E7-A allowed service-operation registry

**Status:** normative. The registry is immutable, reviewed, and exact-versioned.

## Purpose

Map protocol methods/tools to existing `wow-service` operations without exposing a generic dispatcher or owner internals.

```text
protocol method/tool
-> exact ProtocolOperationEntry
-> exact input schema and session/binding requirements
-> exactly one existing service operation
-> exact result/error mapping
```

## Registry entry

Each entry freezes:

```text
protocol kind and profile
method/tool name and version
request | notification
input/output schema IDs and digests
service operation name/version
allowed lifecycle/binding/document states
required negotiated capability
PureRead | SnapshotReadWithRetention | SessionBindingEffect | DurableDomainEffect | ProtocolLifecycleEffect
domain OperationId requirement
cancellation/progress/partial/continuation behavior
authority ceiling and mandatory nonclaims
privacy/source/credential/tool-permission profile
resource limits
protocol result/error mapping
canonical digest
```

## Registry ownership

The registry is repository-owned data validated by `wow-service`. Neither client, workspace, source file, provider, model, MCP server discovery, nor editor extension can add or modify entries.

No runtime reflection over public service methods.

## Initial common lifecycle entries

```text
protocol/profile.validate
protocol/session.initialize
protocol/session.get
protocol/session.bindWorkspace
protocol/session.rebind
protocol/request.cancel
protocol/request.reconcile
protocol/session.shutdown
protocol/session.exit
protocol/transcript.validate
```

Applications map these to their exact standard/custom protocol messages as defined by their pinned profile.

## Initial LSP operation projection

The initial LSP profile may expose only this reviewed subset when the delegated service operations are implemented/frozen:

```text
initialize
initialized notification
shutdown
exit notification
$/cancelRequest
workspace/didChangeWorkspaceFolders under exact profile
textDocument/didOpen
textDocument/didChange
textDocument/didSave
textDocument/didClose
textDocument/diagnostic
workspace/diagnostic
textDocument/hover
workspace/symbol
workspaceSymbol/resolve when exact profile supports it
custom wow/status
custom wow/context
custom wow/requestReconcile
```

### Intended service mappings

```text
textDocument/diagnostic
    -> existing exact `check` use case for admitted overlay binding

workspace/diagnostic
    -> existing exact project `check` use case

textDocument/hover
    -> exact position/entity resolution plus existing bounded context inspection

workspace/symbol
    -> E4 exact-generation search query returning Candidate-aware protocol items

workspaceSymbol/resolve
    -> exact selected result/candidate explanation; no implicit selection

wow/status
    -> status operation

wow/context
    -> existing exact context operation

wow/requestReconcile
    -> exact durable domain operation reconciliation
```

Where an exact position-to-entity owner seam is not implemented/frozen, hover/diagnostic mapping remains disabled/`NotEvaluated`; protocol code must not implement its own parser/resolver.

## Disabled LSP entries

```text
workspace/applyEdit
workspace/executeCommand
textDocument/rename
textDocument/codeAction
textDocument/formatting
textDocument/rangeFormatting
textDocument/onTypeFormatting
edit-producing completion or resolve
arbitrary workspace/configuration reads
dynamic register/unregister capability
unknown custom wow/* dispatch
```

No edit-producing result type is allowed in E7-A.

## Initial MCP tool projection

The initial MCP profile exposes a fixed `tools/list` snapshot and may allow this exact family when delegated operations are implemented/frozen:

```text
wow_status
wow_check
wow_context_build
wow_context_continue
wow_search_query
wow_search_continue
wow_search_explain
wow_search_select
wow_search_context
wow_external_query
wow_external_continue
wow_external_explain
wow_external_map
wow_external_select
wow_external_context
wow_operation_reconcile
```

Each tool has a closed JSON schema and one service operation mapping. Exact registry version is returned during/listed under the pinned MCP profile.

## Disabled MCP capabilities

```text
arbitrary tools/call name
provider MCP pass-through
resources/list/read/subscribe
prompts/list/get
sampling/model invocation
elicitation
roots discovery/mutation beyond exact session binding
server-to-client tool calls
dynamic tool registration
shell/SQL/script/plugin/source execution
edit/apply/publish/activate/release tools
```

A future package may activate a capability only with a new registry/profile and full privacy/permission/lifecycle tests.

## Candidate and authority projection

Registry mappings preserve owner ceilings:

- workspace symbol/search results remain Candidate where applicable;
- external results remain `semantic_candidate` / `Candidate`;
- explicit search/external selection is a control receipt, not evidence;
- context output retains evidence/coverage/conflicts/nonclaims;
- protocol success does not convert partial/blocked/`NotEvaluated`/`OutcomeUnknown` to success/proof.

## Tool permission

Registry entry contains a permission class. E7-A active entries are read/query/context/session operations only. No filesystem write, source edit, process/network tool, provider management, pack activation, or release permission is exposed.

Client capability or tool list presence does not grant additional permission.

## Schema compatibility

Input/output schemas are exact and closed. Unknown fields are rejected or explicitly preserved as unsupported according to the protocol entry. No generic JSON object reaches owner operations.

A service operation schema change requires:

- new entry version/digest;
- protocol capability/profile compatibility review;
- request/result/error/wire fixtures;
- a new session registry snapshot.

## One-call rule

After lifecycle/binding/input validation, one protocol entry invokes exactly one public service use case. It may not call a second service operation to guess, enrich, select, repair, or fall back.

Explicit multi-stage workflows remain separate client calls with exact returned IDs.

## Status representation

Completed domain requests return structured service envelopes. Protocol items may include bounded display fields plus exact opaque `data`/ID fields under the protocol schema, but display text never replaces authority/evidence records.

## Registry validation

Reject:

- duplicate/case-colliding method/tool names under the profile;
- missing/unimplemented service operation;
- mutable/dynamic entry source;
- arbitrary raw owner handle;
- missing schema/authority/privacy/cancellation/effect fields;
- edit/command/provider/release effect in E7-A;
- mapping one method to multiple hidden operations;
- alias without an exact entry;
- method/tool name derived from workspace/source/provider text.

## Tests

- exact registry and capability projection;
- all methods/tools map to one service operation;
- unknown/case-folded/prefix tool;
- client/source attempts dynamic registration;
- disabled edit/command/sampling/resources/prompts;
- arbitrary provider tool passthrough;
- service schema/operation version mismatch;
- Candidate rendered proven;
- one-call mutation and hidden fallback;
- shuffled registry source order produces same canonical registry.
