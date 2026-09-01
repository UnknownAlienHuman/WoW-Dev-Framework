# E3-C leases, cancellation, retention, and resource closure

**Status:** normative lifecycle protocol.

## Resource classes

Conceptual resources acquired by E3-C:

```text
PrimaryPublishedProjectViewLease
OptionalPlatformPublishedProjectViewLease
ReferenceViewLease or immutable retained view guard
Context owner operation resources
Continuation generation-retention receipts
```

Concrete owner types remain private behind typed ports.

## Global ordering

Acquire:

```text
primary project -> optional platform project -> reference -> context invocation resources
```

Release:

```text
context invocation resources -> reference -> optional platform -> primary project
```

One order prevents peer deadlocks and nondeterministic behavior. Owners may use narrower internal ordering without exposing it.

## Partial acquisition stack

After every successful acquisition, push one private close action with stable owner/resource identity. On any later error or cancellation, unwind the stack in reverse order.

The public report contains only safe stable resource/receipt IDs and results, not callbacks, pointers, locks, connection IDs, file descriptors, paths, or transaction objects.

## Cancellation checkpoints

Check at least:

- before configuration/request validation;
- before and after each selector resolution/acquisition;
- before and after compatibility validation;
- before E3-B universe binding;
- before and after every context operation/validation/render call;
- before continuation retention admission;
- before envelope canonicalization;
- during close/unwind where owner operations support cancellation safely.

Once close starts, cancellation cannot skip mandatory release. It can be recorded as `CancelledDuringClose`; cleanup continues synchronously under bounded close policy.

## No background work

- no detached cleanup;
- no daemon lease renewal;
- no background continuation;
- no result returned while mandatory close is pending;
- no task survives process/service operation completion.

E7 transports may add an explicit long-lived session contract; E3-C does not.

## Success sequence

```text
execute and validate context result
-> build draft service outcome
-> admit exact continuation retention roots if a continuation is emitted
-> close ordinary operation resources in reverse order
-> validate ResourceClosureReport = Complete
-> finalize/canonicalize public success envelope
```

If continuation retention admission fails, the service may return the already-built artifact only if the operation profile explicitly permits a truncated result without usable continuation and E3-B emits an omission/continuation-unavailable record. It cannot advertise a broken continuation.

## Failure sequence

```text
record primary failure
-> stop new semantic work
-> unwind every acquired resource
-> collect bounded secondary close failures
-> build one failure result after unwind
```

The primary failure remains primary. Close failures are linked as secondary operational errors unless they independently indicate integrity/security compromise requiring stronger classification.

## Cancellation sequence

```text
observe cancellation
-> stop admitting new owner/context work
-> cancel bounded owner operation when supported
-> ignore/reject late results not part of a completed atomic owner call
-> unwind resources
-> return one cancelled result
```

No partial success envelope followed by cancellation.

## Close failure

Mandatory close failure means:

- no `complete`/`partial`/`truncated` public success envelope;
- return `ContextServiceFailureResult` with safe artifact IDs only when useful;
- do not expose source artifact bytes after privacy/lease closure uncertainty unless policy proves it safe;
- report exact owner/stage/code and unresolved resource identity;
- never hide as a warning.

## Continuation retention

A context continuation across service invocations requires exact retained roots for every referenced store generation/object according to E2-D policy.

Service records:

```text
continuation ID/request/universe/profile
exact generations
retention reason/reference
owner receipt IDs
admission state
release/expiry semantics
```

It does not implement store GC or lease tables.

## Continuing later

`context_continue`:

1. validates the bounded continuation object and original IDs;
2. reopens exact retained generations directly;
3. validates retention receipts/current owner state;
4. never resolves current;
5. binds the same ContextUniverseSet/profiles/total budget chain;
6. invokes E3-B continuation;
7. replaces/releases retention according to the returned continuation state;
8. closes ordinary resources before success.

Unavailable retained generation returns typed failure; no restart on current.

## Retention release

When continuation completes, is explicitly abandoned, expires under owner policy, or becomes invalid, service requests release through owner ports. Release is idempotent by exact continuation/receipt ID.

No context semantic artifact ID depends on operational expiration time.

## Panic/unwind boundary

Implementation must use scope guards/RAII and a top-level panic/invariant boundary appropriate to Rust policy. Panics are not normal errors, but every safely unwindable resource must still release. Abort-mode behavior must be addressed by owner crash/recovery tests and cannot be claimed clean closure.

## Broken transport/output

Applications can encounter broken pipe after service success. The service operation has already closed resources; application failure does not reopen or mutate the service result. App must not invoke the service operation twice automatically.

## Lifecycle tests

- failure/cancel after each acquisition stage;
- failure during context generation, validation, rendering, canonicalization;
- close failure at each reverse-order stage;
- cancellation arrives during close;
- continuation retention admitted/denied/replaced/released;
- continuation generation GC race blocked by owner retention;
- expired/unavailable continuation does not use current;
- late owner result after cancellation;
- panic/unwind scope guard behavior;
- no public success before close;
- no handle/source/private data leak in closure report.
