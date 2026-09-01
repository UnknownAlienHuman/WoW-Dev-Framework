# `wow-context` contract router

**Status:** E3-B Project Map, L0/L1 skeleton, and context-pack contract is implementation-ready documentation; no Rust code exists.

`wow-context` converts exact published project, Blizzard UI source, graph, analyzer-derived, and reference views into deterministic bounded context artifacts. It does not parse source, own graph/search/storage, infer facts with a model, or mutate projects.

## Contract history

- [`PRE_E3_OVERVIEW.md`](PRE_E3_OVERVIEW.md) preserves the initial crate brief.
- [`LEGACY_E3_A_CONTEXT_DRAFT.md`](LEGACY_E3_A_CONTEXT_DRAFT.md) maps the retired context milestone label to the single active E3-B model.

## Active E3-B route

1. [`e3/MILESTONE_RENUMBERING.md`](e3/MILESTONE_RENUMBERING.md)
2. [`e3/README.md`](e3/README.md)
3. [`e3/AGENTS.md`](e3/AGENTS.md)
4. [`e3/DECISIONS.md`](e3/DECISIONS.md)
5. [`e3/INPUT_UNIVERSE_AND_VIEWS.md`](e3/INPUT_UNIVERSE_AND_VIEWS.md)
6. [`e3/DATA_MODEL.md`](e3/DATA_MODEL.md)
7. [`e3/OPERATIONS.md`](e3/OPERATIONS.md)
8. [`e3/CONTEXT_REQUEST_AND_PROFILES.md`](e3/CONTEXT_REQUEST_AND_PROFILES.md)
9. [`e3/PROJECT_MAP.md`](e3/PROJECT_MAP.md)
10. [`e3/L0_SKELETON.md`](e3/L0_SKELETON.md)
11. [`e3/L1_SKELETON.md`](e3/L1_SKELETON.md)
12. [`e3/CONTROL_AND_EFFECT_MODEL.md`](e3/CONTROL_AND_EFFECT_MODEL.md)
13. [`e3/EXPANSION_SELECTION_AND_STOPPING.md`](e3/EXPANSION_SELECTION_AND_STOPPING.md)
14. [`e3/COVERAGE_AUTHORITY_AND_OMISSIONS.md`](e3/COVERAGE_AUTHORITY_AND_OMISSIONS.md)
15. [`e3/SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md`](e3/SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md)
16. [`e3/BUDGETS_TOKENIZATION_AND_PRUNING.md`](e3/BUDGETS_TOKENIZATION_AND_PRUNING.md)
17. [`e3/CONTEXT_PACK_AND_RENDERING.md`](e3/CONTEXT_PACK_AND_RENDERING.md)
18. [`e3/CACHE_AND_DETERMINISM.md`](e3/CACHE_AND_DETERMINISM.md)
19. [`e3/SECURITY_AND_PRIVACY.md`](e3/SECURITY_AND_PRIVACY.md)
20. [`e3/METRICS_AND_EVALUATION.md`](e3/METRICS_AND_EVALUATION.md)
21. [`e3/ERROR_MODEL.md`](e3/ERROR_MODEL.md)
22. [`e3/TEST_MATRIX.md`](e3/TEST_MATRIX.md)
23. [`e3/IDENTITY_DAG_TESTS.md`](e3/IDENTITY_DAG_TESTS.md)
24. [`e3/IMPLEMENTATION_PLAN.md`](e3/IMPLEMENTATION_PLAN.md)
25. [`e3/CONTRACT.json`](e3/CONTRACT.json) and [`e3/examples/`](e3/examples/README.md)

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

No direct dependency on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-service`, or applications. Relevant analyzer, recognizer, and persistence state arrives only through exact public project/graph/reference views.

## Output hierarchy

```text
ContextUniverseSet
-> ProjectMap
-> L0Skeleton(s)
-> L1Skeleton(s)
-> ContextSemanticPack
-> RenderedContextArtifact(s)
```

Every layer is immutable, exact-generation-bound, evidence-preserving, budgeted, and independently identified. Rendering never becomes semantic truth.

## Hard boundaries

- exact roots only; search/fuzzy/natural-language resolution is outside E3-B;
- no second TOC/XML/Lua parser or raw-source semantic inference;
- no model inference, summarization, ranking, embedding, or tool call in the canonical path;
- no current-generation switch after binding;
- no user-project, Blizzard UI, reference, external, runtime, or historical identity merge;
- no source text interpreted as framework or agent instruction;
- no claim without exact origin/evidence/coverage closure;
- no hidden omission, silent truncation, or exact-token claim without a frozen tokenizer;
- no unbounded source/graph export;
- no physical cache/persistence implementation;
- no diagnostics, remediation, edits, lineage, impact, or runtime truth;
- no Cargo/Rust/CI activation during this documentation phase.

## Current implementation state

```text
documentation frontier: E3-B
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
