# E4-C service data model

**Status:** normative transport-independent model.

## Configuration

```text
E4ServiceConfiguration
    selector/profile alias registry
    acquisition/stability/retry profile
    owner port catalog IDs
    search build/query profile registry
    lineage/review/migration/impact profile registry
    context handoff profile registry
    result/envelope/canonicalization/error profiles
    privacy/license/consumer trust profile
    budget/cancellation/retention/idempotency profile
    canonical digest
```

## Selectors

```text
ProjectPublicationSelector
    ExactStoreGeneration
    ExactPublicationSet
    CurrentPublished + optional expected-current guard

ReferenceSelector
    ExactReferenceGeneration/View
    CurrentCompatible + explicit product/flavor/build/profile guard

SearchShardSelector
    ExactSearchShard
    ForExactOwnerGeneration + exact SearchProfileSet

LineageSnapshotSelector
    ExactLineageGraphSnapshot
    ForExactComparison + exact LineageProfileSet
```

`ForExactOwnerGeneration` and `ForExactComparison` are deterministic catalog lookups, not latest selectors. Zero or multiple eligible artifacts is explicit unavailable/conflict.

## Acquired views

```text
ResolvedSearchAcquisition
    exact project/platform/reference publication selections
    exact owner views and generation leases
    exact validated search shard views
    SearchUniverseSetId
    compatibility/stability report
    acquisition attempt manifest
```

```text
ResolvedLineageAcquisition
    exact before/after project or platform or Reference owner views
    exact before/after graph/source/reference views
    optional exact E4-A shard views for Candidate production
    exact lineage profile and optional retained LineageGraphView
    compatibility/stability report
    leases/retention receipts
```

## Search service requests

```text
SearchIndexBuildServiceRequest
    exact owner selector(s)
    SearchProfileSetId
    build/idempotency/output/retention profiles
    operation ID
    budgets/cancellation
```

```text
SearchQueryServiceRequest
    owner/shard selectors
    exact or current acquisition policy
    owner SearchRequest payload
    required lane/capability policy
    privacy/consumer policy
    budgets/cancellation
```

```text
SearchSelectionRequest
    exact SearchResultId/ResultSetManifestId
    exact SearchCandidateId
    expected entity/universe/shard/result digest guards
    selection origin class and optional attestation ref
    target action: ReceiptOnly | Context
    context request/profile when target is Context
```

## Search selection receipt

```text
SearchSelectionReceipt
    receipt ID
    exact result/result-set/candidate/entity IDs
    exact search universes/shards/query/ranking profiles
    rank/band/signal/explanation refs as provenance
    selection origin and authorization policy result
    target exact root token
    explicit nonauthority declarations
    canonical digest
```

## Lineage build request

```text
LineageBuildServiceRequest
    exact before/after universe selectors
    project/reference/search producer profile IDs
    E4-B relation/proof/component/review/change/migration/impact profiles
    optional validated review envelopes
    exact expected lineage catalog/base guard
    operation/idempotency/retention/output profiles
    budgets/cancellation
```

## Review envelope

```text
LineageReviewDecisionEnvelope
    envelope schema/version
    exact comparison/component/proposal/relation/profile IDs
    decision: Accept | Reject | Defer | MarkConflict | Supersede
    requested confidence/proof class
    structured reason codes
    bounded untrusted note: optional
    reviewer principal/role/scope refs
    attestation/signature/key/verification profile refs
    issuance/expiry/revocation state where supported
    canonical decision digest
```

Authorization metadata is operational/security evidence; it is not lineage proof. Graph proof ceiling remains independently enforced.

## Lineage query requests

```text
LineageCompareServiceRequest
LineageTraceServiceRequest
LineageExplainServiceRequest
LineageValidateServiceRequest
```

Each binds exact lineage snapshot/comparison/entity/assertion roots, filters, confidence/coverage policy, budgets and cancellation.

## Migration and impact requests

```text
MigrationCandidatesServiceRequest
    exact lineage snapshot and entity/change roots
    exact Reference/project/search enrichment policy
    migration candidate profile
    budgets/cancellation

MigrationValidateServiceRequest
    exact candidate/recipe artifact and governing snapshot
    validation/privacy/license/security profile
    budgets/cancellation

ImpactServiceRequest
    exact lineage snapshot and change/assertion roots
    exact target graph snapshots/universes
    impact relation/direction/confidence profile
    depth/fanout/node/edge/path/output budgets
    continuation/cancellation
```

## Invocation plan

```text
E4ServiceInvocationPlan
    normalized request ID
    exact acquisition plan and order
    owner operations and inputs
    required retention/idempotency steps
    context handoff step when present
    status/validation/output plan
    cancellation and cleanup plan
    canonical digest
```

## Outcomes

```text
SearchServiceOutcome
    owner SearchShard/SearchResult/Explanation/SelectionReceipt IDs
    owner status/miss/coverage/conflict/omission/budget records

LineageServiceOutcome
    producer partitions/components/reviews/LineageGraphSnapshot/query records
    proof ceilings/ambiguity/conflicts/coverage

MigrationServiceOutcome
    candidate/recipe/validation records and nonclaims

ImpactServiceOutcome
    plan/direct effects/reason paths/coverage/conflicts/continuation/nonclaims

SearchContextOutcome
    SearchSelectionReceipt
    exact E3-C context service result
    no search-rank-to-context-authority conversion
```

## Lifecycle records

```text
E4ResourceLeaseSet
E4RetentionReceiptSet
E4ResourceClosureReport
E4IdempotencyReceipt
E4ResponseLossRecoveryReceipt
E4ServiceContinuation
```

## Canonical result envelope

```text
E4ServiceResultEnvelope
    envelope schema/version
    operation/request/result IDs
    exact resolved selectors and generation bindings
    owner outcome refs or structured failure
    conservative service status
    validation state
    evidence/provenance/confidence/proof/coverage/conflicts/ambiguity/omissions
    budgets/continuation/retention/idempotency/closure summaries
    privacy/license/authorization summaries
    explicit nonclaims
    canonical digest
```

Operational timings, process IDs, terminal state, host paths, raw signatures/tokens and lease handles do not enter semantic identity.
