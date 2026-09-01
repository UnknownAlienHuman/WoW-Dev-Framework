# E4-C selectors, exact acquisition, and compatibility

**Status:** normative.

## Selector principle

Owner operations use exact retained identities. Symbolic selectors exist only at the service boundary and are resolved to exact IDs before canonical owner invocation.

## Search acquisition

A search operation can bind:

```text
primary user project publication
optional Blizzard UI source publication
exact Reference publication
exact compatible validated SearchShard for each included universe
```

The service first resolves/acquires owner publications and then resolves shards by exact owner generation plus exact `SearchProfileSetId`, unless an exact shard ID is supplied.

A missing shard returns `SearchShardUnavailable` or `NotEvaluated`. Query never triggers indexing.

## Lineage acquisition

A lineage build/query binds one exact compatible comparison class:

```text
user project before -> after
Blizzard UI source before -> after
Reference generation before -> after
```

Project-to-Reference or project-to-Blizzard pairs are bridge/use relationships, not lineage comparisons.

Before and after remain distinct even when their display metadata matches.

## Current selectors

Approved aliases:

```text
CurrentPublished project/platform publication
CurrentCompatible Reference publication
```

A current selector is resolved once per acquisition attempt. The resolved canonical request contains exact IDs and an observation receipt.

There is no current selector for an arbitrary SearchShard or LineageGraphSnapshot. Those are found only by exact shard/snapshot ID or by deterministic exact owner-generation/comparison lookup.

## Stable-double-collect

When a request requires stable current across independent mutable owners:

```text
collect current owner records in canonical order
-> resolve exact intended owner generations
-> acquire exact retained views/leases
-> resolve exact compatible shards/reference views
-> validate compatibility
-> collect the same current owner records again in the same order
-> accept only when the canonical record set is unchanged
-> otherwise close all resources and retry within the frozen finite limit
```

Reported proof:

```text
StableAcrossAcquisitionCollects
```

It does not claim a distributed atomic snapshot or that the selections remain current after the response.

Exact selectors do not need this loop.

## Fixed acquisition order

Canonical order for the superset E4-C path:

```text
1 primary project before/current publication
2 primary project after publication when distinct
3 optional platform before publication
4 optional platform after publication when distinct
5 Reference before publication/view
6 Reference after publication/view when distinct
7 exact SearchShard views ordered by universe then generation
8 exact LineageGraphView when querying/reviewing
9 context invocation resources when search_context is requested
10 retention/idempotency resources needed by the operation
```

Acquire only required entries, preserving relative order. Release in exact reverse order.

## Compatibility checks

Validate:

- exact owner store/publication/project/graph/analyzer/reference generations;
- product/flavor/channel/build/Interface/Profile compatibility;
- before/after comparison class and ordering profile;
- SearchShard source binding, document/query/ranking profile, validation/seal state and owner generation;
- LineageUniverseSet, comparison, relation/proof/review/migration/impact profile compatibility;
- graph registries and entity keys;
- source/privacy/license/consumer trust state;
- required capabilities/coverage/conflicts/truncation;
- retained generation and continuation availability.

No date, commit chronology, semantic version string or name similarity substitutes for an explicit compatibility record.

## Exact catalog lookup

`ForExactOwnerGeneration` or `ForExactComparison` lookup returns:

```text
UniqueEligible(exact artifact ID)
None
MultipleConflicting(exact artifact IDs)
NotEvaluated
Failed
```

The service never picks newest, first, smallest, highest version or last-written from multiple artifacts.

## Failure isolation

- acquisition failure closes previously acquired resources;
- compatibility failure invokes no owner algorithm beyond read/validation needed to classify it;
- current churn retries the full acquisition attempt, not individual handles;
- retry exhaustion returns typed `AcquisitionUnstable`;
- no partial view set enters a complete result;
- no close failure is downgraded to warning.

## Retention

Before returning a continuation or durable result handle, service obtains exact retention receipts for all required publications, shards, lineage snapshots and context artifacts under the operation profile.

If retention cannot be admitted, continuation is unavailable; the operation does not advertise a cursor that cannot be reopened.

## Security

Selectors are strict typed IDs, not paths, URLs, SQL, regex or source text. Errors/logs use stable IDs and structured reasons, not private roots, raw credentials, source bodies or review signatures.
