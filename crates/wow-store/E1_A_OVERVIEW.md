# `wow-store` implementation contract

**Status:** E1-A implementation-ready storage-foundation contract; no Rust code yet.

## Mission

`wow-store` owns the narrow persistence substrate used by Reference Packs and later project/external generations. It provides deterministic SQLite lifecycle, versioned schema application, typed transaction boundaries, atomic generation publication, integrity verification, and content-addressed objects without importing WoW domain semantics.

The crate exists to make persistence boring, inspectable, crash-safe within an explicit durability contract, and impossible to confuse with platform authority. It does not decide what an API, frame, event, restriction, finding, or graph edge means.

## E1-A outcome

A future implementation agent must prove:

```text
one repository-owned schema bundle can be validated and applied to a staging SQLite store
one immutable reference store can be built, verified, sealed, atomically published, and reopened read-only
one content-addressed object can be written atomically, verified, deduplicated, resolved, and garbage-collected only when unreferenced
one migration graph can reject unknown/skipped/tampered transitions
one publication failure/crash point leaves the previously active generation intact
one store manifest records exact schema, SQLite/configuration, generation, object, and integrity identities
```

E1-A prepares but does not activate the mutable project-store/WAL path required by E2.

## Owned responsibilities

- store/profile kind and immutable generation identities;
- SQLite connection/open-mode lifecycle;
- repository-owned schema-bundle validation and registration;
- standard metadata tables and schema migration ledger;
- exact schema-version/digest compatibility checks;
- staging-store creation;
- migration planning, transactional execution, verification, and rollback/abort;
- immutable reference-store sealing and read-only opening;
- atomic publication of validated store generations;
- active-generation pointer/manifest replacement;
- one-writer transaction discipline for future mutable stores;
- read snapshots/transactions and explicit consistency boundaries;
- integrity, foreign-key, schema, checksum, and manifest verification;
- content-addressed logical object IDs;
- encoded-object manifests, compression metadata, and payload verification;
- atomic object writes, deduplication, reads, reference tracking, and safe garbage collection;
- store budgets, cancellation, lock/busy policy, and failure isolation;
- deterministic store/publication manifests and reports;
- store security controls: root confinement, no extension loading, no arbitrary attach/SQL surface;
- backup/replace/last-known-good retention policy at the storage layer.

## Explicit non-responsibilities

`wow-store` does not:

- parse Blizzard/APIDocumentation/Lua/XML/TOC/source files;
- know API, entity, relation, restriction, rule, finding, search, or project semantics;
- select a WoW profile or decide whether a profile is current;
- derive Reference Pack or ProjectGeneration domain facts;
- expose raw SQL/database handles through service/CLI/MCP/LSP;
- accept SQL/DDL/migration code from analyzed or external repositories;
- execute repository/source code, hooks, installers, or generators;
- fetch/download source or packs;
- mutate Codebase Memory or third-party databases;
- migrate immutable released Reference Stores in place;
- use a database presence/row count as evidence completeness;
- silently repair, ignore, or skip schema versions/migrations;
- publish a staging/corrupt/unverified generation;
- delete objects referenced by any retained generation;
- claim durability stronger than the selected platform adapter actually achieved;
- own release signing, distribution, or transport.

## Required reading

Before implementation, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`AGENTS.md`](AGENTS.md)
6. [`DECISIONS.md`](DECISIONS.md)
7. [`DATA_MODEL.md`](DATA_MODEL.md)
8. [`SCHEMA_AND_MIGRATIONS.md`](SCHEMA_AND_MIGRATIONS.md)
9. [`SQLITE_PROFILE.md`](SQLITE_PROFILE.md)
10. [`TRANSACTIONS_AND_PUBLICATION.md`](TRANSACTIONS_AND_PUBLICATION.md)
11. [`OBJECT_STORE.md`](OBJECT_STORE.md)
12. [`REFERENCE_STORE.md`](REFERENCE_STORE.md)
13. [`PROJECT_STORE.md`](PROJECT_STORE.md)
14. [`INTEGRITY_AND_SECURITY.md`](INTEGRITY_AND_SECURITY.md)
15. [`ERROR_MODEL.md`](ERROR_MODEL.md)
16. [`TEST_MATRIX.md`](TEST_MATRIX.md)
17. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
18. [`CONTRACT.json`](CONTRACT.json)
19. current storage consumers' contracts (`wow-reference` in E1, `wow-project` in E2)

Normative repository sources:

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

Primary SQLite references for the implementation agent:

- [Transactions](https://www.sqlite.org/lang_transaction.html)
- [Write-Ahead Logging](https://www.sqlite.org/wal.html)
- [PRAGMA statements](https://www.sqlite.org/pragma.html)
- [URI filenames and read-only/immutable modes](https://www.sqlite.org/uri.html)
- [Atomic commit](https://www.sqlite.org/atomiccommit.html)
- [Foreign keys](https://www.sqlite.org/foreignkeys.html)
- [How database files can become corrupt](https://www.sqlite.org/howtocorrupt.html)
- [Online backup API](https://www.sqlite.org/backup.html)

Pin the actual Rust SQLite library/version and probe its behavior before code; this contract does not select one from memory.

## Direct dependency

```text
wow-core
```

No domain crate dependency. Domain crates depend on `wow-store` and provide repository-owned schema/operation bundles through the public storage seam.

## Storage topology

```text
ReferenceStore
    immutable SQLite file for one exact ReferenceGeneration/Profile
    built in staging, verified, sealed, atomically published
    runtime open read-only/query-only/immutable when supported
    never migrated in place after release

ProjectStore
    mutable rebuildable SQLite store for project generations
    WAL + one writer + read snapshots
    E2 implementation; contract prepared here, inactive in E1-A

ExternalStore
    optional external manifest/source-handle metadata
    later milestone; no third-party source body redistribution

ObjectStore
    content-addressed logical objects and encoded payloads
    shared only through explicit manifest/reference ownership
```

## Standard store metadata

`wow-store` owns only generic metadata namespaces, conceptually:

```text
store_meta
schema_bundle
schema_migration
store_generation
publication_record
integrity_record
object_manifest
object_reference
retention_record
```

Domain tables/indexes belong to registered compile-time schema bundles supplied by owning crates. `wow-store` validates/applies them but does not interpret their WoW fields.

## Schema-bundle seam

A domain crate supplies a repository-owned, versioned, compile-time schema bundle:

```text
SchemaBundle
    namespace
    version
    parent version(s)
    canonical DDL/migration operation catalog
    expected schema digest
    required SQLite capabilities
    declared tables/indexes/triggers/virtual tables
    validation queries/checks
    destructive-change policy
```

Rules:

- no runtime SQL from user/source/MCP/external repository;
- no arbitrary SQL string accepted through service/application APIs;
- every bundle/migration has a stable ID/digest;
- migration graph is explicit and acyclic;
- unknown/skipped/tampered transitions fail;
- static domain adapters encode/decode domain records outside `wow-store`;
- store transaction/statement handles never escape to applications/transports.

## E1-A active path

```text
validate store root/configuration
validate SQLite capability/profile probe
validate/register standard metadata schema + one reference domain schema bundle
create staging ReferenceStore on same publication volume
apply exact migration path from empty -> target
write domain records through registered prepared operation catalog
write/verify content-addressed objects
run schema/foreign-key/integrity/application validation
build deterministic store manifest/checksums
seal staging store
atomically publish generation directory/file and active pointer
open published ReferenceStore read-only
run exact read/integrity checks
```

`wow-reference` owns the domain record adapter/build plan. `wow-store` owns lifecycle/transactions/publication/integrity.

## ReferenceStore rules

- one exact profile/reference generation per store;
- immutable after sealing;
- no WAL/SHM/journal files in released artifact;
- no in-place migration; build/validate/publish a new generation;
- open with least write capability and `query_only`/immutable semantics where supported;
- validate manifest, schema digest, file digest, store metadata, foreign keys, integrity, generation/profile before use;
- mismatch/corruption rejects activation and retains last-known-good active pointer;
- readers cannot observe staging or partially published files.

See [`REFERENCE_STORE.md`](REFERENCE_STORE.md).

## ProjectStore rules (deferred to E2)

- one writer owner/actor per store;
- WAL mode and busy policy explicit/pinned;
- write transaction publishes one coherent project-store generation;
- readers use immutable snapshots/transactions;
- failed/cancelled write does not advance current generation;
- checkpoint/retention/backup policy explicit;
- rebuildable data remains distinguishable from source truth;
- no E1-A Cargo activation merely to satisfy the final diagram.

See [`PROJECT_STORE.md`](PROJECT_STORE.md).

## Content-addressed object rules

Logical object identity:

```text
ObjectId = sha256(canonical uncompressed logical bytes)
```

Encoded payload identity is separate:

```text
payload digest
codec + codec version/parameters
encoded length
logical length
```

Consequences:

- compression changes do not change logical ObjectId;
- payload is verified before publication/read;
- object paths derive only from validated fixed-format digest, with bounded fanout;
- writes use same-volume temp file, durable flush contract, atomic no-replace/replace policy, and verify-after-write;
- existing valid object deduplicates; mismatched existing payload is corruption, not overwrite;
- object references are explicit per retained generation;
- garbage collection deletes only verified unreferenced objects after retention/lease checks;
- no source path/name/content excerpt in object filename.

See [`OBJECT_STORE.md`](OBJECT_STORE.md).

## Atomic publication

Publication state machine:

```text
Candidate
-> StagingBuilt
-> SchemaValidated
-> DataValidated
-> IntegrityValidated
-> ManifestFinalized
-> Sealed
-> GenerationPublished
-> ActivePointerPublished
```

Failure/cancellation before final pointer publication leaves the prior active generation unchanged. A published generation is immutable and independently addressable.

The active pointer is a small versioned manifest/file atomically replaced on the same volume. Symlinks are not required. Platform adapter reports the achieved durability level and required fsync/flush behavior.

See [`TRANSACTIONS_AND_PUBLICATION.md`](TRANSACTIONS_AND_PUBLICATION.md).

## Integrity and security

- root-confined normalized paths;
- no extension loading;
- `trusted_schema` disabled/defensive mode where supported;
- foreign keys enabled and checked;
- no arbitrary ATTACH/DETACH or writable external DB;
- strict read-only/reference open mode;
- schema objects compared with registered bundle/digest;
- quick/integrity/semantic validation selected explicitly;
- bounded page/file/object/query/output sizes;
- untrusted SQLite imported/rebuilt into owned schema rather than opened writable;
- no SQL from analyzed source;
- no raw SQL application/MCP surface;
- corruption never auto-repaired into a new truth without explicit rebuild/evidence;
- manifests/logs exclude secrets/private local paths.

See [`INTEGRITY_AND_SECURITY.md`](INTEGRITY_AND_SECURITY.md).

## Determinism

Equivalent logical inputs/configuration must produce equivalent canonical:

```text
schema bundle/migration ledger
logical rows and ordering-independent store digest report
object IDs/manifests
store manifest/generation ID
publication report
```

SQLite file byte identity is required only where the selected build pipeline can guarantee it. If raw file bytes contain nondeterministic layout, the canonical logical/store manifest digest remains authoritative and nondeterministic physical fields are isolated/reported. Reference Pack release gate may later require a reproducible vacuum/export procedure after measurement.

Do not falsely claim byte-identical SQLite files without proving it.

## E1-A hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No domain semantics/types in store.
- No raw SQL/connection in service/CLI/MCP/LSP.
- No user/source/external SQL or extension loading.
- No in-place migration of released ReferenceStore.
- No ProjectStore/WAL implementation activation in E1-A.
- No publish before all validation/seal steps.
- No active-pointer update before generation publication.
- No delete of referenced/leased objects.
- No overwrite of digest collision/mismatch.
- No durability claim beyond measured platform adapter.
- No database row absence interpreted as platform negative authority.
- No CI/release automation.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- SQLite capability/profile probe;
- schema bundle and migration graph;
- immutable ReferenceStore build/open/publication cases;
- atomic publication crash/cancellation points;
- content-object write/dedup/corruption/GC cases;
- deferred ProjectStore behavior;
- manifest/checksum freeze.

Exact SQLite library/version/probe, schema/migration IDs, generation/object/store digests, and SHA-256 fixture values freeze before the first `wow-store` Rust commit.

## Definition of done

E1-A implementation is complete only when:

```text
store depends only on wow-core and imports no WoW domain semantics
one registered schema bundle can be applied only through its exact migration path
one ReferenceStore generation builds in staging, validates, seals, publishes atomically, and reopens read-only
all publication failure/crash/cancellation points retain the prior active generation
one logical object writes/deduplicates/verifies/resolves and referenced-object GC is impossible
all schema/integrity/foreign-key/manifest/digest mismatches reject activation
no raw SQL/extension/attach/source-execution/application DB handle escapes
ProjectStore path remains explicitly deferred
logical/manifests/reports are deterministic under randomized insertion/order/temp roots
all TEST_MATRIX cases pass
```

Until then, this directory remains an implementation-ready persistence contract, not a storage engine.
