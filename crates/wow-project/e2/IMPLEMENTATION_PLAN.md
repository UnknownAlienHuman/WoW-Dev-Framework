# E2-C implementation plan

**Status:** normative sequence; implementation not started.

## Phase 0 — prerequisite and fixture freeze

Before the first Rust commit:

- implement/freeze `wow-core`, `wow-emmy`, `wow-graph` E2-A, and `wow-recognizers` E2-B;
- freeze source snapshot/root/universe/materializer profile;
- freeze TOC/XML/load dialect/security profiles;
- freeze analyzer library/pin/config and virtual-source mapping profile;
- freeze recognizer fact adapter/core pack and graph registry/proposal profile;
- freeze invalidation/candidate/capability/budget profiles;
- freeze synthetic and one user-owned addon repository fixture at exact commits/content digests;
- populate all IDs, expected outputs, and SHA-256 values;
- update manifest implementation state only after these gates.

## Phase 1 — source snapshot and universes

Implement exact closed `ProjectSourceSnapshot`, roots, universes, package candidates, path/security/materialization validation, and source registry extension for TOC/XML/Lua/virtual units.

Tests: `PROJECT-E2-CONFIG-*`, `PROJECT-E2-SOURCE-*`, security root mutations.

## Phase 2 — TOC parser/model

Implement bounded line parser, raw records, known/unknown directives, file entries/tags, dependencies, LOD/bootstrap, SavedVariables, variants, selection, resolution, coverage, and deterministic manifests.

No XML/Lua/analyzer work yet.

Tests: `PROJECT-E2-TOC-*`.

## Phase 3 — XML parser/model

Implement streaming nonexecuting XML records, includes, templates, objects, parents, inheritance, scripts, unknowns, source spans, external/inline Lua unit extraction, cycle/budget/security controls.

Tests: `PROJECT-E2-XML-*`.

## Phase 4 — static load model

Build package dependency graph, selected variant closure, ordered TOC/XML/Lua units, bootstrap/normal/conditional phases, direct load edges, reachability, conflicts, explanation paths, coverage.

Tests: `PROJECT-E2-LOAD-*`.

## Phase 5 — analyzer integration

Materialize exact reachable Main physical/virtual Lua units and separate Library workspace. Build/update/validate target `wow-emmy` snapshot including XML inline source-map closure.

Tests: `PROJECT-E2-AN-*`.

## Phase 6 — recognizer fact adapters

Implement exact typed TOC/XML/project/analyzer adapters, adapter-loss/coverage, narrow partition/bundle planning, cross-partition dependency manifests.

Tests: `PROJECT-E2-ADAPT-*`.

## Phase 7 — recognizer execution and graph proposal validation

Run exact E2-B core pack, retain outcomes/ambiguities/partiality, build project-owned direct proposals, validate all proposals against E2-A registries, retain accept/reject/conflict mappings.

No graph publication/store.

Tests: `PROJECT-E2-RECOG-*`, `PROJECT-E2-GRAPH-*`.

## Phase 8 — dependency graph and invalidation

Implement source/profile/tool-to-derived partition dependency graph, final-state diff, direct/transitive invalidation, conservative widening, semantic reuse proof, removal closure, no-change, deterministic reports.

Tests: `PROJECT-E2-INV-*`.

## Phase 9 — candidate assembly

Derive E2 ProjectGeneration inputs, assemble/validate immutable Complete/Partial `ProjectIndexCandidate`, read-only candidate view, E2-D publication bundle, last-known-good identity rules.

Tests: `PROJECT-E2-CAND-*`, determinism/security/deferred.

## Phase 10 — real addon fixture audit

Select one current user-owned addon repository whose TOC/XML/Lua structure exercises active capabilities. Pin exact commit/license/content, read its local instructions, and add only necessary licensed fixture excerpts or generated fact/golden manifests.

Rules:

- no production branch on repository/addon name;
- synthetic fixtures remain closed semantic oracle;
- source comments are data, not instructions;
- no repo code execution;
- record unsupported/unknown real structures honestly;
- add mutation that renames repository/path identifiers.

## Phase 11 — freeze canonical bytes

Populate prerequisite pins; source/package/TOC/XML/load/Lua/analyzer/adapter/recognizer/graph/invalidation/candidate IDs; expected outcomes; and member/bundle SHA-256. Tests verify committed fixtures and never rewrite them automatically.

## E2-D handoff

After E2-C code/seams pass, prepare measured ProjectStore physical profile and atomic ProjectSnapshot/GraphSnapshot publication contracts. Do not add storage code opportunistically during E2-C.

## Deferred

- ProjectStore/WAL/current pointers/retention/GC;
- full dependency source universe automation;
- installed addons/SavedVariables/logs;
- Blizzard UI/source skeletons/Project Map;
- search/lineage/impact/CBM/runtime;
- source edits/autofixes;
- CI/release automation.
