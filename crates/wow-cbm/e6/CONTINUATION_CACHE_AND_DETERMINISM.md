# E6-A continuation, cache validation, and determinism

**Status:** normative.

## Continuation binding

An external continuation binds:

```text
provider descriptor/capability set
adapter/transport profile
exact ExternalStateBinding
normalized query and output profiles
prior result manifest and ordering
protected provider cursor reference/digest
cumulative item/byte/time/memory budgets
privacy/license state
last stable bridge ordering key
integrity digest
```

Continuation cannot switch provider/state/schema/query/profile, broaden scope, change filters, reset budgets, hide prior truncation/loss, or expose raw provider cursor bytes.

## State-specific continuation

- `StableExternalGeneration`: continuation allowed when provider/profile proves cursor remains within that generation.
- `ObservedMutableGeneration`: allowed only within the exact observation/session binding.
- `OpaqueExternalState`: disabled by default; if allowed, limited to the same exact opaque episode and labeled nonreproducible.

If exact state is unavailable, return stale/`NotEvaluated`; never refresh silently.

## Cache key

```text
provider descriptor/capability/state IDs
normalized query
adapter/normalization/loss/score profiles
privacy/license/output profile
budget profile and relevant hard limits
implementation contract version
```

A cache hit is valid only if exact key, artifact bytes/digest/schema, state accessibility, and privacy/license profile match.

## Cache authority

Cache preserves original Candidate authority, state class, staleness, coverage, loss, conflicts, and zero-result classification. It cannot turn opaque into stable, stale into fresh, partial into complete, or candidate into verified.

## Cache storage

E6-A defines cache entry validation only. Physical storage, retention, GC, encryption, and durable operation ownership belong to E6-B/service/store. E6-A accepts no filesystem path, database handle, or arbitrary cache callback.

## Deterministic ordering

Canonical bridge ordering uses reviewed fields and stable candidate IDs after retaining provider-local rank metadata. It never depends on map/DB iteration, arrival order, worker completion, clock, network latency, cache state, host, or process.

## Rebuild comparison

Equivalent exact descriptor/capability/state/query/response bytes and profiles produce identical normalized candidates, ordering, loss records, coverage, explanations, result/artifact IDs, and canonical bytes under 1/2/N workers and shuffled response field/item order where the provider schema declares order nonsemantic.

Provider-semantic order is preserved when the descriptor declares it meaningful. Reordering then changes exact raw/result identity but not authority.

## Opaque-state determinism

E6-A can deterministically normalize one captured opaque response. It cannot claim repeated live provider queries return the same response.

## Operational metadata

Network duration, retries, transport instance, host/process IDs, cache hit/miss, and logs are noncanonical. E6-B may audit them separately.

## Corruption/mismatch

Digest/schema/profile/state mismatch rejects the entry. E6-A never repairs cached bytes under the same ID. Recompute using a new exact live request when the caller explicitly requests and E6-B authorizes it.