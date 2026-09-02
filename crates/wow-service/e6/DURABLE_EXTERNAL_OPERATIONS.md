# E6-B durable external operations and response-loss recovery

**Status:** normative.

## Effect classification

Operations are classified before dispatch:

```text
ReadOnlyProviderObservation
ReadOnlyCandidateQuery
ProviderReadWithServerSideReceipt
LocalArtifactPersistence
LocalAuditOrRetentionEffect
OwnerMappingRead
ContextBuildEffect
```

The classification and reconciliation requirements are profile inputs. “Query” is not assumed side-effect-free merely from its name.

## Registration

Before the first provider/store/context effect:

```text
OperationId
CanonicalRequestDigest
operation kind
exact provider/descriptor/adapter/profile
external-state request/binding
exact owner publication/profile when applicable
privacy/license/budget/cancellation profiles
expected prior durable state
```

is atomically registered through `DurableExternalOperationPort`.

Rules:

- same ID + same digest resumes/reconciles or returns the recorded result;
- same ID + different digest is rejected;
- a completed exact result is returned without re-execution;
- operation IDs are not reused across operation kinds;
- no effect occurs before registration succeeds.

## Canonical query lifecycle

```text
validate request and profiles
-> register durable operation
-> authorize/acquire provider session
-> validate descriptor/capabilities/external state
-> persist dispatch-intent receipt
-> invoke wow-cbm/E6-A query through session adapter
-> persist provider response/effect receipt
-> validate and normalize result through E6-A
-> persist immutable result/artifact
-> admit catalog and retention records
-> close session/resources
-> finalize canonical service envelope
```

## Durable states

```text
Planned
SessionAcquiring
Registered
Dispatched
ResponseReceived
Normalized
Persisted
Cataloged
RetentionAdmitted
Completed
NoChange
Cancelled
Failed
OutcomeUnknown
Quarantined
Superseded
```

Transitions are append-only or guarded CAS. Missing rows/output do not imply a state.

## Response loss

Inject response loss after every boundary:

```text
authorization decision
session acquisition
external-state observation
dispatch-intent persistence
provider dispatch
provider response
E6-A normalization
artifact persistence
catalog admission
retention admission
mapping owner call
selection record
context operation
resource close
service serialization
```

When an effect may have happened but no authoritative receipt is available, state becomes `OutcomeUnknown`.

While `OutcomeUnknown`:

- no blind provider/store/context repeat;
- no `Failed`, `Cancelled`, `NoChange`, or `Complete` guess;
- query exact provider/store/owner operation receipt by operation/request identity;
- validate recovered artifact/receipt before accepting it;
- quarantine conflicting duplicate effects;
- preserve audit and recovery refs.

## Provider reconciliation

```text
reconcile_provider_effect(OperationId, CanonicalRequestDigest, dispatch receipt)
```

returns:

```text
NoEffectProven
EffectCommitted(receipt/result refs)
EffectInProgress
EffectFailedWithProof
ConflictingEffects
OutcomeUnknown
Unsupported
```

`Unsupported` after a potentially effecting dispatch leaves the service operation `OutcomeUnknown`.

## Local persistence reconciliation

Artifact/store ports must support exact lookup by operation/request and expected artifact digest. If response is lost after commit, retry returns the same immutable artifact. Duplicate differing artifacts are quarantined.

## Mapping and context reconciliation

Owner mapping reads should be immutable read operations. If an owner creates durable receipts, service reconciles by exact request. Context build follows the existing E3-C idempotency contract and does not receive a new root on retry.

## Cancellation

Cancellation is checked before and during every stage. It:

- requests typed cancellation where supported;
- records exact last durable state;
- does not prove provider effect absence after dispatch;
- closes resources synchronously;
- starts no background continuation;
- cannot return `Complete`;
- preserves immutable artifacts already committed under their real state.

## Retry operation

`external_candidate_operation_reconcile` is not a generic rerun. It accepts exact prior operation/request identity and may:

- return existing completed result;
- reconcile provider/local effects;
- resume only from an owner-declared safe durable boundary;
- return `OutcomeUnknown`/blocked;
- never change provider, query, state binding, owner publication, profiles, privacy, or budgets.

A new request requires a new operation ID.

## NoChange

`NoChange` requires exact owner/store proof that the canonical requested artifact/result already exists and matches every input/profile/digest. Empty provider output, cache miss, skipped call, unavailable capability, or same display text is not proof.

## Resource acquisition order

Canonical superset:

```text
1 durable operation record
2 provider descriptor/catalog
3 credential authorization
4 provider session and external-state binding
5 external artifact catalog/store
6 exact project/reference owner view for mapping
7 context owner view when requested
8 retention/cache/audit resources
```

Acquire only needed resources while preserving order. Close in exact reverse order.

## Public success gate

Before success:

- every effect receipt persisted and validated;
- exact artifact IDs/digests known;
- required retention admitted;
- audit events appended;
- provider/owner/context resources closed;
- close receipts validated;
- canonical envelope serialized.

A close/retention/audit/serialization failure after useful work returns failure/`OutcomeUnknown` with recovery refs, not success plus warning.
