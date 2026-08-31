# `wow-context` contract router

**Status:** E3-A context projection contract is implementation-ready; Rust implementation has not started.

`wow-context` builds compact, deterministic, evidence-bearing Project Maps, L0/L1 skeletons, progressive context bundles, exact detail routes, faithful bounded source excerpts, coverage/loss records, and context metrics over one exact published project/graph/reference state.

It does not parse source, run analyzers or recognizers, perform search, infer diagnostics/fixes/runtime safety, publish generations, persist caches, or invoke models in the correctness path.

## Canonical route

Read the E3-A package in this order:

1. [`e3/README.md`](e3/README.md)
2. [`e3/AGENTS.md`](e3/AGENTS.md)
3. [`e3/DECISIONS.md`](e3/DECISIONS.md)
4. [`e3/INPUT_VIEWS_AND_SCOPES.md`](e3/INPUT_VIEWS_AND_SCOPES.md)
5. [`e3/DATA_MODEL.md`](e3/DATA_MODEL.md)
6. [`e3/OPERATIONS.md`](e3/OPERATIONS.md)
7. [`e3/PROJECT_MAP.md`](e3/PROJECT_MAP.md)
8. [`e3/SKELETONS.md`](e3/SKELETONS.md)
9. [`e3/CONTROL_AND_EFFECT_MODEL.md`](e3/CONTROL_AND_EFFECT_MODEL.md)
10. [`e3/DETAIL_AND_EXPANSION.md`](e3/DETAIL_AND_EXPANSION.md)
11. [`e3/CONTINUATION_AND_STOPPING.md`](e3/CONTINUATION_AND_STOPPING.md)
12. [`e3/EVIDENCE_COVERAGE_AND_LOSS.md`](e3/EVIDENCE_COVERAGE_AND_LOSS.md)
13. [`e3/SOURCE_EXCERPTS_AND_SECURITY.md`](e3/SOURCE_EXCERPTS_AND_SECURITY.md)
14. [`e3/BUDGETS_AND_TOKENIZATION.md`](e3/BUDGETS_AND_TOKENIZATION.md)
15. [`e3/RENDERING_AND_CANONICALIZATION.md`](e3/RENDERING_AND_CANONICALIZATION.md)
16. [`e3/METRICS_AND_EVALUATION.md`](e3/METRICS_AND_EVALUATION.md)
17. [`e3/ERROR_MODEL.md`](e3/ERROR_MODEL.md)
18. [`e3/TEST_MATRIX.md`](e3/TEST_MATRIX.md)
19. [`e3/IMPLEMENTATION_PLAN.md`](e3/IMPLEMENTATION_PLAN.md)
20. [`e3/CONTRACT.json`](e3/CONTRACT.json)
21. [`e3/examples/`](e3/examples/README.md)

The original pre-E3 scaffold is preserved as [`PRE_E3_OVERVIEW.md`](PRE_E3_OVERVIEW.md).

## Active E3-A direct dependencies

```text
wow-core
wow-reference
wow-project
wow-graph
```

The broader dependency table lists the maximum future edges. E3-A does **not** activate direct dependencies on `wow-store`, `wow-emmy`, or `wow-search`:

- store/read-transaction details remain behind coherent published project/graph views;
- analyzer facts arrive through the published project view;
- search/ranking supplies exact roots only in E4 or through a higher layer.

## Exact input identity

E3-A consumes one coherent snapshot containing:

```text
ProjectStoreEpochId
ProjectStoreGenerationId
ProjectPublicationSetId
ProjectGenerationId / ProjectSnapshotId / ProjectViewId
AnalyzerSnapshotId
GraphGenerationId / GraphSnapshotId / GraphViewId
optional exact ProfileId / ReferenceGenerationId / ReferenceViewId
source-universe, query-catalog, capability, coverage, and conflict manifests
```

`StoreImageId` and whole-SQLite-generation assumptions are forbidden by the selected E2-D contract.

## E3-A output identity DAG

```text
input snapshot + profiles + normalized request
-> plan/frontier
-> Project Map, skeletons, control/effects, source, evidence, loss records
-> ContextBundleCore
-> renderer artifact
-> metrics
-> evaluation report
-> outer delivery envelope
```

No earlier artifact contains a later artifact ID. Renderer bytes, token counts, timings, and model-evaluation scores never determine semantic bundle identity.

## E3-A / E3-B boundary

E3-A can consume a pinned Blizzard UI source universe only after a separate E3-B producer has acquired, materialized, parsed, analyzed, graphed, licensed, covered, and published the exact source snapshot.

E3-A itself never downloads, extracts, parses, indexes, or claims completeness for Blizzard UI source. API documentation/reference facts and platform UI source remain different evidence universes.

## Current implementation state

```text
documentation frontier: E3-A
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Directory presence does not authorize implementation before E0-E2 prerequisites, exact read catalogs, profile registries, synthetic/real fixtures, evaluation gates, and all checksum pins are frozen.
