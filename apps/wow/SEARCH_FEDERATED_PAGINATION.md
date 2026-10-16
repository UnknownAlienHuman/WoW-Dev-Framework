# Captured federation pages

Add `--page-json` to the existing exact `wow search federated` request:

```json
{
  "schema":"wow-search/federated-page-policy/1",
  "page_size":2,
  "max_pages":8,
  "max_delivered_candidates":32,
  "max_delivered_candidate_bytes":8388608,
  "max_delivered_page_bytes":16777216,
  "max_replayed_manifest_bytes":33554432
}
```

The initial operation still runs both genuine owner queries and exact compatible
fusion. It stores the final ordered Search result and atomically retains four
independent catalog roots: result session, primary SearchShard, Reference
SearchShard and the original ReferenceView. Existing no-page request/digest/result
bytes are unchanged; selecting pages uses its own operation identity.

```text
wow search federated continue --continuation-json <EXACT-TOKEN>
  --search-store <SAME-STORE> --operation-id <ID>
  --allow-local-metadata-only <ORIGINAL-STORE/OUTPUT-LIMITS>

wow search federated results release --session-object-id <EXACT-SESSION>
  --search-store <SAME-STORE> --operation-id <ID>
  --allow-local-metadata-only <STORE/OUTPUT-LIMITS>
```

Continuation verifies the exact prior completed response/token, original limits,
policy and cumulative budget. It reacquires both exact shards, using the original
retained ReferenceView for native Reference validation. Captured component owner/
query/universe metadata and selected original documents are checked against
their actual shard manifests. Global result/hash/query/universe/profile,
candidate IDs/bands/caps/ordinals/integer contributions/rank/order are checked
before whole-candidate rendering. Completion CASes all four roots and cannot
recreate a concurrently released session.

Graph traversal and native Project/Graph acquisition are not repeated. Recorded
graph paths, confidence, support and component explanations retain their original
meaning and bytes; `result_summary` carries the original metadata and truncation.
These are captured Search observations. The page port returns no reconstructed
native owner or sealed query result and grants no new search-to-context authority.
Source/config files and the entire original SourceStore can be offline.

The same cumulative PageBudget charges pages, candidates, candidate bytes,
canonical page bytes and reread captured-manifest bytes. The complete summary is
charged per page and is never silently shortened. A page does not change the
enumeration cap or turn original truncation into completeness. Tampered policy,
position, universe, session or counters reject; exhaustion/cancellation does not
advance or recreate a session.

Reference closes before primary, including later Reference open/validation/
release failure. If both cleanup paths fail, both reconciliation identities
remain in the typed unknown-outcome error. No success precedes mandatory closure.

Ordinary retirement removes original shard roots and the initial Project Source
hold. Each session's four roots remain independent until explicit release. One
session release cannot remove another's custody. After the last release, native
GC can collect both shards and ReferenceView. Exact completed response replay
remains historical and creates no roots.

New page/session/cursor consumers have separately named wire identities; physical
Store and admitted standalone/no-page profiles remain unchanged. This selected
two-universe continuation does not qualify Blizzard UI admission, additional
document kinds, aliases, native context selection or full E4/TierX/runtime/launch.
