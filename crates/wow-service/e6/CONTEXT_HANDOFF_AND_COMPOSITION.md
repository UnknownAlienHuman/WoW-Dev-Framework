# E6-B exact-root context handoff and result composition

**Status:** normative.

## Preconditions

`external_candidate_context_build` requires:

```text
one exact validated external result/candidate
one retained ExactMapped mapping record
one retained Selected receipt for the declared context use
exact project/reference/graph publication selectors and guards
one exact existing E3 context operation/request/profile
provider sidecar privacy/license/output policy
OperationId + CanonicalRequestDigest
```

No mapping or selection is inferred during context build.

## Acquisition

Service resolves any permitted outer current selectors exactly once, then reacquires the exact retained owner views referenced by the mapping/selection. It validates:

```text
mapping owner generation still available and byte-identical
selected mapped handle belongs to the acquired owner view
project/reference/graph generations are compatible
E3 context capabilities and profiles are available
privacy/license scope permits exact source/context and sidecar output
no mapping/selection/configuration revocation or conflict
```

## Invocation

Service reuses the internal E3 acquisition/use-case primitives and invokes exactly one `wow-context` operation with the exact mapped root. It does not call the E3 public service API recursively.

The E3 request can be map, inspect, build, continue, validate, or render only when that operation accepts the exact root/profile and the E6-B command maps to it explicitly.

## Evidence separation

Result composition has two independent components:

```text
ContextService semantic/rendered artifacts
ExternalCandidateSidecar
```

The context artifact contains only normal exact project/reference/graph evidence under E3 contracts. The sidecar contains provider/result/candidate/query/state/rank/score/locator/snippet/summary fields allowed by policy, plus mapping and selection receipts.

Provider fields never become context facts, graph edges, source excerpts, API contracts, lineage, replacement, impact, or remediation. Exact mapping can annotate which locator fields matched the owner record, but it does not validate provider interpretation.

## Context root

The root is the exact owner handle from the mapping record. Service cannot substitute a parent, nearest symbol, path, repository root, search result, top candidate, or current generation.

If the mapped entity is not accepted by the requested E3 root profile, context build is `NotEvaluated`/blocked; service does not broaden it automatically.

## Continuation

External-candidate context continuation binds both:

```text
exact E3 context continuation and retained universe
exact external result/mapping/selection/sidecar policy
```

It cannot refresh provider state, change candidate/mapping/selection, reset context or external budgets, or switch owner generations.

## Status composition

External and context lanes retain independent statuses. Conservative outer precedence:

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Truncated
Partial
Complete
NoChange
```

A complete exact context artifact can coexist with an omitted/redacted sidecar. A valid sidecar cannot make failed/partial context complete. Provider unavailability after a retained result does not invalidate that historical result, but freshness/nonreproducibility remains visible.

## Publication and retention

Context result publication, sidecar publication, and combined envelope publication use distinct immutable records/receipts. Retain exact E3 artifacts and owner leases plus result/mapping/selection evidence before advertising a durable combined handle.

## Cancellation and close

Cancellation propagates to context owners and prevents new sidecar/render work. It does not erase existing result/mapping/selection records. Close all context, project, reference, graph, store, provider-session, retention, and audit resources in reverse order before public success.

## Nonclaims

Combined context does not prove external provider correctness, global source coverage, runtime behavior, replacement/migration safety, absence of relevant alternatives, edit authorization, or production recognizer eligibility.