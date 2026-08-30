# `wow-reference` E1-B decisions

**Status:** normative for persistent ReferenceData ingestion, normalization, corrections, storage, and exact reads.

## REF-E1-001 — E1 extends rather than rewrites E0

**Decision:** the existing fixture-backed E0 contract remains the executable vertical seam; E1 adds a versioned persistent package under `e1/`.

**Consequence:** E0 consumers/tests do not silently change while the full builder is implemented.

## REF-E1-002 — Direct framework dependencies are core and store only

**Decision:** `wow-reference` depends on `wow-core` and `wow-store`; parsing uses a pinned external parser/adaptor if required.

**Consequence:** no cycle through `wow-emmy`, annotations, project, service, or applications.

## REF-E1-003 — Materialized content is authority; provider is provenance

**Decision:** profile/source identity is based on exact materialized files/digests/manifests, not provider/repository popularity or branch name.

**Consequence:** equivalent providers can yield the same logical ReferenceData.

## REF-E1-004 — Source acquisition is outside the crate

**Decision:** caller supplies a validated materialized snapshot root/manifest; E1 performs no network/checkout/update.

**Consequence:** reproducible ingestion and narrower security boundary.

## REF-E1-005 — One exact profile per ReferenceData generation

**Decision:** flavor/Interface/build/source digest/correction set/schema/parser identity are explicit and never mixed.

**Consequence:** no cross-profile API or restriction leakage.

## REF-E1-006 — Fixture and release profiles are distinct

**Decision:** incomplete synthetic fixture profiles cannot masquerade as release-eligible ReferenceData.

**Consequence:** tests remain useful without weakening production identity.

## REF-E1-007 — Parse Lua; never execute it

**Decision:** APIDocumentation ingestion uses syntax facts plus a restricted declarative evaluator.

**Consequence:** no arbitrary source side effects or dependence on a WoW/Lua runtime.

## REF-E1-008 — One pinned parser compatibility line

**Decision:** parser crate/version/API behavior is pinned and probed; no second parser or regex correctness path.

**Consequence:** deterministic spans/dialect behavior and upgrade rollback.

## REF-E1-009 — Evaluator semantics are allow-listed

**Decision:** only explicitly frozen literals/tables/bindings/access/constant expressions/registration calls/helper forms evaluate.

**Consequence:** unknown syntax is diagnosed/quarantined instead of guessed.

## REF-E1-010 — Unsupported constructs are partitioned failures, not silent drops

**Decision:** each unsupported/malformed construct records source/evaluator reason and downgrades dependent capability partitions.

**Consequence:** unaffected partitions can remain complete while negative authority stays honest.

## REF-E1-011 — Source order is explicit

**Decision:** declared TOC/registration/input order is preserved where semantics depend on it; unordered sets are canonically sorted.

**Consequence:** filesystem/thread/hash order never changes output.

## REF-E1-012 — Duplicate semantics are explicit

**Decision:** repeated registrations/fields/keys are handled by a frozen source/evaluator rule; conflicting observations remain evidence/conflicts.

**Consequence:** no accidental last-write-wins.

## REF-E1-013 — Raw canonical value tree is first-class

**Decision:** all parsed fields, including unknown nested values, are preserved as canonical raw observations.

**Consequence:** future schema/annotation support can be added without reacquiring source or losing metadata.

## REF-E1-014 — Unknown differs from absent/null/default

**Decision:** raw/normalized models encode missing, explicit nil, unsupported, unknown field, and defaulted projection separately.

**Consequence:** no false contract completion.

## REF-E1-015 — Normalized facts reference raw observations

**Decision:** every supported projection retains exact raw observation/source IDs and producer version.

**Consequence:** query result can explain provenance and projection loss.

## REF-E1-016 — Raw and normalized stores are separate logical layers

**Decision:** annotations are another later projection, not the canonical store.

**Consequence:** restriction/unknown fields do not disappear to fit LuaCATS.

## REF-E1-017 — Entity identity is structured and profile-bound

**Decision:** kind/name/namespace/owner/signature/profile/generation rules define stable keys; no fuzzy similarity identity.

**Consequence:** exact lookup and profile isolation are deterministic.

## REF-E1-018 — Exact duplicates may canonicalize; conflicts never collapse

**Decision:** only structurally equivalent observations under exact identity/digest policy deduplicate.

**Consequence:** competing source evidence remains inspectable.

## REF-E1-019 — Restriction facets are open

**Decision:** known facets normalize; unknown fields/facets remain raw and downgrade dependent checks.

**Consequence:** new Blizzard security metadata does not become “safe by omission.”

## REF-E1-020 — Runtime/hotfix-sensitive data is scoped

**Decision:** static source facts describe mechanisms/contracts; runtime-only current spell/data state is not frozen universally without exact later evidence contract.

**Consequence:** no permanent spell whitelist or source-only runtime certainty.

## REF-E1-021 — Corrections are reviewed data, never hidden code

**Decision:** each correction has ID/version/profile/target/field/expected digest/replacement/evidence/reviewer/rationale.

**Consequence:** auditability and deterministic correction-set identity.

## REF-E1-022 — Correction digest mismatch expires/rejects

**Decision:** no fuzzy target or best-effort application.

**Consequence:** upstream changes force re-review rather than silently corrupting projection.

## REF-E1-023 — Corrections preserve original raw value

**Decision:** correction changes only normalized projection and records corrected/raw relation.

**Consequence:** source history/evidence remains intact.

## REF-E1-024 — Correction set affects generation identity

**Decision:** same source with a different accepted correction set produces a different ReferenceData generation/build identity.

**Consequence:** queries/reproduction know exactly which corrected contract they used.

## REF-E1-025 — Coverage is capability plus narrow partition

**Decision:** completeness is never one global boolean.

**Consequence:** partial input does not disable unrelated queries and cannot authorize broad negatives.

## REF-E1-026 — Completeness and conflict are independent

**Decision:** fully ingested source may still have unresolved contradictory observations/corrections/projections.

**Consequence:** conflict blocks dependent authority even when ingestion is Complete.

## REF-E1-027 — Negative authority is computed, not stored as a naked boolean

**Decision:** exact profile/query/partition/capability/generation/conflict/truncation/staleness conditions are evaluated per request.

**Consequence:** empty exact lookup under partial coverage is not “absent.”

## REF-E1-028 — No fuzzy fallback in ReferenceView

**Decision:** exact lookup/list/raw/capability operations only.

**Consequence:** search/lineage/replacement remains a later owning crate.

## REF-E1-029 — Persistent schema semantics are owned by reference

**Decision:** `wow-reference` defines static domain schema/operation/validation bundles; `wow-store` executes/lifecycles them.

**Consequence:** store remains domain-neutral and reference does not own SQLite mechanics.

## REF-E1-030 — No raw SQL or connection crosses the seam

**Decision:** typed adapters invoke registered operation IDs with encoded records/results.

**Consequence:** no service/application/storage bypass.

## REF-E1-031 — ReferenceStore is immutable and generation-specific

**Decision:** changes build a new staging/sealed/published store through `wow-store`.

**Consequence:** no in-place row/schema/correction/profile update.

## REF-E1-032 — Build plan is deterministic and typed

**Decision:** canonical ordered operation/object/validation plan contains stable IDs/records, not callbacks with hidden state or arbitrary SQL.

**Consequence:** store integration and golden tests are reproducible.

## REF-E1-033 — Store rows do not define authority alone

**Decision:** persistent coverage/conflict/provenance records remain required for query decisions.

**Consequence:** missing row cannot bypass negative-authority rules.

## REF-E1-034 — ReferenceData and annotations are separate artifacts

**Decision:** `wow-reference` does not depend on `wow-annotations`; a higher assembly tool combines outputs.

**Consequence:** no dependency cycle and no projection format becoming canonical.

## REF-E1-035 — E1 UI graph scope is bounded

**Decision:** complete TOC/XML/FrameXML/function graph and source skeletons remain E2/E3 unless an exact E1 source/provenance field is required.

**Consequence:** E1 proves API/reference correctness before broad UI indexing.

## REF-E1-036 — Build/report identities exclude volatile data

**Decision:** timestamps/temp roots/worker order/provider checkout path/prose do not enter canonical identity.

**Consequence:** equivalent logical input produces equivalent manifests.

## REF-E1-037 — Every stage is budgeted/cancellable

**Decision:** input/file/syntax/evaluation/table/string/fact/raw/store/query/output bounds are explicit.

**Consequence:** malformed/huge source cannot consume unbounded resources or publish partial truth.

## REF-E1-038 — Cancellation publishes nothing partial

**Decision:** abort before immutable ReferenceStore publication; no background continuation.

**Consequence:** prior active generation remains unchanged.

## REF-E1-039 — Exact read results carry context/evidence/coverage

**Decision:** ReferenceView never returns only an unscoped fact/value.

**Consequence:** downstream rules/service can verify generation and authority.

## REF-E1-040 — Raw metadata reads are bounded and typed

**Decision:** callers request exact entity/field/observation handles with byte/count budgets.

**Consequence:** no whole-source/raw-store dump by default.

## REF-E1-041 — Current KB remains external

**Decision:** live guidance routes through the KB and exact source; it is not duplicated into persistent static contract logic.

**Consequence:** no stale “current” rules embedded in the builder.

## REF-E1-042 — Freeze source/parser/schema/fact/query vectors before code

**Decision:** all exact pins, generation/store/fact/correction/coverage/build/query IDs/digests remain null while documentation-only and become mandatory before first E1-B Rust commit.

**Consequence:** implementation cannot choose convenient unreviewed semantics.
