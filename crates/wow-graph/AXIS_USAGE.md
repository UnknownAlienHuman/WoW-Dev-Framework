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

`bind` preserves the legacy single-definition recipes. For a source graph, prepare
the service request with `wow_service::graph::source_graph_axis_profile` against
the artifact's exact registry. The service independently reconstructs that same
recipe and checks the query's profile digest. Native graph callers can use
`GraphAxisProfile::bind_definitions(registry, axis, &["exact_definition_id", ...])`
to declare their reviewed selection explicitly.

## Reviewed projections

Forward is axis direction, not a rewrite of the physical edge. Reverse inverts
all listed directions; Both admits either direction independently per family.

| Axis | Stored relation families in forward traversal | Shape |
|---|---|---|
| Ownership | `Owns` outgoing: owner to owned | Multi-parent |
| Load | `Loads` outgoing, `DependsOn` incoming; registries with extended families also require `LoadsBefore` outgoing and `OptionalDependsOn` incoming | Network |
| Inheritance | `Inherits`, `MixesIn` incoming: base/mixin to derived | Multi-parent |
| Registration | Nine existing native-event, custom-signal, CVar, script and hook families, outgoing | Network |
| Lifecycle | `FactoryCreates`, `Instantiates` outgoing | Network; static creation/instantiation slice only |
| State | `ReadsState`, `WritesState` outgoing | Network |
| Call | `Calls` outgoing; `UsesApi` is not assumed to be a call | Network |
| Object | `ParentOf` outgoing, explicit source parentage | Multi-parent |
| Lexical | Unsupported; `Contains`/`Defines` do not silently acquire named-axis meaning | No substituted semantics |

Every required family must be represented. Legacy `bind` still requires exactly
one definition per family. Explicit `/4` profiles admit several exact IDs for a
family, in canonical ID order. Unknown, duplicate or out-of-family selections
reject. Registry digest, direction, endpoint kinds and confidence constraints
remain exact. Roots must exist and have a registered entity kind.

For `/4`, each retained accepted relation joins its original proposal ID to its
exact registry definition, then uses the publication owner's edge constructor.
One materialized edge may have several selected `(edge_id, relation_id)` bindings;
`edge_definitions()` preserves them all. Selecting one definition does not select
another with the same endpoints, enum, confidence and evidence. No filtered
snapshot or inferred inverse edge is created. Foundation edges without retained
producer proposals require one unique registered definition; ambiguity rejects.

The canonical source recipe is owned by `wow-project/src/graph/axes.rs`. Ownership
excludes `xml_occurrence_source_span` and `xml_map_piece_source_span`: those are
source associations, available through original proposals/evidence and ordinary
graph reads. An unknown definition in a source axis family rejects rather than
widening its meaning. `UsesApi` remains outside Call.

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
confidence and limits. Execution validates canonical families/directions and the
registry binding; service execution also reconstructs the independently owned
reviewed selection. A caller's alternate self-consistent profile cannot replace
the service recipe because its query digest must name that recipe exactly.
Source comments and addon configuration never register new axes.

Single-definition axis profiles `/1`, `/2` (extended Load) and `/3` (Object) keep
their bytes and identities. Explicit selection uses axis-profile `/4`, axis
query/result `/2`, and nested subgraph query/result `/3`. The nested query retains
the profile digest and can execute only inside the retained axis owner; passing
it to ordinary subgraph execution rejects. Ordinary uniform `/1` and directed
`/2` subgraph reads continue to select broad enums explicitly.

GraphEdge, registry, proposal/report, materialized snapshot, partition and store
schemas do not change. Existing retained batches supply the definition identity
on reopen; no new validation dialect or persisted sidecar is introduced. The
new versions describe read-only query interpretation, not a producer ladder.

## Bounds and authority

Confidence defaults to Proven/Derived; Possible/Candidate require explicit opt-in.
The original confidence/evidence is retained, never promoted. The snapshot's
node/edge/evidence limits and subgraph limits still apply. `max_scanned_edges`
bounds accepted-assertion association and adjacency indexing together; the result
exposes both counters and `scanned_edges()` returns their sum. Existing owner
validation/rebuild is separately snapshot-bounded, not counted as query scans.
Cancellation is checked throughout admission and the shared traversal.

The canonical output limit includes the entire profile, query and nested result.
Definition association metadata has an 8-MiB encoded ceiling. Each admitted
edge's binding bytes are charged atomically with its edge/node in the shared BFS.
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
For definition-selected queries the nested absence flag is also always false:
existing coverage records describe broad enums rather than individual definitions.
No source/runtime safety or full E2 acceptance follows from a complete traversal.

Implementation: `src/axes.rs`, `src/axes/profile.rs`, shared `src/subgraph/`.
Contract: `e2/AXES_AND_VIEWS.md`, `e2/QUERY_MODEL.md`, `e2/SECURITY_AND_BUDGETS.md`.
