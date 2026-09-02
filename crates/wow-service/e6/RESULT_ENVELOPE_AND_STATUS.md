# E6-B result envelope and status model

**Status:** normative.

Every public operation returns one tagged `ExternalCandidateServiceResultEnvelope` containing:

```text
operation/request IDs and operation kind
configuration/descriptor/authorization/session/state/query IDs
result/artifact/continuation IDs
mapping/selection/context/sidecar IDs
operation-specific payload
conservative outer status
provider and exact-local lane statuses
Candidate authority and source-verification state
coverage/zero/partial/truncation/conflict/loss/omission records
budgets/quota/cancellation/response-loss/reconciliation
privacy/license/redaction/retention/audit/close state
mandatory nonclaims
canonical digest
```

## Outer statuses

```text
Complete
NoChange
CandidateOnly
ExactMapped
SelectionRecorded
ContextReady
Partial
Truncated
Blocked
ConflictBlocked
OutcomeUnknown
NotEvaluated
Cancelled
Failed
```

Default conservative precedence:

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Truncated
Partial
ContextReady
SelectionRecorded
ExactMapped
CandidateOnly
NoChange
Complete
```

Operation-specific payloads remain authoritative. A completed validation may contain `Invalid`; a completed selection-record operation may contain `Rejected` or `Deferred`; a query with zero candidates remains `CandidateOnly` or `Complete` only with an explicit scoped-zero payload and no negative-authority claim.

## Lane separation

The envelope separately reports:

```text
external provider/configuration/session/query lane
result publication/validation lane
project/reference mapping lane
selection lane
exact context lane
```

Provider failure cannot mark the exact local lane failed when local work was not requested or remains independently available. Exact context success cannot hide external-sidecar omission/conflict.

## Required status wording

- `CandidateOnly` must not say verified, authoritative, mapped, selected, or exact source.
- `ExactMapped` must state that only locator identity was mapped.
- `SelectionRecorded` must expose `Selected`, `Rejected`, or `Deferred` and state that selection is not verification.
- `ContextReady` must state the exact project/reference/graph generations and keep external sidecar evidence separate.
- `OutcomeUnknown` must state unsafe-to-retry and provide exact reconciliation IDs.
- Partial/truncated/multiple/conflict/`NotEvaluated` states cannot be hidden in warnings.

## NoChange

Requires exact owner proof of an already-existing identical operation/effect and retained state. Same provider/query text, candidate count, result digest, mapped path, or selected candidate is insufficient without operation/request closure.

## Nonclaims

Include as applicable:

```text
external-candidate-not-source-proof
provider-score-not-framework-confidence
zero-result-not-negative-authority
mapping-validates-locator-identity-only
selection-is-not-verification-or-edit-authorization
context-does-not-validate-provider-interpretation
provider-failure-does-not-degrade-local-capability
not-runtime-verified
not-core-pack-admitted
not-publicly-distributed
```

## Redaction

Default envelopes and errors redact credentials, private endpoints, provider cursor bytes, process/client handles, private source, local paths, account identifiers, and confidential notes. Exact digest/ID relationships remain available where policy permits.