# `wow-project` contract router

**Status:** E0-D, E2-C, and E3-A contracts are implementation-ready documentation; no Rust code exists.

`wow-project` owns exact materialized source snapshots, project/source universes, TOC/XML/load structure, analyzer and recognizer orchestration, incremental invalidation, project-generation identity, and domain publication bundles. It never executes addon/source code and never implements storage, graph, analyzer, recognizer, rule, search, or context algorithms owned by other crates.

## Contract routes

### E0-D — minimal project generation

Read [`E0_OVERVIEW.md`](E0_OVERVIEW.md), then the root E0-D contract package:

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

### E2-C — addon-project TOC/XML/load/incremental indexing

Read [`e2/README.md`](e2/README.md). E2-C turns one exact addon source snapshot into a validated, deterministic, nonpersistent `ProjectIndexCandidate` and defines the handoff to E2-D coherent ProjectStore publication.

### E3-A — Blizzard UI source universe and structural graph

Read [`e3/README.md`](e3/README.md). E3-A activates an exact `blizzard_ui_source` project kind using the existing E2 parser/analyzer/recognizer pipeline plus E2-D publication. It produces a separate published platform-source ProjectSnapshot/GraphSnapshot and a bounded skeleton-input view for `wow-context`.

E3-A does **not** generate Project Maps, L0/L1 skeletons, context packs, search rankings, migration lineage, patch impact, or runtime truth. Those remain downstream packages.

## Direct dependencies by frontier

```text
E0-D: wow-core, wow-emmy
E2-C: wow-core, wow-emmy, wow-graph, wow-recognizers
E3-A publication: wow-core, wow-emmy, wow-graph, wow-recognizers, wow-store
```

No direct dependency on `wow-context`, `wow-search`, `wow-rules`, `wow-service`, or applications.

## Current implementation state

```text
documentation frontier: E3-A
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Every exact source, tool, profile, graph, store, fixture, benchmark, and checksum pin remains blocking before the first implementation commit.
