# `wow-search` E4-A exact-generation retrieval core

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-search/e4-a/exact-generation-retrieval-core`

## Mission

Build and query immutable, exact-generation search shards without converting retrieval similarity into domain authority.

```text
exact project/reference/graph owner view
+ frozen document/field/tokenizer/index profiles
-> deterministic bounded SearchDocument projection
-> immutable generation-local SearchShard
-> independent validation and read-only publication

exact SearchUniverseSet
+ explicit structured query and/or bounded literal text
+ exact lane/ranking/budget profiles
-> exact/alias/prefix/text/fuzzy/shape lanes
-> graph expansion only from selected seeds
-> per-lane evidence-bearing SearchCandidateSignal records
-> deterministic integer/ordinal fusion
-> ranked SearchCandidate records and explanations
-> stable snapshot-bound pagination or honest miss classification
```

## E4-A scope

- separate immutable shards for exact user-project, Blizzard UI source, and ReferenceView generations;
- closed versioned field/document/tokenizer/index/query/lane/ranking registries;
- exact identity, canonical-name, explicit-alias, namespace/member/prefix lanes;
- generation-local FTS5 text lane over explicitly allowed bounded fields;
- optional deterministic identifier trigram/edit-distance lane;
- structured kind/receiver/signature/parameter/return/restriction/load/role shape lane;
- bounded graph-assisted expansion from existing search seeds;
- deterministic candidate fusion/ranking/tie-breaking and explanations;
- exact result/detail handles, coverage/conflicts/omissions/budgets;
- exact/partial/candidate-only/miss classifications;
- stable pagination/continuation;
- logical SearchStore schema/operation/validation contract;
- synthetic, pinned addon, pinned Blizzard UI, and ReferenceView corpora.

## Deferred

- inferred cross-build lineage or same-entity decisions;
- replacement/deprecation/migration authority beyond explicit exact owner records already present in the bound generation;
- patch impact and change planning;
- service/CLI search operations and search-to-context root selection;
- model/embedding/vector/Codebase Memory/external candidates;
- runtime observations;
- physical cache outside SearchShard stores;
- LSP/MCP/release/CI.

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
run_exact_identity_lane
run_exact_name_lane
run_exact_alias_lane
run_prefix_member_lane
run_text_lane
run_fuzzy_identifier_lane
run_shape_lane
run_graph_lane
fuse_and_rank_search_candidates
explain_search_candidate
evaluate_search_miss
paginate_search_results
continue_search_results
validate_search_result
```

## Fundamental distinction

```text
SearchEntityRecord
    exact owner entity/source/reference/graph facts

SearchCandidateSignal
    why one query lane retrieved an exact entity

SearchCandidate
    exact entity plus query-relative candidate assessment
```

A highly ranked candidate is not proof that it is the user's intended entity, a replacement, or the same lineage as another entity.

## Shard model

Each shard binds exactly one source universe and one owner generation set. Text corpus statistics are local to that shard. Cross-universe query federation merges lane ordinals and typed features, not raw FTS/BM25 values.

Initial physical candidate profile:

```text
search-shard-immutable-sqlite-fts5-v1
```

It remains blocked on exact SQLite/binding/FTS5/tokenizer probes and benchmark gates before implementation.

## Completion gate

E4-A is complete only when exact-generation shards exclude stale/cross-generation documents, exact evidence outranks approximate signals without becoming query-intent authority, raw FTS scores never cross shard boundaries, all rankings/explanations/pagination are deterministic under 1/2/N workers and shuffled owner/storage order, no approximate lane emits lineage/replacement truth, exact misses require complete relevant coverage, all query text is compiled through a safe closed AST, and every fixture/profile/checksum/benchmark/security gate passes.
