# E6-A data model

**Status:** normative.

## Provider descriptor

```text
ExternalCandidateProviderDescriptor
    provider descriptor ID/version
    adapter contract ID/version
    provider family/product identity as provenance only
    supported transport profile IDs
    allowed operation schemas
    response schema versions
    score/rank interpretation classes
    external-state capabilities
    continuation/cancellation capabilities
    privacy/license/source-field capabilities
    hard system limits
    canonical digest
```

The descriptor contains no credential, executable command, private endpoint, database path, or arbitrary tool name.

## Capability set

```text
ProviderCapabilitySet
    exact descriptor/adapter/session observation
    supported/unsupported/unknown operations
    schema/profile compatibility
    generation/state capabilities
    pagination/cancellation behavior
    source locator/snippet fields
    score/rank fields and units
    provider-declared coverage fields
    validation/loss report
    canonical digest
```

Runtime negotiation cannot add operations absent from the reviewed descriptor.

## External state

```text
ExternalStateBinding
    class:
        StableExternalGeneration
        ObservedMutableGeneration
        OpaqueExternalState
    provider/descriptor/adapter IDs
    generation/index/corpus ID and digest when stable
    observation/session receipt when mutable
    explicit opaque reason/profile when opaque
    repository/corpus scope claims as provider metadata
    staleness/freshness evidence
    coverage and conflicts
    canonical digest
```

## Query

```text
ExternalCandidateQuery
    query ID/schema
    exact provider descriptor/capability/state binding
    operation kind
    bounded structured terms/seeds/filters
    repository/universe scope as provider-side scope only
    requested result fields
    item/byte/depth/time/memory limits
    continuation: optional
    privacy/license/output profile
    cancellation
    canonical digest
```

No raw SQL, FTS, regex program, shell, MCP JSON, script, callback, model prompt, or arbitrary tool name.

## Raw response

```text
RawProviderResponseRecord
    exact request/transport/state binding
    provider response schema/version
    bounded raw fields and unknown-field records
    result/pagination/coverage/error metadata
    raw response digest/size
    validation report
```

Raw response is untrusted and is not a public authority artifact.

## Candidate

```text
ExternalCandidate
    candidate ID
    result-set/query/provider/state IDs
    provider-local result identity
    candidate kind
    provider labels as quoted metadata
    provider-local rank/score with interpretation profile
    UnverifiedProviderLocator[]
    bounded snippet/summary fields as untrusted data
    raw-field origin/loss/unknown/conflict records
    provider-declared and bridge-observed coverage
    provenance = semantic_candidate
    confidence = Candidate
    negative_authority = unavailable
    canonical digest
```

## Locator

```text
UnverifiedProviderLocator
    provider repository/root/revision/path/URI/symbol/span/digest fields when supplied
    normalization profile
    missing/unknown/conflicting fields
    original provider field origins
    verification state = Unverified
    canonical digest
```

No project/reference entity or stable source handle is embedded.

## Result set

```text
ExternalCandidateResultSet
    result-set ID
    exact descriptor/capability/state/query/transport profile
    ordered candidate records
    zero-result classification
    coverage/partial/truncation/conflict/failure state
    loss and unknown-field reports
    continuation record
    budgets/cancellation
    privacy/license state
    canonical digest
```

## Artifact

```text
ExternalCandidateArtifact
    artifact ID
    exact result set
    selected candidate subset only when explicitly supplied as IDs by caller; no ranking selection
    explanation records
    retained raw-field/loss/coverage/conflict state
    mandatory Candidate/negative-authority nonclaims
    canonical digest
```

## Explanation

```text
ExternalCandidateExplanation
    provider/state/query/result/candidate IDs
    exact raw fields used
    normalization rules and transformations
    provider-local score/rank interpretation
    locator fields
    loss/unknown/conflict/coverage
    authority ceiling and nonclaims
```

## Comparison

```text
ExternalCandidateComparison
    exact compatible result-set IDs
    field/set/order/rank changes within declared comparable profiles
    incompatibilities and omissions
    no truth/winner/quality conclusion
    canonical digest
```

## Cache entry

```text
ExternalCandidateCacheEntry
    exact descriptor/capability/state/query/profile key
    exact result/artifact bytes and digest
    original staleness/coverage/authority
    cache validation report
    operational cache metadata excluded from semantic identity
```

## Coverage axes

Keep provider scope, external-state identity, transport, response schema, normalization, field/loss, candidate enumeration, pagination, privacy/license, cache validation, and cancellation coverage separate.