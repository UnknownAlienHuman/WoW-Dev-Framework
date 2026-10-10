# Project root quarantine: contract version 1

**Status:** executable initial W16 quarantine checkpoint, verified on Windows
on 2026-10-09. This checkpoint covers quarantine selection,
typed inspection, and read-only observation. Guarded restore, cleanup, and
migration are not implemented by this checkpoint; platform durability remains
unaccepted.

## Authority and scope

A schema-3 outer root-registry quarantine record holds the entire selected
physical instance. Its authority does not depend on successfully decoding Current,
generation membership, or a SQL retention row. Frozen SQL profiles v1/v2/v3,
the original EpochManifest, and Epoch/Generation IDs remain unchanged.

This extends the logical quarantine location described in
[E2 IDEMPOTENCY_AND_RECOVERY](e2/IDEMPOTENCY_AND_RECOVERY.md): the root registry
provides the authoritative hold when damaged database state cannot safely carry
or expose that decision. It preserves the E2 requirements to block normal
admission, retain evidence, and avoid resuming a quarantined subject as trusted.
No database repair, schema migration, or path move is required to establish the hold.

The quarantine record binds the original physical selection and epoch, raw
Current observation, operation/request identity, and recovery evidence through
versioned canonical bytes and digests. Serialized recovery or validation flags
cannot manufacture owner approval. Absolute roots remain private runtime inputs;
physical locations follow the confined owner rules in [PROJECT_REGISTRY](PROJECT_REGISTRY.md).

## Initial quarantine checkpoint

Quarantine binds the exact preceding normal registry selector and recovery
evidence. The confined archive is `quarantines/<operation-derived>/` and retains:

- `selection.json`: the exact original normal selector bytes.
- `evidence.json`: the exact recovery evidence bound by the quarantine request.
- `record.json`: the versioned quarantine record and exact receipt identity.

The selected schema-3 registry binds this archive. Establishing the hold requires
the exact original normal selector, raw Current observation, evidence, and
operation/request guards; substitution or stale state rejects. Failure or
response loss requires reconciliation against the exact selected record and
archive, rather than blind retry. The hold does not rewrite the selected database
or establish a new normal publication.

## Standalone inspection of a damaged instance

`QuarantineInspection::open(root, &catalog, stop)` admits the canonical normal
registry, exact epoch manifest, and confined physical files independently of
normal writable database admission. It retains the root and selected-instance
writer locks and attempts bounded read-only SQL observation. A damaged SQL body
or rejected header can therefore be held without opening a normal writable
`ProjectStore`.

When SQL connection or header admission prevents observation, inspection retains
`CurrentObservation::Unreadable` and unavailable physical evidence with the
failure code. Failed evidence does not become pointer absence, a successful
recovery report, or domain approval. Cancellation remains an error.

`inspection.quarantine(&operation, stop)` explicitly selects the authoritative
schema-3 hold. It rechecks the original normal selector, Current observation,
and exact evidence before dispatch, preserving the immutable archive described
above. An already-selected exact receipt reconciles without a second selector
rename. After selection, observation uses the separate `QuarantinedStore` owner;
this path provides no cleanup, migration, or restore capability.

## Raw Current observation

`CurrentObservation` distinguishes:

- `Absent`: the pointer row was observably absent.
- `Pointer`: the raw pointer has an exact digest and length, with an optional
  parsed `CurrentRecordId` when parsing succeeds.
- `Unreadable`: the pointer could not be observed under the bounded read.

Pointer parsing does not validate the referenced publication or its closure.
A pointer without a parsed ID remains a pointer; it is never rewritten as
absence. Unreadable state never authorizes an absence CAS or a clean negative.
Physical recovery reports retain integrity failures and incomplete scopes rather
than converting missing observations into validated domain state.

## Admission and observation

Once quarantine is selected, normal new reads, writes, generation activation,
replacement activation through normal guards, GC, and backup reject with
`StoreErrorCode::Quarantined`. Existing normal owners must honor the root hold
for new operations rather than continuing on a cached normal selector.
Quarantine does not grant cleanup or deletion permission.

Typed registry inspection identifies the selected quarantine and its exact
receipt. The separate typed `QuarantinedStore` exposes bounded read-only physical
recovery observation and the exact quarantine receipt. It may acquire the existing owner
locks, but does not expose a writable SQLite handle, normal ProjectView, domain
approval, or a repair API. Its receipt identifies the hold and operation; physical
recovery observations do not authorize normal publication.

Preserve original files, WAL/SHM, Current/operation/validation evidence, and held
reader lifetimes. Already-held snapshots stay usable, bound to their original
instance, and continue to apply their ordinary immutable-record checks; no reader is silently
redirected to a repaired or last-known-good publication.

## Next work: explicit guarded restore from quarantine

This section specifies follow-up requirements, not an implemented restore API.
The initial quarantine checkpoint has no cleanup, migration, or restore path.

A future restore must select a new private physical instance explicitly.
Require the exact quarantine selector, original pointer observation and
evidence guards, operation/request binding, and independently validated target.
An ordinary expected-current guard cannot substitute for the raw observation
when its pointer is malformed or unreadable.

Before selection, validate target bytes, SQLite/profile/schema/catalog identity,
the complete retained generation/membership/partition/object closure, and all
compiled-owner checks. Preserve the target's original semantic identities and
validated Current/history. Do not merge restored and held data or fabricate
missing attestations. Partial or incomplete validation cannot release normal admission.

The resulting normal registry must use a versioned replacement-intent guard
binding the archived immutable quarantine record. This transition selects the new
instance; it does not erase the old hold, release its files, or authorize GC.
Retain the shared root writer lock and all old reader/file lifetimes across the
guarded selector switch. Recheck the exact selector and original observation/
evidence guards before publication; reject substitution or stale state.

Failure and response loss must be reconciled against exact durable records,
without blind retry, automatic rollback, or silent repair. Archived quarantine evidence must
remain available after successful restore. No deletion, automatic last-known-good
selection, background continuation, directory-flush guarantee, or power-loss
claim follows from this checkpoint. See [E2 restore and retention](e2/RECOVERY_BACKUP_RETENTION_GC.md).

## Executed scope

Seven store regressions cover malformed/raw-unreadable Current, damaged history
and payload, stale evidence, cancellation, exact operation reconciliation,
archive substitution, old leased snapshots and root locks, pointer shape/budgets,
and a separately selected physical instance with absent Current. Normal new reads,
publication/activation, planned GC and backup refuse the hold. A real Windows
handle denying selector replacement produces `OutcomeUnknown`; original Current
survives and the exact staged intent succeeds only after explicit reconciliation.

The native service regression publishes two actual Project/Graph pairs, rejects
a stale inspection, selects a hold, preserves both old pairs and independently
reopens the read-only owner. A physically valid report does not release the hold.
Thin service wrappers expose `LiveProjectQuarantineInspection` and
`QuarantinedLiveProject` with exact current and frozen legacy catalog admission.

Workspace policy, fmt, check, strict Clippy, tests (895 passed, 1 ignored,
107 targets), strict rustdoc and build pass. A final diagnostic-only CLI adjustment
passed focused strict Clippy/build; repository policy/fmt are checked afterward.
Guarded restore, incompatible-epoch migration, fine-grained/domain quarantine,
interruption inside writes/OS calls, power loss, cleanup, source/Ketho/runtime and
full W16/E2 acceptance remain open.
