# E5-B retention, idempotency, response-loss recovery, and closure

**Status:** normative.

## Durable-operation classes

Effecting operations include:

```text
calibration_corpus_admit
calibration_run_submit
calibration_run_cancel
calibration_run_retry
calibration_candidate_build
calibration_review_record
calibration_holdout_request
calibration_holdout_execute
calibration_promotion_prepare
```

Read-only validation/status/get/list/explain operations still bind exact retained inputs but do not create domain artifacts unless an explicit audit profile requires a durable read receipt.

## Idempotency record

Before the first effect:

```text
OperationId
CanonicalRequestDigest
operation kind
exact resolved inputs
profile and budget IDs
expected prior state/guards
```

is durably registered.

Same ID/same digest:

- returns an existing terminal result;
- resumes from an explicitly resumable safe state;
- reconciles an uncertain owner effect;
- never duplicates an artifact/access/review/submission.

Same ID/different digest is rejected.

## Response-loss recovery

Every effecting port must supply one of:

```text
exact idempotent owner operation
query-by-operation-and-request identity
transaction receipt with exact artifact ID/digest
explicit no-effect proof
```

When none is available, the operation cannot be safely exposed as retryable.

Timeout, disconnect, serialization failure, or caller cancellation after dispatch does not prove that an effect did not happen.

## Recovery states

```text
NoEffectProven
EffectCommittedReceiptRecovered
EffectInProgress
EffectFailedWithProof
ConflictingEffectsQuarantined
OutcomeUnknown
```

`OutcomeUnknown` blocks duplicate dispatch and propagates as a public blocker.

## Run/candidate/submission identity

Artifact IDs derive from exact canonical inputs and owner outputs. They exclude retry count, process identity, wall-clock duration, host path, worker scheduling, and response delivery state.

An identical logical artifact produced by a safe idempotent retry has the same owner artifact identity; a new request/profile/input has a new identity.

## Retention graph

Before returning a durable handle, service closes retention over:

```text
operation record
source/project/analyzer/graph/fact publications
corpus/provenance/labels/split/pack/mutation profiles
run/case/metric/graph/security/determinism reports
candidate/deactivation artifact
review authorization/decision/audit records
holdout generation/grant/execution/audit/consumption records
promotion submission and target profile
```

Retention records include owner, reason, exact artifact, policy, and state. They cannot be approximated by timestamps or local references.

## GC race

Acquisition and retention ports must close the race between catalog lookup and garbage collection. Service cannot implement a lease with process-local memory alone.

If an artifact disappears before retention admission:

- close all remaining resources;
- do not return its handle;
- do not substitute another generation;
- return typed unavailable/retry state.

## Resource closure

Canonical acquisition/close order is package-defined. On success, failure, cancellation, panic boundary, owner error, serialization failure, or output error:

- release resources in exact reverse order;
- record every mandatory close result;
- do not return a success envelope before all mandatory closes finish;
- do not run detached cleanup;
- retain already committed owner artifacts under their exact states when close fails.

A close failure after successful work produces service failure with recovery references, not success plus warning.

## Holdout-specific recovery

A lost response around holdout access is conservative:

- access may have occurred;
- audit/consumption state must be reconciled;
- no second vault open/evaluation is dispatched while uncertain;
- inability to prove nonaccess prevents `UntouchedForCandidateLineage`.

## Review-specific recovery

Immutable review records are keyed by exact decision digest. Retrying after response loss returns the existing record. It cannot append a duplicate decision or consume quorum twice.

## Promotion-specific recovery

A submission build retry returns/reconciles the same immutable submission for the same request. It does not advance E5-C state, create a second submission to bypass blockers, or choose a newer candidate.

## Continuation

List/audit/explain operations may return snapshot-bound continuation. A cursor binds exact catalog/audit snapshot, filters, ordering, cumulative budgets, privacy profile, and last stable key. It does not resolve current or reset limits.

## Determinism

Equivalent exact inputs yield identical canonical operation, receipt, artifact-reference, review, audit, submission, and envelope bytes excluding explicitly noncanonical operational timings.

## Tests

Inject failures before/after each:

```text
operation registration
catalog acquisition
owner dispatch
owner commit
receipt persistence
authorization
vault open/evaluation/close
audit append
retention admission
resource close
response serialization
```

Prove no duplicate effect and no false success.
