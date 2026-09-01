# E4-B lineage persistence boundary

**Status:** normative logical schema and registered-operation contract. Physical storage remains owned by `wow-store`.

## Ownership

`wow-graph` owns:

- lineage relation/change/migration/impact schemas and invariants;
- logical record and index definitions;
- complete producer-partition replacement semantics;
- lineage snapshot identity and manifests;
- registered write/read/validation operation catalogs;
- exact query and golden-result semantics.

`wow-store` owns:

- SQLite library/binding/runtime/VFS/compile options;
- physical database/epoch/files/transactions/WAL/checkpoint;
- one-writer enforcement;
- immutable partition materialization;
- read snapshots and generation leases;
- durability, backup, restore, recovery, retention and garbage collection;
- physical row/page/index layout.

## Separate overlay

E4-B lineage data is a separate immutable logical overlay. It does not write cross-generation records into generation-local E2 GraphSnapshots.

The selected physical placement can be:

```text
separate LineageStore epoch
or
reviewed lineage namespaces/partitions inside a compatible shared store epoch
```

The exact choice must be measured and frozen before implementation. Either choice must preserve independent immutable snapshot identity, atomic producer-partition publication, exact readers and retention closure. Documentation does not claim one is already benchmarked.

## Logical record families

```text
lineage_registry_manifest
lineage_universe_set
lineage_input_partition
lineage_generation_entity_ref
lineage_proposal
lineage_candidate_component
lineage_review_decision
lineage_assertion
lineage_conflict
lineage_change_set
lineage_change_record
lineage_absence_decision
lineage_migration_candidate
lineage_migration_recipe
lineage_static_impact_index_or_cache_manifest
lineage_snapshot_manifest
lineage_validation_report
```

Derived indexes/materializations must be deterministically rebuildable from authoritative logical records.

## Required logical indexes

- exact before/after generation entity lookup;
- proposals by source, target, relation, producer partition and component;
- components by entity and comparison scope;
- accepted assertions by source/target/relation/proof class;
- review/conflict/evidence/coverage refs;
- change records by entity pair/change kind/facet;
- removal/introduction decisions by closed scope;
- replacement/migration records by source/target;
- impact roots and exact bounded adjacency/path seeds;
- deterministic ordered pagination;
- stale producer-partition replacement/removal.

Index/table names remain physical details.

## Registered write operations

```text
insert_or_validate_lineage_registry
insert_or_validate_lineage_universe_set
replace_lineage_input_partitions
replace_lineage_proposal_partitions
replace_lineage_components_and_conflicts
insert_or_replace_lineage_review_partition
replace_accepted_lineage_assertion_partitions
replace_change_absence_migration_partitions
publish_lineage_snapshot_manifest
retain_or_remove_lineage_generation
```

All operations are schema-bound and parameterized. Caller-provided SQL is forbidden.

## Registered read operations

```text
open_lineage_snapshot
lineage_entity_ref_exact
lineage_proposals_by_entity_or_component
lineage_component_exact
lineage_assertions_by_entity_relation_or_generation
lineage_review_and_conflict_records
lineage_change_set_and_records
lineage_absence_decisions
lineage_migration_records
lineage_snapshot_manifest_and_coverage
lineage_ordered_neighbors_and_paths
```

Complex impact/trace algorithms may use graph-owned bounded in-memory traversal over registered reads. They do not load the full store by convenience.

## Generation publication

```text
validated LineagePublicationRequest
-> logical complete target partition membership
-> registered store operation plan
-> one-writer inactive transaction/publication
-> committed PublishedInactive lineage generation
-> fresh exact read snapshot
-> graph-owned logical and golden-query validation
-> immutable sealed LineageGraphSnapshot
```

Current remains an outer service/store concern. Failed targets stay inactive/quarantined under their own IDs.

## Idempotency

An operation ID binds one canonical request digest and exact base/input generations. Retry with same ID/digest returns/resumes the durable exact state; same ID/different digest is rejected.

Response loss after commit/validation cannot cause duplicate partition publication or a second different snapshot. Durable operation/receipt records preserve state.

## Read snapshots

One `LineageGraphView` binds one exact store snapshot and generation lease. Existing readers remain on the old snapshot while a new generation publishes. A query cannot mix proposal/assertion/change/index rows from different lineage generations.

## Retention roots

```text
active or explicitly selected lineage snapshot
last-known-good retained snapshot
generation read lease
active continuation
review/audit/evidence source
migration/impact result reference
evaluation corpus/debug pin
recovery/quarantine subject
backup/restore root
policy pin
```

GC proves closure:

```text
lineage generation
-> registry/universe/input/proposal/component/review/assertion/conflict/change/migration partitions
-> owner graph/source/reference/search evidence handles
-> content-addressed objects
```

It must not delete referenced owner generations; owner-store retention admission is coordinated above the storage primitives.

## Breaking schema/profile changes

A breaking lineage registry/schema/physical/runtime profile change creates a new compatible epoch or explicitly migrated immutable generation according to the store contract. Published snapshots are never migrated in place.

## Validation after publication

- exact schema/runtime/profile IDs;
- complete membership and expected counts/digests;
- no stale producer records;
- every entity/proposal/assertion/change/migration/evidence ref resolves;
- proof/review/conflict/coverage closure;
- no generation-local graph mutation;
- no cross-universe or cross-generation leakage;
- reverse/index consistency;
- golden compare/trace/change/migration/impact queries;
- read-only enforcement;
- logical determinism.

## Physical reproducibility

Logical IDs, manifests, queries and results must be deterministic. SQLite bytes/WAL/page order are separately classified after executable probes. Physical differences cannot alter semantic lineage output.

## Forbidden interfaces

No raw:

```text
SQL
FTS MATCH string
connection/transaction callback
rowid/table/index/PRAGMA
extension or VFS handle
absolute physical path
mutable storage object
```

crosses the E4-B public API.
