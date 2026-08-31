# `wow-graph` E2-A typed graph contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-graph/e2-a/typed-assertion-partition-query-core`

## Mission

`wow-graph` stores and queries exact graph assertions produced by trusted framework components. The primary truth units are producer-owned assertions with evidence, coverage, confidence, universe, profile, and generation identity. A materialized entity or relation view is derived for one immutable graph snapshot; it is never a lossy overwrite of competing assertions.

```text
versioned kind/relation/axis registry
+ normalized proposed assertions from owning producers
+ exact evidence/coverage/generation context
-> validate semantic identities and endpoint schemas
-> atomically replace producer partitions
-> retain conflicts and independent assertions
-> publish one immutable GraphSnapshot
-> execute exact bounded snapshot-bound queries
```

## E2-A scope

- open versioned entity, relation, attribute, and axis registries;
- semantic entity keys independent of insertion and producer order;
- entity and relation assertion records;
- producer partition manifests and atomic replacement;
- graph conflict, coverage, and publication reports;
- immutable graph generation/snapshot identity;
- logical persistent schema and registered store-operation contracts;
- exact entity lookup, neighbors, axis traversal, bounded paths, bounded subgraph, and relation explanation;
- deterministic ordering, continuation, cancellation, and truncation;
- synthetic project/load/API fixture only.

## Deferred scope

- TOC/XML/Lua parsing and source inventory (`wow-project`/`wow-emmy`);
- recognizer matching (`wow-recognizers`);
- complete Blizzard UI graph and source skeletons (E3);
- lineage inference, patch impact, and migration graph (E4);
- search ranking/FTS (`wow-search`);
- Codebase Memory/external semantic candidates (E6);
- runtime observations unless a later explicit producer contract activates them;
- unbounded administrative graph export.

## Required reading

- [`../../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../../docs/PROVENANCE_AND_COVERAGE.md`](../../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../../docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md)
- [`../../../docs/SECURITY_MODEL.md`](../../../docs/SECURITY_MODEL.md)
- [`../../../docs/TEST_STRATEGY.md`](../../../docs/TEST_STRATEGY.md)
- [`../../wow-store/PROJECT_STORE.md`](../../wow-store/PROJECT_STORE.md)
- [`../../wow-project/README.md`](../../wow-project/README.md)
- [`../../wow-recognizers/README.md`](../../wow-recognizers/README.md)
- current external KB workflow and subsystem router.

## Authority order

1. one exact `wow-core` generation/evidence contract;
2. graph registry bundle selected by the graph build request;
3. producer assertions from an exact project/reference generation and producer partition;
4. retained conflict and coverage records;
5. derived snapshot views and paths;
6. candidate/external assertions only when a later explicit universe/profile enables them.

Graph traversal cannot upgrade evidence or repair missing producer facts.

## Core distinctions

```text
EntityKey
    semantic identity in one universe/profile scope

EntityAssertion
    one producer's evidence-bearing claim about that entity

RelationKey
    semantic directed relation identity

RelationAssertion
    one producer's evidence-bearing claim about that relation

GraphEntityView / GraphRelationView
    snapshot-bound derived view over retained assertions
```

Assertion IDs include producer/partition/evidence identity. Semantic keys do not.

## Direct dependencies

```text
wow-core
wow-store
```

The store edge is activated only for registered logical schema/operation/read-snapshot contracts. No raw SQL or storage implementation leaks through the graph API.

## Public E2-A operations

```text
validate_graph_registry_bundle
validate_graph_partition_batch
plan_graph_partition_replacement
publish_graph_snapshot
open_graph_view
entity_exact
neighbors
traverse_axis
bounded_paths
project_subgraph
explain_entity
explain_relation
```

No public unbounded traversal or “dump everything” operation.

## Completion gate

E2-A code is complete only when randomized assertion/partition order produces the same semantic keys, assertion IDs, snapshot ID, canonical manifests, and bounded query results; stale partition facts disappear atomically; other producers remain intact; conflicts remain visible; cross-universe/profile/generation facts cannot merge; all axes are explicit; candidate/possible assertions stay labeled; and every query has deterministic budgets, truncation, continuation, and evidence closure.
