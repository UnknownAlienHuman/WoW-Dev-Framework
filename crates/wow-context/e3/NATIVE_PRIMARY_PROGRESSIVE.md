# Native Primary progressive expansion and packs

This selected contract covers explicit native batches under
`wow-context/primary-native-progressive/1`. It extends the
[one-round route](NATIVE_PRIMARY_EXPANSION.md) without changing that route's
profile or bytes. Remaining broader facets and E3-B/Tier A/X/runtime acceptance
stay in [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).

## Exact ordered selection

Start from an explicit exact GraphEntity root, the original default
`wow-context/primary-l1/1` skeleton and its immutable native detail map.
The caller supplies 1..4 ordered batches, each containing 1..32 distinct exact
GraphEntity IDs. The first batch selects endpoints in that initial map; later
batches select endpoints retained by earlier rounds. Batch order is part of the
selection, not an inferred traversal plan.

Exclude the initial root and all previously queried expansion roots.
Duplicate roots within or across batches, empty/excess batches, nonmembers,
mixed one-round/progressive selection and incompatible profiles refuse.
L0, Both, Project and selected StateDetail/ReferenceFacts L1 are outside this
profile. Native query direction is independent of inspection level.

Each selected endpoint receives an original native direct-neighborhood query
against the same acquired graph. Existing relation vocabulary, confidence and
per-query neighbor/edge/explanation/evidence limits apply. Parallel native edges
retain their distinct IDs. No source parser, name join, inferred transitive edge,
automatic frontier, source excerpt or runtime observation is introduced.

## Immutable custody and history

One exact project/configuration/graph/Reference universe remains bound throughout.
Every round preserves the original skeletons, prior immutable round artifacts,
genuine native receipts, aggregate origins, inclusion reasons and previously
selected records. Deduplication preserves exact node/edge identities, original
evidence and source handles/digests/spans, with applicable package/TOC/load custody.

History retains original coverage, conflicts, truncation and scoped omissions.
Missing mandatory support, mixed owners or a nonmember endpoint refuses;
serialized requests, decoded artifacts or labels cannot mint native authority.
Round identity binds the original universe and inputs, explicit ordered batches,
prior history, reviewed profile and selected limits.

## One cumulative budget and finite stopping

The actual private Budget moves through the rounds and consuming pack. Whole
batch scans, retained history/receipts, indexes, records/reasons, support/custody,
canonical encoding, validation, rendering and final service delivery share that
continuing ledger. Per-query limits remain independent; no fresh round allowance
or ledger reconstructed from scalar usage/remaining counters is accepted.
Charge before retention. Mandatory overflow refuses instead of dropping earlier
rounds or evidence to fit.

Planning, queries, admission, retention and encoding remain cancellable.
Delivery requires the actual remaining metadata allowance after validation and
all requested renders, followed by checked read/store closure. A close failure
suppresses the draft payload.

Stop after the final explicit batch or the four-round profile bound.
An empty added set does not establish NoNewEvidence, CompleteForRequest, a clean
negative or runtime completeness. Output remains Partial or Truncated with
absence_authoritative=false; earlier native truncation/omissions remain visible.
There is no background expansion, automatic next frontier or retained
cross-request continuation.

## Inspect, pack and render selection

[Inspect](../../../apps/wow/CONTEXT_INSPECT.md#selected-native-progressive-expansion)
uses `--level l1`, an exact `--root graph-entity@<base64url-id>`,
`--expansion-profile-id wow-context/primary-native-progressive/1` and repeatable
`--frontier-round '<token>[,<token>...]'`. Each token uses that same exact
graph-entity encoding. The CLI passes explicit batches mechanically; native
membership is checked after acquisition. Existing host and Current/Reference
guards remain in force.

[Build](../../../apps/wow/CONTEXT_BUILD.md#selected-progressive-pack) uses the
same root/level/expansion selection with this family:

| Contract | ID |
|---|---|
| Pack profile | `wow-context/primary-progressive-pack/1` |
| Pack schema | `wow-context/semantic-pack/primary-progressive/1` |
| JSON renderer | `wow-context/primary-progressive-pack-json/1` |
| Markdown renderer | `wow-context/primary-progressive-pack-markdown/1` |

One service operation uses one held native acquisition. Build consumes genuine
round history, original skeletons and the actual private Budget; it does not call
inspect, import its envelope or rebuild native authority from serialized data.
The pack retains immutable round artifacts/support and original node/edge items.
JSON emits exact canonical pack bytes. Markdown uses the existing escaped
metadata boundary and exact semantic trace, retaining the complete history.
Zero/one/both renderer selection keeps fixed JSON-then-Markdown ordering;
renderer choice remains outside semantic pack identity.

## Compatibility and remaining scope

The old one-round/default/state/Reference selections retain their profiles,
bytes, scoped omissions and original skeleton behavior. New selection fields
are omitted when unselected. No Project/Reference producer, registry, persistence
or replay change is introduced by this context query.

This finite selected route does not settle broader facets, automatic selection,
source excerpts/privacy/token fit, platform/combined views, complete-corpus or
runtime qualification. Cross-request continuation is Service-owned: it reopens
the exact retained generation and reconstructs each issued prefix through this
same native planner. Context persists no serialized step. Its in-process
`ContextBudgetReceipt` comes from a validated native pack and exact closed
Service envelope; it cannot create or deserialize a `ContextExpansionStep`.
For each resume, the latest verified cumulative historical ledger is carried
once, and the freshly replayed native cost of every prior prefix is charged
separately. The saved pre-delivery usage must equal that reconstructed private
ledger; the exact delivered envelope bytes are then charged as well. No saved
counter grants a remaining allowance, and overflow refuses the next round.
Rendering and additional native queries do not upgrade evidence or make absence authoritative.
