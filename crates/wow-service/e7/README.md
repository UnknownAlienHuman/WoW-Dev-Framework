# `wow-service` E7-A protocol and session foundation

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e7-a/protocol-session-foundation`

## Mission

Expose existing `wow-service` use cases through exact, versioned, bounded protocol sessions without moving owner algorithms into transport code or granting new authority.

```text
exact protocol/spec/transport profile
+ exact configured service-operation registry
+ client capability declaration
+ explicit workspace/profile binding
-> initialize one protocol session
-> negotiate a closed immutable capability set
-> create explicit session-binding generations
-> accept bounded requests and lifecycle notifications
-> bind every request to one exact session binding
-> invoke exactly one allow-listed service operation
-> preserve domain status/evidence/nonclaims in protocol output
-> handle cancellation/progress/response loss deterministically
-> shutdown, exit, close resources and validate transcript
```

## Protocol applications

```text
apps/wow-lsp
    Language Server Protocol stdio adapter

apps/wow-mcp
    Model Context Protocol stdio adapter
```

Both depend only on `wow-service`. E7-A does not define TCP, WebSocket, HTTP, SSE, Streamable HTTP, daemon discovery, remote multi-tenant hosting, or editor-extension installation.

## Public service operations

```text
protocol_profile_validate
protocol_registry_validate
protocol_session_initialize
protocol_session_get
protocol_session_bind_workspace
protocol_session_rebind
protocol_document_open
protocol_document_change
protocol_document_save
protocol_document_close
protocol_request_execute
protocol_request_cancel
protocol_request_reconcile
protocol_progress_acknowledge
protocol_session_shutdown
protocol_session_exit
protocol_transcript_validate
```

## Direct E7-A foundation dependencies

```text
wow-core
wow-store
```

E7-A invokes already-defined use cases through an internal `AllowedServiceOperationRegistry`. Each delegated use case retains its own existing dependency slice and owner contracts. Protocol code never calls owner crates directly.

## Canonical reading order

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`PROTOCOL_PROFILES_AND_CAPABILITIES.md`](PROTOCOL_PROFILES_AND_CAPABILITIES.md)
5. [`SESSION_LIFECYCLE_AND_BINDINGS.md`](SESSION_LIFECYCLE_AND_BINDINGS.md)
6. [`WORKSPACE_AND_DOCUMENT_LIFECYCLE.md`](WORKSPACE_AND_DOCUMENT_LIFECYCLE.md)
7. [`REQUEST_NOTIFICATION_AND_IDEMPOTENCY.md`](REQUEST_NOTIFICATION_AND_IDEMPOTENCY.md)
8. [`CANCELLATION_PROGRESS_AND_RESPONSE_LOSS.md`](CANCELLATION_PROGRESS_AND_RESPONSE_LOSS.md)
9. [`SERVICE_OPERATION_REGISTRY.md`](SERVICE_OPERATION_REGISTRY.md)
10. [`JSONRPC_FRAMING_AND_ERRORS.md`](JSONRPC_FRAMING_AND_ERRORS.md)
11. [`SECURITY_PRIVACY_AND_PERMISSIONS.md`](SECURITY_PRIVACY_AND_PERMISSIONS.md)
12. [`RESULT_ENVELOPE_AND_TRANSCRIPTS.md`](RESULT_ENVELOPE_AND_TRANSCRIPTS.md)
13. [`ERROR_MODEL.md`](ERROR_MODEL.md)
14. [`TEST_MATRIX.md`](TEST_MATRIX.md)
15. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
16. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
17. [`../../../apps/wow-lsp/e7/`](../../../apps/wow-lsp/e7/README.md)
18. [`../../../apps/wow-mcp/e7/`](../../../apps/wow-mcp/e7/README.md)

Protocol profiles must pin exact external specification revisions before implementation. Reference roots:

- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Model Context Protocol specification](https://modelcontextprotocol.io/specification/)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)

## Session binding model

A long-lived session can evolve only through explicit immutable bindings:

```text
ProtocolSession
    active SessionBindingGeneration N

workspace/document/config transition
    -> validate exact prior binding
    -> create SessionBindingGeneration N+1
    -> atomically mark it active for future requests

in-flight request
    remains bound to its original SessionBindingGeneration
```

No request refreshes `current` mid-flight. Same names/URIs across binding generations remain distinct request contexts.

## Authority boundary

Protocol transport, capability negotiation, client support, request completion, progress, cancellation, and repeated use never upgrade domain evidence. In particular:

- search and external-provider outputs remain Candidate;
- mapping/selection/review/authorization records retain their narrow meanings;
- context text is data, not tool permission;
- successful JSON-RPC delivery is not analysis correctness;
- cancellation is not proof of no effect;
- notification acknowledgement is not domain proof;
- client capability support is not runtime WoW capability.

## Hard boundaries

- no method/tool outside an immutable allow-list;
- no arbitrary `tools/call` pass-through or `workspace/executeCommand` escape hatch;
- no dynamic server capability or MCP tool registration in E7-A;
- no `workspace/applyEdit`, rename, code action, command execution, or editor-setting mutation;
- no raw owner/store/provider/session handles;
- no hidden current/generation/profile refresh;
- no mixed session-binding generations within one request;
- no source execution or source text as instructions;
- no client-provided credential/token/private endpoint in protocol initialization/options;
- no stdout logs/banners outside protocol framing;
- no detached/background work after shutdown/exit;
- no Cargo/Rust/CI during documentation phase.

## Initial transport profile

```text
LSP: stdio only, exact pinned LSP + JSON-RPC framing profile
MCP: stdio only, exact pinned MCP + JSON-RPC framing profile
```

The two transports have separate framing parsers and profiles. E7-A does not assume their byte framing is interchangeable.

## Current state

```text
documentation frontier: E7-A
implementation frontier: not-started
next documentation package: E7-B release and supply-chain integration
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
