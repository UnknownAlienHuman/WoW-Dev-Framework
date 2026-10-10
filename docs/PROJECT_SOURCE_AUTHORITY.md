# Portable project source authority

`wow-store` retains original source-epoch selector and quarantine evidence beside
an exported migration target. The evidence keeps its original epoch and record
catalog. It can be reopened with the artifact without access to the original
database or its physical instances. This document describes implemented owner
behavior; acceptance evidence is recorded separately, with no W16 or E2 acceptance
PASS claimed here.

## Retained representation

`wow_store::project::SourceAuthorityReference` exposes `manifest_digest()` and
`manifest_length()`. `BackupManifest::source_authorities()` returns the complete
flat reference inventory. `ProjectStore::retained_source_authorities(&self)`
returns `StoreResult<Vec<SourceAuthorityReference>>` after full live registry
admission. Deserializing a reference grants no owner capability.

Each entry lives under `source-authorities/<64-hex-manifest-digest>/` and contains
canonical `authority.json`, exact `selection.json`, and its native quarantine
archives. Its manifest binds the original `EpochManifest`, `RegistrySelection`,
selector byte length, guarded source snapshot digest, retained hold references
and authority dependencies. The manifest digest uses the
`project-source-authority:sha256:` prefix.

Admission validates each entry under its own epoch/catalog, admits its selector
without opening historical SQL, and requires the dependency list to equal the
union required by that selector and its hold archives. Every dependency must be
present in the flat inventory; cycles reject. This representation preserves
selector/hold evidence, without supplying original source SQL/history hydration.

## Schemas and identities

| Artifact | Schema with source authority | Existing encoding when sources are empty |
| --- | --- | --- |
| Authority manifest | `wow-store/project-source-authority/1` | No authority entry |
| Backup manifest and intent | `wow-store/project-backup/3`, `wow-store/project-backup-intent/3` | `/1` without holds; `/2` with holds |
| New restored registry | `wow-store/project-registry/7` | Bare `EpochManifest` without holds; `/5` with holds |
| Same-epoch replacement registry | `wow-store/project-registry/8` | `/2` without holds; `/4` with holds |

Same-epoch staging keeps `replacement-intent.json`: ordinary replacement uses
`wow-store/project-replacement-intent/1`; quarantine restore uses
`wow-store/project-replacement-intent/2` with its exact hold guard. Source authority
does not introduce a separate restore-intent schema.

Empty source-reference fields are omitted, preserving the existing canonical
encodings and snapshot recipes. With sources present, the backup snapshot hashes
the `/3` envelope containing the existing SQL/selected-hold snapshot digest and
sorted source references, using the `project-backup-snapshot` prefix. The portable
export therefore binds additional context beyond the original ready artifact.
These artifact schemas are separate from SQL physical profiles and payload
catalog versions; adding source authority performs no in-place schema migration.

## Guarded export

The native entry is `ProjectStore::export_ready_migration_to_new`:

```rust
pub fn export_ready_migration_to_new(
    &self,
    migration: &ValidatedMigration,
    ready: &ReadyMigration,
    root: &Path,
    operation: &OperationId,
    expected: &RegistrySelection,
    expected_current: Option<&CurrentRecordId>,
    stop: &AtomicBool,
) -> StoreResult<VerifiedBackup>
```

The owner verifies the exact live source epoch, selector, explicit optional
Current and complete snapshot closure against the migration's retained source,
both before copying and afterward. It also verifies the ready receipt and closed
target inventory, captures the original selector/holds and inherited authority,
and copies the ready target into a new immutable artifact. A stale source rejects
with `CurrentConflict`; quarantine blocks live staging/export. The migration and
ready artifacts remain retained, and no live registry is selected by export.

`wow_service::live_project::LiveProjectStore::export_ready_migration_to_new` takes
the same guards with `operation_id: &str` and returns `ServiceResult<VerifiedBackup>`.
It replays every exported native Project/Graph pair through `AcquiredProjectPair`.
`prepare_live_project_migration` and its exact resume API produce the held
`ReadyMigration` prerequisite. The offline `export_live_project_migration` API
exports the baseline target separately; it does not capture live source authority.

## Independent reopen and restore

1. Retain the operation ID and full `BackupManifest::snapshot_digest()`. Drop the
   artifact owner before reopening its exclusively locked path.
2. Call `VerifiedBackup::open(root, catalog, operation, snapshot_digest, stop)`.
   Open and `verify` read all referenced entries, recapture SQL membership,
   validate payload bytes, and reconstruct the complete canonical manifest and
   intent. Serialized manifest fields alone cannot authorize restoration.
3. Call `VerifiedBackup::restore_to_new` for a separate private copy, then obtain
   fresh `ValidatedRead` capabilities for every retained generation and call
   `finish_restore`. The service `restore_live_project_to_new` performs these
   Project/Graph owner checks before finishing.

A source-bearing private finish writes canonical
`restored-source-authorities.json` before its `/7` selector. Full registry
admission requires that marker and retains its reference subset through later
selection. A completed marker without a selector supports exact reopen/finish
retry; conflicting marker bytes reject. Ambiguous file/publication outcomes stay
`OutcomeUnknown`, with retained effects for explicit reconciliation.

Ordinary backups retain selected source authority. Same-epoch replacement and
quarantine restore preserve inherited references. `ProjectStore::stage_replacement`
and `QuarantinedStore::stage_restore` use `source_authority::preflight_transport`
to admit the complete source/candidate union before creating `instances` or
copying a target. Activation repeats union admission before writing archives and
authority entries. This transport does not activate a migration across epochs or
claim acknowledgment of a lost response.

## Bounds and rejection

Admission allows at most 32 authority manifests and 32 distinct hold identities
across selected and archival inventories, keyed by original epoch plus operation.
Reused hold identities must have identical references. Combined manifest,
selector and hold-archive bytes are limited to 64 MiB; each authority manifest
is limited to 128 KiB. Registry selectors and replacement intents use the actual
`MAX_REGISTRY = 128 * 1024` byte bound (131,072 bytes). Existing native database/read
budgets still apply, and bounded operations observe cancellation checkpoints.

Missing files/dependencies, duplicate or unordered references, cycles, foreign
epoch holds, noncanonical or substituted bytes, and unsafe/reparse paths reject
admission. Files use confined digest-derived paths and exact-or-new writes;
conflicting content is never overwritten. Failed or cancelled artifacts remain
retained and do not constitute a completed export or clean negative result.

Owner code: [source authority](../crates/wow-store/src/project/source_authority.rs),
[backup](../crates/wow-store/src/project/backup.rs),
[registry](../crates/wow-store/src/project/registry.rs),
[guarded export](../crates/wow-store/src/project/migration/authority.rs) and
[service integration](../crates/wow-service/src/live_project/migration.rs).
