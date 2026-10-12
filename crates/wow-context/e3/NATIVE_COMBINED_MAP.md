# Combined native Primary and platform map

The selected family is `wow-context/combined-native-one-hop/1`, exported as
`COMBINED_NATIVE_ONE_HOP_PROFILE`. It composes original maps with separate
universes, map IDs, support and omissions; each construction stage keeps its
existing role rather than receiving another family version. Selected lifecycle/CLI
verification is recorded separately from full acceptance under
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).

## Consuming original owners

The [owner implementation](../src/combined.rs) uses:

```text
build_primary_map_for_composition(selection, receipts, stop)
  -> PrimaryMapForComposition
build_combined_context_map(primary, platform_view, reference, reference_profile, stop)
  -> CombinedContextMap
```

The sealed Primary handoff retains its original map and actual prepared-selection
Budget. Combined construction consumes that handoff before platform binding and
its one native Skeleton read. It does not compose separately budgeted maps or
reconstruct authority/allowances from decoded records. The original maps
remain distinct in `core.primary` and `core.platform`; their cached canonical
bytes are separately retained;
the outer artifact has its own `combined_map_id`. Original Primary platform-exclusion
omissions remain in that component while the outer composition supplies a separate
platform map. Both native owners stay held through construction and delivery.

The full ProfileIdentity, Reference generation, schema and full-view digest must
agree between hosts and with the held Primary binding. Platform binding checks
its actual configured profile/generation and performs bounded native full
Reference validation. Shared Reference association does not merge source/graph
universes, prove record/type enrichment or mint accepted links.
`cross_universe_relations` stays `not_evaluated`; future links require a genuine
accepted relation owner and exact support, never names or content heuristics.

## Construction budget scope

One actual Budget starts with the genuine prepared Primary selection and continues
through its map, platform validation/read/map, combined identity and encoding.
The consuming builder has no replacement-limits argument. Earlier acquisition,
universe validation and evidence-catalog preparation retain their existing owner
bounds; the construction ledger does not govern work preceding its creation.

Both hosts must select equal owner limits and evidence limits. Aggregate input
preflight precedes acquisition. Construction input accounting includes both
original tuples, conservatively counting the shared Reference twice. Checked
arithmetic, native scan/support reservations and cancellation remain in force.
For explicit ReferenceFacts, the continuing ledger also precharges selected origin
preparation: origin-vector construction, graph reconstruction, exact Reference
lookup and callable-signature digest work before the L1 operation begins. L1 and
subsequent inspect/build/pack/render work continue that same ledger. Earlier
acquisition, universe validation and evidence-catalog preparation keep their
existing owner bounds. Final service encoding must fit `remaining_metadata_bytes()`
after all work; this is a continuing allowance, not a reset cap. Reservations do
not measure CPU, OS memory or tokens; mandatory overflow refuses without a
prefix-success fallback.

## Exact service transport

[The service driver](../../wow-service/src/context/combined.rs) exposes
`ContextCombinedHostInput::from_json_slice`,
`ContextCombinedRequest::from_json_slice` and
`execute_context_combined(host, request, stop) -> ServiceResult<ContextMapResult>`.
Decoding supplies inert intent; original acquired owners supply authority.

| Document | Exact schema and fields |
|---|---|
| Host | `wow-service/context-combined-native-host/1`: `schema`, `primary`, `platform` |
| Request | `wow-service/context-combined-request/1`: `schema`, `operation="map"`, `"inspect"` or `"build"`, `profile_id="wow-context/combined-native-one-hop/1"`, `primary`, `platform`; inspect requires `inspection`, build requires `build` |
| Result | `wow-service/context-combined-result/1` |

Host `primary` is the complete existing
`wow-service/context-map-native-host/1` document; `platform` is the complete
`wow-service/context-platform-native-host/1` document, including its original
structured `selection` and mandatory composed-candidate catalog.
Request `primary` has exactly `project_store_id`, `store_generation_id`,
`publication_set_id`, `map_profile_id` and required typed `root`. It uses the
existing direct/project-navigation profile and root admission.
Request `platform` is the existing
`wow-service/context-platform-request/1` document with `operation="map"`, exact
candidate ID/digest and native Skeleton binding/query/limits. Unknown fields and
null required fields refuse. Both generations/publications and roots are explicit;
there is no Current resolution or omitted-root inference in this selection.

One service operation holds the Primary read/catalog and admitted platform
candidate together. It qualifies each exact publication and native binding,
encodes a private draft, then closes platform and Primary reads/stores in nested
reverse order. Failed closure suppresses payload; failures retain the actual
`resource_closure` observation. Payload contains `resolved_request_id`, separate
`primary`/`platform` read bindings, original `platform_selection`,
`combined_map_byte_length` and original `combined_map`.

Either truncated component makes the outer state Truncated; otherwise it is
Partial. Original coverage/conflicts/support/NotEvaluated stay nested and
`absence_authoritative=false`. The [explicit map CLI](../../../apps/wow/CONTEXT_MAP.md#selected-combined-native-map)
uses envelope JSON only. Existing Primary/platform contracts remain separate;
cross-universe links and runtime qualification remain open. No behavioral or
acceptance result is asserted by this contract.

## Selected combined inspection

The specified inspect selection uses `operation="inspect"` and requires
an `inspection` object with exactly `level` (`l0`, `l1` or `both`) and optional
`l1_profile_id`. A project root selects L0 only; L1/Both use a genuine detail map
and an eligible exact entity root. Omitting `l1_profile_id` preserves the existing
default Primary skeleton selection with unchanged request/artifact bytes. The only
accepted non-null explicit values are `wow-context/primary-state-detail-l1/1` and
`wow-context/primary-reference-facts-l1/1`; either requires `level="l1"` and an
exact `GraphEntity` root. Explicit `both` (an L0/L1 mix), L0, project roots, null,
non-string, the default ID and unknown IDs refuse before either owner read is acquired.
The combined map operation does not accept this selection.
Missing inspection for inspect, explicit null, unknown fields/levels and inspection
on map refuse. The nested platform request still uses `operation="map"`; it
supplies the original native map read, not a second inspection acquisition. The
same host/request/result /1 schemas and
`wow-context/combined-native-one-hop/1` family apply.

Inspection consumes the genuine combined construction under its actual continuing
Budget. By default, Primary skeletons use the original L0/L1 engines and Both
keeps their fixed original order. The explicit StateDetail selection uses the
existing native Primary StateDetail L1 engine and retains its nested
`wow-context/l1-skeleton/primary-state-detail/1` schema and original skeleton ID
semantics. ReferenceFacts uses the original native `reference_origins` with the
existing L1 Reference engine and retains `wow-context/l1-skeleton/primary-reference-facts/1`,
including its original origins and identity. Platform inspect borrows the admitted
map/page/support without
another platform read. Maps, universes and their IDs remain distinct; inspection
cannot upgrade native omissions or infer cross-universe links. Omitting inspection
preserves the existing map request/artifact bytes.

Inspection work/retained bytes and final delivery use the continuing construction
allowance; earlier acquisition/universe/catalog work retains its existing owner
bounds. Checked platform-then-Primary read/store closure precedes payload exposure;
overflow/cancellation/close failure suppresses payload conservatively. Partial/
Truncated and nonauthoritative absence remain. The
[combined inspect CLI](../../../apps/wow/CONTEXT_INSPECT.md#selected-combined-native-inspect)
uses explicit host/request files and envelope JSON only. The service result operation
is `context_combined_inspect`; payload retains `combined_inspect_byte_length` and
original `combined_inspect`, alongside the existing native read bindings/selection.
Selected behavioral verification is tracked in #122; full E3 acceptance remains open.

## Selected combined semantic pack and rendering

Build consumes the genuine combined Inspect with
`build_combined_context_semantic_pack(inspect, stop)`. Its original maps, Primary
L0/L1 and platform inspection remain in `core.origins`; no inspect-envelope import,
second platform read or decoded record supplies native authority. Items retain
original node/edge payloads, qualified by Primary/platform owner, native universe
and generation, exact scope and origin map ID. Ordering is Primary then platform,
each with native node IDs before edge IDs; selection traces retain required reasons.
Original support, confidence, coverage, conflicts and omissions remain scoped to
their original owners, including unevaluated platform neighbor support closure.

| Selection | Exact reviewed ID/schema |
|---|---|
| Pack profile | `wow-context/combined-native-structural-pack/1` |
| Semantic pack | `wow-context/semantic-pack/combined/1` |
| JSON renderer | `wow-context/combined-pack-json/1` |
| Markdown renderer | `wow-context/combined-pack-markdown/1` |
| Rendered artifact | `wow-context/rendered-context-artifact/combined/1` |

The existing combined request family selects `operation="build"` and a required
non-null `build` object with exactly `level`, `context_profile_id`, `renderer_profile_ids`
and optional `l1_profile_id`; no other fields are accepted. Level is `l0`, `l1` or `both`, with the original
root/layer admission; project roots select L0 only. Omission of `l1_profile_id`
preserves the existing default Primary selection with unchanged request/artifact
bytes. The only accepted non-null explicit values are
`wow-context/primary-state-detail-l1/1` and
`wow-context/primary-reference-facts-l1/1`; either requires `level="l1"` and an
exact `GraphEntity` root. Explicit `both` (an L0/L1 mix), `l0`, project roots, null,
non-string, the default ID and unknown IDs refuse before owner acquisition. The
context profile is the pack
ID above. The required renderer array is empty for None, contains one reviewed
renderer, or both distinct reviewed IDs. Both renders JSON then Markdown. Unknown/
mixed profiles, duplicates, excess renderers, `inspection` on build and `build`
on map/inspect refuse. The outer profile remains
`wow-context/combined-native-one-hop/1`; the nested platform request remains Map
with its exact candidate/publication/binding. Each explicit recipe retains its
original L1 schema and identity; ReferenceFacts origins survive semantic-pack
construction and rendering. No combined, pack or renderer /2 family is introduced.

Map, Inspect, pack construction/validation and every requested render/validation
continue the same actual private Budget. The pack moves Inspect custody and its
borrowed ledger; serialized budget snapshots are not fresh allowances. Earlier
acquisition/universe/catalog preparation retains its existing owner bounds. Final
service delivery uses the pack remainder after all rendering/validation and waits
for checked platform-then-Primary close. Mandatory overflow, required renderer
refusal, cancellation or close failure suppresses payload; no pack-only or
supported-prefix fallback is delivered. None returns a validated pack without
rendered artifacts. Renderer choice is excluded from semantic pack bytes/identity.

JSON content is the exact canonical pack without transport LF. Markdown uses
fixed escaped metadata records and exact UTF-8 trace ranges, including one ORIGINS
record for the complete original combined Inspect closure. Rendering preserves
data and grants no new authority. The pack remains Partial/Truncated with
`absence_authoritative=false`; selected ReferenceFacts origins and identity remain
in the original L1 scope. Broader Reference enrichment, cross-universe links,
control/effects, excerpts/tokens, deeper continuation and runtime remain NotEvaluated.
Source bodies are excluded and source inclusion is not permitted. The
[explicit combined Build CLI](../../../apps/wow/CONTEXT_BUILD.md#selected-combined-native-build)
returns envelope JSON only. Selected gates are tracked in #122; this contract
makes no full E3 or current-source/runtime acceptance claim.
