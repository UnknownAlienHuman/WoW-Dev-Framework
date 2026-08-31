# `wow-context` E3-A Project Map and progressive context contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-context/e3-a/project-map-skeleton-progressive-context`

## Mission

Convert one exact published project/graph/reference state into deterministic, evidence-bearing, budgeted context artifacts without turning summaries into authority or bulk-loading source.

```text
exact ProjectStore epoch/generation/publication
+ exact ProjectSnapshot / AnalyzerSnapshot / GraphSnapshot
+ optional exact ReferenceView
+ frozen context/profile/query/source/security/renderer/evaluation contracts
+ exact roots and lanes
-> validate coherent input and capabilities
-> build Project Map
-> build L0 identity/role skeletons
-> build selected L1 signatures, direct relations, and control/effect skeletons
-> expand only requested bounded branches
-> resolve faithful bounded source excerpts only when requested
-> retain evidence, coverage, conflicts, ambiguity, loss, omissions, and stopping
-> produce ContextBundleCore
-> render/measure/evaluate as later DAG layers
```

## Active direct dependencies

```text
wow-core
wow-reference
wow-project
wow-graph
```

No direct E3-A dependency on `wow-store`, `wow-emmy`, or `wow-search`. Published project/graph views mediate storage/analyzer state; exact roots are supplied by the caller.

## Normative route

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`INPUT_VIEWS_AND_SCOPES.md`](INPUT_VIEWS_AND_SCOPES.md)
4. [`DATA_MODEL.md`](DATA_MODEL.md)
5. [`OPERATIONS.md`](OPERATIONS.md)
6. [`PROJECT_MAP.md`](PROJECT_MAP.md)
7. [`SKELETONS.md`](SKELETONS.md)
8. [`CONTROL_AND_EFFECT_MODEL.md`](CONTROL_AND_EFFECT_MODEL.md)
9. [`DETAIL_AND_EXPANSION.md`](DETAIL_AND_EXPANSION.md)
10. [`CONTINUATION_AND_STOPPING.md`](CONTINUATION_AND_STOPPING.md)
11. [`EVIDENCE_COVERAGE_AND_LOSS.md`](EVIDENCE_COVERAGE_AND_LOSS.md)
12. [`SOURCE_EXCERPTS_AND_SECURITY.md`](SOURCE_EXCERPTS_AND_SECURITY.md)
13. [`BUDGETS_AND_TOKENIZATION.md`](BUDGETS_AND_TOKENIZATION.md)
14. [`RENDERING_AND_CANONICALIZATION.md`](RENDERING_AND_CANONICALIZATION.md)
15. [`METRICS_AND_EVALUATION.md`](METRICS_AND_EVALUATION.md)
16. [`ERROR_MODEL.md`](ERROR_MODEL.md)
17. [`TEST_MATRIX.md`](TEST_MATRIX.md)
18. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
19. [`CONTRACT.json`](CONTRACT.json)
20. [`examples/`](examples/README.md)

## Exact input

```text
ProjectStoreEpochId
ProjectStoreGenerationId
ProjectPublicationSetId
ProjectGenerationId / ProjectSnapshotId / ProjectViewId
AnalyzerSnapshotId
GraphGenerationId / GraphSnapshotId / GraphViewId
optional exact ProfileId / ReferenceGenerationId / ReferenceViewId
source-universe/query-catalog/capability/coverage/conflict manifests
```

All identities cohere. `StoreImageId` is forbidden by the current E2-D model.

## Detail levels

### Project Map

Strictly bounded project/package/TOC/load/source-unit/owner/role/signal/state/API-use/blocker overview with exact next-detail routes. Default compact renderer target is approximately 2 KiB, but mandatory semantics are never discarded to hit it.

### L0

Exact identity, kind, owner/load/role, public surface headings, direct important relations, evidence, blockers, and routes. No bodies.

### L1

Selected exact signatures/members/direct relations/reason paths plus a closed control/effect node projection over published facts. It is not a new parser or CFG/data-flow engine.

### Source detail

Exact bounded faithful source span through a validated source handle and explicit license/privacy/security/source budget. No path fallback or reconstructed source.

## Identity DAG

```text
input/request
-> plan/frontier
-> map/skeleton/control/source/evidence/loss records
-> ContextBundleCore
-> renderer artifact
-> metrics
-> evaluation report
-> envelope
```

This order is normative and prevents bundle/metric/evaluation hash cycles.

## Public operations

```text
validate_context_profiles
validate_context_request
build_project_map
build_l0_skeletons
build_l1_skeletons
plan_context_expansion
expand_context_frontier
build_context_source_excerpts
build_context_coverage_and_loss
build_context_bundle
continue_context_bundle
measure_context_bundle
compare_context_bundles
validate_context_bundle
```

All are read-only and transport-independent.

## E3-A source-universe boundary

Initial exact universes include first-party project, explicitly supplied dependency metadata/source, reference platform API, and optionally an already-published pinned platform UI source universe.

The pinned Blizzard UI source universe must be built by E3-B or another reviewed producer with exact acquisition, build/revision, license, source manifest, project/analyzer/graph generations, coverage, and source handles. E3-A never acquires or indexes it.

## Progressive context

```text
Project Map
-> exact L0 root
-> selected L1 lane/control/effect detail
-> exact evidence/reason path
-> bounded source excerpt only when needed
-> stop at requested closure, no-new-evidence, explicit boundary, or budget
```

No full repository/graph/evidence dump and no hidden search.

## Proof and coverage

Every material field has exact evidence or deterministic derivation. Project, analyzer, recognizer, graph, reference, store-read, context, source, renderer, tokenizer, and evaluation coverage remain independent.

Context never derives absence from an empty/omitted/truncated section and never upgrades Possible/Candidate or partial/conflicted inputs.

## Security

Source is untrusted quoted data. It cannot control policy, headings, links, templates, tools, traversal, budgets, or agent instructions. Private paths, credentials, SavedVariables/log/client/runtime payloads, unrestricted objects, and full source are excluded.

## E3-B and later

- E3-B: pinned Blizzard UI source acquisition/materialization/project/analyzer/graph producer.
- E3-C: service/application context orchestration and evaluated use cases, if kept separate.
- E4: exact/migration/shape/FTS/graph search, lineage, and impact.
- E5+: calibration packs, optional semantic candidates, transports, release/publishing.

## Completion gate

E3-A is complete only when exact inputs produce byte-identical semantic maps/skeletons/bundles/continuations under shuffled input and 1/2/N workers; every material claim has evidence closure; unknown/collapsed/omitted detail is explicit; mandatory blockers survive tight budgets; source/security/tokenizer/renderer gates pass; continuations never switch generations or reset budgets; the identity DAG is cycle-free; synthetic and pinned real-project evaluation shows useful compression without missing mandatory records; and all frozen fixtures/checksums pass.
