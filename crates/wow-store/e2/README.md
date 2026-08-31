# `wow-store` E2-D ProjectStore generation contract

**Status:** implementation-ready documentation; no Rust code.

**Contract ID:** `wow-store/e2-d/project-store-generation-publication`

## Mission

Provide a domain-neutral physical persistence primitive that can atomically materialize, seal, validate, retain, and open one immutable ProjectStore generation containing coherent project and graph logical partitions.

```text
validated registered schema/operation/validation bundles
+ exact expected base/head identity
+ exact ProjectPublicationBundle
+ content-addressed object plan
-> create isolated staging generation
-> execute one writer transaction
-> validate logical and physical closure
-> checkpoint and close staging WAL
-> seal immutable generation
-> reopen read-only
-> return exact generation/integrity/open report
```

The project/graph publication coordinator lives in [`../../wow-project/e2d/`](../../wow-project/e2d/README.md). `wow-store` does not understand TOC, XML, Lua, recognizers, graph relation meaning, or project generation policy.

## Selected physical profile

E2-D freezes a **file-per-generation SQLite profile**:

```text
store-root/
  registry/
    project-heads.sqlite or equivalent reviewed atomic registry
  generations/
    <StoreGenerationId>/
      store.sqlite
      manifest.json
      checksums.json
      object-refs.json
  objects/
    sha256/<prefix>/<digest>
  staging/
    <opaque-build-id>/
  quarantine/
```

The exact paths are profile-owned and never public semantic identity. One published generation is immutable. A shared content-addressed object store is referenced by immutable generation manifests.

A row-versioned single mutable database is not the E2-D profile. It remains a deferred alternative requiring separate benchmark, crash, migration, and reader-isolation evidence.

## Direct dependency

```text
wow-core
```

Domain records arrive through registered operation payloads and expected manifests. No direct dependency on `wow-project`, `wow-graph`, `wow-emmy`, or `wow-recognizers`.

## Core operations

```text
validate_project_store_profile
validate_registered_bundle_set
begin_project_store_generation
execute_registered_generation_plan
validate_staging_generation
commit_and_seal_generation
open_sealed_generation
validate_open_generation
compare_and_swap_publication_head
acquire_generation_lease
release_generation_lease
classify_recovery_inventory
recover_or_quarantine_generation
compute_retention_roots
plan_generation_and_object_gc
execute_validated_gc_plan
```

## Identity separation

```text
ProjectStoreGenerationId
    logical generation identity derived from exact registered bundle versions,
    logical manifests, object-reference manifest, and store contract version

ProjectStoreArtifactId
    physical sealed database/manifest/checksum artifact identity

ProjectPublicationHeadId
    one coherent head record identity owned semantically by wow-project
    and atomically stored by wow-store
```

Physical SQLite bytes may differ under a reviewed physical-profile change while logical project/graph generations remain semantically equal. Such a change creates a distinct artifact/profile identity and cannot be hidden.

## Completion gate

E2-D store code is complete only when crash/fault/cancel injection at every phase proves old-or-new publication; sealed generations are immutable and read-only; one head CAS cannot expose mixed generations; existing readers remain stable through leases; store operations are registered and bounded; recovery distinguishes staging, sealed inactive, current, leased, corrupted, and quarantined states; GC is root-based rather than age-only; and deterministic logical manifests survive worker/input-order changes.
