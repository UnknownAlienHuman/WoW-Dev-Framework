# E3-A context data model and identity DAG

**Status:** normative semantic Project Map, skeleton, expansion, source, bundle, renderer, metric, evaluation, and continuation contract.

## Identity DAG

```text
exact domain input views + frozen profiles + normalized request
-> ContextInputSnapshotId / ContextRequestId
-> ContextPlanId / frontier
-> ProjectMapId, SkeletonId, SourceExcerptId, expansion/loss/evidence records
-> ContextBundleCoreId
-> ContextRendererArtifactId
-> ContextMetricsId
-> ContextEvaluationReportId
-> ContextBundleEnvelopeId
```

No earlier artifact contains a later artifact ID. In particular, semantic bundle identity excludes renderer bytes, token counts, metrics, evaluation scores, timings, and envelope identity.

## Context input snapshot

```text
ContextInputSnapshot
    snapshot ID
    ProjectStoreEpochId
    ProjectStoreGenerationId
    ProjectPublicationSetId
    ProjectGenerationId / ProjectSnapshotId / ProjectViewId
    AnalyzerSnapshotId
    GraphGenerationId / GraphSnapshotId / GraphViewId
    ProfileId
    ReferenceGenerationId / ReferenceViewId: optional exact set
    SourceUniverseManifestId[]
    project/graph/reference query catalog IDs
    capability/coverage/conflict manifest IDs
    canonical digest
```

`StoreImageId` is forbidden. Operational read transaction/lease IDs are held by the caller/view owner and excluded from semantic identity.

## Profiles

### `ContextProfile`

```text
profile ID/version
active artifact/detail kinds
input capability requirements
relation lane registry
confidence/provenance/coverage policies
mandatory evidence/blocker fields
canonicalization and compatible profile matrix
canonical digest
```

### `ProjectMapProfile`

```text
section registry/order
principal-root and grouping rules
entry-point/direct-neighborhood rules
mandatory and optional fields
strict default renderer target/cap
next-detail route rules
canonical digest
```

### `SkeletonProfile`

```text
supported subject kinds and L0/L1 fields
signature/member/control-effect registries
heading/role vocabulary
projection/loss policy
canonical digest
```

### `DetailExpansionProfile`

```text
allowed lanes/directions/relation kinds
confidence and coverage policy
depth/path/cycle/frontier rules
priority/fairness/stopping/continuation policy
canonical digest
```

### Other profiles

```text
SourceExcerptProfile
ContextBudgetProfile
TokenizerProfile: optional
ContextSecurityProfile
ContextRendererProfile
ContextEvaluationProfile
```

Each is versioned, bounded, immutable for a request, and content-addressed.

## Context request

```text
ContextRequest
    request ID
    exact ContextInputSnapshot
    root EntityKey[]
    artifact target = ProjectMap | L0 | L1 | Bundle
    selected lanes/directions/detail/source/evidence policies
    confidence/coverage/conflict policy
    all profile IDs
    optional tokenizer/renderer IDs
    requested overrides within profile maxima
    continuation cursor: optional
    cancellation
    canonical digest
```

Exact roots only. Search/ranking is not a request field in E3-A.

## Context plan

```text
ContextPlan
    plan ID
    normalized request ID/digest
    input/profile IDs
    resolved roots and statuses
    requested stages and lane query plans
    mandatory inclusion set
    budget reservations/priority rounds
    source/evidence query plan
    stopping/continuation policy
    expected capability dependencies
    canonical digest
```

## Project Map

```text
ProjectMap
    ProjectMapId
    exact input/request/context/project-map profile IDs
    project/publication/profile header
    ordered ProjectMapSection IDs
    evidence/coverage/conflict/loss/omission/stopping IDs
    next-detail route IDs
    canonical semantic digest
```

It does not contain metrics or evaluation report IDs.

```text
ProjectMapSection
    section ID/kind
    exact subject/group IDs
    frozen heading/presentation role
    ordered item/relation/route IDs
    evidence/coverage/conflict refs
    projection/loss/omission/truncation state
    canonical digest
```

Initial section kinds:

```text
project_identity
packages_and_toc_variants
load_units_and_entrypoints
files_and_source_units
ownership_and_universal_roles
lifecycle_and_registration
signals_callbacks_and_hooks
state_roots_and_paths
platform_api_usage
capabilities_conflicts_and_gaps
next_detail_routes
```

## Skeleton

```text
SkeletonRecord
    SkeletonId
    level = L0 | L1
    exact subject EntityKey/kind/universe/generation
    logical/display names from owning facts
    owner/package/file/load/role refs
    signature/member IDs
    direct relation/path IDs
    control/effect node IDs
    evidence/source/coverage/conflict/ambiguity refs
    projection/loss/omission/stopping refs
    next-detail route IDs
    canonical semantic digest
```

```text
SkeletonMember
    member ID/kind/semantic ordinal
    exact owner/subject/source refs
    proven name/type/signature/value metadata
    direct relation/evidence/coverage/conflict refs
    projection status
    canonical digest
```

```text
ControlEffectNode
    node ID/kind
    subject/file/source-unit/source handle/span
    semantic ordinal and parent/child IDs
    exact input fact IDs and typed attributes
    relation/evidence/coverage/conflict refs
    projection and unknown/collapsed/omitted refs
    canonical digest
```

## Detail route

```text
ContextDetailRoute
    route ID
    source artifact/subject
    target entity/root IDs
    lane/direction/detail target
    reason/evidence/path IDs
    required capabilities
    labeled estimated structural/byte/token cost profile
    priority class/stable key
    canonical digest
```

An estimate cannot create eligibility or authority.

## Frontier and expansion

```text
ContextFrontier
    frontier ID
    exact input/request/profile IDs
    ordered pending work items
    included/visited set digests
    used/reserved/remaining budgets
    stopping/blocker/no-new-evidence refs
    canonical digest
```

```text
ContextExpansionStep
    step ID/ordinal
    input/output frontier IDs
    exact query request/result IDs
    root/lane/reason
    new/duplicate/rejected/blocked record IDs
    budget delta
    coverage/conflict/truncation state
    canonical digest
```

## Source excerpt

```text
ContextSourceExcerpt
    excerpt ID
    exact source handle/file/content digest/generation
    requested and actual half-open byte spans
    faithful source bytes/text under explicit profile
    line/column projection
    prefix/suffix truncation markers
    license/provenance/security/injection-label refs
    excerpt digest
```

Source excerpt identity is independent of Markdown fencing/wrapping; renderer transformation is recorded later.

## Evidence and projection records

```text
ContextEvidenceLink
    context artifact/record/field ID
    project/graph/reference/evidence/source IDs
    exact generations/universe
    derivation rule/path IDs
    confidence/coverage/conflict refs
    canonical digest
```

```text
ContextCoverageRecord
    artifact/request/profile and field/section/lane/detail partition
    input coverage/conflict refs
    exact considered/included/omitted/unsupported/truncated counts/digests
    Complete | Partial | Unknown | Failed | NotApplicable | NotEvaluated
    loss/omission/stopping refs
    canonical digest
```

```text
ContextLossRecord
    subject/artifact/field
    category/severity/reason
    emitted compact/sidecar/omitted representation
    exact lost semantics/detail
    affected tasks/capabilities
    evidence/coverage/conflict and route refs
    canonical digest
```

```text
ContextOmissionRecord
    exact IDs or partition/count/digest
    mandatory/optional and priority
    reason/budget/blocker
    continuation/detail route
    canonical digest
```

```text
ContextStoppingRecord
    RequestedComplete | NoNewEvidence | BudgetExhausted | DepthLimit |
    CycleClosed | CoverageBoundary | ConflictBoundary | UnsupportedDetail |
    Cancelled | Failed
    exact scope/frontier/query/budget/coverage refs
    continuation availability
    canonical digest
```

## Projection status

```text
Exact
ExactWithEvidenceSidecar
CompactButCompleteForDeclaredFields
LossyDeclared
Unsupported
NotEvaluated
Truncated
```

## Context bundle core

```text
ContextBundleCore
    ContextBundleCoreId/version
    exact input/request/profile IDs
    optional ProjectMapId
    ordered L0/L1 skeleton/member/control-effect IDs
    relation/path/evidence/source-excerpt IDs
    coverage/loss/omission/stopping IDs
    final frontier/continuation ID: optional
    semantic status/eligibility
    canonical semantic digest
```

No renderer, token, metric, evaluation, timing, or operational fields.

## Continuation

```text
ContextContinuation
    continuation ID/version
    exact input/request/all relevant profile IDs
    ordering/continuation version
    frontier ID/digest
    included/visited set digests
    used/reserved/remaining total-request budget state
    last stable work-item key
    creating stop records
    integrity digest
```

## Renderer artifact

```text
ContextRendererArtifact
    RendererArtifactId
    ContextBundleCoreId
    renderer profile ID/version
    output bytes digest/length/line/scalar counts
    renderer coverage/loss/security refs
    optional exact TokenizerResultId
    canonical manifest digest
```

## Tokenizer result

```text
ExactTokenCount
    tokenizer profile ID
    exact RendererArtifactId/output bytes digest
    special/template policy
    count and optional token-ID digest
    canonical digest
```

```text
TokenEstimate
    estimate profile
    exact byte/scalar/word subject measures
    estimate range/point and uncertainty
    explicitly_exact = false
```

## Metrics

```text
ContextMetrics
    ContextMetricsId
    ContextBundleCoreId and optional RendererArtifactId
    input source/project/graph/reference measures
    output record/evidence/source measures
    bytes/scalars/lines/structured nodes
    exact token result or separate estimate refs
    duplicate avoided and mandatory recall counts
    omitted/truncated/partial/conflict counts
    expansion/query cost summaries
    deterministic metric digest
    supplemental scoped timing/memory outside canonical digest
```

## Evaluation report

```text
ContextEvaluationReport
    EvaluationReportId
    exact corpus/input/request/profile IDs
    ContextBundleCoreId / renderer / metric refs
    mandatory expected versus included records
    relevance/redundancy/compression/budget/continuation/source results
    consumer task outcomes under pinned protocol
    hard-gate decision and loss/error refs
    canonical digest for deterministic judgments
```

## Outer envelope

```text
ContextBundleEnvelope
    EnvelopeId
    ContextBundleCoreId
    renderer artifact IDs
    metric IDs
    optional evaluation report IDs
    transport-safe status/manifest
    canonical envelope digest
```

The envelope is convenient delivery composition, not semantic bundle identity or authority.

## Artifact eligibility

```text
Fixture
Candidate
ValidatedForDeclaredContextProfile
```

Validation is scoped to the declared context profile and input capabilities; it never means complete source/project/platform truth.

## Canonical ordering

```text
sections by profile order
roots by universe/kind/semantic key/ID
skeletons by level/kind/owner/load/source/key/ID
members/control nodes by semantic/source ordinal
relations by lane/kind/direction/source/target/qualifier/assertion
paths by length then semantic tuple
source excerpts by source unit/span/ID
evidence/loss/omission/stopping by artifact field/scope/category/reason/ID
frontier by priority/root/lane/path/detail/stable key
```

No row, page, hash map, filesystem, worker, query completion, wall-clock, or model-score ordering.
