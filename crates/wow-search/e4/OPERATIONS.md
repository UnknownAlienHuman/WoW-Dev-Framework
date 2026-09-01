# E4-A public operation contracts

**Status:** normative operation behavior. Concrete Rust APIs may differ only with same-change contract and fixture updates.

## `validate_search_profiles`

Input: exact profile bundle and prerequisite capability records.

Output:

```text
Valid
Invalid(errors)
NotEvaluated(missing executable probe/profile)
```

Checks schema closure, version compatibility, bounded values, no executable fields, and exact cross-profile dependencies.

## `build_search_document_partition`

Input: one exact owner partition/read view plus document/field/projection/privacy profiles.

Behavior:

- enumerate through the owner-public bounded catalog;
- project only allowed records/fields;
- preserve all origins/evidence/coverage/conflicts;
- canonicalize and validate;
- emit one immutable partition candidate.

No owner mutation or source fallback.

## `plan_search_shard_build`

Input: exact source binding, target document partitions, optional exact base shard, SearchProfileSet.

Output: complete target membership, reuse/new/removal plan, logical index/FTS operations, expected manifests, validation catalog, budgets.

Stale or incompatible base is rejected; no silent rebase.

## `build_search_shard`

Delegates physical materialization to `wow-store` using the frozen logical plan. Returns an inactive artifact/manifest. It does not mark validation success.

## `validate_search_shard`

Fresh read-only validation of owner binding, profiles, documents, origins, logical indexes, FTS mapping/integrity, stale-removal closure, privacy, coverage, golden queries, and physical read-only state.

Nonrepairing.

## `open_search_shard_view`

Opens one exact validated/sealed shard. Rejects current/latest, unvalidated, failed, corrupt, profile-mismatched, or writable artifacts.

## `bind_search_universe_set`

Combines exact compatible shard views without merging their corpora or identities. Missing optional universe is explicit. Required incompatibility fails.

## `validate_search_request`

Checks closed query schema, exact universe set, roots/text/features, lanes, profiles, privacy, limits, and continuation.

## `normalize_search_request`

Produces exact and approximate identifier forms, bounded text terms, typed filters/features, and a safe FTS AST. Original and normalized digests retained.

No hidden intent classification.

## `plan_search_query`

Freezes shard/lane order, lane eligibility, per-lane budgets, fallback, fusion, explanation, result-set, and cancellation behavior.

## Exact lanes

### `run_exact_identity_lane`
Exact entity key/ID equality.

### `run_exact_name_lane`
Case-sensitive canonical qualified/short-name equality.

### `run_exact_alias_lane`
Exact equality against explicit owner alias records only.

### `run_member_prefix_lane`
Exact namespace/member/receiver-method plus bounded prefix candidate signals.

Each returns a separate `SearchLaneResult`.

## Approximate lanes

### `run_text_lane`
Runs safe generation-local FTS per shard. Returns local ordinals and matched field origins. No cross-shard raw rank comparison.

### `run_identifier_similarity_lane`
Runs frozen bounded trigram/edit-distance features over identifier fields.

### `run_shape_lane`
Matches explicit typed shape features and preserves unknown/partial/conflicted fields.

### `run_graph_lane`
Expands from exact/candidate seeds through reviewed bounded graph relations and reason paths.

## `fuse_and_rank_search_candidates`

Groups signals by exact entity, assigns authority band, computes canonical integer/ordinal rank tuple, applies caps/penalties, and creates complete explanations.

No model or domain inference.

## `explain_search_candidate`

Returns the exact owner entity, field origins, signals, paths, band, contributions, penalties, tie key, lane states, coverage/conflicts/omissions, and nonclaims.

## `evaluate_search_miss`

Applies the exact scoped negative-authority gates. Never uses approximate empty output as proof.

## `materialize_search_result_set`

Creates the immutable ordered candidate manifest after all required lanes/fusion complete. It records candidate cap and completeness separately from page size.

## `paginate_search_results`

Returns one whole-candidate page from the exact result-set manifest.

## `continue_search_results`

Validates cursor, exact retained shards/result set, profiles, privacy, and cumulative budget; returns the next page without current resolution or budget reset.

## `validate_search_result`

Read-only verification of request/query/shards, lane manifests, candidate ordering, explanations, miss, coverage/conflicts/omissions, budget, and continuation.

## Common operation states

```text
Complete
Partial
Truncated
NoChange
NotEvaluated
Unsupported
Cancelled
Failed
```

No empty/default success.

## Common guarantees

- exact generation/profile binding;
- deterministic output;
- typed bounded errors;
- cancellation and no background work;
- mandatory uncertainty/explanation closure;
- no source, storage, editor, process, network, client, model, or project mutation;
- no lineage/replacement/migration/impact claim in E4-A.
