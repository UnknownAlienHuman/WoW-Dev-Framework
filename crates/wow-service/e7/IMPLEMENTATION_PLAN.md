# E7-A implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — freeze prerequisites

- implement/freeze every E0–E6 capability that the initial frontend registry will advertise;
- pin exact LSP 3.18 and MCP 2025-11-25 schema/library adapter versions;
- freeze `wow-local-jsonrpc/1` framing and endpoint profiles;
- freeze service operation registry descriptors and request/result/error schemas;
- freeze platform path/position-encoding/session/privacy/license/security profiles;
- freeze canonical wire fixtures and checksums.

Do not advertise unavailable later capabilities to make clients look complete.

## Phase 1 — operation registry

Implement immutable descriptors, registry generation, implementation capability manifest, schema validation, exposure/effect/authorization classification, and deterministic capability projection.

## Phase 2 — session and workspace model

Implement exact session/client identity, workspace registration, platform path validation, consumer/privacy/license scoping, leases, close state, and no implicit discovery.

## Phase 3 — project-owned document overlays

Implement full open/change/save/close/snapshot owner seam, UTF-8 canonical bytes, negotiated UTF-8/UTF-16 position conversion, strict versions, immutable overlays, disk reconciliation, and privacy cleanup.

## Phase 4 — frontend service operations

Implement diagnostics, hover, definition, references, document/workspace symbols, completion, signature help, code actions, call hierarchy, cancel/get/reconcile using existing owners without duplicating algorithms.

## Phase 5 — one-shot CLI compatibility

Refactor existing CLI routing to consume the same operation registry where applicable while preserving exact existing command/output/exit contracts.

## Phase 6 — LSP 3.18 stdio

Implement lifecycle, framing, capability negotiation, incremental sync, pull diagnostics, negotiated push fallback, language method mappings, progress/cancellation, shutdown, and strict stdout/stderr behavior.

## Phase 7 — MCP 2025-11-25 stdio

Implement lifecycle, fixed read-only tools, exact resources, strict JSON schemas, structured output, logging/progress/cancellation, and no prompts/sampling/elicitation/tasks.

## Phase 8 — local daemon

Implement Windows named-pipe and Unix-domain-socket profiles, OS peer checks, bounded JSON-RPC framing, sessions, operation calls, response journal, reconnect, backpressure, and graceful shutdown.

Daemon may remain disabled in the first preview until both platform profiles pass.

## Phase 9 — optional local MCP Streamable HTTP

Implement only after stdio and daemon profiles pass. Require loopback binding, Origin validation, session authentication, protocol headers, bounded SSE replay, disconnect/noncancel semantics, and security tests. Keep disabled by default.

## Phase 10 — transport equivalence

For every operation available through multiple frontends, prove identical canonical service request/result semantics and explicit, lossless transport projections. Editor/model/terminal differences cannot change owner decisions.

## Phase 11 — resilience and isolation

Run cancellation/disconnect/reconnect/response-loss at every stage; malformed framing; workspace/path/encoding attacks; queue pressure; multi-client data isolation; crash/log redaction; session expiry; forced process termination and startup recovery.

## Phase 12 — client matrix

Validate at least:

```text
one reference LSP 3.18 client
one VS Code-compatible LSP client
one MCP 2025-11-25 stdio host
Windows named-pipe local daemon profile
Unix-domain-socket profile when claimed supported
```

A named client/version remains an exact test input and is not a permanent compatibility assumption.

## Phase 13 — measured limits

Freeze maximum frame/document/change/workspace/session/concurrency/progress/resource/result sizes and latency/memory targets using small, medium, and large addon fixtures.

## Phase 14 — freeze implementation evidence

Populate implementation commits, protocol libraries, registry/schema digests, platform/client fixtures, canonical wire bytes, benchmark thresholds, and all SHA-256 manifests. Only then mark E7-A implemented and permit E7-B release candidate packaging.

## Deferred

- public release publication and updater;
- remote hosted MCP/HTTP service;
- editor-specific source mutation UX;
- autonomous model sampling/elicitation/tasks;
- OS service installation;
- generic plugin/tool system;
- CI until real commands and release gates exist.