# E6-B durable query operations and immutable result catalogs

**Status:** normative.

## Registration

Before provider/session dispatch, persist:

```text
OperationId
CanonicalRequestDigest
operation kind
exact provider configuration/descriptor/query/profile selectors
expected external-state/session policy
credential-authorization scope
budget/cancellation/privacy/license profiles
```

Same operation ID and digest reconciles the same operation. Same ID with another digest fails.

## Query lifecycle

```text
Registered
Authorized
SessionAcquired
Dispatched
ProviderOutcomeUnknown
ProviderResponseReceived
ResultRecorded
Validated
Partial
Truncated
Cancelled
Failed
```

States are append-only/superseding durable records. The original request, provider receipt, raw response digest, normalized result, and failures are immutable.

## Dispatch receipt

`ExternalProviderDispatchReceipt` binds exact operation/request, session, descriptor/capability/state, transport operation, request bytes/digest, dispatch state, provider operation key when available, quota/budget consumption evidence, response-delivery state, and reconciliation capability.

A successful transport write does not prove a response was recorded. A received response does not prove result publication. A catalog publication does not prove validation.

## Result publication

```text
validate bounded provider receipt/raw response
-> invoke E6-A normalization/validation
-> prepare immutable result-set object
-> publish result catalog record through wow-store
-> admit retention
-> close write resources
-> reacquire fresh read snapshot by exact result ID
-> validate bytes/digest/schema/configuration/state/query/authority closure
-> persist validation receipt
```

There is no current/default result pointer. Lists never select a result for mapping, selection, or context.

## Artifact build

`external_candidate_artifact_build` accepts one exact validated result and an explicit caller-supplied candidate-ID subset or all-candidates request allowed by bounded profile. It invokes E6-A artifact construction and publishes a distinct immutable artifact. Service cannot choose top/best/sole candidates.

## Get and list

Get uses exact IDs. List binds an immutable catalog snapshot, exact filters, deterministic ordering, page budgets, continuation, consumer/privacy profile, and last stable key. Lists are discovery only and cannot establish selection or latest/current semantics.

## Query continuation

Continuation requires:

```text
exact prior query/result/continuation
same configuration/descriptor/capability/state/query/output profiles
same authorized consumer and privacy/license scope
valid session/state reacquisition policy
cumulative item/byte/time/quota budgets
prior partial/truncation/loss state
```

Continuation publishes a new immutable page/result record linked to the prior manifest. It does not mutate the prior result or reset budgets.

## Cancellation

Cancellation records intent and asks the exact session/transport owner to stop new work. It does not prove the provider did not execute or return a result. If dispatch may have completed, reconciliation remains required.

## OutcomeUnknown

Enter `OutcomeUnknown` when any provider dispatch, response receipt, result publication, retention, or catalog confirmation may have occurred without a trustworthy final receipt.

While unresolved:

- do not redispatch;
- query exact session/provider/store owner by operation/request/effect identity;
- preserve possible quota/state effects;
- quarantine conflicting duplicate receipts;
- return explicit recovery state.

If the provider cannot reconcile a read query, the old operation remains unknown. The caller may later authorize a new explicit operation, but it is not a retry of the old observation.

## NoChange

`NoChange` requires proof that the exact same operation/request already produced the same retained validated result/artifact. Equal query text, provider result count, candidate bytes, or display name is insufficient.

## Retention

Retain configuration/authorization/session receipts, query/dispatch/raw response, E6-A result/artifact, validation, continuation lineage, mapping/selection/context references, audit, and reconciliation state while referenced or under policy hold.

## Determinism

Canonical service records exclude wall duration, host/process, network latency, retry counter, cache hit, worker order, physical row/object key, and response delivery timing. E6-A canonical normalized bytes remain authoritative for candidate payload identity.