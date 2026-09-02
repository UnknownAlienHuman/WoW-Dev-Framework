# E7-B data model

**Status:** normative.

## Identity chain

```text
ReleaseSourceSnapshot + ReleasePlan
-> ReleaseBuildExecution[]
-> UnsignedReleaseArtifactSet
-> ReproducibilityComparison
-> ReleaseEvidenceSet
-> ReleaseSignatureSet
-> ReleaseBundle
-> ReleaseCandidate
-> ChannelReleaseRecord
-> SignedUpdateManifest
-> InstallationPlan -> InstallationReceipt
-> UpdatePlan -> UpdateReceipt
-> UpdateRollbackReceipt
```

Later IDs never enter earlier artifact identities. Operational delivery/install state cannot change immutable bundle identity.

## Source snapshot

```text
ReleaseSourceSnapshot
    repository and exact commit/tree IDs
    complete path/mode/blob manifest
    submodule/vendor/dependency source manifests
    Cargo.lock and workspace manifest digests
    Rust toolchain/profile inputs
    dirty/untracked/forbidden-file report
    source license/notice/provenance state
    canonical digest
```

Branch/tag names are metadata only.

## Release plan

```text
ReleasePlan
    release version and channel intent
    exact source snapshot
    target triples and platform profiles
    package/features/default-feature set
    dependency/vendor/materialization profile
    build-script/proc-macro/native-tool policy
    environment allowlist and deterministic time/path profile
    operation registry/compatibility/schema inputs
    artifact/layout/compression profile
    SBOM/provenance/license/signing profiles
    install/update/rollback/support profiles
    independent builder requirements
    resource budgets and authorization
    canonical digest
```

## Build execution

```text
ReleaseBuildExecution
    BuildOperationId + CanonicalRequestDigest
    exact release plan/source/materialized inputs
    builder implementation/host/target profile
    executor authorization and receipt
    build phases and fixed command-plan IDs
    stdout/stderr/log artifact refs under redaction
    produced unsigned artifacts and digests
    reproducibility-relevant environment receipt
    resource/result/retention/close state
    canonical digest
```

Builder host instance and timing are operational evidence, not artifact identity.

## Unsigned artifact set

```text
UnsignedReleaseArtifactSet
    artifact-set ID
    target triple/platform/package/features
    wow executable bytes/digest
    compatibility manifest and operation registry
    public schemas/config examples/user docs
    license and notice candidates
    exact archive-layout candidate
    artifact validation/self-description report
    canonical digest
```

No platform signature/notarization is included.

## Reproducibility comparison

```text
ReproducibilityComparison
    exact release plan and independent build IDs
    per-artifact byte/digest comparison
    permitted nonsemantic container differences if profile allows
    path/timestamp/debug/build-ID normalization evidence
    conflicts/omissions/NotEvaluated
    conclusion: Reproducible | ReproducibleWithDeclaredWrapperVariance | Mismatch | NotEvaluated | Failed
    canonical digest
```

## Evidence set

```text
ReleaseEvidenceSet
    exact unsigned artifact set
    SBOM documents
    build/source provenance attestations
    dependency/vendor/source integrity reports
    license/notice/redistribution decisions
    vulnerability/advisory report under exact data snapshot when required
    operation registry/schema/compatibility validation
    platform self-check/test/benchmark reports
    reproducibility comparison
    blockers/nonclaims
    canonical digest
```

A vulnerability database result is time/snapshot-scoped and never proves future absence.

## Signature set

```text
ReleaseSignatureEnvelope
    domain-separated target kind/ID/digest
    signing profile/algorithm
    nonsecret key/trust-root ID/version
    authorization receipt
    detached signature bytes
    issuance/expiry/revocation/time evidence
    verification report
    canonical digest

ReleaseSignatureSet
    exact required signature envelopes
    signature-policy closure
    canonical digest
```

Platform signing/notarization receipts remain separate from portable detached signature identity.

## Release bundle

```text
ReleaseBundle
    bundle ID/version
    exact target/channel eligibility profile
    unsigned artifact/evidence/signature sets
    deterministic file layout, names, modes and archive profile
    manifest/checksum/signature verification entry points
    optional exact offline data-pack members
    installation/update compatibility metadata
    complete size/digest/member manifest
    canonical digest
```

## Support matrix

```text
ReleaseSupportMatrix
    release/bundle IDs
    supported OS editions/architecture/target/runtime prerequisites
    local IPC endpoint profiles
    CLI/LSP/MCP protocol profiles
    store/schema/migration compatibility ranges
    Reference/core/provider artifact compatibility
    supported WoW flavor/build/profile classes
    tested feature/exposure profiles
    resource envelopes
    support/retirement dates or release-relative policy
    known exclusions and NotEvaluated capabilities
    canonical digest
```

## Release candidate

```text
ReleaseCandidate
    candidate ID
    exact release plan/source/build/artifact/evidence/signature/bundle/support refs
    channel target
    install/update/rollback validation reports
    release notes/changelog provenance
    blockers/conflicts/partial/NotEvaluated state
    authorization/review/audit/retention
    state: Draft | Blocked | Prepared | Validated | ReadyForChannel | Published | Revoked | Retired | Quarantined
    canonical digest
```

## Channel record

```text
ChannelReleaseRecord
    channel/profile ID
    exact ReleaseCandidate/Bundle/Manifest
    expected prior channel record/digest
    publication authorization and publisher receipt
    availability/read-back verification
    rollout/visibility state
    supersession/revocation/retirement links
    canonical digest
```

## Update manifest

```text
SignedUpdateManifest
    manifest ID/version
    channel and exact release/bundle
    target/platform/profile applicability
    minimum/current/blocked version relationships
    bundle locations as distribution-adapter records
    exact sizes/digests/signatures/trust roots
    install/update/migration/rollback policy refs
    revoked/retired release exclusions
    expiry/freshness policy and trusted evidence
    detached signature/verification
    canonical digest
```

## Installation state

```text
InstallationRecord
    installation ID and product data root
    exact installed release/bundle/target/profile
    executable/member digests
    active/current installation pointer and expected prior digest
    store/config/schema/data-pack compatibility state
    last-known-runnable qualification
    install/update/rollback history
    verification/health/retention/audit
    canonical digest
```

Paths are operational owner records, not portable bundle identity.

## Installation/update plans

```text
InstallationPlan
    exact current installation or empty state
    exact target bundle/support/update manifest
    destination/staging/backup owner handles
    disk/path/permission/process-lock profile
    executable replacement strategy
    store/config/schema migration steps
    post-install validation/self-check
    rollback target and cleanup policy
    authorization/budgets/cancellation
    canonical digest

UpdatePlan
    exact current and target installations
    channel/update manifest resolution receipt
    download/materialization/verification plan
    installation plan
    restart/process-handoff strategy
    rollback and retention plan
    canonical digest
```

## Effect receipts

```text
ReleasePublicationReceipt
InstallationReceipt
UpdateReceipt
UpdateRollbackReceipt
ReleaseRevocationRecord
ReleaseRetirementRecord
```

Each binds exact operation/request, expected prior state, owner effects, read-back/verification, retention/audit/reconciliation, and canonical digest.

## Last-known-runnable

```text
LastKnownRunnableInstallation
    exact installation/release/bundle
    qualifying verification/self-check/platform evidence
    rollback compatibility and retained member set
    designation authorization and expected prior designation
    canonical digest
```

It is never inferred from previous/newest.

## Coverage axes

Keep source, dependency, toolchain, build, reproducibility, artifact, evidence, signature, bundle, target/platform, protocol, store/schema, WoW profile, distribution, installation, migration, update, rollback, support, revocation, retirement, privacy/license, retention, audit, and delivery coverage independent.