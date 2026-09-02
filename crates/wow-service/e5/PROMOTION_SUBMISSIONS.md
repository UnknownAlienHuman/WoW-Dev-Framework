# E5-B promotion submissions

**Status:** normative. A submission is evidence for E5-C, not publication.

`PromotionSubmission` binds the exact E5-A candidate artifact and pack bytes, admitted corpus/provenance/labels/split/fact snapshots, visible run/case/mutation/metric/anti-overfitting reports, independent graph validation, security/determinism/deactivation reports, license/privacy/notices, authorized review records, holdout grant/execution/audit/consumption records, blockers, claimed generalization scope, explicit nonclaims, and requested E5-C target profile.

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

No state means published, active, default, canary, rollout-complete, or runtime-verified.

Build procedure:

```text
register durable operation
-> acquire exact retained evidence
-> validate identity/profile closure
-> validate review authorization independently
-> validate holdout state independently
-> preserve every blocker and nonclaim
-> build immutable submission
-> validate canonical bytes and references
-> admit retention/audit records
-> close resources
```

Validation is read-only and never repairs missing evidence. Aggregate metrics cannot override candidate, graph, leakage, security, determinism, deactivation, authorization, holdout, license/privacy, coverage, conflict, or `NotEvaluated` blockers.

Any change to candidate pack, corpus, split, labels, implementation, thresholds, graph registry, review, holdout generation, or target profile creates a new submission. E5-C must independently revalidate the exact submission and owns publication, signing, canary, activation, rollout, rollback, and last-known-good.