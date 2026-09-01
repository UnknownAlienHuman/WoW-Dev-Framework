# E5-A shadow partitions, supersession, and deactivation

**Status:** normative lifecycle contract.

## Ownership

Every candidate rule execution owns an exact shadow producer partition:

```text
CalibrationShadowPartitionKey
    pack_id + pack_version + canonical_pack_digest
    rule_id + rule_version
    exact input fact-snapshot/generation/partition
    matcher implementation/profile
    graph registry/profile
    run profile
```

Repository, owner, addon, path, split display name, wall clock, worker ID, temp path, and database row ID are not partition identity.

## Partition contents

```text
CalibrationShadowOutputPartition
    partition key
    exact match/proposal/evidence/coverage/conflict manifests
    graph validation results
    status and completeness
    resource/cancellation report
    canonical digest
```

The partition is evaluation evidence only. It is not a published `GraphSnapshot`, does not satisfy default core coverage, and cannot be queried as production truth.

## Lifecycle states

```text
Prepared
ShadowComplete
ShadowPartial
ShadowConflict
ShadowCancelled
ShadowFailed
Quarantined
Superseded
Disabled
```

Only `ShadowComplete` can contribute complete case results. It still carries `calibration` trust and `Derived`/`Possible` output ceilings.

## Replacement

A changed pack/rule/input/profile creates a new partition key. The new run does not overwrite the old partition. A versioned supersession record connects the exact old and new identities and states why comparison is or is not valid.

No last-write-wins, newest-first, branch-name, filesystem-order, or wall-clock selection is allowed.

## Disable and rejection

Disabling/rejecting a candidate produces an explicit empty replacement or tombstone for each exact owned active shadow partition, plus a deactivation report. It must not:

- delete historical case/metric evidence needed for reproducibility;
- mutate E2 core partitions;
- mutate another calibration pack's partitions;
- relabel prior output as core/default;
- retain stale candidate assertions as active shadow coverage;
- claim project-wide absence from removed calibration coverage.

## Deactivation plan

```text
CalibrationDeactivationPlan
    exact candidate/pack/rule/version/input identities
    owned active shadow partition IDs
    stale proposal/assertion/result/reference IDs
    remove/retain/tombstone action per object class
    expected coverage downgrade by role/rule/scope
    unaffected core and foreign partition digests
    validation operations and expected receipts
    blockers/nonclaims
    canonical digest
```

## Reference closure

The plan traces references from:

```text
run -> case results -> matches/proposals -> shadow partitions
candidate artifact -> reports -> partition IDs
comparison artifacts -> old/new run IDs
future E5-B submissions -> candidate artifact IDs
```

An object required to reproduce an accepted historical decision is retained or replaced by a policy-compliant tombstone. Sensitive data may be redacted according to privacy/license policy while stable non-sensitive identity and decision history remain.

## Coverage downgrade

Removing a candidate pack can only reduce candidate/shadow coverage. It cannot:

- reduce or alter core pack semantics;
- turn `Unknown`/`NotEvaluated` into Negative;
- establish that a role does not exist;
- raise another pack's confidence;
- automatically activate a fallback pack.

The exact affected rules, roles, scopes, examples, and reports are listed.

## Graph boundary

`wow-recognizers` produces graph proposals and graph-validation receipts. It does not own graph publication. E5-A deactivation therefore proves removal of recognizer-owned shadow artifacts and references; any future published core-pack rollback is an E5-C operation over immutable graph/project publication contracts.

## Idempotency and response loss

Deactivation validation is pure and repeatable. A later orchestration layer may use exact operation ID plus request digest for durable effects, but E5-A defines no durable effect owner or implicit retry.

## Validation

Fail when:

- an owned partition cannot be enumerated exactly;
- foreign/core objects appear in the removal set;
- stale active output remains reachable after disable/supersede;
- historical decision evidence is destroyed without an allowed tombstone;
- coverage downgrade is omitted or overstated;
- deactivation depends on current/latest, names, paths, or storage row order;
- repeated validation changes semantic output;
- privacy/license requirements are violated.
