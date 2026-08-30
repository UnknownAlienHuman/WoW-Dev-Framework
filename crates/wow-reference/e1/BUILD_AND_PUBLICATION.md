# ReferenceData build and immutable publication

**Status:** normative E1-B build pipeline, stage state, store integration, validation, cancellation, and manifest contract.

A successful parse is not a published ReferenceData generation. All input, evaluation, normalization, correction, coverage, persistent closure, integrity, and exact-read gates must pass for the declared capability/eligibility profile.

## 1. Build state machine

```text
Requested
-> SnapshotVerified
-> ProfileValidated
-> InputsResolved
-> Parsed
-> Evaluated
-> RawPreserved
-> Normalized
-> CorrectionsEvaluated
-> ConflictsResolvedOrRecorded
-> CoverageFinalized
-> StorePlanFinalized
-> StoreBuiltAndValidated
-> StorePublishedAndReopened
-> ReferenceViewValidated
-> ReferenceDataManifestFinalized
-> Completed
```

Terminal candidate states:

```text
Cancelled
Failed
Quarantined
```

No stage skipped. Per-partition Partial/NotEvaluated can coexist with a candidate/release build only when capability declaration/eligibility policy allows and every blocker is explicit.

## 2. Build request

```text
ReferenceBuildRequest
    request ID
    SourceSnapshotManifest / root
    exact ReferenceProfile candidate
    input partition policy
    parser/evaluator environment/field registry IDs
    correction set
    persistent schema/operation/validation bundle IDs
    wow-store configuration/runtime profile
    object/redistribution/license policy
    capability/eligibility target
    budgets/cancellation
    requested durability level
```

No implicit current profile/source/correction/schema selection.

## 3. Snapshot/profile preflight

Before parsing:

- verify root/file manifest/digests/path/security/size/license records;
- validate flavor/Interface/build/source identity;
- validate parser/evaluator/normalizer/schema/correction compatibility;
- resolve required/optional/ignored/unexpected input partitions/files/order;
- verify budgets and cancellation;
- verify `wow-store` runtime/schema prerequisites;
- reject mixed/contradictory/stale requested target.

Failure produces no candidate ReferenceGeneration.

## 4. Parsing/evaluation scheduling

Files may parse/evaluate concurrently only when:

- semantic environment/order dependencies respected;
- result identity/order independent of completion order;
- deterministic failure/cancellation aggregation;
- bounded worker/resources;
- no shared mutable Lua/runtime environment.

Registration observation order follows declared semantic/file/source order, not completion order.

## 5. Raw preservation stage

Persist/build candidate raw records before or together with normalization plan, but do not publish partial raw-only store as complete ReferenceData.

Checks:

- all successful evaluator values/registrations have raw observations/source handles;
- unknown/unsupported/parse diagnostics recorded;
- duplicate occurrences retained;
- raw manifests/counts/digests complete for processed partitions;
- large raw objects validated under license/object policy;
- no raw source mutation by correction.

## 6. Normalization stage

- classify exact entity candidates;
- build stable entity/fact/member/type/restriction/deprecation records;
- preserve raw/evidence closure;
- resolve exact intra-generation references;
- detect duplicates/conflicts;
- no fuzzy/external/other-profile fallback;
- validate normalized invariants/counts/manifests;
- unsupported dependent facts remain absent with blocker records, not malformed projections.

## 7. Corrections stage

- validate exact correction set/profile/source constraints;
- build dependency graph/order;
- apply only exact digest-bound corrections to normalized projection;
- record Applied/Expired/Rejected/Conflict/NotApplicable;
- preserve raw before value/source;
- recompute exact dependent fact/coverage/conflict/manifests;
- expired/conflicting mandatory correction can block release eligibility/authority.

## 8. Coverage/conflict stage

For every declared capability partition:

- aggregate source/parser/evaluator/raw/normalizer/correction/reference-resolution/store-plan state;
- record Complete/Partial/Unknown/Failed/NotApplicable/NotEvaluated;
- link unknown/unsupported/conflict/truncation/runtime blockers;
- build conservative summaries/release capability declaration;
- validate dependency closure;
- do not precompute broad naked authoritative absence.

## 9. ReferenceData generation identity

Construct after canonical input/model/correction/schema/coverage manifests are stable and before store plan, using domain-separated noncyclic fields:

```text
ProfileIdentity
SourceSnapshot/content digest
parser/evaluator/normalizer/field-registry versions
correction-set ID/digest
persistent schema/operation/validation bundle IDs/digests
canonical raw/normalized/conflict/coverage manifest digests
build contract version
```

StoreGenerationId later binds exact ReferenceGeneration and logical data/object manifests. Neither identity directly hashes a structure containing itself.

## 10. Store build plan

```text
ReferenceStoreBuildPlan
    plan ID
    exact ReferenceGeneration/Profile
    schema/runtime/store configuration IDs
    canonical phase-ordered registered operation invocations
    object write/reference plan
    expected row/object/count/digest manifests
    mandatory validation catalog invocations
    budgets/cancellation/durability
```

Validate all records/operation IDs/ordering/dependencies/bounds before handing to `wow-store`.

No raw SQL, source paths as SQL identifiers, or callback with hidden behavior.

## 11. Store publication handoff

Invoke `wow-store`:

```text
build staging store
apply exact schema/migration path
execute registered write plan
write/verify objects and reference set
run store + reference validation catalogs
seal and publish immutable generation
reopen final path read-only
optionally publish active pointer for StoreId/profile policy
```

`wow-reference` validates returned store IDs/manifests/reports against exact build plan/generation/profile. It does not reimplement SQL/file publication.

Any store failure/cancellation means no ReferenceData completion/manifest activation. Prior active store remains unchanged.

## 12. Post-publication read validation

Open exact published store through ReferenceView and execute frozen queries:

```text
profile/generation/manifest
known API found
known event/table/restriction/raw metadata found
known exact absent under complete partition -> authoritative absence
known absent under partial/conflict -> nonauthoritative
profile/generation mismatch rejected
bounded list/raw source-handle operations
coverage/correction/conflict manifests/counts
```

Read results must match canonical build records/manifests.

## 13. ReferenceData manifest finalization

After successful store/read validation:

```text
ReferenceDataManifest
    ProfileIdentity / ReferenceGenerationId
    source snapshot/content/provider provenance refs
    parser/evaluator/normalizer/field registry IDs
    correction set/application manifest
    raw/normalized/unknown/unsupported/conflict/coverage manifests
    store generation/manifest/integrity/open-validation IDs
    raw metadata/source-map object refs
    license/provenance refs
    declared capability/eligibility state
    build report/checksums/tool versions
    canonical digest
```

Manifest does not claim annotation parity/output or full UI graph.

## 14. Candidate/release eligibility

### Fixture

Synthetic/incomplete; exact declared fixture capabilities only.

### Candidate

Real source build with explicit pending/partial/deferred gates; not release authority beyond declared exact capabilities.

### Release

Mandatory gates pass:

```text
exact profile/source/licenses
all mandatory input partitions handled
parser/evaluator/normalizer/correction/store/read compatibility
raw unknown preservation
no unresolved mandatory conflict/expired correction
capability/coverage manifest complete for declared release capabilities
immutable store published/reopened/validated
checksums/manifests complete
no security/budget/truncation blocker
```

No annotation requirement inside E1-B; final Reference Pack release later may require annotation artifact/parity before pack-level eligibility.

## 15. Failure isolation

Per-file/partition issue:

- preserve diagnostics/partial records;
- keep independent facts/capabilities usable;
- do not publish a manifest claiming stronger capability;
- release eligibility depends on declared mandatory scope.

Fatal build issues:

```text
snapshot/profile/root identity invalid
parser/evaluator policy incompatible/untrusted
raw/normalized identity/reference closure invalid
schema/operation/build plan invalid
store publication/integrity/reference-view mismatch
manifest/checksum/reference closure invalid
security violation
```

## 16. Cancellation

Check at every stage/file/batch/store handoff. Cancellation:

- no background continuation;
- no completed ReferenceData manifest;
- no active pointer change through this build unless store atomic call already completed and state classified exactly;
- candidate/local records discarded/quarantined per policy;
- published immutable but inactive store (if cancellation after generation publication) classified by store recovery, not claimed complete ReferenceData until read/manifest gates rerun.

## 17. Resumption/retry

E1 may restart from source inputs deterministically. Reuse exact valid content-addressed objects allowed. Reuse parsed/fact/store intermediate caches requires a separately frozen exact identity/validation contract; no path/mtime/name-based reuse.

If store generation published but ReferenceData manifest finalization failed, revalidate exact generation/build inputs and resume only through an explicit recovery operation; never mutate store.

## 18. Reports

Per stage record:

```text
inputs/outputs IDs/counts/digests
processed/failed/partial/skipped partitions/files
budgets/cancellation
unknown/unsupported/conflict/correction states
store/object/publication/open results
warnings/errors/deferred capabilities
```

Human timing/host/temp path supplemental/noncanonical.

## 19. Determinism

Equivalent logical source/profile/parser/evaluator/normalizer/correction/schema/build policy yields equivalent:

```text
ReferenceGenerationId
raw/normalized/correction/conflict/coverage manifests
ReferenceStoreBuildPlan
logical StoreGeneration/ReferenceData manifest refs
ReferenceDataBuildReport semantic digest
ReferenceView golden results
```

Physical SQLite byte reproducibility reported by `wow-store` separately.

## 20. Required operations

```text
validate_reference_build_request
preflight_reference_build
parse_and_evaluate_input_partitions
build_raw_reference_records
build_normalized_reference_records
apply_reference_corrections
build_reference_conflicts_and_coverage
build_reference_generation
build_reference_store_build_plan
invoke_reference_store_publication
validate_published_reference_store_against_build
open_and_validate_reference_view
build_reference_data_build_report
build_reference_data_manifest
classify_reference_data_eligibility
abort_or_recover_reference_build_candidate
```

## 21. Required tests

- full fixture/candidate/release build states;
- every stage failure/cancellation/budget point;
- deterministic source/worker/operation order;
- partial independent partition retention;
- expired correction/unknown restriction/parse failure release gating;
- store build/validation/publication failure leaves prior active;
- published inactive recovery before ReferenceData completion;
- post-open query/manifests/counts mismatch rejects completion;
- no annotation/UI graph/search/runtime claim;
- no raw/store/source identity leakage;
- no cache/reuse without exact identity.

## 22. Hard stops

- no successful manifest before store/read gates;
- no published partial truth under undeclared capability;
- no background work after cancel;
- no in-place store update;
- no store success overriding ingestion partial/conflict;
- no annotation/parity/full UI graph claim;
- no hidden profile/source/correction/schema selection;
- no physical SQLite determinism overclaim.
