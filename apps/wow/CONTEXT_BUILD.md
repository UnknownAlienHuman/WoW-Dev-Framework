# Native primary context build

`wow context build` acquires one registered immutable Primary project, builds a
sealed L0/L1 semantic pack and optionally renders that same pack. Platform is
explicitly omitted. The [owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_PACK.md)
and [frozen descriptors](../../crates/wow-context/e3/NATIVE_PRIMARY_PACK_PROFILES.md)
define this selected route; [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122)
tracks its remaining qualification.

## Exact CLI selection

```text
wow context build --native-host-config <host.json>
  --project-store <registration-key>
  --publication current|store-generation:<exact-native-generation-id>
  --root project@<unpadded-base64url-of-exact-ProjectId>|graph-entity@<unpadded-base64url-of-exact-node-id>
  --level l0|l1|both
  --context-profile-id wow-context/primary-structural-pack/1|wow-context/primary-frontier-pack/1|wow-context/primary-progressive-pack/1
  [--l0-profile-id wow-context/primary-l0/1|wow-context/primary-state-root-l0/1]
  [--l1-profile-id wow-context/primary-l1/1|wow-context/primary-state-detail-l1/1|wow-context/primary-reference-facts-l1/1]
  [--expansion-profile-id wow-context/primary-native-frontier/1
   --frontier graph-entity@<base64url-id> ...]
  [--expansion-profile-id wow-context/primary-native-progressive/1
   --frontier-round '<token>[,<token>...]' ...]
  [--renderer-profile-id wow-context/primary-pack-json/1|wow-context/primary-frontier-pack-json/1|wow-context/primary-progressive-pack-json/1]
  [--renderer-profile-id wow-context/primary-pack-markdown/1|wow-context/primary-frontier-pack-markdown/1|wow-context/primary-progressive-pack-markdown/1]
  [--platform omitted] [--output envelope-json]
  [--expect-current-record <id>] [--expect-current-digest <sha256:digest>]
  [--expect-profile <id> --expect-reference-generation <id> --expect-reference-view <sha256:digest>]
```

Context profile, level and exact root are mandatory. Default L0 accepts the acquired
ProjectId or an exact native `source_file` / `source_package` container. L1 accepts
an exact Primary graph entity. Both requires one explicit graph entity admitted
by the selected L0 profile, shared unchanged by both layers. Root existence/kind
and level compatibility are service checks; the CLI does not infer a root.

Explicit `--l0-profile-id wow-context/primary-state-root-l0/1` selects L0 or Both
over an exact original ordinary selected-TOC state_root GraphEntity. It retains
the [typed root/declarations and selected literal paths](CONTEXT_INSPECT.md),
with all repeated support and account/character/ambiguity preserved. Both still
uses old primary-l1/1 detail. No runtime values or exhaustive-field authority is
added. Bounded execution evidence is recorded in #122.

Root tokens are canonical unpadded base64url UTF-8 IDs, with no lookup or search.
L0 options select one closed profile; omission/default preserves primary-l0/1.
The L1 option selects a closed profile; omission/default preserves primary-l1/1.
Current guards require `--publication current`; Reference guards form one complete
three-option group. Unknown options/profiles, repeated non-repeatable options and
incompatible level/profile assertions refuse before invocation.

Each renderer option may occur once. No renderer selects `none` and returns a
validated semantic pack with an empty `rendered_artifacts` array. One selects JSON
or Markdown; both select JSON then Markdown regardless of argument order.
There is no `--render` flag, standalone render/import route, auto profile selection
or text/artifact stdout mode. Requested renderers are required: refusal suppresses
the delivery payload rather than returning a pack-only fallback.

## Native API evidence in structural L1 packs

An ordinary native project may explicitly select
`"native_api_uses": "reference_v1"` in its native input manifest. The selection
is admitted into the published project configuration and replay. Context reads
that exact selected publication; it does not select this producer through a new
context option.

Use the existing `--level l1`, `--context-profile-id wow-context/primary-structural-pack/1`
and default `wow-context/primary-l1/1` with an exact graph-entity root whose
bounded native neighborhood contains the accepted UsesApi assertion. Existing
L1 inspect, semantic pack construction and zero/one/both structural renderers
retain the original physical Main function, accepted `native_api_callable`
endpoint and UsesApi relation. The endpoint comes from the
`wow-service.native-api-callables` producer; the relation stays in the existing
calls partition. API endpoint identity retains the exact native Reference
profile/generation/view and original record key/digest.

The host's complete `reference_profile` and `reference` must equal the profile
and ReferenceView held by the genuinely requalified native owner. Matching a
generation alone is insufficient; a mismatched host refuses with IdentityMismatch.
The existing optional three-part Reference guard remains an additional exact
assertion. Store reopening requalifies retained native producer inputs even after
the original source directory has been deleted.

One bounded evidence catalog combines original Project function/call support
with original API source handles and evidence. The service checks the common
GenerationContext, preflights the combined encoded inputs before catalog copies,
and rejects unequal records sharing an evidence or handle ID. Native explanation
receipts carry that genuine support into the existing skeleton, pack and render
records; they do not reconstruct authority from endpoint labels.

With default L1, this evidence handoff leaves Reference record/signature enrichment NotEvaluated.
It adds no source excerpts, source-inclusion permission, runtime authority or
clean-negative claim; failed or missing native documents retain their Partial
coverage and omissions. Existing context/pack/render profiles, schemas, commands
and unselected bytes remain unchanged.

## Selected native state-detail L1 pack

Pair `--level l1 --l1-profile-id wow-context/primary-state-detail-l1/1` with
`--context-profile-id wow-context/primary-structural-pack/1` and an exact
GraphEntity root for an original ordinary selected-TOC state_root or state_path.
Project, L0/Both, other root kinds and frontier selections refuse. Root/path
custody must come from the genuine native owner; names cannot substitute for it.

The embedded L1 retains its existing state-detail profile/schema, original
literal keys, plural supporting TOC declarations, scope/ambiguity and unchanged
direct graph/source/load/evidence custody. Required root/path/support missing
or omitted by truncation refuses; no empty successful facet is substituted.
The [state-detail owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_STATE_DETAIL.md)
defines those same joins for inspect and build.

Zero/one/both renderers use the existing structural JSON/Markdown family.
Native skeleton construction hands its actual Budget into pack finalization,
validation and each requested renderer; build does not call inspect or reconstruct
a ledger from serialized counters. The selected final service envelope must fit
the actual remaining metadata allowance after rendering/validation. Mandatory
overflow or cancellation suppresses delivery.

The optional selected l1_profile_id in the nested native pack request binds
this selection. Its default None is omitted, preserving old pack bytes; no new
pack/render profile or version is introduced. Metadata does not attest state
values, loaded state, combat safety or runtime behavior. The bounded source-absent
build lifecycle, cumulative refusal and actual Windows CLI pass in #122.

## Selected native Reference-facts L1 pack

Select `--level l1 --l1-profile-id wow-context/primary-reference-facts-l1/1`
with `--context-profile-id wow-context/primary-structural-pack/1` and an explicit
exact graph-entity root on a genuine `native_api_uses=reference_v1` publication.
Project/L0/Both and frontier selections refuse. The complete host Reference/profile
must equal the held requalified native owner even when no optional Reference
guard is supplied.

The pack embeds the selected L1 schema
`wow-context/l1-skeleton/primary-reference-facts/1` and its original accepted
API-to-Reference record/source associations. It retains original canonical payload
data, source IDs, restriction facets, partition coverage and exact graph/source/
evidence support. The held Reference catalog adds genuine normalized ordered
source parameters/returns, unresolved type names/inner types, tri-state nilability
and original raw default/stride values and spans. Context does not parse payloads;
unknown fields stay in the original ReferenceRecord. Source type resolution,
corrected overlays, annotation display lowering, broader restriction interpretation,
runtime behavior and authoritative absence remain unevaluated. The original map's
`ReferenceContractNotEvaluated` custody remains scoped to that map; the selected
L1 association does not rewrite its bytes or authorize Reference source excerpts.

Zero/one/both renderers use the existing structural JSON/Markdown profiles. One
acquisition and the continuing native Budget cover selected L1 joins, pack
construction, validation, each renderer and final delivery. Mandatory custody,
work/byte overflow or cancellation suppresses delivery; there is no pack-only
fallback when a required renderer refuses. Optional selected l1_profile_id binds
this recipe into pack identity; omitted/default selection preserves old bytes,
profiles and omissions, including existing state-detail behavior. Persisted
Project/Reference records and their recipes stay unchanged; the retained typed
catalog is runtime owner custody, not a new replay/store representation.

The [selected owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_REFERENCE_FACTS.md)
defines this narrow handoff. Bounded gate outcomes belong to #122; this page
makes no full E3/Tier A/X acceptance claim.

## Selected frontier pack

Select `--context-profile-id wow-context/primary-frontier-pack/1` together with
`--expansion-profile-id wow-context/primary-native-frontier/1` and repeatable
`--frontier graph-entity@<base64url-id>`. This family requires `--level l1` and
an explicit exact graph-entity root. The 1..32 distinct frontier IDs must already
belong to the initial detail map and exclude its root. Profile/frontier options
are paired; duplicates, nonmembers, incompatible levels or mixed pack/renderer
families refuse.

Optional renderers are `wow-context/primary-frontier-pack-json/1` and
`wow-context/primary-frontier-pack-markdown/1`; zero/one/both keeps the existing
selection and fixed JSON-then-Markdown ordering. The service performs one native
frontier batch under the same acquisition, then consumes its exact step, original
L1 and actual Budget into the sealed pack. It does not invoke inspect as another
service call or rebuild the graph/map/skeletons.

Pack schema is `wow-context/semantic-pack/primary-frontier/1`; items use
`wow-context/context-item/primary-frontier/1`. The embedded expansion preserves
its exact artifact bytes/ID. Each item and selection trace binds the expansion
ID and exact item index, retaining every original inclusion reason through that
native record; origin_map_id names the aggregate origins. Original support,
source/load custody, coverage, conflicts and omissions remain explicit.

JSON emits the same exact canonical selected pack. Markdown adds one typed
EXPANSION record immediately after SKELETONS, before entity/relation items, using
the existing metadata-safe encoding and exact content-byte trace ranges.
The selected final service envelope must fit the actual remaining metadata budget
after all requested rendering/validation; it cannot reset the allowance.
Mandatory overflow suppresses the payload. The default family and absent selection
fields retain their original bytes. The focused source-absent lifecycle and actual
Windows CLI verify this selected consumer, including cumulative refusal; see the
[owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_FRONTIER_PACK.md).

## Selected progressive pack

Select `--context-profile-id wow-context/primary-progressive-pack/1` with
`--expansion-profile-id wow-context/primary-native-progressive/1`,
`--level l1`, the default L1 profile and an explicit exact graph-entity root.
Repeat `--frontier-round '<token>[,<token>...]'` for 1..4 ordered batches;
each contains 1..32 distinct exact `graph-entity@<base64url-id>` endpoints.
The first batch uses the initial detail map; later batches use retained prior
round endpoints. Initial/repeated roots and mixed one-round/progressive selections
refuse. The [progressive contract](../../crates/wow-context/e3/NATIVE_PRIMARY_PROGRESSIVE.md)
defines exact native admission, immutable history and conservative stopping.

Pack schema is `wow-context/semantic-pack/primary-progressive/1`. The pack retains
the original skeletons and immutable round artifacts, native receipts, every
inclusion reason, support and scoped coverage/omissions. Selected items remain
original native nodes/edges; history does not manufacture transitive relations.

Optional renderers are `wow-context/primary-progressive-pack-json/1` and
`wow-context/primary-progressive-pack-markdown/1`. Zero/one/both uses the existing
selection conventions and fixed JSON-then-Markdown order; mixed families refuse.
JSON is the exact canonical selected pack. Markdown preserves its semantic fields
and round history through the existing escaped metadata boundary and exact trace.
Renderer choice does not change semantic pack bytes/identity.

One acquisition and one build service call execute the explicit native rounds,
then consume the genuine history and actual private Budget into the pack.
Construction, validation, rendering and final delivery continue that same ledger;
there is no inspect-envelope import, fresh per-round allowance or reconstruction
from scalar counters. Mandatory overflow/cancellation suppresses delivery and
checked close still precedes output. Partial/Truncated and nonauthoritative
absence persist at the final explicit batch/profile bound. No automatic frontier,
inferred NoNewEvidence or retained cross-request continuation is supplied.
Unselected/default/frontier/state/Reference families keep their existing bytes.

## Host, bounds and custody

The strict `wow-service/context-map-native-host/1` document is shared with
[context map](CONTEXT_MAP.md). It supplies explicit store registration/path,
ProjectId, complete Reference profile/view and validated native limits. It has
no scripts, interpolation, source discovery or acquisition. Host paths remain
outside semantic identities, requests, results and errors.

Host JSON is at most 16 MiB. Selected `max_input_bytes` bounds combined encoded
owner inputs, up to 64 MiB; `max_metadata_bytes` bounds the whole final service
envelope, up to 16 MiB. Positive finite `max_context_work` continues through
construction, validation and every requested rendering. Native neighbor,
explanation/evidence and direct-edge limits retain their own validation.
Reservations cover repeated custody, traces and JSON escaping; they are not RSS,
CPU-time or tokenizer measurements. Mandatory overflow refuses, without clipping.

The service pins Current once or reads the exact store generation, checks the
project/current/Reference guards, and uses retained native source provenance and
graph read proof. Project-root L0 retains every selected source-file projection;
there is no invented project graph node. Context consumes original typed reports;
selected native API store reopening requalifies retained native producer inputs.
Source inputs remain data and are never executed.
The private encoded draft is delivered only after checked read/store closure.
Close failure supersedes a successful draft or earlier semantic refusal.

## Wire and delivery

`ContextBuildRequest::primary` constructs the Serialize-only request;
`execute_context_build` performs the single semantic operation. Request schema
is `wow-service/context-build-request/1`; result schema is
`wow-service/context-build-result/1`, with operation `context_build`.
The request retains store/publication, `platform="omitted"`, level/root, render
selection, exact context profile and normalized renderer-profile array. Selected
L0/L1 profiles are recorded; absent guard/profile fields are omitted. The native
pack request's selected l0_profile_id binds state-root L0; its optional selected
l1_profile_id binds state-detail or Reference-facts L1 into request/pack identity.
Default
ContainersV1/EntityV1 retain None for these native pack fields, omitted from
serialization to preserve old pack bytes. Default pack/renderer profile IDs stay
the existing choices. Frontier selection binds its normalized profile/frontier and
named pack/renderer family. Progressive selection binds ordered batches and
immutable native round history under its own pack/renderer family. Absent
expansion fields are omitted. No source, producer, storage or replay selection
is introduced.

The envelope retains request identity/bytes, status, resource closure and optional
failure/payload. Payload contains resolved request, publication/store generation,
configured registration, optional native store namespace and Current observation /
digest, `semantic_pack`, its exact canonical `semantic_pack_byte_length`,
`rendered_artifacts`, continuing `operation_budget_usage` and
`validation="exact_native_binding_and_canonical_bytes"`.

Intent identity includes renderer selection. Resolved identity additionally binds
publication/store generation, exact universe, root and limits. Renderer selection
is excluded from the semantic pack recipe: changing only renderer selection keeps
the same pack bytes/ID. Rendered artifact identity binds its pack, exact profile,
content digest and trace; it is distinct from the digest of rendered content.

JSON content is the exact canonical pack UTF-8 without trailing LF. Markdown uses
fixed, LF-ended typed records and escaped quoted canonical JSON metadata, with no
source excerpts. Trace ranges are zero-based half-open UTF-8 content byte ranges.
JSON has one whole-pack trace record; Markdown has one for each fixed metadata
record and each context item, including its LF. These are not source spans or
byte ranges in the outer service envelope.

Stdout is the finalized canonical service envelope followed by one LF, including
finalized Failed/Cancelled results. The application performs no retry. Partial /
Truncated exits 2; semantic refusal 3; internal/canonicalization, checked-close or
output failure 4; argument/host transport failure 64; cancellation 130.
`resource_closure` preserves `not_acquired`, `closed`, `failed` or `not_evaluated`;
unknown native closure is not treated as success.

Every returned pack is Partial or Truncated with `absence_authoritative=false`.
Original producer omissions remain scoped to their embedded maps/layers. Native
additional recipe admission, broader Reference/type enrichment, excerpts/tokens,
automatic expansion/retained cross-request continuation, platform composition and
complete-corpus/runtime qualification remain open. The [default-profile synthetic fixture](../../crates/wow-service/e3/native-primary-pack/README.md)
freezes the actual pack/render bytes; broad E3/Tier A/X acceptance remains open.
Executed gate evidence belongs to the canonical #122 milestone.

Ordinary physical Main+Library may explicitly select the
[Primary callable files input](PRIMARY_CALLABLE_INPUT.md). Its retained native
declaration/signature observations reach this same pack route; omitted selections
still remain NotSelected and incomplete observations keep their uncertainty.
