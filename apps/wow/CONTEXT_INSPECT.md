# `wow context inspect`

The selected Primary-only route calls
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
and an L0 flag conflicts with level l1. Unknown/duplicate options and other
profiles/platform/output modes refuse; map-target/profile flags are not inspect
options. Service checks actual root kind/existence, original custody and project
identity after acquisition. A method/function cannot silently become an L0
container. Old default /1 requests and artifacts retain their behavior/bytes.

Current guards require publication current. Reference guards require all three
options together. Service pins Current once or reads the exact store generation;
the CLI performs no selector resolution, search, alias lookup or retry.

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
after construction. Default inspect bytes remain unchanged. Inspect behavioral
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
profiles retain their existing bytes and omissions. The same selection has a
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
Inspect retains its original result bytes. The separate
[selected frontier build/pack/render family](CONTEXT_BUILD.md#selected-frontier-pack)
consumes the native step with its continuing Budget. Its bounded source-absent
service/CLI path is verified in #122. Progressive continuation remains open. The focused
source-absent inspect lifecycle passes; this does not imply full E3-B/Tier A/X
acceptance. See the
[native expansion contract](../../crates/wow-context/e3/NATIVE_PRIMARY_EXPANSION.md).

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
Existing context profiles, request/schema recipes and unselected bytes remain
unchanged; this route does not claim full E3 or Tier A/X acceptance.

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
in the request/identity. Unselected expansion fields are omitted, preserving old
inspect transport bytes and original L1/map bytes.

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
Broader scopes/facets, control/effect enrichment, progressive expansion /
continuation, broader Reference/type enrichment, excerpts/privacy/tokens and platform/combined
context remain open. [Build/pack/render](CONTEXT_BUILD.md) consumes the native
step, not a decoded inspect envelope. Existing map commands/profile and unselected
inspect bytes remain unchanged.
