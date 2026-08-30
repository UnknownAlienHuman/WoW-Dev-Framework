# `wow-store` E1-A normative examples

These files define the closed SQLite runtime, schema/migration, immutable ReferenceStore publication, content-addressed object, and checksum-freeze contract for the future E1-A implementation.

## Files

- [`sqlite-runtime-profile.json`](sqlite-runtime-profile.json) — exact Rust binding/SQLite capability and open/PRAGMA/security/durability probe shape.
- [`schema-migration-cases.json`](schema-migration-cases.json) — valid empty-to-target migration and graph/ledger/tamper/in-place/dynamic-SQL rejection cases.
- [`reference-publication-cases.json`](reference-publication-cases.json) — staging, validation, seal, final generation publication, active-pointer publication, crash/cancellation/idempotency/read-only cases.
- [`object-store-cases.json`](object-store-cases.json) — logical ObjectId, encoded payload, write/read/dedup/corruption/path/resource/reference/GC cases.
- [`CHECKSUMS.json`](CHECKSUMS.json) — runtime/schema/migration/store/object/publication/checksum freeze gate.

## Current state

No E1-A Rust or SQL implementation exists. The exact SQLite binding/version/features, runtime profile, standard metadata schema, reference domain schema/operation bundle, migration graph, StoreGeneration, ObjectId/payload, publication, and checksum vectors are not frozen.

Therefore these fields remain null:

```text
SQLite/binding/source/probe/runtime-profile IDs and digests
platform publication adapter and durability evidence
metadata/reference schema bundle and operation catalog IDs/digests
migration graph/plan/edge/ledger/target schema IDs/digests
StoreConfiguration/StoreId/StoreBuild/Staging/StoreGeneration/Manifest/Integrity/Pointer IDs
SQLite file/logical data/object reference-set digests
ObjectId/payload/manifest/reference/lease/GC report IDs and digests
publication/crash/cancellation transition report IDs
member and bundle SHA-256 values
```

Nulls are valid only while `crates/MANIFEST.json` reports `wow-store.implementation_state = not-started`.

Before the first `wow-store` Rust commit, the implementation agent must:

1. select, pin, audit, and probe the exact Rust SQLite binding and SQLite runtime;
2. freeze the platform path/publication/durability adapter behavior;
3. freeze the standard metadata schema and static operation catalog;
4. freeze the actual E1 `wow-reference` domain schema/operation/validation bundle before full integration;
5. freeze migration graph, plan, ledger, and target schema vectors;
6. freeze the ReferenceStore build, validation, seal, publication, active-pointer, read-only-open, and crash/cancellation vectors;
7. freeze object logical/payload/reference/GC vectors;
8. classify raw SQLite file reproducibility honestly;
9. canonicalize all examples through `wow-core` canonicalization;
10. write all member and bundle SHA-256 values;
11. update `CONTRACT.json` and manifest implementation state;
12. execute every applicable `TEST_MATRIX.md` case.

Tests verify frozen files and never rewrite them automatically.

## Domain boundary

The examples use generic or opaque domain-operation IDs. They do not define WoW API/entity/restriction semantics. The actual `wow-reference` schema bundle owns those records; `wow-store` owns only persistence lifecycle and validation.

## ReferenceStore boundary

A successful candidate is still not active until:

```text
schema/data/object validation
integrity validation
manifest finalization
seal
final generation publication
final-path read-only reopen/validation
active-pointer atomic replacement
```

Every failed/cancelled/crash case preserves the previous active generation.

## Object boundary

Logical identity is SHA-256 of canonical uncompressed logical bytes. Codec/payload identity is separate. Existing mismatch is corruption and never overwritten. GC requires a complete reference/lease snapshot and deletes only `eligible=yes`.

## Determinism boundary

Canonical logical/schema/object/store/publication reports must be deterministic. Raw SQLite file byte reproducibility is a separate measured classification and must not be claimed from logical equivalence alone.

## Change protocol

Any semantic change must update:

- the owning store/schema/SQLite/publication/object/integrity document;
- `CONTRACT.json`;
- affected example cases;
- `TEST_MATRIX.md`;
- all runtime/schema/store/object/publication/checksum vectors after implementation starts.

Do not weaken atomic publication, read-only sealing, schema digest, object verification, GC safety, or security expectations merely to make an implementation pass.
