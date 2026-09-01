# E4-B lineage data model

**Status:** normative semantic model. Concrete Rust names may vary only with same-change contract and fixture updates.

## Comparison universe

```text
LineageUniverseSet
    lineage_universe_set_id
    universe class
    exact before generation binding
    exact after generation binding
    exact before/after GraphSnapshot/View IDs
    exact project/reference/source owner-view IDs
    optional exact before/after SearchShard/View IDs
    comparison/profile compatibility report
    input capability/coverage/conflict manifests
    canonical digest
```

No mutable current pointer is part of semantic identity.

```text
GenerationBinding
    universe class and universe ID
    owner store/publication/generation/snapshot/view IDs
    graph generation/snapshot/view IDs
    source/reference/analyzer/profile IDs as applicable
    exact read catalog and capability manifest IDs
    canonical digest
```

## Profiles

```text
LineageProfileSet
    relation registry ID
    producer schema registry ID
    blocking/candidate-generation profile ID
    fingerprint/feature profile IDs
    proof-ceiling profile ID
    ambiguity/component profile ID
    review/promotion profile ID
    change-classification profile ID
    migration/replacement profile ID
    impact profile ID
    store/publication/query profile IDs
    coverage/negative-authority profile ID
    privacy/license/security/budget profile IDs
    canonicalization/error schema IDs
    canonical digest
```

## Generation entity reference

```text
GenerationEntityRef
    exact universe/generation
    exact entity key/ID
    entity kind
    owner/source/reference/graph handles
    identity/fingerprint/shape manifest refs
    evidence/provenance/confidence/coverage/conflict refs
    canonical digest
```

This is a reference to an existing generation-local entity, not a copied or merged replacement entity.

## Producer input partition

```text
LineageInputPartition
    input_partition_id
    producer class/ID/version
    exact lineage universe set/profile
    before/after owner generation scope
    ordered input records
    evidence/source/coverage/conflict manifests
    completeness/truncation/cancellation state
    canonical digest
```

## Proposal

```text
LineageProposal
    proposal_id
    relation kind
    exact source GenerationEntityRef
    exact target GenerationEntityRef or generation boundary
    producer class/ID/version and partition ID
    proposed confidence
    maximum proof ceiling
    typed feature/evidence records
    derivation rule/profile ID
    ambiguity component ID
    source/reference/search/graph evidence IDs
    coverage/conflict/blocker records
    review requirement
    canonical digest
```

A proposal never changes source/target entity IDs.

## Candidate component

```text
LineageCandidateComponent
    component_id
    exact comparison scope and blocking key(s)
    ordered before entity refs
    ordered after entity refs
    ordered proposal IDs
    component shape:
        OneToOne
        OneToMany
        ManyToOne
        ManyToMany
        BeforeOnly
        AfterOnly
    ambiguity/conflict/coverage state
    candidate-generation budget/truncation state
    canonical digest
```

Component shape alone does not establish split, merge, removal or introduction.

## Review decision

```text
LineageReviewDecision
    review_decision_id
    exact proposal/component/assertion target
    reviewer authority/profile class
    decision:
        Accept
        Reject
        Defer
        Conflict
        Supersede
    requested and effective proof ceiling
    structured reason code
    bounded untrusted note: optional
    evidence additions/requirements
    coverage/conflict conditions
    predecessor/superseded decision refs
    canonical digest
```

Reviewer identity and note handling follow privacy policy. Free prose does not alter schemas or proof rules.

## Accepted assertion

```text
LineageAssertion
    lineage_assertion_id
    relation kind/version
    exact source/target GenerationEntityRefs
    accepted proposal IDs
    effective confidence/proof ceiling
    exact supporting evidence and derivation IDs
    review decision IDs when required
    coverage/conflict state
    validity/comparison profile
    producer partition/publication generation
    canonical digest
```

## Conflict

```text
LineageConflictRecord
    conflict_id
    exact universe/comparison/component/entity/assertion scope
    competing proposal/assertion/review IDs
    conflict kind
    affected lineage/change/removal/migration/impact capabilities
    status/resolution refs
    canonical digest
```

## Change set

```text
GenerationChangeSet
    change_set_id
    exact lineage universe and before/after bindings
    accepted lineage assertion manifest
    ordered ChangeRecord IDs
    unmatched before/after entity manifests
    removal/introduction authority records
    coverage/conflict/NotEvaluated summaries
    canonical digest
```

```text
ChangeRecord
    change_record_id
    exact source/target entity refs or generation boundary
    governing lineage assertion IDs
    change kind:
        UnchangedIdentity
        Moved
        Renamed
        Split
        Merged
        CopiedOrExtractedCandidate
        SignatureChanged
        TypeChanged
        RestrictionChanged
        OwnershipChanged
        LoadRoleChanged
        RelationSetChanged
        Deprecated
        Replaced
        Removed
        Introduced
        UnmatchedBefore
        UnmatchedAfter
        Conflict
        NotEvaluated
    exact before/after typed values or relation manifests
    evidence/derivation/proof/coverage/conflict refs
    canonical digest
```

## Field/relation difference

```text
TypedChangeFacet
    facet ID and schema
    before state/value/origin
    after state/value/origin
    comparison result:
        Equal
        Added
        Removed
        Changed
        Unknown
        Unsupported
        Conflict
        NotEvaluated
    semantic compatibility class
    evidence/coverage/conflict refs
```

`Missing`, `ExplicitNull`, `Unknown`, `Unsupported`, `Omitted` and `Conflict` remain distinct.

## Removal/introduction authority

```text
GenerationAbsenceDecision
    decision_id
    subject entity/generation boundary
    kind: RemovedAfter | IntroducedIn | UnmatchedOnly | NotEvaluated
    exact closed scope definition
    before/after owner and lineage coverage records
    unresolved candidate-component/conflict/truncation blockers
    negative-authority proof records
    canonical digest
```

## Migration

```text
MigrationCandidate
    migration_candidate_id
    exact source/target entity refs
    lineage/replacement/search/reference/change evidence refs
    applicability candidate profile
    confidence/proof ceiling
    missing evidence/preconditions
    coverage/conflicts
    canonical digest
```

```text
MigrationRecipe
    migration_recipe_id/version
    exact source/target contract scopes
    governing replacement/deprecation/lineage assertions
    applicability preconditions
    typed transformation steps
    forbidden transformations
    semantic constraints and postconditions
    required static/client/runtime validation steps
    remediation tier: plan_only | validated_recipe
    evidence/provenance/confidence/coverage/conflicts
    canonical digest
```

No executable code or edit operation is stored in E4-B.

## Static impact

```text
StaticImpactRequest
    request_id
    exact lineage graph/change set/target GraphSnapshot IDs
    exact change roots
    relation/axis/direction/confidence policy
    impact category profile
    depth/fanout/node/edge/path/output budgets
    cancellation/continuation
    canonical digest
```

```text
StaticImpactPath
    impact_path_id
    exact root ChangeRecord/LineageAssertion
    exact target generation entity
    ordered direct relation assertion IDs
    path categories
    confidence/proof cap
    evidence/coverage/conflict/truncation state
    canonical digest
```

```text
StaticImpactResult
    result_id
    request/universe/lineage/change/graph/profile IDs
    direct affected entities
    bounded transitive affected entities and paths
    possible/conflict-blocked/NotEvaluated partitions
    coverage/omissions/budgets/continuation
    validation report ID
    canonical digest
```

## Lineage graph snapshot

```text
LineageGraphSnapshot
    lineage_graph_snapshot_id
    lineage graph generation ID
    exact LineageUniverseSet and profile set IDs
    relation registry ID
    complete input/proposal/review/assertion/conflict/change/migration partition membership
    logical store generation ID
    capability/coverage/conflict manifests
    validation/golden-query reports
    canonical digest
```

## Queries and continuations

```text
LineageQueryContext
    exact lineage graph snapshot
    exact before/after generations
    relation/change/proof/confidence filters
    budgets/cancellation/continuation
```

```text
LineageContinuation
    exact snapshot/universe/request/profile/result manifest IDs
    stable ordering/frontier state
    cumulative budgets
    prior conflicts/omissions/truncation
    integrity digest
```

## Canonical ordering

```text
universe class and exact generation bindings
entity kind and canonical generation entity key
producer class/ID/version/partition
relation registry order
source and target generation entity keys
proposal/assertion/review/conflict IDs
change kind/facet key
impact root/path/target stable keys
```

No insertion, SQL row, hash-map, worker, timestamp, repository popularity or reviewer chronology controls semantic order.
