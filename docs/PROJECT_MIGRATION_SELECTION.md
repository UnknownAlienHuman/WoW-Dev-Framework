# Guarded cross-epoch selection

**Status:** implemented, stopping at a selected, adopted live epoch. Two native store
cases and one native service case pass. Workspace policy, fmt, check, strict Clippy,
tests (919 passed, 1 ignored, 107 targets), strict rustdoc and build passed on
2026-10-09. Implemented scope is not accepted completeness.

Migration [PROJECT_MIGRATION](../crates/wow-store/PROJECT_MIGRATION.md) stops at a validated inactive
target, and [PROJECT_MIGRATION_READY](../crates/wow-store/PROJECT_MIGRATION_READY.md) stops at an immutable
ready artifact. This owner installs that prepared artifact as the live epoch of the
same root. Same-epoch physical replacement stays in
[PROJECT_REGISTRY](../crates/wow-store/PROJECT_REGISTRY.md). The implementation is
[migration/selection.rs](../crates/wow-store/src/project/migration/selection.rs), with
[selection/model.rs](../crates/wow-store/src/project/migration/selection/model.rs) and
[selection/io.rs](../crates/wow-store/src/project/migration/selection/io.rs).

## Entry points

`stage_ready_selection` persists an intent and then copies the portable target once.
`reopen_ready_selection` reconciles exactly one original request. `activate_ready_selection`
publishes the selector and adopts the held target. `migration_selection_receipt` reads
historical installation evidence for one operation and re-dispatches nothing. The service
wrappers on `LiveProjectStore` forward to these and add no behavior of their own, except
that `activate_ready_selection` replays every target pair and collects the owner
capabilities the store then rechecks.

A candidate exposes only `request_digest`, `target_generations` and `read_generation`.
Decoded selection evidence cannot construct one.

## Stage and the WAL handoff

Stage runs every admission before the first directory effect: it re-guards the source,
re-verifies the ready artifact, captures the live source authority set, builds the
intent, and captures the source, migration and ready evidence. The ledger and instance
directories are both required to be absent, so an existing artifact conflicts rather
than being overwritten.

The copy is produced by the ordinary export owner, so the native database it writes is
already a WAL store. `working_from_backup` then turns that artifact into the working
owner. Before any WAL configuration is touched it writes the exact target epoch manifest
and the durable `migration-selection-working.json` marker, and only then connects writable
and runs `enable_writer`. The marker records the selection request digest, the target
epoch identity and the portable snapshot, so a later run can tell a handed-off working
body from a bare copy.

The candidate therefore holds a private, unselected `ProjectStore` rather than an
immutable backup. Reads through the candidate are live reads of that working body, and
`verify_target` additionally requires the marker to be present and the working root to
have no selector.

## Ledger and instance

The ledger under `migration-selections/<id>` holds the canonical intent plus the source,
migration and ready sidecars, each bound by the intent's own evidence bindings. The
instance under `instances/<id>` receives the same sidecars alongside the copied database.

Reading the ledger is data-only. `io::read` validates the intent, checks every sidecar
against its binding, and admits the archived source selector as a shallow normal
selector under the source epoch's catalog. It hydrates no history, opens no SQL and
confers no capability.

The ledger describes an instance; it is not the instance. Reopen requires both
directories, both `io::read` calls, and a recomputed intent that equals the stored one.

## Reopen distinguishes two complete bodies

When the working marker is present, `open_working_target` treats the instance as an
already-handed-off working body. It re-admits the root, requires the selector to still be
absent, compares both epoch manifests, checks the database and sidecar sizes, acquires
the instance writer lock under the held root lock with shared reader admissions, opens
the database readonly, validates the header, and captures the full native snapshot.
That snapshot must match the planned generations, the expected current and the portable
snapshot before any writer configuration runs, so a foreign or partial body is refused
while the artifact is still untouched.

When the working marker is absent but the copy is otherwise complete, reopen goes through
`VerifiedBackup::open` and then the same handoff, which writes the marker before
configuring WAL.

There is no path that reconstructs a missing sidecar, recopies a partial instance,
replans a fresh intent, or deletes an interrupted artifact. A partial native copy is
retained and refused.

## Activation

`activate_ready_selection` first checks that the candidate belongs to this root and
shares the reader-admission cell. It re-reads the instance sidecars, re-verifies the
target, and computes the owner digest over the actual generations, which replays each
generation and rejects a set that differs from the planned generations. Target owners
therefore validate before any effect.

The two published states are distinguished. An uncommitted selection requires the
working body, re-runs the original source guard, propagates the admitted source
authorities to the root, and stages the outer selector. A committed selection requires
the already-selected read owner, re-reads the record, and performs no second rename and
no SQL activation.

Between the final source guard and the single selector rename there are no writes and no
cancellation branches. The rename is one OS call, followed by an independent read of the
selector and migration record, adoption of the working target database, and a fresh read
whose independent snapshot must equal the intent's portable snapshot. The adopted
database keeps the working owner's lifetime, which already carries the inherited root
lock, the instance lock and the shared reader admissions established when the marker was
written.

## Receipt

`MigrationSelectionReceipt` is serialize-only and reports the operation and request
digests, the previous and selected selectors, both epochs, the activated current, the
portable snapshot digest, and an acknowledgment that starts as Unknown.

`migration_selection_receipt` reads only the historical selected record for one
operation, conflicts on a digest mismatch and returns `None` otherwise. A later
legitimate publication does not rewrite it, and it never compares mutable SQL against the
installation snapshot. A quarantined root rejects lookup with `Quarantined`.

## Old readers and interruption

Old physical roots and their readers keep their own connection and lease, and the
previously staged target snapshot keeps its leases too. The shared root lock is inherited
rather than reopened, and reader admissions are shared rather than reset, so the
process-wide reader bound holds across selection.

Ordinary interruption recovers through the exact explicit reopen, which reconciles the
one staged request instead of inferring a newest state. Acknowledgment stays Unknown
after a lost response, and durability is limited to the file-synced selector replacement
and read back, with no power-loss claim.

## Not claimed

Arbitrary partial-copy recovery, recopy or deletion of an incomplete artifact, power-loss
durability, cleanup of superseded instances, and installing a target under a different
owner or catalog are out of scope. The passing workspace gates do not establish full
migration, W16/E2, or domain quarantine acceptance.
