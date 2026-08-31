# E3-A implementation plan

**Status:** normative implementation order; Rust implementation has not started.

## Phase 0 — prerequisite and byte freeze

- complete/freeze E0/E1/E2 implementations and fixtures;
- pin exact SQLite/store, graph registry, recognizer, analyzer, annotation/reference profiles;
- select and freeze source materializer profile;
- freeze synthetic platform-source corpus;
- freeze one exact real mirror snapshot, complete content manifest, build/profile assertion, and license/provenance state;
- freeze all normalized expected outputs and member/bundle SHA-256 values;
- update implementation frontier before adding Cargo/Rust.

## Phase 1 — source profile and snapshot validation

Implement E3-A profile types, source authority/compatibility assertions, root/inventory validation, byte/path/case/license records, and security/resource limits. No parsing or persistence.

Run `BUI-SRC-*`, `BUI-PKG-001..005`, security/path/license mutations.

## Phase 2 — platform project/package specialization

Implement `BlizzardUiPlatformSource` project kind and configure the existing E2-C root/package/TOC variant pipeline. Do not fork parser code.

Run package/variant/dependency/order tests.

## Phase 3 — XML/load/Lua-unit specialization

Use E2-C XML/load and virtual-Lua contracts with platform-source universe/profile binding. Implement exact package/file/unit manifests and source mappings.

Run `BUI-XML-*` and load tests.

## Phase 4 — analyzer plan and fact adapters

Build exact `wow-emmy` Main/Library workspace, validate immutable analyzer snapshot, and adapt only supported typed facts with loss/coverage records.

Run `BUI-ANA-*`.

## Phase 5 — graph producer partitions

Implement source inventory, TOC/load, XML, analyzer-adapter, and E2-B core recognizer proposal partitions. Validate through the exact E3 graph registry profile.

Run `BUI-GRAPH-*`, producer replacement, confidence, conflict, and authority mutations.

## Phase 6 — candidate, fingerprints, and skeleton inputs

Assemble/validate `BlizzardUiIndexCandidate`, exact-generation structural fingerprints, and bounded `SkeletonInputView`. No Project Map/L0/L1 rendering.

Run `BUI-FP-*`, `BUI-SKEL-*`.

## Phase 7 — incremental update and removal closure

Implement dependency-driven invalidation/reuse/no-change and complete stale-record closure from source through graph/fingerprint/skeleton input.

Run `BUI-UPD-*` and randomized update sequences.

## Phase 8 — E2-D publication

Compose project/graph/store schema-operation-validation contracts, build inactive generation, fresh read-back validation, CAS activation, exact readers, restart/recovery, retention, and GC integration.

Run `BUI-PUB-*` plus E2-D crash/lease/recovery tests.

## Phase 9 — real mirror and rename corpus

Execute the frozen real source snapshot with:

- complete configured-root inventory;
- package/TOC/XML/load/analyzer/graph/skeleton-input expectations;
- provider/repository/root/package/path rename mutations;
- 1/2/N worker and shuffled-input/update determinism;
- size/latency/memory/database/WAL/skeleton-input benchmark classification;
- license/provenance/excerpt policy checks.

No source is executed.

## Phase 10 — final freeze and handoff

Populate all implementation commits, tool/profile IDs, source/manifests, expected IDs, benchmark reports, and SHA-256 values. Verify tests read committed fixtures and never regenerate them automatically.

Publish the E3-B handoff document containing exact public types/operations and deferred context responsibilities.

## Deferred

- model/natural-language context selection;
- Project Map/L0/L1/context pack generation;
- search ranking/FTS;
- cross-build lineage and impact;
- named calibration packs;
- external candidates/Codebase Memory;
- runtime observations;
- CI/release automation.
