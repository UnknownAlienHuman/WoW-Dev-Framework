# Migration-ready artifact preparation

**Status:** implemented and verified at the private immutable-artifact boundary on
2026-10-09. Full W16/E2 acceptance remains open.

The migration contract itself is [PROJECT_MIGRATION](PROJECT_MIGRATION.md), which
produces a validated inactive target. This document covers the private preparation
step that follows it: reconstruct the target's mapped pins and its target-local
Current inside a private root, then freeze the result as a nested immutable backup.

Preparation establishes a private target selector and target-local Current; it never
selects a live epoch. The original migration baseline, source archive and live registry
stay unchanged
throughout, and the baseline and source archive are re-verified rather than trusted.

## Owner boundary

Preparation consumes the completed target export. `MigrationPreparation::stage`
([ready.rs](src/project/migration/ready.rs)) re-verifies the held migration and the
export, requires the export's epoch and snapshot digest to equal the migration's
target epoch and baseline snapshot, and requires the export to have no current and no
retained quarantine references. It then writes the exact intent before any mutable
handoff.

`MigrationPreparation::create` creates the baseline export once and then stages it.
The service's `prepare_live_project_migration` and
`resume_live_project_migration_preparation`
([migration.rs](../wow-service/src/live_project/migration.rs)) replay every retained
Project/Graph pair before obtaining fresh native owner checks and finishing preparation.

`MigrationPreparation::open` and `finish` reconcile one original request by admission
alone. They re-read the durable intent, require the exact planned request digest, the
same operation, and canonical intent bytes, then reconstruct the intent from the held
baseline and compare. Exactly four observed states are opened and reconciled:

- a completed copy handoff, reopened as an export with its exact copy operation and
  baseline snapshot, requiring unchanged manifest bytes and acceptable sidecars;
- a private root holding a partial subset of the planned roots, opened as a live store
  under the original catalog, then admitted against the closed native plan;
- a private root whose mapped Current is already activated, admitted by verifying the
  operation, request, generation, validation and singleton activation history, then
  skipping activation rather than re-running it;
- a completed final artifact, reopened with the deterministic output operation and the
  reconstructed ready snapshot, then independently verified.

An earlier `VerifiedBackup::verify` is deliberately not applied to an already-mutable
registry, because the copy manifest is historical metadata and the live root is
admitted against the closed native plan instead.

An incomplete native copy is retained and refused with an explicit incomplete or
uncertain outcome. There is no recopy, no rebuild and no overwrite: a partial native
copy with no complete canonical manifest cannot be reopened at all, and a final output
directory that exists without its canonical manifest returns an explicit unknown
outcome rather than being regenerated.

## Reconstructed inputs

The intent binds exact identities: the preparation operation, the migration request
and receipt digest, source and target epoch manifests with the original catalog, the
source archive snapshot, the baseline snapshot, the exact copy binding, the complete
target generation set, the complete mapped root set, the optional Current plan, and
the deterministic final output operation.

Source pins are reconstructed rather than copied. Each source root keeps its own ID,
kind and holder, while its epoch and generation are replaced through the complete
migration map, so the target pin digest is recomputed rather than relabelled. A
missing mapping rejects, and distinct root IDs stay distinct even when their
generations converge. A source profile without a retention table admits an empty root
list, but a profile that has one rejects a missing or unreadable root rather than
treating it as empty.

The Current plan is explicit and target-local. With no source Current, the plan is
absent and no activation happens. With one, its source generation resolves through
the map to the representative operation, and the expected target Current is computed
from the target manifest with no predecessor, because migrated requests carry no
expected Current. The source Current record ID is never used as a target CAS, and no
source history record is ever relabelled as target history.

## Admitted deltas

The closed inventory validator compares the observed root against the unchanged
baseline record by record: every generation manifest, partition, validation and
operation must match, and no extra generation, partition, operation, release, policy
or GC receipt is permitted.

Only two deltas are ever admitted. An exact subset of the planned roots may be
present, matched by root ID and value, and substituted same-ID roots or extra roots
reject. The single selected operation may transition from `ValidatedInactive` to
`Activated`, with the planned target Current and a singleton target history. An
observed activation additionally requires the complete planned root set, so a
half-pinned activation cannot pass. The absent-current plan and the activated plan are
each admitted only with their exact expected shape.

Owner checks are always fresh. `validate_checks` requires exactly one check per
planned generation, re-reads each generation, and recomputes its record against the
target catalog. Stored phase labels or a durable owner digest never substitute for a
compiled owner verdict.

## Immutable artifact and interruption

`finish` freezes the prepared root into a fixed confined `ready-artifact` output
directory using the ordinary native backup route, so the artifact is independently
verifiable. The receipt binds the intent, the artifact snapshot, its copy binding, and
the fresh owner validation digest, and its acknowledgment is unknown.

Before effects, `finish` requires the saved intent and historical copy metadata to
remain exact; a Working body also requires its inner epoch manifest. Existing final
output must have a canonical manifest, match the complete planned inventory and pass
independent backup verification. Any existing ready record must equal the reconstructed
canonical receipt before mutable handoff. An absent record after a completed copy can
be finalized; a foreign record rejects without repair.

Cancellation or an uncertain commit retains the intent and the observed state, and
progress resumes by admission of the original request rather than by blind retry.

## Not claimed

Cross-epoch selector selection, live-root activation, whole-source portability, and
power-loss acceptance are out of scope, as are deletion or rewriting of archived
history and source state. Implemented scope does not establish complete migration,
W16/E2, or domain quarantine acceptance.

## Verification checkpoint (2026-10-09, Windows)

Three native store cases cover genuine partial mapped-root continuation, target-local
Current, independent immutable artifact reopen, exact completed retry without recopy,
wrong request/foreign pin/foreign final-record refusal, and missing output/inner
manifest preflight without SQL or selector mutation. The extended native service
lifecycle preserves actual Project/Graph IDs, original Current, old readers and the
inactive baseline, and reconciles an identical receipt with fresh owner replay.

Repository policy, fmt, workspace check, strict Clippy, tests (914 passed, 1 ignored,
107 targets), strict rustdoc and workspace build pass. The ignored consumer gate,
arbitrary interrupted native-copy recovery, inside-write/power-loss/platform faults,
cross-epoch selection, portable full migration history and Gethe/Ketho/client acceptance
remain NotEvaluated or separately open.
