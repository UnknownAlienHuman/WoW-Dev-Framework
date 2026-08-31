# E3-A context data model

**Status:** normative semantic Project Map, skeleton, expansion, bundle, loss, metric, and continuation contract.

## 1. Object graph

```text
ContextRequest
├── exact ContextInputSnapshot
├── ContextProfile
├── ProjectMapProfile
├── SkeletonProfile
├── DetailExpansionProfile
├── SourceExcerptProfile
├── ContextBudgetProfile
├── TokenizerProfile: optional
├── ContextSecurityProfile
├── ContextEvaluationProfile
└── cancellation

-> ContextPlan
-> ProjectMap and/or SkeletonRecord[]
-> ContextExpansionStep[]
-> SourceExcerptRecord[]
-> ContextCoverage/Loss/Omission records
-> ContextBundle
-> ContextMetrics
-> ContextContinuation
```

## 2. Exact input snapshot

```text
ContextInputSnapshot
    PublicationSetId
    StoreGenerationId / StoreImageId
    ProjectGenerationId / ProjectSnapshotId / ProjectViewId
    GraphGenerationId / GraphSnapshotId / GraphViewId
    ProfileId / ReferenceGenerationId / ReferenceViewId: optional exact set
    project/graph/reference query bundle IDs
    capability/coverage/conflict manifest IDs
    canonical digest
```

All present identities must cohere. No floating current/latest token remains after acquisition.

## 3. Context profiles

### `ContextProfile`

```text
profile ID/version
active artifact/detail kinds
input capability requirements
relation lane registry
confidence/provenance/coverage policies
mandatory evidence/blocker fields
ordering and canonicalization profile
compatible renderer/evaluation profiles
canonical digest
```

### `ProjectMapProfile`

```text
included root/entity/load/role/capability classes
principal-root selection rules
entry-point and direct-neighborhood rules
max summary cardinalities and deterministic grouping
next-detail route rules
mandatory blocker/evidence display fields
canonical digest
```

### `SkeletonProfile`

```text
supported entity kinds and L0/L1 fields
signature/member/relation/source-node projection rules
role/heading vocabulary registry
source-excerpt exclusion/default policy
loss/unsupported policy
canonical digest
```

### `DetailExpansionProfile`

```text
allowed lanes/directions
per-lane confidence/coverage policy
path/depth/frontier/cycle rules
inclusion priority and stable ordering
no-new-evidence and stopping rules
continuation profile
canonical digest
```

### `SourceExcerptProfile`

```text
allowed source origins/roles
license/privacy/security requirements
span expansion/context-line policy
encoding/normalization/escaping policy
per-excerpt/entity/bundle budgets
prompt-injection labeling
canonical digest
```

### `ContextBudgetProfile`

```text
profile ID/version
max roots/entities/skeletons/relations/paths/depth
max evidence/source handles/conflicts/loss records
max source excerpts/bytes/lines
max structured nodes/fields
max output UTF-8 bytes/Unicode scalars
optional tokenizer budget policy
per-lane/per-root reservations and global limits
canonical digest
```

### `TokenizerProfile`

```text
tokenizer profile ID
implementation/package/revision/version
vocabulary/model file digest
normalization/special-token/config policy
input encoding and exact bytes policy
counting API/probe identity
canonical digest
```

### `ContextSecurityProfile`

```text
allowed origins/object roles
private-path/payload/source rules
source instruction labeling/escaping
object/excerpt limits
consumer output policy
canonical digest
```

### `ContextEvaluationProfile`

```text
mandatory structural/evidence record corpus
relevance task/request classes
redundancy equivalence rules
compression baseline
consumer utility protocol
optional external consumer/model pin
acceptance thresholds and non-goal fields
canonical digest
```

## 4. Context request

```text
ContextRequest
    request ID
    exact ContextInputSnapshot
    root EntityKey/IDs[]
    artifact target: ProjectMap | L0 | L1 | ContextBundle
    selected relation lanes/directions
    selected detail/source/evidence policies
    confidence/coverage/conflict policy
    context/project-map/skeleton/expansion/source/budget/security/evaluation profile IDs
    tokenizer profile ID: optional
    requested budget overrides within profile bounds
    continuation cursor: optional
    output renderer profile: optional higher-layer field
    cancellation
    canonical digest
```

Exact roots must resolve in the selected graph/project/reference universe. Root absence is classified with coverage, not guessed.

## 5. Context plan

```text
ContextPlan
    plan ID
    normalized request ID/digest
    exact input snapshot/profile IDs
    resolved roots and root statuses
    requested artifact stages
    lane-specific query/expansion plans
    mandatory inclusion set
    budget reservations/priorities
    source/evidence query plan
    stopping/continuation policy
    expected capability/coverage dependencies
    canonical digest
```

## 6. Project Map

```text
ProjectMap
    project_map_id/version
    exact input/context/project-map profile IDs
    project/publication/profile header
    package/load-unit/file/XML/Lua-unit summaries
    principal entity/role/entry-point IDs
    direct ownership/load/lifecycle/registration/state/API-use lane summaries
    capability/coverage/conflict/truncation summary IDs
    next-detail route IDs
    evidence/source handle refs
    context projection coverage/loss/omission records
    metrics
    canonical digest
```

## 7. Project Map section

```text
ProjectMapSection
    section ID/kind
    exact subject entity/group IDs
    heading/presentation role from frozen vocabulary
    ordered item/edge/route IDs
    input/evidence/coverage/conflict refs
    projection status
    omission/truncation refs
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

## 8. Skeleton identity

```text
SkeletonId
    domain-separated digest over:
        exact ContextInputSnapshot semantic identities
        subject EntityKey
        detail level
        skeleton/context profile IDs
        canonical semantic content and required blocker/evidence refs
```

Renderer path/line/whitespace and output order do not determine identity.

## 9. Skeleton record

```text
SkeletonRecord
    skeleton ID
    detail level = L0 | L1
    exact subject EntityKey/kind/universe/generation
    stable display/logical name fields
    owner/package/file/load/role refs
    signature/member/public-surface refs
    direct relation summary/edge/path IDs
    source-backed structural node IDs
    evidence/source handle refs
    confidence/provenance/coverage/conflict/ambiguity state
    projection/loss/omission/truncation refs
    next-detail route IDs
    canonical digest
```

## 10. Skeleton member

```text
SkeletonMember
    member ID/kind/semantic ordinal
    owner skeleton/entity
    exact source/graph/project/reference subject refs
    canonical name/type/signature/value metadata when proven
    direct relation refs
    evidence/source/coverage/conflict refs
    projection status
    canonical digest
```

No inferred member/type/body when the input contract lacks one.

## 11. Source-backed structural node

```text
SourceSkeletonNode
    node ID/kind
    exact project file/source unit and source handle/span
    containing semantic entity
    structural role = declaration | signature | field | direct call | registration | guard | branch heading | state access | return heading | other reviewed
    normalized proven attributes and related fact IDs
    children/relations by exact IDs
    faithful excerpt ref: optional
    confidence/coverage/conflict state
    canonical digest
```

This is structured analyzer/project evidence, not reconstructed source text.

## 12. Detail route

```text
ContextDetailRoute
    route ID
    source subject/artifact
    target entity/root IDs
    lane/direction/detail target
    reason/evidence/path IDs
    required capabilities
    estimated structural/byte/token cost profile
    priority class and stable ordering key
    canonical digest
```

Cost estimates are labeled and profile-bound; they cannot silently change semantic priority.

## 13. Expansion frontier

```text
ContextFrontier
    frontier ID
    exact input/request/profile IDs
    pending root/entity/lane/path work items
    visited semantic entity/relation/evidence sets
    included artifact IDs
    used/reserved/remaining budgets
    stopping/blocker/no-new-evidence records
    canonical digest
```

## 14. Expansion step

```text
ContextExpansionStep
    step ID/ordinal
    input frontier ID
    exact query request/result IDs
    lane/root/reason
    newly included entity/relation/evidence/artifact IDs
    duplicate/rejected/skipped IDs and reasons
    budget delta
    coverage/conflict/truncation state
    output frontier ID
    canonical digest
```

## 15. Source excerpt

```text
ContextSourceExcerpt
    excerpt ID
    exact source handle/file/content digest/generation
    exact requested and actual half-open byte span
    line/column projection under explicit profile
    faithful normalized bytes/text
    prefix/suffix truncation markers
    license/provenance/security refs
    injection/untrusted-data label
    excerpt digest
```

Excerpt is not allowed without exact current source-handle validation.

## 16. Evidence link

```text
ContextEvidenceLink
    link ID
    context artifact/record/field ID
    project/graph/reference/evidence/source IDs
    exact generations/universe
    derivation rule/path IDs
    confidence/coverage/conflict refs
    canonical digest
```

Every material claim has at least one link or an explicit deterministic derivation chain.

## 17. Projection status

```text
Exact
ExactWithEvidenceSidecar
CompactButCompleteForDeclaredFields
LossyDeclared
Unsupported
NotEvaluated
Truncated
```

Compact does not mean source/graph complete; it is scoped to declared fields/profile.

## 18. Context loss/omission record

```text
ContextLossRecord
    loss ID/category/severity
    exact input/subject/artifact/field IDs
    reason = budget | unsupported_detail | source_unavailable | conflict | partial_coverage | privacy_security | deduplication | renderer_limit | deferred_capability
    emitted compact/sidecar/omitted representation
    affected capabilities/tasks
    evidence/coverage/conflict refs
    continuation/detail route refs
    canonical digest
```

```text
ContextOmissionRecord
    omitted exact entity/relation/evidence/source IDs or count/partition manifest
    reason and priority class
    whether recoverable by continuation
    budget/blocker refs
    canonical digest
```

## 19. Stopping record

```text
ContextStoppingRecord
    stop ID
    reason = RequestedComplete | NoNewEvidence | BudgetExhausted | DepthLimit | CycleClosed | CoverageBoundary | ConflictBoundary | UnsupportedDetail | Cancelled | Failed
    exact frontier/root/lane
    query/budget/coverage/conflict refs
    continuation availability
    canonical digest
```

`NoNewEvidence` is not authoritative absence.

## 20. Context bundle

```text
ContextBundle
    bundle ID/version
    exact input snapshot/request/profile IDs
    ProjectMap ID: optional
    included L0/L1 skeleton/member/source-node IDs
    included entity/relation/path/evidence/source-excerpt IDs
    coverage/loss/omission/stopping IDs
    frontier/continuation ID: optional
    metrics/evaluation refs
    artifact eligibility/status
    canonical ordering and digest
```

## 21. Continuation cursor

```text
ContextContinuation
    continuation ID
    exact input/request/profile/budget/tokenizer IDs
    ordering/profile version
    current frontier ID/digest
    included/visited set digests
    used/reserved budget state
    last stable work-item key
    integrity digest
```

## 22. Context metrics

```text
ContextMetrics
    metric set ID
    input source/project/graph size measures
    output entities/relations/skeletons/evidence/source records
    output UTF-8 bytes/Unicode scalars/lines/structured nodes
    exact token count + tokenizer profile: optional
    estimated token range/profile: optional separate field
    redundancy/duplicate avoided counts
    mandatory-record recall and evidence-closure counts
    omitted/truncated/partial/conflict counts
    expansion steps/query costs
    canonical digest for deterministic fields
```

Timing/memory may be supplemental and hardware/profile scoped.

## 23. Evaluation report

```text
ContextEvaluationReport
    report ID
    exact fixture/request/input/profile IDs
    mandatory structural/evidence expected versus included
    relevance labels/results
    redundancy/compression/budget results
    continuation stability/no-new-evidence results
    consumer task outcomes under pinned protocol
    error/loss/blocker records
    acceptance decision
    canonical digest for deterministic judgments
```

## 24. Artifact eligibility

```text
Fixture
Candidate
ValidatedForDeclaredContextProfile
```

Validated requires all mandatory structural/evidence/security/determinism/budget gates for the declared profile. It does not mean project/graph/source complete beyond input capabilities.

## 25. Canonical ordering

```text
Project Map sections by profile section order/ID
roots by universe/kind/semantic key/ID
skeletons by detail/kind/owner/load/source/semantic key/ID
members by semantic ordinal/kind/name/ID
relations by lane/kind/source/target/qualifiers/ID
reason paths by length then relation/entity stable tuple
source nodes/excerpts by file/source-unit/span/ID
evidence links by artifact/field/provenance/source/ID
loss/omission/stopping by subject/category/reason/ID
frontier items by priority class/lane/root/path/stable semantic key
```

No storage row, hash, filesystem, worker, query completion, or model-score ordering.
