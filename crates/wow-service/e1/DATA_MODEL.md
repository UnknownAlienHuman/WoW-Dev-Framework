# E1-D data model

**Status:** normative orchestration and pack-level contract.

## 1. Build request

```text
ReferencePackBuildRequest
    request_id
    exact SourceSnapshotManifest/root handle
    exact ProfileIdentity candidate
    component contract/implementation/schema/profile identities
    correction set and ReferenceData eligibility target
    annotation semantic/type/layout/dialect/sanitization/map/loss profiles
    oracle and consumer result/profile identities
    pack layout profile
    staging/materialization policy
    pack eligibility target
    budgets/cancellation
    deterministic execution profile
```

No raw command line, environment-derived current profile, or arbitrary callback belongs here.

## 2. Component set

```text
ReferencePackComponentSet
    wow_core implementation/schema/canonicalization IDs
    wow_store implementation/runtime/schema/operation/validation IDs
    wow_reference implementation/parser/evaluator/normalizer/correction IDs
    wow_annotations implementation/semantic/type/layout/consumer/oracle IDs
    compatibility report IDs
    component fixture/checksum bundle IDs
```

The set is immutable for one build.

## 3. Build context

```text
ReferencePackBuildContext
    request and component-set IDs
    source snapshot/profile IDs
    staging session ID
    exact budgets/cancellation token
    build attempt ID
    prior destination/last-known-good observation: optional, noncanonical
```

A build attempt is not a pack identity.

## 4. Component outputs

```text
ReferenceDataComponent
    ReferenceGenerationId
    ReferenceDataManifestId
    StoreGenerationId / immutable ReferenceStore handle
    ReferenceViewId
    build/coverage/conflict/correction/license reports

AnnotationComponent
    AnnotationArtifactId
    semantic/file/source-map/coverage/loss/parity/consumer manifests
    rendered file descriptors/bytes or object refs
    eligibility/build report
```

Component identities must share the exact profile/reference generation required by the pack profile.

## 5. Pack layout profile

```text
ReferencePackLayoutProfile
    profile_id/version
    allowed member kinds
    fixed logical paths
    path normalization/case policy
    manifest/checksum/license locations
    object embedding/reference policy
    compression/container policy: optional/deferred
    canonical member ordering
    per-member and total budgets
    required/optional/deferred member rules
    canonical digest
```

## 6. Staging and materialization

```text
PackStagingSession
    session_id
    destination intent ID
    application-owned staging root handle
    root policy ID
    existing destination observation
    cleanup/quarantine policy
```

```text
PackMaterializationEntry
    member_id
    member kind
    artifact-relative path
    content source = inline canonical bytes | immutable object ref | sealed store file ref
    expected byte length/digest
    mode/metadata policy
    license/provenance refs
```

```text
PackMaterializationPlan
    plan_id
    staging session/layout/profile/reference generation
    ordered entries
    directory set
    expected manifest/checksum IDs
    atomic finalization instruction
    canonical digest
```

## 7. Pack manifest

```text
ReferencePackManifest
    pack_manifest_id/version
    pack_id
    ProfileIdentity / ReferenceGenerationId
    source snapshot/content/provider provenance
    component set and component manifest IDs
    layout profile ID
    ordered member entries
    capability/coverage/conflict/loss/parity/consumer summaries with source IDs
    license/redistribution manifest ID
    checksum manifest ID
    build and validation report IDs
    eligibility state
    deferred capabilities
    tool/schema versions
    canonical digest
```

The manifest contains no field that recursively hashes itself.

## 8. Pack member

```text
ReferencePackMember
    member_id/kind/path
    logical identity
    content/object/store/artifact identity
    bytes/digest/encoding
    profile/reference generation
    required/optional policy
    provenance/license refs
    validation rules/results
```

Initial E1 member kinds:

```text
pack-manifest
pack-checksums
reference-store
reference-data-manifest
reference-build-report
annotation-file
annotation-semantic-manifest
annotation-file-manifest
annotation-source-map
annotation-projection-coverage
annotation-projection-loss
annotation-parity-report
annotation-consumer-manifest
annotation-consumer-probe-result
annotation-artifact-manifest
license-or-notice
provenance-manifest
```

Raw APIDocumentation objects may be referenced as immutable objects when the selected pack profile/license policy includes them; they are not automatically embedded.

## 9. Validation request/report

```text
ReferencePackValidationRequest
    exact candidate root/manifest handle
    expected pack/profile/reference/layout/component IDs: optional explicit assertions
    validation profile
    budgets/cancellation
```

```text
ReferencePackValidationReport
    request/candidate/pack IDs
    path/member/checksum/schema/store/reference/annotation/map/loss/parity/consumer/license checks
    exact passed/failed/NotEvaluated records
    capability/eligibility decision
    mutation/repair count = 0
    canonical digest
```

## 10. Gate record

```text
PackGateRecord
    gate_id
    gate kind
    mandatory/advisory policy
    exact input report/member/capability/loss/conflict IDs
    status = Passed | Failed | NotEvaluated | NotApplicable
    blocker/reason IDs
    canonical digest
```

No naked boolean release decision.

## 11. Build outcome

```text
ReferencePackBuildOutcome
    status = completed | blocked | failed | cancelled
    candidate/pack/materialization/report IDs when available
    component outcomes
    gate records
    cleanup/quarantine instruction
    prior destination unchanged assertion
    canonical digest for semantic outcome
```

## 12. Deterministic rebuild comparison

```text
ReferencePackRebuildComparisonRequest
    one logical build request
    execution profiles: worker counts/order seeds/platform profiles
    comparison profile
    budgets/cancellation
```

```text
ReferencePackRebuildComparisonReport
    build outcome IDs
    semantic/logical manifest comparisons
    canonical JSON/text/file comparisons
    annotation byte comparisons
    object comparisons
    SQLite logical comparison
    SQLite physical-byte classification
    container/archive classification
    normalized difference records
    gate decision
    canonical digest
```

## 13. Recovery record

```text
ReferencePackRecoveryRecord
    failed/cancelled stage
    staged/published immutable component observations
    destination/active pointer observations
    safe cleanup/quarantine/resume actions
    prohibited mutation actions
    exact identities required to resume
```

## 14. Progress

Progress is supplemental and noncanonical:

```text
stage
processed/total when exact
bounded status message code/args
```

Wall clock, host, temp root, thread IDs, and throughput do not enter pack identity.
