# E7-A results, errors, protocol projection, and telemetry

**Status:** normative.

## Service result envelope

```text
E7AServiceResultEnvelope
    schema/operation version
    OperationId / CanonicalRequestDigest
    SessionId / exact SessionViewSetId / OverlayGenerationId
    exact document/root/entity/result/continuation IDs
    operation-specific owner result
    service/validation/authorization status
    capability/coverage/conflicts/blockers/omissions
    cancellation/progress/retention/closure state
    privacy/license/source-boundary state
    required nonclaims
    canonical digest
```

Transport request IDs and framing do not enter the semantic envelope digest.

## Statuses

```text
Complete
NoChange
Partial
CandidateOnly
Blocked
ConflictBlocked
Truncated
Busy
OutcomeUnknown
NotEvaluated
Cancelled
Failed
```

Validation payloads remain:

```text
Valid
Invalid
NotEvaluated
```

Authorization payloads retain exact domain states rather than booleans.

## Conservative folding

Default severity/precedence:

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Busy
Truncated
Partial
CandidateOnly
NoChange
Complete
```

Operation-specific valid/invalid outcomes remain separate. A completed diagnostic query can be `Complete` while findings exist; “complete” means operation closure, not clean source.

## Protocol errors versus domain results

Protocol failures:

```text
invalid framing
parse error
invalid request/schema
method/tool/resource not found
unsupported protocol revision
invalid request ID/version/cursor
message too large
server/session not initialized
```

Domain results:

```text
valid request with findings
valid request with no findings under partial coverage
multiple definitions
candidate-only search
authorization denied
operation blocked or NotEvaluated
truncated references/context
owner capability unavailable
```

Adapters use protocol error objects only when required. Valid domain states are returned in the protocol’s normal result shape with exact status/metadata, not collapsed into generic internal errors.

## LSP projection

- Findings map to diagnostics without changing finding ID, severity, source, evidence, or coverage semantics.
- Diagnostic result IDs bind exact session/overlay/generation/profile state.
- Multiple definition/reference locations remain ordered exact results.
- Hover content uses typed static templates and source-data boundaries.
- Advisory actions contain no executable command/edit in E7-A.
- Partial/truncated results include explicit data/progress/continuation metadata where the frozen profile supports it.
- Unsupported LSP fields are omitted only under a documented projection-loss record.

## MCP projection

- Tools return structured content matching a fixed schema and exact service result references.
- Resources return exact immutable artifact/context/source-boundary representations under opaque URIs.
- Provider/result text remains untrusted candidate data.
- Tool result `isError` or protocol errors follow the frozen MCP profile; domain candidate/partial/blocker states are not misrepresented as transport success without status metadata.
- No resource/tool result becomes an instruction or permission grant.

## Projection loss

```text
ProtocolProjectionLossRecord
    service result field/item
    target protocol method/tool/resource field
    reason: Unsupported | CompactPresentation | PrivacyDenied | SizeBound | Deferred
    affected authority/coverage/completeness
    fallback/reference path when available
```

A projection that loses mandatory evidence/coverage/conflict/blocker state is invalid for that operation/profile.

## Errors

`E7AServiceError` includes:

```text
stable code
operation/stage/session/request IDs
exact view/overlay/document/result IDs known
owner/protocol/authorization error references
coverage/conflict/privacy/cancellation/effect/closure state
bounded structured arguments
recovery class
```

Errors exclude credentials, private roots, unrestricted source, hidden review/holdout labels, raw owner handles, transport secrets, and unbounded stack traces.

## Required nonclaims

Depending on operation:

```text
session-bound-not-global-current
unsaved-overlay-not-published-or-saved
static-analysis-not-runtime-proof
candidate-not-verified
search-rank-not-selection-or-lineage
advisory-action-not-applied
source-text-is-untrusted-data
protocol-capability-not-authorization
no-edit-tool-model-or-publication-authority
partial-or-truncated-results-not-complete-absence
```

## Diagnostic unchanged results

`NoChange`/unchanged diagnostic responses require exact proof that:

- request/profile/session view/overlay generation are compatible;
- relevant owner dependency/result manifest is unchanged;
- prior result ID/digest is retained;
- coverage/conflicts/authorization/privacy state is unchanged;
- no newly required capability was omitted.

Empty output or same document version alone is not proof.

## Telemetry

Operational telemetry may include:

- request/method/tool/resource kind;
- bounded stage/counters;
- duration/CPU/memory/queue/cache metrics;
- cancellation/backpressure/transport outcome;
- stable error/status codes;
- owner call counts and closure state.

Telemetry is noncanonical unless a specific count is part of a semantic result. It cannot affect ranking, confidence, authorization, or artifact identity.

## Telemetry privacy

Default telemetry excludes:

```text
source bodies/snippets
private absolute paths
symbol/query/document text unless explicitly safe and bounded
credentials/tokens/signatures
hidden holdout/review material
raw MCP arguments or LSP document content
provider/private URLs
owner/store handles
```

Stable pseudonymous IDs may be used only under a frozen privacy profile.

## Serialization

- canonical service JSON has frozen field/tag ordering and exact UTF-8 bytes;
- adapter framing is protocol-specific and separately identified;
- stdout/stderr/pipe/socket framing is not part of semantic owner identity;
- invalid/nonfinite numbers and duplicate JSON keys are rejected;
- source strings are structurally escaped;
- no locale/terminal/editor-dependent text in canonical output.

## Determinism

Equivalent exact inputs/profile yield the same service result IDs and canonical bytes independent of transport request ID, client name/version, worker scheduling, queue timing, cache history, host path, or progress frequency.
