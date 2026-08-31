# E2-D logical schema and registered-operation boundary

**Status:** normative cross-crate seam.

## Principle

`wow-store` owns physical execution, not project or graph meaning. Domain crates provide immutable versioned bundles that define logical records, indexes, registered operation implementations, query shapes, and validation catalogs. The store validates compatibility and executes those bundles under one physical transaction/profile.

## Bundle families

```text
ProjectLogicalSchemaBundle
ProjectOperationBundle
ProjectQueryBundle
ProjectValidationBundle

GraphLogicalSchemaBundle
GraphOperationBundle
GraphQueryBundle
GraphValidationBundle
```

The concrete type names may be domain-owned. E2-D treats them through generic `StoreSchemaBundle` and `StoreOperationBundle` descriptors plus trusted registered implementations.

## Bundle descriptor requirements

```text
bundle ID/version/owner
contract and minimum store profile compatibility
schema objects and migration/create-from-empty path
registered write/read/validation operations
parameter/result schemas and limits
required constraints/indexes
phase and dependency rules
canonical serialization/digest
license/provenance/tool identity
```

Bundles are repository-owned reviewed code/data. Project source cannot define or modify them.

## New-generation schema construction

Every E2-D generation is created from empty using exact selected schema bundles. It does not migrate a prior project generation image in place.

Schema evolution options:

```text
new generation from canonical logical input under new schema
or
explicit validated logical export/import conversion under a versioned domain plan
```

A prior sealed image remains unchanged.

## Registered operation shape

```text
RegisteredStoreOperationDefinition
    operation ID/version/owner
    operation class = schema | write | read | validation
    accepted parameter/result schema IDs
    allowed phases
    dependency/cardinality limits
    deterministic ordering contract
    required transaction/read mode
    side-effect class
    canonical digest
```

Allowed side effects are limited to the current store transaction/database or operation-owned object staging according to explicit phase. No network/process/editor/client/source execution.

## E2-D phases

```text
0 registry/request preflight
1 generation schema create
2 project base/manifests/source/load/analyzer records
3 graph registries/assertions/conflicts/coverage
4 deterministic derived/index records
5 logical manifest/object-reference closure
6 in-transaction validation
7 commit/checkpoint/seal
8 read-only store validation
9 external project/graph domain attestations
10 registry activation CAS
```

An invocation declares one allowed phase and dependencies. Cycles or cross-phase backward dependencies are rejected.

## Project operation examples

Domain-owned operation IDs may cover:

```text
insert project identity/profile/source snapshot manifests
insert root/universe/package/file/TOC/XML/load records
insert Lua-unit/analyzer/fact/finding manifest refs
insert adapter/recognizer result and proposal-validation reports
insert invalidation/reuse/candidate manifests
insert project capability/coverage/conflict records
build project indexes and project logical manifest
```

These are examples, not store-owned semantic definitions.

## Graph operation examples

Domain-owned operation IDs may cover:

```text
insert exact graph registry bundle
replace/insert project-direct producer partitions
replace/insert recognizer producer partitions
insert entity/relation assertions
insert conflicts and coverage
build deterministic materialized entity/relation views/indexes
insert graph logical/snapshot candidate manifests
```

Graph owns semantic identity and replacement policy; store enforces transaction/operation plan execution.

## Parameters

Only bounded canonical typed values accepted by the operation schema:

```text
core IDs and digests
small enums/booleans/integers/finite numbers
bounded UTF-8 strings
ordered lists/maps with declared element schema
content-addressed object refs
opaque domain records validated by exact schema IDs
```

Forbidden:

```text
SQL text
identifier/table/index/PRAGMA names from caller
filesystem paths
function pointers/closures/callbacks
Lua/JS/Wasm/native code
unbounded JSON/property bags
host/database handles
```

## Plan validation

Before opening a writer where practical, validate:

- exact bundle IDs/digests and store compatibility;
- all operation IDs/versions registered;
- parameters satisfy schema and budgets;
- invocation IDs unique;
- dependency graph acyclic and complete;
- phase ordering valid;
- no ambiguous duplicate/exclusive operation;
- expected logical manifests/counts/digests declared;
- required object refs supplied/verified;
- no missing project/graph mandatory phase;
- no operation from an unselected owner/bundle;
- cancellation/budget/durability policy valid.

## Transaction semantics

All generation write/schema operations execute inside one store-owned transaction. Operation implementations cannot commit, rollback, open another database, change journal/profile, or mutate registry/current pointer.

Failure rules:

```text
one invocation failure -> generation transaction abort
one result-count/digest mismatch -> abort
cancellation before commit -> abort
late work after rollback -> forbidden
```

Object writes can occur before the database transaction but become authoritative only through the committed generation object-reference manifest.

## Query bundles

Read-only domain validation and consumers use registered queries with:

```text
exact StoreGenerationId / StoreImageId
exact query bundle ID/version
query ID
bounded canonical parameters
row/output/time budgets
canonical result schema/order
```

No arbitrary SQL/expressions. Queries cannot switch generations.

## Validation bundles

Store-level generic checks and domain checks remain distinct:

```text
Store validation
    SQLite/schema/constraint/member/object/manifests/read-only-open

Project validation
    project logical closure and golden views

Graph validation
    registry/assertion/endpoint/conflict/coverage/view/query closure
```

A validation operation returns structured IDs/counts/digests/status. Human message text is not identity.

## Result journal

For each invocation the build report retains:

```text
invocation/operation/bundle IDs
phase and dependency IDs
structured outcome
affected logical count/digest
budget usage
error/cancellation ref
```

It excludes raw SQL, private paths, and sensitive record payloads.

## Compatibility

Changing any of these requires a new bundle/version and compatibility review:

- logical record identity/meaning;
- required column/constraint/index behavior;
- operation parameters/results;
- ordering or replacement semantics;
- validation rules;
- query absence/coverage semantics.

Store profile compatibility does not imply domain schema compatibility.

## Architecture tests

- crate dependency remains only `wow-core`;
- no project/graph type imports in store source;
- no public SQL/connection/transaction/PRAGMA/table interfaces;
- every operation/query used appears in exact selected bundle;
- project source cannot register operations;
- one transaction spans all generation writes;
- domain validation remains independently attributable;
- randomized invocation input with equivalent dependency graph yields canonical execution plan and logical result;
- a removed/reordered mandatory invocation changes/fails the expected logical manifest.
