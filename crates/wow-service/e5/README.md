# `wow-service` E5-B calibration orchestration, review, holdout, and promotion-submission contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e5-b/calibration-review-holdout-promotion-submission`

## Mission

Coordinate exact retained E5-A calibration artifacts into durable, reviewable evaluation operations without moving recognizer, graph-validation, corpus, label, split, metric, or deactivation algorithms into `wow-service`.

```text
exact immutable E5-A corpus/split/pack/fact/candidate artifacts
+ exact project/analyzer/graph publication identities
+ durable operation and retention profiles
+ independent reviewer authorization
+ independently authorized sealed-holdout access
-> exact artifact acquisition and compatibility validation
-> durable shadow/mutation/evaluation run orchestration
-> immutable run, audit, review, and access receipts
-> candidate artifact validation and reviewer decision records
-> sealed-holdout execution with contamination/consumption accounting
-> immutable PromotionSubmission candidate for E5-C
```

E5-B prepares evidence for publication consideration. It does not publish, activate, canary, roll out, or roll back a core recognizer pack.

## Canonical reading order

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`ARTIFACT_CATALOGS_AND_SELECTORS.md`](ARTIFACT_CATALOGS_AND_SELECTORS.md)
5. [`DURABLE_RUN_ORCHESTRATION.md`](DURABLE_RUN_ORCHESTRATION.md)
6. [`REVIEW_AUTHORIZATION.md`](REVIEW_AUTHORIZATION.md)
7. [`SEALED_HOLDOUT_ACCESS.md`](SEALED_HOLDOUT_ACCESS.md)
8. [`PROMOTION_SUBMISSIONS.md`](PROMOTION_SUBMISSIONS.md)
9. [`RETENTION_IDEMPOTENCY_AND_RECOVERY.md`](RETENTION_IDEMPOTENCY_AND_RECOVERY.md)
10. [`RESULT_ENVELOPE_AND_STATUS.md`](RESULT_ENVELOPE_AND_STATUS.md)
11. [`SECURITY_PRIVACY_AND_AUDIT.md`](SECURITY_PRIVACY_AND_AUDIT.md)
12. [`ERROR_MODEL.md`](ERROR_MODEL.md)
13. [`TEST_MATRIX.md`](TEST_MATRIX.md)
14. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
15. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
16. [`../../../apps/wow/e5/`](../../../apps/wow/e5/README.md)

## Public operations

```text
calibration_status
calibration_source_validate
calibration_corpus_validate
calibration_corpus_admit
calibration_split_validate
calibration_run_submit
calibration_run_get
calibration_run_list
calibration_run_cancel
calibration_run_retry
calibration_case_explain
calibration_candidate_build
calibration_candidate_validate
calibration_review_validate
calibration_review_record
calibration_holdout_request
calibration_holdout_execute
calibration_holdout_audit
calibration_promotion_prepare
calibration_promotion_validate
calibration_promotion_get
calibration_deactivation_validate
```

## Active direct dependency slice

```text
wow-core
wow-store
wow-project
wow-graph
wow-recognizers
```

`wow-reference`, `wow-emmy`, and source materializers are not direct E5-B dependencies. Their exact immutable outputs arrive through project/publication and E5-A artifacts. `wow-context`, `wow-search`, `wow-rules`, `wow-cbm`, and application crates are not service dependencies for this package.

## Owner boundaries

`wow-recognizers` owns E5-A artifact validation, corpus admission semantics, fact-snapshot validation, pack validation, shadow matching, mutation execution, metric evaluation, case explanation, candidate construction, and deactivation-plan validation.

`wow-graph` owns independent graph proposal validation and graph proof/coverage semantics.

`wow-project` supplies exact retained project/analyzer/fact publication identities and read-only handles.

`wow-store` owns durable operation, artifact, receipt, retention, audit, idempotency, and recovery storage through registered operations.

`wow-service` owns exact selector resolution, acquisition ordering, cross-owner compatibility, durable operation orchestration, authorization-port invocation, holdout-vault orchestration, conservative result envelopes, and reverse-order closure.

`apps/wow` owns only strict CLI transport over this service contract.

## Required distinctions

```text
metric eligibility != reviewer authorization
reviewer authorization != graph validity
reviewer authorization != holdout access authorization
holdout access != promotion approval
promotion submission != core publication
core publication != activation
activation != runtime correctness
```

No combination of metrics, unique candidates, repository ownership, GitHub identity, or successful graph validation collapses these gates.

## Holdout boundary

A sealed holdout may be accessed only after exact candidate pack bytes, implementation/profile IDs, run request, evaluator identity, authorization scope, and retention policy are frozen. The service records every request, authorization, access, execution, disclosure, denial, failure, and consumption state.

Raw holdout membership or labels are not exposed to pack authors or the CLI unless an explicit reviewed disclosure profile permits a bounded artifact. Normal operation returns evaluation summaries and exact audit receipts.

## Promotion submission boundary

`PromotionSubmission` is immutable evidence requesting E5-C consideration. It binds exact candidate, corpus, split, runs, graph validation, mutation, metrics, review, holdout, license/privacy, deactivation, and blocker records.

It cannot:

- change candidate pack bytes;
- authorize itself;
- publish a core pack;
- become a default execution profile;
- erase failed or superseded evidence;
- claim runtime behavior or ecosystem-wide generalization.

## Documentation phase

No `Cargo.toml`, `.rs` source, workflow, CI, placeholder module, fake authorization adapter, fake holdout vault, fake measured run, or fake promotion evidence is added in E5-B documentation.

## Completion gate

E5-B implementation is complete only when exact artifact catalogs, durable operation state, response-loss recovery, cancellation, reviewer authorization, holdout access, contamination accounting, promotion submission, retention, audit, privacy, and CLI contracts are implemented and frozen; all owner calls remain delegated; no hidden current/latest or source execution exists; no authorization/evidence gate is upgraded; and all synthetic plus admitted real-corpus tests pass with exact checksums.
