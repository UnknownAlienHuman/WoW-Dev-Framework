# E1-D decisions

**Status:** normative.

## E1D-001 — Service orchestrates; components remain authoritative owners

`wow-service` coordinates public component contracts. It never reimplements store, reference, or annotation algorithms.

## E1D-002 — Dedicated operational surface

E1-D exposes `reference_pack_build`, `reference_pack_validate`, and `reference_pack_rebuild_compare`. These do not expand the normal runtime query surface.

## E1D-003 — Materialized source only

Build input is an exact verified materialized source snapshot manifest. Acquisition, branch selection, mirrors, and network download are outside E1-D.

## E1D-004 — One exact profile and component set

One build binds one exact profile, source snapshot, core/store/reference/annotation implementation set, schema set, correction set, renderer/consumer/oracle profiles, and pack layout profile.

## E1D-005 — Build and validation are separate operations

Build may produce a candidate. Validation independently reads it and never repairs, fills, or regenerates members.

## E1D-006 — Pack assembly follows component finalization

Pack manifest and pack identity are created only after ReferenceData/Store and annotation artifacts have stable identities and validated reports.

## E1D-007 — Application materializes a typed plan

The library produces a root-confined `PackMaterializationPlan`. The dedicated application performs filesystem writes. No generic filesystem or shell callback enters domain code.

## E1D-008 — Staging and atomic destination replacement

Writes occur in a unique staging root. Final destination publication is atomic where supported; an existing validated pack is never modified in place.

## E1D-009 — Local validation is not release publication

E1-D can produce `validated-local`; signing, registry upload, channel activation, release notes, and distribution belong to E7.

## E1D-010 — Component capability axes remain separate

Reference source coverage, store integrity, annotation projection coverage/loss, parity, consumer compatibility, and pack assembly closure remain distinct. One complete axis cannot erase another blocker.

## E1D-011 — No hidden current or latest

No profile, source, dependency, oracle, consumer, or output layout is selected from a floating token.

## E1D-012 — Validation is nonrepairing

A missing/mismatched member fails or blocks according to policy. Validation never rewrites checksums or regenerates content.

## E1D-013 — Logical and physical determinism are separate

ReferenceData logical identity, canonical manifests, annotations, maps, and sidecars can require byte equality. SQLite physical bytes and archive bytes require explicit store/container guarantees.

## E1D-014 — Rebuild comparison is a first-class test operation

Equivalent logical requests are built under frozen worker/order profiles and compared by declared equivalence classes. It is not inferred from one build.

## E1D-015 — Partial output is quarantined

Cancelled/failed staging output has no final pack manifest or validated eligibility. Cleanup/quarantine policy is explicit and bounded.

## E1D-016 — Prior valid output remains untouched

A failed candidate never replaces an existing destination or active pointer. Last-known-good identity is reported, not relabeled as the requested target.

## E1D-017 — Licenses and redistribution policy are mandatory members

Every included object/file has source/provenance/license/redistribution classification. Unknown mandatory licensing blocks validated-local status.

## E1D-018 — No source or generated artifact execution

Build and validation are parser/data operations. No Lua, annotation stubs, hooks, scripts, package managers, or external repositories are executed.

## E1D-019 — External probes are explicit adapters

Ketho/EmmyLua/LuaLS processes run only through reviewed test/tool adapters outside the library crate. Service consumes validated typed results and identities.

## E1D-020 — No CI by documentation convention

E1-D defines local deterministic commands and test contracts. It does not add GitHub Actions, publishing, or release automation.
