# `wow-project` E3-A Blizzard UI source-universe index contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-project/e3-a/blizzard-ui-source-universe-index`

## Mission

Turn one exact, already-materialized Blizzard UI source snapshot into a separately scoped, persistently published structural source universe for graph traversal and later context construction.

```text
exact materialized Blizzard UI source snapshot
+ exact client/profile/reference/analyzer/parser/recognizer/graph/store profiles
-> validate provenance, roots, packages, files, licenses, limits, and complete inventory
-> select one exact TOC variant per package
-> reuse E2-C bounded TOC/XML/Lua/analyzer pipeline
-> build static package/file/XML/script/load structure
-> emit project-owned and core-recognizer graph proposal partitions
-> validate proposals through wow-graph
-> assemble one BlizzardUiIndexCandidate
-> publish through the E2-D coherent ProjectStore protocol
-> open exact BlizzardUiProjectView + GraphView
-> expose a bounded SkeletonInputView to wow-context
```

## Why a separate source universe

Blizzard UI implementation source is valuable evidence for load order, templates, mixins, frame construction, events, hooks, state access, and implementation patterns. It is not the same thing as:

- the Reference Pack/API contract;
- the user's addon project;
- an installed runtime client state;
- SavedVariables or logs;
- an external implementation candidate corpus;
- historical lineage across builds.

All identities and relations therefore carry `universe = blizzard_ui_source` and one exact source/profile generation.

## Active E3-A scope

- exact source-origin/mirror/materialization profile;
- complete configured-root inventory with content and license provenance;
- platform-source project/package/TOC/XML/Lua unit model;
- one selected client-flavor/TOC variant per package;
- static dependency, file order, include, script, load-phase, and reachability model;
- exact Emmy analyzer snapshot over physical and XML-virtual Lua units;
- E2-B core structural recognizers only;
- project-owned and recognizer-owned graph partitions;
- source-universe graph validation and coherent E2-D publication;
- separate source/project/reference universe binding;
- bounded direct-source and graph read view for downstream skeleton generation;
- structural fingerprints exported only as future lineage inputs;
- exact incremental update/removal closure;
- security, license/redistribution, resource, cancellation, determinism, fixtures, and freeze gates.

## Explicitly deferred

- Project Map and L0/L1 rendering (`wow-context` E3-B);
- context-pack selection/token budgeting (`wow-context` E3-B);
- fuzzy/exact search ranking and FTS (`wow-search` E4);
- cross-build entity lineage, moved/renamed/replaced edges, and patch impact (E4);
- named addon/framework recognizer packs (E5);
- Codebase Memory/external candidate integration (E6);
- runtime event payloads, logs, SavedVariables contents, taint observations, and combat state;
- any claim that source structure proves API legality, Secret Value readability, taint safety, protected-operation safety, runtime load success, or performance;
- source redistribution/release bundling without explicit license policy;
- CI.

## Direct dependencies

```text
wow-core
wow-emmy
wow-graph
wow-recognizers
wow-store
```

`wow-project` consumes public contracts only. It never imports storage internals or graph implementation details.

## Required reading

1. repository and `crates/` agent instructions;
2. [`../README.md`](../README.md);
3. [`../e2/README.md`](../e2/README.md) and all E2-C parser/index contracts;
4. [`../../wow-graph/e2/README.md`](../../wow-graph/e2/README.md);
5. [`../../wow-recognizers/e2/README.md`](../../wow-recognizers/e2/README.md);
6. [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md);
7. this complete E3-A package;
8. current external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routing for Blizzard UI source, lifecycle/load, events, hooks, XML, security, Secret Values, and relevant subsystems.

Candidate materialization source: [Gethe/wow-ui-source](https://github.com/Gethe/wow-ui-source). It is a mirror/provider input, not floating authority; implementation requires an exact commit/tree/content manifest and applicable provenance/license evidence.

## Public E3-A operations

```text
validate_blizzard_ui_index_request
validate_blizzard_ui_source_profile
validate_blizzard_ui_source_snapshot
select_blizzard_ui_packages_and_toc_variants
build_blizzard_ui_load_model
build_blizzard_ui_analyzer_plan
validate_blizzard_ui_analyzer_snapshot
build_blizzard_ui_recognizer_fact_bundles
run_blizzard_ui_core_recognizers
validate_blizzard_ui_graph_proposals
assemble_blizzard_ui_index_candidate
validate_blizzard_ui_index_candidate
build_blizzard_ui_publication_bundle
publish_blizzard_ui_index
open_blizzard_ui_project_view
open_blizzard_ui_skeleton_input_view
plan_blizzard_ui_incremental_update
```

No operation clones/fetches a repository, scans an installed client, executes source, builds context packs, or returns a floating current snapshot.

## Completion gate

E3-A implementation is complete only when one frozen synthetic platform-source corpus and one frozen real mirror snapshot can be indexed and published without source execution; every configured file/package/unit is accounted for; variant/flavor universes never merge; graph partitions retain exact evidence/coverage/conflicts; dynamic relations remain `Possible` or `NotEvaluated`; removal and profile changes invalidate exactly or conservatively; published project/graph/store/analyzer identities are coherent; 1/2/N workers and shuffled inputs produce identical logical outputs; source/provider names cannot trigger production semantics; and the downstream skeleton-input view can reconstruct every included signature/span/relation/provenance claim from exact published records.
