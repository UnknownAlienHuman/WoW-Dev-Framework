# `wow-store` implementation contract

**Status:** deferred until E1/E2; contract scaffold only.

## Mission

`wow-store` owns durable local storage mechanics for immutable Reference Packs, mutable project generations, external manifests, and content-addressed objects. It provides transactions, migrations, integrity checks, bounded queries, and retention primitives without owning WoW domain extraction or query semantics.

## Owned responsibilities

- SQLite connection/open-mode policy;
- schema version discovery and migration execution;
- immutable/read-only reference database access;
- mutable project database WAL/transaction lifecycle;
- atomic project-generation publication;
- generic entity/edge/source-span/restriction/alias/lineage persistence primitives defined by approved schemas;
- content-addressed object put/get/verify/garbage collection;
- FTS table lifecycle and deterministic row maintenance;
- database integrity, checksum, and manifest binding checks;
- retention of last-known-good generations;
- bounded pagination and cancellation at the storage boundary.

## Explicit non-responsibilities

`wow-store` does not:

- decide which domain entities or relations should exist;
- parse Lua, TOC, XML, APIDocumentation, or annotations;
- rank search results;
- execute recognizers or diagnostics;
- open arbitrary external SQLite files as trusted writable stores;
- expose raw SQL to MCP/LSP/CLI callers;
- own Reference Pack acquisition/download;
- infer schema compatibility from column presence alone.

## Physical stores

Planned responsibilities:

```text
reference.sqlite
    immutable, checksum-bound, one exact profile

project.sqlite
    mutable WAL database, rebuildable, generation-published

external.sqlite
    optional manifests and source-handle metadata only

objects/
    content-addressed compressed raw/skeleton/evidence objects
```

Physical schemas are activated only when their owning milestone defines versioned migrations and fixtures.

## Required operations

| Operation | Required behavior |
|---|---|
| `open_reference_store` | Open verified reference storage read-only; bind to manifest/schema/profile identity. |
| `create_project_store` | Create an owned project database at an approved root with explicit schema version. |
| `open_project_store` | Validate identity/version and recover or reject incomplete transactions safely. |
| `plan_migrations` | Produce an ordered migration plan without applying it. |
| `apply_migrations` | Apply approved migrations transactionally with rollback/last-known-good behavior. |
| `begin_generation_write` | Start an isolated generation transaction with one generation context. |
| `replace_partition` | Replace facts for a named partition/generation, never append stale duplicates. |
| `publish_generation` | Atomically make one complete generation visible to readers. |
| `abort_generation` | Discard incomplete writes without corrupting the previous published generation. |
| `read_generation_snapshot` | Provide a stable read view that cannot mix publication epochs. |
| `put_object` | Store content by digest after size/type checks; return stable object identity. |
| `get_object` | Verify digest and budget before returning bytes/decoded content. |
| `collect_unreferenced_objects` | Remove only objects not referenced by retained generations/manifests. |
| `integrity_report` | Report schema, SQLite, manifest, checksum, orphan, and generation consistency. |
| `bounded_query_page` | Return deterministic ordered pages with explicit truncation/cursor semantics. |

## Transaction and concurrency rules

1. Reference stores are immutable after pack publication.
2. One writer publishes project generations; readers use immutable snapshots/leases.
3. WAL is a project-store implementation detail, not a public correctness signal.
4. A reader never observes half a partition replacement.
5. Failed migration leaves the previous schema/data usable or returns a typed unrecoverable state.
6. Cancellation before publication aborts the new generation.
7. Long reads must not retain obsolete generations indefinitely without an explicit lease policy.
8. Query order is explicit; never depend on SQLite default row order.

## Untrusted-input controls

- reject databases with unsupported application/schema identity;
- verify external pack checksums before opening;
- open released reference DBs read-only and preferably immutable;
- bound page count, SQL variables, JSON payload sizes, blob sizes, decompression, and query duration;
- use prepared statements only;
- never deserialize arbitrary extension objects into executable behavior;
- reject absolute/traversal paths in object metadata;
- do not attach arbitrary user databases to the owned connection.

## Schema ownership

`wow-store` owns migration mechanics, not domain meaning. Domain crates define approved logical records; public interchange schemas live under `schemas/`; storage migrations map those contracts into SQLite.

A schema change requires:

- version bump;
- forward migration;
- rollback or explicit irreversible boundary;
- old/new fixture round-trip;
- compatibility report;
- deterministic dump comparison;
- retention and object-reference analysis.

## Milestone activation

### E1

Implement the minimum store needed for Reference Pack manifests, normalized API facts, raw metadata, source maps, annotations metadata, checksums, and capability partitions.

### E2

Add mutable project generations, graph/entity partitions, incremental replacement, WAL, and last-known-good retention.

Do not implement E2 project tables during E1 unless an E1 acceptance fixture requires a shared lower-level primitive.

## Required tests

- create/open/read-only mode;
- unsupported schema/version rejection;
- migration success, interruption, and rollback;
- atomic generation visibility under concurrent readers;
- failed generation leaves prior generation intact;
- deterministic query order and pagination;
- checksum/digest mismatch rejection;
- malicious SQLite metadata and oversized blob rejection;
- object deduplication and safe GC;
- WAL/recovery scenario;
- profile/generation mismatch rejection;
- schema round-trip from fixture pack/project.

## Documentation sources

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## Definition of done

The crate is ready for a milestone only when interrupted writes cannot corrupt the last published generation, untrusted stores cannot escape validation, every query is generation/profile-bound and deterministic, and no higher-layer domain logic is required to understand storage safety.
