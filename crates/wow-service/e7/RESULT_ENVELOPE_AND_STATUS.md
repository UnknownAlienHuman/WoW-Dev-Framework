# E7-A result envelopes and status

**Status:** normative.

## Service envelope

Every E7-A service operation returns one tagged `FrontendServiceResultEnvelope` containing:

```text
schema/operation/registry versions
transport request and service request IDs
optional durable OperationId + CanonicalRequestDigest
session/client/protocol/capability IDs
workspace/document/overlay/generation IDs
operation-specific owner result or structured failure
transport/session/workspace/document/operation/delivery states
owner evidence/coverage/conflicts/omissions/nonclaims
progress/cancellation/reconnect/backpressure summary
privacy/license/authorization/retention/audit/close state
canonical digest
```

Owner result bytes and statuses are preserved; frontend metadata cannot overwrite them.

## Outer statuses

```text
Complete
NoChange
Ready
Partial
Blocked
ConflictBlocked
Truncated
OutcomeUnknown
ResynchronizationRequired
NotEvaluated
Cancelled
Closed
Failed
```

Default conservative precedence:

```text
Failed
OutcomeUnknown
Cancelled
ResynchronizationRequired
NotEvaluated
ConflictBlocked
Blocked
Truncated
Partial
Closed
Ready
NoChange
Complete
```

Operation-specific state remains separate. A completed diagnostic request can contain findings; a completed validation can be `Invalid`; a session can be `Ready` while one capability is unavailable.

## Transport projection

### CLI/local daemon

Return the exact canonical service envelope or a lossless wrapper with protocol metadata. No second status calculation.

### LSP

Map ranges/severities/result IDs and method-specific types while preserving stable diagnostic codes, source/evidence references, data payload with exact service result IDs, and partial/`NotEvaluated` state. When an LSP type cannot express a required field, include a bounded `data` reference or omit the capability; do not silently discard canonical meaning.

### MCP

`structuredContent` contains the exact schema-conforming projection. A text block mirrors it faithfully. Completed domain invalidity/blocking remains a structured result; protocol/argument errors remain protocol errors. Failed/unknown operations retain recovery IDs.

## Delivery versus completion

These are independent:

```text
service operation completed
result retained
transport response prepared
response sent
response acknowledged or connection lost
```

A delivery failure after service completion does not turn the operation into a failure or cause recomputation. It produces an exact delivery/retrieval state.

## ResynchronizationRequired

Used for stale/out-of-order/mismatched document overlay changes. It requires the client to send a full exact document state under a new valid version/operation. It is not `Failed` and does not apply the suspect change partially.

## NoChange

Requires exact proof that the same requested session/workspace/document/operation state already exists. Same path, client version, method, or result text alone is insufficient.

## Nonclaims

Include as applicable:

```text
transport-success-is-not-semantic-proof
progress-is-not-completion
client-identity-is-not-authorization
workspace-root-is-untrusted-input
overlay-result-is-bound-to-exact-document-version
published-generation-data-may-be-stale-for-unsaved-overlay
mcp-tool-invocation-is-not-user-authorization
external-candidate-remains-Candidate
not-runtime-verified
not-source-edit-authorized
not-publicly-released
```

## Canonicalization

Transport-specific IDs, connection order, queue timing, endpoint path, process ID, terminal/editor details, network latency, and progress delivery do not enter owner semantic result digests. Exact session/overlay identities enter only operations whose semantics depend on them.