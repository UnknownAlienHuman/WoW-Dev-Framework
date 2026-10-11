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
  [--l0-profile-id wow-context/primary-l0/1]
  [--l1-profile-id wow-context/primary-l1/1]
  [--platform omitted] [--output envelope-json]
  [--expect-current-record <id>] [--expect-current-digest <digest>]
  [--expect-profile <id> --expect-reference-generation <id> --expect-reference-view <id>]
```

Roots encode one exact UTF-8 ID with canonical unpadded base64url. The CLI does
not infer kinds/existence or choose a root. Both `--root` and `--level` are
required; there is no omitted-root shortcut for inspect.

| Level | Accepted acquired root | Fixed service profile(s) |
|---|---|---|
| `l0` | Matching project ID, or exact source_file/source_package graph container | `wow-context/primary-l0/1` |
| `l1` | Exact native graph entity | `wow-context/primary-l1/1` |
| `both` | One explicit source_file/source_package graph container | Both profiles, with distinct artifact IDs |

Profile flags are optional assertions of these exact IDs. An L1 profile flag
conflicts with level l0, and an L0 flag conflicts with level l1. Unknown/duplicate
options and other profiles/platform/output modes refuse; map-target/profile
flags are not inspect options. Service validates actual kind/existence and
project identity after acquisition. A method/function graph root can select L1
but cannot silently become an L0 container for Both.

Current guards require publication current. Reference guards require all three
options together. Service pins Current once or reads the exact store generation;
the CLI performs no selector resolution, search, alias lookup or retry.

## Host input and service result

Inspect reuses the strict `wow-service/context-map-native-host/1` transport
[document and limits](CONTEXT_MAP.md), including explicit store registration /
host path, ProjectId, complete ProfileIdentity and exact ReferenceView. The
host path is transport-only and is absent from semantic requests/results.
`ContextInspectRequest::primary` constructs the Serialize-only request with
schema `wow-service/context-inspect-request/1`, publication, explicit root,
level, platform="omitted", applicable fixed profile IDs and optional guards.
Absent profiles/guards are omitted. There is no arbitrary-profile JSON authority.

The result schema is `wow-service/context-inspect-result/1`, operation
`context_inspect`. Its payload retains resolved_request_id, publication-set /
store-generation IDs, exact Current observation/digest where applicable,
configured registration and native namespace ID where available, and
`skeletons` with schema `wow-context/primary-skeleton-inspection/1`.
L0/L1 nested schemas are `wow-context/l0-skeleton/primary/1` and
`wow-context/l1-skeleton/primary/1`; each carries its static descriptor, exact
origin-map custody, selected native metadata and explicit omissions.

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
Broader scopes/facets, control/effect enrichment, expansion, semantic packs,
rendering, excerpts/privacy/tokens/continuation and platform/combined context
remain open. Existing map commands/profile bytes are unchanged by this route.
