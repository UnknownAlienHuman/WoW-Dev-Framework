# `wow-search` contract router

**Status:** E4-A exact-generation retrieval contract is implementation-ready documentation; Rust implementation has not started.

`wow-search` owns immutable generation-bound search projections, safe structured query normalization, exact and approximate retrieval lanes, deterministic candidate fusion, complete ranking explanations, honest miss classification, and stable continuation. It does not own source parsing, reference/project/graph truth, lineage authority, context construction, diagnostics, remediation, service policy, or application transport.

## Canonical route

The original pre-E4 brief is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md).

Read E4-A in this order:

1. [`e4/README.md`](e4/README.md)
2. [`e4/AGENTS.md`](e4/AGENTS.md)
3. [`e4/DECISIONS.md`](e4/DECISIONS.md)
4. [`e4/DATA_MODEL.md`](e4/DATA_MODEL.md)
5. [`e4/SEARCH_UNIVERSE_AND_SHARDS.md`](e4/SEARCH_UNIVERSE_AND_SHARDS.md)
6. [`e4/FIELD_AND_DOCUMENT_SCHEMA.md`](e4/FIELD_AND_DOCUMENT_SCHEMA.md)
7. [`e4/INDEX_BUILD_AND_PUBLICATION.md`](e4/INDEX_BUILD_AND_PUBLICATION.md)
8. [`e4/QUERY_MODEL_AND_NORMALIZATION.md`](e4/QUERY_MODEL_AND_NORMALIZATION.md)
9. [`e4/EXACT_ALIAS_AND_PREFIX_LANES.md`](e4/EXACT_ALIAS_AND_PREFIX_LANES.md)
10. [`e4/TEXT_FUZZY_AND_SHAPE_LANES.md`](e4/TEXT_FUZZY_AND_SHAPE_LANES.md)
11. [`e4/GRAPH_ASSISTED_RETRIEVAL.md`](e4/GRAPH_ASSISTED_RETRIEVAL.md)
12. [`e4/RANKING_FUSION_AND_EXPLANATIONS.md`](e4/RANKING_FUSION_AND_EXPLANATIONS.md)
13. [`e4/MISS_PAGINATION_AND_CONTINUATION.md`](e4/MISS_PAGINATION_AND_CONTINUATION.md)
14. [`e4/PERSISTENCE_AND_FTS5_PROFILE.md`](e4/PERSISTENCE_AND_FTS5_PROFILE.md)
15. [`e4/COVERAGE_AUTHORITY_AND_LINEAGE_BOUNDARY.md`](e4/COVERAGE_AUTHORITY_AND_LINEAGE_BOUNDARY.md)
16. [`e4/SECURITY_AND_BUDGETS.md`](e4/SECURITY_AND_BUDGETS.md)
17. [`e4/EVALUATION_AND_CALIBRATION.md`](e4/EVALUATION_AND_CALIBRATION.md)
18. [`e4/OPERATIONS.md`](e4/OPERATIONS.md)
19. [`e4/ERROR_MODEL.md`](e4/ERROR_MODEL.md)
20. [`e4/TEST_MATRIX.md`](e4/TEST_MATRIX.md)
21. [`e4/IMPLEMENTATION_PLAN.md`](e4/IMPLEMENTATION_PLAN.md)
22. [`e4/CONTRACT.json`](e4/CONTRACT.json) and [`e4/examples/`](e4/examples/README.md)

Also read [`../AGENTS.md`](../AGENTS.md), [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md), [`../WORKSTREAMS.md`](../WORKSTREAMS.md), and the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes before addon-facing work.

## Direct dependencies

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
```

No direct dependency on `wow-context`, `wow-cbm`, `wow-service`, applications, analyzer actors, or recognizer engines is active in E4-A.

## Output boundary

```text
exact owner entities and fields
-> immutable SearchDocument partitions
-> one immutable SearchShard per exact owner generation
-> exact SearchUniverseSet
-> safe NormalizedSearchQuery
-> per-lane SearchCandidateSignal records
-> deterministic SearchCandidate ordering and explanations
-> SearchResult and exact continuation
```

The output is a ranked set of exact entity candidates. It is not proof of user intent, lineage, replacement, migration, impact, safety, or runtime behavior.

## Current implementation state

```text
documentation frontier: E4-A
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
