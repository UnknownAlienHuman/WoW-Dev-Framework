# Native Primary frontier pack and rendering

This closed selected consumer extends [native frontier expansion](NATIVE_PRIMARY_EXPANSION.md).
The selected consumer reaches native service and CLI build/JSON/Markdown under
one continuing budget; bounded verification is recorded in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
Full E3-B/Tier A/X and Goal acceptance remain open.

## Named family and exact input

The family is frozen in [profiles.json](native-frontier-pack/profiles.json):

| Contract | ID |
|---|---|
| Pack profile | `wow-context/primary-frontier-pack/1` |
| Pack schema | `wow-context/semantic-pack/primary-frontier/1` |
| Item schema | `wow-context/context-item/primary-frontier/1` |
| JSON renderer | `wow-context/primary-frontier-pack-json/1` |
| Markdown renderer | `wow-context/primary-frontier-pack-markdown/1` |

`build_context_semantic_pack_from_expansion(universe, root, step, stop)`
consumes the genuine ContextExpansionStep once. The exact Entity root and bound
universe must match its original native input. The step supplies its already-built
L1 skeletons, immutable expansion artifact and actual private Budget. No graph
query, source acquisition, map/skeleton rebuild or ledger reconstruction occurs
inside this consumer. Serialized data cannot create native custody.

The existing expansion profile `wow-context/primary-native-frontier/1` retains
one explicit 1..32-member batch, Both direction, 27 native kinds, IncludeCandidate
and depth at most 2. Original root exclusion, native membership and one-round
stopping remain unchanged. Packing does not unlock another frontier.

## Immutable items, reasons and origins

The pack embeds the unchanged exact expansion artifact bytes/ID alongside its
original skeletons. Item and selection-trace `expansion` references retain
`expansion_id` and `item_index` into that native artifact. Every initial/frontier
inclusion reason and previously-selected state remains in that indexed record;
packing does not replace the reason vector with a generic summary. Entity and relation payloads
remain original native records, deduplicated by their exact IDs.

`origin_map_id` points to the aggregate
`wow-context/project-map/primary-frontier-origins/1` origins, distinct from the
unchanged initial one-hop map. Exact universe/project/configuration/graph/Reference
binding, support/evidence, source handles/digests/spans and applicable package /
TOC/load custody remain retained. Coverage, conflicts, truncation and omissions
keep their original scope. Mixed binding or missing mandatory custody refuses.

The native pack request binds the exact `expansion_id`; selected semantic
identity binds the typed pack core and named family. Renderer selection and
delivery accounting remain outside that identity.
Old structural-pack profiles/domains and omitted optional fields retain bytes;
no producer, registry, project replay/storage or generic rendered-artifact
version is introduced.

## Lossless renderers and continuing budget

Both renderers consume the same sealed selected pack and use existing native
validation. JSON emits its exact canonical UTF-8 bytes without a trailing LF.
Markdown adds one typed EXPANSION record immediately after SKELETONS and before
entity/relation items. It retains the complete embedded artifact using the
existing escaped quoted canonical-JSON metadata boundary. Per-record traces
bind semantic targets and zero-based half-open UTF-8 content byte ranges,
including each Markdown record's LF; metadata cannot manufacture templates.

Construction, validation, each requested renderer and artifact validation
continue the actual moved Budget once. Reservations cover retained original
artifacts, new request/items/index references/trace, canonical pack, renderer
records. Delivery checks the final selected service envelope against the actual
remaining metadata allowance after rendering, not a fresh full allowance.
Mandatory overflow/cancellation refuses; requested renderers cannot silently
fall back to a different selection. Reservations are not RSS, runtime duration
or exact token measurements.

## Service/CLI and honest scope

[Context build](../../../apps/wow/CONTEXT_BUILD.md#selected-frontier-pack) pairs
the explicit expansion profile/frontier tokens with this pack/renderer family,
L1 and an exact GraphEntity root. One checked acquisition and one native frontier
batch precede consuming the step; it is not an inspect service call or envelope
import. Zero/one/both renderers keep fixed JSON-then-Markdown ordering. Checked
resource closure still precedes delivery. Inspect artifacts and old default
transport bytes remain unchanged.

Every returned pack remains Partial or Truncated with
`absence_authoritative=false`. Packing/rendering does not turn an empty added
set into NoNewEvidence, a clean negative or runtime completeness. Progressive
rounds/continuation, Reference enrichment, source excerpts/token fit, optional
platform composition and complete-corpus/runtime qualification remain open.
