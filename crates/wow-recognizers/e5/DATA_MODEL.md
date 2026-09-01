# E5-A calibration data model

**Status:** normative semantic model.

## Candidate source

```text
CalibrationCandidateSource
    candidate_source_id
    repository provider/owner/name audit identity
    exact revision/commit/tree
    materialization request/profile and expected roots
    source inventory/content manifest IDs when available
    project/analyzer/graph publication IDs when available
    fact bundle/adapter profile IDs when available
    upstream/fork/copy/provenance group state
    license/notice/privacy/redistribution state
    admission state and blockers
    canonical digest
```

Repository identity is audit metadata and is excluded from matcher fields.

## Admitted corpus

```text
CalibrationCorpusManifest
    corpus_id/version
    purpose and declared generalization scope
    exact admitted candidate sources
    source/project/analyzer/graph/fact snapshot manifests
    example and label-set manifests
    provenance/upstream-lineage group manifest
    license/privacy/notice manifest
    split eligibility and exclusions
    mutation/evaluation/profile bindings
    coverage/conflicts/omissions
    canonical digest
```

## Corpus example

```text
CalibrationCorpusExample
    example_id
    exact source/publication/fact generation
    bounded RecognizerFactBundle partition refs
    exact scope/entity/source-handle refs
    expected label-set ID
    provenance group ID
    structural-shape group IDs
    semantic feature inventory
    required capability/coverage state
    privacy/license class
    mutation family refs
    split assignment ref
    canonical digest
```

The example does not contain pack output as expected truth.

## Expected labels

```text
CalibrationExpectedLabelSet
    label_set_id/version
    example ID
    expected universal entity/relation/role outputs
    label classes: Positive | Negative | Possible | NotEvaluated | Unknown | Conflict
    exact expected key ingredients/attributes/confidence ceiling
    decisive positive and negative evidence refs
    allowed ambiguity/cardinality
    reviewer decision records and disagreements
    label coverage/limitations
    canonical digest
```

## Review record

```text
CalibrationLabelReview
    review_id
    example/label item
    decision and structured reason codes
    exact evidence/source/fact refs
    reviewer principal/role/attestation refs when implemented
    independence declaration from candidate output
    bounded untrusted note
    status: Proposed | Accepted | Rejected | Deferred | Conflict | Superseded
    canonical digest
```

E5-A defines records; E5-B later owns durable authorization/workflow.

## Split model

```text
CalibrationSplitManifest
    split_manifest_id/version
    corpus ID
    group-key profile
    Train group IDs
    Dev group IDs
    Test group IDs
    SealedHoldout opaque group/member manifest digest
    Challenge group IDs
    Quarantine group IDs
    leakage-analysis report ID
    author/evaluator visibility policy
    canonical digest
```

## Mutation suite

```text
CalibrationMutationSuite
    suite_id/version
    source example IDs
    mutation cases and expected semantic deltas
    invariant fields and sensitivity fields
    generated fact-bundle IDs/digests
    source-coordinate/provenance mapping
    budgets/security profile
    canonical digest
```

## Pack candidate

```text
CalibrationPackCandidate
    pack_id/version
    trust_class = calibration
    rollout_state = shadow_only
    declared universal role/relation contracts
    E2-B fact and graph schema profiles
    bounded rule/capture/output definitions
    justified exact convention literals
    corpus/split/evaluation/mutation bindings
    generalization-scope declaration
    license/provenance/review metadata
    budgets
    canonical bytes/digest
```

## Run

```text
CalibrationRun
    run_id
    exact pack/corpus/split/profile/implementation IDs
    selected visible splits
    worker/order/cache/resource profile
    case-result manifest
    graph-validation report
    mutation/leakage/determinism/security/benchmark reports
    aggregate metric report
    hard-gate report
    status
    canonical digest
```

## Case result

```text
CalibrationCaseResult
    case_result_id
    exact run/example/mutation/rule IDs
    expected label item(s)
    observed match/proposal/output partition
    TP/FP/FN/TN eligibility and class
    Possible/Unknown/NotEvaluated/Partial/Conflict/Truncated state
    evidence/capture/explanation/coverage refs
    graph validation result
    resource/cancellation state
    pass/fail/blocker reason codes
    canonical digest
```

## Metrics

```text
CalibrationMetricReport
    exact case-result manifest
    counts and denominators by rule/role/relation/split/provenance group
    TP/FP/FN/TN where meaningful
    precision/recall and confidence intervals/profile when defined
    possible/unknown/not-evaluated/partial/conflict/truncated counts
    graph rejection and authority-upgrade counts
    mutation invariance/sensitivity counts
    leakage/security/determinism/resource results
    weighted summaries with unweighted mandatory failures retained
    threshold profile and gate decisions
    canonical digest
```

## Anti-overfitting report

```text
CalibrationAntiOverfittingReport
    split/provenance leakage checks
    repository/owner/addon/path/local-name invariance
    decisive literal/structural/coverage sensitivity
    near-miss clause coverage
    duplicate/fork/vendor/generated-code influence
    train-dev-test-holdout reuse history
    hidden named-condition/static analysis results
    generalization-scope conclusion
    blockers
    canonical digest
```

## Candidate artifact

```text
CalibrationCandidateArtifact
    candidate_artifact_id
    pack bytes/digest
    corpus/split/run/metric/mutation/anti-overfitting reports
    graph validation and deactivation plan
    license/provenance/notice records
    promotion eligibility state
    blockers/nonclaims
    canonical digest
```

It is input to E5-B review, not a promoted core pack.

## Deactivation plan

```text
CalibrationDeactivationPlan
    exact pack/rule/version/input generations
    owned shadow producer partitions
    stale proposal/assertion/result references
    expected coverage downgrade
    unaffected core/other-pack partitions
    cleanup/validation operations
    canonical digest
```

## Identity DAG

```text
candidate sources
-> admitted corpus + examples + labels + provenance groups
-> split and mutation manifests
-> pack candidate
-> run + case results
-> metrics/anti-overfitting/deactivation
-> CalibrationCandidateArtifact
-> future E5-B promotion submission
```

Labels/corpus identity never depends on pack output, preventing evaluation cycles.
