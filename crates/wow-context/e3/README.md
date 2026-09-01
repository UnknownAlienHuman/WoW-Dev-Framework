# `wow-context` E3-B Project Map, L0/L1, and context-pack contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-context/e3-b/project-map-l0-l1-context-pack`

## Mission

Build compact, deterministic, evidence-preserving context artifacts from exact immutable user-project, Blizzard UI source, graph, and reference views.

```text
exact ContextUniverseSet
+ exact root IDs
+ reviewed map/skeleton/intent/expansion/budget/tokenizer/privacy/render profiles
-> validate generation and compatibility closure
-> build deterministic Project Map projections
-> build required L0 container skeletons
-> build required L1 entity/local-neighborhood skeletons
-> expand only reviewed typed axes under explicit bounds
-> fetch selected exact source/reference evidence when allowed
-> deduplicate semantic items while retaining every origin
-> select mandatory closure and deterministically prune optional items
-> emit omissions, coverage, conflicts, loss, and budget accounting
-> validate one immutable ContextSemanticPack
-> render separately identified canonical artifacts
```

## Milestone assignment

[`MILESTONE_RENUMBERING.md`](MILESTONE_RENUMBERING.md) and [`../LEGACY_E3_A_CONTEXT_DRAFT.md`](../LEGACY_E3_A_CONTEXT_DRAFT.md) are migration notes. E3-A belongs to the `wow-project` Blizzard UI source producer; this is the only active `wow-context` implementation package and is E3-B.

## Normative route

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`INPUT_UNIVERSE_AND_VIEWS.md`](INPUT_UNIVERSE_AND_VIEWS.md)
4. [`DATA_MODEL.md`](DATA_MODEL.md)
5. [`OPERATIONS.md`](OPERATIONS.md)
6. [`CONTEXT_REQUEST_AND_PROFILES.md`](CONTEXT_REQUEST_AND_PROFILES.md)
7. [`PROJECT_MAP.md`](PROJECT_MAP.md)
8. [`L0_SKELETON.md`](L0_SKELETON.md)
9. [`L1_SKELETON.md`](L1_SKELETON.md)
10. [`CONTROL_AND_EFFECT_MODEL.md`](CONTROL_AND_EFFECT_MODEL.md)
11. [`EXPANSION_SELECTION_AND_STOPPING.md`](EXPANSION_SELECTION_AND_STOPPING.md)
12. [`COVERAGE_AUTHORITY_AND_OMISSIONS.md`](COVERAGE_AUTHORITY_AND_OMISSIONS.md)
13. [`SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md`](SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md)
14. [`BUDGETS_TOKENIZATION_AND_PRUNING.md`](BUDGETS_TOKENIZATION_AND_PRUNING.md)
15. [`CONTEXT_PACK_AND_RENDERING.md`](CONTEXT_PACK_AND_RENDERING.md)
16. [`CACHE_AND_DETERMINISM.md`](CACHE_AND_DETERMINISM.md)
17. [`SECURITY_AND_PRIVACY.md`](SECURITY_AND_PRIVACY.md)
18. [`METRICS_AND_EVALUATION.md`](METRICS_AND_EVALUATION.md)
19. [`ERROR_MODEL.md`](ERROR_MODEL.md)
20. [`TEST_MATRIX.md`](TEST_MATRIX.md)
21. [`IDENTITY_DAG_TESTS.md`](IDENTITY_DAG_TESTS.md)
22. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
23. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)

## Inputs

```text
one exact primary user ProjectSnapshot/View
one exact primary user GraphSnapshot/View
zero or one exact Blizzard UI ProjectSnapshot/View
zero or one exact Blizzard UI GraphSnapshot/View
zero or one exact Blizzard UI SkeletonInputView
one exact ReferenceProfile / ReferenceGeneration / ReferenceView
```

All views are fixed for the operation. Missing optional universes produce explicit partial/omission state only when the request permits it. No previous/current/fallback generation is substituted.

## Outputs

- `ContextUniverseSet` and compatibility report;
- one separately identified `ProjectMap` per project universe plus explicit combined-map references;
- L0 project/package/load/file/module/service/library skeletons;
- L1 exact entity/local-neighborhood skeletons;
- control/effect projections over already-published fact registries only;
- typed expansion plans, frontiers, selection traces, omissions, and continuation;
- exact byte and honest token-accounting reports;
- immutable `ContextSemanticPack`;
- canonical JSON and deterministic Markdown artifacts;
- cache keys/validation only, not cache storage.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

## Canonical public operations

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

Historical plural and `bundle` names are migration terminology only and must not become duplicate APIs.

## Deferred

- natural-language/fuzzy/ranked roots;
- search, migration, lineage, rename/move/replacement, and impact;
- named addon/framework calibration;
- Codebase Memory and external semantic candidates;
- runtime observations, SavedVariables contents, logs, event payloads, or client memory;
- model-generated summaries/ranking/correctness;
- diagnostics, remediation, edits, or task completion;
- physical cache/context persistence;
- service/application/tool authorization and CI.

## Completion gate

Equivalent exact inputs/profiles must produce byte-identical canonical semantic packs and renderings under 1/2/N workers and shuffled storage/query order. Every claim resolves to exact origins. Every excluded candidate has an omission/selection reason. Mandatory identity/evidence/coverage/boundary records are never pruned. Exact token counts require a frozen tokenizer over exact final bytes. Source remains structurally isolated untrusted data. Inputs never switch generations. All graph, source, authority, profile, budget, privacy, boundary, cache, cancellation, continuation, and determinism tests must pass.
