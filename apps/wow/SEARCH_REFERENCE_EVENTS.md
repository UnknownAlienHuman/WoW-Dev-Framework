# Native Reference callable and event documents

The explicit source-reference input field
`"native_documents": "callables_and_events_v1"` selects the mixed native
Reference projection. Omission retains the original callable-only view, Search
profile, handle and bytes. Null or unknown selection values reject.

Use the existing `search reference build/query/continue/results release/retire`
commands and `search index gc` lifecycle. The mixed handle is
`wow-service/native-reference-document-search-handle/1`; its exact document
selection participates in the Search profile, operation identity and retained
session. Removing/changing it is not a fallback to the old component.

The existing native normalizer supplies each original `EventFact`. The event
partition is `reference.native.apidoc.event`, with `ReferenceRecordKind::Event`
and an owner key derived from the exact `LiteralName`. The sealed event signature
retains original generation/view/record/source digests, registration `Name`,
definition namespace, ordered payload fields and their original ranges. There is
no public constructor or payload decoder for this authority.

Search uses the exact event literal as both identifier projections. Registration
`Name` and the definition namespace remain metadata; neither becomes an alias or
a callable namespace/member. The event document's `event` field carries original
definition metadata and ordered payload name/type/inner-type/tri-state nilability
and source ranges. Its callable argument/return vectors are empty and are not
callable-shape evidence. Types remain original unresolved source identities.

The normal identity, exact-name, prefix, safe literal and explicit similarity
lanes apply to the recorded literal. Similarity stays Candidate, including a
zero-distance or case-only match. Shape queries use independent fields:

```json
{
  "shape": {
    "event_payload_count": 2,
    "event_payload": [
      {"position": 1, "original_type": "bool", "nilability": "allows_nil"}
    ]
  }
}
```

Positions are zero based. Callable and event shape fields cannot be mixed in one
selector. Event payload count zero matches an empty recorded payload; it does not
match a zero-argument API. An API-only component reports an event-shape lane as
unavailable. Field-origin explanations name the actual event owner port.

Each materialized event is indexed once. Conflicting candidates remain in the
Reference owner and full conflict summary; no first winner is selected or indexed.
Event coverage is Partial, and the mixed Search profile grants no authoritative
absence. Source inputs are data: no donor/generated Lua is executed, and no
runtime event, dispatch, callback, loaded state, current-client or safety claim
follows from a static declaration.

The same native normalization pass and physical SQLite/FTS5 backend are reused.
Original per-record and aggregate native bounds and explicit Search bounds still
apply. Metadata SourceView and SearchShard are published atomically; whole-result
pages retain result/session, SearchShard and the exact ReferenceView independently.
Continuation validates the original full result and cumulative budgets after
source/config files disappear, ordinary retirement or another session release.
Completed historical replay recreates no roots.

Donor evidence for this operation: freshly resolved Ketho master
`d0b5b51fac4c52c493371b9b18e66ce604ea4326`,
`luasrc/annotate/literals.lua` blob
`1c79f0e9c92a9836218938a34244540db2a999e6` uses exact `LiteralName` and ordered
payload. This port consumes the already normalized Rust model; annotation/type
lowering was not changed. It is not a fresh WoW/Ketho runtime or full Tier X parity
claim. Additional document kinds, aliases, federation and full E4 qualification
remain tracked in #169.
