# E4-A search data model

**Status:** normative semantic/logical model.

## Profiles

```text
SearchProfileSet
    document schema/field registry
    identifier normalization profile
    text normalization/tokenizer profile
    FTS5 physical/query profile
    fuzzy identifier profile
    shape feature profile
    graph expansion profile
    lane eligibility profile
    fusion/ranking profile
    miss/negative-authority profile
    pagination/continuation profile
    privacy/license profile
    budget/security profile
    canonicalization/error schema profiles
    canonical digest
```

## Shard source binding

```text
SearchShardSourceBinding
    universe kind/ID
    exact owner store/project/reference generation IDs
    exact ProjectSnapshot/GraphSnapshot/ReferenceView IDs as applicable
    owner read catalog IDs
    source/privacy/license/capability/coverage/conflict manifests
    canonical digest
```

## Search document

```text
SearchDocument
    document_id
    exact universe and owner entity key/ID
    document kind/schema version
    canonical display/identifier fields
    explicit alias records
    namespace/member/receiver fields
    structured signature/type/restriction/load/role features
    bounded searchable text fields
    source/reference/graph/detail handles
    field-origin records
    entity evidence/provenance/confidence/coverage/conflicts
    privacy/license class
    producer/partition/version
    canonical logical digest
```

## Field origin

```text
SearchFieldOrigin
    document/field ID
    owner record/source/reference/graph assertion IDs
    exact source handle/span where applicable
    authority class
    provenance/confidence/coverage/conflicts
    transformation/normalization records
    privacy/license decision
```

## Document partition

```text
SearchDocumentPartition
    partition key and partition version ID
    exact owner generation/profile
    producer ID/version
    ordered document IDs/digests
    field/origin/evidence/coverage/conflict manifests
    logical counts/bytes
    canonical digest
```

## Shard manifest

```text
SearchShardManifest
    search_shard_id
    SearchShardSourceBindingId
    SearchProfileSetId
    exact SQLite/FTS5 runtime profile ID
    complete ordered document partition membership
    logical document/field/term/statistics manifests
    FTS integrity/validation/golden-query reports
    capability/coverage/conflict/omission summaries
    physical artifact/object handle and digest classification
    state: Building | PublishedInactive | Validated | SealedReadOnly | Failed | Quarantined
    canonical digest
```

SearchShard identity excludes physical SQLite bytes unless the profile explicitly classifies them reproducible.

## Search universe set

```text
SearchUniverseSet
    search_universe_set_id
    one exact primary user-project SearchShard
    optional exact Blizzard UI SearchShard
    exact Reference SearchShard
    compatibility report
    query/ranking profile compatibility
    capability/coverage/conflict summaries
    canonical digest
```

No current pointer.

## Search request

```text
SearchRequest
    request_id
    exact SearchUniverseSetId
    explicit query intent class
    literal query text: optional
    exact entity key/ID/name/alias/prefix fields: optional
    structured kind/universe/namespace/member/receiver/signature/type/restriction/load/role filters/features
    enabled/required lanes
    confidence/provenance/coverage policy
    graph expansion profile/overrides
    ranking profile
    result/detail/source privacy policy
    budgets/cancellation
    continuation: optional
    canonical digest
```

## Query AST

```text
NormalizedSearchQuery
    normalized terms/phrases/prefixes
    exact identifier/alias/member tokens
    structured filters/features
    safe FTS expression AST
    lane eligibility and skip reasons
    canonical digest
```

No raw SQL/FTS/regex/expression string.

## Lane result

```text
SearchLaneResult
    lane ID/version
    shard/universe IDs
    normalized query ID
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
    lane/signal kind
    exact matched fields/origin records
    exact-match/alias/prefix/text/fuzzy/shape/graph features
    shard-local rank ordinal and optional raw diagnostic score
    graph relation/path/evidence refs
    authority/provenance/confidence/coverage/conflicts
    penalties/caps
    canonical integer feature vector
    canonical digest
```

Raw floating diagnostic score is excluded from canonical fusion identity unless a frozen decimal representation/profile explicitly permits it; ordinal/tie results are canonical.

## Ranked candidate

```text
SearchCandidate
    candidate_id
    exact entity/universe/detail handles
    exact SearchEntityRecord reference
    all contributing signal IDs
    all skipped/rejected/capped signal summaries relevant to explanation
    authority band
    canonical integer lane contributions/penalties/total
    stable tie key
    query-relative match status
    entity evidence/provenance/confidence/coverage/conflicts
    result omissions/truncation
    canonical digest
```

## Result

```text
SearchResult
    result_id
    universe/request/query/profile IDs
    ordered candidates
    lane result manifests
    ranking explanation manifest
    miss classification
    coverage/conflicts/omissions/budget
    continuation
    validation report ID
    canonical digest
```

## Miss classification

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
    result/candidate manifest or replay contract
    lane cursors/frontiers
    last stable fusion key
    total budget state
    prior omissions/truncation
    integrity digest
```

## SearchStore logical records

```text
search_shard_source_binding
search_profile_manifest
search_document_partition/version/membership
search_document and typed fields/origins
exact/alias/prefix/shape indexes
FTS5 content/index mapping
shard manifest/validation/golden reports
retention/GC roots
```

SQLite rowid remains private physical identity.
