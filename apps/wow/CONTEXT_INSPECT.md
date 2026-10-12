# `wow context inspect`

The selected Primary route calls
[`execute_context_inspect`](../../crates/wow-service/src/context/inspect.rs)
once over an existing acquired native project/graph pair and exact registered
Reference. It does not refresh source or acquire optional platform context.
The owner contract is [Native Primary inspection](../../crates/wow-context/e3/NATIVE_PRIMARY_INSPECT.md);
remaining capability/qualification is tracked in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).

## Grammar

```text
wow context inspect --native-host-config <host.json>
  --project-store <registration-key>
  --publication current|store-generation:<ProjectStoreGenerationId>
  --root project@<base64url-id>|graph-entity@<base64url-id>
  --level l0|l1|both
  [--l0-profile-id wow-context/primary-l0/1|wow-context/primary-state-root-l0/1]
  [--l1-profile-id wow-context/primary-l1/1|wow-context/primary-state-detail-l1/1|wow-context/primary-reference-facts-l1/1]
  [--expansion-profile-id wow-context/primary-native-frontier/1
   --frontier graph-entity@<base64url-id> ...]
  [--expansion-profile-id wow-context/primary-native-progressive/1
   --frontier-round '<token>[,<token>...]' ...]
  [--platform omitted] [--output envelope-json]
  [--expect-current-record <id>] [--expect-current-digest <digest>]
  [--expect-profile <id> --expect-reference-generation <id> --expect-reference-view <id>]
```

Roots encode one exact UTF-8 ID with canonical unpadded base64url. The CLI does
not infer kinds/existence or choose a root. Both `--root` and `--level` are
required; there is no omitted-root shortcut for inspect.

| Level | Accepted acquired root | Selected service profile(s) |
|---|---|---|
| `l0` default | Matching project ID, or exact source_file/source_package graph container | `wow-context/primary-l0/1` |
| `l0` state | Exact original ordinary TOC state_root GraphEntity | `wow-context/primary-state-root-l0/1` |
| `l1` default | Exact native graph entity | `wow-context/primary-l1/1` |
| `l1` state detail | Exact original ordinary TOC state_root/state_path GraphEntity; no frontier | `wow-context/primary-state-detail-l1/1` |
| `l1` Reference facts | Exact native graph entity on a genuine native_api_uses=reference_v1 publication; no frontier | `wow-context/primary-reference-facts-l1/1` |
| `both` default | One explicit source_file/source_package graph container | Default L0 + old L1, distinct artifact IDs |
| `both` state | One exact original ordinary TOC state_root GraphEntity | Selected state L0 + old L1, distinct artifact IDs |

An omitted/default L0 flag preserves primary-l0/1. To select state-root L0, pass
`--l0-profile-id wow-context/primary-state-root-l0/1` with level l0 or both and
an explicit graph-entity root. The retained ordinary selected-TOC Main/Library
owner supplies the root, every exact supporting typed declaration and selected
literal state_path members; there is no name lookup or exhaustive-field claim.
Bounded execution evidence is recorded in #122.

An omitted/default L1 flag preserves primary-l1/1; state detail and Reference
facts require explicit
selection. An L1 flag conflicts with level l0,
and an L0 flag conflicts with level l1. Unknown options, repeated non-repeatable
options and other profiles/platform/output modes refuse; map-target/profile flags are not inspect
options. Service checks actual root kind/existence, original custody and project
identity after acquisition. A method/function cannot silently become an L0
container. Existing root/profile selection remains; handler-bearing artifacts
gain the metadata described below.

Current guards require publication current. Reference guards require all three
options together. Service pins Current once or reads the exact store generation;
the CLI performs no selector resolution, search, alias lookup or retry.

## Native XML handler L1 metadata

Use existing `--level l1`, an exact `graph-entity@` root and default
`wow-context/primary-l1/1` for a retained native `xml_source_handler`.
Unreleased EntityV1 consolidates the original `script_id`, `unit_id`,
`document`, `semantic_context` and accepted node/source/evidence support.
Runtime binding and effective receiver remain `NotEvaluated`. No new flag or
ScriptDetail profile is introduced. Handler-containing L1 artifacts/pack IDs
intentionally change. Traversal and published-node lookup of nonempty handler
provenance also debit work when no handler is selected. The original script source
is retained by an exact unique support join; a handler-root row is mandatory.
L0 and persisted Project/registry/replay stay unchanged.
See [the existing inspection contract](../../crates/wow-context/e3/NATIVE_PRIMARY_INSPECT.md#native-xml-handler-l1-metadata).

## Selected native state-detail L1

Pass `--level l1 --l1-profile-id wow-context/primary-state-detail-l1/1` with an
exact `graph-entity@` root for an original ordinary TOC state_root or state_path.
This selection requires non-frontier L1: Project/L0/Both and frontier options
refuse. The same profile also has a structural [build/pack/render consumer](CONTEXT_BUILD.md#selected-native-state-detail-l1-pack).
Neither route infers a SavedVariables root from a global name.

The named L1 schema is `wow-context/l1-skeleton/primary-state-detail/1`.
It retains original roots/paths and typed literal keys, every exact supporting
TOC declaration, account/character scope and ambiguity, plus the unchanged
native direct neighborhood and source/load/evidence custody. Required native
root/path/support missing or omitted by truncation refuses; an empty successful
state facet is not a fallback. Other native omissions remain visible.

One owner ledger covers the immutable selected skeleton and its custody;
the selected final envelope must fit the actual remaining metadata allowance
after construction. Inspect behavioral
verification passes on the bounded source-absent lifecycle and Windows CLI byte
checks. The selected build consumer has its own bounded lifecycle and CLI checks.
This metadata projection does not attest runtime
values, loaded state, combat safety, API parity or current-source qualification.
See [Native Primary state detail](../../crates/wow-context/e3/NATIVE_PRIMARY_STATE_DETAIL.md).

## Selected native Reference-facts L1

Pass `--level l1 --l1-profile-id wow-context/primary-reference-facts-l1/1`
with an exact `--root graph-entity@<base64url-id>` on an ordinary publication
that genuinely selected `native_api_uses=reference_v1`. Project/L0/Both and
frontier selections refuse. The host's complete Reference profile/view must
match the held requalified native owner; optional Reference guards are additional
assertions and cannot replace that mandatory fence.

The selected L1 schema is `wow-context/l1-skeleton/primary-reference-facts/1`.
For each admitted local `native_api_callable` node, it retains the original
ReferenceRecord, record key/digest, source IDs and restriction facets, exact
accepted producer assertion/final node, original source handle/evidence and
Reference partition coverage. All five native endpoint identity fields and the
record/source association are checked against the same bound owners. An endpoint
label, decoded sidecar or inert origin input cannot supply that proof.

The genuine retained Reference catalog also supplies source-normalized ordered
parameters/returns, original type names/inner types, tri-state nilability and
original raw default/stride values with source spans and support. This is existing
normalizer output, not payload parsing or resolved types. Unknown raw fields remain
in the original ReferenceRecord. Source type resolution, corrected overlays,
annotation display lowering, broader restriction interpretation and runtime
qualification remain unevaluated.

The embedded map still reports `ReferenceContractNotEvaluated` source custody;
the selected L1 facet supplies exact record/source association and those retained
source observations,
without relabeling that map omission or including source excerpts. Default/state
profile IDs and scoped omissions are retained. The same selection has a
structural [build/JSON/Markdown route](CONTEXT_BUILD.md#selected-native-reference-facts-l1-pack).
See [the selected owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_REFERENCE_FACTS.md)
for custody, continuing limits and nonclaims; execution evidence remains in #122.

## Selected native frontier expansion

With `--level l1` and an exact graph-entity root, pair
`--expansion-profile-id wow-context/primary-native-frontier/1` with repeatable
`--frontier` tokens for 1..32 distinct exact GraphEntity IDs already in the initial
L1 origin map, excluding its root. Profile and frontier options are required
together. Invalid tokens, duplicates, empty/excess/nonmember/root frontiers,
unsupported profiles and L0/Both/Project selections refuse. The service
normalizes frontier order by native ID; it does not select a frontier by name.

Each selected frontier receives one native direct neighbor query: Both direction,
all 27 reviewed kinds and IncludeCandidate. This is one round at depth at most 2
from the initial root; Both direction does not select level `both`. Native IDs
deduplicate added/previously-selected records while retaining every initial and
frontier inclusion reason, exact support, source/load custody and scoped omissions.
The unchanged initial skeleton is retained alongside the expansion artifact
`wow-context/primary-frontier-expansion/1`. Aggregate origins use
`wow-context/project-map/primary-frontier-origins/1`; they do not replace the
original one-hop map.

The existing cumulative ledger includes the original L1, whole batch scans,
retained records/support and final output. Mandatory overflow refuses; native
query limits/truncation remain visible. The selected one-round bound ends
expansion even when nothing is added; it does not establish NoNewEvidence,
absence or completeness. No recursive round or inferred edge is emitted.
The one-round expansion keeps its existing selection. The separate
[selected frontier build/pack/render family](CONTEXT_BUILD.md#selected-frontier-pack)
consumes the native step with its continuing Budget. Its bounded source-absent
service/CLI path is verified in #122. Explicit additional batches use the
separate progressive profile below. The focused source-absent inspect lifecycle
passes; this does not imply full E3-B/Tier A/X
acceptance. See the
[native expansion contract](../../crates/wow-context/e3/NATIVE_PRIMARY_EXPANSION.md).

## Selected native progressive expansion

With `--level l1`, the default `wow-context/primary-l1/1` selection and an exact
graph-entity root, select
`--expansion-profile-id wow-context/primary-native-progressive/1` and repeat
`--frontier-round '<token>[,<token>...]'` once per explicit batch. Each token is
`graph-entity@<base64url-id>` using the existing exact root encoding. Supply
1..4 ordered batches, each with 1..32 distinct exact retained endpoints. Batch
order is semantic; the CLI does not choose or discover roots.

The first batch selects from the initial native detail map; subsequent batches
select endpoints already retained by earlier rounds. The initial root and every
previously queried expansion root are excluded. Empty/excess batches, duplicate
roots within or across batches, nonmembers, mixed `--frontier`/`--frontier-round`
selections and incompatible profiles/levels refuse. L0/Both/Project and selected
state-detail/Reference-facts L1 are outside this expansion profile.

Every round queries original native direct neighborhoods under the same held
project/graph/Reference binding. Original skeletons, immutable round history,
native receipts, inclusion reasons, evidence and scoped omissions survive.
No inferred transitive edge or automatic frontier selection is added.

One actual private Budget continues through every round and final encoding;
per-query limits still apply. Mandatory overflow or cancellation suppresses the
payload, with checked resource closure. The final explicit batch or four-round
profile bound ends the request. An empty added set does not infer NoNewEvidence,
absence or completeness; Partial/Truncated and absence_authoritative=false remain.
There is no background or retained cross-request continuation.

The [selected progressive pack/render family](CONTEXT_BUILD.md#selected-progressive-pack)
consumes this genuine history and continuing ledger. See
[Native Primary progressive expansion](../../crates/wow-context/e3/NATIVE_PRIMARY_PROGRESSIVE.md).
Existing one-round/default/state/Reference profile IDs are retained;
broader facets and runtime qualification remain open.

## Selected native UsesApi evidence

An ordinary published project with retained configuration
`native_api_uses=reference_v1` supplies native API source evidence to the existing
L1 route after genuine reopen. This is a Project input selection, not a new CLI
flag or context profile. Select an exact graph entity whose admitted native
neighborhood contains the relevant `UsesApi` relation.

Source-absent reopen reruns the retained native Library producers and compares
the accepted API/call producer partitions and final graph membership. The host's
configured Reference profile, generation and view digest must match that held
requalified native Reference exactly. A mismatch refuses before context
projection; optional Reference guards do not replace this mandatory fence.

Service admits original Project and API support through one bounded evidence
catalog after aggregate input preflight. Different contexts or conflicting
records under the same IDs refuse. L1 preserves the actual accepted `UsesApi`
edge and its original caller, call-site and API source evidence, including exact
source handles, digests and spans. `UsesApi` remains distinct from `Calls`.

The same evidence reaches structural [build/pack/JSON/Markdown](CONTEXT_BUILD.md)
through the native owner path. With default L1, Reference record/type enrichment
and Reference source excerpts remain `NotEvaluated`; existing captured native
signature observations retain their original availability. Static accepted
relations and source observations do not prove runtime execution, API legality,
loaded state, protected/Secret behavior, combat safety or authoritative absence.
Existing context profile IDs and request/schema recipes are retained;
this route does not claim full E3 or Tier A/X acceptance.

## Host input and service result

Inspect reuses the strict `wow-service/context-map-native-host/1` transport
[document and limits](CONTEXT_MAP.md), including explicit store registration /
host path, ProjectId, complete ProfileIdentity and exact ReferenceView. The
host path is transport-only and is absent from semantic requests/results.
The Serialize-only request uses schema `wow-service/context-inspect-request/1`,
publication, explicit root, level, platform="omitted", applicable selected profile
IDs and optional guards. ContextInspectRequest::primary keeps the default /1 choice.
Absent profiles/guards are omitted. There is no arbitrary-profile JSON authority.
Selected expansion additionally binds its exact profile and normalized frontier
in the request/identity. Progressive selection binds its named profile and
ordered batches to the same acquired universe and native round history.
Unselected expansion fields are omitted, preserving old inspect transport bytes
and the original map. Handler metadata remains part of the retained L1.

The result schema is `wow-service/context-inspect-result/1`, operation
`context_inspect`. Its payload retains resolved_request_id, publication-set /
store-generation IDs, exact Current observation/digest where applicable,
configured registration and native namespace ID where available, and
`skeletons` with schema `wow-context/primary-skeleton-inspection/1`.
Default nested schemas remain `wow-context/l0-skeleton/primary/1` and
`wow-context/l1-skeleton/primary/1`. Selected state L0 uses
`wow-context/l0-skeleton/primary-state-root/1`; Both retains the existing combined
schema and old L1. Selected state-detail L1 uses its named nested schema; the
outer inspect/result and combined skeleton schemas remain unchanged. Selected
Reference-facts L1 uses its own named nested schema. Each layer
carries its descriptor, original typed records,
exact origin-map support/custody and explicit omissions. Root declarations are
plural, retaining repeated supporting TOC rows, account/character and ambiguity.

The intent request ID binds symbolic request bytes. The resolved request ID
additionally binds exact publication, store generation, universe, root and all
selected host limits. Both reuses one origin acquisition/map and one aggregate
skeleton budget; it is not a second semantic service call. Unselected Primary
callable recipes remain NotSelected; captured native observations retain their
original availability/uncertainty. No bodies, body shape, inferred roles,
source excerpts or complete-signature/runtime claim is added.

The service finalizes only after checked read/store closure. A close failure
suppresses the drafted payload. Failures retain failure_code and resource_closure
as not_acquired/closed/failed/not_evaluated by the actual native route. Returned
Partial/Truncated skeletons keep absence_authoritative=false; selected member or
per-query edge counts are not total project/corpus truth.

## Bounds and output

Host input is at most 16 MiB. Combined encoded native inputs fit selected
max_input_bytes, at most 64 MiB. Retained metadata and the complete final service
envelope fit selected max_metadata_bytes, at most 16 MiB. Positive finite
max_context_work is cumulative; max_direct_edges (1..=100000) and native
neighbor/explanation/evidence limits retain independent validation. Mandatory
custody/selected metadata overflow refuses rather than dropping a level/member
to fit. Encoded/work reservations are not OS peak-memory or token measurements.

The CLI writes exact finalized canonical service JSON plus one LF. Partial and
Truncated exit 2; semantic refusal exits 3; internal/canonicalization/checked-close
or output failure exits 4; argv/host-input failure exits 64; cancellation exits
130. Finalized failure/cancellation envelopes pass through unchanged, with no
semantic retry after output begins.

This route documents selected functionality, not full E3 or Tier A/X acceptance.
Broader scopes/facets, control/effect enrichment, automatic expansion and retained
cross-request continuation, broader Reference/type enrichment, excerpts/privacy/tokens
and combined context remain open. [Build/pack/render](CONTEXT_BUILD.md) consumes the native
step, not a decoded inspect envelope. Existing map commands and root/profile
selections remain in use.

## Selected native platform inspect

The platform route calls `execute_context_platform` once and consumes the native
map into inspect without a second Skeleton read:

```text
wow context inspect --platform native --native-host-config <host.json>
  --platform-request-input <request.json> [--output envelope-json|text]
```

Use the [platform host/request fields](CONTEXT_MAP.md#selected-native-platform-map),
with `operation="inspect"` and `profile_id="wow-context/platform-native-one-hop/1"`.
The exact candidate/publication and native Skeleton binding are required. Primary
`--root`, `--level`, profile/guard/frontier selectors and mixed mode do not apply;
the request supplies the exact native root/query/limits. There is no native
platform build or artifact-output route.

The `wow-service/context-platform-result/1` envelope has operation
`context_platform_inspect` and retains the original inspect in `payload.artifact`,
including its original map/page/support. Text projects the same bounded public
typed summary and closure/status fields as map, without decoding canonical JSON.
Envelope output remains exact service bytes plus LF; Partial/Truncated exit 2,
semantic refusal 3, internal/close/output failure 4, cancellation 130 and transport
errors 64. Checked closure failure suppresses payload; output failure never retries.
Source/page omissions, configured Reference association and neighbor closure keep
[their owner scope](../../crates/wow-context/e3/NATIVE_PLATFORM_MAP.md).
Selected lifecycle and CLI verification evidence is retained in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122);
broader acceptance remains open.

## Selected combined native inspect

The selected combined transport is:

```text
wow context inspect --platform combined --native-host-config <host.json>
  --combined-request-input <inspect-request.json> [--output envelope-json]
```

Reuse the exact combined host and `wow-service/context-combined-request/1`
request family with `operation="inspect"`,
`profile_id="wow-context/combined-native-one-hop/1"` and required
`inspection`, for example `{"level":"both"}`; level is exactly `l0`, `l1` or
`both`. Optional `inspection.l0_profile_id="wow-context/primary-state-root-l0/1"`
selects only L0/GraphEntity through the original native StateRootV1 owner and
navigation map on the continuing Budget. It preserves original root/path/TOC
custody and emits no L1. Native state_path refuses with checked closure; other
explicit L0 profile/level/root combinations refuse before acquisition. Explicit
L0/L1 recipe fields cannot coexist. The optional `inspection.l1_profile_id` accepts only
`wow-context/primary-state-detail-l1/1` or
`wow-context/primary-reference-facts-l1/1`, each paired with `level="l1"` and an
exact GraphEntity root. StateDetail also supports `level="both"` for a genuine
state_root: original StateRoot L0 precedes StateDetail L1 on the same detail map
and continuing Budget. State_path Both refuses after native kind validation and
checked closure; ReferenceFacts Both refuses before acquisition. Omission of all recipe/facet fields
preserves default recipe request/artifact bytes. Null, non-string, the default ID,
unknown IDs and incompatible level/root or mixed operation fields refuse before
owner acquisition. The platform
subrequest remains `operation="map"`. Generations/publications, Primary root,
platform candidate and native binding are explicit in the request. Primary argv
selectors, standalone platform input, mixed modes, text/artifact output and
combined build options are outside this grammar; `--level` is not a combined argv
option.

One consuming service operation retains the selected original Primary skeleton
and borrowed original platform inspect under the actual continuing construction
Budget, without a second platform read. The held native API supplies original
`reference_origins` to the existing L1 Reference engine; origin identity and
support stay with their owners. Checked reverse closure and
final remaining allowance govern payload exposure. Envelope JSON is exact service
bytes plus one LF; Partial/Truncated exit 2 and output failure never retries.
The service result uses the same combined /1 schema, operation
`context_combined_inspect` and original `payload.combined_inspect`.
Unselected map requests/artifacts remain exact. See the
[combined owner contract](../../crates/wow-context/e3/NATIVE_COMBINED_MAP.md#selected-combined-inspection)
for budget scope and conservative omissions. Selected verification is tracked
under #122; full acceptance remains open. Combined build/pack is described in its
[build contract](CONTEXT_BUILD.md#selected-combined-native-build). Combined links
and full acceptance remain open.

Optional `inspection.platform_root_facet="callable_metadata"` requests the exact
platform query root's original native callable metadata with the ordinary default
Primary recipe. It cannot accompany explicit L0/L1 recipe fields. Null/unknown
values refuse before acquisition; acquired wrong-kind or missing native custody
refuses with checked closure. Original declaration/signature states and support
remain qualified to that root; unselected signatures remain `NotSelected`.
Omission preserves existing bytes. The same explicit JSON-file transport and
envelope output apply; the CLI does not resolve or reconstruct the facet.
