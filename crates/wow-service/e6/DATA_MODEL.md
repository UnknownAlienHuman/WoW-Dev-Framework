# E6-B service data model

**Status:** normative transport-independent model. Concrete Rust names may change only with equivalent contract/fixture updates.

## Configuration

```text
E6BServiceConfiguration
    provider catalog profile
    provider descriptor and capability profile
    credential authorization profile
    provider session profile
    durable operation/idempotency/reconciliation profile
    external artifact/catalog/store profile
    project/reference mapping profiles
    explicit selection profile
    context handoff profile
    retention/cache/audit profile
    privacy/license/security/budget profile
    result/error/canonicalization profile
    canonical digest
```

Configuration contains nonsecret identifiers only.

## Provider reference and session

```text
ProviderReference
    provider descriptor ID/version/digest
    adapter implementation/profile ID
    expected capability set
    permitted operation family
```

```text
ProviderCredentialAuthorizationReceipt
    nonsecret authorization receipt ID
    provider/principal/service/scope profile refs
    decision state
    issuance/expiry/revocation/replay evidence refs
    credential material present = false
    canonical digest
```

```text
ProviderSessionReceipt
    provider/descriptor/adapter/profile IDs
    authorization receipt ID
    opaque session handle: operational only
    negotiated capability report ID
    external-state observation/generation receipt
    lifecycle/reconciliation capability
    privacy/security state
```

The opaque handle is never serialized into public/canonical output.

## External state binding

```text
ExternalStateBinding
    class:
        StableExternalGeneration
        ObservedMutableGeneration
        OpaqueExternalState
    exact E6-A state artifact/receipt ID
    provider/session/descriptor/profile refs
    state digest or observation receipt where available
    reproducibility/continuation/cache limits
    coverage/conflicts
    canonical digest
```

## Durable operation

```text
ExternalCandidateOperationRecord
    OperationId
    CanonicalRequestDigest
    operation kind
    exact provider/descriptor/session/external-state bindings
    exact owner publications and profiles where applicable
    durable state
    provider/store/owner/context effect receipts
    response-loss/reconciliation records
    output artifact IDs
    retention/audit/closure state
    error/blocker refs
    canonical digest
```

States:

```text
Planned
SessionAcquiring
Registered
Dispatched
ResponseReceived
Normalized
Persisted
Cataloged
RetentionAdmitted
Mapping
AwaitingExplicitSelection
ContextBuilding
Completed
NoChange
Cancelled
Failed
OutcomeUnknown
Quarantined
Superseded
```

## Service query request

```text
ExternalCandidateServiceQueryRequest
    OperationId
    ProviderReference
    exact ExternalStateBinding or explicit observation request
    closed E6-A candidate query
    expected descriptor/capability/profile digests
    provider/session/reconciliation profiles
    persistence/retention/privacy/license/audit profiles
    budgets/cancellation
    canonical digest
```

No raw endpoint, credential, tool name, MCP JSON-RPC body, database path, SQL, script, or model prompt.

## Stored result descriptor

```text
ExternalCandidateStoredResult
    exact E6-A result-set/artifact ID and digest
    provider/session/external-state/query identities
    E6-A authority ceiling record
    immutable object/catalog refs
    coverage/conflicts/truncation/continuation
    privacy/license classification
    retention/audit refs
    canonical digest
```

## Mapping request

```text
ExternalCandidateMappingRequest
    OperationId
    exact result-set ID/digest
    exact candidate ID/digest
    exact UnverifiedProviderLocator ID/digest
    target owner kind: Project | Reference
    exact owner publication/view/generation/profile
    exact mapping profile ID
    permitted source/metadata disclosure
    budgets/cancellation
    canonical digest
```

## Mapping receipt

```text
ExternalCandidateMappingReceipt
    mapping receipt ID/version
    exact request/result/candidate/locator IDs
    target owner publication/view/generation/profile
    owner mapping operation/implementation/profile IDs
    status:
        ExactMapped
        MultipleMappings
        NoMappingWithOwnerAuthority
        NoMappingPartial
        Conflict
        NotEvaluated
        Failed
    exact mapped owner root(s), source handle(s), or reference entity IDs
    comparison/evidence records owned by mapper
    coverage/conflicts/truncation
    authority and nonclaim records
    retention/audit refs
    canonical digest
```

`ExactMapped` maps locator identity only.

## Selection request and receipt

```text
ExternalCandidateSelectionRequest
    OperationId
    exact result-set/candidate/mapping-receipt IDs and digests
    exact mapped owner root
    explicit selection origin:
        UserExplicit
        CallerExplicit
        ReviewedPolicyExplicit
    selector profile ID
    expected authority/coverage/conflict state
    bounded reason code and optional untrusted note
    retention/audit profile
    canonical digest
```

```text
ExternalCandidateSelectionReceipt
    selection receipt ID/version
    exact request/result/candidate/mapping/root IDs
    explicit origin and policy
    decision: Selected | Rejected | Deferred
    validation state
    authority remains Candidate
    no edit/tool/runtime/platform permission
    retention/audit refs
    canonical digest
```

## Context request and handoff

```text
ExternalCandidateContextRequest
    OperationId
    exact selection receipt
    exact mapped project/reference root
    exact ContextUniverseSet or allowed symbolic outer selectors
    exact context intent/profile/budgets/privacy/license/output profile
    external evidence disclosure profile
    cancellation
    canonical digest
```

```text
ExternalCandidateContextHandoffReceipt
    exact E3-C context operation/request/result IDs
    mapped root and owner view identities
    external result/mapping/selection refs retained outside context fact authority
    exclusions proving provider prose/rank/score were not injected as framework facts
    context status/coverage/conflicts/omissions
    retention/audit/closure refs
    canonical digest
```

## Service result envelope

```text
E6BServiceResultEnvelope
    schema/operation version
    public/normalized request and operation IDs
    provider/descriptor/session/external-state identities
    exact external result/candidate/artifact/mapping/selection/context IDs
    operation-specific typed payload
    service, validation, mapping, selection, and context statuses
    authority ceiling and mandatory nonclaims
    coverage/conflicts/partial/truncation/continuation
    durable effect/response-loss/reconciliation state
    privacy/license/credential/retention/cache/audit/closure state
    canonical digest
```

## Statuses

```text
Complete
NoChange
CandidateOnly
Partial
Blocked
ConflictBlocked
Truncated
OutcomeUnknown
NotEvaluated
Cancelled
Failed
```

Validation:

```text
Valid
Invalid
NotEvaluated
```

## Cache validation request

```text
ExternalCandidateCacheValidationRequest
    exact E6-A cache key/entry/result IDs
    provider/descriptor/adapter/external-state/query/profile identities
    privacy/license/consumer scope
    retention and artifact digest guards
    canonical digest
```

Cache state cannot become evidence of provider freshness or negative authority.

## Audit record

```text
ExternalCandidateAuditRecord
    immutable event ID/sequence/previous digest
    operation/request/provider/session/state identities
    event kind
    nonsecret principal/service/authorization refs
    exact result/mapping/selection/context/effect refs
    disclosure/privacy/license state
    response-loss/reconciliation state
    trusted time evidence when required
    canonical digest
```

## Identity exclusions

Wall-clock duration, process/thread ID, socket/connection handle, local database/cache path, retry count, host, terminal, environment, raw credential, provider cursor bytes, and physical storage layout never enter Candidate semantic identity.
