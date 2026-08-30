# `wow-annotations` E1-C decisions

**Status:** normative for semantic projection, type lowering, deterministic rendering, source mapping, projection loss, parity, and consumer probes.

## ANN-E1-001 — Annotation output is a projection

**Decision:** the exact ReferenceView remains canonical; annotation artifacts never become platform/reference authority.

**Consequence:** raw fields, restrictions, corrections, coverage, and provenance stay owned by `wow-reference`.

## ANN-E1-002 — Direct dependencies are core and reference only

**Decision:** no store, analyzer, service, project, search, or application dependency.

**Consequence:** the crate is a pure deterministic transformation over exact reference facts.

## ANN-E1-003 — One exact ReferenceView per artifact

**Decision:** profile/reference generation cannot mix within semantic model, files, source maps, loss reports, or parity results.

**Consequence:** no cross-build signature or restriction leakage.

## ANN-E1-004 — Build semantic model before rendering

**Decision:** reference facts lower into a versioned consumer-neutral annotation semantic model first.

**Consequence:** parity and identity do not depend on whitespace/file partitioning.

## ANN-E1-005 — Semantic declaration IDs are layout-independent

**Decision:** declaration/member/type IDs derive from exact input identity and lowering semantics, not generated path/line.

**Consequence:** layout changes can preserve semantic parity/source mappings through explicit new spans.

## ANN-E1-006 — Rendering profiles are explicit

**Decision:** Ketho-compatible layout/text conventions are one versioned profile, not hardcoded hidden behavior.

**Consequence:** profile changes update artifact identity and fixtures.

## ANN-E1-007 — Ketho is a differential oracle

**Decision:** Ketho output can reveal semantic/layout differences but cannot override ReferenceView facts automatically.

**Consequence:** discrepancy classification is required; no match-at-any-cost patching.

## ANN-E1-008 — Semantic parity is primary

**Decision:** compare symbols, ownership, parameters, returns, members, types, modifiers, and selected metadata; byte equality is secondary.

**Consequence:** harmless formatting/file-layout differences do not fail semantic parity, while hidden type loss does.

## ANN-E1-009 — Consumer compatibility is separate from oracle parity

**Decision:** EmmyLua and LuaLS profiles/probes independently validate actual consumer behavior.

**Consequence:** matching Ketho text does not prove analyzer compatibility.

## ANN-E1-010 — The crate does not spawn consumer/oracle tools

**Decision:** external test/tool adapters execute pinned consumers/oracles and return typed probe results.

**Consequence:** no process/network/editor/filesystem mutation in the library crate.

## ANN-E1-011 — No editor mutation

**Decision:** artifact creation does not alter user/workspace settings, library paths, diagnostics, globals, or extensions.

**Consequence:** correctness is portable and explicit.

## ANN-E1-012 — No analyzer diagnostic suppression

**Decision:** renderer/probe cannot disable diagnostics merely to imitate expected output.

**Consequence:** consumer gaps and noise are measured/classified, not hidden.

## ANN-E1-013 — Type lowering is total over input status

**Decision:** every requested input type/fact receives Exact, ExactWithSidecar, LossyDeclared, Unsupported, or NotEvaluated.

**Consequence:** silent omission/widening is impossible by contract.

## ANN-E1-014 — `any` is never a silent fallback

**Decision:** `any` appears only when source explicitly means it or a versioned lossy rule emits it with a loss record.

**Consequence:** unknown/unresolved types do not disappear.

## ANN-E1-015 — Optionality and nilability remain distinct where source does

**Decision:** lowering cannot collapse missing parameter, optional parameter, nullable type, omitted return, and unknown field without explicit consumer limitation/loss.

**Consequence:** signatures stay faithful.

## ANN-E1-016 — Raw restrictions do not collapse to a boolean

**Decision:** known supported restriction facets may project into nominal types/tags/sidecars; unknown/conditional/runtime facets remain explicit loss/NotEvaluated.

**Consequence:** no false safe ordinary type.

## ANN-E1-017 — Nominal Secret types are analysis-only

**Decision:** generated `WowSecretValue` classes document analysis behavior, not runtime wrapper objects.

**Consequence:** consumers/rules cannot infer object methods/runtime representation.

## ANN-E1-018 — Runtime spell state is not projected as permanent static truth

**Decision:** only exact selected ReferenceView source facets project; runtime/hotfix gaps remain sidecar/loss.

**Consequence:** no permanent spell whitelist.

## ANN-E1-019 — Generated code is inert

**Decision:** functions/methods/globals use fixed nonfunctional stub forms selected by rendering profile.

**Consequence:** artifact cannot contain source-provided executable bodies.

## ANN-E1-020 — Source text cannot inject artifact structure

**Decision:** names/docs/strings are validated/escaped/sanitized under renderer-owned templates.

**Consequence:** no extra directives/code/files/globals/comment/string termination.

## ANN-E1-021 — Documentation is nonidentity and bounded

**Decision:** docs can render only through a versioned sanitization policy; omission/truncation gets a loss record.

**Consequence:** prose cannot change declaration semantics or explode artifacts.

## ANN-E1-022 — Invalid identifiers require explicit rendering

**Decision:** use safe table-index/alias form only when consumer profile supports exact semantics; otherwise Unsupported/LossyDeclared.

**Consequence:** no invalid Lua or silent rename.

## ANN-E1-023 — Stable deterministic layout

**Decision:** file partitioning/path/names/declaration order/member order/line endings are versioned and canonical.

**Consequence:** 1/2/N workers and input/store row order yield identical files.

## ANN-E1-024 — No full Blizzard UI implementation in normal library

**Decision:** artifact contains declarations/stubs/docs/source maps, not implementation source bodies.

**Consequence:** bounded context/library size and no runtime code redistribution.

## ANN-E1-025 — Generated source maps are mandatory

**Decision:** material declarations/members/types/docs map back to exact reference facts/raw/corrections/evidence/source handles and lowering rules.

**Consequence:** downstream diagnostics/parity can explain every projection.

## ANN-E1-026 — Source spans derive after final rendering

**Decision:** generated span identity depends on final file bytes/manifest and cannot be precomputed from model order alone.

**Consequence:** maps cannot drift silently after formatter/layout changes.

## ANN-E1-027 — Every representational gap gets a loss record

**Decision:** unsupported, omitted, approximated, sanitized, split, consumer-specific, conflict-blocked, and partial-source cases are explicit.

**Consequence:** successful artifact generation does not imply complete fidelity.

## ANN-E1-028 — Projection coverage is independent of reference coverage

**Decision:** artifact can be incomplete because reference is partial or because consumer format cannot express a complete fact.

**Consequence:** reports distinguish source gap from projection/consumer gap.

## ANN-E1-029 — Sidecars preserve non-LuaCATS semantics

**Decision:** source maps/loss/restriction/provenance manifests can carry exact metadata that generated Lua cannot.

**Consequence:** no need to force every field into comments/types.

## ANN-E1-030 — Semantic artifact identity includes all profiles

**Decision:** ReferenceGeneration, semantic model, type-lowering, layout/rendering, dialect/global, sanitization, source-map/loss, and consumer profiles enter artifact identity.

**Consequence:** no hidden behavior drift.

## ANN-E1-031 — Physical file manifest is separate from semantic manifest

**Decision:** semantic declarations/types and rendered files/checksums have distinct identities linked by renderer version.

**Consequence:** semantic-equivalent layout changes are classifiable.

## ANN-E1-032 — Output writes are outside the library

**Decision:** crate returns artifact file bytes/manifests/plans; configured root-confined writing belongs to a higher application/tool.

**Consequence:** no hidden filesystem mutation.

## ANN-E1-033 — Consumer profiles are capability matrices

**Decision:** supported tags/types/declaration forms/diagnostic behavior are pinned and probed per consumer/version.

**Consequence:** no assumption that EmmyLua and LuaLS interpret all LuaCATS identically.

## ANN-E1-034 — Lowest common denominator is not automatically correct

**Decision:** artifact may be consumer-profile-specific or use explicit sidecars rather than erase richer semantics globally.

**Consequence:** compatibility does not force needless fidelity loss.

## ANN-E1-035 — Cross-consumer output strategy is explicit

**Decision:** one shared artifact is allowed only if frozen probes prove required semantics for all declared consumers; otherwise separate artifact profiles.

**Consequence:** no accidental divergence hidden under one path.

## ANN-E1-036 — Parity discrepancies are classified

**Decision:** Equal, SemanticallyEquivalent, ExpectedProjectionDifference, OurDefect, OracleDefectOrStale, InputMismatch, ConsumerDisagreement, or Unresolved.

**Consequence:** maintainers know whether to fix, update baseline, or investigate.

## ANN-E1-037 — Oracle inputs must match

**Decision:** compare only same source snapshot/profile/reference semantics where possible; input mismatch is not renderer defect.

**Consequence:** no stale-current false failures.

## ANN-E1-038 — Unknown fields remain outside silent artifact success

**Decision:** ReferenceView unknown/unsupported/conflict blockers feed loss/coverage even if no Lua declaration can represent them.

**Consequence:** annotation consumer cannot unknowingly claim completeness.

## ANN-E1-039 — Bounded artifacts and docs

**Decision:** file count, declarations, members, type depth, docs, source maps, loss records, and bytes are explicit budgets.

**Consequence:** malformed/huge inputs cannot create unbounded output.

## ANN-E1-040 — Cancellation publishes no complete artifact

**Decision:** candidate model/files/maps remain unpublished; no background continuation.

**Consequence:** no partial artifact masquerade.

## ANN-E1-041 — Fixture, candidate, and release-ready annotation states differ

**Decision:** eligibility depends on declared projection/consumer/parity/source-map/loss/determinism gates.

**Consequence:** a file set that parses is not automatically release-ready.

## ANN-E1-042 — Final Reference Pack assembly remains external

**Decision:** a higher tool links ReferenceData, annotation artifact, checksums/licenses and later signatures.

**Consequence:** no dependency cycle with `wow-reference` or release logic in this crate.

## ANN-E1-043 — Freeze renderer/oracle/consumer/artifact vectors before code

**Decision:** exact pins/profiles/declarations/files/maps/loss/parity/checksums remain null until implementation begins and then become mandatory.

**Consequence:** no unreviewed renderer behavior.
