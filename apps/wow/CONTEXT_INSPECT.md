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
  [--l1-profile-id wow-context/primary-l1/1]
  [--expansion-profile-id wow-context/primary-native-frontier/1
   --frontier <exact-GraphEntity-token> ...]
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
| `l1` | Exact native graph entity | `wow-context/primary-l1/1` |
| `both` default | One explicit source_file/source_package graph container | Default L0 + old L1, distinct artifact IDs |
| `both` state | One exact original ordinary TOC state_root GraphEntity | Selected state L0 + old L1, distinct artifact IDs |

An omitted/default L0 flag preserves primary-l0/1. To select state-root L0, pass
`--l0-profile-id wow-context/primary-state-root-l0/1` with level l0 or both and
an explicit graph-entity root. The retained ordinary selected-TOC Main/Library
owner supplies the root, every exact supporting typed declaration and selected
literal state_path members; there is no name lookup or exhaustive-field claim.
Bounded execution evidence is recorded in #122.

L1 profile flags remain exact assertions. An L1 flag conflicts with level l0,
and an L0 flag conflicts with level l1. Unknown/duplicate options and other
profiles/platform/output modes refuse; map-target/profile flags are not inspect
options. Service checks actual root kind/existence, original custody and project
identity after acquisition. A method/function cannot silently become an L0
container. Old default /1 requests and artifacts retain their behavior/bytes.

Current guards require publication current. Reference guards require all three
options together. Service pins Current once or reads the exact store generation;
the CLI performs no selector resolution, search, alias lookup or retry.

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
This selection is inspect-only: expanded build/pack/continue remains a next
consumer in #122. The focused source-absent native lifecycle passes; this does
not imply full E3-B/Tier A/X acceptance. See the
[native expansion contract](../../crates/wow-context/e3/NATIVE_PRIMARY_EXPANSION.md).

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
schema and old L1. Each layer carries its descriptor, original typed records,
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
Broader scopes/facets, control/effect enrichment, wider expansion, expanded
build/pack/continuation, excerpts/privacy/tokens and platform/combined context
remain open. Existing [build/pack/render](CONTEXT_BUILD.md) selections remain
separate; the frontier result is not a pack input. Existing map commands/profile
bytes are unchanged by this route.
