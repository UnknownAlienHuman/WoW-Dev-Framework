# Bounded paths in one immutable graph

`GraphPathQuery` is the E2-A Phase 4 simple-path operation. It reads a validated
`GraphSnapshot`; it does not publish, modify, or synthesize edges.

```rust,ignore
let query = GraphPathQuery::new(
    snapshot.snapshot_id().clone(),
    root_id,
    target_id,
    GraphDirection::Outgoing,
    vec![GraphRelationKind::Calls],
    GraphPathLimits::default(),
)?;
let page = query.execute(&snapshot, None, &cancelled)?;
// Reuse exactly this query and snapshot for any returned continuation.
let next_page = match page.continuation() {
    Some(cursor) => Some(query.execute(&snapshot, Some(cursor), &cancelled)?),
    None => None,
};
```

## Meaning and ordering

The endpoints must exist and differ. The operation enumerates nonempty simple
paths, never repeating a node. It does not enumerate cyclic walks or promise
shortest paths. An iterative depth-first traversal visits incident edges in
canonical edge-ID order. Paths are ordered lexicographically by their edge-ID
sequences, independently of insertion order. Incoming and bidirectional traversal
retain the original edge direction and identity in the result.

The default confidence ceiling admits only Proven and Derived edges. Use
`with_confidence(GraphPathConfidence::IncludePossible)` or `IncludeCandidate`
explicitly to include weaker evidence. A path's confidence is the weakest edge's
confidence. Original node and edge evidence records are retained. A path is not a
new direct edge and does not establish runtime delivery, execution, or safety.

## Budgets and cancellation

| Limit | Default | Hard range |
|---|---:|---:|
| Maximum depth | 16 | 1–64 |
| Paths per page | 64 | 1–256 |
| Examined adjacency entries | 100,000 | 1–1,000,000 |
| Canonical result bytes | 1 MiB | 16 KiB–8 MiB |

The declared depth multiplied by paths must also fit the snapshot's
`max_query_edges`. Snapshot validation and adjacency indexing are separately
bounded by the snapshot's node/edge/evidence limits; their work is not counted as
traversal expansions. The expansion count includes cycle checks, depth-boundary
checks, and any replay needed for pagination.

Each accepted path is measured in canonical JSON, including its evidence. The
operation reserves 16 KiB plus the encoded query and coverage sizes for result
metadata and the bounded cursor, then verifies the entire final result against
the byte limit. This is a conservative capacity check, not a claim of maximally
packed pages. Insufficient metadata space, or a next path too large for an empty
page, returns BudgetExceeded rather than a nonprogressing continuation.

Depth, expansion, path-count and byte stops are explicit truncation reasons.
Depth exhaustion is conservative: an admissible unvisited extension at the
boundary prevents negative authority, even if that extension might ultimately
fail to reach the target. Cancellation is checked before and after snapshot
validation, during indexing/traversal/cursor validation, and around serialization.
There is no background continuation or wall-clock-driven result ordering.

## Continuation

Path-count and output-byte page boundaries return a cursor after the last emitted
path. The cursor binds the exact snapshot through the complete normalized query
digest, confidence policy, direction, relations, all budgets, ordering version,
and last path, with an integrity digest. Validation checks both the digest and
that the last path is connected, simple, policy-admitted, and ends at the target.
A changed request or snapshot rejects it. The digest is an integrity check, not
an authentication credential or proof that a caller consumed previous pages.

Continuation replays the same deterministic search and counts replay work toward
the expansion budget. It never silently resets or enlarges that budget. Depth
and expansion stops alone have no continuation; increasing those bounds is a
new query. Every continued result retains `prior_truncation = true`, even when
its remaining suffix is exhausted. A continued empty page never proves absence.

## Coverage and authority

Results retain selected relation coverage and distinguish Complete, Partial,
NotEvaluated and Truncated. Failed coverage is Partial; missing or unevaluated
coverage cannot authorize absence. An authoritative empty result requires a
fresh query, exhaustive traversal, complete negative-authority coverage for every
selected relation, and a non-candidate-inclusive policy. Its scope is only the
exact snapshot and requested direction/relations/confidence; it says nothing
about other graphs, other confidence tiers, or platform/runtime availability.

## Remaining scope

Bounded neighborhood projection is available separately through
[GraphSubgraphQuery](SUBGRAPH_USAGE.md). Neither operation implements axes, full
assertion explanation, cross-generation impact, durable E2-D publication, or
service/CLI routing. Existing graph snapshot schemas and producer partition semantics remain
unchanged.
