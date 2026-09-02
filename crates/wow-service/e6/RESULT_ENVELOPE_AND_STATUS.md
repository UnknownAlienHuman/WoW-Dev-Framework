# E6-B service result envelopes and conservative status

**Status:** normative.

## Envelope

Every public E6-B operation returns one tagged `E6BServiceResultEnvelope`:

```text
schema and operation version
public request ID, normalized request ID, OperationId, request digest
exact provider/descriptor/adapter/session/external-state refs
exact E6-A query/result/candidate/artifact/explanation/cache refs
exact project/reference owner publication and mapping refs
exact selection/context refs when present
operation-specific typed payload
service, validation, mapping, selection, context, and cache statuses
authority ceiling and mandatory nonclaims
coverage/conflicts/partial/truncation/continuation
provider effect/response-loss/reconciliation state
privacy/license/credential/retention/audit/closure state
canonical digest
```

Owner records are preserved, not rewritten into stronger service claims.

## Service statuses

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

Mapping:

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

Selection:

```text
Selected
Rejected
Deferred
Invalid
NotEvaluated
```

## Conservative precedence

Default outer folding:

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Truncated
Partial
CandidateOnly
NoChange
Complete
```

Operation-specific completed validations may return outer `Complete` with `validation = Invalid`. A successful provider query normally returns `CandidateOnly`, because its evidence remains Candidate.

## Typed payloads

```text
ExternalCandidateStatusOutcome
ExternalProviderValidationOutcome
ExternalGenerationValidationOutcome
ExternalCandidateQueryOutcome
ExternalCandidateContinuationOutcome
ExternalCandidateCatalogOutcome
ExternalCandidateValidationOutcome
ExternalCandidateExplanationOutcome
ExternalCandidateArtifactOutcome
ExternalCandidateMappingOutcome
ExternalCandidateSelectionOutcome
ExternalCandidateContextOutcome
ExternalCandidateOperationOutcome
ExternalCandidateCacheOutcome
ServiceFailureOutcome
ServiceCancelledOutcome
```

Unknown/mismatched tags are invalid.

## Mandatory authority record

Every payload containing provider-origin evidence includes:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Mapping payload adds:

```text
mapping authority = locator-to-exact-owner-record only
provider claims validated = false
```

Selection payload adds:

```text
selection authority = explicit choice only
edit/tool/runtime/platform permission = none
```

## Mandatory nonclaims

As applicable:

```text
not-project-or-reference-source-until-owner-mapped
not-platform-contract
not-runtime-evidence
not-lineage-or-replacement-proof
not-migration-or-impact-proof
not-negative-authority
not-edit-or-tool-authorization
provider-rank-and-score-are-provider-local-metadata
zero-result-is-request-scoped-only
exact-mapping-does-not-validate-provider-prose-or-relations
explicit-selection-does-not-upgrade-confidence
context-framework-facts-exclude-provider-prose-rank-and-score
```

## Query outcome

Preserve exact E6-A:

- provider/external-state/query profiles;
- immutable candidate ordering and provider-local scores/ranks;
- coverage/conflicts/truncation/continuation;
- Candidate authority and zero-result classification;
- validation/cache state;
- durable effect and retention state.

Service cannot call a provider zero-result `not found` without the E6-A scoped wording.

## Mapping outcome

Preserve exact mapping receipt and owner evidence/coverage. `MultipleMappings`, partial, conflict, and `NotEvaluated` cannot be rendered as selected or mapped.

## Context outcome

Contains exact context result plus separately labelled external Candidate attachment. Provider metadata is not merged into context facts. Context success does not validate provider claims.

## Failure outcome

```text
ServiceFailureOutcome
    error code/stage/operation
    exact identities known before failure
    provider/store/owner/context error refs
    response-loss/reconciliation state
    mapping/selection/context partial state
    retention/audit/closure/recovery refs
    safe bounded metadata under privacy profile
```

Default output excludes raw credentials, endpoints, session handles, provider cursors, private paths/URIs/snippets, source bodies, and raw owner/store handles.

## NoChange

Allowed only with exact owner/store proof of an existing artifact/result/receipt that matches the whole canonical request. Cache hit, same text, zero result, skipped call, or missing capability is not `NoChange`.

## OutcomeUnknown

Indicates a potentially committed provider/store/context effect not reconciled. It maps to a hard recovery state and forbids blind repeat.

## Canonicalization

- frozen object/tag/field ordering;
- canonical UTF-8 strings, IDs, enums, nulls, arrays, numbers;
- no host, process, timing, connection, retry, terminal, or physical storage state in semantic digest;
- provider/local operational receipts referenced by exact IDs/digests;
- one canonical JSON document at service boundary.

## Validation

Validate tag/status compatibility, exact identity closure, E6-A authority ceiling, mapping/selection/context boundaries, mandatory nonclaims, coverage/conflicts/truncation, response-loss, privacy/license/credential redaction, retention/audit/closure, and canonical bytes.
