# E6-A external state and generation classes

**Status:** normative.

## Stable external generation

Requires an immutable provider-issued generation/index/corpus identity with validation sufficient to bind repeated queries to the same logical state:

```text
provider descriptor/adapter identity
stable generation ID
scope/corpus/repository identity as provider metadata
content/index digest or equivalent immutable receipt
schema/profile identity
coverage/staleness evidence
```

A revision string alone is insufficient if the provider can mutate results under that revision without an immutable index identity.

Claims allowed: exact replay/cache/continuation only to the degree the provider contract proves. Candidate authority remains unchanged.

## Observed mutable generation

Used when the provider exposes mutable state but one session/observation receipt can bind a query episode:

```text
observation/session receipt ID
provider state token/version when available
acquisition and capability-set identity
explicit mutable-state classification
```

Repeated later queries are not assumed to use the same state. Continuation is valid only if the transport proves it remains within the same observed state/session.

## Opaque external state

Used when no stable or observed state identity is available. It supports only explicitly nonreproducible one-shot discovery under narrow limits.

Restrictions:

- no exact replay claim;
- no long-lived semantic cache claim;
- no deterministic cross-run comparison claim;
- no continuation unless provider cursor itself proves same opaque episode and profile permits;
- no freshness or completeness claim;
- result state and nonreproducibility remain visible.

## Invalid substitutes

The following do not create generation identity:

```text
wall-clock timestamp
provider uptime/session duration
same repository name
same top result
same result count
same query text
successful health check
provider says current/latest
local cache hit
```

## State compatibility

Every query, result, explanation, continuation, comparison, and cache entry binds one exact state record. Cross-state comparisons require an explicit compatible comparison profile and remain descriptive.

## Staleness

Staleness is a separate axis. A stable generation can be old but exact; a mutable observation can be recent but nonreproducible. Neither age nor exactness changes Candidate authority.

## Conflicts

Conflicting provider generation/scope/digest metadata creates an explicit conflict and blocks claims dependent on exact state. E6-A never resolves conflict by newest timestamp, majority fields, or provider assertion priority.

## State loss

If a continuation/result references a state no longer accessible, return stale/unavailable/`NotEvaluated`; do not bind it to a newer provider state.