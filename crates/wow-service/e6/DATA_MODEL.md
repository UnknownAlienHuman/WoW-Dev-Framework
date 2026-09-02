# E6-B data model

**Status:** normative.

## Provider configuration

```text
ExternalProviderConfiguration
    configuration ID/version/digest
    E6-A provider descriptor ID/digest
    adapter contract and transport profile IDs
    credential reference class, never credential bytes
    session acquisition and cancellation policy
    allowed external-state/query/output profiles
    privacy/license/consumer policy
    hard resource and quota budgets
    enabled/disabled state and reason
```

## Authorization

```text
ExternalProviderUseAuthorization
    authorization receipt ID
    exact configuration/provider/credential-reference scope
    operation/query purpose
    consumer and privacy/license profile
    issue/expiry/revocation/replay state
    canonical digest
```

Authorization does not alter candidate authority or source truth.

## Session binding

```text
ExternalProviderSessionBinding
    session binding ID
    configuration/descriptor/adapter IDs
    nonsecret credential-reference and authorization IDs
    transport contract/profile
    capability observation/set
    external-state acquisition policy
    provider operation reconciliation capability
    cancellation/late-response/close behavior
    opened/closed state and receipts
    canonical digest
```

No token, key, cookie, password, private endpoint, command, process handle, or database path is exposed.

## Durable query operation

```text
ExternalCandidateQueryOperation
    OperationId + CanonicalRequestDigest
    exact configuration/authorization/session-policy IDs
    E6-A descriptor/capability/state/query/profile IDs
    current-resolution receipts when explicitly permitted
    cumulative budgets and cancellation
    dispatch/result/reconciliation state
    immutable effect receipts
    canonical digest
```

## Query/result catalog records

```text
ExternalCandidateQueryRecord
    operation/request/query/session/state identities
    dispatch and provider receipt state
    result-set/artifact references
    partial/truncation/zero/conflict/failure state
    retention/audit/reconciliation

ExternalCandidateResultCatalogRecord
    exact E6-A RawProviderResponseRecord reference
    exact ExternalCandidateResultSet bytes/digest
    validation report
    provider/state/query/profile bindings
    Candidate authority and nonclaims
    privacy/license/redaction state
    retention receipt
```

## Mapping request

```text
ExternalLocatorMappingRequest
    exact result/candidate/locator IDs
    target owner kind: project | reference
    exact owner store/publication/generation/view selector and guard
    mapping profile and requested entity/source classes
    consumer/privacy/license/budget/cancellation profiles
    OperationId + CanonicalRequestDigest
```

## Mapping result

```text
ExternalLocatorMappingRecord
    mapping record ID
    exact request/result/candidate/locator IDs
    exact owner view/generation and mapping implementation/profile
    status:
        ExactMapped
        MultipleMappings
        NoMappingWithOwnerAuthority
        NoMappingPartial
        Conflict
        NotEvaluated
        Failed
    zero/one/many exact owner source/entity handles
    owner coverage/negative-authority/conflict evidence
    provider fields validated versus owner fields
    fields not validated
    nonclaims
    retention/audit/effect receipt
    canonical digest
```

## Selection

```text
ExternalCandidateSelectionRequest
    exact result/candidate/mapping IDs
    exact intended use and target context profile
    caller/authorization/consumer IDs
    decision: Selected | Rejected | Deferred
    reason codes and bounded note
    OperationId + CanonicalRequestDigest

ExternalCandidateSelectionReceipt
    immutable supplied decision
    exact mapping state and mapped owner handle when selected
    authority remains Candidate
    no automatic verification/promotion/edit authorization
    retention/audit/effect receipt
    canonical digest
```

`Selected` requires `ExactMapped` for context handoff.

## Context handoff

```text
ExternalCandidateContextRequest
    exact selection/mapping/result/candidate IDs
    exact retained project/reference/graph selectors and guards
    exact mapped root
    existing E3 context operation/request/profile
    provider sidecar output policy
    OperationId + CanonicalRequestDigest

ExternalCandidateContextResult
    exact normal ContextService semantic/rendered artifact references
    separate ExternalCandidateSidecar
    mapping/selection evidence
    cross-owner generation/profile validation
    omissions/coverage/conflicts/budgets/continuation
    retention/audit/closure state
    canonical digest
```

## Candidate sidecar

```text
ExternalCandidateSidecar
    provider/result/candidate/state/query identities
    permitted labels/rank/score/locator/snippet/summary fields
    mapping and explicit selection references
    provenance = semantic_candidate
    confidence = Candidate
    negative_authority = unavailable
    source verification limited to mapping receipt fields
    privacy/license/redaction/loss/nonclaims
```

The sidecar is not part of `ContextSemanticPack` truth.

## Operation reconciliation

```text
ExternalCandidateOperationReconciliation
    exact operation/request/configuration/session/provider identities
    owner/provider query-effect state
    result/catalog/mapping/selection/context effect state
    duplicates/conflicts
    safe-to-return | unsafe-to-retry | new-explicit-operation-required
    recovery instructions and canonical digest
```

## Coverage axes

Keep configuration, authorization, session, provider state, transport, query, response, normalization, result publication, owner mapping, selection, local context, privacy/license, retention/audit, cancellation, close, and reconciliation coverage separate.