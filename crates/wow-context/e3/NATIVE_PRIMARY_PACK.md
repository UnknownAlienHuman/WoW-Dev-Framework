# Native Primary semantic pack and rendering

This page describes the selected owner implemented in [`pack.rs`](../src/pack.rs)
and [`render.rs`](../src/render.rs). It extends the genuine
[Primary L0/L1](NATIVE_PRIMARY_INSPECT.md) route without declaring full E3-B
acceptance. [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122)
is the canonical tracker; broader request/profile and L0/L1 contracts remain open.

## Closed profiles and selection

The [reviewed descriptors](NATIVE_PRIMARY_PACK_PROFILES.md) are frozen in
[profiles.json](../../wow-service/e3/native-primary-pack/profiles.json).
Compiled `PRIMARY_CONTEXT_PROFILES_BYTES` must match those canonical descriptors.

| Profile | Selected contract |
|---|---|
| `wow-context/primary-structural-pack/1` | One exact root, explicit L0/L1/Both, mandatory original native entity/relation items |
| `wow-context/primary-pack-json/1` | Lossless canonical pack JSON, UTF-8, no trailing LF |
| `wow-context/primary-pack-markdown/1` | Lossless fixed typed metadata records, UTF-8, LF-ended |

L0 accepts the acquired ProjectId or a native source-file/source-package graph
container. L1 accepts an exact native graph entity; Both shares one explicit
container admitted by L0. L0 direct relations remain its seven structural kinds,
even when Both shares L1's broader detail-origin map. Existing native direct
queries supply records and truncation; packing does not expand paths.

Explicit PrimaryL1Profile::StateDetailV1 selects the existing
[ordinary TOC state-detail L1](NATIVE_PRIMARY_STATE_DETAIL.md) for non-frontier
L1 build over an exact native state_root/state_path GraphEntity. Project,
L0/Both, other root kinds and frontier selection refuse. Original root/path,
literal keys, plural supporting declarations and custody remain mandatory;
missing/truncated required support refuses rather than yielding an empty facet.
This uses the existing structural pack and JSON/Markdown family.

## Native construction and immutable records

After `bind_context_universe_set`,
`build_context_semantic_pack(universe, map, graph, root, level, stop)` consumes the
exact bound universe, immutable ProjectMap, native GraphPartitionRead and typed
L0Scope/SkeletonLevel.
`build_context_semantic_pack_with_l1_profile(universe, map, graph, root, profile, stop)`
selects L1 over those same native owners. StateDetailV1 carries the actual Budget
returned with its native skeletons into pack construction; it does not rebuild
from an inspect artifact or derive a new ledger from serialized usage.
`validate_context_semantic_pack(universe, &mut pack, stop)` checks native binding
and canonical bytes using that continuing budget.
`ContextSemanticPack` is sealed and Serialize-only. Its public accessors are
`semantic_pack_id()`, `canonical_bytes()`, `state()` and
`operation_budget_usage()`; serialized bytes cannot reconstruct native custody.

Pack schema is `wow-context/semantic-pack/primary/1`; the owner request schema is
`wow-context/primary-pack-request/1`. The core retains the frozen profile, exact
universe/binding/request, boundary notice, original PrimarySkeletons, typed
items, selection trace, scoped omissions, budget and state.
Optional selected l1_profile_id in the nested native pack request binds
StateDetailV1; EntityV1 retains None and omits the field, preserving old bytes.
Validation requires the exact selected root, L1 profile/descriptor and schema.
State-detail schema/profile and structural pack/render versions remain unchanged.
Original layer maps preserve project/configuration/graph/Reference identities,
source/artifact handles and digests/spans, package/TOC/load custody, original
support/evidence, confidence, coverage and conflicts. Optional platform absence
remains explicit; context does not create richer source authority.

Every node and edge in the selected native map becomes an Entity or Relation
item borrowing its original GraphNode or GraphEdge. Nodes precede edges, each in
native ID order. An item retains exact root/universe/origin-map binding, payload
digest, inclusion reason and privacy/license/source-boundary labels. Selection
trace binds its item ID, origin map and reason with `required=true`. A project
root remains the acquired ProjectId and original source-file inventory, without a fabricated
project graph node or contains edge. No mandatory item is silently dropped to fit.

The semantic pack ID binds the typed core under its schema domain. Renderer
selection, validation reports and later delivery accounting are excluded.
Original skeletons/maps remain embedded native artifacts with their own scoped
identities and omissions. Selected callable declaration/signature observations
keep their original recipe, order and uncertainty; unselected Primary recipes
remain NotSelected. Function bodies, source-shape and excerpts are excluded.

## Rendering and byte traces

`render_context_pack_json(&mut pack, stop)` and
`render_context_pack_markdown(&mut pack, stop)` return Serialize-only
`RenderedContextArtifact` presentations. They continue the same private ledger;
pack core, canonical bytes and semantic identity stay fixed.
`validate_rendered_context_artifact(&mut pack, &artifact, stop)` checks pack /
profile binding, ordered semantic targets, exact content/digest, trace ranges,
UTF-8 boundaries and artifact identity. No imported JSON becomes native authority.

Artifact core schema is `wow-context/rendered-context-artifact/primary/1`.
It retains semantic_pack_id, exact renderer descriptor, content, byte_length,
content_digest, `content_digest_basis="sha256_exact_rendered_utf8_bytes"` and trace.
The rendered artifact ID hashes the typed core under its schema domain; raw
content SHA-256 is a separate identity. Accessors expose those IDs, profile,
content, byte length, content digest and trace.

JSON emits `pack.canonical_bytes()` exactly and has one `canonical_pack_json`
trace record covering the whole content. Markdown emits PACK, PROFILE, UNIVERSE,
REQUEST, BOUNDARY, SKELETONS, each ENTITY/RELATION, SELECTION_TRACE, COVERAGE and
BUDGET in the frozen order. Payloads are canonical typed JSON represented as
quoted JSON strings. Quotes/backslashes, ASCII punctuation, controls, Unicode
line separators and bidi controls are escaped; metadata cannot manufacture
Markdown templates or boundaries. Both profiles declare no rendering loss.

Each ContextRenderTrace retains record/template IDs, semantic field targets,
optional item_index and zero-based half-open UTF-8 byte_start/byte_end. Markdown
ranges include the complete fixed record and LF. Ranges refer to content bytes,
not outer artifact JSON, source spans or character offsets. Trace does not grant
source inclusion permission. Requested renderers are required; refusal cannot
clip output or return a different semantic selection.

## External default Primary L0 rendering

`validate_and_render_primary_l0_pack` admits the same finite StructuralOnly
external JSON input as `validate_primary_l0_pack`, then renders JSON or Markdown
through the existing native templates. `PrimaryPackRenderResult` exposes the
validation report, optional artifact and cumulative `total_work()`. Invalid or
NotEvaluated input has no artifact. A success-only private decoded view preserves
the original pack without creating native owner custody.

The decoder's canonical-byte metadata reservation is made before decoding and
carried into the render budget with the already-consumed validation work.
Rendering cannot reset either allowance. Native and external artifacts retain
identical content, digest, trace and artifact ID for the same supported pack.
This route covers default L0 source_file/source_package GraphEntity metadata;
broader levels, recipes, excerpts and exact owner/runtime closure stay deferred.

## Cumulative limits and honest coverage

Construction continues the skeleton Budget. Native work and encoded reservations
cover selected records/support, repeated custody, validation, renderer plans,
content/trace retention and artifact envelopes, with cancellation checkpoints.
`core.budget.reservation_basis="before_semantic_pack_finalization"` records its
pre-finalization snapshot; exact final pack length comes from canonical_bytes.
The skipped private ledger continues through rendering/validation;
operation_budget_usage uses `before_service_delivery`; the service separately
bounds its final delivery envelope. These values are bounded
work/encoded reservations, not physical memory, elapsed time or exact tokens.
No renderer resets limits. For selected state-detail build, the final service
envelope must fit the actual remaining metadata allowance after all requested
rendering/validation. Mandatory overflow or cancellation refuses.

Returned state is Partial or Truncated and `absence_authoritative=false`.
Upstream omission references keep producer/artifact identity, interpreted as
`producer_local_omissions_retained_in_original_artifacts`; they do not negate a
selected layer's own evaluated scope. Pack NotEvaluated fields cover Reference
fact enrichment, declaration/signature qualification, control-effect projection,
source excerpts/tokens, deeper expansion/continuation, optional platform universe
and complete-corpus/runtime. Privacy is local_metadata_only, license is
source_inclusion_not_permitted, tokens are unavailable and continuation is
not_evaluated. Partial/truncated coverage never proves absence or a clean negative.

The [service/CLI route](../../../apps/wow/CONTEXT_BUILD.md) pins publication,
validates guards, requires explicit profiles and closes native resources before
delivery. The [selected synthetic fixture](../../wow-service/e3/native-primary-pack/README.md)
freezes exact default bytes. State-detail build has bounded source-absent lifecycle
and Windows CLI verification; broad E3/Tier A/X acceptance remains open. This contract does not claim source
permission, model/token fit, runtime semantics, lineage or full package acceptance.
