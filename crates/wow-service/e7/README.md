# `wow-service` E7-A frontend session and transport contract

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `wow-service/e7-a/frontend-session-operation-registry`

## Mission

Expose existing `wow-service` capabilities through one exact session/operation registry projected into the command line, a local daemon protocol, Language Server Protocol 3.18, and Model Context Protocol revision 2025-11-25 without moving domain logic into a transport.

```text
exact transport profile
+ exact client/session identity
+ explicit workspace/project/profile registration
+ exact document overlay or retained project generation
+ one operation registry entry
+ one strict request
-> negotiate protocol and service capabilities
-> open an isolated frontend session
-> validate workspace/document/version/position bindings
-> invoke exactly one service operation
-> stream bounded nonauthoritative progress when supported
-> return one exact service result envelope
-> retain/reconcile durable effects when required
-> close session/resources explicitly
```

## Pinned protocol profiles

```text
local daemon protocol: wow-local-jsonrpc/1
LSP profile:            3.18
MCP profile:            2025-11-25
```

These are documentation pins, not implementation claims. A later protocol revision creates a new compatibility profile and fixture set.

## Public service operations

```text
frontend_status
frontend_capabilities
frontend_session_open
frontend_session_get
frontend_session_configuration_change
frontend_session_close
frontend_workspace_register
frontend_workspace_get
frontend_workspace_folders_change
frontend_workspace_files_changed
frontend_workspace_unregister
frontend_document_open
frontend_document_change
frontend_document_save
frontend_document_close
frontend_document_snapshot_get
frontend_document_diagnostics
frontend_workspace_diagnostics
frontend_hover
frontend_definition
frontend_references
frontend_document_symbols
frontend_workspace_symbols
frontend_completion
frontend_signature_help
frontend_code_actions
frontend_call_hierarchy_prepare
frontend_call_hierarchy_incoming
frontend_call_hierarchy_outgoing
frontend_operation_cancel
frontend_operation_get
frontend_operation_reconcile
```

`frontend_workspace_folders_change` atomically validates an LSP folder-add/remove notification in one service operation. `frontend_workspace_files_changed` records bounded file-watch hints and delegates exact reacquisition to project owners; the notification itself never establishes bytes or successful reindexing.

The registry may expose an existing E0–E6 service operation directly when one transport request maps one-to-one to that exact operation and the transport profile authorizes it. It never creates a generic `call_service(name, json)` escape hatch.

## Active dependency slice

The package coordinates only implemented capabilities required by a requested registry entry. Its maximum reviewed slice is:

```text
wow-core
wow-store
wow-reference
wow-emmy
wow-project
wow-graph
wow-rules
wow-search
wow-context
wow-cbm
```

E5 effecting operations remain under their own authorization profiles and are absent from default LSP/MCP profiles.

## Frontend modes

```text
wow CLI             one-shot command transport
wow daemon          explicit foreground local multi-client host
wow lsp             LSP 3.18 over stdio
wow mcp             MCP 2025-11-25 over stdio by default
wow mcp --http      explicit local-only Streamable HTTP profile, disabled by default
```

One binary may host all modes. No extra framework crate is introduced merely to share transport code.

## Capability discipline

A frontend advertises only registry entries whose owner implementations, profiles, fixtures and exact service schemas are available. Documentation-only or `NotEvaluated` operations are not advertised as working. Negotiation can narrow but cannot add an unreviewed operation.

## Workspace and overlay discipline

Workspace roots are explicit untrusted inputs. There is no upward directory search, Git-root inference, addon discovery, WoW installation scan or editor-state fallback. Unsaved documents become exact versioned overlay snapshots owned by `wow-project`; they never mutate the underlying generation in place.

Configuration changes are validated as one explicit session operation and create new exact configuration/workspace bindings when semantic state changes. They never mutate a profile silently.

## Authority and output discipline

Transport lifecycle, progress, editor position conversion, MCP metadata and daemon session state cannot raise evidence confidence, create negative authority, authorize an effect or hide owner coverage/conflicts. The final machine result is the canonical service envelope or a lossless transport projection.

## Default exposure

- LSP advertises implemented read/analysis capabilities only.
- MCP exposes a fixed non-source-mutating analysis tool set and exact resources. Every tool keeps its real `PureRead` or `DurableLocalEffect` class; no blanket read-only annotation is allowed.
- Operations that mutate user source, provider state, calibration/publication state, activation, release state or external systems are absent from the default MCP profile.
- MCP sampling, elicitation, prompts and task-augmented execution are unsupported initially.
- Remote network listeners are disabled by default.

## Deferred to E7-B

- reproducible release builds and binary packaging;
- public release assets, checksums, attestations, SBOM and signatures;
- install/update/rollback/retirement and support policy;
- editor package distribution and public compatibility matrix;
- release CI after executable commands exist.

## Completion gate

E7-A implementation is complete only when the operation registry, session/workspace/overlay model, local daemon, LSP 3.18, MCP 2025-11-25, exact progress/cancellation/reconnect/backpressure, multi-client isolation, privacy/license/security, canonical transport bytes and every nondeferred fixture pass with real E0–E6 service implementations.