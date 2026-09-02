# E6-B retention, idempotency, recovery, and resource lifecycle

**Status:** normative.

## Durable operations

The following may create durable or externally observable effects:

```text
provider/session acquisition when owner classifies it as effecting
provider query dispatch and quota/state consumption
provider response receipt
result/artifact publication and validation
mapping publication
selection receipt publication
context/sidecar/combined-result publication
cancellation, retention, audit, reconciliation, and quarantine
```

Each effect binds exact `OperationId + CanonicalRequestDigest` and exact targets.

## Idempotency

- same operation ID and request digest returns/reconciles the same effect;
- same operation ID with another digest fails;
- already-completed exact effect is not redispatched;
- continuation is a new operation linked to the prior result/continuation;
- a caller-authorized fresh query after an unreconcilable unknown operation is a new operation, never a retry of the same observation;
- mapping, selection, and context operations each have independent idempotency domains.

## Effect receipts

Persist at least:

```text
operation/request/configuration/session/provider/state/query IDs
owner operation kind and exact target
prepared/dispatched/committed/no-effect/unknown state
response receipt state
result/mapping/selection/context record IDs
validation and close state
reconciliation handle
retention/audit references
```

## Retention graph

Retain while directly or transitively referenced:

```text
provider configuration/descriptor/capability/state bindings
credential-use authorization and nonsecret session receipts
query request, dispatch, raw response and normalized E6-A result
result/artifact catalog and validation records
continuation lineage and cumulative budgets
owner mapping request/result and retained owner view evidence
selection requests/receipts/supersession chain
context universe/artifacts/sidecar/combined result
privacy/license/redaction decisions
audit/reconciliation/quarantine records
```

Active continuations, unresolved `OutcomeUnknown`, context artifacts, selection/mapping references, incidents, legal/privacy/license holds, and evaluation fixtures prevent GC.

## Acquisition order

Default ordering:

```text
durable operation registry
-> provider configuration catalog
-> credential authorization
-> provider session/transport
-> result/artifact catalog and store resources
-> project/reference/graph owner views for mapping/context
-> context resources
-> retention
-> audit
```

Acquire only what the operation needs. Close in reverse order. Package-specific operations may refine ordering but cannot create cycles or publish success before mandatory close.

## Response-loss recovery

Recovery queries only exact registered owners/ports and durable records. It classifies:

```text
registered but not dispatched
session acquired but query not dispatched
dispatched with provider outcome unknown
provider response received but result not published
result published but not read-back validated
mapping effect unknown
selection effect unknown
context publication partial/unknown
conflicting duplicate effect
orphan retained result/artifact
revoked configuration/authorization referenced by active continuation
```

Recovery never infers state from timestamps, files, process presence, result similarity, provider count, or cache entries.

## Provider reconciliation capability

If a provider/session adapter supports exact operation-key reconciliation, service validates the recovered provider receipt against operation/request/session/state/query bindings.

If it does not, a post-dispatch timeout remains `OutcomeUnknown`. Service cannot call the same provider again and declare one of the responses canonical by newest/majority/equality.

## Startup behavior

Startup recovery may finalize missing catalog/read-back/audit closure for already-known exact effects when safe, or quarantine/block. It cannot dispatch new provider queries, select candidates, perform mapping, or build context without an explicit operation request.

## Cancellation

Cancellation is durable intent. It prevents new stages but does not erase already-observed provider effects or records. Late provider responses are discarded, retained as reconciliation evidence, or quarantined according to the adapter contract; they are never silently accepted as a successful cancelled operation.

## Backup/restore

`wow-store` owns physical backup/restore. E6-B post-restore validation checks object/catalog identities, result/artifact/mapping/selection/context references, audit/retention chains, and unresolved effects. Restore cannot create new semantic IDs, refresh provider state, or reactivate revoked configurations.

## No background work

No detached query, mapping, context build, cleanup, retry, or polling continues after return unless a future E7 daemon contract explicitly owns a durable job. E6-B current operations finish, cancel, or return `OutcomeUnknown` synchronously.