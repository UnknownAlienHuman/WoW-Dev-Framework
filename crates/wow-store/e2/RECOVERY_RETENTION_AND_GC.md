# Recovery, retention, and garbage collection

**Status:** normative.

## Recovery inventory

Recovery enumerates only the owned store root through the validated host adapter. It classifies registry, generation, staging, quarantine, and object entries without following links or opening arbitrary databases.

For each generation:

- locate and validate manifest/checksum members;
- compare registry head/history/lease/root references;
- verify sealed/open state when safe;
- classify transaction/seal/publication phase;
- record exact corruption or ambiguity;
- propose a typed action.

## Recovery actions

```text
discard_uncommitted_staging
quarantine_ambiguous_or_corrupt_staging
revalidate_sealed_inactive
adopt_validated_inactive_via_coordinator
retain_current_or_rooted
mark_unreachable_for_gc
restore_registry_from_reviewed_backup
manual_intervention_required
```

No action edits a sealed database.

## Sealed inactive adoption

Requires:

- exact original publication bundle and generation manifests;
- all checksums/open validation;
- exact project and graph post-open validation;
- expected current head still matches;
- no newer incompatible target;
- explicit coordinator authorization;
- fresh CAS.

A physically valid store is not enough.

## Last-known-good

Last-known-good is an explicit root to an original coherent head/generation. It may equal current. It never assumes the target's identities or fills missing target partitions.

## Retention roots

Mandatory root classes:

```text
current-head
last-known-good
explicit-pin
active-reader-lease
recovery-in-progress
quarantine-for-investigation
evidence-or-regression-reference
backup/export-in-progress
```

Optional policy roots are versioned and explicit.

## Mark phase

Traverse:

```text
root -> head -> store generation -> project/graph/domain manifests
     -> object-reference manifests -> content-addressed objects
     -> retained validation/evidence artifacts
```

Missing/cyclic/inconsistent reference closure blocks deletion and creates an integrity finding.

## Sweep plan

A `GcPlan` is built against exact inventory, head registry, lease registry, and root-set snapshot. Before deletion, revalidate all preconditions and obtain the GC writer/maintenance lock.

Delete/quarantine in safe order:

1. unreferenced generation artifacts;
2. unreferenced manifests/support files;
3. unreferenced objects after a second reachability check;
4. empty owned directories.

Current registry/head is never rewritten by sweep.

## Cancellation/failure

GC is restartable and reports every member. Partial deletion never causes a retained generation to lose a referenced object. If object reachability becomes uncertain, retain.

## No age-only GC

Age may prioritize candidates after nonreachability proof. It cannot establish nonreachability.

## Compaction

Database vacuum/repack/physical optimization occurs only while constructing a new sealed artifact. Never compact a published generation in place. Logical equality and physical difference are reported separately.
