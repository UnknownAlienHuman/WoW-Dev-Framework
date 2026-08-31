# ProjectStore schema, operation, and validation composition

**Status:** normative owner-separated composition contract.

## Bundle set

E2-D composes:

```text
wow-store metadata/partition/publication bundle
wow-project logical project-index bundle
wow-graph logical graph bundle
optional owner-approved materialized-query-index bundles
```

Each bundle is repository-owned, versioned, digest-bound, and registered before opening or writing the epoch.

## Store-owned bundle

Owns only generic epoch/schema/catalog ledgers, immutable partition manifests, generation membership, store generation/publication set metadata, inactive validation/current records, retention/GC/object references, and generic integrity constraints.

It contains no API names, source AST semantics, event/frame meaning, graph relation meaning, findings, or rule policy.

## Project-owned bundle

Expected families:

```text
source/root/universe/package/file manifests
TOC variants/directives/files/dependencies/SavedVariables
XML documents/includes/templates/objects/inheritance/scripts
physical and virtual Lua unit manifests
analyzer snapshot/fact/finding bindings
load units/edges/reachability
recognizer input/output and graph-proposal-validation mappings
invalidation/reuse/removal manifests
ProjectSnapshot and project read indexes
project capability/coverage/conflict/deferred records
```

`wow-project` defines exact schema and prepared-operation semantics.

## Graph-owned bundle

Expected families:

```text
graph registry manifest
entity/relation assertions
producer partition manifests
conflicts and coverage
GraphSnapshot manifest
exact/reverse/axis query indexes
graph validation/golden-query catalog
```

`wow-graph` defines exact schema and operation semantics.

## Prepared operations

Each descriptor freezes:

```text
operation ID/version/digest
owner bundle
phase: partition-build | generation-build | validation | activation | retention | GC | read
parameter/result schemas
allowed epoch/generation/partition states
cardinality and byte budgets
transaction requirement
idempotency policy
```

No caller submits SQL or table names.

## Partition insertion

Domain insertion operations must require exact `PartitionVersionId`, verify row/manifests/counts/digests, reject writes to existing sealed versions unless equivalent, never read another generation implicitly, and preserve source/evidence/coverage closure.

## Generation membership

Store-owned operations write the complete target membership only after all referenced partition versions are sealed and present. Missing, duplicate, or conflicting keys fail.

## Validation catalogs

Generic checks include:

```text
schema objects/digests/ledger
foreign keys and owner closure
partition state/content manifests
complete membership/no duplicate key
publication-set/store-generation identity closure
object references
current-record referential closure
cross-generation leakage sentinels
SQLite integrity policy
```

Project and graph checks come from owner catalogs. A missing owner check is not pass.

## Read catalogs

Registered exact operations resolve active partition versions, project source/load/analyzer/recognizer records, graph assertions/conflicts/coverage, snapshot manifests, and bounded owner queries. Domain crates decode domain types; store does not expose raw rows.

## Compatibility

- bundle/canonicalization/operation changes alter schema-set identity;
- unsupported bundle combinations reject epoch open;
- additive compatibility is explicit and tested;
- breaking change normally creates a new epoch;
- no silent operation fallback;
- no old catalog interpreting new rows;
- no schema introspection used to invent domain behavior.
