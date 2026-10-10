# ProjectStore recovery and verified backup

Updated 2026-10-09. This W16 slice implements read-only physical reconciliation,
native verified backup, isolated restore and guarded same-epoch physical-instance
replacement, explicit quarantine and guarded restore retaining portable hold
authority. Store fixtures and native Project/Graph replay pass. Fine-grained quarantine,
incompatible-epoch migration, power loss and full platform acceptance remain open.

## Recovery observation

`ProjectStore::recovery_report` requires exact registry/runtime/schema/profile
admission and an idle writer. One held SQLite read transaction observes current,
all generations and their complete membership descriptors, partition hashes,
validations, history, operation receipts, retention roots, policy, GC receipts,
foreign keys and SQLite quick-check. Admission failures return an error; no
successful closure report is manufactured.

Current is `absent`, `validated`, `corrupt` or `unverified`. Each scope is
`validated`, `invalid`, `incomplete` or `not_applicable`. Unselected v3 policy is
valid; frozen v1/v2 tables are not queried when absent. Incomplete inventories
cannot establish a clean negative. Current validation reads its full membership
and seals; every other manifest also checks schema/length/key/version against its
seals. The partition scope independently hashes each stored payload once.

Operations retain their exact Prepared, Published, Validated, Activated receipt
or Released disposition. Prepared may legitimately lack a target or some seals.
Released evidence remains valid after collection. An activation receipt does not
prove delivery to a caller: acknowledgment is always `unknown`. Expected bases
and original activation IDs remain explicit; recovery does not select, rebase,
repair, release or delete anything.

The service exposes `recover_live_project` and `LiveProjectStore::recovery_report`.
The public transport is:

```text
wow project recover --store-root <directory> [--format json|text]
```

Exit 0 means complete applicable physical coverage; 2 means incomplete coverage;
4 means an invalid scope/current or an admission/output error; 130 means cancelled.
Invalid takes precedence when invalid and incomplete scopes coexist; the report
retains both. Physical validation is separate from compiled domain replay.

## Backup artifact

`ProjectStore::backup_to_new(root, operation_id, stop)` captures one complete
inline snapshot, including committed WAL state, through rusqlite's native SQLite
Backup API. The artifact owns a new directory and writer lock. Durable intent
precedes database writes; its final manifest follows independent read-back and
file synchronization. It has no normal project registry and cannot be opened as
an ordinary store. Failed/cancelled partial roots remain for explicit inspection.
An existing directory refuses creation without overwrite or cleanup.

Copying uses 128-page steps, at most 2,057 calls, at most eight contention outcomes
with finite 20 ms backoff, and a 262,144-page/1 GiB ceiling. Only `Done` completes
the copy. Destination WAL is fully checkpointed and the connection is closed
before body length/hash and independent reopen. No destination API runs while
the backup handle is alive; no main-file-only copy, serialization shortcut or
unbounded `run_to_completion` is used.

The canonical manifest binds operation/request, unchanged epoch/runtime/schema/
physical profile, current, all included generation and partition IDs, whole-body
digest/length, physical coverage and the exact logical snapshot digest. The latter
also binds every validation/history ID, operation receipt, root, policy and GC
receipt. The profile is self-contained inline partitions; the separate generic
object store and external files are not backup authorities here.

`VerifiedBackup::open` requires the named operation, caller's exact catalog and
previously observed snapshot digest. It reconstructs every manifest field from
actual database/file state and compares canonical bytes, including durable intent.
`verify` repeats whole-artifact checks. `read` holds the same owner lock and checks
the selected generation's complete membership/seals through the existing reader.
It does not rehash the entire backup for each selected member. External-process
writers remain unsupported.

`LiveProjectStore::backup_to_new` additionally replays every included generation
through the real `AcquiredProjectPair` owners before returning it. A domain error
leaves the physical artifact for inspection; it never becomes domain success.

## Isolated restore

`VerifiedBackup::restore_to_new` creates another explicit private candidate using
the same native route. `finish_restore` requires unique, exact owner-validation
capabilities for every included generation before writing the new private registry.
Existing staged epoch-manifest bytes reconcile only by exact equality. Ambiguous
file writes stay `OutcomeUnknown`; a caller must inspect the original candidate.

`restore_live_project_to_new` runs native Project/Graph replay for all included
generations and supplies those capabilities. Original semantic epoch, generation,
validation, publication and owner identities survive. The physical path is separate;
this operation finishes a separate root. Source current and its older leased
readers continue unchanged. Explicit live replacement uses the separate
[versioned physical selector](PROJECT_REGISTRY.md), exact source/current guards
and repeat native owner validation. It preserves IDs, keeps old files/readers,
and reconciles original staged or already-selected intent without blind effects.
`LiveProjectStore::restore_replace` and `resume_replacement` expose that path;
no CLI restore or production-store operation was executed by this checkpoint.

## Evidence and remaining work

Actual isolated Windows tests include an older leased reader with committed WAL
frames beyond its checkpoint, independent backup reopen, exact reads of both
generations, original receipt identities after restore/reopen, body truncation,
manifest substitution, cancellation, an existing destination sentinel, canonical
orphan-manifest descriptor corruption, and frozen v1/v2 backups. Native service
replay verifies both retained Project/Graph pairs and the still-held source reader.

The physical replacement fixtures also verify stale guards, corrupt/missing-owner
targets, shared admission/locks across successive instances, explicit unknown
result reconciliation and Windows selector-sharing failure. Four native child
termination probes cover completed prepare/stage/validate/activate boundaries.
They do not establish interruption inside a write/OS call, power loss, hostile OS
access, cleanup faults, full W16/E2 or Gethe/Ketho/runtime acceptance. The separate
[held-instance restore](PROJECT_QUARANTINE_RESTORE.md) covers whole-instance
quarantine and portable archive authority. Supported migrations, domain quarantine
and object/epoch reclamation remain separate requirements.
