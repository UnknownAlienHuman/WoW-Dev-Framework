# E7-A cancellation, progress, concurrency, and backpressure

**Status:** normative.

## Request identity

Adapters register a mapping before dispatch:

```text
ProtocolSessionId + protocol request ID
-> service OperationId + CanonicalRequestDigest
-> exact SessionViewSetId / OverlayGenerationId
```

Duplicate protocol request IDs in one active session are rejected unless the exact protocol profile defines a safe notification replay rule.

## Cancellation

Cancellation is best-effort only where the owner operation is cancellable, but reporting is exact.

```text
cancellation requested
-> validate session/request/operation binding
-> append CancellationRequested receipt
-> signal owner/service token
-> observe safe-stop, terminal result, committed effect, or uncertain effect
-> close resources
-> return CancellationReceipt
```

Terminal states distinguish:

```text
CancelledBeforeDispatch
CancelledAtSafePoint
CompletedBeforeCancellation
EffectCommittedBeforeCancellation
OutcomeUnknown
CancellationUnsupportedForOperation
CancellationRequestStale
Failed
```

A cancellation request never rewrites a completed or committed result as cancelled.

## Transport loss

Abrupt disconnect is not automatically equivalent to cancellation. The adapter initiates profile-defined cancellation/closure, while the service reconciles effecting operations. Lost response after dispatch may be `OutcomeUnknown`.

No blind retry occurs on reconnect. A new client must use exact operation/status/reconciliation interfaces allowed by the service contract.

## Progress

Progress receipts use frozen stage enums and bounded counters:

```text
Queued
AcquiringViews
ValidatingInputs
Analyzing
Searching
BuildingContext
Serializing
Closing
Completed
```

Operation-specific stages may be added only in a versioned profile.

Progress records can include:

- completed/known-total counts;
- current bounded stage;
- partial-result artifact/result IDs when the operation supports them;
- cancellation availability;
- noncanonical timing/throughput telemetry.

Progress text from owners or source is untrusted and not emitted as framework instructions.

## Progress nonclaims

- 100% does not prove success before result validation/closure.
- A completion notification does not replace the final response.
- Progress timing/counts do not enter semantic result IDs unless an explicit source evidence contract requires the count itself.
- Missing progress does not imply failure.
- Client-reported progress capability does not authorize additional operations.

## Partial results

Partial result streaming is supported only for explicitly profiled operations. Every chunk binds:

```text
OperationId
exact session view/overlay
result-set/page/continuation identity
stable ordering range
cumulative budgets
coverage/conflict/truncation state
integrity digest
```

Chunks never reset budget, switch generation, reorder prior results, or hide omissions. A final response states the complete/partial terminal state and closes the sequence.

## Queue model

Each session/transport has separate bounded queues for:

```text
inbound frames/messages
parsed requests/notifications
per-document mutations
analysis operations
progress/partial results
outbound responses/notifications
shutdown/closure work
```

System limits are frozen per protocol profile. Client-requested limits can only narrow them.

## Backpressure

When capacity is reached:

- stop reading or reject new requests according to transport/profile;
- return a typed busy/resource-exhausted error for requests;
- coalesce only explicitly coalescible notifications under exact rules;
- never drop responses, cancellation, shutdown, document version transitions, authorization decisions, or effect receipts silently;
- never discard diagnostic/evidence state and report success;
- record overload telemetry without private source.

## Coalescing

Initial profile may coalesce only superseded progress notifications or explicitly replaceable document-change notifications when the protocol and full-content digest closure make the intermediate versions unnecessary for semantics. The canonical overlay result still validates version/order rules.

Document changes, save/close, effecting operations, review/publication operations, and exact continuation pages are not coalesced by convenience.

## Fairness

Scheduling is bounded and deterministic at the semantic layer:

- per-session and global concurrency ceilings;
- per-document mutation serialization;
- no starvation of cancellation/shutdown;
- no priority based on repository/addon/client/provider identity;
- no result ordering based on worker completion;
- effecting operations obey owner serialization/idempotency rules.

Scheduling timings are noncanonical.

## Timeouts

Timeouts are explicit operation/profile inputs and can only narrow system maxima. Timeout means a bounded stop request and status classification; it does not prove no effect.

Timeout handling follows the same cancellation/reconciliation/closure contract.

## Shutdown

During shutdown:

1. stop new admission;
2. process cancellation/shutdown control messages;
3. signal or drain active operations under profile;
4. reconcile uncertain effects;
5. close owner views/leases in reverse order;
6. flush required audit/operation receipts;
7. emit terminal protocol response where possible;
8. close transport.

No detached worker remains.

## Tests

Cover cancellation before/after every owner boundary, late cancellation, duplicate IDs, disconnect after dispatch, progress reorder/overflow, partial-result continuation, queue saturation, notification floods, document-change bursts, slow readers/writers, broken pipe, shutdown races, 1/2/N workers, and semantic-result determinism under different schedules.
