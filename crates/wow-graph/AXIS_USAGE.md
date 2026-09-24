# Registry-bound graph axes

`GraphAxisProfile` and `GraphAxisQuery` implement named, bounded views over an
existing `GraphPartitionSnapshot`. They reuse the subgraph BFS; no source,
provider, store, Lua session or background worker is opened.

```rust,ignore
let profile = GraphAxisProfile::bind(owner.registry(), GraphAxis::Load)?;
let query = GraphAxisQuery::new(
    owner.snapshot().snapshot_id().clone(),
    &profile,
    vec![root_id],
    GraphAxisTraversal::Forward,
    GraphSubgraphLimits::default(),
)?;
let view = query.execute(&owner, &profile, &cancelled)?;
// Inspect view.projection(): original nodes, edges, witnesses, coverage and limits.
// Inspect view.boundaries() before interpreting the named axis more broadly.
```

## Reviewed projections

Forward is axis direction, not a rewrite of the physical edge. Reverse inverts
all listed directions; Both admits either direction independently per family.

| Axis | Stored relation families in forward traversal | Shape |
|---|---|---|
| Ownership | `Owns` outgoing: owner to owned | Multi-parent |
| Load | `Loads` outgoing, `DependsOn` incoming: loader/prerequisite to consumer | Network |
| Inheritance | `Inherits`, `MixesIn` incoming: base/mixin to derived | Multi-parent |
| Registration | Nine existing native-event, custom-signal, CVar, script and hook families, outgoing | Network |
| Lifecycle | `FactoryCreates` outgoing | Network; creation slice only |
| State | `ReadsState`, `WritesState` outgoing | Network |
| Call | `Calls` outgoing; `UsesApi` is not assumed to be a call | Network |
| Lexical / Object | Unsupported: `contains`/`declares`/`parent_of` have no current stored kind | No substituted semantics |

Every required family must have exactly one definition in the selected registry.
Missing definitions return `AxisUnsupported`, not an empty complete axis. Multiple
definition IDs sharing one stored relation enum return `AxisProfileInvalid`:
materialized edges cannot distinguish those meanings. The selected registry's
endpoint and confidence constraints are checked for all selected stored edges,
including foundation edges. Roots must exist and have a registered entity kind.

`Parents` and `Children` accept only a multi-parent profile and `max_depth = 1`.
They return every admitted parent/child, never choose a single winner. Network
axes reject these forms. Longer forward/reverse neighborhoods enumerate branches;
there is no unique-parent chain, root discovery or flattened runtime order.
Cycles retain original edges and bounded visited state; they are not diagnosed as
runtime conflicts by this view. A discovery edge remains a traversal witness.

## Identity and compatibility

Profiles are immutable query sidecars, identified by the reviewed recipe version,
axis, exact registry digest, shape, ordering, cycle policy and resolved relation
IDs/directions. Query identity also binds the exact snapshot, roots, traversal,
confidence and limits. Execution rebuilds the profile from the reviewed recipe;
recomputed hashes on altered deserialized definitions cannot redefine its meaning.
Source comments and addon configuration never register new axes.

The inner request uses `GraphSubgraphQuery::new_directed` and the new v2 subgraph
query/result profile with an explicit direction for every selected relation.
Uniform-direction subgraph queries retain their existing v1 bytes and hashes.
Graph registry, assertion and snapshot identities are unchanged. Persistence of
axis profiles and public service/CLI routing remain separate work.

## Bounds and authority

Confidence defaults to Proven/Derived; Possible/Candidate require explicit opt-in.
The original confidence/evidence is retained, never promoted. The snapshot's
node/edge/evidence limits and subgraph limits still apply. `max_scanned_edges`
bounds both registry-admission and adjacency-indexing passes together; the result
exposes both counters and `scanned_edges()` returns their sum. Existing owner
validation/rebuild is separately snapshot-bounded, not counted as query scans.
Cancellation is checked throughout admission and the shared traversal.

The canonical output limit includes the entire profile, query and nested result.
Header bytes are reserved before traversal; the inner budget must retain at least
16 KiB, otherwise execution returns `BudgetExceeded`. No evidence is removed to
fit. Depth/node/edge/expansion/byte truncations retain their original reasons.
As with subgraphs, an admitted edge beyond the depth boundary marks `Depth` even
for one-level parent/child views; the result never claims exhaustive ancestry.

`state()` describes the exact stored-family projection. Full named-axis absence
is always non-authoritative: conflict assessment, external evidence bodies and
relation families not represented in the current schema remain explicit result
boundaries. The nested projection's narrower absence flag applies only to its
exact stored relation/direction/confidence request, not to the whole named axis.
No source/runtime safety or full E2 acceptance follows from a complete traversal.

Implementation: `src/axes.rs`, `src/axes/profile.rs`, shared `src/subgraph/`.
Contract: `e2/AXES_AND_VIEWS.md`, `e2/QUERY_MODEL.md`, `e2/SECURITY_AND_BUDGETS.md`.
