# `wow-store` contract router

**Status:** E1-A storage foundation, E2-D ProjectStore publication, and the E6-B external-candidate generic storage seam are implementation-ready; no Rust code exists.

`wow-store` owns the persistence substrate only. It depends directly on `wow-core` and never imports WoW project, graph, reference, rule, search, external-provider, mapping, selection, context, or application semantics. Domain crates/service supply reviewed schema, prepared-operation, validation, and logical-manifest contracts.

## Contract routes

### E1-A — storage foundation and immutable ReferenceStore

The complete pre-E2 E1-A overview is preserved byte-for-byte as [`E1_A_OVERVIEW.md`](E1_A_OVERVIEW.md). Its historical `PROJECT_STORE.md` target is preserved separately as [`PROJECT_STORE_PRE_E2_BOUNDARY.md`](PROJECT_STORE_PRE_E2_BOUNDARY.md). Read the root E1-A package for schema/migration registration, immutable ReferenceStore build/seal/publication, content-addressed objects, validation, and generic store lifecycle.

### E2-D — ProjectStore and coherent project/graph publication

Read [`e2/README.md`](e2/README.md). The selected physical profile is:

```text
project-store-wal-manifested-partitions-v1
```

It uses one owned SQLite database per ProjectStore epoch, WAL with one writer, immutable content-addressed partition versions, complete generation membership maps, published-inactive build, read-back validation, separate CAS activation, snapshot-bound readers, and explicit retention/GC.

### E6-B — external-candidate generic persistence

Read [`E6_B_EXTERNAL_CANDIDATE_STORAGE.md`](E6_B_EXTERNAL_CANDIDATE_STORAGE.md).

The store may persist registered immutable objects and append-only records for provider/session references, query/dispatch receipts, bounded raw response objects, E6-A result sets/artifacts, mapping/selection/context manifests, reconciliation, retention, and audit.

It does not call providers, own credentials/sessions, parse results, validate Candidate authority, map locators, choose candidates, build context, or interpret privacy/license semantics. There is no current/default external result pointer.

## Direct dependency

```text
wow-core
```

Domain crates and service depend on `wow-store` through narrow contracts; the reverse dependency is forbidden.

## Current implementation state

```text
documentation frontier: E6-B generic storage seam
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Directory presence and documented seams do not bypass prerequisite implementation, probe, benchmark, fixture, adapter, or checksum gates.