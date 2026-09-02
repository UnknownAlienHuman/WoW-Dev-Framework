# `wow-project` contract router

**Status:** E0-D, E2-C, E3-A, the E4-B lineage-input seam, and the E6-B external-locator mapping seam are implementation-ready documentation. Rust implementation has not started.

`wow-project` owns exact materialized project/source universes, TOC/XML/load interpretation, analyzer/recognizer orchestration, incremental project generations, project publication semantics, Blizzard UI source indexing, project-owned lineage inputs, and exact mapping of bounded external locators into retained project generations. It does not own graph acceptance, search ranking, external-provider semantics, candidate selection, context orchestration, migration application, static-impact traversal, service orchestration, or storage internals.

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

### E6-B — external locator mapping owner

Read [`E6_B_EXTERNAL_LOCATOR_MAPPING.md`](E6_B_EXTERNAL_LOCATOR_MAPPING.md). It defines exact mapping of an owner-neutral bounded `UnverifiedProviderLocator` projection into one retained project generation.

Mapping returns `ExactMapped`, `MultipleMappings`, `NoMappingWithOwnerAuthority`, `NoMappingPartial`, `Conflict`, `NotEvaluated`, or `Failed`. It validates repository/revision/path/digest/span/symbol/entity fields only through project-owned records. It never follows provider paths, chooses by rank/name/proximity, verifies provider summaries, or depends on `wow-cbm`/`wow-service`.

## E4/E6 service handoff

```text
E4: exact before/after ProjectPublication selectors
    -> wow-service acquisition
    -> project lineage-input producer partitions
    -> wow-graph validation

E6: exact retained ProjectPublication/Generation
    + bounded external locator projection
    -> project-owned identity validation
    -> zero/one/many exact owner handles + coverage/conflicts
    -> wow-service mapping record, explicit selection, optional context
```

`wow-service` cannot alter project facts, mapping results, proof ceilings, or negative authority. `wow-project` never resolves current, calls external providers/search/context/service, or observes credentials.

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

Generation-local project entities, ProjectSnapshots, GraphSnapshots, and mapping evidence remain exact-generation-bound. E4-B output references exact before/after generations; E6-B mapping references one exact generation. Neither rewrites IDs.

A repository, owner, package, path, name, signature, body digest, fingerprint, provider label, or search rank cannot establish lineage or provider correctness by itself. `Removed`/`Introduced` and clean no-mapping require exact closed owner coverage.

## Current implementation state

```text
documentation frontier: E6-B project mapping seam
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Implementation still begins from E0 dependency order; the E6-B seam cannot activate before E2/E3 project publication, exact mapping profiles/fixtures, E6-A, and E6-B service gates are implemented and frozen.