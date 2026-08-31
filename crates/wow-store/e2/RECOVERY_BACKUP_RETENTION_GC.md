# Recovery, backup, retention, and garbage collection

**Status:** normative E2-D operational state contract.

## Startup recovery scan

Under one writer/recovery owner:

1. validate the outer epoch registry;
2. open the selected epoch with the recovery profile;
3. inspect DB/WAL/SHM and SQLite recovery result;
4. validate schema/runtime/epoch manifests;
5. read and validate the current record;
6. enumerate building/inactive/validated/failed generations;
7. verify partition/object/reference closure;
8. classify actions without changing current implicitly.

## Recoverable inactive generation

It may resume only when exact epoch/schema/runtime/profile, generation/publication/partition/object manifests, owner contracts, and expected base still match. Otherwise retain for evidence/quarantine or GC.

## Current corruption

- report explicit degraded/corrupt state;
- do not auto-repair rows or recalculate IDs;
- do not silently choose last-known-good;
- allow explicit rollback to a retained validated publication through a new CAS transaction;
- preserve failed current evidence where safe;
- rebuild from source when trust cannot be restored.

## Backup

Use the selected SQLite online backup/snapshot API or another probed correct mechanism. Manifest exact epoch/current/generation set, DB digest/length, schema/runtime profile, object closure, validation report, and original identities.

Never copy the main DB alone while committed WAL frames may be absent from it.

## Restore

Restore creates a candidate epoch or private recovery path, validates files/objects/schema/current/generations/domain queries, preserves original semantic IDs, and activates the outer registry explicitly. It cannot relabel a backup.

## Retention policy

Mandatory pins:

```text
current publication
configured last-known-good
active readers
explicit comparison/debug
evidence/task references
validated inactive awaiting activation
recovery/quarantine policy
```

Optional count/size policy cannot remove mandatory pins.

## Generation GC

Within one writer transaction:

1. snapshot current/retention/lease state;
2. select only `eligible=yes` generations;
3. remove membership/publication/generation rows through registered operations;
4. retain current and all pins;
5. compute unreachable partition versions;
6. execute owner domain partition-delete catalogs;
7. verify schema/FK/reference/current closure;
8. commit;
9. produce object GC candidates from committed reference state.

Unknown state means no deletion.

## Partition and object GC

Partition versions delete only when no retained membership, evidence/debug/recovery pin, lease, or owner reference remains and all domain rows/indexes can delete atomically.

Object deletion occurs after authoritative committed DB reference accounting and a final recheck. Shared objects require explicit global ownership; path coincidence is not evidence.

## Epoch GC

An old epoch deletes only when not selected, unleased, unpinned for rollback/evidence/backup/recovery, fully reference-closed, and path-safe.

## Rebuild

ProjectStore is derived. Rebuild from exact materialized source/profile/tool inputs is preferred over speculative repair and creates a new generation or epoch through normal gates.

## No age-only deletion

Time may prioritize candidates but cannot prove reachability or eligibility.
