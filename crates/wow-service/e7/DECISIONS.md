# E7-A protocol and session decisions

**Status:** normative.

## S7A-001 — Protocol specifications are exact external profiles

LSP and MCP versions, transports, framing, capability schemas, and compatibility exceptions are frozen implementation inputs. “Latest” is not a durable protocol profile.

## S7A-002 — `wow-service` owns transport-independent session orchestration

Applications parse protocol messages and project them to public service requests. They do not coordinate lower owners directly.

## S7A-003 — LSP and MCP are separate applications

`apps/wow-lsp` and `apps/wow-mcp` have independent framing, capability, security, and conformance profiles while depending only on `wow-service`.

## S7A-004 — One request maps to one service operation

No adapter-side multi-owner workflow, hidden retry, automatic fallback, or arbitrary operation dispatch.

## S7A-005 — Sessions bind exact immutable generation sets

Normal requests never resolve or refresh current. Explicit rebind creates a new `SessionViewSet`.

## S7A-006 — Independent stores are not one distributed transaction

Session binding validates an exact compatible set of retained publications and records acquisition receipts. It does not claim cross-store atomicity beyond owner guarantees.

## S7A-007 — Unsaved documents are immutable ephemeral overlays

An overlay is derived from an exact base generation, document identity, protocol version, content digest, and validated edit sequence. It never mutates the published generation.

## S7A-008 — Full content digest closes incremental edit identity

Every accepted document change produces a canonical full-content digest. An incremental edit list alone is not the durable overlay identity.

## S7A-009 — Save notifications are observations

`didSave` or an MCP notification does not prove filesystem bytes, repository state, index publication, or runtime behavior changed.

## S7A-010 — Position encoding is negotiated and explicit

Line/character coordinates are converted under one frozen encoding/newline/source-map profile. Mixed or unsupported encodings fail or become `NotEvaluated`.

## S7A-011 — Pull-style exact diagnostics are the canonical result model

Diagnostic result IDs bind exact session view, overlay, rule/reference/project generations, capability state, and request profile. Push projection may be supported only as a transport view over the same result identity.

## S7A-012 — Protocol capabilities do not grant authority

Client-advertised capabilities, MCP host approval, editor trust, or session identity cannot raise evidence confidence, authorize source edits, or grant tool/network/model/publication access.

## S7A-013 — Code actions are advisory and nonexecuting

E7-A may expose exact typed remediation/action candidates and resolve additional data. It does not return or execute arbitrary commands, apply workspace edits, or claim runtime correctness.

## S7A-014 — Unsupported features remain unavailable

Completion, formatting, rename, execute-command, semantic tokens, inlay hints, call hierarchy, workspace mutation, sampling, elicitation, prompts, dynamic tool registration, and remote daemon transport require explicit later profiles. They are never emulated through raw source or generic calls.

## S7A-015 — MCP exposes an allow-listed schema surface

Every tool/resource has a versioned static descriptor and exact service operation or immutable artifact owner. No generic `call_tool`, arbitrary JSON-RPC passthrough, source-defined tool, or provider-defined executable operation.

## S7A-016 — MCP resources use opaque exact-generation URIs

Canonical resource URIs identify retained framework artifacts and views. They are not arbitrary filesystem paths, `file://` access, repository URLs, or provider locators.

## S7A-017 — MCP sampling/model invocation is outside E7-A

The server never requests model completion, delegates proof to a model, or treats model output as framework evidence.

## S7A-018 — Source and prompt text are structurally isolated data

Document content, comments, MCP arguments, tool descriptions, resource text, and diagnostic messages cannot alter profiles, authorization, routing, or agent instructions.

## S7A-019 — Cancellation is exact and synchronous at the public boundary

Cancellation binds one request/operation. No work continues detached after terminal response, session shutdown, or transport loss.

## S7A-020 — Progress is nonsemantic

Progress percentages, messages, timestamps, and completion notifications are operational telemetry; they do not enter result identity or prove an effect completed.

## S7A-021 — Backpressure is explicit

Every transport/session has bounded inbound, work, progress, partial-result, and outbound queues. Overload produces typed busy/resource errors or cancellation, never silent loss.

## S7A-022 — Response loss does not prove no effect

Effecting service operations preserve durable `OperationId + CanonicalRequestDigest` behavior. A disconnected client cannot trigger blind repetition.

## S7A-023 — Authorization is independent from protocol authentication

Transport authentication may identify a peer; operation authorization separately validates exact scopes and profiles. Neither creates semantic proof.

## S7A-024 — Stdio is the initial local transport candidate

LSP stdio and MCP stdio are the smallest initial profiles. Network listeners, remote HTTP transports, TLS, origin policy, multi-tenancy, and daemon lifecycle are deferred until separately frozen.

## S7A-025 — One process/session is not global current

Operational process/session IDs do not enter project/reference/graph result identities and cannot select or mutate current publications.

## S7A-026 — Protocol errors and domain results remain separate

Malformed framing/schema/method errors are protocol failures. A valid request yielding `Partial`, `Blocked`, `NotEvaluated`, or invalid diagnostics remains a domain result with its exact envelope.

## S7A-027 — Applications do not own caches or project persistence

They may hold bounded session-local framing/overlay/result references. Durable stores, publications, search shards, context packs, and retention remain with owners/service ports.

## S7A-028 — Canonical semantic results are transport-neutral

LSP/MCP envelopes and request IDs do not change owner/service result identity. Protocol projections have separately identified bytes and loss records.

## S7A-029 — No source mutation is required for E7-A correctness

The first protocol implementation proves read/analysis/context/search behavior without edits, formatting, rename, or execute-command.

## S7A-030 — Release/supply-chain activation is E7-B

Toolchain/workspace activation, build artifacts, signing, SBOM, reproducible packaging, release publication, update channels, and rollback are not E7-A responsibilities.
