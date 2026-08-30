# `wow-store` E1-A implementation plan

**Status:** ordered handoff plan for a future coding agent. This documentation change adds no Rust/SQL implementation.

## Phase 0 — prerequisites and selection

1. Confirm E0-A `wow-core` implementation/canonicalization/ID vectors are merged.
2. Confirm detailed E1 `wow-reference` persistent schema/operation/build-plan contract exists before activating the real domain bundle.
3. Read every file listed in [`AGENTS.md`](AGENTS.md).
4. Audit/select one Rust SQLite binding and exact version/features/license.
5. Materialize official SQLite/binding docs/source needed for capability probe.
6. Confirm no competing storage/SQLite/object implementation exists.
7. Confirm ProjectStore remains Deferred.

**Gate:** no store code begins without an explicit SQLite/binding candidate and synthetic/reference schema fixture plan.

## Phase 1 — minimal crate skeleton

Create only responsibilities such as:

```text
configuration
path_policy
sqlite_adapter
runtime_probe
schema_registry
migration
operation_catalog
transaction
validation
manifest
publication
reference_store
object_store
retention
error
fixture
```

Rules:

- framework dependency exactly `wow-core`;
- no domain types/semantics;
- no ProjectStore/WAL module beyond typed deferred operation if needed;
- no raw SQL/connection public API;
- no generic database engine abstraction/plugin system;
- no network/process/source execution.

**Gate:** crate compiles with storage primitives only and no placeholder successful ProjectStore.

## Phase 2 — path/platform adapter

Implement:

```text
validate_store_root_and_path_policy
resolve_confined_store_path
same_volume_check
atomic_no_replace/replace adapter
file/directory flush adapter
permission/open no-follow adapter
error normalization
```

Run root/path/link/volume/race fixtures.

**Gate:** staging/final/pointer/object paths cannot escape; achieved atomic/durability capabilities reported exactly.

## Phase 3 — SQLite binding pin and runtime probe

Implement:

```text
probe_sqlite_runtime
validate_sqlite_runtime_probe
build_sqlite_runtime_profile
validate_sqlite_runtime_profile
apply_and_verify_connection_profile
apply_runtime_limits
```

Freeze exact:

```text
SQLite version/source ID
binding/version/features
compile options/capabilities
open flags/URI behavior
PRAGMA profiles
limits
error mappings
```

Run all `SQLITE-*`.

**Gate:** read-only/foreign-key/transaction/integrity/security capabilities proven; lost mandatory capability blocks activation.

## Phase 4 — store configuration and metadata schema

Implement:

```text
validate_store_configuration
build_store_configuration_id
register standard metadata schema bundle
register metadata operation catalog
build/validate schema registry
```

Freeze metadata schema/operation IDs/digests.

Run `STORE-CONFIG-*`, metadata portions of `SCHEMA-*`.

**Gate:** domain-neutral metadata only and canonical registry.

## Phase 5 — schema/migration framework

Implement:

```text
validate/register SchemaBundle
validate/register PreparedOperationCatalog
build/validate MigrationGraph
plan/validate MigrationPlan
apply migration plan to staging
record/validate ledger
inspect normalized schema
compute/validate canonical schema digest
```

Use a small synthetic domain-neutral fixture bundle until actual `wow-reference` bundle freezes, then replace/extend integration fixture without weakening the generic tests.

Run all `SCHEMA-*`, `MIG-*`.

**Gate:** exact empty-to-target path, no skip/force/dynamic SQL/in-place released migration.

## Phase 6 — transaction and staging lifecycle

Implement:

```text
create_store_build_candidate
create_staging_store
begin write transaction
execute registered operation batch
commit/rollback
cancel/abort/quarantine cleanup
```

Run `BUILD-*` before publication code.

**Gate:** one candidate writer, no partial exposure, exact transactional/rollback state.

## Phase 7 — logical object identity and atomic payload write/read

Implement:

```text
validate ObjectId
stream hash/encode/write temp
validate payload/logical digests/length
atomic publish/dedup/existing mismatch handling
manifest publication/read/verification
resource limits/cancellation/path safety
```

Freeze codec only after review/probe. Raw/no-compression codec is acceptable for first vertical proof; compression is not required merely because final pack plans `.zst` artifacts.

Run `OBJECT-001..014`, security limits.

**Gate:** known hash/read/dedup/corruption/path/bomb vectors pass; no overwrite mismatch.

## Phase 8 — object references and GC safety

Implement:

```text
build/validate reference set
record candidate/published generation references
reader/object leases supported at declared process scope
classify GC eligibility
bounded conservative GC
orphan scan/quarantine
```

Run `GC-*`.

**Gate:** no referenced/leased/unknown deletion; age never authority.

## Phase 9 — candidate validation and manifest

Implement:

```text
schema/ledger/FK/quick-or-integrity/application checks
file digest/length
object closure/digests
StoreValidationReport
StoreManifest / StoreGenerationId noncyclic identity
```

Run `VALIDATE-*`.

**Gate:** all mandatory checks pass; skipped/unavailable is not pass; no repair.

## Phase 10 — sealing

Implement:

```text
close/flush transaction/connection
checkpoint/finalize/remove allowed sidecars
final file/object/manifest verification
seal report
immutable no-write state
```

Run seal/write-after-seal/sidecar fixtures.

**Gate:** sealed generation complete and immutable before publication.

## Phase 11 — generation and pointer publication

Implement state machine exactly:

```text
publish generation final path
reopen/validate final path
build/write/flush/atomic replace active pointer
reopen/validate pointer+generation
record durability/publication
```

Inject failure/cancellation at every `PUB-*` point.

**Gate:** prior active always valid until final pointer success; no mismatch overwrite; idempotent retries exact.

## Phase 12 — sealed ReferenceStore reader

Implement:

```text
resolve active/exact generation
open read-only/query-only/verified immutable policy
validate manifest/schema/runtime/file/object/sidecars
execute registered read operations
hold/release generation/object retention lease
```

Run all `REFSTORE-*`.

**Gate:** no write/sidecar/switch/raw connection/absent-row authority.

## Phase 13 — security/integrity corpus

Run all `STORE-SEC-*` plus fixtures in `INTEGRITY_AND_SECURITY.md`:

```text
malicious DB/schema/object/path/link/SQL/size/race/corruption/leak
```

**Gate:** no execution/escape/unbounded resource/repair/activation/leak.

## Phase 14 — deterministic reports and physical reproducibility classification

Run randomized order/temp/worker/codec metadata fixtures.

Freeze:

```text
runtime profile
schema registry/migration graph/ledger
logical data/object/reference manifests
StoreGenerationId/StoreManifest
publication/pointer/integrity reports
```

Measure whether raw SQLite file bytes are reproducible. Record one of:

```text
byte_reproducible_proven
logical_reproducible_only
unresolved
```

Never fabricate byte reproducibility.

## Phase 15 — actual `wow-reference` E1 integration seam

Once its detailed contract exists:

- register exact reference schema/operation bundle;
- consume deterministic build/write/object plan;
- build one fixture ReferenceStore;
- validate domain checks through registered catalog;
- publish/open/read exact fixture profile/reference generation;
- confirm store never imports reference domain types or decides coverage/authority.

Run cross-crate integration fixtures.

## Phase 16 — ProjectStore deferred enforcement

Implement only typed unavailable surface/test if exposed.

Run `PROJECTSTORE-DEFER-*` and static dependency/module checks.

**Gate:** no WAL/project tables/physical model/writer actor in E1-A.

## Phase 17 — fixture/checksum freeze

Before or with first implementation commit:

1. freeze core implementation bundle;
2. freeze SQLite/binding/platform adapter/profile/probe;
3. freeze metadata and consumer schema/operation/migration bundles;
4. freeze candidate/store/generation/manifest/pointer/integrity/publication vectors;
5. freeze object logical/payload/reference/GC vectors;
6. freeze crash/cancellation outcomes and prior active identities;
7. canonicalize every example;
8. write member/bundle SHA-256 values;
9. update `CONTRACT.json` and manifest implementation state;
10. reject null values after activation.

Tests verify fixtures; they never rewrite them automatically.

## Phase 18 — completion report

Report:

```text
SQLite/binding/platform adapter pin and capability profile
schema/operation/migration IDs/digests
active direct dependency and domain-boundary proof
ReferenceStore candidate/seal/generation/pointer/open IDs
publication crash/cancellation outcomes and old active preservation
object hash/codec/payload/reference/GC vectors
durability and raw-byte reproducibility classification
integrity/security test results
all commands/tests: pass | fail | skipped
ProjectStore and other deferred capabilities
```

## Forbidden shortcuts

Do not:

- select a binding/version without probe;
- expose raw SQL/connection;
- execute SQL from source/user/external input;
- import domain semantics;
- mutate released ReferenceStore;
- use `user_version` as sole authority;
- skip/force/repair migrations;
- publish before seal/final-path validation;
- update pointer before generation publication;
- overwrite ID mismatch;
- treat age as GC authority;
- delete referenced/leased/unknown object/generation;
- claim power-loss/raw-byte determinism without proof;
- activate ProjectStore/WAL;
- change fixtures to suit easier behavior.

## Completion boundary

E1-A ends with a verified storage foundation, one immutable ReferenceStore vertical publication/open path, and one safe content-addressed object path. Mutable project/external stores, search/FTS semantics, release signing/distribution, and CI remain outside this work package.
