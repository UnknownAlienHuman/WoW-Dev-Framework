# Native primary context map

`wow context map` calls one service operation over an explicitly registered
retained primary project. It emits exact canonical service JSON followed by one
LF. Supported profiles are `wow-context/primary-direct-navigation/1` and
`wow-context/primary-project-navigation/1`;
the [owner contract](../../crates/wow-context/e3/NATIVE_PRIMARY_MAP.md) defines its
projections and remaining E3-B scope. Issue #122 remains the acceptance tracker.

```text
wow context map --native-host-config <host.json>
  --project-store <registration-key>
  --publication current|store-generation:<exact-native-generation-id>
  --map-target primary
  --map-profile-id wow-context/primary-direct-navigation/1
  --root graph-entity@<unpadded-base64url-of-exact-node-id>
  [--platform omitted] [--output envelope-json]
  [--expect-current-record <id>] [--expect-current-digest <sha256:digest>]
  [--expect-profile <id> --expect-reference-generation <id> --expect-reference-view <sha256:digest>]
```

For project-container navigation, select
`--map-profile-id wow-context/primary-project-navigation/1` and use
`--root project@<unpadded-base64url-of-exact-ProjectId>`, or omit `--root`.
Service checks the explicit ID against the acquired project, or resolves the
omitted root to that acquired ProjectId. The CLI does not infer a root.
Graph-entity roots are rejected for this profile.

Root transport is UTF-8, unpadded canonical base64url with zero unused bits.
Decoding performs no search or identity resolution. Direct navigation still
requires an explicit exact graph-entity root; a matching project root or omitted
root returns ComponentUnavailable in that profile. The original ProjectSourceOrigin
container is retained without inventing a project graph node or contains edge.

Current guards require `--publication current`. Reference guards form one complete
three-option group. Unknown/duplicate options, noncanonical roots, oversized input,
other targets/profiles/platforms and other output modes fail before invocation.
The application does not select Current or retry an operation.

The strict host JSON has schema `wow-service/context-map-native-host/1` and exactly
these fields:

| Field | Required value |
|---|---|
| `schema` | Exact schema above |
| `project_store_id` | Explicit opaque registration key, matching `--project-store` |
| `project_store_root` | Absolute host path to an existing native live-project store |
| `project_id` | Exact acquired ProjectConfiguration project ID |
| `reference_profile` | Actual complete ProfileIdentity |
| `reference` | Actual exact ReferenceView |
| `limits` | Native `neighbors`, `explain`, `evidence` limits, plus positive `max_direct_edges`, `max_input_bytes`, `max_metadata_bytes`, `max_context_work` |

The registration key permits ASCII letters/digits and `. _ - : @`, with at most
512 bytes. It is not fabricated native namespace authority: when a store has a
real ProjectStoreNamespace, its distinct ID appears as `native_project_store_id`.
Legacy primary stores retain their exact registration/project guard.

Host JSON is at most 16 MiB; combined encoded owner inputs are at most the selected
`max_input_bytes` (up to 64 MiB). Final envelope bytes fit selected
`max_metadata_bytes` (up to 16 MiB). Argv limits are 4096 arguments, 1 MiB total and
32768 units per value; decoded IDs are at most 512 bytes. Native graph limits keep
their own validation. `max_direct_edges` and neighbor limits apply per native
root query; work, retained views/support and final metadata must fit the complete
aggregate budget. Every retained source-file projection is mandatory for project
navigation: an oversized batch refuses rather than returning a root prefix.
The host document has no scripts, includes, interpolation,
source discovery or source execution. Host paths never enter semantic identities,
requests, results or errors.

The service resolves Current once, acquires the selected immutable pair, binds
Reference, and reuses the graph's checked read proof for native neighbors and
evidence explanations. It retains a private draft until checked native read and
store closure. Reconstruction refusal explicitly closes the acquired read.
Lower-level native acquisition failures do not expose their checked close result:
their failed envelope reports `resource_closure=not_evaluated`. Checked close
failure reports `failed` and suppresses the map. Entry refusal reports
`not_acquired`; fully checked unwind reports `closed`.

Project navigation uses map schema
`wow-context/project-map/primary-project-navigation/1` and nested
`navigation.schema=wow-context/project-navigation/1`. Its exact `source_file_roots`
and native `neighborhoods` retain immutable original file/node/support custody.
Each neighborhood keeps its query counts and truncation; summed per-query
omissions can count the same edge in multiple queries. Deduplicated node/edge
support does not turn that sum into a unique project-wide census.
Both profiles emit Partial or Truncated with `absence_authoritative=false`.

The intent `request_id` binds symbolic request bytes. The distinct
`resolved_request_id` binds that intent, exact publication/store generation,
Reference/profile/view, resolved root and all selected limits. An omitted project
root remains omitted in the request; its resolved identity names the acquired
ProjectId. The map identity also binds the exact retained root set and its
selected descriptor. `current_record_digest` is
SHA-256 of wow-core canonical JSON for the exact CurrentPublication observation;
it is distinct from the Current record-ID derivation material. The map is passed
through unchanged in `wow-service/context-map-result/1`.

Partial/Truncated maps exit 2; semantic/request refusal exits 3; internal,
canonicalization, checked-close or output failure exits 4; argument/host transport
failure exits 64; cancellation exits 130. Finalized failure/cancellation envelopes
remain exact service bytes. Successful output has no diagnostic text on stdout.

The [frozen direct-profile request/envelope/profile](../../crates/wow-service/e3/native-primary-map/README.md)
exercise actual native owners, rather than live source parity. Direct profile
bytes/shape and the outer request/result /1 schemas are preserved. Text/artifact
modes, other context commands, L0/L1, packs/rendering, excerpts/tokens/continuation,
platform/combined maps and full Tier A/X qualification remain open. Project
navigation covers the retained source-file projection, not complete-corpus
qualification or full E3-B acceptance.
