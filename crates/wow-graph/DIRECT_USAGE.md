# Exact entity and bounded direct-neighbor reads

```rust,ignore
let query = GraphEntityQuery::new(snapshot.snapshot_id().clone(), node_id.clone(), 1_048_576)?;
let entity = query.execute(&snapshot, &cancelled)?;
// node() is None only for a non-authoritative missing-row observation.

let selection = GraphNeighborQuery::new(
    node_id, GraphDirection::Outgoing,
    vec![GraphRelationKind::Inherits, GraphRelationKind::MixesIn], 256,
)?;
let query = GraphNeighborReadQuery::new(
    snapshot.snapshot_id().clone(), selection, GraphNeighborReadLimits::default(),
)?;
let neighbors = query.execute(&snapshot, &cancelled)?;
// Inspect state(), coverage(), missing_coverage(), omitted_edges(), truncations().
```

These are E2-A `entity_exact` and `neighbors` projections over a validated,
immutable `GraphSnapshot`. A `GraphPartitionSnapshot` consumer validates its
complete owner first, as the service does. Native results borrow the original
records and retain their universe/generation; they cannot publish new graph truth.
No source, provider, store or compiler is opened. Exact keys in this stored profile
are materialized `GraphNodeId`s, not a newly introduced semantic EntityKey schema.

## Entity inspection

`GraphEntityQuery` requires exact snapshot ID, node ID and a canonical output limit
of 16 KiB–8 MiB. It validates the request on execution as well as construction.
A snapshot mismatch fails. An existing record yields Found/Complete for lookup
only, preserving its original evidence. Missing yields
NotFoundWithPartialCoverage/NotEvaluated, `no_new_evidence = true`, and no node.
Entity-kind coverage is not retained in the current relation-only schema; absence
is always non-authoritative. External evidence, conflicts and producer assertions
are not resolved by this direct read. Use the existing explanation operation for
retained support, without conflating its separate boundaries with lookup success.

## One-hop neighbors

`GraphNeighborReadQuery` binds the existing selection to an exact snapshot,
`GraphPathConfidence` ceiling (default ProvenAndDerived), and scan/output limits.
Unknown/noncanonical/duplicate relation collections and zero edge bounds reject
at execution. Possible/Candidate are explicit opt-ins and are never promoted.
The selection uses the same predicate as the legacy neighbor API.

Every snapshot edge is scanned in validated edge-ID order, checking cancellation
even for nonmatches. Root-incident edges are admitted once; reverse traversal does
not reverse stored endpoints. Parallel/opposite edges retain their own evidence.
Adjacent nodes are deduplicated and ordered by node ID. There is no second hop,
induced neighborhood, depth policy, cursor, hidden continuation or wall-clock order.

Limits: requested `max_edges` 1–100,000, also capped by snapshot query policy;
`max_scanned_edges` 1–4,000,000 (default 500,000); `max_output_bytes` 16 KiB–8 MiB
(default 1 MiB). A scan budget below the full edge inventory rejects before query
scanning. Existing snapshot validation/rebuild is separately snapshot-bounded.
No adjacency index or full matching-edge clone is allocated.

Output admits a canonical edge prefix. Each edge and a newly needed endpoint fit
atomically; full evidence is included. The first failed admission stops output
packing, not counting. All remaining matches are counted without serialization,
producing exact matching/omitted totals. Edge and byte stops are distinct. Even
an empty truncated prefix never authorizes absence. Metadata/root/coverage are
measured before allocating canonical buffers, with a 2 KiB reserve for counters
and flags; the entire final canonical result is checked again. Oversize metadata
fails rather than omitting coverage or producing an invalid partial graph.

Coverage includes every selected relation or an explicit missing record. State
folding is shared with the legacy API: Truncated, then Partial for failed/partial
coverage, then NotEvaluated for missing/unevaluated coverage, otherwise Complete.
`no_new_evidence` means no admitted incident edge beyond the supplied root.
The narrow owner absence flag additionally requires zero matches, complete
negative-authority coverage and a non-Candidate-inclusive policy; it applies only
to that exact retained relation/direction/confidence query. The service always
keeps its outer absence flag false for standalone imported graph data.

Cancellation is checked before/after owner validation, on every edge, while
measuring serialization and around canonical allocation. Owner validation and a
single core canonicalizer invocation are not forcibly interrupted. Cancellation
returns a typed error, not an incomplete successful view or background work.

## Compatibility and routing

The existing `GraphNeighborQuery`/`GraphNeighborResult` wire layout and accepted
canonical result bytes are unchanged. Legacy execution retains its all-confidence
policy and supplied-snapshot API; it now revalidates request fields and clones
only the requested prefix, rather than all matching edges before truncation.
It is not exposed as the bounded public application entry point.

The new owner profiles are `wow-graph/entity-exact/e2-a/1` and
`wow-graph/neighbors/e2-a/1`. Their query digests bind the complete typed request,
including limits and exact snapshot. Existing graph/snapshot/producer/registry
identities are untouched. Applications use `wow graph entity` and
`wow graph neighbors` through the existing service; see
[GRAPH_INPUT.md](../../apps/wow/GRAPH_INPUT.md).

Implementation: `src/direct.rs`, `src/direct/neighbors.rs`, shared selection and
state folding in `src/query.rs`. Contracts: `e2/QUERY_MODEL.md` and
`e2/SECURITY_AND_BUDGETS.md`. Full E2 acceptance, entity-kind coverage, external
conflict/evidence resolution and durable ProjectStore remain separate.
