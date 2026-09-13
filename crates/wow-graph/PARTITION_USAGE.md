# Exact producer partition replacement

The executable E2-A partition profile is synchronous and in-memory. It is not a durable ProjectStore protocol.

## Inputs

Start `GraphPartitionSnapshot::new(registry, foundation, source_context_id, cancelled)` with a validated immutable source-owner `GraphSnapshot`. The foundation fixes input universe, generation and limits. A changed source generation or registry requires a fresh explicit foundation; this API does not silently rebase.

Use `input_view(cancelled)` to obtain endpoint IDs for `GraphProposalEndpoint::Existing`. Those IDs remain in the foundation's input generation. Do not use node IDs from `snapshot()`: the materialized publication has a separate content-derived graph generation.

Construct a `GraphProposalBatch` with the same registry ID/digest, input universe/generation and source context. Its `producer_partition_id` is the ownership key. The graph revalidates the entire batch; it does not accept a caller-supplied validation report as authority.

## Plan and publish

```rust,ignore
let before = session.view();
let request = GraphPartitionReplacement {
    expected_snapshot_id: before.snapshot().snapshot_id().clone(),
    expected_partition_digest: before
        .partition(batch.producer_partition_id())
        .map(|partition| partition.partition_digest().into()),
    producer_version: "1.0.0".into(),
    batch,
    coverage,
};
let plan = before.prepare_replacement(request, &cancelled)?;
let after = session.publish(plan, &cancelled)?;
```

Create the session with `GraphPartitionSession::new(initial, cancelled)`. It requires exclusive mutable access to publish. Reading/cloning an `Arc<GraphPartitionSnapshot>` does not mutate it, and old views remain usable after another publication.

`expected_partition_digest: None` is create-only, not an unchecked update. The previous digest binds producer version as well as batch, report and coverage. Version updates are permitted only against the exact previous digest.

The plan is built without changing current state. Publish rechecks its expected snapshot and validates the candidate before a single Arc assignment. Validation failure, stale state or cancellation before that assignment preserves the current Arc. Cancellation after an accepted assignment cannot retroactively undo it. Reapplying identical partition content returns the same current Arc.

## Atomic replacement sets

`prepare_replacements(requests, cancelled)` prepares one plan for 1–64 independent producer partitions. Every request must name the same current `expected_snapshot_id` and the exact previous digest for its own partition (`None` remains create-only). Duplicate partition IDs are rejected, not merged or applied last-wins.

```rust,ignore
let before = session.view();
// Both requests were constructed against `before`, not intermediate candidates.
let plan = before.prepare_replacements(vec![replacement_a, replacement_b], &cancelled)?;
for change in plan.changes() {
    // Canonical partition order; exact previous and target ownership digests.
    let _ = (change.partition_id(), change.previous_partition_digest(),
             change.target_partition_digest());
}
let after = session.publish(plan, &cancelled)?;
```

Planning removes all selected old partitions before resolving any replacement. Each batch may reference the immutable foundation, unchanged surviving partitions, or its own same-batch proposals. It cannot borrow another replaced partition's old nodes or another new batch's intermediate output. Batches requiring such cross-replacement dependencies are outside this independent-set profile; request order is never an implicit dependency schedule.

The complete final graph is validated only after every replacement is assembled. Consequently an endpoint-owning partition and its dependent edge-owning partition can be disabled together, even when removing the first alone would leave dangling edges. An unchanged producer's dangling edge still rejects the entire set.

One invalid, stale, over-budget, or cancelled member aborts the whole plan. Publication uses the existing single Arc assignment; it never loops over individual publications. The plan lists every requested before/after partition digest, including tombstones and unchanged requests. Repeating an unchanged set retains the exact current Arc. Permuting independent requests produces identical plans and snapshot bytes. Single-partition preparation delegates to this same implementation.

## Removal and ownership

Replace a producer with an empty batch and explicit downgraded coverage to disable it. The empty partition is retained as a tombstone. Omitting previously declared relation coverage inserts a NotEvaluated record instead of erasing coverage loss.

Other producer batches and reports remain byte-identical. Shared semantic nodes retain evidence from surviving assertions. Materialized edge confidence is preserved; candidates are never promoted by aggregation. If a surviving edge requires a node that only the removed partition supplied, replacement fails rather than deleting that edge or synthesizing its endpoint.

The candidate generation hashes the exact foundation, registry, source context and canonical partition contents. Independent replacements performed in different orders produce the same final bytes. Intermediate snapshot IDs and wall clock are not generation inputs.

## Coverage and limits

Each partition must provide canonical, unique per-relation coverage. Producer partitions cannot grant graph-wide negative authority. Missing coverage, explicit failures and partial work remain visible in the aggregate view; an empty batch alone proves no platform absence.

This profile bounds producer partitions to 64 and counts all retained node/edge assertions against foundation limits, including repeated semantic support. Per-record evidence unions remain bounded and fail rather than truncate. There is no partial replacement on budget excess.

## Durable boundary

Do not publish this ownership stream through the older `PersistentGraphStore::publish_current` facade. The selected E2-D contract requires a coherent ProjectPublicationSet, inactive generation, exact post-open validation and compare-and-swap activation. That integration, cross-replacement dependency scheduling, richer conflicts and multi-process recovery remain unimplemented.
