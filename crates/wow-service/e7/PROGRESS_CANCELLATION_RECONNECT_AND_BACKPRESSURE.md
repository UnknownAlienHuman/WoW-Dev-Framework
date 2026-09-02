# E7-A progress, cancellation, reconnect, and backpressure

**Status:** normative.

## Progress

Progress reports operational stage only. A progress event can include bounded counts, current stage, total when exactly known, and a redacted message. It is always marked nonauthoritative.

Progress cannot prove:

```text
operation completion
successful durable effect
complete source/graph/query coverage
clean negative result
final candidate ordering
authorization
retention or close success
```

Progress sequences are monotonic within one stream, but missing/coalesced events are permitted. The final service result is authoritative.

## Cancellation

Every cancelable transport request maps to one `FrontendOperationTicket`. Cancellation states:

```text
NotRequested
Requested
ForwardedToOwner
OwnerAcknowledged
CompletedBeforeCancel
CancelledBeforeDispatch
CancelledAfterPossibleEffect
CancellationOutcomeUnknown
```

A transport cancellation notification is intent, not proof. The service preserves owner/durable effect state and returns/reconciles the exact result.

Cancellation never starts an unowned detached cleanup or automatically issues compensating effects.

## Disconnect

Closing stdio, named pipe, Unix socket, HTTP/SSE connection, editor window, or MCP client does not imply cancellation. The session profile determines whether pure reads are abandoned and whether durable/external effects continue to a recoverable state.

For any effect that may have dispatched, the exact operation remains queryable/reconcilable. A new connection cannot reuse the wire request ID as proof of the same operation.

## Reconnect

Reconnect requires:

```text
exact daemon/server identity
compatible protocol/profile and registry generation
client/session resume credential or OS peer binding when supported
consumer/privacy/license compatibility
unexpired session/operation lease
exact durable operation/result IDs
```

If session resume is unavailable, the client opens a new session and may retrieve/reconcile durable results by exact authorization. Unsaved overlay state is not assumed recoverable; the client must replay full documents under new overlay identities unless an explicit secure checkpoint exists.

## Response delivery loss

Delivery states:

```text
Prepared
Sent
Acknowledged
ConnectionLostBeforeSend
ConnectionLostAfterPossibleSend
ReplayAvailable
ReplayExpired
```

Delivery uncertainty does not alter the underlying service operation. Replaying a final response is allowed only for the same exact session/consumer or another authorized reader and never re-executes the operation.

## Backpressure

Bound separately:

```text
inbound frame/message queue
active requests per connection/session/workspace
owner operation concurrency
outbound final-result bytes
outbound progress/log queue
resource reads and source bytes
SSE replay window
response journal entries and retention
```

Priority:

```text
final response/error
cancellation and required state transitions
authorization/session expiry
registry/resource invalidation
progress
logs
```

Progress/log messages may be coalesced or dropped with an explicit counter. Final results/errors are never silently dropped; if delivery cannot complete, the service records delivery failure and exposes exact retrieval/reconciliation where supported.

## Timeouts

Each request has idle and absolute deadlines. Progress may reset the idle deadline only within the profile; it never extends the absolute maximum. Timeout requests cancellation, then returns exact service/owner state. Timeout after possible effect can yield `OutcomeUnknown`.

## Partial results and streaming

A transport may stream partial-result chunks only when the service operation defines immutable deterministic chunking and a final manifest. Partial delivery cannot be reassembled from mixed operation/generation/profile IDs. Source and graph streams remain bounded.

The first profiles permit streaming for large diagnostics/search/context lists only after their chunk schemas and continuation semantics are frozen. Otherwise return a retained result handle plus bounded summary.

## Fairness

Per-session and global quotas prevent one client from starving others. Scheduling order is operational and excluded from semantic output. Equivalent exact requests under the same profiles produce identical final semantic bytes regardless of queue order.

## Shutdown

Graceful shutdown stops new sessions/requests, requests cancellation as defined, awaits/reconciles mandatory effects within bounded time, persists response journals/retention/audit, closes resources/endpoints, and exits. Forced termination triggers startup recovery; it is never reported as graceful.