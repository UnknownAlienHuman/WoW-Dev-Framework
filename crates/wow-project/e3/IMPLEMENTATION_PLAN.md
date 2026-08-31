# E3-B implementation plan

**Status:** normative implementation order; Rust implementation has not started.

## Phase 0 — prerequisites and freeze

Before code:

- implement/freeze `wow-core`, `wow-emmy`, E2-A graph, E2-B recognizers, E2-C project indexing and E2-D ProjectStore/public-view seams;
- freeze exact implementation commits and fixture digests;
- select one synthetic Blizzard UI source collection and one separately reviewed real sealed source snapshot;
- freeze provider/revision/content/root/file/build-binding/license/redistribution manifests;
- freeze source, parser, analyzer, adapter, recognizer, graph, bridge, store, security, budget and canonicalization profiles;
- freeze synthetic/real normalized outputs, publication/crash/reader/invalidation vectors and SHA-256 values;
- update `MANIFEST.json` implementation state before first Rust commit.

The real source fixture may remain local/handle-only, but its exact content manifest, build-binding state and license/redistribution decisions must be recorded. No production-current or release claim can be built from synthetic data.

## Phase 1 — profile, build binding and snapshot admission

Implement bounded immutable types and validation for:

- provider/revision/materializer;
- source profile and root definitions;
- materialized snapshot/root/file/content manifests;
- build-binding evidence/state;
- license/redistribution records;
- security/coverage/conflict reports.

No source parsing or filesystem/network acquisition.

Acceptance: `UISRC-PROFILE-*`, `UISRC-MAT-*`, `UISRC-BIND-*`, foundational license/security cases.

## Phase 2 — source inventory and package/global-unit registry

Implement manifest-backed root/file/source-handle inventory, file-kind classification, TOC package candidates and explicit global/shared units.

No host path traversal. Freeze source generation identity and canonical inventory manifests.

Acceptance: `UISRC-ROOT-*`, `UISRC-PKG-001..003`, path/collision/determinism cases.

## Phase 3 — TOC, XML and static load

Reuse E2-C parser/load contracts under the source profile:

- exact TOC variant/order/dependency/LOD/bootstrap/SavedVariables records;
- bounded XML include/template/object/parent/inheritance/script records;
- physical and virtual Lua units/source maps;
- direct static load graph and reachability classes.

Acceptance: remaining `UISRC-PKG-*`, `UISRC-XML-*`, `UISRC-LOAD-*`.

## Phase 4 — analyzer workspace and snapshot

Build the exact source Main workspace and annotation Library workspace. Invoke `wow-emmy`; validate exact snapshot/unit/fact/finding/source-map closure.

No second parser or raw-source fallback.

Acceptance: `UISRC-AN-*`.

## Phase 5 — fact adapters and universal recognizers

Implement source inventory/TOC/XML/load/analyzer adapters with explicit loss/coverage. Run only approved E2-B universal core packs.

Acceptance: `UISRC-FACT-*`, `UISRC-REC-*` and repository/path/name mutation corpus.

## Phase 6 — direct source graph proposals

Implement project-owned direct source entity/relation proposals and independent recognizer proposal partitions. Validate through the exact source graph registry.

Acceptance: `UISRC-GRAPH-*` excluding persistent publication cases.

## Phase 7 — reference/source bridges

Implement the frozen bridge profile and exact source/reference endpoint resolution over public views. Preserve ambiguity, profiles, authority, coverage and evidence.

Do not implement user-project bridges beyond typed unavailable/contract fixtures.

Acceptance: `UISRC-BRIDGE-*`, reference compatibility and authority mutations.

## Phase 8 — candidate and invalidation engine

Assemble/validate source candidates, dependency graph, final-state diff, exact reuse proof, removal closure, no-change and update-order determinism.

Acceptance: `UISRC-INV-*`, candidate/coverage/conflict tests.

## Phase 9 — logical source/graph schema and E2-D publication

Supply source-domain schema/operation/validation bundles and graph plan to E2-D:

```text
build dedicated source inactive generation
-> commit
-> open fresh exact read snapshot
-> validate source/analyzer/graph/bridges/license/removal/golden queries
-> CAS current source selector
```

Acceptance: `UISRC-PUB-*`, crash/recovery/reader/retention cases.

## Phase 10 — exact public source view

Implement read-only operations for:

- exact source publication/profile/generation;
- roots/packages/files/source handles;
- source graph and reference/source bridges;
- build/license/coverage/conflict state;
- bounded source queries used by context/search/service later.

No raw store/source checkout handles.

## Phase 11 — context handoff

Validate E3-A handoff fixtures:

- exact source and graph snapshots;
- source-handle/excerpt capability with license/redaction policy;
- L0/L1/Project Map input shapes;
- no full source dump or authority upgrade.

`wow-context` remains storage- and ingestion-independent.

## Phase 12 — security, resource and deterministic freeze

Run all acceptance/mutation/security suites under:

- 1/2/N workers;
- shuffled file/fact/assertion/update order;
- hostile path/XML/Lua/comment/license/bridge inputs;
- cancellation at every major phase;
- synthetic and real source scale profiles;
- crash/read-snapshot/recovery/GC scenarios.

Freeze all committed machine/rendered fixture bytes and checksums. Tests verify but never rewrite golden files.

## Deferred packages

### E3-C — persistent context partitions and service operations

Publish context artifacts through project/store orchestration and expose exact service/application operations.

### E4 — search, lineage, migration and impact

Resolve fuzzy/natural-language queries to exact project/reference/source entities; build cross-build lineage and bounded impact queries.

### E5 — calibration packs

Evaluate named framework/source corpora without hard-coded product/path conditions in universal rules.

### E7 — redistributable release pack

Only after explicit license/notice decisions, reproducibility, signing and user authorization.

## Hard stops

- no code before prerequisite/profile/real-fixture/checksum freeze;
- no acquisition/network/filesystem discovery in the library;
- no source execution or second Lua parser;
- no implementation-source API/runtime/security authority;
- no universe merge or nearest-name bridge;
- no user project current mutation;
- no source redistribution without exact positive decision;
- no CI until executable workspace commands exist and the user authorizes it.
