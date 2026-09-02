# E7-A local daemon transport

**Status:** normative profile `wow-local-jsonrpc/1`.

## Purpose

The local daemon optionally shares one initialized service/store runtime across multiple trusted local frontends while preserving per-client sessions, workspaces, overlays, authorization, budgets, and output isolation.

The first implementation may omit daemon mode and run LSP/MCP embedded in the `wow` process. Daemon capability is not advertised until its full profile passes.

## Endpoint profiles

```text
Windows: named pipe with current-user SID ACL
Unix:   Unix-domain socket under an explicit runtime directory with mode 0600
```

No TCP listener, wildcard bind, public socket, browser endpoint, or remote tunnel is part of the default daemon profile.

Endpoint discovery is explicit:

- caller supplies `--endpoint` or a release-profile-defined current-user runtime path;
- endpoint identity is printed by `wow daemon status` under local policy;
- no network scan, random port search, registry search, or cross-user discovery.

## Framing

The daemon uses JSON-RPC 2.0 objects over a length-delimited UTF-8 stream. The exact profile freezes header/length encoding, maximum frame size, canonical request schema, and error mapping. Batches are not accepted in version 1.

Only reviewed methods are available:

```text
initialize
capabilities/get
session/open
session/get
session/close
operation/call
operation/cancel
operation/get
operation/reconcile
ping
shutdown
```

`operation/call` is not generic: its `operation` field must name an entry in the exact negotiated `FrontendOperationRegistry`, and arguments must validate against that entry's schema. There is no arbitrary method, shell, plugin, or lower-owner handle.

## Initialization

The client sends supported daemon protocol versions, client implementation identity, requested registry generation, consumer/privacy/license profile, and supported framing/progress features. The server selects one exact compatible version and returns server identity, registry/capability set, limits, and peer/session policy.

No operation except `ping` and initialization is accepted before successful negotiation.

## Peer authentication

The default local profile requires OS peer identity and endpoint access control. Optional challenge/nonce binding may protect against endpoint confusion. Identity proves local peer scope only; it does not authorize service effects. Effecting operations still require their own authorization receipts.

No bearer token is accepted on the command line or written to logs. If an additional session secret is needed, it is inherited through a protected handle or current-user file with strict permissions and bounded lifetime.

## Multi-client isolation

Every connection has a client connection ID and zero or more explicit frontend sessions. The daemon enforces limits and separation for:

```text
workspace roots and access policies
document overlays
operation registry/capability set
authorization and consumer profiles
progress/output queues
durable operation visibility
private source and result resources
leases and close behavior
```

Shared immutable artifacts are visible only when their consumer/privacy/license policy allows.

## Operation lifecycle

```text
validate connection/session/registry entry
-> create transport request ID
-> construct one typed service request
-> invoke exactly one service operation
-> emit bounded progress notifications
-> journal final response delivery when required
-> return exact final envelope/error
```

Disconnect does not cancel. The operation follows its service policy and can be cancelled/reconciled by exact IDs after reconnect.

## Backpressure

Per-connection and per-session outgoing queues are bounded. Priority:

```text
final response/error
cancellation/state transition
required authorization/expiry notice
resource/registry invalidation
progress
logs
```

Progress/log events may be coalesced or dropped with counters. Final responses are never silently dropped; failed delivery is journaled and retrievable when the operation/result is durable.

## Shutdown

`shutdown` requires local administrative authorization and drains or cancels active operations according to exact profiles. New sessions stop; existing operations are reconciled; journals/retention/audit flush; endpoints close; resources close in reverse order. Killing the process is not graceful shutdown and triggers startup recovery.

## Security hard stops

- no cross-user endpoint access;
- no symlink/reparse/device endpoint substitution;
- no arbitrary operation names or batches;
- no raw source/credential in handshake/logging;
- no remote listener or auto-port exposure;
- no endpoint file with permissive ACL/mode;
- no silent reconnect to another daemon identity;
- no daemon progress treated as semantic completion.