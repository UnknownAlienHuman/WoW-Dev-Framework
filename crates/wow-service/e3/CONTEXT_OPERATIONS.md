# E3-C context operation contracts

**Status:** normative transport-independent service behavior.

## Common request processing

Every operation follows the applicable subset of:

```text
validate service configuration
-> validate and canonicalize service request
-> resolve configured profile aliases to exact IDs
-> resolve symbolic publication selectors once
-> acquire exact retained owner views
-> validate capabilities and compatibility
-> bind/validate ContextUniverseSet when required
-> construct exact wow-context request
-> invoke one declared context operation plan
-> validate returned semantic/render artifacts
-> derive service status and draft envelope
-> close resources in reverse order
-> finalize canonical result or failure/cancellation
```

No operation refreshes current, searches for roots, retries against another generation, or invokes undeclared lanes.

## `context_status`

Implemented slice: Primary retained-publication metadata status. This is a bounded
availability/identity report, not a context-generation or evaluation operation.
The result remains `partial`; context generation and evaluation remain
`NotEvaluated`.

Request:

```text
explicit native host registration
primary project store registration
publication = Current | exact StoreGeneration
optional exact current-record and/or current-digest guard (Current only)
optional complete exact Reference profile/generation/view guard
platform = omitted
detail = summary | capabilities | profiles | generations (default summary)
host metadata/input/work budgets and cancellation
```

The CLI command is `wow context status`. It accepts an explicit host config,
store ID, `current` or `store-generation:<ID>`, the four details above, optional
Current and Reference guards, and only `envelope-json` output. It does not accept
publication-set selectors, roots, map/build/profile
options, or artifact/text output. The app constructs one typed Service request;
only Service resolves Current.

Service resolves Current once or opens the requested exact retained generation,
checks the optional expected-current guard against the single acquired Current
observation, checks the exact Reference guard when supplied, binds the retained
Primary project/graph and configured exact Reference view, and charges bounded
metadata work. A stale Current guard or wrong Reference guard fails with checked
resource closure. An exact-old generation stays selected even when Current has
advanced; Current metadata in the result records the observation made with that
read.

The result reports exact publication/store, project, graph, and Reference
identities; project/reference/graph coverage and conflict summaries; and finite
capability/profile availability. Detail is closed: `summary` omits optional
detail arrays, while each of `capabilities`, `profiles`, and `generations`
includes only its matching array/object. Generation detail includes project and
graph snapshot/generation IDs, graph universe/registry digest, source registry
ID, and analyzer snapshot ID.

For this Primary-only implementation, `context_generation`, `evaluation`,
`last_known_good`, and `failed_targets` are `NotEvaluated`; continuation
retention is `not_implemented`; optional platform status is `not_selected`.
Capability entries mark the implemented operations and the deferred
continuation/validate/render operations separately. Profile availability means
the profile is registered for a selected operation. Platform and combined
profiles report `requires_platform_selection` for this Primary-only request.
Availability does not establish
root-specific completeness, successful context generation, or runtime
behavior.

The status path reads metadata from the acquired owners only. It never calls
Project Map, skeleton, semantic-pack, source-excerpt, or renderer operations and
returns no map/skeleton/pack/source/render artifact or source body. Success is
finalized only after checked read/store closure. Budget, cancellation, guard,
identity, and closure failures return a failure/cancelled envelope without a
success payload. This Primary route does not select platform metadata.

The separate `execute_context_platform_status` operation accepts
`wow-service/context-platform-status-request/1`: an exact registration,
store generation, publication set, candidate ID/digest, platform one-hop profile
and closed detail string. It reacquires the original namespace/candidate and
checks configured Project/Reference compatibility without opening a root or
skeleton view. Original coverage remains distinct; Primary/combined operations
require Primary selection. Its optional Reference guard is a complete exact
tuple. The result remains Partial with root coverage and generation/evaluation
NotEvaluated and checked closure. See [the CLI route](../../../apps/wow/CONTEXT_STATUS.md).
Full E3-C acceptance remains pending.

The `context_combined_status` operation uses request schema
`wow-service/context-combined-status-request/1` and result schema
`wow-service/context-combined-status-result/1`. Its exact request selects a
Primary registration/store generation/publication set and a platform
registration/store generation/publication set/candidate ID/digest/profile,
plus one closed detail value. It has no Current selector or root; the explicit
combined host supplies the shared Reference view and limits. The
`execute_context_combined_status` operation reacquires both exact retained
selections, checks candidate and shared Reference compatibility, and serializes
separate Primary/platform metadata with the shared Reference identity. It does
not invoke a root, map, skeleton, pack, excerpt, or renderer. Success is Partial
only after checked platform-then-Primary closure; root coverage, context
generation, evaluation, last-known-good, and failed targets remain
`NotEvaluated`.

Both configured Reference copies validate under one cumulative work budget;
matching claimed digest fields alone are insufficient. The metadata preserves
the platform namespace ID and both original Project/Graph identity tuples,
without creating a context-universe binding or invoking context engines.

## `context_map`

Purpose: return a deterministic E3-B `ProjectMap` for the exact acquired primary project, optional Blizzard UI project, or explicit combined-map profile.

Request:

```text
publication selectors and reference guard
map target = Primary | Platform | Combined
exact ProjectMapProfileId or configured alias
optional exact root scope accepted by the map profile
confidence/coverage/privacy policy IDs
budget/cancellation
```

Behavior:

- bind one exact `ContextUniverseSet`;
- derive the exact project root only from the acquired view when no narrower exact map root is supplied;
- call `wow-context build_project_map`;
- validate map identity/origins/coverage/omissions/budget;
- return the unchanged `ProjectMap` in `ContextMapPayload`.

No direct Markdown rendering is invented by service. Human text projection belongs to the application and must use only service/map records. A full context renderer is requested through `context_build` or `context_render`.

## `context_inspect`

Purpose: build exact L0 and/or L1 structural detail for already-resolved roots.

Request:

```text
publication selectors/reference guard
one or more exact ContextRootSelector values
level = L0 | L1 | L0AndL1
exact L0/L1/intent/expansion profiles
requested facets/axes/relations
source-excerpt candidate policy
budget/cancellation
```

Behavior:

- reject fuzzy/name/path/natural-language selectors;
- bind exact universes;
- call `build_l0_skeleton` and/or `build_l1_skeleton` in frozen order;
- optionally invoke bounded expansion only when declared by the inspect profile;
- validate roots, source spans, relation/path distinction, evidence/coverage/conflicts/omissions;
- return unchanged skeleton artifacts and optional continuation.

Inspect does not construct a full `ContextSemanticPack` unless the request explicitly selects the build operation instead.

## `context_build`

Purpose: build one E3-B `ContextSemanticPack` and optionally one or more exact rendered artifacts.

Request:

```text
publication selectors/reference guard
exact root selectors
exact intent/map/L0/L1/expansion/selection/budget/tokenizer/privacy/source-boundary profiles
zero or more exact renderer profile IDs
continuation retention policy
budget/cancellation
```

Behavior:

1. bind exact universe set;
2. validate E3-B profiles/request;
3. invoke map/skeleton/expansion/source/selection/pack operations through the single context-engine operation contract;
4. validate the semantic pack;
5. invoke requested context renderers only after semantic validation;
6. validate each rendered artifact;
7. if continuation exists, admit exact generation-retention roots before advertising it;
8. close ordinary resources;
9. return `ContextBuildPayload`.

Renderer failure cannot mutate or relabel the semantic pack. Policy decides whether a validated semantic pack with a failed optional renderer yields `partial` or service failure; the exact decision is frozen per renderer request class.

### Selected combined native inspect/build

The optional non-null `platform_root_facet="callable_metadata"` selects only the
exact platform root's original native callable metadata independently of the
valid default or explicit Primary recipe. Ordinary Primary level/root
admission stays in force. Native acquisition verifies `lua_source_function`,
original producer assertion and accepted final-node membership before retaining
the original metadata/handle/evidence and existing root support. It uses the held
input view and continuing Budget, with joins/lookup precharged and no second read.
Wrong kind or missing custody refuses with checked closure. Omission preserves
existing bytes. Combined Inspect identity and consuming Build origins include
the selected facet; renderers preserve original states and semantic-pack identity.
Original native-page omissions remain page-scoped; no Reference/cross-universe
link or runtime claim is added.

The closed alternate value `xml_handler_metadata` selects an exact native
`xml_source_handler` under the same admission/closure/Budget contract. It retains
original `ProjectGraphInlineHandler`, matching `ProjectGraphScriptSource`, native
handle/evidence and root support without source bodies or inferred receiver/runtime
dispatch. The new facet stays in original Inspect/Build origins; omission and
existing callable-selected bytes remain unchanged.

The closed value `xml_declaration_metadata` selects only an exact accepted native
`xml_source_declaration` root. The original `ProjectGraphXmlDeclaration` must
match its producer assertion and accepted final root, with the original source,
evidence and root support retained from the held `SkeletonInputView`. The facet
preserves declaration identity and source span custody only; it adds no XML
contents, schema/classification, type, inheritance, parent, runtime or bridge
inference. It composes independently with each valid Primary recipe through the
same Inspect/Build path and cumulative Budget. Missing, mismatched, ambiguous or
incomplete custody refuses with checked closure. Omission preserves existing
bytes.

The closed value `xml_reference_metadata` selects only an exact accepted native
`xml_source_declaration` root. Its declaration identity/span must match the
original declaration, producer assertion and accepted final root. The original
source, evidence and root support remain those of the held `SkeletonInputView`.
The projection joins the per-root `XmlDeclarationSite` and `XmlReferenceRecord`
to applicable `ProjectGraphXmlReference` receipts through exact package, load-
plan, occurrence, content-digest and owner `source_path` custody. Receipt
documents are package-qualified; report/site documents are package-local and
are bound only by that exact owner mapping. Preserve original parent/inherits
kinds, ordinals, spans, names, targets/name groups, resolution/order/cycle/
ambiguity outcomes and receipt outcomes. `ProducerOutcomeProjected` is a
producer proposal; it does not establish an accepted graph relation or runtime/
effective class, type, parent, inheritance or bridge behavior. No XML text is
parsed or emitted. Reuse the original declaration, map, read and cumulative
Budget; do not acquire a second view. Missing, mismatched, ambiguous or
incomplete custody refuses with checked closure. Existing facets and omission
retain their bytes and counters. Positive integrated reference-fixture and
runtime qualification remain NotEvaluated.

The optional non-null `l0_profile_id` selects only
`wow-context/primary-state-root-l0/1` with L0/GraphEntity. It invokes the original
StateRootV1 L0 on the ordinary navigation map and continuing Budget. Native
state_root provenance/TOC support is mandatory; state_path refuses with checked
closure. Explicit L0 fields cannot coexist with an explicit L1 recipe, and L1/Both,
project/null/default/unknown L0 choices refuse before acquisition. No L1 is emitted.

The combined transport accepts the explicit L1 IDs
`wow-context/primary-state-detail-l1/1` and `wow-context/primary-reference-facts-l1/1`,
each with `level="l1"` and an exact GraphEntity root. StateDetail also admits
`level="both"`: the existing StateRoot L0 precedes StateDetail L1 under the same
continuing Budget and exact map/root. Only a native state_root is eligible;
state_path refuses after native kind validation with checked resource closure.
ReferenceFacts also admits Both for an exact accepted native API root: the typed
Reference-root L0 identity/navigation precedes original ReferenceFacts L1 on the
same detail map/root/Budget. Wrong-kind or missing custody refuses after owner
validation with checked closure. Omission of all recipe/facet fields preserves
default request/artifact bytes.
Either L1 profile with level L0, project roots, null, the default ID and unknown IDs refuse before
owner acquisition. The held native API
supplies original `reference_origins` to the existing L1 Reference engine.
Origin-vector construction, graph reconstruction, exact Reference lookup and
signature digest work are precharged to the continuing construction Budget before
the selected L0/L1 work; earlier acquisition retains its existing bounds. The original ReferenceFacts
L1 schema, origins and identity survive combined semantic-pack construction and
rendering. Request/result, pack and renderer schemas remain /1. See the
[combined contract](../../wow-context/e3/NATIVE_COMBINED_MAP.md).

## `context_continue`

Purpose: continue one previously returned exact E3-B frontier/page chain.

Request:

```text
bounded ServiceContextContinuation bytes/object
expected original request/pack IDs: optional exact guard
no publication current selector
no changed semantic profiles/roots/privacy/renderer/total budget
cancellation
```

Behavior:

- validate continuation schema/digest/size;
- resolve exact retained generations from the continuation and retention receipts;
- acquire exact views directly, never current;
- validate the same `ContextUniverseSet`, profiles, roots, frontier, selected/omitted manifests, and total budget chain;
- call `continue_context_semantic_pack`;
- validate new page/pack and requested original renderers;
- replace/release continuation retention receipts according to returned state;
- close resources and return exact continuation outcome.

If a generation or receipt is unavailable, fail. No automatic restart against current or reconstruction from names.

## `context_validate`

Purpose: nonrepairing validation of bounded artifact bytes. The first private route is limited to one semantic-pack JSON artifact under `wow-context/primary-structural-pack/1`.

Selected request contract:

```text
artifact_kind = semantic_pack
media_type = application_json
validation_profile_id = wow-context/primary-structural-pack/1
origin_closure_level = StructuralOnly
bounded input, metadata, work, depth, and token limits
cancellation
```

The selected route supports default ContainersV1 L0 over an exact source_file/source_package GraphEntity and default EntityV1 L1 over an exact native GraphEntity. L1 binds the original entity-detail-origin map and its 27-kind direct vocabulary, exact selected local entities, declarations and item/trace identities. A declaration must retain its actual producer/accepted-record/ownership/source/evidence join; selected membership alone is insufficient. Required support outside the captured neighborhood remains NotEvaluated. Both, explicit state/reference recipes, project roots, expansion, excerpts, platform/combined families, rendered inputs and ExactOwnerClosure remain NotEvaluated. StructuralOnly acquires no project/graph/reference generations.

The Service receives bytes, never the CLI path or stdin identity. It invokes `wow_context::validate_primary_pack` once and carries the owner report unchanged in the result envelope. For explanation chains, the owner recomputes derivation record IDs from each record plus its partition ID/digest and closes local assertion references within the same partition. It admits only complete, bounded, conflict-free chains closed over their witnesses; missing records/references, conflicts, cycles, incomplete chains, and cross-partition references remain `NotEvaluated`. Declaration links join original producer assertions to accepted graph records, then rebind original node IDs to the published graph generation; ownership edges retain original relation, confidence, and evidence while using published endpoints. `Valid` means the selected structural schema/profile, pack identity, internal references, and supported local chains passed; it does not establish native owner-generation closure, native producer/catalog authority, source authority, or runtime behavior. Native producer/catalog authority, exact owner-generation closure, source authority, and runtime authority stay `NotEvaluated`; producer/catalog authority is not a StructuralOnly positive prerequisite. Unsupported selections remain `NotEvaluated`.

Decoded Graph coverage, Core coverage and deferred records are re-admitted by
their existing owners before `Valid`. Project records bind the original source
context and native fixed partitions; capability/partition and deferred rows
remain ordered and unique. Reference partition identifiers and conflicts use
the Reference owner grammar, canonical population and exact partition join.
These checks include retained explanation coverage. Recomputing enclosing IDs
cannot bypass a malformed owner record. This remains structural admission,
without a full source-registry, Reference catalog or runtime authority claim.

The public Service operation status is `complete` when the owner returns any of `Valid`, `Invalid`, or `NotEvaluated`; the owner report is carried unchanged as the payload. `Valid` is limited to the selected StructuralOnly checks. `owner_generation_closure` and `source_or_runtime_authority` remain `NotEvaluated`. The typed L0 root is a source_file/source_package GraphEntity; default L1 preserves its exact native GraphEntity root. Service receives artifact bytes and typed identity data, never the CLI filesystem path.

`max_metadata_bytes` is the caller-provided cap for the complete serialized result envelope, subject to the shared context result ceiling; the owner also applies it to input/metadata admission. Service hashes the input in 8 KiB chunks and subtracts `ceil(input_bytes / 8192)` from `max_context_work` before passing the remaining cumulative allowance to the owner. The owner charges its own input hash, scanner, typed decode, and wire traversal against that remaining shared budget. Its `max_tokens` counts syntactic JSON scanner tokens, not model tokens. Limit exhaustion and owner/invariant errors produce typed failed operation outcomes; cancellation produces a typed cancelled outcome. A genuine positive StructuralOnly witness and affected-owner Tier S are required before publication. The independent Quick gate runs on the published exact head; no Quick result is claimed here. Full E3-C acceptance remains pending.

## `context_render`

Purpose: render one already validated semantic pack under one exact renderer profile without changing semantic selection.

### Selected native route

`ContextRenderRequest::primary_semantic_pack_structural` selects JSON or Markdown
for the same finite external Primary L0 GraphEntity input supported by
`context_validate`. `execute_context_render` hashes the supplied bytes, subtracts
that work and invokes `validate_and_render_primary_l0_pack` exactly once. It
borrows the unchanged validation report and optional artifact into its bounded
envelope. Invalid and NotEvaluated reports carry no artifact or null placeholder.
The payload's `work_usage` reports Service input-hash work, owner validation/render
work and their checked cumulative total under the original caller allowance.
The operation completes only after the selected owner call and envelope finish;
its validation state remains distinct from operation status.

One typed decode is retained only after successful structural admission. The
existing native renderer templates, profiles, content, traces and artifact IDs
are reused. Decoder metadata reservation and validation/render work continue
under the same owner bounds. No native custody or exact owner/source/catalog/
runtime authority is reconstructed. L1/Both, source excerpts, expansions,
platform/combined families and ExactOwnerClosure remain unsupported here.

### Broader contract (acceptance remains open)

Request:

```text
bounded canonical ContextSemanticPack bytes/value
exact RendererProfileId
exact tokenizer/framing/output policy
validation/origin-closure level
budget/cancellation
```

Behavior:

1. validate semantic pack structurally;
2. acquire exact owner views only when the validation profile requires origin closure;
3. validate the semantic pack against those exact origins;
4. call the exact `wow-context` renderer;
5. validate artifact bytes, item mappings, source boundaries, privacy/license, and token accounting;
6. close resources;
7. return `ContextRenderPayload`.

No renderer-specific semantic replan occurs unless the input is a new explicit `context_build` request with a distinct semantic profile and pack ID.

## Common outcome rules

- useful artifact + budget stop = `truncated` with continuation/omissions;
- useful artifact + nonbudget incomplete requested scope = `partial`;
- no useful artifact because required capability was legitimately unevaluable = `not_evaluated` only when the operation contract permits;
- invalid request, missing exact generation, incompatible bindings, security violation, invalid owner result, or close failure = `failed` result family;
- cancellation before publication = `cancelled` result family;
- no empty/default success;
- no source, lease, continuation internals, or private path in errors.
