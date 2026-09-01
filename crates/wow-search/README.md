# `wow-search` contract router

**Status:** E4-A exact-generation retrieval core is implementation-ready documentation; Rust implementation has not started.

`wow-search` owns deterministic bounded retrieval over immutable exact-generation search shards. It does not own project/reference/graph truth, resolve current publications, infer lineage or replacements, generate context, call models, or mutate source.

The original pre-E4 scaffold is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md). It is historical design input; the E4-A package controls implementation where the texts differ.

## Canonical route

Read:

1. repository [`AGENTS.md`](../../AGENTS.md);
2. crate instructions and [`../MANIFEST.json`](../MANIFEST.json);
3. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md);
4. [`../WORKSTREAMS.md`](../WORKSTREAMS.md);
5. [`e4/README.md`](e4/README.md) and the complete E4-A reading order;
6. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routing when interpreting patch-sensitive WoW facts;
7. exact pinned owner repositories/snapshots used by the assigned fixture.

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

## Direct dependencies

Maximum active E4-A dependencies:

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
```

No direct E4-A dependency on:

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

Exact string equality proves only the declared string relation. Text, fuzzy, shape, and graph proximity remain candidate evidence. No E4-A operation can assert:

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

## Deferred route

The next documentation package after E4-A is E4-B: explicit cross-generation lineage, migration, and impact contracts. E4-C later exposes search and candidate-to-exact-context-root orchestration through `wow-service` and the thin CLI.

E5 calibration, E6 external/Codebase Memory candidates, and E7 LSP/MCP/release remain later packages.

## Implementation gate

No `Cargo.toml` or `.rs` file may be added until:

- E0–E3 prerequisite implementations and fixture bundles exist;
- exact owner read/search projection ports are frozen;
- exact SQLite/Rust binding/static FTS5/tokenizer profile is probed;
- all E4-A machine fixtures and SHA-256 manifests are frozen;
- recall, zero-false-authority, latency, memory, index-size, fanout, security, and determinism thresholds are accepted.

Missing evidence is blocking/`NotEvaluated`, never pass.
