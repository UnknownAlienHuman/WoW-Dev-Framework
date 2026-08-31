# `wow-store` contract router

**Status:** E1-A storage foundation and E2-D ProjectStore publication contracts are implementation-ready; no Rust code exists.

`wow-store` owns the persistence substrate only. It depends directly on `wow-core` and never imports WoW project, graph, reference, rule, search, or application semantics. Domain crates supply reviewed schema, prepared-operation, validation, and logical-manifest contracts.

## Contract routes

### E1-A — storage foundation and immutable ReferenceStore

The complete pre-E2 E1-A overview is preserved byte-for-byte as [`E1_A_OVERVIEW.md`](E1_A_OVERVIEW.md). Read it before the detailed root package:

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`SCHEMA_AND_MIGRATIONS.md`](SCHEMA_AND_MIGRATIONS.md)
5. [`SQLITE_PROFILE.md`](SQLITE_PROFILE.md)
6. [`TRANSACTIONS_AND_PUBLICATION.md`](TRANSACTIONS_AND_PUBLICATION.md)
7. [`OBJECT_STORE.md`](OBJECT_STORE.md)
8. [`REFERENCE_STORE.md`](REFERENCE_STORE.md)
9. [`INTEGRITY_AND_SECURITY.md`](INTEGRITY_AND_SECURITY.md)
10. [`ERROR_MODEL.md`](ERROR_MODEL.md)
11. [`TEST_MATRIX.md`](TEST_MATRIX.md)
12. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
13. [`CONTRACT.json`](CONTRACT.json)
14. [`examples/`](examples/README.md)

E1-A proves schema/migration registration, immutable ReferenceStore build/seal/publication, content-addressed objects, validation, and generic store lifecycle. It does not activate a mutable project store.

### E2-D — ProjectStore and coherent project/graph publication

Read [`e2/README.md`](e2/README.md), then the complete E2-D package. The selected physical profile is:

```text
project-store-wal-manifested-partitions-v1
```

It uses:

```text
one owned SQLite database per ProjectStore epoch
WAL with one writer
immutable content-addressed partition versions
a complete generation-to-partition membership map
no recursive delta chain
published-inactive generation build
read-back validation
separate compare-and-swap current activation
snapshot-bound readers and explicit retention/GC
```

The stable project source, analyzer, recognizer, and graph semantics remain owned by their domain crates. `wow-store` executes only registered storage operations and generic publication protocols.

## Direct dependency

```text
wow-core
```

`wow-project` and `wow-graph` depend on `wow-store`; the reverse dependency is forbidden.

## Current implementation state

```text
documentation frontier: E2-D
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Directory presence and a selected physical profile do not bypass prerequisite implementation, probe, benchmark, fixture, or checksum gates.
