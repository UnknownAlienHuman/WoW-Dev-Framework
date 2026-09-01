# `wow-search` contract router

**Status:** E4-A exact-generation retrieval core and its E4-B Candidate-only lineage handoff are implementation-ready documentation; E4-C service orchestration is defined separately. Rust implementation has not started.

`wow-search` owns deterministic bounded retrieval over immutable exact-generation search shards. It does not own project/reference/graph truth, resolve current publications, accept lineage assertions, infer replacements, generate context, call models, or mutate source.

The original pre-E4 scaffold is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md). It is historical design input; the E4-A package controls implementation where the texts differ.

## Canonical route

Read:

1. repository [`AGENTS.md`](../../AGENTS.md);
2. crate instructions and [`../MANIFEST.json`](../MANIFEST.json);
3. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md);
4. [`../WORKSTREAMS.md`](../WORKSTREAMS.md);
5. [`e4/README.md`](e4/README.md) and the complete E4-A reading order;
6. [`e4/LINEAGE_CANDIDATE_HANDOFF.md`](e4/LINEAGE_CANDIDATE_HANDOFF.md) for the E4-B Candidate-only producer seam;
7. [`../wow-graph/e4/README.md`](../wow-graph/e4/README.md) for accepted lineage/change/migration/static-impact ownership;
8. [`../wow-service/e4/README.md`](../wow-service/e4/README.md) for E4-C public orchestration;
9. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routing when interpreting patch-sensitive WoW facts;
10. exact pinned owner repositories/snapshots used by the assigned fixture.

## Active E4-A contract

```text
contract: wow-search/e4-a/exact-generation-retrieval-core
manifest: e4/CONTRACT.json
state: implementation-ready-documentation-no-rust-code
implementation: not-started
```

E4-A defines:

- separate immutable user-project, Blizzard UI source, and ReferenceView search shards;
- bounded typed search documents with exact origin/evidence/privacy/license closure;
- exact identity, name, explicit alias, member, prefix, FTS, identifier-similarity, structured-shape, and seeded-graph lanes;
- safe closed query normalization and FTS AST;
- authority bands and canonical integer/ordinal fusion;
- complete candidate explanations;
- exact versus partial versus candidate-only miss semantics;
- whole-candidate pagination and exact retained-shard continuation;
- logical SearchStore and static FTS5 physical-profile boundary;
- security, cancellation, determinism, evaluation, benchmark, and checksum gates.

## E4-B supporting producer seam

`wow-search` may produce exact-generation `search_lineage_candidate` partitions for an explicit before/after comparison. Every pair remains `Candidate`, regardless of exact-name band, rank, signal count, signature/shape/fingerprint similarity, graph proximity, or uniqueness.

It does not accept or publish:

```text
same_lineage_as
lineage_successor_of
moved_from
renamed_from
replaced_by
removed_after
introduced_in
migration recipe
static impact conclusion
```

Those semantics belong to [`wow-graph/e4`](../wow-graph/e4/README.md). E4-C service coordinates exact shard acquisition and producer handoff; `wow-search` never calls the service or graph publication path.

## Direct dependencies

Maximum active E4-A dependencies:

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
```

No direct dependency on:

```text
wow-context
wow-service
wow-cbm
wow-emmy
wow-recognizers
wow-rules
applications
```

Owner facts produced through analyzer/recognizer/rules may appear only through exact project/graph/reference public records. Search does not invoke those producers.

## Authority boundary

```text
owner fact
    exact domain fact with its existing evidence/provenance/confidence/coverage

search signal
    query-relative reason an exact entity was retrieved

ranked candidate
    query-relative candidate assembled from signals
```

Exact string equality proves only the declared string relation. Text, fuzzy, shape, and graph proximity remain candidate evidence. No search operation can assert:

```text
user intended this entity
same lineage
moved or renamed
replaced by
migration path
patch impact
runtime behavior or safety
platform/API authority beyond exact ReferenceView facts
```

## E4-C public route

[`wow-service/e4`](../wow-service/e4/README.md) resolves symbolic selectors, acquires exact shards, invokes E4-A operations, preserves ranking/miss state, requires explicit result/candidate selection, and passes only the selected exact entity root to `wow-context`.

Missing shards never trigger implicit search-index construction. The CLI remains a thin `wow-service` adapter.

## Later route

E5 adds audited calibration corpora/packs while preserving universal outputs and anti-overfitting gates. E6 external/Codebase Memory candidates remain optional and Candidate. E7 LSP/MCP/release remains later.

## Implementation gate

No `Cargo.toml` or `.rs` file may be added until:

- E0–E3 prerequisite implementations and fixture bundles exist;
- exact owner read/search projection ports are frozen;
- exact SQLite/Rust binding/static FTS5/tokenizer profile is probed;
- all E4-A machine fixtures and SHA-256 manifests are frozen;
- recall, zero-false-authority, latency, memory, index-size, fanout, security, and determinism thresholds are accepted.

Missing evidence is blocking/`NotEvaluated`, never pass.
