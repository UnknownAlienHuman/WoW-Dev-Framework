# E5-C canary cohorts, observations, and evaluation

**Status:** normative.

## Purpose

Canary execution gathers bounded evidence for one exact published pack and execution profile before broader activation. It cannot prove ecosystem-wide or future runtime correctness.

## Cohort plan

`CanaryCohortPlan` binds:

```text
ValidatedInactive publication/artifact
execution profile and target WoW/project profile classes
cohort selection algorithm/version
exact membership list or privacy-preserving commitment plus authorized resolver
inclusion/exclusion and independence rules
maximum cohort size/fraction
start/stop/expiry and resource budgets
required observation schemas and minimum coverage
per-signal success/fail/pause/rollback criteria
authorization, privacy, license, retention and audit profiles
```

A bare percentage, random seed without population identity, repository popularity, owner identity, or “some users” is invalid.

## Assignment

`core_pack_canary_start` creates exact assignment records after authorization and validates that every member is eligible, retained, profile-compatible, and not duplicated across forbidden groups. Assignment does not update global/current/default records.

## Observation ports

```text
CanaryObservationPort
    append exact typed observation for an authorized assignment
    reconcile by operation/request/observation identity

CanaryObservationValidationPort
    validate schema, source adapter, profile, member, window, privacy, coverage and digest
```

No arbitrary logs, source bodies, SavedVariables contents, account/character data, process memory, shell output, free-form anecdotes, issue counts, stars, or model summaries are accepted as canonical signals.

## Signal families

Profiles may define bounded typed signals such as:

```text
recognizer execution completed/failed/cancelled
rule/case/proposal counts under exact denominator
unexpected diagnostic delta under exact baseline
producer partition validation state
resource ceilings
explicit user/operator rollback signal under authorization
client/runtime probe result under an exact approved adapter
```

Each signal states provenance, scope, denominator, coverage, conflicts, privacy, and nonclaims.

## Evaluation

Evaluate per signal first, then apply one frozen gate matrix. Missing required signal, insufficient denominator, partial/conflict/truncation, stale profile, unvalidated observation, or unavailable adapter yields `InsufficientEvidence`, `Conflict`, pause, or fail—not pass.

Conclusion states:

```text
Pass
Fail
Pause
InsufficientEvidence
Conflict
Cancelled
```

Aggregate averages cannot hide mandatory failures.

## Privacy

Cohort membership and observations are exposed only under exact consumer/audit profiles. Public/service envelopes use commitments, counts, stable IDs, and redacted summaries where required. The app never enumerates private cohort members unless explicitly authorized.

## Adaptive contamination

Canary results may influence rollout decisions, but their reuse to modify a future artifact creates a new candidate/submission/publication lineage. A changed pack cannot reuse prior canary evidence as if it tested the new bytes.

## Response loss and cancellation

Lost response after assignment or observation append becomes `OutcomeUnknown` until exact reconciliation. Cancellation stops new work but does not erase assignment/observation/audit records or assume no effect.

## Nonclaims

A canary pass does not establish all-addon compatibility, all-WoW-profile compatibility, absence of all false positives, Secret/taint/combat safety, performance under unobserved loads, publication integrity outside validated artifacts, or future-version correctness.