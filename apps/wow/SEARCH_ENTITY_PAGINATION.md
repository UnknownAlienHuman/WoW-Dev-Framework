# Retained ProjectEntity result pages

The existing `wow search entities` query accepts optional `--page-json`:

```json
{
  "schema":"wow-search/project-entity-page-policy/1",
  "page_size":1,
  "max_pages":8,
  "max_delivered_candidates":64,
  "max_delivered_candidate_bytes":4194304,
  "max_delivered_page_bytes":8388608,
  "max_replayed_manifest_bytes":16777216
}
```

The initial query still acquires and validates the exact retained Project/Graph
pair, executes Search and closes the owners before success. It captures the full
ordered Search result and atomically retains two independent catalog roots:
the immutable result session and its exact SearchShard. The original no-page
query, profile, request digest and result bytes are preserved when the option is
absent. Page size does not change the candidate enumeration cap or truncation.

Continue and release with the original SearchStore/output limits:

```text
wow search entities continue --continuation-json <EXACT-TOKEN>
  --search-store <SAME-STORE> --operation-id <ID>
  --allow-local-metadata-only <STORE/OUTPUT-LIMITS>

wow search entities results release --session-object-id <EXACT-SESSION>
  --search-store <SAME-STORE> --operation-id <ID>
  --allow-local-metadata-only <STORE/OUTPUT-LIMITS>
```

Continuation follows E4-A's preferred persisted result-manifest route. It checks
the exact preceding completed operation and returned token, original policy,
result/query/universe/profile/owner/shard identities and both catalog roots.
Search validates and pages its own recorded result representation. It does not
rerun graph traversal or reconstruct native Graph authority from captured JSON.
No SourceStore path, source files, Current selector or implicit rebuild is needed.

Pages retain whole candidates, original directed graph witnesses, ranks and
confidence. `result_summary` retains the original owner evidence, graph result,
explanations, coverage and truncation. These remain recorded Search observations,
with their original authority ceilings. The complete summary is charged on each
page; it is never silently shortened to fit. Native owner access or subsequent
search-to-context selection requires its own exact owner acquisition/retention.

The shared cumulative page budget charges pages, delivered candidate count and
bytes, canonical page bytes and reread captured-manifest bytes. Cursor digests
bind all counters and identities; the previous completion receipt authorizes the
actual cursor. Changing a budget, position, policy, profile or session rejects.
Exhaustion refuses another page. Cancellation does not advance the immutable
result or advertise a completed chain.

Ordinary index retirement releases the build's Source hold and original shard
catalog entry. Existing result-session roots independently retain the captured
result and shard. One session's release cannot release another session. Completion
CASes both original roots and cannot recreate a concurrently released session.
After the last session release, ordinary Search GC can collect the shard. Exact
historical completed response replay creates no new roots.

The new wire consumers are the ProjectEntity page policy/cursor, captured-result
page, Service session and prior-receipt continuation. Physical Store and admitted
shard/query schemas are unchanged. This does not qualify current WoW, runtime,
lineage, search-to-context, federation or full E4/launch acceptance.
