# Immutable ReferenceStore

**Status:** normative E1-A store-kind contract for one exact Reference Pack profile/generation.

## 1. Purpose

A ReferenceStore is the persistent queryable SQLite component of one exact Reference Pack. It is produced from pinned/validated domain inputs by `wow-reference`, built/published by `wow-store`, and consumed read-only.

It is not a floating cache, mutable knowledge base, or place for current live KB notes.

## 2. Identity

A ReferenceStore binds exactly:

```text
StoreKind = Reference
StoreId / StoreGenerationId
ProfileIdentity
ReferenceGenerationId
Reference Pack/domain manifest identity
schema registry and bundle set
SQLite runtime profile
logical data manifest digest
SQLite file digest/length
object reference-set digest/count
integrity validation report
```

No mixed profile/reference generations.

## 3. Domain boundary

`wow-reference` owns:

- normalized API/UI/restriction/source-map logical records;
- completeness/coverage/provenance semantics;
- record encoding/decoding and query adapter;
- domain validation expectations;
- exact ReferenceGeneration/Profile identity.

`wow-store` owns:

- static schema/operation bundle registration/validation;
- staging database lifecycle/transactions;
- physical SQLite/open profile;
- generic metadata/migration ledger;
- integrity/file/object/publication/retention;
- read-only opening and typed prepared-operation transport.

Missing rows never create negative authority inside store.

## 4. Build input

```text
ReferenceStoreBuildRequest
    exact StoreConfiguration
    exact ProfileIdentity / ReferenceGenerationId
    exact reference domain schema bundle/operation catalog
    deterministic domain write operation plan
    expected logical input manifest/digests/counts
    object write/reference plan
    domain validation catalog
    budgets/cancellation
    requested durability level
```

No arbitrary SQL, source path scan, or network fetch.

## 5. Build flow

```text
create candidate/staging store
-> apply metadata + exact reference schema migration path
-> execute deterministic registered domain write operations
-> create/verify object payloads and candidate reference set
-> run schema/migration/foreign-key/domain/integrity checks
-> finalize logical/file/object manifests and StoreGenerationId
-> seal candidate
-> publish immutable generation
-> reopen/validate final path read-only
-> atomically update active pointer when requested
```

## 6. Sealed artifact layout

Conceptual generation directory:

```text
reference/<StoreGenerationId>/
    reference.sqlite
    store-manifest.json
    integrity-report.json or referenced object/report
    checksums.json
```

Objects may live in shared ObjectStore by ObjectId and are referenced by manifest. Exact layout freezes in implementation.

No:

```text
-journal
-wal
-shm
temp DB
mutable lock/state file
private absolute source path
```

## 7. Seal requirements

- no active transaction/connection with write capability;
- all staged writes committed and validated;
- migration ledger/schema digest exact;
- foreign keys/integrity/domain checks pass;
- object references present/verified;
- no unexpected schema object/sidecar;
- SQLite file final digest/length known;
- StoreManifest finalized and self-consistent;
- generation directory content complete;
- intended read-only/open-mode behavior tested;
- no cancellation/truncation/budget gap.

## 8. Publication

Published generation path is derived from validated StoreGenerationId. Existing same path:

- exact equivalent manifest/file/object set -> idempotent publication;
- any mismatch -> corruption/collision, fail/quarantine; no overwrite.

After final-path publication, reopen with sealed reader profile and rerun required validation before active-pointer update.

## 9. Active pointer

One StoreId can have an active generation pointer. It records exact generation/path/manifest digest and previous active ID.

Pointer does not define the generation identity and can change while older readers retain/open the old generation explicitly.

Failure before pointer replacement leaves old active generation. Published but inactive new generation can be recovered/activated later only after exact revalidation.

## 10. Read-only open

```text
open_reference_store(generation selector/exact active pointer)
-> resolve normalized generation path
-> open read-only/query-only/immutable policy
-> validate manifest/profile/reference/store/schema/runtime/file/object identities
-> validate no sidecars/write capability
-> build read transaction/prepared operation view
```

Consumers cannot obtain raw writable connection or arbitrary SQL.

## 11. Read consistency

A reader holds one immutable StoreGenerationId. Active pointer changes do not switch it. Every returned read/report retains generation/profile/context.

Because the store is immutable, concurrent readers require no writer coordination, but file replacement/deletion/retention must respect open-generation leases and platform semantics.

## 12. Read operation catalog

`wow-reference` registers static read operation descriptors through the schema bundle. Store validates operation ID/catalog/store state/parameter/result/budget and executes prepared statements.

No application/service source text/raw SQL. Domain adapter decodes result rows.

## 13. Validation on open

Configurable levels:

### Mandatory fast open

```text
manifest/schema/runtime/generation/profile IDs
file presence/length and manifest digest relationship
schema metadata/ledger expected state
no unexpected sidecars
read-only profile effective
```

### Periodic/full activation/release validation

```text
full file checksum
foreign-key check
quick/integrity check per policy
domain validation catalog
object manifest/payload/decode/logical digest checks
```

The selected level and retained evidence are explicit. A missing required full validation is not pass.

## 14. Corruption/mismatch

Reject store on:

- file/manifest/object digest mismatch;
- schema/ledger/object set mismatch;
- profile/reference/generation mismatch;
- unexpected sidecar/schema object;
- foreign-key/integrity/domain validation failure;
- write/open-profile violation;
- path/root/symlink escape;
- unsupported SQLite runtime profile.

Do not edit/recover in place. Higher layer selects prior last-known-good or rebuilds new generation.

## 15. Migration/update

To update profile/reference/schema/content:

```text
build new store generation from authoritative logical inputs
validate/seal/publish
activate new pointer
retain old generation per policy
```

No in-place ALTER/data update against released store, even if SQLite supports it.

## 16. Retention

Retain at least:

```text
active generation
last-known-good until new generation activation/open verification and policy allows deletion
configured historical/reference target generations
any generation with active reader/object lease
published generation under recovery/investigation
```

Deletion requires exact generation/object reference/lease scan. No age-only deletion.

## 17. Determinism

Canonical ReferenceStore identity/manifests are independent of:

```text
input record order where unordered
temp/staging root
worker scheduling
build timestamp/host
active pointer state
SQLite page order unless reproducible export proved
```

Logical record/object/schema manifests are authoritative deterministic surfaces. Raw SQLite file byte reproducibility is separately tested/reported.

## 18. Budgets

Bound:

- schema/migration operations;
- row/record/object counts and sizes;
- SQLite file/page size;
- transaction batch size;
- validation/check work;
- read result rows/bytes;
- open/manifest/object verification work.

Budget/truncation before complete seal/open validation rejects build/open; never publishes partial store.

## 19. Required operations

```text
build_reference_store_candidate
write_reference_store_registered_operations
validate_reference_store_candidate
seal_reference_store
publish_reference_store_generation
activate_reference_store_generation
resolve_active_reference_generation
open_reference_store_read_only
validate_reference_store_open
execute_reference_store_read_operation
close_reference_store_read_view
retain_release_reference_generation
```

## 20. Required tests

- exact successful build/seal/publish/open;
- mixed profile/reference generation rejected;
- active pointer changes while existing reader retains old generation;
- write/DDL attempt on sealed reader fails/no sidecar/file change;
- in-place released migration rejected;
- manifest/file/schema/object/profile mismatch rejected;
- unexpected sidecars/schema objects rejected;
- final-path reopen failure prevents pointer update;
- same generation exact idempotent publication;
- same ID mismatch rejected/no overwrite;
- old active retained after every publication failure point;
- open validation levels explicit/not falsely pass;
- domain row absence never yields store negative-authority decision;
- random input/temp order -> same logical generation/manifest ID;
- GC cannot remove active/leased/configured historical generation objects.

## 21. Hard stops

- no mixed/floating profile;
- no writable/open released store;
- no in-place migration/update;
- no sidecar release artifact;
- no raw SQL/connection to application;
- no absent-row authority;
- no activation before final-path validation;
- no corruption auto-repair;
- no reader generation switch;
- no deletion under retention/lease uncertainty.
