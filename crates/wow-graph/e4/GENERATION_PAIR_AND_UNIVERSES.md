# E4-B generation pairs and lineage universes

**Status:** normative exact comparison boundary.

## Lineage universe classes

E4-B compares two exact generations only within one universe class:

```text
UserProject
BlizzardUiSource
ReferencePlatformContract
```

A `UserProject` comparison binds the same exact logical project identity under two retained ProjectStore publications. A `BlizzardUiSource` comparison binds the same source-universe profile under two exact source publications. A `ReferencePlatformContract` comparison binds explicit compatible ReferenceProfiles/Generations under a reviewed comparison profile.

## Not lineage

The following are not cross-generation lineage pairs:

```text
user project entity -> ReferenceView API entity
user project entity -> Blizzard UI implementation entity
Blizzard UI implementation entity -> ReferenceView API entity
dependency entity -> first-party project entity merely because names match
runtime observation -> static entity
external candidate -> project entity
```

They can have generation-local bridge, use, implementation, observation or candidate relations under their owning contracts, but not `same_lineage_as` by convenience.

## Exact generation binding

```text
GenerationComparisonSet
    comparison ID
    universe class and logical universe ID
    exact before owner publication/generation/snapshot/view
    exact before graph generation/snapshot/view
    exact after owner publication/generation/snapshot/view
    exact after graph generation/snapshot/view
    exact source/reference/analyzer/profile identities as applicable
    optional exact before/after SearchShard/View IDs
    comparison/profile compatibility report
    coverage/conflict/capability manifests
```

Every view is retained and immutable for the operation. `before` and `after` are explicit roles, not inferred from timestamps, branch names, semantic versions or provider chronology.

## Ordering and direction

Directional relation semantics use the exact comparison order:

```text
source/old/before -> target/new/after
```

A comparison can be reversed only by a new canonical request and distinct result identity. Symmetric queries over `same_lineage_as` still retain the original before/after assertion scope.

## Project comparison compatibility

Validate:

- same exact logical ProjectId/universe or an explicit reviewed project-identity bridge;
- compatible source-root/project schema/canonicalization profiles;
- exact before/after project and graph publications;
- exact analyzer/recognizer/graph registries and their compatibility reports;
- exact selected TOC/product flavor/profile for both generations;
- complete source/inventory/TOC/XML/analyzer/recognizer/graph coverage needed by the requested lineage/change capabilities;
- no mixed entity rows from another generation.

A repository rename or move can coexist with the same logical project identity only when the owner publication explicitly preserves or reviews that project identity. Repository URL/name alone does not establish continuity.

## Blizzard UI comparison compatibility

Validate:

- same `blizzard_ui_source` universe/profile class;
- explicit before/after product, flavor, channel, build and source materialization identities;
- exact source roots/package discovery/TOC/XML/analyzer/recognizer/graph profiles;
- explicit compatibility policy for profile changes;
- source/license/coverage state for both generations;
- no Retail/PTR/Beta/Classic filling or nearest-build substitution.

A source mirror commit order is provenance, not proof of client build order or exact continuity.

## Reference comparison compatibility

Validate:

- explicit before/after ReferenceProfile and ReferenceGeneration IDs;
- product/flavor/channel/build/Interface comparison policy;
- reference schema/evaluator/correction/coverage compatibility;
- exact transition/deprecation/replacement/correction records;
- no current/latest/nearest-profile fallback;
- source and correction conflicts retained.

Reference transition evidence is scoped to the exact profiles and cannot be extrapolated to another product/build without a separate comparison.

## Optional search shards

E4-A SearchShard inputs can assist candidate generation only when:

- each shard binds exactly the corresponding before or after owner generation;
- document/normalization/query/lane/ranking profiles are explicitly compatible;
- retained shard validation reports pass;
- no prior/current/other-generation corpus leakage exists;
- search results preserve exact signal/evidence/coverage/conflict state.

Missing search shards do not block owner-explicit lineage evidence, but they can reduce candidate-generation coverage. Search cannot fill missing owner facts.

## Current selection

`wow-graph` accepts exact generation bindings only. E4-C service may later support symbolic current selectors by resolving and retaining exact before/after publications once before invoking E4-B.

No E4-B operation:

- refreshes current;
- swaps after generation during pagination;
- chooses last-known-good automatically;
- silently substitutes a close build;
- retries with a different generation;
- compares moving live views.

## Comparison set states

```text
ValidComplete
ValidPartial
ConflictBlocked
NotEvaluated
Invalid
Cancelled
Failed
```

A partial comparison is allowed only when the requested operation/profile permits it and every affected capability is explicit. Partial cannot produce exact removal/introduction or other conclusions requiring closed coverage.

## Multiple generation sequences

E4-B v1 compares one ordered generation pair per canonical operation. A multi-hop lineage query can traverse assertions from multiple already-published pair overlays, but it must validate compatibility and retain every intermediate generation/assertion.

A direct A→C lineage assertion is not inferred solely from A→B and B→C paths. Transitive reachability remains a path unless the relation/profile explicitly allows a separately derived assertion with all evidence and blockers.

## No-change comparisons

If before and after owner/graph/source/reference inputs and all relevant profile/producer manifests are identical, the comparison may return canonical `NoChange` without publishing a new lineage generation. It cannot use mtime, branch label, provider event or cache claim alone.

## Retention

Every comparison operation requires retention/lease closure for:

- before and after owner publications;
- before and after graph snapshots;
- before and after Reference/Search shards when used;
- all source/evidence records needed by proposals/assertions/review/change/impact results;
- the LineageGraphSnapshot and active continuation chains.

GC loss is an exact failure; no fallback to current or reconstructed same-name entities.
