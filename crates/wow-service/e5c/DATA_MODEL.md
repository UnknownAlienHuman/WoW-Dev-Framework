# E5-C data model

**Status:** normative transport-independent model.

## Identity DAG

```text
exact PromotionSubmission + referenced E5-A/B evidence + publication profile
-> SubmissionRevalidationReport

SubmissionRevalidationReport + exact core pack semantic bytes + owner validation receipts
-> CorePackArtifact

CorePackArtifact + provenance/SBOM/license/notice attestations
-> CorePackAttestationSet

CorePackArtifact + signing authorization/profile/key reference
-> DetachedSignatureEnvelope

artifact + attestations + signatures + catalog profile
-> CorePackPublication

publication + fresh read-back validation
-> ValidatedInactivePublication

validated publication + canary profile/cohort/authorization
-> CanaryPlan -> CanaryAssignment

assignment + typed observations
-> CanaryEvaluation

validated publication + canary evaluation + rollout profile/authorization
-> RolloutPlan -> RolloutStageReceipt[]

exact publication + execution profile + expected current digest + activation authorization
-> CurrentCorePackRecord

exact validated active publication + evidence + retention + designation authorization
-> LastKnownGoodDesignation

exact failed/current publication + exact retained rollback target + expected current digest
-> RollbackReceipt
```

No later ID is included in an earlier artifact identity.

## Core pack artifact

```text
CorePackArtifact
    artifact ID/version
    exact PromotionSubmission and revalidation report
    trust_class = core
    immutable pack/rule/operator/literal/schema/registry/profile bytes and digests
    producer namespace/profile
    compatibility matrix
    required project/graph reindex profile
    deactivation and stale-closure plan refs
    evidence/blocker/nonclaim manifest
    canonical digest
```

## Attestation set

```text
CorePackAttestationSet
    artifact ID
    build/reproducibility/provenance attestations
    dependency/SBOM manifest
    license/notice/redistribution decisions
    toolchain/implementation/profile identities
    builder identity under authorization policy
    canonical digest
```

## Signature envelope

```text
DetachedSignatureEnvelope
    artifact/attestation digest targets
    signature algorithm/profile
    nonsecret key ID/version/trust-root refs
    signature bytes
    authorization receipt
    verification policy/result
    issuance/expiry/revocation evidence where required
    canonical digest
```

## Publication

```text
CorePackPublication
    publication ID/version
    exact artifact/attestation/signature IDs
    catalog/store/schema/profile IDs
    state: Building | PublishedInactive | ValidatedInactive | CanaryAssigned | CanaryActive | RolloutPaused | Active | Superseded | Revoked | Quarantined | GCEligible
    publication/read-back reports
    retention/audit refs
    canonical digest
```

## Canary

```text
CanaryCohortPlan
    exact publication and execution profile
    cohort-selection profile and exact membership commitment
    privacy policy
    start/stop/expiry/budget
    required observation schemas/signals
    success/pause/rollback criteria
    authorization receipt
    canonical digest

CanaryObservation
    exact publication/cohort/project/profile/window
    registered signal kind/schema
    typed value/classification
    source/runtime adapter and provenance
    coverage/conflict/privacy state
    append-only sequence/audit refs
    canonical digest

CanaryEvaluation
    exact observation manifest
    per-signal result and denominator
    missing/partial/conflict/NotEvaluated state
    scoped conclusion: Pass | Fail | Pause | InsufficientEvidence | Conflict | Cancelled
    explicit nonclaims
    canonical digest
```

## Rollout and activation

```text
RolloutPlan
    exact publication/profile/canary evidence
    finite ordered stages
    exact cohort expansion per stage
    required authorization/signals
    pause/fail/rollback criteria
    maximum duration/budget
    canonical digest

RolloutStageReceipt
    stage ID
    expected prior stage/current record
    exact assignments and observations
    gate decision
    durable effect/audit/retention state
    canonical digest

CurrentCorePackRecord
    execution profile/current-record ID
    exact active publication/artifact/producer namespace
    previous current record
    CAS base digest
    activation authorization and validation refs
    canonical digest
```

## Last-known-good

```text
LastKnownGoodDesignation
    exact execution profile
    exact retained publication/artifact
    qualifying validation/canary/rollout evidence
    designation policy and authorization
    supersedes prior designation
    retention receipt
    canonical digest
```

## Rollback/revocation/closure

```text
RollbackPlan
    exact current/failed publication
    exact retained rollback target and LKG designation if used
    expected current digest
    affected execution/project profiles
    project reindex and graph closure plan
    authorization/budgets

RollbackReceipt
    exact old/new current records
    activation effect receipt
    project/graph/recognizer closure receipts
    observation/audit/retention state
    canonical digest

RevocationRecord
    exact artifact/publication/signature/profile scope
    reason codes/evidence/authorization
    serving/activation/deactivation requirements
    supersession and audit refs
    canonical digest

PartitionClosureReport
    exact pack producer namespace/version
    affected project generations and target reindex generations
    old/new graph producer partition manifests
    stale/retained/foreign/core partition proof
    coverage downgrade/change report
    validation status
    canonical digest
```

## Durable operation

All effects use `OperationId + CanonicalRequestDigest`, exact expected state, owner receipts, response-loss reconciliation, retention, audit, and reverse closure. Host clocks/process IDs/paths do not enter semantic identities.