# `wow-graph` contract router

**Status:** E2-A typed graph contract is implementation-ready; Rust implementation has not started.

`wow-graph` owns the versioned WoW domain graph schema, semantic entity identity, evidence-bearing entity/relation assertions, atomic producer-partition replacement, immutable graph snapshots, independent graph axes, and deterministic bounded queries. It does not parse source, run recognizers, publish project generations, rank search results, or expose raw storage internals.

## Canonical route

Read the E2-A package in this order:

1. [`e2/README.md`](e2/README.md)
2. [`e2/AGENTS.md`](e2/AGENTS.md)
3. [`e2/DECISIONS.md`](e2/DECISIONS.md)
4. [`e2/DATA_MODEL.md`](e2/DATA_MODEL.md)
5. [`e2/KIND_AND_RELATION_REGISTRY.md`](e2/KIND_AND_RELATION_REGISTRY.md)
6. [`e2/IDENTITY_AND_ASSERTIONS.md`](e2/IDENTITY_AND_ASSERTIONS.md)
7. [`e2/PARTITIONS_AND_PUBLICATION.md`](e2/PARTITIONS_AND_PUBLICATION.md)
8. [`e2/AXES_AND_VIEWS.md`](e2/AXES_AND_VIEWS.md)
9. [`e2/QUERY_MODEL.md`](e2/QUERY_MODEL.md)
10. [`e2/PERSISTENCE_BOUNDARY.md`](e2/PERSISTENCE_BOUNDARY.md)
11. [`e2/CONFLICT_COVERAGE_AND_PROVENANCE.md`](e2/CONFLICT_COVERAGE_AND_PROVENANCE.md)
12. [`e2/SECURITY_AND_BUDGETS.md`](e2/SECURITY_AND_BUDGETS.md)
13. [`e2/ERROR_MODEL.md`](e2/ERROR_MODEL.md)
14. [`e2/TEST_MATRIX.md`](e2/TEST_MATRIX.md)
15. [`e2/IMPLEMENTATION_PLAN.md`](e2/IMPLEMENTATION_PLAN.md)
16. [`e2/CONTRACT.json`](e2/CONTRACT.json) and [`e2/examples/`](e2/examples/README.md)

Also read [`../AGENTS.md`](../AGENTS.md), [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md), [`../WORKSTREAMS.md`](../WORKSTREAMS.md), and the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes.

## Direct framework dependencies

```text
wow-core
wow-store
```

`wow-project` and `wow-recognizers` call the graph through public contracts; `wow-graph` does not depend on them. `wow-search`, `wow-context`, and `wow-service` remain higher-level consumers.

## Current implementation state

```text
documentation contract: complete
closed fixture shapes: complete
required prerequisite pins and SHA-256 freeze: pending implementations
Cargo workspace activation: not started
Rust source: absent
```

Directory presence does not authorize later E3/E4 lineage, impact, search, source-skeleton, external-candidate, or runtime features.
