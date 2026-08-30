# `wow-project` implementation contract

**Status:** minimal E0 slice active; full implementation deferred to E2.

## Mission

`wow-project` owns the normalized model of an addon workspace: configured roots, file identity, TOC/XML/load structure, declared dependencies, SavedVariables roots, incremental invalidation, and coherent project-generation publication. It coordinates analyzer and recognizer inputs without owning their internals.

## Owned responsibilities

- project configuration and root identity;
- first-party/dependency/external/runtime universe separation;
- normalized file inventory and content digests;
- TOC parsing, variants, flavor/interface metadata, dependencies, optional dependencies, load-on-demand units, SavedVariables, and declared file order;
- structural XML parsing for includes, scripts, frames, templates, inheritance, children, and source spans;
- workspace assembly inputs for `wow-emmy`;
- project-generation state and affected-partition invalidation;
- load/reachability facts and use-before-load prerequisites;
- invocation of universal recognizers over normalized facts;
- project snapshot read interfaces;
- generated project metadata inputs for Project Map/context.

## Explicit non-responsibilities

`wow-project` does not:

- implement the Lua parser/analyzer;
- define reference/API truth;
- own generic graph storage/query algorithms;
- implement diagnostics or search ranking;
- load arbitrary installed addons without explicit configuration;
- infer runtime load state from directory presence;
- execute addon code, TOC directives, XML scripts, repository hooks, or generators;
- mutate editor settings;
- copy project-specific behavior into framework-wide hardcoded branches.

## Universe model

Every source file belongs to exactly one configured universe:

```text
workspace       writable first-party addon files
dependency      declared library/addon inputs, read-only
external        selected example repositories, read-only candidate evidence
runtime         optional SavedVariables/log/probe records, explicit opt-in
reference       selected Blizzard Reference Pack, separate from project files
```

Universe identity participates in source handles and must not be inferred from path naming alone.

## Project generation

A project generation binds:

```text
project configuration digest
selected reference profile/generation
file inventory and digests
TOC/XML parse partitions
Emmy analyzer snapshot token
recognizer versions and output partitions
project graph/index generation
capability/coverage report
```

One generation is published atomically. Partial updates remain private until all mandatory participating slices agree on the same generation context.

## Required operations

| Operation | Required behavior |
|---|---|
| `load_project_config` | Parse explicit project roots/flavor/profile/dependency configuration without filesystem-wide guessing. |
| `inventory_project_files` | Classify files by universe/type/TOC reachability and record normalized digests. |
| `parse_toc_manifest` | Preserve ordered entries, metadata, flavor/interface scope, dependencies, LOD, SavedVariables, and unsupported directives. |
| `resolve_toc_variants` | Produce separate variant/load units; never merge flavors implicitly. |
| `parse_xml_document` | Structurally parse includes/templates/frames/scripts/inheritance/children with strict bounds and source spans. |
| `assemble_emmy_workspace` | Produce explicit first-party/library file sets for `wow-emmy`; no editor mutation. |
| `plan_project_update` | Compare old/new inventory and identify affected files/TOC/XML/recognizer/graph/rule partitions. |
| `apply_project_update` | Drive analyzer/recognizer updates under one unpublished generation transaction. |
| `build_load_graph_facts` | Emit package/file/dependency/order/reachability facts with evidence and coverage. |
| `build_state_root_facts` | Emit declared SavedVariables roots and literal path facts without reading persisted values. |
| `run_project_recognizers` | Invoke approved recognizer packs over normalized facts and capture producer/version/coverage. |
| `publish_project_generation` | Publish one coherent snapshot or keep the last-known-good generation. |
| `open_project_view` | Expose files, owners, load facts, registrations, state roots, and capability status through narrow reads. |
| `resolve_project_source_handle` | Resolve only within registered project roots/generation and validate digest/span. |

## TOC rules

1. File order is semantic and preserved.
2. Missing files, duplicate entries, unsupported directives, and dependency cycles are explicit findings/coverage states.
3. `Dependencies` and `OptionalDeps` remain distinct.
4. Load-on-demand units are not treated as startup-reachable.
5. Flavor/Interface variants remain separate.
6. Embedded libraries are classified by declared/project evidence, not directory-name heuristics alone.
7. SavedVariables declarations establish state roots, not permission to parse live user data.
8. A path present on disk but absent from the active load graph is not automatically reachable.

## XML rules

- use a structural parser with external entities/network disabled;
- preserve document/include order and source spans;
- bound depth, attributes, children, text, and include recursion;
- distinguish templates, concrete frames, inheritance, object parentage, and script bodies;
- Lua bodies inside XML become source facts for Emmy; they are never executed;
- unknown elements/attributes are preserved or reported, not guessed;
- cross-file template resolution is generation/profile scoped.

## Incremental invalidation

The invalidation planner must distinguish:

```text
content-only Lua change
TOC metadata/order/dependency change
XML include/template/inheritance change
project configuration/profile change
recognizer pack/version change
reference profile/generation change
file add/remove/rename
```

A reference/profile change invalidates every dependent project fact even when source files are unchanged. A local comment-only change must not force unrelated graph partitions to rebuild if the analyzer confirms no semantic effect.

## E0 minimal slice

Implement only:

- one explicit project root and one fixture Lua file set;
- normalized file IDs/digests;
- analyzer workspace assembly;
- one coherent project generation token;
- update/publish/last-known-good behavior sufficient for the E0 golden test;
- capability reporting for unsupported TOC/XML/project features.

E0 does not require general TOC/XML parsing, persistent graph storage, filesystem watch mode, multi-root workspaces, or SavedVariables.

## Full E2 implementation sequence

1. project config and file inventory;
2. TOC parser/variants/load facts;
3. XML parser/include/template facts;
4. graph partition emission;
5. core recognizer invocation;
6. incremental invalidation and generation publication;
7. Project Map inputs;
8. launch addon corpus and false-positive evaluation.

## Required tests

### E0

- one project generation binds exact file/analyzer/profile identities;
- changed digest produces a new generation;
- failed analyzer update leaves last-known-good active;
- cross-generation input rejected;
- deterministic inventory order.

### E2

- TOC order, dependencies, optional deps, LOD, variants, missing file, duplicate entry;
- XML include order, inheritance, templates, scripts, unknown nodes, entity expansion rejection;
- unreachable/use-before-load facts;
- SavedVariables root extraction without reading persisted data;
- universe separation;
- update invalidation matrix;
- profile change invalidation;
- malformed file partition isolation;
- no code execution;
- deterministic project snapshot.

## Documentation sources

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/AGENT_WORKFLOW.md`](../../docs/AGENT_WORKFLOW.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [Current WoW addon development workflow](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_DevWorkflow.md)
- [Current Blizzard subsystem/source router](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_SubsystemRouter.md)
- [Current LOD/bootstrap route](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_Lifecycle_LoadOnDemand.md)
- [Current XML/templates/pools route](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_XMLTemplates_Pools.md)
- [Current TOC reference](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/addon/Addon_TOC_Reference.md)

## Definition of done

The E0 slice is complete when one workspace publishes a coherent reproducible generation. Full E2 is complete when TOC/XML/load/project facts are exact, incrementally replaceable, profile-isolated, safely parsed, and sufficient for reachability diagnostics and a bounded Project Map without executing addon content.
