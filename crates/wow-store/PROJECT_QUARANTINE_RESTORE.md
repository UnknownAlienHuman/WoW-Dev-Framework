# Whole-instance restore from quarantine: contract version 1

**Status:** executable bounded W16 store/service checkpoint, verified on Windows
on 2026-10-09. Workspace checks and 901 tests pass (1 ignored, 107 targets).
The final capacity preflight passed 40 store tests plus refreshed workspace
policy/fmt/check/Clippy/rustdoc/build. Budget-exhaustion fault fixtures are
NotEvaluated.
The initial [root quarantine checkpoint](PROJECT_QUARANTINE.md) is separate.
This checkpoint does not close full W16/E2, migration, domain quarantine, cleanup, or
power-loss acceptance.

## Scope and preserved identities

An explicit `VerifiedBackup` may supply a new private physical instance to replace
the entire selected held instance. This operation does not repair or merge the
held database, select a last-known-good target implicitly, or release its hold.
The backup must match the held instance's exact EpochManifest, catalog, runtime,
and frozen SQL profile v1/v2/v3. Its original Epoch, generation, partition,
validation, and Current/history identities remain unchanged.

The source and target need not have identical generation sets or Current: the
target is the explicitly supplied backup, independently validated in full.
Unreadable source state cannot be reconstructed or claimed present in that
backup. The original held files and evidence remain retained.

Incompatible profiles/catalogs require a separate new-epoch migration contract.
This same-epoch physical replacement extends [PROJECT_REGISTRY](PROJECT_REGISTRY.md)
and preserves the [E2 restore requirements](e2/RECOVERY_BACKUP_RETENTION_GC.md).
`QuarantinedStore::stage_restore`, `reopen_restore`, and `activate_restore` own
the held-source path. `VerifiedBackup::finish_restore` owns isolated finalization.

## Versioned source guards

| Intent schema | Source authority | Required source guard |
| --- | --- | --- |
| `wow-store/project-replacement-intent/1` | Normal selected instance | Exact normal selector and live expected CurrentRecordId, or explicitly observed absence |
| `wow-store/project-replacement-intent/2` | Selected schema-3 quarantine | Exact quarantine/archive selector and hold reference, plus matching Current and evidence observations |

Schema 1 retains its existing encoding and expected-current CAS semantics. It
must reject a quarantined selector. Schema 2 is an explicit held-source guard;
it must not encode that guard as a missing schema-1 expected-current field.
Intent identity also binds the restore operation, unchanged epoch, and exact
target backup snapshot. A different target or source guard under the same
operation conflicts.

The held guard requires canonical admission of the selected
`wow-store/project-registry/3` record and its confined
`quarantines/<operation-derived>/selection.json`, `evidence.json`, and
`record.json`. Match exact selection and record bytes, evidence digest/length,
epoch, operation/request identity, and selector revision. A caller-supplied
receipt or serialized validation flag cannot replace these owner checks.

`CurrentObservation::Unreadable` may be retained in this exact held guard.
It never means absent Current, never authorizes an absence CAS, and never proves
a clean negative. An unparsed Pointer likewise remains a Pointer. Holding and
restoring a damaged SQL body does not require normal writable source admission;
the authority is the selected hold and its exact evidence. Compare the exact
schema-3 selector and archived original selector/record/evidence, and reobserve
Current and physical recovery evidence before staging and again before selector
dispatch. Changed observations or substituted authority reject.

Equal observations establish only equality of the captured pointer and evidence
observations. In particular, repeated Unreadable/unavailable observations cannot
prove that corrupt physical database bytes are unchanged. The held guard does
not claim a byte-for-byte source snapshot or successful source SQL validation.

## Target validation and selection

Verify the explicit backup and copy it into a new confined operation-derived
instance. Retain the main-root writer lock, candidate instance lock, and shared
reader admission lifetime. An existing candidate requires exact reconciliation,
not overwriting, deleting, or repeating the copy.

Before selection, independently validate target bytes, SQLite/header/schema/
runtime/catalog identity, every retained generation, complete membership and
partition/object closure, validations, history, and target Current. Compiled
domain owners must actually validate every target generation and provide their
exact read capabilities; a physical report does not grant domain approval.
Missing, invalid, incomplete, or cancelled checks block selection.

The target's original validated Current may be genuinely absent. That conclusion
comes from target verification and is independent of the source's Unreadable
observation. Never manufacture a predecessor or relabel backup identities.

Persist the exact intent, target record, and required archive closure before
publication. Recheck the held selector and source archive guards, then replace
the root selector once with the canonical normal schema-4 record. Canonical
readback and an independent complete target snapshot check precede normal owner
adoption. After dispatch, classify the effect without caller cancellation or
implicit rollback. Preserve the old instance, WAL/SHM, evidence, locks, and held
snapshots; existing snapshots stay on their original instance.

## Normal registry schemas 4/5 and flat archive authority

`wow-store/project-registry/4` selects the restored normal instance while
retaining finite sorted exact quarantine references. Later normal replacements
carrying these references also use schema 4. It binds the
unchanged epoch, replacement intent, checked monotonic selector revision,
confined target instance, request and owner-validation identities, and target
Current at selection time. Later ordinary publication may advance live Current
without rewriting that historical observation.

An isolated `finish_restore` carrying quarantine authority publishes the root normal
wrapper `wow-store/project-registry/5`: it has no physical instance directory.
The wrapper binds the exact EpochManifest, restore operation, backup snapshot
digest, owner-validation digest, target Current, and finite sorted quarantine
references. Its selector revision is derived with checked arithmetic and must
be strictly above the greatest referenced hold revision. It has no instance ID;
its database uses the root's confined `epochs/<unchanged-epoch-hash>/` location,
without inventing instance-directory replacement evidence.

The schema-5 wrapper and transported archives must admit before exposing the
isolated normal owner. Its Current is the validated target observation at
finalization, not an inferred source pointer. Both normal formats preserve all
SQL, epoch, generation, partition, validation, and Current identities; wrapper
revision and archive references do not enter those SQL identities.

Each quarantine reference identifies one immutable schema-3 hold and its exact
selection/record/evidence archive. The declared set includes the restored hold,
its inherited dependencies, and any authority carried by the target backup.
Equal references deduplicate canonically; an identical reference identity with
different bytes is a conflict. No reference grants deletion permission.

Admission must enforce these flat-closure requirements:

- Implemented bounds are at most 32 archive records and 64 MiB aggregate archive
  bytes, including archived selection, record, and evidence bytes. Individual
  sizes and open handles also have compiled finite bounds. Overflow or exhaustion
  rejects admission, never drops a reference or truncates evidence.
- Every referenced archive exists under the confined root and matches its exact
  canonical metadata and digests. Missing or substituted evidence rejects.
- Every archived normal selection's dependency references are a subset of the
  admitted complete set. A dependency's hold revision is strictly earlier than
  that archived normal selector's revision, which precedes its referring hold;
  each hold also precedes the selected normal record.
- Validate each archive against that flat set. Do not recursively call registry
  admission through archived selections or construct a recursive history tree.
  Self references, cycles, future revisions, and undeclared dependencies reject.
- Preserve original archive revisions and identities; do not renumber old holds
  to make an incompatible reference set admissible.

Legacy raw EpochManifest and valid schema-1 intent/schema-2 normal-registry bytes
remain readable unchanged. Introducing archived holds requires the explicit
schema-4 instance record or schema-5 isolated-root wrapper; no legacy decode
silently discards or invents that authority.

## Authority through replacement, backup, and isolated restore

Later normal replacements must preserve all inherited quarantine references and
admit any additional target-backup references under the same flat bounds and
revision rules. They use normal live-Current guards, not the historical held
guard. Selecting another instance never erases prior holds.

Newly staged normal replacements persist `replacement-source.json` alongside
the intent. These are the original normal selector bytes, matched to the intent's
expected selector digest during stage. Activation and exact published-intent
adoption shallowly reconstruct the original epoch, selection, and reference
closure from this durable file. They read its actual source archives and merge
them with the verified target archives; they never infer source authority from
the newly selected record.

Normal admission checks an existing source sidecar even for empty/schema-2
records; retained normal replacements require it. Genuine legacy schema-2
records without the new sidecar keep their original admission behavior.
The held-source route reconstructs its original closure from the exact immutable
quarantine archive's `selection.json` and schema-3 record.
Schema-4 normal admission checks that this shallow source selection and epoch
match the intent, and that every original source reference is a member of the
selected sorted reference set. Omitting inherited source authority rejects.
Archived selectors are decoded as data; no recursive history admission is used.

Backup manifest `wow-store/project-backup/2` binds the exact reference set and its
complete archive closure alongside the committed SQLite payload. `open` and
`verify` import only the reference descriptors, independently validate their
archives, reconstruct the body/recovery/manifest identities, and compare exact
canonical manifest and intent bytes. An inline partition closure
alone does not preserve external quarantine authority. Independent reopen and
isolated restore must verify and reproduce that authority before exposing a
normal owner. A valid SQL body is insufficient when required archives are absent.
Archive transport is shallow and data-only: preserve the exact selection,
record, and evidence bytes, validate their flat references, and do not recursively
materialize history. The original corrupt SQL database and its sidecars are not
copied by archive transport. They remain held at their original location.
The verified target backup supplies the restored SQL body separately. Transported
archives preserve identities; absolute roots remain private runtime inputs.

With no retained references, backup manifest 1 omits the added field and uses
the original snapshot/request digest paths, preserving v1 wire bytes and IDs.
An isolated restore with no references retains the original raw EpochManifest
registry; the schema-5 wrapper is used only when archive authority is present.

## Reconciliation, gates, and open acceptance

Exact durable reconciliation must distinguish an unpublished candidate, the
identical selected operation, and conflicting or later selections. An identical
published intent may be adopted only after repeat physical and all-owner checks,
without another selector rename. Response loss is not proof of failure or
permission to retry effects; receipt acknowledgment remains Unknown.

Required executable gates cover unreadable and malformed source pointers,
exact hold/evidence/selector guards, target substitution and missing owner checks,
flat dependency subsets/revisions/bounds, inherited authority through successive
replacement/backup/isolated restore, schema-5 root admission without an instance,
legacy bytes, cancellation, uncertain readback, and old snapshots/files/locks.
Observation equality must not be reported as unchanged corrupt source bytes.
Five new store regressions pass: malformed-pointer restore preserves old
leases and hold authority through publication, normal replacement, backup and
source-independent isolated restore; stale hold/evidence, omitted owners and
archive substitution reject with exact staged-intent reconciliation; unreadable
header restore retains unavailable evidence and uses the held guard; repeated
holds survive target replacement, source omission and source-independent portable
roundtrip; real Windows selector-sharing refusal preserves the hold and exact
selected adoption performs no second selector effect. The native service replays
all target Project/Graph generations while old leased pairs remain readable.
Workspace policy, fmt, check, strict Clippy, tests (901 passed, 1 ignored,
107 targets), strict rustdoc and build passed. After final archive-capacity
preflight, refreshed workspace gates and 40 store tests passed; the broad suite
was not repeated. This does not establish full W16/E2 acceptance or unexecuted
budget-exhaustion/platform faults.

Durability is limited to the measured file-sync, selector replacement, and
readback behavior. No directory-flush or power-loss guarantee follows. Full
migration, fine-grained/domain quarantine, cleanup/deletion, and full W16/E2
acceptance remain open; no automatic repair, background continuation, or silent
last-known-good selection is permitted.
