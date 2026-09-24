# `wow-graph` implementation status

## Implemented E2-A boundaries

- Exact graph universe and generation identities, canonical node/edge/snapshot records and domain-separated SHA-256 identities.
- Explicit relation families and `Proven`, `Derived`, `Possible`, `Candidate` confidence without automatic promotion.
- Duplicate, endpoint, self-edge, universe, generation, evidence, canonical-order and snapshot identity validation.
- Bounded direct incoming/outgoing/both neighbor queries with separate Complete, Partial, NotEvaluated and Truncated states.
- Authoritative empty queries only under complete negative-authority coverage without truncation.
- Content-addressed entity/relation registry bundles and typed proposal validation against an exact input graph.
- Accepted proposal records retain source/evidence/coverage IDs; rejected proposals retain typed reasons.
- Existing E1-style immutable snapshot storage and catalog compare-and-swap, retention leases, bounded GC, integrity scans and logical manifests. This is not the selected E2-D coherent ProjectStore publication protocol.

## Bounded path queries

`GraphPathQuery` implements the Phase 4 simple-path reader. See [PATH_USAGE.md](PATH_USAGE.md).

- Exact snapshot identity, distinct existing root/target, direction and canonical relation whitelist.
- Iterative cycle-safe traversal in deterministic edge-ID order; no shortest-path or cyclic-walk claim.
- Proven/Derived defaults; Possible and Candidate require explicit inclusion. Path confidence never exceeds its weakest edge.
- Original nodes, edges and evidence remain unchanged. Transitive paths do not create direct edges.
- Validated depth, page-path, expansion, returned-edge and full canonical-output byte budgets.
- Separate depth/expansion/path-count/byte truncation reasons and typed cancellation.
- Checked query-bound continuation for count/byte pages, with replay work inside the same expansion budget and explicit prior truncation.
- Complete/Partial/NotEvaluated/Truncated coverage; continued, candidate-inclusive, incomplete or truncated queries cannot authorize absence.
- Seven grouped acceptance tests exercise direction, cycles, confidence, evidence, determinism, pagination, cursor/request/snapshot tampering, limits, cancellation, coverage and oversized output.

## Producer partition replacement

`GraphPartitionSnapshot`, `GraphPartitionReplacementPlan` and `GraphPartitionSession` implement the in-memory ownership/publication boundary. See [PARTITION_USAGE.md](PARTITION_USAGE.md).

- One immutable foundation, registry bundle and source context bind the input stream.
- Each producer partition retains its complete original batch, version, validation report and coverage under an exact digest.
- Single and bounded multi-partition replacements name one expected graph snapshot and each previous partition digest; absent means create-only.
- Multi-partition plans reject duplicates, validate all guards against the same base, use one fixed surviving endpoint view and publish all or none. The ordered change manifest retains exact before/after partition digests.
- The owner removes only the named partitions, validates independent replacements against surviving endpoints and checks the complete final graph for dangling references. Endpoint and dependent-edge owners can be disabled together.
- Rejected proposals cannot become a partially accepted replacement.
- Shared semantic nodes retain other producers' evidence; original assertions remain separate and unchanged.
- Empty replacements retain partition tombstones. Omitted previously declared coverage becomes NotEvaluated. Aggregation never grants negative authority to producer observations.
- Materialized graph generation derives from the foundation, registry, source context and canonical partition contents, not execution order or previous publication history.
- Published nodes/edges use the derived graph generation; an explicit input view retains endpoint identities in the original input generation.
- Candidate validation and cancellation checks precede the single in-memory Arc assignment. Stale competing plans fail; retained views never change. A true no-change publication retains the same Arc.
- Exact deserialized snapshots revalidate their batches, reports, ownership, coverage, generation and materialized projection.
- Partition count and total node/edge assertions are bounded before materialization.

The partition acceptance tests cover atomic multi-partition publication, combined budgets and 64-partition boundaries, independent input order, shared ownership, removal, stale state, version changes, cancellation, dangling dependencies, no-change identity, tampering, budget excess and source-context mismatch. These are synthetic owner-boundary tests, not WoW runtime evidence.

## Remaining E2-A scope

- Cross-replacement dependency scheduling and richer assertion/conflict/attribute policies.
- Remaining axis families/forms, full conflict/derivation explanations and normative fixture closure.
- Cross-store source/evidence/coverage resolution, beyond preserved and validated IDs.
- Inactive durable generation, post-open golden validation and coherent ProjectPublicationSet activation through E2-D.
- Multi-process reader leases, crash recovery and durable last-known-good policy for partition snapshots.

The partition session deliberately does not write the legacy graph.current catalog. E2-C/E2-D must bind it to coherent project/store generations before durable use.

## Authority boundary

Direct edges are not transitive paths. Static observations do not prove runtime delivery, execution, readiness, performance, taint, combat, protected or Secret Value behavior. Missing, partial, conflicted, failed or truncated input never proves absence. This owner does not parse source, run recognizers, index projects, rank search, orchestrate services, edit source or implement E4 lineage/migration/impact.

## Bounded subgraph projection

`GraphSubgraphQuery` projects a multi-root breadth-first neighborhood from one
exact snapshot with direction/relation/confidence filters, original evidence,
discovery witnesses, explicit coverage and node/edge/depth/scan/expansion/byte
bounds. See [SUBGRAPH_USAGE.md](SUBGRAPH_USAGE.md). No graph mutation, new
assertion, continuation cursor, axis or service/CLI route is introduced.
Full producer/evidence explanation and package acceptance remain separate.


## Retained-support explanations

`GraphExplainQuery` explains an exact materialized entity or relation through its
validated `GraphPartitionSnapshot`: original foundation and producer proposals,
accepted records, registry definitions, confidence and evidence/source/coverage
handles. Relation coverage retains every partition, including missing records.
Canonical support/byte limits and cancellation are explicit. See
[EXPLANATION_USAGE.md](EXPLANATION_USAGE.md). This is retained-support enumeration,
not external evidence dereferencing, a complete conflict/derivation model, or a
service/CLI route. Existing identities and package-acceptance gates are unchanged.

## Registry-bound axes

`GraphAxisProfile` and `GraphAxisQuery` add ownership/load/inheritance/registration/
lifecycle/state/call views, with per-relation directions and exact registry/snapshot
bindings. Multi-parent forms remain separate from general directed networks.
Lexical/object families are explicitly unsupported in the current stored schema.
The shared BFS preserves all original evidence, confidence, budgets and cancellation;
profile/header bytes and both query scan passes count toward the axis limits.
See [AXIS_USAGE.md](AXIS_USAGE.md). Complete traversal is not complete semantic or
conflict coverage. Axis persistence, root discovery, unique-parent chain policy,
service/CLI routing and full package acceptance remain separate.

## Direct graph navigation

Exact node lookup and bounded one-hop neighbors now have snapshot-bound owner
APIs and service/CLI routing. The legacy neighbor selection is reused with
execution-time request validation and bounded prefix retention. New reads keep
original evidence, explicit missing coverage, exact omission counts and typed
cancellation. See [DIRECT_USAGE.md](DIRECT_USAGE.md). Entity absence cannot gain
negative authority from relation-only coverage; full E2 acceptance stays open.
