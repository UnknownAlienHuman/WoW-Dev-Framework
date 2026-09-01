# E4-A search data model

**Status:** normative semantic/logical model.

## Profiles

```text
SearchProfileSet
    document schema and field registry
    identifier and Unicode normalization profile
    text normalization/tokenizer profile
    physical SearchShard/FTS5 profile
    exact/alias/member/prefix lane profiles
    identifier-similarity profile
    structured-shape profile
    graph-expansion profile
    lane eligibility/fallback profile
    fusion/ranking/explanation profile
    miss/negative-authority profile
    result-manifest/pagination/continuation profile
    privacy/license/snippet profile
    budget/security/cancellation profile
    canonicalization/error schema profiles
    canonical digest
```

## Shard source binding

```text
SearchShardSourceBinding
    universe kind and ID
    exact owner store/project/reference generation IDs
    exact ProjectSnapshot/GraphSnapshot/ReferenceView IDs as applicable
    owner read-catalog IDs
    source/privacy/license/capability/coverage/conflict manifests
    owner schema/profile compatibility
    canonical digest
```

## Search entity record

```text
SearchEntityRecord
    exact owner entity key/ID
    owner universe/generation
    entity kind
    owner detail/source/reference/graph handles
    evidence/provenance/confidence/coverage/conflicts
    privacy/license class
```

This record is referenced, not rewritten as search truth.

## Search document

```text
SearchDocument
    document_id
    exact SearchEntityRecord reference
    document kind/schema version
    canonical display and identifier fields
    explicit alias records
    namespace/member/receiver fields
    structured signature/type/restriction/load/role features
    bounded approved text fields
    source/reference/graph/detail handles
    SearchFieldOrigin records
    entity evidence/provenance/confidence/coverage/conflicts
    privacy/license class
    producer/partition/version
    canonical logical digest
```

## Field origin

```text
SearchFieldOrigin
    document/field/value ID
    owner record/source/reference/graph assertion IDs
    exact source handle/span where applicable
    authority class
    provenance/confidence/coverage/conflicts
    transformation/normalization/loss records
    privacy/license decision
```

## Document partition

```text
SearchDocumentPartition
    partition key/version ID
    exact owner generation/profile
    projection producer ID/version
    ordered document IDs/digests
    field/origin/evidence/coverage/conflict manifests
    logical counts and bytes
    canonical digest
```

## Shard build plan

```text
SearchShardBuildPlan
    exact source binding/profile set
    base shard: optional and exact
    complete ordered target partition membership
    new/reused/removed partition versions
    logical exact/alias/prefix/shape index operations
    bounded FTS document operations
    expected counts/digests/golden results
    store operation/validation catalogs
    budgets/cancellation
    canonical digest
```

## Shard manifest

```text
SearchShardManifest
    search_shard_id
    SearchShardSourceBindingId
    SearchProfileSetId
    exact SQLite/FTS5 runtime profile ID
    complete ordered partition membership
    logical document/field/token/statistics manifests
    exact/alias/prefix/shape/FTS index manifests
    integrity/validation/golden-query reports
    capability/coverage/conflict/omission summaries
    physical artifact/object handle and reproducibility class
    state:
        Building
        PublishedInactive
        Validated
        SealedReadOnly
        Failed
        Quarantined
    canonical digest
```

## Search universe set

```text
SearchUniverseSet
    search_universe_set_id
    one exact primary user-project SearchShard
    optional exact Blizzard UI SearchShard
    exact Reference SearchShard when required
    compatibility report
    query/ranking profile compatibility
    capability/coverage/conflict summaries
    canonical digest
```

No current pointer or symbolic latest field.

## Search request

```text
SearchRequest
    request_id
    exact SearchUniverseSetId
    explicit query-intent class
    bounded literal text: optional
    exact entity key/ID/name/alias/member/prefix fields: optional
    structured kind/universe/namespace/receiver/signature/type/restriction/load/role features
    enabled and required lanes
    confidence/provenance/coverage policy
    graph expansion policy
    ranking/explanation profile
    result/detail/snippet privacy policy
    result and cumulative budgets
    cancellation
    continuation: optional
    canonical digest
```

## Normalized query and plan

```text
NormalizedSearchQuery
    normalized exact identifiers
    normalized approximate identifier tokens
    bounded terms/phrases/prefixes
    structured filters and shape features
    safe FtsQueryAst
    lane eligibility and skip reasons
    canonical digest
```

```text
SearchQueryPlan
    exact shards and lane order
    per-lane requests and budgets
    fallback conditions
    candidate cap/fusion plan
    result-manifest policy
    canonical digest
```

## Lane result

```text
SearchLaneResult
    lane ID/version
    shard/universe/query IDs
    ordered SearchCandidateSignal[]
    executed/skipped/partial/failed/truncated state
    query/index/owner coverage
    budget/cancellation
    canonical digest
```

## Candidate signal

```text
SearchCandidateSignal
    signal_id
    exact entity/document/universe/shard IDs
    lane and signal kind
    matched fields/origin records
    exact/alias/member/prefix/text/fuzzy/shape/graph features
    shard-local rank ordinal
    optional noncanonical raw diagnostic score
    graph relation/path/evidence refs
    authority/provenance/confidence/coverage/conflicts
    caps/penalties
    canonical integer feature vector
    canonical digest
```

## Candidate

```text
SearchCandidate
    candidate_id
    exact entity/universe/detail handles
    SearchEntityRecord reference
    all contributing signal IDs
    relevant skipped/rejected/capped signal summaries
    authority band
    canonical integer contributions/penalties/lexicographic rank tuple
    stable tie key
    query-relative match class
    entity evidence/provenance/confidence/coverage/conflicts
    result omissions/truncation
    canonical digest
```

## Explanation

```text
SearchCandidateExplanation
    candidate/request/result IDs
    exact owner entity and field origins
    matched and unmatched exact constraints
    lane contributions in canonical order
    authority-band decision
    caps/penalties and reasons
    local lane ordinals
    final rank tuple and tie key
    skipped/failed/partial lanes
    conflicts/coverage/omissions
    nonclaims
    canonical digest
```

## Result-set manifest and result

```text
SearchResultSetManifest
    exact universe/request/query/profile IDs
    ordered candidate IDs and rank tuples
    lane-result manifest IDs
    miss/coverage/conflict/omission/budget summaries
    total known candidate count under stated coverage
    canonical digest
```

```text
SearchResult
    result_id
    exact result-set manifest
    one whole-candidate page
    explanations/detail handles as requested
    miss classification
    coverage/conflicts/omissions/budget
    continuation
    validation report ID
    canonical digest
```

## Miss classes

```text
ExactFound
ExactNotFoundWithAuthority
ExactNotFoundPartial
NoCandidatesUnderExecutedLanes
CandidateOnly
LaneUnavailable
ConflictBlocked
Truncated
Cancelled
Failed
```

## Continuation

```text
SearchContinuation
    exact shard/universe/request/query/profile IDs
    SearchResultSetManifestId
    last whole-candidate ordering key
    next page boundary
    cumulative budget state
    prior omissions/truncation
    retention requirements
    integrity digest
```

## SearchStore logical records

```text
search_shard_source_binding
search_profile_manifest
search_document_partition/version/membership
search_document and typed fields/origins
exact/alias/member/prefix/shape indexes
FTS content/index mapping
shard manifest/validation/golden reports
result-set manifest when retained
retention/GC roots
```

SQLite rowid remains private physical identity.
