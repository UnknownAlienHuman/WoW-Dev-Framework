# E7-A cancellation, progress, and response-loss semantics

**Status:** normative.

## Cancellation target

A cancellation message/notification binds:

```text
ProtocolSessionId
exact transport request ID and ProtocolRequestRecordId
exact admitted SessionBindingGenerationId
cancellation profile and receipt ID
```

It cannot cancel another session, reused request ID, another binding generation, or an unrelated domain operation.

## Cancellation lifecycle

```text
request InProgress
-> cancellation receipt accepted
-> request Cancelling
-> typed cancellation propagated to exact service operation
-> owner/service responds with:
       CancelledBeforeEffect
       CancelledAtSafePoint
       CompletedBeforeCancellation
       EffectCommitted
       OutcomeUnknown
       CancellationUnsupportedOrTooLate
-> protocol result/error mapped by exact profile
```

A cancellation acknowledgement is not proof that no effect occurred.

## Domain effects

If the delegated service operation uses durable effects:

- preserve its `OperationId + CanonicalRequestDigest`;
- persist cancellation timing relative to durable receipts;
- reconcile response loss/uncertain effect;
- never blindly repeat or compensate;
- retain committed artifacts under their real state;
- expose `OutcomeUnknown` when effect status cannot be proven.

## Duplicate and late cancellation

The profile defines deterministic outcomes:

```text
first valid cancellation accepted
duplicate exact cancellation -> idempotent receipt
cancellation after terminal response -> TooLate/ignored-with-receipt
unknown request -> UnknownTarget without domain effect
reused transport ID -> resolve exact active/retained request record or reject ambiguity
```

No cancellation response is sent when the protocol models cancellation as a notification, but internal receipts are retained.

## Progress

Progress is request-scoped, bounded, optional telemetry. It contains no new domain truth.

```text
ProgressBegin
ProgressReport
ProgressEnd
```

A record binds session/request/binding, exact progress token, sequence, operation stage, bounded message/code/percent/count fields, and delivery state.

## Progress rules

- token creation/use follows exact negotiated profile;
- no progress without negotiated support;
- no source bodies, credentials, private paths, provider cursors, hidden holdout data, or unrestricted messages;
- percentages/counts are explicitly approximate or exact under profile;
- progress cannot change result identity, authority, severity, coverage, authorization, selection, completion, or exit state;
- progress order is monotonic by sequence;
- at most one terminal progress end;
- no progress after terminal response, shutdown close, or session exit;
- dropped progress never changes domain outcome;
- client acknowledgement, when supported, is telemetry only.

## Partial results versus progress

Partial result pages are typed immutable domain/protocol artifacts with exact continuation and cumulative budgets. Progress messages are nonauthoritative transient telemetry. They cannot substitute for each other.

A client that lacks partial-result support can receive a bounded final result or explicit truncation; the server does not stream fragments through progress fields.

## Response loss classes

```text
BeforeRequestAdmission
AfterAdmissionBeforeDomainDispatch
AfterDomainDispatchBeforeReceipt
AfterDomainReceiptBeforeProtocolSerialization
AfterSerializationBeforeWrite
PartialFrameWritten
AfterWriteBeforeDeliveryProof
TransportEOFWithInFlightRequest
```

Each class records exact session/request/binding/domain operation and available receipts.

## Recovery

### Before domain dispatch

If exact state proves no effect, the request can fail/cancel safely; a client retry is a new protocol request and may reuse the same domain idempotency identity as allowed.

### After possible domain dispatch

Query the existing service/owner reconciliation operation using the exact domain identity. Do not dispatch again while uncertain.

### Domain completed, response not delivered

Retain the exact result/artifact. A future explicit reconciliation request can retrieve it. The original protocol session/response is not recreated under a new request ID as if never attempted.

### Partial frame

Close/fail the transport according to framing profile; never append a second response to repair the byte stream. Preserve domain result and delivery uncertainty separately.

## Reconnect

New stdio process/session does not inherit request IDs or document state. It may invoke an allow-listed reconciliation/status service operation using exact durable domain operation identity and expected request digest.

No transparent protocol-session resume in E7-A.

## Shutdown interaction

During shutdown:

- stop new ordinary request admission;
- issue cancellation only under the exact drain policy;
- wait within bounded shutdown budget;
- retain unresolved effect state;
- emit shutdown response only after required session-level closure that precedes it;
- exit/EOF performs final synchronous close.

Shutdown timeout does not mark unresolved operations failed/no-effect.

## Resource limits

Bound:

```text
active requests
cancellations per request/session
progress tokens/messages/bytes/frequency
partial pages and cumulative bytes
reconciliation attempts
shutdown drain time/work count
retained uncertain requests/results
```

Exceeding progress limits can suppress further telemetry with a record; it cannot cancel or change the domain operation unless the explicit resource policy says so.

## Tests

- cancel before/after dispatch and after completion;
- duplicate/late/unknown cancellation;
- request ID reuse ambiguity;
- unsupported cancellation;
- progress begin/report/end ordering;
- progress after response/exit;
- private/source data in progress;
- partial result confused with progress;
- response loss at every class;
- partial frame write;
- reconnect with explicit domain reconciliation;
- shutdown with unresolved effects;
- no blind retry or false no-effect claim.
