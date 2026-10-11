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
  --context-profile-id wow-context/primary-structural-pack/1
  [--l0-profile-id wow-context/primary-l0/1|wow-context/primary-state-root-l0/1]
  [--l1-profile-id wow-context/primary-l1/1]
  [--renderer-profile-id wow-context/primary-pack-json/1]
  [--renderer-profile-id wow-context/primary-pack-markdown/1]
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
The L1 option remains an optional exact assertion for the selected level.
Current guards require `--publication current`; Reference guards form one complete
three-option group. Unknown options/profiles, duplicate options and incompatible
level/profile assertions refuse before invocation.

Each renderer option may occur once. No renderer selects `none` and returns a
validated semantic pack with an empty `rendered_artifacts` array. One selects JSON
or Markdown; both select JSON then Markdown regardless of argument order.
There is no `--render` flag, standalone render/import route, auto profile selection
or text/artifact stdout mode. Requested renderers are required: refusal suppresses
the delivery payload rather than returning a pack-only fallback.

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
there is no invented project graph node. No source is reparsed or executed.
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
pack request's selected l0_profile_id binds the explicit state-root selection
into its request/pack identity. Default ContainersV1 retains None, omitted from
serialization, preserving old pack bytes. Pack/renderer profile IDs stay the
existing choices; no source, producer, storage or replay selection is introduced.

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
additional recipe admission, Reference enrichment, excerpts/tokens, deeper expansion /
continuation, platform composition and complete-corpus/runtime qualification
remain open. The [default-profile synthetic fixture](../../crates/wow-service/e3/native-primary-pack/README.md)
freezes the actual pack/render bytes; broad E3/Tier A/X acceptance remains open.
Executed gate evidence belongs to the canonical #122 milestone.

Ordinary physical Main+Library may explicitly select the
[Primary callable files input](PRIMARY_CALLABLE_INPUT.md). Its retained native
declaration/signature observations reach this same pack route; omitted selections
still remain NotSelected and incomplete observations keep their uncertainty.
