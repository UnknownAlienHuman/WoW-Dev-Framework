# Project registry v2: guarded physical-instance replacement

**Status:** executable bounded W16 slice, verified with native Windows fixtures.
Full W16/E2 acceptance, guarded recovery from quarantine, incompatible-epoch migration and actual
power-loss durability remain open.

## Scope and E2 exception

The outer `project-store-registry.json` selects one physical database instance.
Ordinary generation publication remains inside that instance, using the E2
current-record CAS; it does not replace the registry.

This contract extends the [E2 outer registry model](e2/DATA_MODEL.md). A restored,
independently validated private instance may replace the selected instance while
retaining the **exact original EpochManifest and EpochId**. This is a physical
selection change, not a new semantic epoch or an in-place schema migration.
Incompatible changes still require the new-epoch procedure in
[PHYSICAL_MODEL](e2/PHYSICAL_MODEL.md).

## Registry formats and identities

`RegistryRecord` uses schema `wow-store/project-registry/2` and contains the
unchanged `EpochManifest`, `ReplacementIntent`, revision, instance ID, request
digest, owner-validation digest, and `activated_current`. The latter records the
target's Current at replacement time; later generation publication can advance
live Current without changing this historical observation.

`ReplacementIntent` uses schema `wow-store/project-replacement-intent/1` and
binds operation ID, original expected `RegistrySelection`, expected live
`CurrentRecordId` or explicit absence, unchanged EpochId, and target backup
snapshot digest. Its canonical bytes determine the `project-replacement-request`
digest. Owner validation has a separate `project-replacement-owners` digest
binding that snapshot and the exact generation-to-validation-ID map.

`RegistrySelection` binds the `project-registry` digest of the complete canonical
registry bytes, EpochId, revision, and optional instance ID. That digest is
derived, not a self-referential field in `RegistryRecord`. Physical-instance IDs
and selector revisions never enter or rewrite Epoch, Generation, partition,
project, or graph IDs.

The legacy raw EpochManifest remains decodable under its exact original
catalog/profile and legacy location rule: revision zero, no instance ID, and
`epochs/<epoch-hash>/project.sqlite`. Reading it does not rewrite its bytes or
widen its catalog. V2 revision is the expected revision plus one, checked for
overflow; its unchanged epoch remains independently admitted under the catalog.

The separate [quarantine schema 3](PROJECT_QUARANTINE.md) holds the preceding
selected physical instance. `RegistrySelection` then includes an operation-derived
quarantine marker. An absent marker is omitted, preserving all valid legacy/v2
bytes. Normal replacement intents reject a quarantine selector; recovery from
that state requires a separately versioned guard and remains follow-up work.
Absent expected/activated Current fields are omitted under strict canonical JSON;
existing valid `Some` encodings remain unchanged.

The instance ID is the operation-derived `project-instance` digest's 64 lowercase
hex characters. The owner resolves only this confined layout:

```text
<private-root>/instances/<instance>/epochs/<unchanged-epoch-hash>/project.sqlite
```

Absolute roots remain private runtime inputs. Validated directory components
cannot introduce arbitrary paths, traversal, or symlink/reparse escapes. A native
candidate is a `VerifiedBackup` at the instance root, with its own `writer.lock`.
Staging and reopening also retain the original main-root writer lock as an
auxiliary lock in the candidate lifetime.

## Replacement guards and validation

`ProjectStore::registry_selection()` observes the exact selector.
`stage_replacement(backup, operation, expected, expected_current, stop)` checks
both source guards and copies the verified backup into a new operation-derived
instance; it refuses an existing instance. `reopen_replacement` requires the same
original guards, operation, and snapshot digest and exact persisted intent,
without repeating the copy. It accepts either the unchanged original base or
the exact already-selected intent observed by the stale original owner after an
uncertain readback; a different request or later selection remains a conflict.

`activate_replacement(candidate, checks, stop)` consumes the held candidate and
compiled-owner capabilities. It requires the exact main-root instance location,
operation, snapshot, and unchanged epoch, and rechecks selector and live Current
before dispatch. Epoch equality alone cannot authorize a different physical
selection. Ordinary stale guards reject; exact published-intent adoption is the
explicit reconciliation exception, not a rebase.

On that adoption path, the original owner's selection and old Current must still
match the intent's original guards. Physical verification and all compiled-owner
checks run again, and the reconstructed selected record must be byte-identical
to the published record. Activation then adopts that instance without another
staged-selector write or rename, while old leased readers remain alive.

The target must match the original epoch, runtime, physical profile, schema and
catalog exactly. Validate its bytes, committed SQLite closure, every retained
generation and membership, partition/object closure, original identities, and
compiled-owner checks before selection. Partial, invalid, incomplete, cancelled,
or serialized claims of successful validation do not authorize replacement.
These requirements implement [E2 restore](e2/RECOVERY_BACKUP_RETENTION_GC.md)
and preserve [E2 publication](e2/PUBLICATION_PROTOCOL.md)'s exact-base guard.

Replacement selects the target's original validated Current/history; it does
not manufacture a predecessor, merge live and restored partitions, relabel
semantic IDs, or infer domain approval from physical recovery observations.

## Publication, readers, and reconciliation

Persist `replacement-intent.json` and `replacement-record.json` at the instance
root. Registry admission requires their canonical bytes to match the selected
record. For first activation, write and file-sync the main-root
`project-store-registry-<instance>.staged`, then recheck the source guards and
cancellation. One `fs::rename` replaces the root selector, followed by canonical
readback of the exact selection and record. No remove-then-rename gap or retry
loop is part of activation. After selector readback, an independent held
connection captures the complete physical/logical snapshot again and requires
its digest to equal the original target snapshot before switching the owner.
Caller cancellation is not consulted after dispatch; this final observation
runs even if cancellation arrived during replacement.

Retain the candidate's instance lock alongside the shared main-root writer lock.
Preserve old database files, WAL/SHM, manifests, and reader lifetimes; held reads
stay on their original instance. New reads follow the selected instance, and an
idle-owner guard rejects a stale selector. Replacement grants no cleanup or
deletion authority over either instance. Main-root, staged/reopened candidate,
and selected-instance reads share one `reader_admissions` counter across the
switch; replacing an instance does not reset the aggregate reader bound.

For an unpublished candidate, cancellation before dispatch leaves the original
selection intact; cancellation never reverses an already-published selection.
Rename/readback failure reports `OutcomeUnknown`; reconciliation may explicitly reopen and adopt
only the exact published intent as described above, never blindly redispatch
the rename or roll back.
`replacement_receipt(operation, request_digest)` observes only the currently
selected replacement. A later selection returning no matching receipt does not
prove the earlier operation never committed; its instance evidence remains.
Receipt acknowledgment is `Unknown`. Durability is limited to file-sync, selector
replacement, and readback: no directory flush or power-loss guarantee is claimed.
No merge, relabeling, schema migration, automatic repair, last-known-good
substitution, or background continuation is permitted.

## Executed scope

Seven native store regressions exercise preserved semantic IDs and old readers,
root-wide reader admission, stale source/current/operation guards, missing owner
checks, mutated target payloads, cancellation, explicit staged/unknown-result
reconciliation, successive physical instances and ordinary publication after
replacement. A Windows handle denying selector replacement produces
`OutcomeUnknown`, preserves source Current, and permits only explicit exact
reconciliation after the handle closes. The native service replays both retained
Project/Graph pairs while newer old-instance readers remain coherent.

A native test process is forcibly terminated after committed preparation,
staging, owner validation and replacement before acknowledgment. Independent
reopen observes the exact durable boundary; staged targets are explicitly
revalidated without copying again, and committed replacement retains unknown
acknowledgment. These probes do not terminate inside a transaction/OS call or
certify power loss, hostile OS access or interrupted cleanup.

Workspace policy, fmt, check, strict Clippy, tests (887 passed, 1 ignored,
107 targets), rustdoc and build passed on 2026-10-09. Gethe/Ketho/runtime and full
package acceptance remain separate gates.
