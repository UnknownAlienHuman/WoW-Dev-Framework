# `wow-project` E2-C project indexing contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-project/e2-c/toc-xml-load-incremental-index`

## Mission

Turn one exact materialized addon-project snapshot into a coherent, profile-bound project index candidate containing validated source inventory, TOC variants/directives/load order, bounded XML structure and embedded-Lua units, analyzer facts, recognizer input/output partitions, and graph proposal-validation results.

```text
exact materialized project source snapshot
+ exact project/profile/reference/analyzer/parser/recognizer/graph profiles
-> validate roots, universes, files, digests, licenses, and budgets
-> select exactly one TOC variant per package/flavor target
-> parse TOC without executing anything
-> resolve package dependencies, file order, LOD/bootstrap metadata, and SavedVariables declarations
-> parse XML with external entities/DTD/execution disabled
-> materialize XML includes, objects/templates/inheritance/scripts, and embedded Lua units
-> build exact Lua workspace/update for wow-emmy
-> receive one generation-bound AnalyzerSnapshot and normalized Lua facts
-> adapt Lua/TOC/XML/project facts into RecognizerFactBundle partitions
-> run E2-B core recognizers
-> validate proposals against the exact E2-A graph registry
-> assemble deterministic invalidation, coverage, load, fact, recognizer, and graph-input manifests
-> produce one immutable ProjectIndexCandidate
```

E2-C stops before persistent ProjectStore/GraphSnapshot publication. E2-D chooses and activates the physical ProjectStore model and atomically publishes coherent ProjectSnapshot + GraphSnapshot generations.

## Direct dependencies

```text
wow-core
wow-emmy
wow-graph
wow-recognizers
```

The `wow-store` edge remains inactive in E2-C; persistence is E2-D. `wow-reference` is selected through exact profile/reference identity and later service/rule joins, not a direct project parser dependency.

## Owned responsibilities

- exact materialized project-source snapshot and universe declarations;
- package/addon/TOC variant identity;
- bounded TOC lexical parser and normalized directive/file/dependency/SavedVariables facts;
- flavor/Interface/variant selection without cross-variant merging;
- bounded streaming XML parser and normalized template/object/inheritance/script/include facts;
- XML-embedded Lua source-unit extraction with exact source mapping, never execution;
- deterministic static load model for package dependencies, TOC entries, XML includes/scripts, LOD/bootstrap phases, and reachability;
- first-party Lua workspace construction and exact `wow-emmy` update/snapshot binding;
- TOC/XML/project fact adapters for E2-B recognizers;
- recognizer execution orchestration and output-partition validation;
- graph proposal validation requests/reports against exact E2-A registries;
- explicit incremental invalidation from source/profile/parser/analyzer/pack/registry changes;
- coherent ProjectGeneration derivation inputs for the full index;
- ProjectIndexCandidate validation/canonicalization;
- capability/coverage/conflict/truncation/NotEvaluated records;
- last-known-good candidate identity without relabeling;
- security, budgets, cancellation, fixtures, mutation tests, and deterministic output.

## Explicit non-responsibilities

E2-C does not:

- discover a floating repository, branch, installed addon, client tree, SavedVariables directory, log, or editor workspace;
- follow unreviewed symlinks/reparse points/submodules or execute repository hooks/build scripts/generators/tests;
- execute Lua, XML scripts, TOC directives, addon code, or generated files;
- parse Lua with a second parser;
- implement recognizer rules or graph semantics;
- publish graph assertions or final GraphGeneration IDs;
- implement SQLite, WAL, migrations, transactions, checkpoints, backups, retention, or GC;
- persist or atomically publish ProjectStore generations;
- decide WoW API existence, event validity, Secret/taint/protected legality, diagnostics, severity, remediation, search ranking, or runtime behavior;
- merge first-party, dependency, annotation-library, reference, external, runtime, or historical universes;
- download missing dependencies or source files;
- infer runtime load success/frame existence from static TOC/XML order;
- add CI.

## Active source classes

```text
first_party_project
    selected package/TOC/XML/Lua files under one exact project snapshot

declared_dependency_metadata
    exact dependency package declarations and optional explicitly supplied dependency manifests

analyzer_library
    generated/reference annotation library bound through wow-emmy; never a first-party project file
```

Full dependency source indexing, installed runtime data, external implementation corpora, and Blizzard UI source indexing activate only through later explicit universes/profiles.

## Required reading

1. repository and `crates/` agent instructions;
2. [`../README.md`](../README.md) and E0-D project contracts at the crate root;
3. [`../../wow-emmy/FACT_MODEL.md`](../../wow-emmy/FACT_MODEL.md);
4. [`../../wow-graph/e2/README.md`](../../wow-graph/e2/README.md);
5. [`../../wow-recognizers/e2/README.md`](../../wow-recognizers/e2/README.md);
6. [`REAL_ADDON_FIXTURE.md`](REAL_ADDON_FIXTURE.md) for the pinned read-only user-addon fixture and its nonclaims;
7. all other E2-C files in this package;
8. current external KB `AGENTS.md`, `INDEX_MINI.md`, workflow, lifecycle/LOD, event, hook, security, and relevant subsystem routes.

## Public E2-C operations

```text
validate_project_index_request
validate_project_source_snapshot
select_project_toc_variants
parse_toc_partition
parse_xml_partition
build_project_load_model
materialize_project_lua_units
build_project_analyzer_update
validate_project_analyzer_snapshot
build_recognizer_fact_bundles
run_project_recognizers
validate_project_graph_proposals
plan_project_incremental_invalidation
assemble_project_index_candidate
validate_project_index_candidate
open_project_index_candidate_view
```

No operation opens arbitrary host paths or returns a published persistent generation.

## Completion gate

E2-C code is complete only when one exact synthetic repository snapshot and the pinned [`UnknownAlienHuman/roth-ui`](REAL_ADDON_FIXTURE.md) fixture can be indexed without source execution; TOC variants never mix; XML entities/includes/scripts remain bounded and source-mapped; Lua is parsed only by the accepted Emmy adapter; load, SavedVariables, native/custom signal, hook, and state proposals preserve their exact proof limits; file/TOC/XML updates invalidate exactly the required analyzer/recognizer/graph-input partitions or widen conservatively; removed facts/proposals disappear; all identities/generations remain coherent; partial/conflicted/truncated areas never become complete; repository/path/name mutations prove fixture independence; and 1/2/N worker plus shuffled input/update sequences produce byte-identical canonical candidates.
