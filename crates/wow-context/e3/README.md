# `wow-context` E3-B Project Map, L0/L1, and context-pack contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-context/e3-b/project-map-l0-l1-context-pack`

## Mission

Build compact, deterministic, evidence-preserving context artifacts from exact immutable project, Blizzard UI source, graph, and reference views.

```text
exact ContextUniverseSet
+ exact root IDs
+ reviewed context/expansion/budget/tokenizer/privacy/render profiles
-> validate generation and compatibility closure
-> build or open deterministic Project Map projections
-> build required L0 container skeletons
-> build required L1 entity/neighborhood skeletons
-> expand only reviewed typed axes under explicit bounds
-> fetch exact bounded source/reference evidence when allowed
-> deduplicate semantic items and evidence
-> prune optional items deterministically
-> emit omissions, coverage, conflicts, and budget accounting
-> validate one immutable ContextSemanticPack
-> render one or more separately identified artifacts
```

## Mandatory renumbering notice

Read [`MILESTONE_RENUMBERING.md`](MILESTONE_RENUMBERING.md) first. The existing context foundation was initially labeled E3-A. The current authoritative assignment is E3-B; inherited documents remain part of this package and do not define a second implementation.

## Normative route

1. [`MILESTONE_RENUMBERING.md`](MILESTONE_RENUMBERING.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`DECISIONS.md`](DECISIONS.md)
4. [`INPUT_UNIVERSE_AND_VIEWS.md`](INPUT_UNIVERSE_AND_VIEWS.md)
5. [`INPUT_VIEWS_AND_SCOPES.md`](INPUT_VIEWS_AND_SCOPES.md) — inherited specialized input/scoping detail
6. [`DATA_MODEL.md`](DATA_MODEL.md)
7. [`OPERATIONS.md`](OPERATIONS.md)
8. [`CONTEXT_REQUEST_AND_PROFILES.md`](CONTEXT_REQUEST_AND_PROFILES.md)
9. [`PROJECT_MAP.md`](PROJECT_MAP.md)
10. [`SKELETONS.md`](SKELETONS.md), [`L0_SKELETON.md`](L0_SKELETON.md), and [`L1_SKELETON.md`](L1_SKELETON.md)
11. [`CONTROL_AND_EFFECT_MODEL.md`](CONTROL_AND_EFFECT_MODEL.md)
12. [`EXPANSION_SELECTION_AND_STOPPING.md`](EXPANSION_SELECTION_AND_STOPPING.md)
13. [`DETAIL_AND_EXPANSION.md`](DETAIL_AND_EXPANSION.md) and [`CONTINUATION_AND_STOPPING.md`](CONTINUATION_AND_STOPPING.md)
14. [`COVERAGE_AUTHORITY_AND_OMISSIONS.md`](COVERAGE_AUTHORITY_AND_OMISSIONS.md)
15. [`EVIDENCE_COVERAGE_AND_LOSS.md`](EVIDENCE_COVERAGE_AND_LOSS.md)
16. [`SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md`](SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md)
17. [`SOURCE_EXCERPTS_AND_SECURITY.md`](SOURCE_EXCERPTS_AND_SECURITY.md)
18. [`BUDGETS_TOKENIZATION_AND_PRUNING.md`](BUDGETS_TOKENIZATION_AND_PRUNING.md)
19. [`BUDGETS_AND_TOKENIZATION.md`](BUDGETS_AND_TOKENIZATION.md)
20. [`CONTEXT_PACK_AND_RENDERING.md`](CONTEXT_PACK_AND_RENDERING.md)
21. [`RENDERING_AND_CANONICALIZATION.md`](RENDERING_AND_CANONICALIZATION.md)
22. [`CACHE_AND_DETERMINISM.md`](CACHE_AND_DETERMINISM.md)
23. [`SECURITY_AND_PRIVACY.md`](SECURITY_AND_PRIVACY.md)
24. [`METRICS_AND_EVALUATION.md`](METRICS_AND_EVALUATION.md)
25. [`ERROR_MODEL.md`](ERROR_MODEL.md)
26. [`TEST_MATRIX.md`](TEST_MATRIX.md) and [`IDENTITY_DAG_TESTS.md`](IDENTITY_DAG_TESTS.md)
27. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
28. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)

## Active E3-B inputs

```text
one exact primary user ProjectSnapshot/View
one exact primary user GraphSnapshot/View
zero or one exact Blizzard UI ProjectSnapshot/View
zero or one exact Blizzard UI GraphSnapshot/View
one exact ReferenceProfile / ReferenceGeneration / ReferenceView
exact source and SkeletonInputView capabilities exposed by wow-project
```

All views remain fixed for the operation. Missing optional universes can yield an explicitly partial pack only when the request profile permits it. No prior/current/fallback generation is substituted or relabeled.

## Active outputs

- `ContextUniverseSet` and compatibility report;
- deterministic `ProjectMap` per bound project universe plus explicit combined-map references;
- L0 project/package/file/module/service/library skeletons;
- L1 entity/local-neighborhood skeletons;
- typed expansion plans and selection traces;
- exact byte/token budget and omission reports;
- immutable `ContextSemanticPack`;
- canonical JSON and deterministic Markdown renderings;
- cache identities only, not physical cache storage.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

No direct dependency on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-service`, or applications.

## Public operations

```text
bind_context_universe_set
validate_context_universe_set
validate_context_profiles
validate_context_request
build_project_map
open_project_map_view
build_l0_skeleton
build_l1_skeleton
plan_context_expansion
expand_context_frontier
build_context_source_excerpts
build_context_coverage_and_omissions
build_context_semantic_pack
continue_context_semantic_pack
validate_context_semantic_pack
render_context_pack_json
render_context_pack_markdown
validate_rendered_context_artifact
measure_context_pack
compare_context_packs
build_context_cache_key
```

Historical plural and `bundle` operation names are documentation aliases only; they do not authorize duplicate APIs.

## Hard boundaries

- exact roots and reviewed profiles only;
- no natural-language query semantics;
- no second parser or raw-source semantic inference;
- no hidden graph edges or transitive-edge materialization;
- no model-generated responsibilities, summaries, priorities, or token counts;
- no project/platform/reference universe merge;
- no generation switch after binding;
- no claim without exact origin/evidence/coverage closure;
- no loss of `Possible`, conflict, partial, truncated, or `NotEvaluated` state;
- no silent omission or byte/token overflow;
- no source excerpt without exact digest/range/privacy/license policy;
- no source text interpreted as framework/agent instruction;
- no unbounded graph/source expansion;
- no persistence or external side effects.

## Deferred

- search/fuzzy root resolution;
- cross-build lineage, rename/move/replacement, migration, and patch impact;
- named addon/framework calibration behavior;
- Codebase Memory/external semantic candidates;
- runtime observations and client data;
- LLM/model inference and generated summaries;
- diagnostics, remediation, edits, or task completion;
- transport/application orchestration and CI.

## Completion gate

Equivalent exact inputs and profiles must produce byte-identical canonical semantic packs and renderings under 1/2/N workers and shuffled storage/query order. Every included claim resolves to exact origins. Every excluded candidate has an omission/selection reason. Mandatory identity/evidence/coverage data can never be pruned. Exact token counts require a frozen exact tokenizer. Source text remains structurally isolated as untrusted data. Input views never switch generations. All source, graph, profile, budget, privacy, cancellation, continuation, boundary, cache, and determinism mutation tests must pass.
