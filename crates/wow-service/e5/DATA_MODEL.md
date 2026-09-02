# E5-B data model

**Status:** normative transport-independent model.

## Configuration

```text
E5BServiceConfiguration
    artifact catalog profile
    owner port catalog IDs
    selector/compatibility profile
    durable operation/idempotency profile
    run orchestration profile
    review authorization profile
    holdout authorization/vault/disclosure/consumption profiles
    promotion submission profile
    retention/audit/recovery profile
    privacy/license/security/budget profiles
    result/error/canonicalization profiles
    canonical digest
```

## Exact selectors

```text
CalibrationArtifactSelector
    ExactArtifactId + expected digest/type/schema
    ForExactBinding + exact owner/profile/generation tuple

CalibrationRunSelector
    ExactRunId + expected request/result digest

PromotionSubmissionSelector
    ExactSubmissionId + expected candidate/profile digest
```

There is no generic latest/best/default selector.

## Acquired artifact set

```text
ResolvedCalibrationArtifactSet
    exact candidate source/corpus/split/fact snapshot/pack IDs
    exact source/project/analyzer/graph publication IDs
    exact E5-A candidate/deactivation artifacts when applicable
    exact validation, mutation, metric, graph and security reports
    compatibility/coverage/conflict report
    retention lease set
    acquisition manifest
    canonical digest
```

## Durable operation

```text
CalibrationOperationRecord
    OperationId
    CanonicalRequestDigest
    operation kind
    exact input artifact IDs
    requested profiles/budgets
    durable state
    owner effect receipts
    output artifact IDs
    cancellation/response-loss/recovery records
    error/blocker refs
    canonical digest
```

States:

```text
Planned
Acquiring
ValidatingInputs
AwaitingAuthorization
Authorized
Executing
Evaluating
BuildingArtifact
ValidatingOutput
RetentionPending
Completed
NoChange
Cancelled
Failed
OutcomeUnknown
Quarantined
Superseded
```

## Calibration run request

```text
CalibrationRunServiceRequest
    exact corpus/split/fact snapshot/pack candidate
    visible split selection
    optional sealed holdout request reference
    matcher/graph/mutation/evaluation/security/determinism profiles
    exact implementation IDs
    OperationId
    budgets/cancellation
    output/retention/privacy profile
    canonical digest
```

## Run receipt

```text
CalibrationRunReceipt
    exact operation/request IDs
    exact E5-A run/case/metric/mutation/graph report IDs
    status and durable state
    visibility and holdout-consumption state
    coverage/conflicts/blockers
    retention/audit receipts
    canonical digest
```

## Review authorization

```text
CalibrationReviewDecisionEnvelope
    schema/version
    exact candidate artifact and review profile IDs
    decision: ApproveForSubmission | Reject | Defer | RequestAdditionalEvidence | Supersede
    bounded structured reason codes
    bounded untrusted note: optional
    reviewer principal/role/scope refs
    attestation/key/verification profile refs
    issuance/expiry/revocation/replay state
    canonical decision digest
```

```text
CalibrationReviewAuthorizationDecision
    Authorized | Unauthorized | Expired | Revoked | ScopeMismatch | ReplayDetected | Unsupported | NotEvaluated | Failed
    exact authorization evidence refs
    permitted decision/scope ceiling
    canonical digest
```

Authorization does not alter candidate evidence.

## Immutable review record

```text
CalibrationReviewRecord
    exact candidate artifact
    exact review envelope digest
    authorization decision
    independent candidate/output validation refs
    recorded decision
    supersedes/superseded-by refs
    retention/audit refs
    canonical digest
```

## Holdout request and grant

```text
HoldoutAccessRequest
    exact sealed holdout generation/digest
    exact candidate pack/candidate artifact
    exact implementation and run request
    evaluator principal/service identity
    requested visibility/disclosure scope
    purpose and retention profile
    OperationId
    canonical digest
```

```text
HoldoutAccessGrant
    grant ID
    exact request
    authorization decision/evidence
    vault scope token reference (nonsecret handle only)
    permitted member/label/result visibility
    use count and expiry/revocation/replay policy
    canonical digest
```

Raw vault credentials are excluded.

## Holdout execution and audit

```text
HoldoutExecutionReceipt
    exact grant/request/candidate/run
    vault/evaluator implementation profile
    exact sealed generation opened
    exact E5-A operation/result IDs
    disclosure class
    contamination/consumption decision
    coverage/conflicts/failure/cancellation
    retention refs
    canonical digest
```

```text
HoldoutAccessAuditRecord
    immutable event ID
    event kind: Requested | Authorized | Denied | Opened | Evaluated | Disclosed | Failed | Cancelled | Revoked | Consumed
    exact actor/principal/service refs
    exact grant/candidate/run/holdout generation
    bounded reason/result refs
    trusted time/sequence evidence refs
    previous audit event digest
    canonical digest
```

## Holdout consumption

```text
HoldoutConsumptionRecord
    sealed holdout generation
    candidate lineage root and exact candidate/run
    disclosure/result visibility class
    consumed: yes/no/unknown
    reason/profile
    descendant-candidate applicability
    exact audit evidence
    canonical digest
```

## Promotion submission

```text
PromotionSubmission
    submission ID/version
    exact E5-A candidate artifact and pack bytes/digest
    corpus/split/provenance/license/privacy/notice refs
    visible-split run and metric reports
    sealed-holdout execution/consumption/audit refs
    graph/mutation/security/determinism/deactivation reports
    immutable review records and authorization decisions
    hard-gate/blocker/conflict/NotEvaluated manifest
    claimed generalization scope and explicit nonclaims
    requested E5-C target channel/profile
    state
    canonical digest
```

States:

```text
DraftCandidate
Blocked
Prepared
Validated
ReadyForE5CReview
Rejected
Withdrawn
Superseded
Quarantined
```

No state means published or active.

## Service envelope

```text
E5BServiceResultEnvelope
    operation/request IDs
    exact resolved input/output artifact IDs
    conservative status and validation state
    durable/idempotency/response-loss state
    reviewer and holdout authorization summaries
    coverage/conflicts/blockers/omissions
    retention/audit/closure summaries
    privacy/license/security summaries
    mandatory nonclaims
    canonical digest
```

## Statuses

```text
Complete
NoChange
Partial
Blocked
CandidateOnly
ConflictBlocked
Truncated
NotEvaluated
Cancelled
Failed
OutcomeUnknown
```

Validation payload:

```text
Valid
Invalid
NotEvaluated
```

## Identity exclusions

Wall-clock duration, process ID, terminal, host path, raw credentials, raw signatures, vault token bytes, thread/worker ID, and physical database layout never enter semantic artifact identity.
