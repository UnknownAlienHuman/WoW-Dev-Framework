# `wow-search` contract router

**Status:** E4-A core retrieval contract is implementation-ready documentation; no Rust code exists.

`wow-search` owns exact-generation search shards, bounded deterministic retrieval lanes, evidence-preserving candidate fusion, ranking explanations, miss classification, and stable pagination. It does not own source/reference/project/graph truth, context construction, lineage authority, service orchestration, or external semantic candidates.

## Contract history

The original combined E4 scaffold is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md). It mixed core retrieval with future lineage/replacement/impact. The active split is:

```text
E4-A = wow-search core indexing and retrieval
E4-B = explicit cross-generation lineage, migration, replacement candidates, and impact
E4-C = wow-service/apps search operations and explicit search-result-to-context-root handoff
```

## Active E4-A route

1. [`e4/README.md`](e4/README.md)
2. [`e4/AGENTS.md`](e4/AGENTS.md)
3. [`e4/DECISIONS.md`](e4/DECISIONS.md)
4. [`e4/DATA_MODEL.md`](e4/DATA_MODEL.md)
5. [`e4/SEARCH_UNIVERSE_AND_SHARDS.md`](e4/SEARCH_UNIVERSE_AND_SHARDS.md)
6. [`e4/INDEX_BUILD_AND_PUBLICATION.md`](e4/INDEX_BUILD_AND_PUBLICATION.md)
7. [`e4/FIELD_AND_DOCUMENT_SCHEMA.md`](e4/FIELD_AND_DOCUMENT_SCHEMA.md)
8. [`e4/QUERY_MODEL_AND_NORMALIZATION.md`](e4/QUERY_MODEL_AND_NORMALIZATION.md)
9. [`e4/EXACT_ALIAS_AND_PREFIX_LANES.md`](e4/EXACT_ALIAS_AND_PREFIX_LANES.md)
10. [`e4/TEXT_FUZZY_AND_SHAPE_LANES.md`](e4/TEXT_FUZZY_AND_SHAPE_LANES.md)
11. [`e4/GRAPH_ASSISTED_RETRIEVAL.md`](e4/GRAPH_ASSISTED_RETRIEVAL.md)
12. [`e4/RANKING_FUSION_AND_EXPLANATIONS.md`](e4/RANKING_FUSION_AND_EXPLANATIONS.md)
13. [`e4/MISS_PAGINATION_AND_CONTINUATION.md`](e4/MISS_PAGINATION_AND_CONTINUATION.md)
14. [`e4/PERSISTENCE_AND_FTS5_PROFILE.md`](e4/PERSISTENCE_AND_FTS5_PROFILE.md)
15. [`e4/SECURITY_AND_BUDGETS.md`](e4/SECURITY_AND_BUDGETS.md)
16. [`e4/ERROR_MODEL.md`](e4/ERROR_MODEL.md)
17. [`e4/TEST_MATRIX.md`](e4/TEST_MATRIX.md)
18. [`e4/IMPLEMENTATION_PLAN.md`](e4/IMPLEMENTATION_PLAN.md)
19. [`e4/CONTRACT.json`](e4/CONTRACT.json) and [`e4/examples/`](e4/examples/README.md)

## Direct dependencies

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
```

No direct dependency on `wow-context`, `wow-service`, `wow-cbm`, `wow-rules`, `wow-emmy`, `wow-recognizers`, `wow-annotations`, or applications. Search consumes exact public project/reference/graph views and registered storage operations.

## Search architecture

```text
exact owner view for one universe/generation
-> immutable SearchDocument projection
-> generation-local immutable SearchShard

exact SearchUniverseSet of one or more shards
+ normalized structured/text query
-> independent bounded lane results
-> deterministic integer/ordinal fusion
-> ranked SearchCandidate records with full explanations
-> exact detail handles and stable continuation
```

User project, Blizzard UI source, and Reference Pack use separate shards. Raw FTS rank values are never compared across shards. Search federation combines typed lane ordinals/features under a frozen integer profile.

## Hard boundaries

- no floating current/latest inside search core;
- no source/TOC/XML/Lua parsing or raw source fallback;
- no combined mutable global index across unrelated universes;
- no retained-generation rows influencing another generation's FTS corpus;
- no raw user-supplied FTS5 `MATCH` syntax, SQL, regex, callbacks, or plugins;
- no model/embedding/reranker/Codebase Memory in E4-A;
- no hidden addon/repository/path/popularity ranking rules;
- no fuzzy/text/shape/graph score converted into lineage, replacement, platform truth, or intended-entity proof;
- no path materialized as a direct graph edge;
- no authoritative miss without exact query class and complete relevant coverage;
- no full source body or unbounded graph export;
- no `wow-context` invocation or context artifact construction;
- no service/CLI behavior;
- no Cargo/Rust/CI during documentation phase.

## Current implementation state

```text
documentation frontier: E4-A
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
