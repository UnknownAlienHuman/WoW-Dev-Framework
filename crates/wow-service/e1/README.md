# `wow-service` E1-D Reference Pack build and validation contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e1-d-reference-pack-build-validation`

E1-D is the orchestration boundary that turns already materialized, explicitly pinned source inputs into one locally validated Reference Pack candidate. It coordinates `wow-store`, `wow-reference`, and `wow-annotations` through their public contracts and exposes transport-independent build, validate, and deterministic-rebuild operations to a thin `wow-reference-builder` application.

E1-D does not acquire floating source, parse source itself, emit annotation syntax itself, execute external repositories, sign or distribute releases, or promote incomplete data to release authority.

## Mission

```text
explicit materialized source snapshot + exact profile + frozen component contracts
-> build and publish immutable ReferenceData through wow-reference/wow-store
-> open exact read-only ReferenceView
-> build deterministic annotation artifact through wow-annotations
-> assemble one root-confined Reference Pack candidate
-> validate manifests, checksums, coverage, loss, parity, consumer probes, licenses, and component closure
-> compare repeated logical builds under deterministic execution profiles
-> return one typed candidate/validated/blocked outcome
```

## Required reading

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../../DEPENDENCY_GRAPH.md`](../../DEPENDENCY_GRAPH.md)
3. [`../../WORKSTREAMS.md`](../../WORKSTREAMS.md)
4. E0 service contract files one directory above
5. [`../../wow-store/README.md`](../../wow-store/README.md)
6. [`../../wow-reference/e1/README.md`](../../wow-reference/e1/README.md)
7. [`../../wow-reference/e1/BUILD_AND_PUBLICATION.md`](../../wow-reference/e1/BUILD_AND_PUBLICATION.md)
8. [`../../wow-annotations/e1/README.md`](../../wow-annotations/e1/README.md)
9. [`AGENTS.md`](AGENTS.md)
10. [`DECISIONS.md`](DECISIONS.md)
11. [`DATA_MODEL.md`](DATA_MODEL.md)
12. [`BUILD_OPERATION.md`](BUILD_OPERATION.md)
13. [`VALIDATE_OPERATION.md`](VALIDATE_OPERATION.md)
14. [`ASSEMBLY_AND_LAYOUT.md`](ASSEMBLY_AND_LAYOUT.md)
15. [`DETERMINISM_AND_REBUILD.md`](DETERMINISM_AND_REBUILD.md)
16. [`APPLICATION_BOUNDARY.md`](APPLICATION_BOUNDARY.md)
17. [`ERROR_MODEL.md`](ERROR_MODEL.md)
18. [`TEST_MATRIX.md`](TEST_MATRIX.md)
19. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
20. [`CONTRACT.json`](CONTRACT.json)
21. [`../../../apps/wow-reference-builder/README.md`](../../../apps/wow-reference-builder/README.md)
22. current routes in the external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)

Normative repository sources:

- [`../../../docs/REFERENCE_PACK.md`](../../../docs/REFERENCE_PACK.md)
- [`../../../docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md)
- [`../../../docs/PROVENANCE_AND_COVERAGE.md`](../../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../../docs/SECURITY_MODEL.md`](../../../docs/SECURITY_MODEL.md)
- [`../../../docs/TEST_STRATEGY.md`](../../../docs/TEST_STRATEGY.md)

## Direct framework dependencies activated by E1-D

```text
wow-core
wow-store
wow-reference
wow-annotations
```

E0 `wow-emmy`, `wow-project`, and `wow-rules` remain independent consumers of the resulting pack. E1-D does not require them to build a pack. EmmyLua/LuaLS compatibility probes are represented by validated `wow-annotations` probe results produced through reviewed external test adapters, not by importing `wow-emmy` into the build pipeline.

## Owned responsibilities

- exact build/validation/rebuild request validation;
- frozen component contract and implementation identity selection;
- one coherent profile/reference generation across component outputs;
- build-stage orchestration and cancellation;
- staging-session identity and application write-plan construction;
- pack layout profile and component artifact closure;
- final manifest/checksum/license/provenance assembly;
- pack-level capability and eligibility classification;
- independent validation of an existing candidate;
- deterministic logical rebuild comparison;
- typed recovery/quarantine instructions;
- transport-independent progress, reports, and errors;
- no silent component fallback or generation substitution.

## Explicit non-responsibilities

E1-D does not:

- download or choose Blizzard source, a mirror branch, or a floating current profile;
- parse Lua/XML/TOC or normalize platform facts;
- execute arbitrary Lua, generated annotations, source repositories, build scripts, or addon code;
- create SQL or mutate a sealed ReferenceStore directly;
- implement annotation lowering/rendering/parity algorithms;
- write arbitrary host files from the library crate;
- mutate editor/user/workspace settings;
- run hidden shell commands or package managers;
- include complete FrameXML/UI graphs, source skeletons, lineage, search indexes, or runtime observations unless a later versioned pack profile explicitly owns them;
- sign, upload, publish, activate, or distribute a release;
- add CI or release automation.

## Public E1-D operations

```text
reference_pack_build
reference_pack_validate
reference_pack_rebuild_compare
```

These are operational service use cases, distinct from the compact runtime query surface. They remain transport-independent and may be exposed only by the dedicated builder application during E1.

## Artifact eligibility

```text
fixture
    synthetic or deliberately partial package; exact declared fixture capabilities only

candidate
    real profile package with explicit pending/nonblocking/blocked gates

validated-local
    all E1-D mandatory local integrity, provenance, coverage, loss, parity, consumer, and determinism gates pass

release-published
    not an E1-D state; signing/distribution/activation belongs to E7
```

`validated-local` does not imply official release publication or current live authority.

## Core invariant

A successful component build is not a successful Reference Pack. The pack becomes `validated-local` only after every declared member, manifest, checksum, profile/generation link, capability gate, projection loss, parity/probe result, license record, and deterministic rebuild contract has been validated at pack level.

## Hard stops

- No implicit profile/source/component version.
- No pack manifest before component and file closure.
- No raw path or arbitrary bytes accepted without a declared member kind and checksum.
- No component success overriding another component's partial/conflict/loss blocker.
- No pack repair during validation.
- No in-place mutation of a candidate under validation.
- No physical SQLite byte-equality claim when only logical determinism is guaranteed.
- No partial or cancelled candidate presented as complete.
- No output outside an application-owned staging root.
- No publication/signing/network/CI.

## Definition of done

E1-D implementation is complete when one exact materialized snapshot can produce a root-confined candidate whose ReferenceData, ReferenceStore, annotations, source maps, loss/parity/probe reports, manifests, checksums, and licenses close exactly; independent validation reaches the same decision; 1/2/N and shuffled builds have the required logical/physical equivalence; failures/cancellation never corrupt prior artifacts; and the dedicated CLI performs only the frozen service requests and safe materialization plan.
