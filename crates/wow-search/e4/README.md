# `wow-search` E4-A exact-generation retrieval core

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-search/e4-a/exact-generation-retrieval-core`

## Mission

Build and query immutable, exact-generation search shards without converting retrieval similarity into domain authority.

```text
exact project/reference/graph owner view
+ frozen document/field/normalization/index profiles
-> deterministic bounded SearchDocument projection
-> immutable generation-local SearchShard
-> independent validation and read-only publication

exact SearchUniverseSet
+ explicit structured query and/or bounded literal text
+ exact lane/ranking/budget profiles
-> exact identity/name/alias/member/prefix lanes
-> generation-local FTS text lane
-> deterministic identifier-similarity and structured-shape lanes
-> graph expansion only from selected exact seeds
-> per-lane evidence-bearing SearchCandidateSignal records
-> authority-banded deterministic fusion
-> ranked SearchCandidate records and complete explanations
-> stable result-manifest-bound pagination or honest miss classification
```

## Active E4-A scope

- separate immutable shards for one exact user-project publication, one exact Blizzard UI source publication, and one exact ReferenceView generation;
- closed versioned document, field, normalization, index, query, lane, ranking, miss, privacy, budget, and continuation profiles;
- exact entity ID/key, canonical qualified/short name, explicit alias, namespace/member/receiver, and prefix lanes;
- bounded generation-local FTS5 text lane over explicitly approved fields;
- deterministic identifier trigram/edit-distance lane;
- structured kind, receiver, signature, parameter, return, restriction, package/load, and universal-role shape lane;
- bounded graph-assisted expansion from exact query candidates or exact caller seeds;
- authority bands, integer/ordinal features, stable tie-breaking, and complete arithmetic explanations;
- exact result/detail handles, coverage, conflicts, omissions, budgets, cancellation, and stable continuation;
- logical SearchStore schema, immutable build/publication, validation, retention, and GC contracts;
- synthetic, pinned addon, pinned Blizzard UI, ReferenceView, collision, adversarial, and update-history corpora.

## Explicitly deferred

- cross-generation same-entity, moved, renamed, replaced, removed, introduced, migration, or impact authority;
- service and CLI search operations;
- automatic candidate selection for context;
- model, embedding, vector, reranker, Codebase Memory, or other external semantic lanes;
- runtime observations;
- source editing, diagnostics, severity, remediation, or code generation;
- LSP/MCP and release automation.

## Direct dependencies

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
```

## Public operations

```text
validate_search_profiles
build_search_document_partition
plan_search_shard_build
build_search_shard
validate_search_shard
open_search_shard_view
bind_search_universe_set
validate_search_request
normalize_search_request
plan_search_query
run_exact_identity_lane
run_exact_name_lane
run_exact_alias_lane
run_member_prefix_lane
run_text_lane
run_identifier_similarity_lane
run_shape_lane
run_graph_lane
fuse_and_rank_search_candidates
explain_search_candidate
evaluate_search_miss
materialize_search_result_set
paginate_search_results
continue_search_results
validate_search_result
```

## Fundamental distinction

```text
SearchEntityRecord
    exact owner entity/source/reference/graph facts

SearchCandidateSignal
    why one query lane retrieved that exact entity

SearchCandidate
    exact entity plus query-relative retrieval assessment
```

A top-ranked candidate is not proof that it is the user's intended entity. A similarity signal is never an alias, lineage edge, replacement, migration recipe, impact fact, or platform/runtime truth.

## Shard model

One shard binds one exact source universe and owner generation set. Text statistics are local to that shard. Cross-universe federation combines lane ordinals and typed canonical features, never raw FTS/BM25 values.

Candidate physical profile:

```text
search-shard-immutable-sqlite-fts5-v1
```

The profile remains blocked until the exact SQLite library, Rust binding, compile options, built-in FTS5/tokenizer capabilities, platform behavior, durability, integrity, and benchmark results are frozen.

## Completion gate

E4-A code is complete only when exact-generation shards exclude stale/cross-generation documents; exact evidence outranks approximate signals without becoming intent authority; raw FTS values never cross shard boundaries; all results, explanations, misses, pages, and continuations are deterministic under 1/2/N workers and shuffled owner/storage order; no approximate lane emits lineage/replacement truth; exact misses require complete relevant authority; query text cannot inject SQL/FTS syntax; source/private data remains bounded; and every fixture/profile/checksum/benchmark/security gate passes.
