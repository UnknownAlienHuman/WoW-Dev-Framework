# Persistent ProjectStore generation holds

Updated 2026-10-09. This W15 slice supplies persistent exact generation pins;
generation/partition GC and backup/recovery acceptance remain open.
The subsequent executable v3 release/GC slice is in
[`PROJECT_GC.md`](PROJECT_GC.md); this document records the preceding v2 boundary.

`ProjectStore::create_with_retention` selects
`project-store-wal-manifested-partitions-v2`. Its static SQLite schema extends
the v1 schema with `retention_roots` and a generation index; the epoch identity
binds the selected physical profile and schema digest. The registered partition
payload catalog is unchanged. `create` still creates a frozen v1 physical epoch.
`open` validates either exact registered profile without migration, DDL or
rewriting old manifests. An unsupported profile is refused.

`LiveProjectStore::create` uses v2. Its open path continues to accept the frozen
owner catalogs and v1 physical epochs. Service retention operations on v1 return
`OperationNotImplementedForMilestone`; store retention operations on v1 return
`ConfigurationInvalid`. Publication and exact historical reads retain their
existing compatibility rules.

## Exact hold API

`RetentionRoot::new` binds one `EpochId`, `RetentionRootId`, closed kind,
`StoreGenerationId` and attributable `held_by`. The canonical
`wow-store/retention-root/1` record includes a digest over all those fields.
Kinds are Evidence, Debug, Rollback, User, Recovery, Quarantine, Backup, Export
and Policy. These are holds, not claims that the named workflow has run.
Current, leased readers and in-progress publications remain store state;
callers cannot declare or retarget them through this API.

The store and service expose typed `put_retention_root`, `retention_roots` and
`remove_retention_root`. Put requires an existing exact target manifest in the
same epoch. Repeating the same complete root returns the original record;
substitution under its ID conflicts. Listing is ordered by root ID. Removal
requires the reviewed pin digest and returns false for an already absent root.
A pin never activates or changes current and has no expiry or wildcard target.

IDs and holders use the existing named-identifier bound of 256 ASCII bytes.
Each record is bounded to 65,536 bytes and an epoch admits at most 1,024 roots.
Every consumed row must match its indexed IDs, canonical bytes, reconstructed
digest and exact generation manifest. The generation foreign key prevents a
root from naming a missing row. Listing validates manifests, not complete domain
integrity or owner read-back acceptance.

Writes use immediate transactions, reserved WAL budget and cancellation before
commit. Post-commit observation requires an autocommit connection: an unresolved
transaction cannot turn pending same-connection reads into durable success.
Unprovable outcomes remain `OutcomeUnknown`. The same guard protects publication
commit observation. No sidecar file acts as pin authority.

## Executable evidence and remaining work

The on-disk store regression covers sealed membership, exact retries,
substitution, foreign epoch, missing generation, cancellation, reopen,
digest-guarded removal and frozen v1 refusal. The service regression publishes
real native Project/Graph data, pins it, reopens it and verifies its exact graph,
alongside current-catalog v1 compatibility. The frozen legacy-catalog service
regression remains separate evidence. An owner-internal publication regression
refuses commit success while a real SQLite transaction remains active; it does
not simulate Windows or power-loss fault acceptance.

Final workspace policy, fmt, check, strict Clippy, tests (864 passed, 1 ignored,
105 targets), rustdoc and build passed on 2026-10-09. The ignored consumer test
remains an unexecuted acceptance gate.

No data deletion is implemented by this slice. GC still needs operation-root
release with durable idempotency evidence, exact current/history/validation and
membership closure, prepared manifests that protect versions before membership,
active leases, shared partitions, explicit retention policy, stale-plan rejection
and committed deletion receipts. Backup/restore, object and epoch collection,
Windows sharing and crash/power-loss acceptance remain open.
