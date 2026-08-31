# E3-B data model

**Status:** normative semantic model; concrete Rust names may differ, identities and ownership may not.

## Universe binding

```text
ContextUniverseSet
    universe_set_id
    primary project ProjectId/ProjectGenerationId/ProjectSnapshotId
    primary project GraphGenerationId/GraphSnapshotId
    primary project source/context-input view ID
    optional Blizzard UI ProjectId/ProjectGenerationId/ProjectSnapshotId
    optional Blizzard UI GraphGenerationId/GraphSnapshotId
    optional Blizzard UI SkeletonInputView profile/view ID
    ReferenceProfileId/ReferenceGenerationId/ReferenceViewId
    exact compatibility report ID
    capability/coverage/conflict summaries
    canonical digest
```

No current pointer or mutable view handle is part of canonical identity.

`ContextInputSnapshot` in inherited documents is a compatibility alias for this exact object. It is not a second model.

## Context profiles

```text
ContextProfileSet
    map profile ID
    L0 profile ID
    L1 profile ID
    control/effect profile ID
    intent profile ID
    expansion profile ID
    selection/pruning profile ID
    continuation/stopping profile ID
    budget profile ID
    tokenizer/estimator profile ID
    source excerpt profile ID
    privacy/consumer trust profile ID
    source boundary profile ID
    semantic-pack schema/profile ID
    renderer profile ID(s)
    metrics/evaluation profile IDs
    canonicalization profile ID
    canonical digest
```

Profiles are immutable repository-owned reviewed inputs.

## Project Map

```text
ProjectMap
    project_map_id
    exact universe/project/graph/reference binding
    map profile ID
    root records
    MapNode[]
    MapEdge[]
    MapFacet[]
    MapGroup[]
    evidence/source/reference indexes
    coverage/conflict summaries
    omission records
    budget report
    deterministic order/version
    canonical digest
```

```text
MapNode
    map_node_id
    exact underlying entity/project/package/file/load-unit keys
    node kind
    canonical label facets
    source/load/role facets
    confidence/provenance/coverage/conflict refs
    child/group summary refs
    origin item IDs
```

```text
MapEdge
    map_edge_id
    source/target map node IDs
    exact graph relation/assertion/path refs
    relation kind/axis/direction
    confidence/provenance/coverage/conflict refs
```

`MapEdge` never invents a graph relation.

## Skeletons

```text
L0Skeleton
    l0_skeleton_id
    exact universe/project/graph/reference binding
    scope kind/key
    map node refs
    identity/source/package/load/role facets
    bounded top-level declaration/export/registration/state/API summaries
    direct owner/load/dependency relationships
    evidence/source/reference indexes
    coverage/conflicts/omissions/budget
    canonical digest
```

```text
L1Skeleton
    l1_skeleton_id
    exact universe/project/graph/reference binding
    exact root entity key(s)
    signature/type/declaration/source-span facets
    direct local relations by axis
    bounded reason paths
    direct API/reference facts
    registration/event/hook/state/call/object/inheritance facets
    closed control/effect node refs
    source excerpt candidate refs
    evidence/source/reference indexes
    coverage/conflicts/omissions/budget
    canonical digest
```

Skeletons contain no source body unless a separate `SourceExcerptItem` is selected.

## Context request

```text
ContextRequest
    context_request_id
    exact ContextUniverseSetId
    exact root selectors
    intent profile ID
    requested axes/relation sets/facets
    required fields/capabilities
    allowed confidence/provenance classes
    source/reference/excerpt/privacy policy
    budget/tokenizer profile
    output/render profiles
    continuation: optional
    cancellation
    canonical digest
```

Roots are exact IDs. Opaque user text can be retained as nonsemantic audit metadata only and does not drive canonical selection.

## Candidate and selected items

```text
ContextCandidateItem
    candidate_item_id
    semantic item kind
    exact origin refs
    dependencies
    selection tier and stable tie key
    byte/token cost records
    privacy/license/boundary class
    confidence/coverage/conflict state
```

```text
ContextItem
    context_item_id
    kind:
        Identity
        BoundaryNotice
        MapReference
        SkeletonReference
        ControlEffectReference
        Fact
        Relation
        ReasonPath
        ReferenceFact
        SourceHandle
        SourceExcerpt
        ExistingFindingEvidence
        Coverage
        Conflict
        Omission
        Budget
        SelectionTrace
    typed payload
    origin/evidence/derivation closure
    canonical digest
```

## Source excerpt

```text
SourceExcerptItem
    context_item_id
    exact source handle/content digest
    source universe/project/generation
    exact original byte range and coordinate range
    encoding/source-map profile
    original bytes/text digest
    returned bytes/text
    transformation/redaction records
    privacy/license/consumer trust decision
    boundary rendering class
    truncation/continuation
    evidence/coverage
    canonical digest
```

## Expansion plan and trace

```text
ContextExpansionPlan
    plan_id
    request/universe/profile IDs
    ordered expansion stages
    exact graph/project/reference operations
    per-stage budgets and stop conditions
    mandatory item requirements
    candidate dependency graph
    canonical digest
```

```text
ContextFrontier
    frontier_id
    plan/request/universe IDs
    ordered pending work items
    visited semantic/evidence/source/reference manifests
    used and remaining total budget state
    cycle state
    stop/continuation state
    canonical digest
```

```text
SelectionTrace
    selection_trace_id
    every candidate considered
    selected/omitted/deferred decision
    decision profile/rule/tier/tie key
    cost before/after
    dependency and dedup decisions
    stop state
    canonical digest
```

## Omission

```text
ContextOmissionRecord
    omission_id
    exact candidate/scope/origin refs
    reason:
        ProfileExcluded
        ConfidenceExcluded
        PrivacyDenied
        LicenseDenied
        UnsupportedCapability
        InputPartial
        ConflictBlocked
        DuplicateCovered
        BudgetPruned
        RendererLimit
        DeferredUniverse
        CancelledBeforeSelection
    affected capability/facet
    whether request completeness is affected
    canonical digest
```

## Coverage and loss

```text
ContextCoverageRecord
    coverage_id
    context capability/partition/scope
    upstream coverage refs
    context enumeration/selection/render state
    blockers/conflicts/truncation/omission refs
    status
    canonical digest
```

```text
ContextProjectionLossRecord
    loss_id
    exact source semantic field/item
    target map/skeleton/pack/render field
    loss kind and reason
    affected authority/consumer capability
    recoverability/detail route
    canonical digest
```

## Budget and token accounting

```text
ContextBudgetReport
    budget_report_id
    budget/tokenizer/renderer profiles
    hard and soft limits
    mandatory/optional allocations
    candidate/selected/omitted costs
    canonical semantic bytes
    rendered bytes per artifact
    token count class: Exact | DeterministicEstimate | UpperBound | Unavailable
    tokenizer/estimator identity and digest
    exact/estimated/upper-bound token values
    overflow/truncation/continuation state
    canonical digest
```

## Semantic pack

```text
ContextSemanticPack
    context_semantic_pack_id
    universe set/request/profile/plan IDs
    ProjectMap refs
    L0/L1/control-effect skeleton refs
    ordered ContextItem[]
    evidence/source/reference indexes
    selection trace and omission manifest
    coverage/conflict/NotEvaluated/loss summaries
    budget report
    source-boundary/privacy/license notices
    continuation and no-new-evidence state
    validation report ID: external later-DAG reference only
    canonical digest
```

`ContextBundleCore` in inherited documents is the compatibility alias for `ContextSemanticPack`. No implementation may expose them as two separate canonical semantic artifacts.

The semantic pack ID excludes renderer, metrics, evaluation, delivery envelope, cache location, timing, and validation report IDs to prevent cycles. A validation report refers to the pack, not vice versa in the hashed core. Implementations can carry a noncanonical outer validation reference.

## Rendered artifact

```text
RenderedContextArtifact
    rendered_artifact_id
    exact ContextSemanticPackId
    renderer/profile/schema IDs
    media type/encoding/line-ending policy
    rendered bytes/object digest
    exact byte/token report
    rendering loss/transformation records
    source boundary verification report
    canonical digest
```

## Metrics and evaluation

```text
ContextMetrics
    metrics_id
    exact semantic pack and rendered artifact refs
    structural/coverage/source/budget/token measures
    noncanonical operational measures separated
    canonical digest
```

```text
ContextEvaluationReport
    evaluation_report_id
    exact corpus/profile/pack/render refs
    hard gate results
    utility/recall/precision/compression measures
    missing-evidence and consumer findings
    canonical digest
```

Neither ID enters `ContextSemanticPackId`.

## Cache key

```text
ContextCacheKey
    context schema/version
    exact universe set
    normalized request
    all semantic profiles
    budget/tokenizer/privacy/boundary profiles
    semantic or renderer target
    implementation contract/profile IDs
    canonical digest
```

Physical cache location, timestamps, hit counters, and eviction state are noncanonical and outside this crate.

## Coverage axes

Keep separate:

- input universe compatibility;
- project/source/graph/reference input coverage;
- map projection coverage;
- L0/L1/control-effect projection coverage;
- expansion/query coverage;
- selection/budget coverage;
- source excerpt/privacy/license coverage;
- semantic-pack validation coverage;
- renderer coverage;
- token accounting coverage;
- evaluation coverage.
