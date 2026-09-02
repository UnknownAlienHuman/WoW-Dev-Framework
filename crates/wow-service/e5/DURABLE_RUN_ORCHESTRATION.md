# E5-B durable calibration run orchestration

**Status:** normative orchestration over E5-A owner operations.

## Run classes

```text
VisibleSplitShadowRun
VisibleSplitMutationRun
VisibleSplitEvaluationRun
AuthorizedSealedHoldoutRun
CandidateBuildRun
CandidateValidationRun
DeactivationValidationRun
```

Each run has one exact immutable request and durable operation identity.

## Submission protocol

```text
validate outer request
-> create/read durable OperationId + CanonicalRequestDigest record
-> acquire exact retained artifacts
-> validate compatibility and visibility
-> build fixed owner-operation plan
-> invoke E5-A owner operations in declared order
-> validate every owner result
-> persist exact receipts and immutable result references after each effect
-> evaluate operation-level status conservatively
-> admit retention/audit records
-> close resources in reverse order
-> finalize canonical service envelope
```

Service never implements matching, mutation generation, expected-label comparison, metric arithmetic, graph validation, or candidate construction.

## Durable states

```text
Planned
Acquiring
ValidatingInputs
AwaitingAuthorization
Authorized
Executing
Evaluating
BuildingArtifact
ValidatingOutput
RetentionPending
Completed
NoChange
Cancelled
Failed
OutcomeUnknown
Quarantined
Superseded
```

Transitions are append-only or compare-and-swap under the registered store contract. A state is not inferred from missing output.

## Idempotency

Canonical key:

```text
OperationId + CanonicalRequestDigest
```

Rules:

- same ID and same digest resumes/reconciles or returns the exact recorded result;
- same ID and different digest is rejected;
- completed exact result is returned without re-executing;
- failed/cancelled retry follows an explicit profile and creates no duplicate owner effect;
- operation IDs cannot be reused across operation kinds;
- current selector resolution is part of the recorded exact request before effects.

## Owner effect receipts

After every effecting owner call, persist:

```text
owner operation ID
exact owner request digest
owner artifact/result IDs and digests
owner durable/effect state
response received state
validation state
reconciliation token/receipt
```

The service does not mark an effect absent because the response was lost.

## `OutcomeUnknown`

Enter `OutcomeUnknown` when an owner/store/vault call may have committed an effect but no authoritative receipt was obtained.

While unresolved:

- do not repeat the effect;
- do not report failed, cancelled, no-change, or complete as a guess;
- query the owner using the exact operation/request identity;
- validate any recovered receipt/artifact;
- quarantine conflicting duplicate effects;
- preserve all evidence and audit events.

If the owner lacks reconciliation capability, the operation remains blocked and the missing capability is explicit.

## Run cancellation

Cancellation is checked before and during acquisition, E5-A calls, graph validation, metric evaluation, persistence, retention, serialization, and closure.

Cancellation:

- signals owners through typed cancellation;
- stops only at owner-defined safe points;
- persists any exact durable intermediate artifact under its real state;
- does not relabel partial work as a completed run;
- closes resources synchronously;
- starts no background continuation;
- can advertise continuation/retry only with an exact retained state contract.

## Run retry

`calibration_run_retry` accepts an exact prior operation/run and a frozen retry profile. It may:

- return an already completed result;
- reconcile `OutcomeUnknown`;
- resume from an owner-declared safe durable boundary;
- start a new operation only when the prior state/profile explicitly requires a new run identity.

It may not silently update corpus, split, pack, implementation, thresholds, labels, or holdout generation.

## Visible split runs

Visible Train/Dev/Test/Challenge runs acquire only authorized visible artifacts. `SealedHoldout` is excluded unless an exact holdout grant is supplied through the dedicated holdout operation path.

## Owner call plan

Typical visible evaluation:

```text
validate candidate source/corpus/split/pack/fact snapshot
-> run_calibration_pack_shadow
-> run_calibration_mutation_suite
-> evaluate_calibration_pack
-> compare_calibration_runs when explicitly requested
-> build/validate candidate artifact when requested
```

Actual operations are request/profile-specific. Service cannot skip a required hard gate based on aggregate metrics.

## Result validation

Validate:

- exact request/input/output identity closure;
- owner status and coverage/conflicts/omissions;
- per-case results before aggregate metrics;
- graph-validation receipts;
- mutation, leakage, security, determinism, and deactivation hard gates;
- no hidden holdout use;
- no confidence/negative-authority/generalization upgrade;
- canonical digest and retention closure.

## NoChange

`NoChange` is valid only when the exact owner/store artifact already exists and matches the entire canonical request. An empty result, skipped operation, no candidate, or unavailable capability is not no-change.

## Response-loss tests

Inject loss after every durable boundary:

```text
operation registration
artifact acquisition
owner execution
owner artifact publication
metric report publication
candidate publication
retention admission
response serialization
```

Retry must return/reconcile the exact effect without duplication.
