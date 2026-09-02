# E7-A local daemon protocol

**Status:** normative baseline local IPC contract.

## Endpoint profiles

```text
windows-named-pipe-local-v1
unix-domain-socket-local-v1
```

The endpoint is explicit in host configuration and created with restrictive owner access. TCP, HTTP, WebSocket, loopback port discovery, browser access, and public network serving are not supported in E7-A.

Endpoint location/name is operational and excluded from semantic IDs.

## Framing

Each connection uses one exact binary framing profile:

```text
4-byte unsigned big-endian payload length
followed by exactly that many UTF-8 JSON bytes
```

The maximum frame size is frozen by the compatibility manifest. Zero/overflow/oversized/truncated/trailing-byte frames fail. Compression, multiplexing inside a frame, and batch semantic requests are disabled in the baseline profile.

The JSON payload is strict, duplicate-key-rejecting and versioned. Canonical service results remain canonical payload fields; the outer daemon frame itself is transport data.

## Handshake

The first client message is `hello`:

```text
protocol profile/version
client implementation/version
expected framework build or accepted compatibility range
expected service registry ID/digest
requested exposure profile
requested resource limits bounded by server maximum
optional reconnect SessionId plus transport capability proof
```

Server returns one of:

```text
hello_accepted
hello_narrowed
hello_rejected
```

The accepted response binds exact build/compatibility/registry/schema/limit IDs and a transport connection ID. Negotiation only intersects capabilities; it cannot add methods or raise limits.

No semantic service operation is invoked during handshake.

## Method registry

Daemon methods are a static map to exact E7-A or earlier service operation IDs. Each semantic request contains:

```text
protocol version
connection and SessionId
expected SessionGenerationId
client request ID
exact daemon method ID/version
OperationId when required
strict typed params
optional progress token
```

There is no `invoke`, arbitrary operation name, generic JSON-RPC method, MCP tool pass-through, SQL, shell, script, model, or plugin method.

## Response

Exactly one final response is associated with each admitted request:

```text
client request ID
service operation ID/version
session generation used
canonical service result envelope or bounded transport error
final delivery/reconciliation references
```

Progress and stream frames use separate fixed message types and exact request/stream sequence numbers. They cannot replace the final response.

## Session lifecycle methods

The baseline registry includes exact mappings for:

```text
session.open
session.get
session.close
session.project.bind
session.profile.bind
session.overlay.open
session.overlay.change
session.overlay.save
session.overlay.close
session.snapshot.get
operation.reconcile
artifact.stream.open
artifact.stream.continue
```

All other semantic methods map one-to-one to a registered service operation.

## Concurrency

A connection may have multiple in-flight requests only up to the exact profile limits. Requests execute against captured immutable session snapshots. Arrival order does not define semantic effect order; effecting session updates use expected-generation CAS and durable operation receipts.

Per-session update serialization may be used by the host as an operational mechanism, but it cannot hide a stale expected generation or alter canonical owner ordering.

## Backpressure

The daemon freezes per-connection/session limits for frames, queued bytes, in-flight requests, progress, streams and output buffering. When capacity is unavailable, the daemon rejects admission with an explicit bounded error before service dispatch whenever possible.

No request is silently dropped, duplicated or reordered. A slow client can lose optional progress or have its connection/session closed under policy, but committed effects/results remain retained/reconcilable.

## Reconnect

Only `DurableLocalDaemonSession` is reconnectable. Reconnect requires exact session/client/host/compatibility/lease scope plus a sensitive transport capability proof. The proof is not a service credential and is never logged or placed in semantic envelopes.

A reconnecting client retrieves retained results or invokes `operation.reconcile`; it never blindly resubmits an unknown effect.

## Cancellation

`request.cancel` identifies exact connection/session/client request/service operation/OperationId. A disconnect is not itself a clean cancellation. Cancellation results preserve owner effect state and `OutcomeUnknown` where necessary.

## Server shutdown

`server.shutdown` is allowed only under explicit local host authorization and maps to host lifecycle, not a semantic service operation. It stops admission, drains/cancels according to profile, persists receipts, closes sessions/streams/service/store resources, and exits.

No arbitrary process-control methods exist.

## Errors

Transport errors are closed and bounded:

```text
ProtocolVersionMismatch
RegistryMismatch
FrameInvalid
MessageInvalid
MethodUnknownOrDisabled
SessionInvalid
GenerationStale
Backpressure
UnauthorizedTransportAccess
ServiceResultUnavailable
OutcomeUnknown
InternalTransportFailure
```

They do not reflect raw payloads, source, secrets, endpoint capabilities, private paths, or stack traces.

## Determinism

Equivalent accepted daemon payloads mapped to the same exact service request/session snapshot produce the same canonical service result as one-shot CLI/LSP/MCP. Connection IDs, framing, queue order, endpoint path and transport timing never enter semantic output.