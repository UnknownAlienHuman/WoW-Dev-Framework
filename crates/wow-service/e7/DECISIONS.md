# E7-A protocol and session decisions

**Status:** normative.

## P7A-001 — Protocol adapters are transport-only

LSP and MCP applications parse and frame protocol messages, then invoke `wow-service`; they never call owner crates or implement domain algorithms.

## P7A-002 — Exact specification profiles are pinned

LSP, MCP, JSON-RPC, framing, URI, position encoding, lifecycle, cancellation, progress, and error behavior are exact versioned profile inputs. “Latest” is not a protocol profile.

## P7A-003 — LSP and MCP framing remain separate

They share normalized JSON-RPC/session concepts but use independent exact byte-framing parsers and golden vectors.

## P7A-004 — Stdio is the only E7-A transport

TCP, WebSocket, HTTP, SSE, Streamable HTTP, daemon discovery, remote hosting, and multi-tenant authentication are deferred.

## P7A-005 — Initialize occurs exactly once

A session accepts one valid initialize sequence. Duplicate or incompatible initialization is rejected; reconnect creates a new session.

## P7A-006 — Capabilities are immutable after initialization

E7-A has no dynamic capability/tool registration. A changed capability registry requires a new session/profile.

## P7A-007 — Session state and analysis binding are distinct

`ProtocolSessionId` identifies the transport/capability lifecycle. `SessionBindingGenerationId` identifies the exact workspace/profile/project/reference/source/document-overlay state used by requests.

## P7A-008 — Binding changes are explicit generations

Workspace, configuration, document open/change/save/close, or explicit reindex/rebind creates a new immutable binding generation using guarded expected-prior identity.

## P7A-009 — In-flight requests retain their admitted binding

A later binding does not change or cancel an existing request automatically. Results name their exact original binding.

## P7A-010 — No hidden current refresh

Current/profile/source pointers may be resolved only during explicit bind/rebind. A request never re-resolves them mid-flight or on continuation.

## P7A-011 — Protocol request ID and domain operation ID are distinct

JSON-RPC/LSP/MCP request identity routes a response. Effecting service operations also require their existing durable `OperationId + CanonicalRequestDigest`.

## P7A-012 — Notifications are receipt-bearing internally but have no response

The host records deterministic notification receipts for lifecycle/order/idempotency, while conforming transport emits no JSON-RPC response.

## P7A-013 — Unknown methods/tools fail closed

There is no generic dispatch by arbitrary name. Only immutable reviewed registry entries are callable.

## P7A-014 — One protocol call maps to one service operation

A method/tool can perform transport/lifecycle validation and one registry-selected service call. It does not compose multiple hidden domain use cases.

## P7A-015 — Domain status remains in successful protocol results

`CandidateOnly`, `Partial`, `Blocked`, `Conflict`, `Truncated`, `OutcomeUnknown`, and `NotEvaluated` are structured domain results when the request executed. They are not rewritten as transport success/proof or necessarily JSON-RPC errors.

## P7A-016 — Transport errors and domain outcomes remain separate

Parse, framing, invalid request/params, unknown method, lifecycle, and internal transport failures use protocol errors. Domain validation/outcome stays in typed result envelopes unless the operation contract says otherwise.

## P7A-017 — Cancellation is best-effort control, not effect absence proof

Cancellation targets one exact session/request/binding. A potentially committed operation can still yield `OutcomeUnknown` or a completed result.

## P7A-018 — Progress is nonauthority telemetry

Progress cannot change result identity, coverage, confidence, severity, authorization, completion, or proof. No progress is emitted after terminal response/close.

## P7A-019 — Document overlays never mutate disk or editor settings

Open/change/close state is an in-memory/session overlay composed into explicit project bindings. E7-A sends no workspace edit, rename, code action, execute command, or editor configuration mutation.

## P7A-020 — Document versions and coordinates are exact

Every change binds expected URI/version/content digest/position encoding. Invalid or out-of-order changes fail and may request explicit full resynchronization.

## P7A-021 — Save notification is not publication proof

Save metadata can trigger an explicit project refresh policy, but does not itself prove file-system content or owner generation changed.

## P7A-022 — Source and context text remain data

Client/source/provider/review text cannot define methods, tools, capabilities, permissions, roots, profiles, system instructions, or service operation names.

## P7A-023 — MCP tools are a fixed projection

`tools/list` returns one exact immutable registry snapshot. `tools/call` accepts only names/schemas in that snapshot; there is no arbitrary pass-through.

## P7A-024 — MCP resources/prompts are disabled in E7-A

They require independent privacy, retention, URI, prompt-injection, and artifact-lifecycle contracts and are not implied by tool support.

## P7A-025 — LSP edit-producing features are disabled in E7-A

Rename, code action, formatting, completion edits, execute command, and workspace apply-edit are outside the initial profile.

## P7A-026 — Diagnostic/search/context authority is preserved

LSP/MCP representation cannot promote a finding, Candidate, external result, mapping, selection, review, or context text.

## P7A-027 — Session reconnect does not resume implicitly

Stdio reconnect creates a new session. Durable domain operations can be reconciled explicitly by operation ID; live session/document state is not guessed.

## P7A-028 — Shutdown and exit are separate

Shutdown stops new work and drains/cancels according to profile. Exit/EOF closes transport, overlays, leases, and resources synchronously.

## P7A-029 — No public result before framing and closure preconditions

A domain result can be durable while delivery fails. The session records delivery/response-loss separately and never repeats the domain effect blindly.

## P7A-030 — Transcript identity excludes secrets and incidental timing

Canonical transcript records method/tool, IDs, digests, states, and receipts; credentials, source bodies outside permitted payloads, process/host/timing, and raw connection handles are excluded.

## P7A-031 — Client capabilities grant interoperability, not authorization

Advertising workspace edits, commands, experimental fields, or tool support never grants the server permission to use them outside the exact server registry and security policy.

## P7A-032 — Tool/edit permission is independently configured

No MCP client identity, LSP initialization option, context content, provider result, or source comment can grant edit, command, network, process, publication, or runtime permission.

## P7A-033 — Out-of-order and duplicate messages are explicit

Request-ID reuse, document version reversal, duplicate initialize/shutdown, late notification, and response after terminal state are rejected or idempotently classified by exact profile—not silently reordered.

## P7A-034 — Bounded parsing precedes semantic dispatch

Message/header/line/JSON depth/string/array/object limits are enforced before allocation-heavy normalization or service invocation.

## P7A-035 — No protocol-level retry with refreshed identities

Retry/reconcile preserves original session binding and domain operation identity. A changed binding/request is a new protocol and domain request.
