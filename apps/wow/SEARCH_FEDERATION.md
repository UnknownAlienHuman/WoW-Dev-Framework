# Exact primary-project and Reference federation

`wow search federated` queries compatible exact ProjectEntity and native
Reference shards in one physical SearchStore and returns one ranked result:

```text
wow search federated --request-json <CLOSED-REQUEST>
  --search-store <SAME-SEARCH-STORE> --operation-id <ID>
  --lease-expires-after <EPOCH> --allow-local-metadata-only
  <STORE/OUTPUT-LIMITS>
```

Build each shard through its existing command. This operation performs no build,
Current resolution, nearest-profile selection or source acquisition from files.
The primary query reads its exact held ProjectStore generation; Reference reads
its immutable ReferenceView in the same SearchStore. Both native owner/query
ports and their checked resource closure are reused.

The outer JSON schema is `wow-service/exact-search-federation-input/1`, with
exactly `primary`, `reference` and `query` records. Unknown fields reject.

| Primary field | Exact input |
| --- | --- |
| `project_store_root` | Absolute original ProjectStore path |
| `generation`, `logical_shard_id`, `object_id`, `profile_digest` | Original ProjectEntity build identities |
| `bounds` | Original typed Search bounds, matching the build profile |
| `build_operation_id` | Original build operation ID |
| `source_store_epoch_id`, `source_generation_hold_id`, `source_generation_hold_digest` | Original build's independent Source hold receipt |
| `query` | Closed `wow-search/project-entity-query/1` request |

`reference` contains the original complete `handle` and a closed native Reference
`query`. Its exact document selection remains part of the handle. Neither public
labels nor a caller-provided compatibility flag establish compatibility.

The top-level `query` is a federation policy, for example:

```json
{
  "schema":"wow-search/exact-primary-reference-query/1",
  "blizzard_ui":"excluded",
  "universe_priority":["primary_user_project","reference_platform_contract"],
  "max_component_result_bytes":8388608,
  "max_input_candidates":256,
  "max_candidates":64,
  "max_result_bytes":16777216
}
```

Both universe priorities are required exactly once. Order is explicit retrieval
preference. This selected profile excludes optional Blizzard UI; it cannot label
a platform-source publication as primary. Search obtains original `ProjectKind`
from the genuine Project owner through its sealed result. Blizzard UI owner
admission and cross-profile comparison remain separate work.

Normal binding requires the full original ProfileIdentity and Reference
generation to agree exactly. Shard/query profiles, source/view/registry identities
and original owner evidence stay in the result. Same build/name or matching API
strings cannot bridge different generations. An incompatible required component
fails; no fallback substitutes a different shard or profile.

The Search owner performs fusion. Lower original authority bands dominate
approximate candidates; opaque exact owner-key matches receive an explicit band-4
cap because those metadata keys are not canonical API names. Within a band, the
explicit universe priority and original local candidate ordinal determine order.
The frozen tuple also exposes `floor(1000000 / (60 + local_ordinal))`; raw FTS
statistics never enter cross-shard comparison. Exact entities in distinct
universes remain distinct, including equal spellings.

Every selected row retains its entire original candidate, exact component result/
universe/shard IDs, source band, cap, local ordinal and integer contribution.
Component summaries preserve original query, lane states, owner evidence,
conflicts, coverage, graph result and explanations. Unselected candidate arrays
are not delivered through a summary. The complete input-candidate bound refuses
an oversized union before fusion; the output cap returns deterministic truncation
and omission count. Input capture and final output bytes are bounded separately.
Required lane failures/conflicts/truncation remain visible. This profile grants no
global authoritative absence, alias, lineage, replacement or intent claim.

The parent durable operation is registered before reads/leases. Both owners close
before success. Atomic completion CASes all three original catalog roots: the
primary shard, Reference shard and ReferenceView. A concurrent retirement cannot
be undone by completion. Exact completed replay returns its original response
after retirement/GC; a new request requires the original live roots and holds.

Without `--page-json`, this operation returns its original inline immutable result.
Optional [captured federation pages](SEARCH_FEDERATED_PAGINATION.md) retain both
shards, result and ReferenceView for exact continuation/release without Graph
recomputation. Additional source universes, full E4 corpus/fault/platform
qualification and launch acceptance remain open in #169. Existing standalone
queries/pages/profiles and physical Store schemas are unchanged.
