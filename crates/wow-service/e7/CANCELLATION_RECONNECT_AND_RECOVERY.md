# E7-A cancellation, disconnect, reconnect, and recovery

**Status:** normative.

## Cancellation classes

```text
TransportRequestCancellation
SessionCloseCancellation
OperationOwnerCancellation
ArtifactStreamCancellation
HostShutdownCancellation
```

Cancellation is a typed request bound to exact client/session/request/operation identity. It is not inferred from socket close, editor exit, timeout, process signal, or missing heartbeat unless the selected profile explicitly maps that event to cancellation intent.

## Admission and dispatch

Before owner dispatch, cancellation can produce a clean cancelled/no-effect result. After any externally observable or durable effect may have started, cancellation records intent and asks the exact owner to stop safely; it does not prove no effect.

Every effecting request has:

```text
OperationId
CanonicalRequestDigest
SessionGenerationId
transport request ID
owner dispatch/effect receipts
final delivery state
```

## Disconnect

Disconnect is transport state. It does not automatically:

- cancel an effecting operation;
- roll back a session update;
- close a daemon session before lease policy says so;
- prove no result was produced;
- authorize retry;
- discard retained output/evidence.

Ephemeral stdio profiles may map process/stream termination to cancellation and session-close intent, but owner reconciliation rules still apply.

## Reconnectable daemon sessions

A daemon session can be reattached only with exact:

```text
SessionId and nonsemantic transport capability proof
client/tenant scope
host/build/compatibility profile
expected SessionGenerationId
lease state
privacy/authorization scope
```

Transport capability proof is sensitive operational data and never enters semantic artifacts/logs. Reconnect cannot broaden operation exposure, source disclosure, or authorization.

## Request recovery

A reconnecting client may call `transport_operation_reconcile` with exact request/session/operation identity. The result distinguishes:

```text
NotAdmitted
AdmittedNotDispatched
DispatchedOutcomeUnknown
CompletedResultRetained
CompletedResultDelivered
CancelledBeforeEffect
CancelledEffectStateKnown
FailedStateKnown
ConflictingReceiptsQuarantined
ExpiredOrUnavailable
```

Only `NotAdmitted` or exact owner proof of no effect can authorize a new execution under the same logical request profile. A completed retained result is returned rather than recomputed.

## Read-only retries

A read-only operation can be retried automatically only if its descriptor states:

```text
no external/durable effect
same exact SessionSnapshot and owner generations retained
same request digest and budgets
no privacy/license/profile drift
```

The retry is a new delivery attempt of equivalent computation, not evidence that the original did not complete. If exact snapshots are unavailable, return stale/`NotEvaluated`.

## Effecting retries

Effecting operations never retry automatically. Same operation ID/digest invokes owner reconciliation or returns the retained outcome. Same operation ID with another digest fails. A new operation ID is a new explicit effect request and cannot be called a retry of an unknown effect.

## Session-update races

Project/profile/overlay/session-close effects use expected-current session generation. If the response is lost, the client reconciles the exact update before submitting another update. It cannot guess whether the session advanced from a locally cached generation.

## Late results

A result arriving after cancellation/disconnect is validated and classified by exact owner/operation state. It may be retained as the canonical effect result, marked cancelled with committed effect, or quarantined if conflicting. It is never silently emitted into a different request or session.

## Host startup recovery

A daemon host examines only owned durable session/request/lease/effect/audit records and exact owner reconciliation ports. It may:

```text
restore eligible sessions and leases
finalize known retained results
reconcile incomplete session updates or streams
expire sessions
quarantine conflicts
close orphan resources
```

It may not run semantic requests, refresh projects/profiles, dispatch provider queries, apply edits, or open source merely because a prior client existed.

## Host shutdown

Graceful shutdown:

```text
stop accepting clients/requests
announce bounded shutdown state
cancel or drain per exact profile
persist request/session/effect receipts
close streams and sessions
close service/owner/store resources
return process status
```

Forced termination is recovered later from durable records; no clean-close claim is made.

## Response delivery

Service outcome and transport delivery are separate:

```text
ServiceCompletedDeliveryPending
ServiceCompletedDelivered
ServiceFailedDeliveryPending
OutcomeUnknown
```

A delivery failure cannot change owner facts or cause a second effect. Final envelopes include recovery identifiers where policy permits.

## No background work

After a one-shot invocation or session close reports completion, no unowned retry, query, index, cleanup, polling, stream, or edit task continues. Long-running daemon work must be represented by an explicit durable request still owned by an active/closing session and operation record.