# Curated corrections

**Status:** normative E1-B reviewed correction data, application, expiry, conflict, and audit contract.

Corrections address exact known upstream/source projection defects. They are not an escape hatch for uncertain APIs, stale documentation, fuzzy matching, or runtime behavior.

## 1. Principles

- raw source observations remain immutable and visible;
- correction affects only an exact declared normalized projection;
- exact expected source/value digest is mandatory;
- applicability is profile/build/entity/field scoped;
- evidence/reviewer/rationale/version are mandatory;
- mismatch expires/rejects; never best effort;
- correction set identity participates in ReferenceGeneration;
- no product/repository/provider-name code branch;
- conflicts/unknown fields are not auto-corrected by model inference.

## 2. Correction record

```text
CuratedCorrection
    correction_id
    correction_version
    correction_set_id
    target profile/build applicability
    target exact entity candidate/entity/raw observation ID
    target structured field path
    expected source file/content/value/observation digest
    expected old canonical value or shape predicate
    replacement canonical raw/normalized value
    correction operation kind
    evidence/source handles[]
    rationale
    reviewer identity/review record
    created/updated supplemental metadata
    dependency/order/conflict policy
    affected capability partitions
    canonical digest
```

## 3. Operation kinds

Initial allow-listed operations:

```text
replace-supported-field-value
add-normalized-projection-from-reviewed-source-evidence
remove-invalid-normalized-projection-while-retaining-raw
rename-normalized-field-or-entity-only-with-exact-source-contract
fix-type-or-member-shape
fix-predicate-or-restriction-projection
fix-explicit-deprecation-transition-projection
```

Not allowed:

```text
execute code
arbitrary SQL/data transform
wildcard/fuzzy target selection
whole-system heuristic patch
runtime spell whitelist
hide/delete raw observation
infer replacement from similar name/prose
suppress unsupported/conflict without evidence
```

## 4. Expected digest binding

At minimum bind to exact:

```text
SourceSnapshot/content digest
source file content digest
raw observation/value digest
entity candidate/field path
normalizer/field registry version if relevant
```

The correction may define a narrow structural expected-value predicate, but it too is versioned/digest-bound and cannot widen target matching silently.

## 5. Application order

Default:

```text
parse/evaluate raw
-> classify/normalize source projection
-> validate correction set/profile
-> select exact target corrections
-> validate expected source/value/shape digest
-> detect correction dependency/conflict
-> apply to normalized projection
-> record before/after/application/provenance
-> recompute dependent facts/coverage/manifests deterministically
```

Corrections do not alter parser/evaluator raw output.

## 6. Correction set

```text
CuratedCorrectionSet
    set ID/version
    profile/build applicability
    ordered correction records
    reviewer/evidence policy
    dependency graph
    expected source snapshot/content constraints
    canonical digest
```

Order is explicit only where one correction depends on another; otherwise canonical by target/field/ID. Dependency graph must be acyclic.

## 7. Application statuses

### `Applied`

Exact target/source/value/shape/applicability match; replacement validates.

### `Expired`

Target/source/value digest changed. Correction does not apply and required capability/profile release eligibility may downgrade pending review.

### `Rejected`

Correction malformed, unsupported operation, missing evidence/reviewer, invalid replacement, wrong profile/generation.

### `Conflict`

Multiple corrections/source facts compete incompatibly for same projection or correction contradicts another accepted evidence source.

### `NotApplicable`

Correction set intentionally includes another profile/build scope.

Every status produces a record; no silent skip.

## 8. Application record

```text
CorrectionApplicationRecord
    application_id
    correction/set/profile/build IDs
    target entity/raw observation/field path
    observed source/value digests
    expected digests
    status/reason
    normalized before/after values/fact IDs
    evidence/reviewer refs
    dependent facts/capabilities/coverage/conflict refs
    producer/version
    canonical digest
```

Raw observation ID remains unchanged.

## 9. Replacement validation

Replacement must satisfy:

- field registry accepted value/type/shape;
- entity/member/signature identity invariants;
- profile/applicability constraints;
- cross-reference target resolves exactly or remains explicit unresolved as allowed;
- restriction/predicate/deprecation structure valid;
- no identity collision unless correction explicitly resolves a reviewed collision;
- no unsupported raw field deletion;
- no capability authority upgrade beyond evidence.

## 10. Conflicting corrections

Detect:

```text
same target/field different replacement
one removal and one replacement
correction dependency cycle
correction target altered by earlier correction unexpectedly
correction versus exact source from another required partition
correction creating duplicate entity/member identity
```

Conflict blocks dependent normalized capability/negative authority unless a reviewed higher correction-set version resolves it.

Do not choose higher file order/newer timestamp/first returned.

## 11. Expiry and review workflow

On source/profile/normalizer change:

1. revalidate every applicable correction;
2. mark exact statuses;
3. generate compact correction review report with source handles/diffs/digests;
4. do not update expected digest automatically;
5. reviewer either removes, updates with new evidence/version, or confirms no longer needed;
6. new correction set/generation identity created;
7. old ReferenceGeneration remains reproducible.

## 12. Provenance and confidence

Corrected normalized fact evidence includes:

```text
raw platform source observation
curated correction application
correction evidence/reviewer
normalizer/producer
coverage/conflict state
```

Confidence can be Proven/Derived only according to exact accepted correction/source contract. A correction is not automatically stronger than platform source; it is a reviewed projection override with transparent provenance.

## 13. Capability/coverage impact

Examples:

- applied correction may restore supported projection completeness when correction policy explicitly allows and all inputs/evidence complete;
- expired mandatory correction makes dependent partition Partial/Conflict/NotEvaluated and blocks release/negative authority;
- rejected malformed correction fails correction-set validation;
- NotApplicable does not downgrade another profile;
- conflict blocks affected fields/entities/capabilities, not unrelated partitions.

Coverage record references application IDs.

## 14. Storage

Persistent schema supports:

```text
correction set/records
expected/observed value/source digests
replacement canonical values/fact IDs
application statuses
before/after fact links
evidence/reviewer/rationale refs
dependency/conflict graph
coverage/capability links
```

Static correction bundle is repository-owned/versioned input; no user/source/runtime mutation through service/application.

## 15. Security/privacy

- correction evidence uses stable handles/digests, not secret/private payloads;
- reviewer identity can be stable public project identity, not local credentials;
- no code/SQL/script execution;
- rationale/prose is nonexecutable and excluded from matching semantics except identity as explicitly defined;
- source comments cannot request/apply a correction;
- no network fetch during application;
- bound correction count/dependency depth/replacement size/report output.

## 16. Determinism

Equivalent source/profile/normalizer/correction set yields equivalent:

```text
selected correction set/order
application statuses
before/after facts
conflict/coverage impacts
correction manifests/digests
```

Independent of file insertion/order where dependencies absent, reviewer prose formatting, timestamps, temp roots, worker order.

## 17. Required operations

```text
validate_curated_correction
validate_curated_correction_set
build_correction_dependency_graph
select_applicable_corrections
validate_correction_expected_target_and_digest
apply_correction_to_normalized_projection
validate_corrected_projection
record_correction_application
detect_correction_conflicts
propagate_correction_status_to_coverage
build_correction_review_report
canonicalize_correction_set_and_applications
```

## 18. Required tests

- exact valid apply;
- raw observation unchanged;
- source file/value/shape/profile/normalizer digest mismatch -> Expired;
- wrong profile -> NotApplicable;
- missing evidence/reviewer/unsupported operation -> Rejected;
- same target conflicting replacements -> Conflict;
- dependency order/cycle;
- correction creating invalid type/entity/duplicate identity rejected;
- applied mandatory correction restoring exact declared capability;
- expired/conflict blocking negative authority/release eligibility;
- correction set changes generation identity;
- source/provider same logical bytes behaves same;
- randomized independent correction order -> same result;
- no wildcard/fuzzy/product-name/runtime-whitelist behavior;
- no raw delete or auto-digest-update.

## 19. Hard stops

- no hidden code patch;
- no fuzzy/wildcard target;
- no digest auto-update/best effort;
- no raw observation mutation/deletion;
- no runtime state whitelist;
- no source-comment/user-driven application;
- no correction conflict first/last wins;
- no authority upgrade without exact evidence/coverage;
- no old ReferenceGeneration mutation.
