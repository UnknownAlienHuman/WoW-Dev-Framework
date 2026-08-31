# E2-A graph data model

**Status:** normative.

## Registry bundle

```text
GraphRegistryBundle
    registry_bundle_id/version
    entity kind definitions
    relation kind definitions
    attribute definitions
    axis definitions
    compatibility rules
    canonical digest
```

## Scope

```text
GraphScope
    universe_id
    profile_id: optional by universe
    reference_generation_id: optional
    project_generation_id: optional
    external_generation_id: optional
```

Exactly the fields required by the universe are present. No floating current token.

## Semantic identity

```text
EntityKey
    scope
    entity_kind_id
    canonical identity fields defined by the kind
    canonical digest
```

```text
RelationKey
    scope
    relation_kind_id
    source EntityKey
    target EntityKey
    semantic qualifier fields defined by the relation kind
    canonical digest
```

## Assertions

```text
EntityAssertion
    assertion_id
    entity_key
    producer_id/version
    partition_key
    generation context
    typed attributes
    source handles
    evidence IDs
    provenance/confidence
    coverage record IDs
    competing assertion/conflict refs
    derivation inputs/rule ID
    canonical digest
```

```text
RelationAssertion
    assertion_id
    relation_key
    producer/partition/generation
    typed attributes
    evidence/source/coverage/confidence
    derivation inputs/rule ID
    canonical digest
```

## Partition

```text
GraphPartitionKey
    producer_id/version
    universe/scope
    partition_kind/id
    source capability partition
```

```text
GraphPartitionBatch
    batch_id
    partition_key
    exact source/reference/project generation context
    registry bundle ID
    entity assertions
    relation assertions
    coverage records
    producer diagnostics/conflicts
    expected base graph generation: optional
    budgets/cancellation
    canonical digest
```

## Conflicts

```text
GraphConflictRecord
    conflict_id
    scope/subject key
    conflicting assertion IDs
    conflict kind
    affected capabilities/axes
    status
    resolution policy/result refs
    canonical digest
```

## Snapshot

```text
GraphSnapshotManifest
    graph_generation_id
    graph_id
    registry bundle ID
    exact reference/project/external generation inputs
    ordered partition manifests/digests
    assertion/entity/relation/conflict/coverage counts and digests
    logical store generation ID
    capability summary
    publication/validation report IDs
    canonical digest
```

## Materialized views

```text
GraphEntityView
    EntityKey
    accepted assertion IDs
    conservative typed attribute view
    confidence/provenance/coverage summary
    conflict IDs
```

```text
GraphRelationView
    RelationKey
    accepted assertion IDs
    conservative attributes/confidence/coverage
    conflict IDs
```

Views are query projections, not alternate truth records.

## Axis definition

```text
GraphAxisDefinition
    axis_id/version
    relation kind + traversal direction members
    endpoint constraints
    hierarchy model = single-parent | multi-parent-acyclic | general-directed
    cycle/duplicate policy
    default confidence policy
    canonical ordering key
```

## Query

```text
GraphQueryContext
    exact graph generation/view ID
    universe/profile/generation assertions
    confidence/provenance/coverage policy
    budgets/cancellation
```

```text
GraphQueryResult
    normalized request ID
    entity/relation/path records
    evidence/coverage/conflict refs
    deterministic order
    visited/returned counts
    truncation/continuation
    NotEvaluated/failed scopes
    canonical digest
```

## Persistence plan

```text
GraphStoreReplacementPlan
    exact base/target graph generation
    registry/schema/operation bundle IDs
    ordered partition delete/upsert/assertion/conflict/coverage operations
    expected logical manifests/counts/digests
    validation catalog invocations
    object references
    budgets/cancellation
```
