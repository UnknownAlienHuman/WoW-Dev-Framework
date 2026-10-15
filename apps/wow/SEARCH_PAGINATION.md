# Exact retained Search pages

`wow search retrieve` keeps its original unpaged behavior unless explicitly given
`--page-json`. This option selects the closed
`wow-search/project-file-result-page-policy/1` policy:

```json
{
  "schema": "wow-search/project-file-result-page-policy/1",
  "page_size": 8,
  "max_pages": 16,
  "max_delivered_candidates": 128,
  "max_delivered_candidate_bytes": 1048576,
  "max_delivered_page_bytes": 8388608,
  "max_replayed_manifest_bytes": 16777216
}
```

Use the same exact file-set handle, query, metadata permission and finite bounds
as [the retrieval command](SEARCH_BUILD_QUERY.md). Unknown policy fields and zero
bounds reject. Pages preserve whole candidates, all their signals/origins/rank
explanations, owner coverage/conflicts, lane states and the original retrieval
state. A small page is not candidate-cap truncation. An exhausted result has
`page.end=true` and no continuation field; an empty approximate result retains
its nonauthoritative outcome.

Before page one becomes deliverable, Service atomically stores the immutable
result-session object, separate result and shard catalog roots, and completed
canonical response through the existing SearchStore operation transaction. These
roots persist until explicit release, including after the ordinary shard catalog
is retired. Every session must eventually be released, including a terminal or
budget-exhausted session. This profile does not promise time-based expiry.

Pass the complete `payload.continuation` JSON object to:

```text
wow search continue --search-store <SAME-STORE> --operation-id <NEW-ID>
  --continuation-json <EXACT-TOKEN> --allow-local-metadata-only
  <ORIGINAL-STORE-LIMITS> --max-output-bytes <ORIGINAL-LIMIT>
```

The token binds the exact prior completed operation/request, result-session
object, result/query/universe/shard/profile and cumulative cursor. Service checks
it against the recorded prior response before using any index or budget. A
cursor digest is not authorization. Tokens cannot change the policy, privacy,
query, ordering or output limits. Current is never resolved and an absent shard
is never rebuilt.

This profile uses the E4 deterministic-replay option: each new page reopens the
exact native shard, reruns the original bounded query and requires byte-identical
full manifest equality before slicing. Query and ranking profile IDs remain
pinned. Cumulative `pages` limits query executions; `replayed_manifest_bytes`
charges each complete manifest, including the initial execution. The original
query caps apply to each execution. Exhausted request/replay budgets reject
before repeating query work.

The cumulative receipt also charges whole delivered candidates, exact canonical
candidate-array bytes, and exact canonical owner-page bytes, including repeated
coverage, budget and cursor records. Service/CLI envelope bytes separately obey
the original per-response output cap, with total envelopes bounded by
`max_pages * max_output_bytes`. Detail/snippet/source-body delivery is unavailable
in this metadata profile. If a mandatory whole candidate or page does not fit,
the operation fails with `BudgetExceeded`; it never silently drops explanation
or restarts a budget. A cursor can name remaining candidates even when its next
page exceeds the remaining budget; continuing it returns that exact failure.

Release the exact `payload.session_object_id` explicitly:

```text
wow search results release --search-store <SAME-STORE> --operation-id <NEW-ID>
  --session-object-id <EXACT-SESSION-OBJECT-ID> --allow-local-metadata-only
  <STORE-LIMITS> --max-output-bytes <LIMIT>
```

Release atomically removes both session roots and records the response. Existing
other roots still protect their objects. A continuation also reasserts both
exact roots in its completion transaction, so concurrent release cannot publish
a new unretained cursor. New continuation attempts after release fail exactly;
the ordinary explicit `search index gc` can collect an otherwise unrooted shard.
An exact retry of a completed operation returns its original response bytes even
after release/GC. Replayed receipts are historical responses and do not revive
their retention roots.

Successful pages preserve the original retrieval exit code: exact identity hit
`0`; Candidate/partial miss/approximate empty/unavailable/truncated `2`. Explicit
release returns `0`; invalid exact continuation/budget/identity returns `3`.
Canonical input/CLI syntax failures retain their existing refusal.

Additional document/name/alias/member/shape/graph/similarity lanes and full E4
corpus, fault, platform and runtime acceptance remain in Issue #169.
