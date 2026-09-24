# Exact retained-support explanations

`GraphExplainQuery` implements the retained-support slice of `explain_entity` and
`explain_relation` on a `GraphPartitionSnapshot`. The request names an exact
**materialized** snapshot and node/edge ID; it does not accept a display name,
producer-local proposal ID, input-generation ID, or an implicit current pointer.

```rust,ignore
let query = GraphExplainQuery::new(
    owner.snapshot().snapshot_id().clone(),
    GraphExplainSubject::Relation(edge_id),
    GraphExplainLimits::default(),
)?;
let explanation = query.execute(&owner, &cancelled)?;
// Entity inspection uses GraphExplainSubject::Entity(node_id).
// Inspect support_complete(), truncations(), coverage(), and boundaries().
```

## What is retained

The result includes the selected materialized record (and both endpoints for a
relation), exact graph/source-context/foundation identities and registry identity.
Its support list preserves the original foundation record, when present, followed
by every contributing accepted producer proposal in partition-ID/proposal-ID
order. Each producer item contains the partition ID/digest, producer version,
batch/report IDs, raw proposal, accepted record, and exact registry definition.
Raw semantic identity fields, confidence, evidence, source handles and coverage
handles are not stripped or merged. Proposal IDs remain scoped to their batch;
the query does not invent a historical assertion identifier.

Publication rebinds input-generation node and edge IDs. The query reverses that
binding using the same identity constructors. It never matches declarations by
display name or treats the projected node's merged evidence as one producer's
assertion. Contributors with identical accepted edges remain separate support
items. Different confidence/evidence edges have different IDs in the current
edge schema and require separate exact queries; they are not silently combined.

`GraphExplanation` borrows its immutable owner. It is serializable, not a
replacement/publishable snapshot. Accessors expose the retained support without
opening source, providers, storage, or another compiler. There is no confidence
filter: explicitly naming an exact Candidate record requests its inspection,
not a promotion of that record.

## Coverage and unresolved boundaries

For a relation, the result includes its coverage at the materialized, foundation
and **every** producer layer, including empty/noncontributing partitions. Missing
records have an explicit `missing` observation. Original blockers, failed/partial
states and negative-authority flags remain untouched. Entity-kind coverage is
not modeled by the existing relation-only coverage schema; entity explanations
say so rather than substituting incidental relation coverage.

`support_complete` means only that all retained contributors were returned. It
is not evidence completeness or full normative explanation acceptance. The
current snapshot retains handles rather than external EvidenceRecord bodies,
derivation DAGs or a conflict assessment; the result explicitly lists these
boundaries. A foundation record also lacks producer/batch metadata. Its unknown
producer is not attributed to a later partition or the query itself. No empty
conflict list is offered as proof of consistency. Rejected replacement batches
are outside a published snapshot and cannot be reconstructed from it.

A missing exact subject is a typed query error, not authoritative absence.
`absence_authoritative` is always false for this inspection operation. Complete
support enumeration cannot waive missing evidence, source, conflict or coverage
validation outside the snapshot owner.

## Limits and cancellation

Defaults: 500,000 scanned producer assertions, 128 returned supports and 1 MiB of
canonical output. Hard maxima: 4,000,000 / 4,096 / 8 MiB; minimum output is 16 KiB.
The scan counts accepted entity or relation records, as appropriate, including
nonmatches. Foundation lookup is binary. The scan budget is preflighted before
full partition validation. Validation itself retains its existing bounded
rebuild semantics and is not represented as query scan work.

Support-count and byte limits return a canonical prefix with a typed truncation
reason and the exact total support count. Scanning continues without serializing
omitted items, so callers know how many supports were omitted. Each support is
returned whole with its definition/evidence or omitted whole; nothing dangles.
There is no continuation cursor or implicit budget reset. A request with larger
limits is a distinct query whose digest binds those limits.

The byte budget includes subject, endpoints, all coverage and support items.
Packing reserves 2 KiB beyond encoded metadata/items for final counters and
boundary flags, then checks the complete canonical result. Metadata alone not
fitting is BudgetExceeded, not a misleading empty explanation. This is a
conservative bound, not maximal packing. Result records borrow the owner; only
bounded canonical buffers and the result's small collections are allocated by
collection. Owner validation still rebuilds its existing snapshot representations.

Cancellation is checked around validation, every assertion, and serialization.
Cancellation aborts; no partial result escapes and no work continues in the
background. Individual serializer/owner-validation subcalls are not forcibly
interrupted.

Implementation: `src/explain.rs`, `src/explain/collect.rs` and read-only owner
accessors in `model.rs`, `partition.rs`, `proposal.rs`.
Contract: `e2/QUERY_MODEL.md` (explain_entity / explain_relation),
`e2/IDENTITY_AND_ASSERTIONS.md`, `e2/CONFLICT_COVERAGE_AND_PROVENANCE.md`.
Full conflict/derivation/evidence dereferencing, axes, service/CLI routing and
E2 acceptance remain separate. No dependencies, existing identities or tests change.
