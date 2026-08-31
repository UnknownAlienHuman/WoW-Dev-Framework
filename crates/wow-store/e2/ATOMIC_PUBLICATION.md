# E2-D atomic publication-set activation

**Status:** normative activation protocol after generation seal and validation.

## Goal

Publish one indivisible relationship:

```text
StoreGenerationId + StoreImageId
ProjectGenerationId + ProjectSnapshotId
GraphGenerationId + GraphSnapshotId
exact project/graph validation attestations
```

There is no separately advancing project pointer, graph pointer, or analyzer/recognizer pointer.

## Publication-set eligibility

Before activation, require:

- exact sealed generation exists at final path;
- image/member/checksum/store integrity report accepted;
- logical manifest equals publication request/plan;
- exact project attestation accepted under policy;
- exact graph attestation accepted under policy;
- attestations refer to the same image, candidate, schema/query bundles, and generations;
- all required source/analyzer/recognizer manifests close;
- no fatal coverage/conflict/truncation/security/budget blocker;
- operation idempotency record is `DomainValidated` or equivalent;
- cancellation not requested before CAS;
- expected current publication-set identity supplied;
- no registry recovery/GC state blocks activation.

## PublicationSet construction

Construct noncyclic canonical layers:

```text
sealed generation + logical/image manifests
-> domain attestation manifest
-> project/graph snapshot manifest IDs
-> PublicationSetId
-> PublicationSetManifest
-> activation request
```

The manifest cannot hash a receipt or registry epoch that is created after it.

## Activation request

```text
ActivateProjectPublicationSetRequest
    publication operation/request digest
    ProjectPublicationKey
    expected current PublicationSetId: optional exact empty state for initial publish
    target PublicationSetManifest/ID
    exact sealed generation/image
    project/graph attestation IDs
    activation policy ID
    durability/cancellation/budget
```

No `force`, `latest`, or silent-rebase flag.

## Registry transaction

One writer transaction performs:

1. load and validate store/configuration/profile state;
2. verify operation ID/request digest/idempotency state;
3. verify target generation is sealed, final-path-valid, nonquarantined;
4. verify manifest and attestation closure;
5. read current pointer and compare exact expected identity/epoch policy;
6. insert or verify target publication-set manifest;
7. record predecessor relationship;
8. mark target generation `Active`;
9. mark predecessor `SupersededRetained` or keep active history according to policy;
10. replace current pointer with target set;
11. insert activation receipt;
12. finalize idempotency record as `Activated`;
13. commit under frozen durability profile.

No domain database write occurs in this transaction.

## Initial publication

Initial activation explicitly expects no current set. Another concurrent initial publisher causes one CAS winner; the other fails stale-current and remains sealed inactive.

## Replacement publication

Expected predecessor must equal registry current. A stale publisher cannot overwrite a newer current set, even if its target generation is otherwise valid.

## Repeated activation

```text
same operation ID + same request digest + target already current/active
    return existing receipt after closure validation

same request target active but another operation ID
    classify as exact-already-active or no-change according to policy; do not create another semantic set

same operation ID + different digest
    reject idempotency conflict

target set exists but current advanced beyond it
    return stale/already-superseded state; never reactivate silently
```

## Crash outcomes

### Before registry commit

Current remains predecessor. Target remains sealed inactive/recoverable.

### During registry commit

SQLite atomicity yields old or new committed registry state. Recovery validates transaction and target file closure.

### After commit before response

Target is current. Retry reads idempotency record and returns the exact existing receipt. It must not rebuild or activate again.

### Pointer references missing/corrupt target

This is an integrity incident. Do not silently point to another generation. Block current reads for that set, create recovery/quarantine record, and require explicit validated rollback or republish operation.

## Snapshot manifest finalization

Project/graph snapshot IDs may be derived before activation from logical candidate plus exact store generation/image and validation attestation. Their publication status becomes active only through the publication-set manifest/pointer; immutable snapshot identity does not change on activation.

If domain contracts require an activation receipt reference, store it in the registry/publication metadata layer, not inside a self-hashing snapshot structure.

## Last-known-good

The predecessor can be retained as an explicit `LastKnownGood` root if it passed required prior gates. Rules:

- predecessor identity stays unchanged;
- failed target recorded separately;
- rollback is a new explicit CAS operation selecting an exact existing validated publication set;
- rollback does not mutate or relabel the old generation;
- a target failure never automatically reports predecessor as target success.

## Rollback activation

An explicit rollback request names:

```text
current expected set
exact validated retained target set
rollback policy/reason/evidence
```

Registry CAS activates the existing set and creates a new activation receipt/history edge. The `PublicationSetId` itself remains the same exact immutable set; activation sequence/history is operational metadata.

## Partial publication policy

A `PartialCandidate` may activate only when all are true:

- project and graph attestations classify exact partial scopes;
- publication policy explicitly permits those scopes;
- no mandatory storage/security/integrity/identity gate is partial;
- current pointer and receipt label the set partial;
- consumers receive exact coverage/blockers.

Storage never interprets which domain capability is safe to degrade.

## Concurrency

- one registry writer serializes activations;
- generation builders may run concurrently under distinct operation/staging IDs if resource policy permits;
- builders cannot reserve/assume a future current pointer beyond expected-current input;
- active readers on predecessor continue through their exact leases after replacement;
- retention keeps predecessor while leases/pins apply.

## Activation validation tests

- initial two-writer race;
- stale replacement race;
- identical retry before/after response loss;
- operation ID digest conflict;
- mixed project/graph attestation;
- attestation against staging path or other image;
- target final bytes change after attestation;
- cancellation immediately before CAS;
- crash at each registry statement/commit boundary;
- current pointer corruption/missing generation;
- explicit rollback with old readers active;
- partial candidate allowed/disallowed policy;
- last-known-good never relabeled.

## Hard stops

- no multi-pointer activation;
- no last-writer-wins;
- no force update bypassing expected current;
- no activation from staging/quarantine;
- no attestation reuse across image/generation;
- no hidden fallback to predecessor;
- no pointer repair without explicit validated recovery/rollback;
- no post-cancel background activation.
