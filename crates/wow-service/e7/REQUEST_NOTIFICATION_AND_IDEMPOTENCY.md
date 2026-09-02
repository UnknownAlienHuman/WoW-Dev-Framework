# E7-A requests, notifications, identity, and idempotency

**Status:** normative.

## Request identity layers

```text
transport request ID
    JSON-RPC string or integer value used to correlate one response

ProtocolRequestRecordId
    session + typed transport ID + method/tool + admitted binding + params digest

Domain OperationId + CanonicalRequestDigest
    existing durable identity required by effecting wow-service use cases
```

These identities are related but never substituted for each other.

## JSON-RPC request IDs

The exact protocol profile defines permitted ID kinds. Initial E7-A profiles permit only the intersection explicitly supported by the pinned LSP/MCP specifications and implementation.

Rules:

- preserve string versus integer type and exact value;
- reject unsupported null, fractional, out-of-range, duplicate-active, or structurally invalid IDs;
- no coercion from `1` to `"1"`;
- no reuse while a request is active or retained under response-delivery uncertainty;
- a reused ID after terminal closure is allowed only if the exact profile defines it, and still creates a distinct `ProtocolRequestRecordId` by sequence/binding/params;
- response echoes the exact request ID type/value.

## Request admission

```text
parse and frame one bounded message
-> validate JSON-RPC envelope and lifecycle state
-> resolve exact method/tool registry entry
-> capture active SessionBindingGeneration
-> validate params schema and permissions
-> normalize params and compute digest
-> validate/create domain OperationId when required
-> create ProtocolRequestRecord
-> invoke exactly one registry target
```

No arbitrary method fallback, reflection, dynamic handler lookup from source/client text, or generic service operation name.

## Method/tool aliases

Aliases are permitted only when they are explicit registry entries with their own protocol name, exact schema, version, and canonical target. No fuzzy/case-folded/name-prefix matching.

## Notifications

A notification has no transport request ID and receives no JSON-RPC response. The host creates an internal receipt with exact session/binding, method, sequence, params digest, and effect/lifecycle result.

Notification categories:

```text
lifecycle ready/exit
workspace/document transitions
cancellation
progress acknowledgement where profile defines
protocol telemetry/trace setting under explicit safe schema
```

Unknown notifications are handled exactly as the pinned protocol profile requires—ignored with receipt or rejected/logged—without generating a response or invoking arbitrary logic.

## Notification idempotency

Lifecycle/document notifications can affect session binding. Each effecting notification receives an internal durable or session receipt keyed by exact session, method, sequence/version, and params digest.

- exact replay can be classified idempotent;
- same document version with different content/digest is conflict;
- duplicate exit/ready/change is not silently reexecuted;
- response loss is not applicable to a notification response, but transport disconnect may leave its effect uncertain and must be reconciled from internal state.

## Service operation registry dispatch

A registry entry identifies one target function and profile. The dispatcher passes:

```text
exact session/binding/request context
validated typed params
exact profile/permission/budget/cancellation context
optional durable domain operation identity
```

It receives one typed service result/error. It cannot call another entry as hidden fallback.

## Read-only versus effecting operations

Every entry declares:

```text
PureRead
SnapshotReadWithRetention
SessionBindingEffect
DurableDomainEffect
ProtocolLifecycleEffect
```

A read can still require retention/response delivery records. A durable domain effect must use its existing operation/reconciliation contract. The protocol method name does not determine effect class.

## Domain idempotency

For effecting delegated operations:

- protocol params include or deterministically bind an explicit domain `OperationId` under the registry schema;
- service validates `OperationId + CanonicalRequestDigest` before effect;
- same protocol request replay returns/reconciles the exact domain outcome;
- transport retry never creates a new domain operation silently;
- changed binding/params/profile creates a new protocol request and requires a new domain identity unless the owner contract explicitly supports the change.

## Response delivery state

After domain completion:

```text
DomainCompleted
-> ProtocolResponsePrepared
-> write/flush framed bytes
-> ResponseDelivered
```

Failures can yield:

```text
ResponseDeliveryUnknown
TransportClosedBeforeDelivery
SerializationFailed
FramingFailed
```

The domain artifact/effect keeps its exact state. A reconnecting client reconciles using exact domain operation IDs, not by assuming the original request was absent.

## Batch messages

JSON-RPC batch requests are disabled in E7-A unless the exact external protocol profile requires and the repository contract separately defines ordering, partial failure, cancellation, resource, and one-call semantics. Initial LSP/MCP stdio profiles reject batches.

## Request ordering

Independent read requests may execute concurrently under exact resource/profile limits. Effecting workspace/document/binding/lifecycle operations are serialized by session and expected-prior guards.

Completion order does not alter canonical results. Responses can follow the pinned protocol’s concurrency rules but retain exact IDs.

## Unknown and unsupported operations

Distinguish:

```text
MethodNotFound
KnownButCapabilityNotNegotiated
KnownButWrongSessionState
KnownButWrongBindingOrDocumentState
KnownButPermissionDenied
KnownButOwnerCapabilityNotEvaluated
```

Do not collapse all into one success/empty result.

## Input text boundary

Request string fields, document source, provider content, context text, review notes, and initialization metadata remain data. They cannot create a method/tool name, registry entry, operation ID, permission, profile, or callback.

## Tests

- string/integer IDs and no coercion;
- null/fractional/duplicate active IDs;
- exact response echo;
- unknown/case-folded/fuzzy method;
- request before initialize/after shutdown;
- one registry target only;
- read/effect classification mismatch;
- notification no-response rule;
- duplicate document/lifecycle notifications;
- domain operation replay with response loss;
- response delivery failure after domain commit;
- batch request rejection;
- concurrency with deterministic outputs;
- source text shaped like method/tool JSON.
