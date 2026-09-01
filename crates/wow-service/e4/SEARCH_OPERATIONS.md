# E4-C search operations

**Status:** normative service orchestration over E4-A.

## `search_index_status`

Reports exact owner-generation-to-shard catalog state, shard validation/seal/profile/capability/coverage/conflict state, and required/missing implementation gates.

It does not report a shard as current merely because its owner generation was current at some earlier time.

## `search_index_build`

Pipeline:

```text
validate request/idempotency/profile
-> acquire exact owner publication/view(s)
-> validate document projection/privacy/license capabilities
-> invoke E4-A document partition and shard build plan
-> invoke owner/store build and inactive publication path
-> invoke independent E4-A shard validation and golden queries
-> admit exact shard into generation/profile catalog
-> obtain result/retention receipt when requested
-> close resources
-> return canonical outcome
```

Hard rules:

- explicit operation ID and request digest;
- no implicit current refresh during build;
- no in-place shard mutation;
- no query before validation/seal/catalog admission;
- failed/cancelled shard remains failed/quarantined under original identity;
- prior eligible shard remains unchanged;
- source bodies and privacy-restricted fields are not broadened by service defaults.

## `search_index_validate`

Opens one exact shard read-only and invokes E4-A nonrepairing validation. `Invalid` is a completed validation payload. Service never fixes, rebuilds, optimizes or replaces the shard under the same operation.

## `search_query`

```text
normalize public request and selectors
-> acquire exact owner views and validated shards
-> bind exact SearchUniverseSet
-> invoke E4-A request validation/normalization/query plan
-> execute enabled lanes through wow-search
-> fuse/rank/explain/miss-classify/materialize result set
-> validate result
-> admit continuation retention if needed
-> close resources
-> return exact SearchResult
```

Service does not inspect raw FTS/BM25, calculate weights, choose lanes, infer aliases, rerank, or hide lane failures.

A no-candidate result can be `Complete` only for the exact E4-A executed request state. Authoritative absence exists only when the owner E4-A result explicitly supplies it.

## `search_continue`

Reopens the exact shards/result-set manifest/request/profiles/budgets named by the cursor. It never resolves current, changes universes, reruns query normalization under new profiles, resets cumulative budgets or silently rebuilds a missing shard.

## `search_explain`

Given exact result/candidate IDs, delegates to E4-A explanation and returns every lane signal, field origin, authority band, contribution/cap/penalty, tie key, skipped/failed/partial lane, coverage/conflict and nonclaim.

No text summary may imply lineage, replacement or intended-user selection beyond the typed explanation.

## `search_select`

Requires:

```text
exact SearchResultId
exact ResultSetManifestId
exact SearchCandidateId
expected candidate/entity/universe/shard/query/profile digests
explicit selection origin
```

The service validates the candidate against the immutable result set and emits a `SearchSelectionReceipt`.

It never:

- accepts rank number alone;
- accepts display name alone;
- selects candidate 0 by default;
- chooses the only candidate automatically;
- treats high authority band as user intent;
- changes entity evidence or search rank.

## `search_context`

Performs `search_select`, then uses the candidate's exact owner entity ID as an exact E3-C context root.

```text
SearchSelectionReceipt
+ exact E3-C context request/profiles
-> acquire/reuse exact compatible context owner views
-> invoke one existing context operation
-> return SearchContextOutcome
```

The context request may use the same exact project/reference/platform publications only. Any mismatch fails; no nearest compatible context generation.

Search explanation remains in the outer service outcome. Context receives no hidden ranking score or natural-language query as semantic authority.

## Status folding

- validated complete result: `Complete`;
- complete exact no-change build: `NoChange`;
- candidate-only/approximate outcome: `CandidateOnly` where owner reports it;
- missing optional lanes/universes: `Partial` or `NotEvaluated` by owner policy;
- truncated lanes/result: `Truncated`;
- conflict-blocked: `ConflictBlocked`;
- cancellation/failure preserved.

## Security

- literal query text remains bounded data;
- no raw SQL/FTS/regex/expression/plugin/model input;
- no source snippet promoted to evidence;
- no private result fields cross consumer/privacy policy;
- no implicit network/source/project/editor/client access;
- result/cursor/artifact bytes validated before use.
