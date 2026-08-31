# Selected ProjectStore physical model

**Status:** normative E2-D model selection, pending executable profile and benchmark freeze.

## Profile

```text
project-store-wal-manifested-partitions-v1
```

## Topology

```text
<private-root>/
  project-store-registry.json       outer epoch selector
  epochs/
    <epoch-id>/
      project.sqlite
      project.sqlite-wal             mutable runtime sidecar
      project.sqlite-shm             mutable runtime sidecar
      epoch-manifest.json
  objects/
    <content-addressed payloads>
  backups/
  quarantine/
```

Public manifests use root-relative validated paths. Absolute paths remain private runtime configuration.

## Why not one database per project generation

Rejected for normal updates because one-file changes can require copying/rebuilding an entire database, historical retention duplicates pages, and incremental producer partitions lose their natural replacement unit. A whole new database remains appropriate only for an incompatible epoch rebuild.

## Why not duplicated rows per generation

Rejected as the default because every retained generation duplicates unchanged project and graph records.

## Why not recursive base plus delta

Rejected because read correctness depends on unbounded ancestry, missing/corrupt bases break descendants, compaction changes lookup shape, retention requires descendant closure, and stale tombstones are difficult to prove.

## Selected representation

### Immutable partition versions

Each project or graph partition is written under a stable `PartitionVersionId`. Domain records are immutable and scoped to that version.

### Complete membership per generation

A generation has a complete map of all active partition keys and versions. Reused partitions add membership rows only; reads do not recurse.

### Materialized query indexes

Owner bundles may define deterministic query indexes scoped to partition versions. Cross-partition indexes must be either generation-scoped and rebuilt for the target generation, or immutable index partitions with explicit dependency manifests.

## Normal publication phases

```text
Phase 0: preflight and writer acquisition
Phase 1: materialize/reuse sealed partition versions
Phase 2: insert complete target membership and semantic manifests
Phase 3: commit PublishedInactive
Phase 4: exact read-back validation
Phase 5: activate by current-record CAS
Phase 6: optional bounded checkpoint/retention maintenance
```

## Database state immutability

Mutable:

```text
writer lease/operational records
building target records inside transaction
current publication record
checkpoint/retention/GC metadata
```

Immutable after seal/publication:

```text
partition version payloads
generation membership
ProjectPublicationSet
ProjectSnapshot/GraphSnapshot manifests
validation report identities
published StoreGeneration semantic manifest
```

## Store-owned physical record families

```text
store_epoch
schema_bundle_ledger
operation_catalog_ledger
partition_version
store_generation
generation_partition_membership
publication_set
inactive_validation
current_publication
publication_history
retention_pin
gc_record
object_reference
```

Project/graph tables come from owner bundles and key records by `partition_version_id` or immutable manifest IDs.

## Limits

The profile freezes page/auto-vacuum policy at epoch creation, effective WAL/synchronous/busy/checkpoint settings, database/page/WAL size limits, statement/result limits, maximum partitions/membership rows, and retained inactive generation count. Defaults are not accepted.

## Epoch changes

The following require a new epoch unless a later compatibility contract proves safe in-place migration:

```text
physical model version
SQLite binding/library/compile options affecting correctness
page/encoding/application layout
breaking metadata/project/graph schema
canonicalization or partition identity change
security/open/locking model change
```

New epoch build:

```text
create sibling epoch
-> build exact current project/graph publication
-> validate complete target
-> close/checkpoint according to profile
-> atomically update outer registry record
-> retain old epoch until readers/rollback policy allow GC
```

## Nonclaims

The model does not promise physical SQLite byte reproducibility, power-loss durability beyond measured adapter evidence, external multi-process readers, or runtime WoW state persistence.
