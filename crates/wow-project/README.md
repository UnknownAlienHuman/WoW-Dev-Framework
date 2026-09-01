# `wow-project` contract router

**Status:** E0-D, E2-C, E3-A and the E4-B lineage-input seam are implementation-ready documentation; E4-C orchestration is defined in `wow-service`. Rust implementation has not started.

`wow-project` owns exact materialized project/source universes, TOC/XML/load interpretation, analyzer/recognizer orchestration, incremental project generations, project publication semantics, Blizzard UI source indexing, and project-owned lineage input producers. It does not own graph acceptance, search ranking, lineage promotion, migration recipes, static-impact traversal, service orchestration, or storage internals.

## Canonical routes

### E0-D — minimal project generation

Read the root contract package beginning with [`E0_OVERVIEW.md`](E0_OVERVIEW.md), [`CONTRACT.json`](CONTRACT.json), and the root normative documents.

### E2-C — full addon project indexing

Read [`e2/README.md`](e2/README.md). It defines exact source snapshots, TOC/XML/load, physical and virtual Lua units, analyzer/recognizer/graph proposal handoff, incremental invalidation, and immutable project candidates.

### E3-A — Blizzard UI source universe

Read [`e3/README.md`](e3/README.md). It defines a separate exact `blizzard_ui_source` project/graph publication and bounded `SkeletonInputView` for context/search consumers.

### E4-B — lineage producer inputs

Read [`E4_B_LINEAGE_INPUTS.md`](E4_B_LINEAGE_INPUTS.md). It defines:

```text
project_stable_identity
project_source_fingerprint
project_structural_change
```

These are exact producer partitions acquired and invoked through [`wow-service/e4`](../wow-service/e4/README.md), then submitted to independent `wow-graph` E4-B validation. Fingerprints and structural similarity remain Candidate evidence; project producers do not accept lineage, declare replacement, build migration recipes, authorize reviews, or run static impact.

## E4-C handoff

```text
exact before/after ProjectPublication selectors
-> wow-service resolves/acquires exact project views
-> wow-project emits exact E4-B producer partitions
-> wow-graph validates proposals/proof ceilings and publishes immutable lineage state
```

`wow-service` cannot alter project facts or proof ceilings. `wow-project` never resolves current, calls search/lineage/context, or observes review credentials.

## Direct dependency boundary

Maximum permitted dependencies:

```text
wow-core
wow-store
wow-emmy
wow-graph
wow-recognizers
```

Active package slices remain narrower. `wow-project` never depends on `wow-search`, `wow-context`, `wow-service`, `wow-cbm`, `wow-rules`, or applications.

## Cross-generation boundary

Generation-local project entities, ProjectSnapshots and GraphSnapshots remain immutable. E4-B output references exact before/after project generations and does not rewrite their IDs.

A repository, owner, package, path, name, signature, body digest, fingerprint or search rank cannot establish lineage by itself. `Removed`/`Introduced` requires exact closed before/after coverage evaluated by `wow-graph`.

## Current implementation state

```text
documentation frontier: E4-B producer seam; E4-C orchestration linked
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Implementation still begins from E0 dependency order; the E4-B seam cannot activate before E2/E3 project, E2 graph and E4-A search prerequisites are implemented and frozen.
