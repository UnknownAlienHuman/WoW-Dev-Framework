# E3-B Blizzard UI source universe contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-project/e3-b/blizzard-ui-source-universe`

## Mission

Materialize, index, validate, graph, and publish one exact Blizzard UI implementation-source snapshot as a separate, profile-bound `blizzard_ui_source` universe. The result is an immutable source project/graph generation that can be opened alongside—but never merged by name with—a user addon project or API reference universe.

```text
exact BlizzardUiSourceProfile
+ exact externally materialized source snapshot
+ exact compatible reference/profile identity
+ exact parser/analyzer/recognizer/graph/store profiles
-> validate provider revision, build binding, roots, files, licenses and coverage
-> select declared source packages and TOC variants
-> parse TOC/XML without execution
-> analyze Lua only through wow-emmy
-> build exact source/load/object/symbol/call/registration/state facts
-> run only approved universal recognizers
-> propose source-universe graph assertions
-> validate exact reference-to-source bridge assertions
-> assemble BlizzardUiSourceIndexCandidate
-> build E2-D inactive ProjectStore/Graph generation
-> fresh read-back source/graph/bridge validation
-> CAS-activate one coherent current source publication
```

## Direct dependencies

```text
wow-core
wow-emmy
wow-graph
wow-recognizers
wow-store
```

`wow-project` does not depend directly on `wow-reference`. Exact reference compatibility and bridge endpoints are supplied through profile IDs and public graph/reference entity views. `wow-context`, `wow-search`, `wow-service`, and applications remain higher-level consumers.

## Owned responsibilities

- exact `BlizzardUiSourceProfile` and source-collection identity;
- validation of one externally materialized immutable source snapshot;
- provider/revision/build/root/file/content/license/coverage manifests;
- declared source-root and package classification without implicit host scanning;
- TOC/XML/load indexing under E2-C parser contracts;
- Lua workspace construction and exact `wow-emmy` snapshot binding;
- source-universe fact adapters and approved recognizer execution;
- graph proposals for source files, packages, functions, methods, XML entities, frames, templates, mixins, callbacks, registrations, calls, state and API-use structure;
- exact source-to-reference bridge proposal inputs and reports;
- source-universe invalidation, candidate assembly and validation;
- separate ProjectStore/Graph publication bundle and current source publication identity;
- source/licensing/redistribution/security/budget/determinism fixtures.

## Explicit non-responsibilities

E3-B does not:

- fetch, clone, update, or discover a floating Blizzard UI source repository;
- trust a branch/tag/build label without exact content and compatibility evidence;
- execute Lua, XML scripts, TOC directives, repository hooks, build/test/release scripts, generators or workflows;
- parse Lua with a second parser;
- treat implementation source as APIDocumentation or public API authority;
- infer that an API exists or is absent solely from source calls or source-tree absence;
- infer Secret Value, taint, protected, forbidden, managed-object, combat, payload-readability, performance or runtime-success semantics from implementation source;
- merge Blizzard source entities with project/reference/dependency/runtime entities by name, path or signature;
- publish project-specific bridge assertions for every user addon without an exact user project generation;
- copy or redistribute source bytes without an explicit license/redistribution decision;
- expose a full source-tree dump through context, service, MCP or LSP;
- implement search, lineage, migration, impact ranking, diagnostics or remediation;
- add CI.

## Authority classes

```text
reference_api
    public API contract, restrictions and exact profile facts

blizzard_ui_source
    implementation-source structure for one exact materialized source snapshot

first_party_project
    user addon source and graph for one exact project generation

runtime
    observed client behavior under an exact runtime profile
```

Each remains a separate universe and evidence class. Explicit graph bridge assertions can connect them; no bridge changes endpoint authority.

## Build-binding states

```text
ExactBuildMatched
    exact source-to-client build/profile binding is independently evidenced

ProviderDeclared
    provider labels the source revision as a build, but independent binding is incomplete

ContentCorrelated
    exact source bytes correlate with other pinned evidence but do not reach exact build proof

Unverified
    source revision is known; build compatibility is not established

Mismatch
    source/profile evidence conflicts
```

Only a policy-approved state may publish as current for production queries. `ProviderDeclared` is not silently upgraded to `ExactBuildMatched`.

## Source snapshot boundary

The library accepts a closed `MaterializedBlizzardUiSourceSnapshot` containing:

- provider/repository/revision provenance;
- exact declared build/profile binding evidence;
- configured logical source roots and roles;
- complete admitted file inventory with canonical content digests;
- symlink/reparse/submodule/LFS/archive/materialization report;
- per-root/file license and redistribution records;
- materialization security report;
- completeness/omission/conflict records.

It never turns a local path or online repository URL into a snapshot itself.

## Separate publication

Blizzard UI source uses a dedicated source project/store namespace:

```text
BlizzardUiSourceCollectionId
BlizzardUiSourceGenerationId
BlizzardUiSourceSnapshotId
BlizzardUiSourceGraphGenerationId
BlizzardUiSourceGraphSnapshotId
BlizzardUiSourcePublicationSetId
ProjectStoreGenerationId
CurrentBlizzardUiSourcePublicationRecord
```

It cannot share a user project's current record. A service operation selects exact user project, reference, and UI source publications independently and verifies compatibility before cross-universe queries.

## Bridge scope

E3-B may publish exact source/reference bridges when both endpoints and profiles are exact, for example:

```text
UI source function --uses_api--> reference API symbol
UI source XML object --references_template/inherits--> exact UI source template
UI source function --registers_event/handles_event--> exact source/reference event entity under the registered relation profile
```

Per-user-project bridges such as hooks, template inheritance, copied patterns or source analogues require an exact user ProjectSnapshot and are produced by a later project integration/search package. E3-B defines compatible endpoint/identity contracts but does not precompute project-specific truth.

## Current documentation routes

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`SOURCE_PROFILE_AND_MATERIALIZATION.md`](SOURCE_PROFILE_AND_MATERIALIZATION.md)
5. [`UNIVERSE_AND_IDENTITY.md`](UNIVERSE_AND_IDENTITY.md)
6. [`INDEXING_PIPELINE.md`](INDEXING_PIPELINE.md)
7. [`REFERENCE_AND_PROJECT_BRIDGES.md`](REFERENCE_AND_PROJECT_BRIDGES.md)
8. [`GRAPH_PUBLICATION.md`](GRAPH_PUBLICATION.md)
9. [`COVERAGE_CONFLICTS_AND_AUTHORITY.md`](COVERAGE_CONFLICTS_AND_AUTHORITY.md)
10. [`LICENSE_SECURITY_AND_REDISTRIBUTION.md`](LICENSE_SECURITY_AND_REDISTRIBUTION.md)
11. [`INVALIDATION_AND_VERSIONING.md`](INVALIDATION_AND_VERSIONING.md)
12. [`ERROR_MODEL.md`](ERROR_MODEL.md)
13. [`TEST_MATRIX.md`](TEST_MATRIX.md)
14. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
15. [`CONTRACT.json`](CONTRACT.json)
16. [`examples/`](examples/README.md)

Supporting owner seams:

- [`../../wow-reference/e3/BLIZZARD_UI_SOURCE_PROFILE.md`](../../wow-reference/e3/BLIZZARD_UI_SOURCE_PROFILE.md)
- [`../../wow-graph/e3/BLIZZARD_UI_SOURCE_GRAPH.md`](../../wow-graph/e3/BLIZZARD_UI_SOURCE_GRAPH.md)
- [`../../wow-context/e3/BLIZZARD_UI_INPUT_HANDOFF.md`](../../wow-context/e3/BLIZZARD_UI_INPUT_HANDOFF.md)

Also read current external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routing for patch-sensitive claims. The framework stores stable contracts and exact profile IDs, not copied live conclusions.

## Public E3-B operations

```text
validate_blizzard_ui_source_profile
validate_materialized_blizzard_ui_source_snapshot
classify_blizzard_ui_source_roots_and_packages
build_blizzard_ui_source_project_index
validate_blizzard_ui_source_analyzer_snapshot
build_blizzard_ui_source_fact_bundles
run_blizzard_ui_source_recognizers
build_blizzard_ui_source_graph_proposals
build_reference_ui_bridge_proposals
validate_blizzard_ui_source_graph_plan
plan_blizzard_ui_source_invalidation
assemble_blizzard_ui_source_candidate
validate_blizzard_ui_source_candidate
build_and_activate_blizzard_ui_source_publication
open_current_blizzard_ui_source_view
open_exact_blizzard_ui_source_view
```

## Completion gate

E3-B implementation is complete only when an exact synthetic source collection and a separately pinned real source snapshot can be indexed without execution; build-binding and license states remain explicit; source/reference/project universes never collide; all source entities and bridges retain exact source/evidence/coverage; implementation source never becomes API/runtime/security authority; omitted roots/files prevent complete claims; source update/removal invalidates all dependent analyzer/recognizer/graph/bridge records; E2-D publishes a coherent inactive-validated-CAS source generation; old/new readers remain exact; prohibited redistribution is prevented; and 1/2/N workers plus shuffled file/fact/assertion order produce identical canonical candidates, graph publications and permitted artifacts.
