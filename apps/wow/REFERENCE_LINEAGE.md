# Retained native Reference callable Candidates

This profile uses the existing `wow lineage build|compare|status|validate|explain|retire|abandon`
transport with `--store-root <BEFORE-STORE> --after-store-root <AFTER-STORE>`,
exact ordinary native ProjectStore generations and an explicit `reference_selection`.
It reuses the original Reference callable comparison,
genuine native API projection and checked Store/read owners. Project and Source
lineage profiles keep their own requests, snapshots and stores.

The two original Stores are independently owned. An ordinary graph universe
includes its full Reference profile, so a changed Reference profile cannot be
published into the preceding carrier's epoch. Both carrier universes and exact
read custodies remain visible. The Reference driver opens/closes them separately
and installs one exact generation hold in each original owner. Operational paths
stay outside semantic request digests.

Select these request schemas:

| Operation | Schema |
|---|---|
| build / abandon | `wow-service/reference-lineage-build/1` |
| compare / retire | `wow-service/reference-lineage-read/1` |
| status / validate / explain / exact pair query | `wow-service/reference-lineage-query/1` |

Every selected request includes `reference_selection` and omits `source_selection`.
The selection has schema `wow-service/reference-lineage-selection/1`,
comparison profile `wow-reference/native-callable-comparison/1`, privacy
`local_metadata_only`, exact `before` and `after` guards, unique exact function
`keys`, and explicit `limits` (`max_keys`, `max_view_bytes`, `max_steps`). Each
guard contains the complete native `profile`, `reference_generation` and
`view_digest`. These are intent: only the original held native owners can qualify
them. Build additionally requires its exact before/after Store generation IDs,
operation ID, Graph lineage limits, Store limits, output bound and
`max_source_bytes` (the selected original Reference view byte ceiling).

The genuine retained Library must carry a versioned target in its native profile:

```json
"reference_target": {
  "schema": "wow-service/native-reference-target/1",
  "product": "wow",
  "channel": "live"
}
```

This is declared native profile provenance, bound into the profile's source
identity and schema commitment. It does not prove current source, client build
or runtime qualification. Both sides must retain compatible exact product,
channel, flavor, edition, environment, schema and original callable projection.
The selected consumer refuses legacy missing-target, corrected/type-resolution
and extended native-document views. A lineage request cannot add missing target
authority to an old native publication.

Native producer-input v3 captures target plus an explicit `original` or
`type_resolution` normalization selector. Old v1/v2 continue to reconstruct their
original bytes and refuse target/normalization injection. The original lineage
consumer selects only `original`. Source-only target acquisition uses
`wow-service/source-reference-target-input/1`; the old source-reference input
refuses that new field. Capture and replay require the actual original input,
not a serialized successful report.

The separate Graph input/profile is
`wow-graph/lineage/reference-callable-key-candidates/1`; its Store is
`wow-graph.lineage-reference-key-store.v1`. The exact callable key is an explicit
Candidate discovery feature. All matches within the executed selected key scope
are retained; uniqueness does not establish identity. Original Reference
profile/generation/view/key/digest remain separate from each accepted local
Project Graph projection address. A local API projection is not a complete
Reference universe. Missing/conflicted/incomplete lookups and missing native
projections remain explicit omissions and never imply introduced or removed.

The selected native field comparison remains available as a recorded producer
observation with its original and canonical capture digests. Its ordered field
observations do not become accepted changes. Before/after generation holds are
independent. Publication uses inactive commit, checked reopen/native validation
and seal; reads use actual snapshot/validation leases and checked closure.
Retirement releases exact snapshot roots and original-generation holds; an
existing recorded query can replay after retirement, while a fresh read refuses.
Abandon reconciles failed preparation and releases its exact holds.

Only Candidate relations are emitted. Accepted lineage count stays zero;
lineage negative authority, review promotion, rename/replacement/removal,
migration/static impact and runtime conclusions are unavailable. Full E4/W21,
current/corpus/client/runtime and launch acceptance remains separate under
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125).
