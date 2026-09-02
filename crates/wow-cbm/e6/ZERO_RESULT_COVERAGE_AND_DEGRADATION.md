# E6-A zero results, coverage, and optional degradation

**Status:** normative.

## Zero-result classification

A zero candidate page/result means only:

```text
the selected provider transport returned no accepted candidates
for this exact descriptor/capability/state/query/profile/page
under the reported provider and bridge coverage, limits, and errors
```

It does not prove absence of an API, symbol, source entity, implementation, relation, call, trace, replacement, impact, bug, or relevant context.

## Result states

```text
CandidatesReturned
ZeroCandidatesReported
ZeroCandidatesAfterValidationLoss
PartialCandidates
TruncatedCandidates
ProviderUnavailable
OperationUnsupported
StateOpaqueOrStale
Conflict
NotEvaluated
Cancelled
Failed
```

`ZeroCandidatesAfterValidationLoss` distinguishes a provider response whose items were all rejected/unsupported from a genuine provider-reported zero.

## Negative authority

E6-A always sets `negative_authority=unavailable`. No descriptor, coverage field, provider label, stable generation, exhaustive flag, index status, zero count, or repeated query changes that rule.

Exact local owner negative authority can be obtained only through existing project/reference/search contracts, outside E6-A.

## Coverage axes

Keep separate:

```text
provider-declared corpus/scope coverage
external-state identity coverage
transport operation coverage
response schema/field coverage
candidate enumeration/page coverage
normalization/loss coverage
locator field coverage
privacy/license output coverage
continuation/cache coverage
cancellation/timeout coverage
```

A provider saying complete is a provider-declared field and does not automatically become bridge authority.

## Degradation

External lane failures never make exact local capabilities fail. The higher service can return exact local results with an external-lane omission/status such as:

```text
Unconfigured
Unavailable
Unsupported
Opaque
Stale
Partial
Truncated
Conflict
NotEvaluated
Cancelled
Failed
```

E6-A itself returns the exact external result/error; it does not invoke a fallback.

## No fallback

Forbidden hidden behavior:

- query another provider;
- reuse stale cache as current;
- call a model or web search;
- run local `wow-search`;
- broaden repository scope;
- remove filters;
- reset budgets;
- substitute another external generation.

A caller may explicitly create another request; it is a distinct operation/result.

## Capability status

`wow-cbm` status is external-lane-local. It cannot downgrade ReferenceView/project/graph/search/context/rules state. Conversely, successful external candidate retrieval cannot upgrade those exact local capabilities.

## Partial and truncation

When provider/bridge limits stop enumeration, preserve continuation if valid, exact visited/returned counts where available, omitted/rejected items, loss reasons, cumulative budgets, and completeness impact. A partial or truncated zero cannot be rendered as no candidates globally.

## Conflict

Conflicting provider state, duplicate incompatible candidate IDs, score semantics, locators, coverage, or pagination metadata remains explicit. No newest/majority/first-value resolution.

## Nonclaims

E6-A does not evaluate provider recall/precision globally, claim indexing completeness, guarantee source freshness, or infer that the external system is unnecessary when it returns zero.