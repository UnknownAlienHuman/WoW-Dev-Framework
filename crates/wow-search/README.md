# `wow-search` contract router

**Status:** partial executable E4-A ProjectFile metadata shard and native E4-C build/query route. Full retrieval, lineage, persistence and package acceptance remain open in [#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169).

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
9. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.
10. exact pinned owner repositories/snapshots used by the assigned fixture.

## Active E4-A contract

```text
contract: wow-search/e4-a/exact-generation-retrieval-core
manifest: e4/CONTRACT.json
state: partial-functional-freeze-open
implementation: selected ProjectFile metadata shard; full E4-A pending
```

The selected [native search command](../../apps/wow/SEARCH_BUILD_QUERY.md) builds
one immutable shard from an exact retained Project/Graph pair, preserves the
owner's file ID and relative-path label, and exposes exact ID/label lookups plus
Candidate-only literal FTS. The one-shot CLI uses an in-memory generic Store.
The separate durable CLI builds a retained shard, reopens its exact handle in a
later process, and explicitly retires/collects it through the operation journal,
catalog roots and leases. Explicit retained file-set queries add identity,
case-sensitive metadata-prefix and literal lanes, ranked explanations and honest
misses. Opt-in [whole-candidate pages](../../apps/wow/SEARCH_PAGINATION.md) preserve
an exact result/shard and cumulative budgets through continuation and release.
Other document/lane classes and full durability/fault qualification remain open.
The separate [native Reference callable component](../../apps/wow/SEARCH_REFERENCE.md)
adds genuine canonical names, namespace/member and original signature fields,
with durable exact owner/shard roots and source-absent queries. It shares the
existing physical Store; it never reads names from opaque payloads or key text.
The historical preimplementation freeze is not accepted.

The explicit file-set component uses a separately named metadata profile and
`wow-search/project-file-set-shard-manifest/1`. It preserves the single-file
profile/manifest bytes, captures one shared owner/evidence set, retains all
selected documents in canonical ID order and validates every physical row on
reopen. The existing native Store owns SQLite/FTS and lease cleanup. The first
public consumers are explicit one-shot build/query and the selected durable
build/query/source-hold/retire/GC lifecycle. Original single-file handles remain
exact; full E4/fault/platform acceptance remains open in #169.

Explicit retained retrieval adds a closed query over the file-set component:
exact entity, case-sensitive metadata-label prefix and native literal FTS with
independent lane states, immutable candidate ordering, authority-banded ordinal
fusion and field-origin explanations. Missing exact IDs stay partial; text misses
and approximate matches carry no absence authority. Unsupported required lanes
are unavailable. Canonical API names/aliases are not inferred from labels or
opaque owner payload. Continuation is not advertised by this query profile.

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
