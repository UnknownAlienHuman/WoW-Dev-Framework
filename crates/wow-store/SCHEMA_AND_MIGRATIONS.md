# Schema bundles and migrations

**Status:** normative E1-A schema registry and migration execution contract.

## 1. Boundary

`wow-store` owns how registered schemas are validated, applied, recorded, and verified. Consumer crates own the logical meaning and typed encoding/decoding of their records.

The storage layer never accepts SQL/DDL from:

```text
analyzed source
user configuration
external repositories
MCP/LSP/CLI requests
downloaded pack metadata
runtime network responses
```

Only static repository-owned, versioned schema/operation bundles compiled into approved framework components are executable.

## 2. Standard metadata schema

E1-A store metadata namespace conceptually contains:

```text
store_meta
schema_bundle
schema_migration
store_generation
publication_record
integrity_record
object_manifest
object_payload
object_reference
retention_record
```

Exact table/column/index names freeze with the first implementation schema. Metadata remains domain-neutral.

Mandatory metadata relations/invariants:

- one store ID/kind and one candidate/published generation identity;
- exact schema registry/bundle/migration ledger;
- exact SQLite runtime profile;
- exact logical/file/object manifest digests;
- publication/seal/integrity state;
- object references scoped to retained generations;
- no domain entity/restriction/project/finding interpretation.

## 3. Domain schema bundle

```text
SchemaBundle
    namespace
    version
    owner contract/version
    parent versions
    declared schema objects
    required SQLite capabilities
    migration edges
    prepared operation catalog
    validation check catalog
    destructive-change policy
    expected schema digest
```

Bundle ID/digest includes normalized semantic declarations, not source-file order/comments/whitespace.

## 4. Schema object declaration

```text
SchemaObjectDeclaration
    object kind: table | index | trigger | virtual table
    canonical name/namespace
    static SQL or structured declaration digest
    owning bundle/version
    required capability
    expected normalized sqlite_schema representation/digest
```

Rules:

- no unqualified cross-namespace collision;
- no SQLite internal/reserved name abuse;
- no temporary schema object in sealed ReferenceStore;
- trigger/virtual-table use requires explicit capability/security review;
- FTS/virtual-table declaration is inactive until its milestone and capability probe;
- unexpected object is validation failure.

## 5. Operation catalog

A bundle defines repository-owned static prepared operations:

```text
PreparedOperationDescriptor
    operation ID
    static statement digest
    parameter schema
    result schema/cardinality
    allowed store kind/state
    transaction requirement
    budget policy
```

Domain adapters bind typed encoded values. Store validates operation ID/catalog/store state and uses prepared statements. Raw SQL string is never supplied by an application/transport/source at runtime.

## 6. Migration graph

```text
SchemaMigrationGraph
    namespace
    versions
    directed MigrationEdge set
    initial/empty version
    supported targets
    canonical digest
```

Required:

- finite and acyclic;
- all edge endpoints declared;
- no duplicate migration ID/from-to semantic edge;
- each supported source-target has an explicit selected path;
- ambiguous multiple paths require a registered deterministic path policy or are rejected;
- no implicit skip by changing `user_version`/metadata only;
- no unknown historical state acceptance;
- no downgrade unless separately declared/tested (none required in E1-A).

## 7. Migration plan

```text
MigrationPlan
    plan ID
    store kind/candidate generation
    source schema registry/bundle versions/digests
    target versions/digests
    ordered migration edge IDs
    required SQLite capabilities
    preflight checks
    transaction grouping
    destructive/data transform declarations
    post-migration validation checks
    expected target schema digest
```

Plan identity excludes temp path/time.

## 8. Empty-to-target E1 path

E1-A must support:

```text
no database / empty staging file
-> wow-store metadata schema v1
-> registered wow-reference E1 domain schema bundle target
```

The exact reference domain bundle is supplied/frozen by the next `wow-reference` E1 contract. Store tests may use a synthetic domain-neutral fixture bundle until that contract is frozen, but implementation activation requires the actual registered bundle identity.

## 9. Preconditions

Before applying a plan:

- store is staging/candidate, never sealed/published ReferenceStore;
- source schema state/metadata/digest matches plan exactly;
- SQLite runtime profile/capabilities match;
- no unrecognized schema object/sidecar/ledger row;
- file/root/size budgets valid;
- no active write transaction;
- cancellation not requested;
- operation/schema catalogs match compiled registry.

Mismatch fails; do not guess or repair in place.

## 10. Transaction policy

By default, apply the complete candidate migration path in one outer write transaction where SQLite/operation semantics allow.

If a specific SQLite operation cannot participate atomically:

- declare it explicitly in migration edge;
- split into named stages with checkpoint/restore/validation semantics;
- never perform it against a released active ReferenceStore;
- keep all work in staging;
- failure destroys/quarantines candidate and leaves active generation unchanged;
- test every interruption boundary.

No implicit auto-commit surprises are accepted without probe/test.

## 11. Migration ledger

Each applied edge records:

```text
migration ID
namespace/from/to
migration/operation digest
registry/bundle digest
application ordinal
transaction/report ID
result status
post-check IDs
```

Ledger is written transactionally with schema/data change. Target metadata is not advanced before the edge succeeds and validates.

## 12. Target schema verification

After migration:

- normalize/inspect `sqlite_schema` objects;
- compare expected object set/kinds/names/digests;
- verify metadata registry/bundle versions/digests;
- verify migration ledger path/order/digests;
- verify required indexes/triggers/capabilities;
- reject unexpected/missing objects;
- run foreign-key/schema/application checks;
- compare expected target canonical schema digest.

`PRAGMA user_version` may be used as a convenience marker but is never the sole authority.

## 13. Data transformations

Migration transform must declare:

```text
input/output table/column/record shape
lossless/lossy/destructive classification
row/cardinality expectations
null/default/collation/encoding policy
validation queries
budget bounds
rollback/abort semantics
```

E1 empty-to-target should avoid destructive transforms. Later project migrations require dedicated fixtures.

## 14. Immutable ReferenceStore rule

A sealed/published ReferenceStore target requiring another schema version is handled by:

```text
open old store read-only if needed
build new staging store at target schema
domain adapter exports/rebuilds exact logical records
validate logical equivalence/migration requirements
seal/publish new generation
retain old generation by retention policy
```

Never open old released file writable or mutate it in place.

## 15. Unknown/tampered migration state

Reject:

- ledger edge not in registry;
- digest mismatch;
- version advanced without edge;
- missing expected edge;
- duplicate/out-of-order edge;
- schema object set inconsistent with ledger;
- current DB schema newer/unsupported;
- edited static SQL/catalog digest;
- unknown attached/temp schema objects;
- partial migration markers.

No “force” flag in E1 service/application surface.

## 16. Cancellation/failure

- cancellation checked before plan/stage/edge and at bounded operation checkpoints;
- transaction rollback on cancellation/failure;
- candidate not sealed/published;
- migration ledger/target metadata not partially committed;
- prior active generation/pointer unchanged;
- candidate temp DB/object refs cleaned or quarantined according to policy;
- no background continuation.

## 17. Determinism

Equivalent source/target bundles and logical operation inputs produce equivalent canonical:

```text
migration plan ID/edge order
ledger records
normalized target schema digest
validation report
```

Do not include SQL source formatting/comments, hash-map order, temp paths, timing, row insertion order where semantics are unordered.

## 18. Required operations

```text
validate_schema_bundle
register_schema_bundle
validate_prepared_operation_catalog
register_prepared_operation_catalog
build_schema_registry
validate_schema_registry
build_migration_graph
validate_migration_graph
plan_schema_migration
validate_migration_plan
apply_migration_plan_to_staging_store
record_migration_edge
validate_migration_ledger
inspect_normalized_schema
compute_canonical_schema_digest
validate_target_schema
abort_or_quarantine_failed_migration
```

## 19. Required tests

- valid standard + fixture domain bundle;
- duplicate namespace/object/version/edge rejected;
- graph cycle/missing parent/ambiguous path rejected;
- empty-to-target path exact;
- unknown/skipped/tampered edge rejected;
- target metadata advanced before edge mutation -> fail;
- ledger/schema mismatch rejected;
- unexpected/missing schema object rejected;
- required SQLite capability absent -> fail before migration;
- cancellation/failure each edge/stage rolls back candidate only;
- sealed ReferenceStore in-place migration rejected;
- source/user/external SQL injection route absent/rejected;
- static catalog digest mutation rejected;
- normalized schema digest independent of declaration/file order;
- no active pointer change on migration failure.

## 20. Hard stops

- no raw SQL public API;
- no dynamic SQL/DDL from source/user/external input;
- no in-place released migration;
- no `user_version`-only authority;
- no skipped/force migration;
- no partial ledger/target state;
- no domain interpretation;
- no untested nontransactional operation;
- no publish/active pointer before validation.
