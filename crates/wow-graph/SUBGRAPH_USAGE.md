# Bounded snapshot neighborhoods

`GraphSubgraphQuery` implements the E2-A `project_subgraph` owner operation over
an existing immutable `GraphSnapshot`. No source, storage or provider is opened.
It also accepts the materialized `snapshot()` of a `GraphPartitionSnapshot`;
producer partitions and their assertion reports remain unchanged and separate.

```rust,ignore
let query = GraphSubgraphQuery::new(
    snapshot.snapshot_id().clone(),
    vec![root_id],
    GraphDirection::Outgoing,
    vec![GraphRelationKind::Calls, GraphRelationKind::UsesApi],
    GraphSubgraphLimits::default(),
)?;
let result = query.execute(&snapshot, &cancelled)?;
// Original node/edge identities and evidence are retained in the result.
// Inspect state(), coverage(), missing_coverage() and truncations().
```

## Projection semantics

The request requires an exact snapshot, 1–64 distinct existing roots, a direction,
a nonempty relation whitelist, confidence ceiling and bounded limits. The snapshot
identity binds its universe and generation; both are returned explicitly. Roots
and relations are canonicalized by the constructor; duplicate or noncanonical
serialized requests reject on execution. Missing roots are query errors, not
proof of authoritative absence.

Multi-source breadth-first traversal discovers the shortest admitted distance
from any root. Roots use node-ID order and each adjacency uses edge-ID order.
Nodes and edges are returned once, in their own ID order; a non-root additionally
retains its first canonical discovery edge. This is a traversal witness, not a
new parent/ownership relation. Cycles, cross edges and original direction are
retained without recursive traversal, path enumeration or transitive edges.

The depth bound limits discovery of new nodes. An edge between already reached
nodes is still retained at the depth boundary. An admitted edge to an unreached
node beyond the boundary marks `Depth`, even when the rest of the bounded ball
is available. Therefore a radius-limited result is never confused with exhaustive
reachable closure. Root-only depth zero is supported under the same rule.

The default admits Proven and Derived edges. `GraphSubgraphConfidence` aliases
the existing `GraphPathConfidence`; Possible and Candidate are explicit opt-ins.
Confidence and evidence on every original edge are unchanged. The result is not
a replacement graph snapshot and cannot publish projected truth into the store.

## Bounds and stopping

| Limit | Default | Hard range |
|---|---:|---:|
| Discovery depth | 4 | 0–64 |
| Returned nodes, including roots | 256 | 1–16,384 |
| Returned edges | 1,024 | 1–100,000 |
| Snapshot edges scanned for adjacency | 500,000 | 1–4,000,000 |
| Examined adjacency entries | 100,000 | 1–1,000,000 |
| Canonical result bytes | 1 MiB | 16 KiB–8 MiB |

The requested edge bound must also fit the snapshot's `max_query_edges`.
Snapshot validation is bounded by its existing node/edge/evidence profile and is
not counted as traversal work. An insufficient scan budget rejects **before**
indexing; the query never walks a partial adjacency index. All snapshot edges,
including filtered edges, count in `scanned_edges`. Traversal expansions count
revisits, confidence-admitted incident entries and depth-boundary inspections.

Depth, node, edge, expansion and output-byte stops are distinct truncation reasons.
A discovery edge and its new endpoint are admitted together, so retained edges
never dangle and discovered nodes never lose their traversal witness. Item sizes
include their complete evidence; evidence is never silently stripped to fit.
The byte budget reserves 4 KiB beyond exact empty-result metadata and encoded
items, then checks the entire canonical result. This is conservative, not maximal
packing. If roots and metadata do not fit, the operation returns BudgetExceeded.
A later item that does not fit returns the admitted prefix with OutputBytes.

Cancellation is checked before/after snapshot validation, during indexing and
traversal, and around final serialization. Cancellation returns a typed error;
no partial result is published. Snapshot validation and an individual serializer
call are not forcibly interruptible. The operation starts no background work.

There is no continuation cursor in this profile. A caller may explicitly narrow
relations, choose a returned branch as a new root, or submit different limits.
That is a new query with a new digest, not an invisible continuation or automatic
budget reset. No wall-clock-dependent traversal ordering is introduced.

## Coverage and evidence

Every requested relation retains its snapshot coverage; missing relation records
are listed separately in `missing_coverage`. Coverage blockers remain intact.
Truncated takes precedence over Partial (partial/failed coverage), NotEvaluated
(missing/unevaluated coverage), and Complete. The underlying coverage still remains
visible when truncation is the primary state.

`no_new_evidence` means no adjacency was returned beyond the requested roots, not
that the graph contains no evidence. `absence_authoritative` additionally requires
exhaustive traversal, complete negative-authority coverage for every requested
relation, and a non-Candidate-inclusive policy. Its scope is only the exact query;
it says nothing about other confidence tiers, relations, graphs or WoW runtime.

Node/edge records preserve original evidence handles. Dereferencing evidence,
full producer/registry/conflict explanations, axis profiles and service/CLI
routing are separate operations, not inferred from this projection. No fixture
freeze, full E2-A acceptance or product-launch gate is advanced by this code.

Implementation: `src/subgraph.rs` and `src/subgraph/walk.rs`.
Contract: `e2/QUERY_MODEL.md` (`project_subgraph`, Confidence policy, Coverage and
absence) and `e2/SECURITY_AND_BUDGETS.md` (Query budgets).

## Per-relation traversal

`new_directed` accepts canonical `GraphRelationDirection` entries and emits the
v2 query/result profile. Every selected relation has one explicit direction;
partial or duplicated direction tables reject. Named [axes](AXIS_USAGE.md) use
this same BFS without reversing or copying stored edges. Uniform-direction
queries still use the original v1 representation and identity.
