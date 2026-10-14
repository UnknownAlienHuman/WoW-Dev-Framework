# Native context build

The Primary `wow context build` selection acquires one registered immutable
project, builds a sealed L0/L1 semantic pack and optionally renders that same pack.
It omits platform; the [explicit combined selection](#selected-combined-native-build)
retains separate original Primary/platform owners. The
[owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_PACK.md)
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
  [--continuation-retention-profile-id wow-service/context-continuation-retention/1]
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

## Exact local source excerpt

Select `--context-profile-id wow-context/primary-source-excerpt-pack/1`,
`--level l1` and one exact `graph-entity@` root with default EntityV1 detail.
Supply these three options together:

```text
--source-excerpt-handle <original-root-source-handle-id>
--source-root <explicit-Main-source-directory>
--source-policy local-project-source
```

Optional renderers are `wow-context/primary-source-excerpt-json/1` and
`wow-context/primary-source-excerpt-markdown/1`; output is `envelope-json`.
The selected handle must be an original physical ByteRange in the root's
admitted native support. The existing Project reader verifies the whole file's
digest and length before returning that range. Source work and report/item/pack
retention are reserved on the continuing Context budget before opening the
directory. The directory closes before adoption, and retained owners close
before delivery. Files are bounded to 16 MiB, excerpts to 64 KiB, with the
caller's remaining budgets also enforced.

The policy records an operator-declared `LocalProjectSource` materialization
label and permission for local use. It grants no external transmission or
license attestation. Source text is a separate untrusted item. Markdown retains
quoted `SOURCE_METADATA`, generated `SOURCE_DATA_BEGIN`/`SOURCE_DATA_END`, and
numbered JSON-quoted `SRC` lines; source cannot create a boundary or policy record.
Unknown or incomplete options, incompatible profiles, wrong-root handles,
unverified files, exhausted budgets and cancellation refuse without a payload.

The route supports omitted platform and default L1 only. Both, other L1 recipes,
frontier/progressive expansion, adjacent ranges, source tokenization and
continuation remain outside this selection. Omitting the source options
preserves the existing metadata-only request, pack and renderer bytes.
Selected native/Windows CLI qualification is recorded in #122; broader source
and E3 acceptance remains open.

## Native XML handler L1 metadata in packs

Existing default EntityV1 L1 build selection carries the original handler facet
from [native inspection](../../crates/wow-context/e3/NATIVE_PRIMARY_INSPECT.md#native-xml-handler-l1-metadata)
into the semantic pack and JSON/Markdown records. Original script/unit/document/
semantic_context fields and accepted support remain source observations; runtime
binding and effective receiver remain `NotEvaluated`. Handler-containing
L1/pack bytes and IDs intentionally change. Nonempty off-map handler provenance
also incurs traversal/published-node lookup work. The uniquely joined original
script source and mandatory handler-root row remain in the retained L1.
Structural pack and
existing renderer families remain in use; no new flag/profile is added.
Serialize-only output does not establish durable decoder/legacy handler
compatibility. L0 and persisted Project/registry/replay stay unchanged;
continuing finite caller-selected limits still apply.

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
remain in use; handler-bearing packs gain the handler metadata described above.

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
this selection. Its default None is omitted; no new
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
this recipe into pack identity; omitted/default selection preserves profile IDs
and scoped omissions, including existing state-detail behavior. Persisted
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
Mandatory overflow suppresses the payload. The default family is retained and
absent selection fields are omitted. The focused source-absent lifecycle and actual
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

Without a continuation retention profile, one build operation executes all
explicit progressive rounds and returns the existing one-operation result. With
`wow-service/context-continuation-retention/1`, Build issues only the first
explicit round and stores the remaining caller-selected rounds. The native host
uses schema `wow-service/context-map-native-host/2` and includes an absolute
`continuation_store_file` alongside its registered ProjectStore. The result
carries a stable continuation ID only after the exact
generation retention root and completed Store operation are read back.

`wow context continue` reopens that exact generation, verifies the completed
issuer and retention root, then replays each previously delivered native prefix
through the existing progressive owner. Each replay checks the exact pack and
rendered bytes before creating a non-serializable Context Budget receipt from
the closed Service envelope. Those owner receipts are conservatively carried
into the next round; the service never accepts a scalar allowance or decoded
native step. The selected round count stays within the existing four-round
profile. Current is never resolved during continuation, and the final round
releases the exact retention root after its result operation is complete.
Incomplete, stale, cancelled, unknown-issuer and over-budget paths return no
continuation token. Default progressive request and pack bytes remain unchanged.

## Selected combined native build

```text
wow context build --platform combined --native-host-config <host.json>
  --combined-request-input <build-request.json> [--output envelope-json]
```

Use the strict combined host/request families described by the
[combined owner contract](../../crates/wow-context/e3/NATIVE_COMBINED_MAP.md#selected-combined-semantic-pack-and-rendering).
The request selects `operation="build"`, outer profile
`wow-context/combined-native-one-hop/1`, exact Primary generation/publication/root
and the original platform Map request with exact candidate and native binding.
The required non-null `build` object has `level` (`l0|l1|both`),
`context_profile_id="wow-context/combined-native-structural-pack/1"`,
`renderer_profile_ids` and optional `l0_profile_id`, `l1_profile_id` or
`platform_root_facet`. L0/L1 recipe fields cannot coexist with each other;
either valid recipe may select a platform facet. Explicit `l0_profile_id="wow-context/primary-state-root-l0/1"`
requires L0/GraphEntity and the original native StateRootV1 kind/provenance/TOC
custody. It emits only L0 through the same navigation map and continuing Budget;
state_path refuses with checked closure. Other explicit L0 profile/level/root
combinations refuse before acquisition. Omission of all recipe/facet fields preserves the
existing default Primary recipe with unchanged request/artifact bytes. The only
accepted non-null `l1_profile_id` values are `wow-context/primary-state-detail-l1/1` and
`wow-context/primary-reference-facts-l1/1`; either accepts `level="l1"` and an
exact GraphEntity root. StateDetail also accepts `level="both"` for a native
state_root, with original StateRoot L0 followed by StateDetail L1 on the same
detail map and continuing Budget. State_path Both refuses after native kind
validation and checked closure. ReferenceFacts also supports Both over an exact
accepted native API root: typed Reference-root L0 then original ReferenceFacts L1
on the same detail map/root/Budget. Wrong-kind or missing custody refuses with
checked closure. Either L1 profile with level L0, project roots,
null, non-string, default ID and unknown IDs refuse before owner acquisition. Mixed
operation fields also refuse. Optional `platform_root_facet="callable_metadata"`
is independent of any valid default or explicit Primary recipe. It adds the
exact platform query root's original native callable metadata to Inspect origins
under the same continuing Budget. Acquired wrong-kind or missing native custody
refuses with checked closure. Original states/support remain qualified to that
root; None/JSON/Markdown retain the same original facet and semantic pack identity.
Null/unknown values refuse before acquisition; omission preserves existing bytes.
The alternate `platform_root_facet="xml_handler_metadata"` selects an exact native
`xml_source_handler` and retains its original inline-handler/script-source
metadata, handle/evidence and support in those same Inspect origins. It uses the
same admission, continuing Budget and None/JSON/Markdown identity rules without
source bodies or inferred receiver/runtime dispatch. The existing callable
selection preserves its bytes.
The closed `platform_root_facet="xml_declaration_metadata"` selects only an
exact accepted native `xml_source_declaration` root. Its original
`ProjectGraphXmlDeclaration` must match the producer assertion and accepted
final root; source, evidence and root support remain those of the held
`SkeletonInputView`. The selected facet preserves declaration identity and
source span custody only, without XML contents or schema/classification/type/
inheritance/parent/runtime/bridge inference. It composes with each valid Primary
recipe in the same Inspect origins, semantic pack and renderer path under the
cumulative Budget. Missing, mismatched, ambiguous or incomplete custody refuses
with checked closure. Omission preserves existing bytes.

The closed `platform_root_facet="xml_reference_metadata"` selects only an
exact accepted native `xml_source_declaration` root. Declaration identity/span,
source/evidence and root support remain bound to the held `SkeletonInputView`.
The projection joins its per-root `XmlDeclarationSite` and `XmlReferenceRecord`
values to applicable original `ProjectGraphXmlReference` receipts through exact
package, load-plan, occurrence, content-digest and owner `source_path` custody.
Receipt documents are package-qualified; report/site documents are package-local
and bind through that exact owner mapping. Preserve parent/inherits kinds,
ordinals, spans, names, targets/name groups, resolution/order/cycle/ambiguity
and receipt outcomes. `ProducerOutcomeProjected` is a producer proposal, not an
accepted graph relation or runtime/effective class, type, parent, inheritance or
bridge fact. No XML text is parsed or emitted. Reuse the original declaration,
map, read and cumulative Budget; do not acquire another view. The required
Inspect/Build consumer carries the held view through origins, semantic pack and
renderers. Missing, mismatched, ambiguous or incomplete custody refuses;
existing facets and omission retain bytes/counters. Positive integrated
reference-fixture and runtime qualification remain NotEvaluated.
StateDetail retains its original nested
`wow-context/l1-skeleton/primary-state-detail/1` schema and artifact ID;
ReferenceFacts retains `wow-context/l1-skeleton/primary-reference-facts/1`,
including original origins and identity through pack and renderers. The
combined request/result, combined pack and renderer families remain /1; no
additional argv flag or request/read/map operation is introduced.
The renderer array is required: `[]` selects None; one selects
`wow-context/combined-pack-json/1` or `wow-context/combined-pack-markdown/1`; both
unique IDs render JSON then Markdown. Project roots select L0 only; original
entity/root/layer admission remains mandatory.

Primary root/level/profile/renderer overrides, mixed platform options, duplicate
options, unknown selections and text/artifact stdout refuse. Inspection on Build,
Build on Map/Inspect, explicit null and duplicate/unknown/mixed renderer IDs refuse.
The CLI checks the operation against Build and invokes one service operation.
It does not import decoded maps/packs or make another inspect/platform query.

Original maps/skeletons/platform inspection stay in pack `core.origins`; items
retain qualified owner/universe/generation/scope/origin-map identities and original
node/edge records with scoped support/omissions. One continuing construction Budget covers Map→Inspect→
Pack→validation→JSON/Markdown validation; earlier acquisition/universe/catalog work
keeps its existing owner bounds. ReferenceFacts origin preparation and L1 origins
continue on the construction ledger through pack and rendering. None still
validates the pack. Renderer choice does not change semantic pack bytes/ID. JSON
is the exact canonical pack; Markdown
uses escaped typed metadata and exact traces, with no source bodies.

Final envelope delivery uses the actual post-render remainder and checked reverse
close. Refusal/cancellation/failed closure returns no successful prefix or
pack-only fallback. Stdout is exact finalized service JSON plus one LF; Partial/
Truncated exits 2, semantic refusal 3, internal/close/output failure 4, transport
failure 64 and cancellation 130. Partial/Truncated and nonauthoritative absence
remain; shared Reference association does not prove enrichment, cross-universe
links or runtime behavior. Gate evidence belongs to #122; no acceptance is claimed.

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
serialization. Default pack/renderer profile IDs stay
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
Original producer omissions remain scoped to their embedded maps/layers. The
selected retained continuation route preserves the existing four-round profile
through exact Service issuance and native replay. Additional recipe admission,
broader Reference/type enrichment, excerpts/tokens, automatic frontier selection,
broader platform/combined facets and complete-corpus/runtime qualification remain
open. The [default-profile synthetic fixture](../../crates/wow-service/e3/native-primary-pack/README.md)
freezes the actual pack/render bytes; broad E3/Tier A/X acceptance remains open.
Executed gate evidence belongs to the canonical #122 milestone.

Ordinary physical Main+Library may explicitly select the
[Primary callable files input](PRIMARY_CALLABLE_INPUT.md). Its retained native
declaration/signature observations reach this same pack route; omitted selections
still remain NotSelected and incomplete observations keep their uncertainty.
