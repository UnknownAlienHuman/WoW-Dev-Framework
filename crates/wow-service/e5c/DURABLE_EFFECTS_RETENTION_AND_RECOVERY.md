# E5-C durable effects, retention, response-loss recovery, and audit

**Status:** normative.

## Effecting operations

Publication, signature request, state transition, canary assignment/start, observation append, rollout advance/pause, activation, LKG designation, rollback, revocation, deactivation, project reindex, partition closure publication, and audit/retention actions are durable effects.

Before dispatch, register:

```text
OperationId
CanonicalRequestDigest
operation kind
exact artifact/publication/profile/current/plan targets
expected prior states/digests
authorization and budget profiles
```

## Idempotency

- Same ID/same digest returns or reconciles the same effect/result.
- Same ID/different digest fails.
- Already-completed exact effect is returned without redispatch.
- Retry never silently changes artifact, key, profile, cohort, stage, current base, rollback target, or budget.

## Effect receipts

Persist after each boundary:

```text
owner operation/request digest
exact effect/artifact/state/current/assignment/observation/reindex IDs
commit/no-effect/unknown status
response received state
validation and authorization state
reconciliation handle
retention/audit references
```

## OutcomeUnknown

A timeout, disconnect, cancellation, serialization error, or process loss after dispatch can yield `OutcomeUnknown`. While unresolved:

- do not repeat the effect;
- query the exact owner/provider by operation/request/target identity;
- validate recovered receipts and actual state;
- quarantine conflicting duplicate effects;
- preserve all evidence;
- expose exact recovery instructions/IDs.

If a required owner cannot reconcile, the operation remains blocked/unknown.

## Retention graph

Retain every artifact needed by publication/current/canary/rollout/LKG/rollback/revocation/audit:

```text
E5-A candidate and E5-B submission lineage
core artifact/attestations/signatures
catalog/publication/read-back reports
canary plans/assignments/observations/evaluations
rollout plans/stage receipts
current/LKG/rollback/revocation/deactivation records
project/graph generations and partition closure reports
authorization/effect/audit receipts
```

Current, active canary/rollout, LKG, rollback target, incident, review, evidence, and legal holds prevent GC.

## Resource lifecycle

Acquire exact catalogs/store views, authorization/signing/canary ports, owner views, retention/audit resources in package-defined order; close in reverse order. No public success before mandatory close results. No detached cleanup/background continuation.

## Startup recovery

Recovery scans only owned durable operation/catalog/current/audit records and exact registered owner reconciliation ports. It classifies:

```text
completed effect with missing response
registered but not dispatched
in-progress owner effect
conflicting duplicate
orphan PublishedInactive
validated inactive without canary
stale rollout/current/LKG record
partial reindex/partition closure
revoked active publication
```

Recovery never guesses from file timestamps or names and never activates an orphan.

## Audit

Append-only hash-linked events record operation, authorization, signing, publication, validation, assignment, observation, rollout, activation, LKG, rollback, revocation, deactivation, closure, denial, failure, cancellation, response loss, reconciliation, and retention state. Required trusted time/sequence evidence is profile-bound.

## Backup and restore

`wow-store` owns physical backup/restore. E5-C validation after restore checks catalog/object/current/LKG/audit/retention/project/graph closure. Restored physical bytes cannot create new semantic IDs or reactivate revoked records.

## Determinism

Canonical artifact, plan, receipt, state, and envelope identities exclude process ID, host path, wall duration, worker order, retry count, cache state, SQLite layout, WAL state, and response delivery timing.