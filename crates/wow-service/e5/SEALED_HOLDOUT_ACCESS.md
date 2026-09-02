# E5-B sealed-holdout access, audit, and consumption

**Status:** normative.

A holdout loses evidentiary value when candidate authors can inspect membership, labels, source, facts, mutations, or detailed results before candidate/run identities freeze.

Before access, freeze:

```text
sealed holdout generation and digest
candidate pack bytes and candidate artifact
matcher/graph/evaluator implementations and profiles
run request and budgets
evaluator service identity
disclosure, retention, contamination, and audit profiles
```

`HoldoutAuthorizationPort` validates principal/service identity, candidate/run/holdout scope, permitted operation/disclosure class, purpose, role, expiry, revocation, replay/use count, independence, and retention obligations. Reviewer authorization is not holdout authorization.

`HoldoutVaultPort` exposes only exact operations:

```text
open_exact_generation
execute_exact_evaluator_request
obtain_bounded_result_receipt
close_access
reconcile_effect
```

Raw credentials never enter service requests/results.

Disclosure classes are:

```text
AggregateGateOnly
PerCaseClassificationWithoutHiddenInputs
BoundedReviewedEvidence
FullReviewedDisclosure
```

Use the narrowest authorized class. Every request, grant, denial, open, execution, disclosure, failure, cancellation, revocation, replay, and consumption event is append-only and hash-linked.

Consumption states are:

```text
UntouchedForCandidateLineage
ExecutedNoAuthorDisclosure
DisclosedAndConsumed
ConsumedByAdaptiveUse
ContaminationUnknown
Invalidated
```

When disclosed results can influence a descendant candidate, the generation is consumed for that lineage. `ContaminationUnknown` cannot be reported untouched.

If vault execution may have occurred but the response is lost, enter `OutcomeUnknown`; do not repeat access/evaluation until exact reconciliation. Inability to prove nonaccess prevents an untouched claim. Public success is impossible before vault closure, audit persistence, and retention closure.