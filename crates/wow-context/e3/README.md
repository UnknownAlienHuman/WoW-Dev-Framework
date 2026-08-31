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
-> build deterministic Project Map projections
-> build required L0 container skeletons
-> build required L1 entity/neighborhood skeletons
-> expand only reviewed typed axes under explicit bounds
-> fetch exact bounded source/reference evidence when allowed
-> deduplicate semantic items and evidence
-> prune optional items deterministically
-> emit omissions, coverage, conflicts, and budget accounting
-> validate one immutable ContextSemanticPack
-> render separately identified artifacts
```

## Exact inputs

```text
one primary user ProjectSnapshot/View and GraphSnapshot/View
zero or one exact Blizzard UI ProjectSnapshot/View and GraphSnapshot/View from E3-A
one exact ReferenceProfile / ReferenceGeneration / ReferenceView
exact source and SkeletonInputView capabilities exposed by wow-project
```

All views remain fixed for the operation lifetime. No prior/current/fallback generation is substituted or relabeled.

## Outputs

```text
ContextUniverseSet and compatibility report
ProjectMap per bound project universe
L0Skeleton per selected container
L1Skeleton per exact entity/neighborhood
ContextExpansionPlan and SelectionTrace
ContextSemanticPack
canonical JSON and deterministic Markdown RenderedContextArtifact
cache identity and validation records, but no cache storage
```

Every layer is immutable, generation-bound, evidence-preserving, budgeted, and independently identified. Rendering never becomes semantic truth.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

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
build_context_semantic_pack
continue_context_semantic_pack
validate_context_semantic_pack
render_context_pack_json
render_context_pack_markdown
validate_rendered_context_artifact
build_context_cache_key
```

All operations are read-only and transport-independent.

## Authority order

1. exact bound project/source/reference/graph records;
2. exact analyzer/recognizer facts already published through owner views;
3. deterministic E3-B projections with explicit derivation records;
4. renderer text as presentation only;
5. external/model/runtime information only under later separately identified universes.

Selection, aggregation, repetition, grouping, and rendering cannot upgrade authority.

## Explicit deferrals

- natural-language/fuzzy root resolution;
- cross-build lineage, migration, replacement, and impact;
- named addon/framework calibration behavior;
- Codebase Memory/external semantic candidates;
- runtime observations, SavedVariables contents, logs, event payloads, or process memory;
- model inference, generated summaries, relevance scoring, or tool calls;
- diagnostics, remediation, edit plans, and task completion;
- persistence/cache storage;
- transport/application behavior;
- CI/release automation.

## Completion gate

Implementation is complete only when equivalent exact inputs and profiles produce byte-identical canonical semantic packs and renderings under 1/2/N workers and shuffled storage/query order; every included claim resolves to exact origins/evidence/coverage; every excluded candidate has an omission reason; mandatory identity/evidence/coverage/conflict data can never be pruned to fit; exact token counts require a frozen exact tokenizer; source text remains structurally isolated as untrusted data; input views never switch generations; and all source, graph, profile, budget, cancellation, privacy, instruction-boundary, truncation, continuation, cache, and identity-DAG mutation tests pass.
