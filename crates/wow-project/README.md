# `wow-project` contract router

**Status:** E0-D, E2-C and E3-B implementation contracts are documented; Rust implementation has not started.

`wow-project` owns exact source/project generations, project/source universe registries, TOC/XML/load modeling, analyzer and recognizer orchestration, incremental invalidation, project/source candidates and domain publication bundles. It does not implement Lua analysis, graph semantics, SQLite internals, diagnostics, search or runtime WoW behavior.

## Current route — E3-B Blizzard UI source universe

Read in order:

1. [`e3/README.md`](e3/README.md)
2. [`e3/AGENTS.md`](e3/AGENTS.md)
3. all normative files listed by [`e3/CONTRACT.json`](e3/CONTRACT.json)
4. supporting owner seams:
   - [`../wow-reference/e3/BLIZZARD_UI_SOURCE_PROFILE.md`](../wow-reference/e3/BLIZZARD_UI_SOURCE_PROFILE.md)
   - [`../wow-graph/e3/BLIZZARD_UI_SOURCE_GRAPH.md`](../wow-graph/e3/BLIZZARD_UI_SOURCE_GRAPH.md)
   - [`../wow-context/e3/BLIZZARD_UI_INPUT_HANDOFF.md`](../wow-context/e3/BLIZZARD_UI_INPUT_HANDOFF.md)

E3-B accepts one externally materialized sealed source snapshot, indexes it as a separate `blizzard_ui_source` universe, validates exact reference/source bridges, and publishes a dedicated source ProjectStore/Graph generation through E2-D. It never treats implementation source as API/runtime/security authority and never mutates a user project current record.

## E2-C — user addon project indexing

Read [`e2/README.md`](e2/README.md), then its complete contract package. E2-C turns one exact materialized addon-project snapshot into a deterministic `ProjectIndexCandidate` with TOC/XML/load/analyzer/recognizer/graph proposal manifests. It stops before persistence; E2-D owns persistent publication.

## E0-D — minimal project generation seam

The original high-level overview is preserved as [`E0_OVERVIEW.md`](E0_OVERVIEW.md). The root detailed E0 files remain normative for the minimal vertical slice:

- [`AGENTS.md`](AGENTS.md)
- [`DECISIONS.md`](DECISIONS.md)
- [`DATA_MODEL.md`](DATA_MODEL.md)
- [`SOURCE_REGISTRY.md`](SOURCE_REGISTRY.md)
- [`UPDATE_MODEL.md`](UPDATE_MODEL.md)
- [`GENERATION_AND_PUBLICATION.md`](GENERATION_AND_PUBLICATION.md)
- [`ERROR_MODEL.md`](ERROR_MODEL.md)
- [`TEST_MATRIX.md`](TEST_MATRIX.md)
- [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
- [`CONTRACT.json`](CONTRACT.json)
- [`examples/`](examples/README.md)

## Direct dependencies by frontier

```text
E0-D
    wow-core
    wow-emmy

E2-C
    wow-core
    wow-emmy
    wow-graph
    wow-recognizers

E2-D publication seam
    wow-store

E3-B
    wow-core
    wow-emmy
    wow-graph
    wow-recognizers
    wow-store
```

Allowed edges are an upper bound; agents activate only dependencies required by the current implementation package.

## Stable ownership boundaries

```text
wow-emmy
    Lua parser/analyzer and semantic facts

wow-recognizers
    universal declarative structural matching and graph proposals

wow-graph
    entity/relation semantics, assertions, conflicts, coverage and GraphSnapshot

wow-store
    SQLite/WAL/transactions/read snapshots/current CAS/recovery/retention/GC

wow-reference
    public API/reference profile and authority; no reversed project dependency

wow-project
    source/project roots, TOC/XML/load, analyzer/recognizer orchestration,
    invalidation, candidate and domain publication bundle
```

## Mandatory external routing

Before changing patch-sensitive WoW assumptions, read the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) `AGENTS.md`, `INDEX_MINI.md`, and exact task route. Stable framework contracts reference exact profiles/evidence; they do not copy live patch conclusions.

## Implementation state

```text
documentation frontier: E3-B
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI workflows: absent
```

No implementation begins until the selected package's prerequisite commits, profiles, real/synthetic fixtures, mutation vectors and SHA-256 freeze gate are complete.
