# E6-B retention, cache validation, continuation, and optional degradation

**Status:** normative.

## Retention closure

Before advertising a durable result, artifact, mapping receipt, selection receipt, continuation, audit handle, or context handoff, service obtains exact retention receipts for every referenced object:

```text
provider descriptor/capability/external-state receipt
E6-A query/result/artifact/explanation/cache record
exact owner publication/view/generation used for mapping
mapping receipt and mapped root
selection receipt
context request/result when present
operation/effect/reconciliation records
audit chain and privacy/license decisions
```

A handle is not returned if any mandatory referenced artifact can be collected immediately.

## Retention owner

`ExternalCandidateRetentionPort` supports exact artifact/reference-set admission, validation, renewal where the profile permits, release, and reconciliation. It does not expose raw database/filesystem operations.

Process-local memory is not sufficient proof of retention.

## GC race

Catalog lookup and retention admission use an owner operation that closes the race:

```text
lookup exact artifact
-> validate digest/state
-> atomically admit retention or fail because unavailable
```

If an artifact disappears before retention closes, service returns unavailable/blocked and does not substitute another generation/result.

## Cache boundary

`wow-cbm` owns the logical external candidate cache-entry contract. `wow-service` coordinates exact catalog/store retrieval and asks E6-A to validate the entry.

A cache entry binds:

```text
provider descriptor/adapter/session profile
external-state class and exact receipt
effect/query/result/profile identities
privacy/license/consumer scope
continuation state and cumulative budgets
artifact digest and retention state
```

## Cache nonclaims

A cache hit does not prove:

- provider freshness;
- same mutable/opaque external state;
- source or platform correctness;
- negative authority;
- selection or mapping validity for a different owner generation;
- runtime behavior;
- permission to disclose under a broader consumer profile.

## Cache outcomes

```text
ValidExactHit
Miss
StaleOrDifferentState
PrivacyOrLicenseMismatch
ArtifactUnavailable
Conflict
NotEvaluated
Failed
```

`Miss` does not trigger hidden provider fallback unless the exact operation request asks for a provider query.

## Continuation

Continuation is delegated to E6-A and wrapped by service durable/retention state. It binds:

- exact provider/session/external-state receipt;
- normalized query and profiles;
- result-set/page chain;
- protected provider cursor reference;
- cumulative provider/query/output budgets;
- privacy/license/consumer profile;
- operation and retention state.

It cannot refresh external state, reset budgets, change provider, broaden privacy, select candidates, map locators, or start context.

## External state retention

- Stable generation: retained generation descriptor/artifact must remain available.
- Observed mutable state: exact observation receipt remains available; later provider state is not a substitute.
- Opaque state: result can be retained as an opaque Candidate artifact but cannot claim reproducible continuation/cache semantics beyond the exact profile.

## Optional degradation

Provider features are optional. Explicit degradation states include:

```text
ProviderNotConfigured
ProviderUnauthorized
ProviderUnavailable
ProviderIncompatible
CapabilityUnavailable
ExternalStateUnstableOrOpaque
QueryPartialOrTruncated
ResultValidationFailed
MappingUnavailable
ContextUnavailable
PrivacyOrLicenseBlocked
OutcomeUnknown
```

Exact local operations that do not require E6-B continue normally. Service does not describe the entire framework as failed.

## No hidden fallback

Forbidden on failure/miss:

```text
another provider
another provider generation
new current observation
stale cache
local E4 search as an external-provider substitute
model/embedding/CBM implementation other than the requested descriptor
provider snippet as context replacement
name/path heuristic mapping
```

A caller can submit a new explicit request with a new operation ID.

## Provider/result listing

List/get operations use immutable catalog snapshots, deterministic ordering, bounded pages, and snapshot-bound continuation. They do not sort by score/newest/best by default or imply catalog completeness when coverage is partial.

## Deactivation and deletion

E6-B v1 does not mutate provider state. Local retention release/GC follows `wow-store` owner contracts and cannot delete provider databases or indexes. Historical audit/operation records remain according to policy.

Any future privacy deletion workflow requires a separate contract proving local/provider/cache/backups/continuation anti-resurrection closure.

## Cancellation

Cancellation closes sessions, views, retention attempts, and output resources synchronously. Already committed artifacts remain under exact state. No background continuation. If provider effect status is uncertain, result remains `OutcomeUnknown`.

## Tests

- retain complete result/mapping/selection/context closure;
- GC race before retention;
- cache hit with exact state;
- mutable-state cache substitution;
- privacy/profile cache widening;
- continuation budget reset/provider switch;
- provider unavailable while local context works;
- forbidden hidden fallback;
- opaque-state continuation claim;
- cancellation/close failures;
- artifact list snapshot changes mid-page.
