# Retained native Reference Search pages

Add `--page-json` to [the exact Reference query](SEARCH_REFERENCE.md). The closed
policy is separate from the ProjectFile policy:

```json
{
  "schema": "wow-search/native-reference-page-policy/1",
  "page_size": 8,
  "max_pages": 16,
  "max_delivered_candidates": 128,
  "max_delivered_candidate_bytes": 1048576,
  "max_delivered_page_bytes": 8388608,
  "max_replayed_manifest_bytes": 16777216
}
```

Unpaged requests keep their original identities and response bytes. Unknown
policy fields, nonpositive limits and another component's policy reject. The
result schema is `wow-search/native-reference-result-page/1`. `page.candidates`
contains whole original candidate objects. `page.result_summary` preserves the
full original result metadata, request/profiles, owner/source bindings and
origins, acquisition omissions, lane states, coverage/conflicts, retrieval state
and absence decision. The original candidate array is replaced by the page's
whole-candidate slice, without silently pruning explanations or source fields.

Page size selects a slice of the complete immutable retained candidate ordering.
It does not change candidate enumeration, ranking, result identity or truncation.
The final or genuinely empty page has `page.end=true` and omits continuation;
the original empty/partial/conflicted/unavailable state remains explicit.

## Retention and continuation

Before page one can be delivered, the existing Store operation transaction
secures **three independent catalog roots** and its completed response:

1. the immutable result/session;
2. the exact native Search shard;
3. the original genuine ReferenceView object needed for validation and replay.

Every session has its own roots. Ordinary `search reference retire` can remove
the original shard/source catalog entries while active result sessions keep
both objects alive. Releasing one session cannot release another session's
source dependency. No source/config files or mutable Current pointer are used
when continuing a retained result.

Pass the complete `payload.continuation` JSON to:

```text
wow search reference continue --search-store <SAME-STORE>
  --continuation-json <EXACT-TOKEN> --operation-id <NEW-ID>
  --allow-local-metadata-only <ORIGINAL-STORE-LIMITS>
  --max-output-bytes <ORIGINAL-LIMIT>
```

The token binds the exact prior completed operation/request, session object,
result/query/universe/shard/profile IDs, page policy, whole-candidate position
and cumulative budget. Service compares it against the recorded prior response
before trusting any cursor counters. An integrity digest is not authorization.
Changing a position, budget, policy, session, source/generation/profile or output
limit cannot continue the old chain.

This profile uses the E4 exact-replay option. After budget admission, Service
reopens the real ReferenceView through its owner and the validated read-only
native shard. It reruns the original bounded query and requires byte-identical
full manifest equality before taking the next slice. Missing/corrupt/mismatched
objects fail exactly; they are never rebuilt or replaced by a Project generation,
decoded public report, another ReferenceView or a current source.

All resources close before success. Completion reasserts all three exact roots
in the same Store transaction as the canonical response. Concurrent release
therefore cannot publish a new cursor backed by missing roots.

## Cumulative bounds

The shared private budget accounting is also used by Project pages; both public
schemas remain distinct. Five counters accumulate through every new page:

- page/query executions;
- delivered whole candidates;
- canonical candidate-array bytes;
- exact canonical owner-page bytes, including repeated metadata, budget and cursor;
- complete replayed manifest bytes, including the initial query.

The original query caps still bound each execution. Exhausted request/replay
limits reject before another query. Detail/snippet/source-body delivery remains
unavailable in this metadata profile. Service/CLI envelope bytes obey the original
per-response cap, bounding all envelopes by `max_pages * max_output_bytes`.
Mandatory whole candidates and metadata cannot be truncated to fit a page. A
nonterminal cursor whose next page would exceed the remaining budget returns
`BudgetExceeded`; the budget never resets. Exact retry of an already completed
operation returns its original bytes without another query or counter charge.

## Explicit release

```text
wow search reference results release --search-store <SAME-STORE>
  --session-object-id <EXACT-ID> --operation-id <NEW-ID>
  --allow-local-metadata-only <STORE-LIMITS> --max-output-bytes <LIMIT>
```

Release atomically removes that session's result/shard/ReferenceView roots and
records its response. Other ordinary or result roots remain independent. Roots
persist until this explicit release, including terminal/budget-exhausted
sessions; this profile promises no time-based expiry. New continuations after
release fail; ordinary `search index gc` can collect otherwise unrooted objects.
Exact replay of a completed page/release remains a historical response and
recreates no roots. Reference and Project continuations are not interchangeable.

Exit codes preserve the original query outcome: exact hit/owner-supported exact
miss `0`, Candidate/partial/empty/conflict/unavailable/truncated `2`; valid request
with stale/tampered/unavailable identity or exhausted budget `3`; malformed CLI
or closed input `64`; cancellation `130`. No runtime/current-source/full E4
acceptance follows from the selected native metadata paging route.
