# Native platform owner universe, one-hop map and inspect

This selected owner contract uses one common family:
`wow-context/platform-native-one-hop/1`. Universe, map and inspect retain
distinct artifact roles/IDs under that family. It does not change the existing
[Primary map](NATIVE_PRIMARY_MAP.md) or [Primary inspection](NATIVE_PRIMARY_INSPECT.md)
contracts. Broader qualification remains in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).

## Genuine native input and custody

The input is an already admitted `SkeletonInputView` over the original native
platform source owner, accepted `GraphPartitionSnapshot` and exact request.
Existing composed candidate/Skeleton custody remains an upstream requirement;
context does not create, select or substitute that admission.
The existing Skeleton admission retains project/configuration/analyzer/source
and graph generations, source profile/package binding, source assertion scope,
registry/foundation and exact native-prefix partition custody. Context borrows
those owners/request; serialized bindings or pages cannot reconstruct authority.
No source acquisition, parsing, graph reconstruction or Primary-kind substitution
occurs in this route.

The owner API in [platform.rs](../src/platform.rs) is:

```text
bind_platform_context_universe(view, reference, reference_profile, limits, stop)
  -> PlatformContextUniverse
build_platform_context_map(universe, stop) -> PlatformContextMap
build_platform_context_inspect(map, stop) -> PlatformContextInspect
```

Binding requires the platform project kind, exact configured target profile/
Reference generation, source context and native graph snapshot, with compatible
caller-selected query/explanation limits. Map construction calls the held
`SkeletonInputView.read` once. Inspect consumes the map and its ledger without
another native read; original map ID/canonical bytes and native page are retained.

## Configured Reference association

The supplied ProfileIdentity is validated and must equal the platform Project's
configured selected profile; Reference generation must match that configuration. Bounded full
Reference validation binds its schema/self-digest, original partition coverage
and conflicts into the universe. This is explicit configured association to the
validated full view, not proof of an API node -> Reference record/source join.
Record/type enrichment, source type resolution and runtime interpretation remain
`NotEvaluated` in this selected route. A matching profile/generation or content digest cannot upgrade those
claims or establish current-client/API/runtime qualification.

## Original native page and honest omissions

Map/inspect retain the exact native one-hop neighborhood, original root and
resolved root support. Native neighbor records keep their original support IDs/
coverage; their evidence closure remains `NeighborhoodEvidenceClosure`
`NotEvaluated`. Root evidence does not establish neighbor evidence closure.

The original page preserves source admission/license, package/load coverage and
issue count, project coverage, deferred capabilities, inventory-span omissions,
query counts and truncation. Its state stays Partial or Truncated; the outer map
has `absence_authoritative=false`, and native `no_new_evidence` remains false.
Typed declarations/signatures/types, package/load-role projection, rejection-to-
root association, coverage Reference resolution, source bytes/excerpts and
axes/paths/continuation retain the native page `NotEvaluated` markers.
No transitive edge, inferred clean negative, runtime or complete-source claim is
added by map/inspect wrapping.

## One continuing budget

One private context Budget starts at universe binding and moves into map, then
inspect. Before the native read, reserve its exact full edge population and the
selected root explanation/evidence work ceilings with checked arithmetic; sparse
output cannot hide scan work. Bounded Reference validation work, retained binding/coverage, returned
native records/support, identities and canonical outputs debit that ledger.
Before the native read, its selected output ceiling must fit the actual remaining
metadata allowance. Native query/support/evidence limits remain independent.
Encoding and validation stay cancellable; overflow or owner refusal does not
become an empty successful artifact.

`remaining_metadata_bytes()` is the post-artifact delivery allowance, not a new
Budget or authority to reconstruct/reset one. Finite caller-selected limits do
not qualify corpus capacity or exact token/runtime measurements.

## Selected service and application

The service [platform driver](../../wow-service/src/context/platform.rs) admits
strict bounded host/request documents and calls `execute_context_platform` once.
The host schema is `wow-service/context-platform-native-host/1`:

| Field | Required original input |
|---|---|
| `schema` | Exact host schema above |
| `project_store_id` | Explicit opaque registration key, matching the request |
| `project_store_root` | Absolute path to the retained native store; transport-only |
| `selection` | Original structured `wow-service/platform-project-selection/1` document |
| `reference_profile` | Complete selected ProfileIdentity |
| `reference` | Exact full ReferenceView |
| `limits` | Native neighbors/explain/evidence limits and positive `max_direct_edges`, `max_input_bytes`, `max_metadata_bytes`, `max_context_work` |

`selection` has exactly `schema`, `owner_project_id`, `source_profile_id` and
`catalogs`. The catalogs carry `xml_schema_classification`,
`physical_function_declarations`, `physical_function_signatures` and
`composed_candidate`. Selection must match the retained catalog/configuration;
`composed_candidate=true` is mandatory. Classification/candidate choices are
booleans; declaration choices are `absent`, `declared_syntax_v1` or
`declared_syntax_and_body_shape_v1`; signatures are `absent` or `indexed_types_v1`.
This object is intent and cannot reconstruct native source/candidate authority.

The request schema is `wow-service/context-platform-request/1`:

| Field | Required exact input |
|---|---|
| `schema` | Exact request schema above |
| `operation` | `map` or `inspect`, matching the CLI command |
| `project_store_id` | Host registration key |
| `store_generation_id` | Exact retained native StoreGenerationId |
| `publication_set_id` | Exact acquired publication |
| `candidate_id`, `candidate_digest` | Exact held composed candidate identity |
| `profile_id` | `wow-context/platform-native-one-hop/1` |
| `input` | Original OpenSkeletonInputRequest: `binding`, `neighborhood`, `limits` |

Unknown fields or null required fields refuse at strict transport admission.
The service opens that exact generation in the selected namespace and requires
publication/candidate equality plus equality with a binding derived from the
held native source and accepted final graph. Host limits narrow every selected
native query/support/evidence limit. The existing read, full Reference validation,
map/inspect ledger and final delivery allowance remain cumulative. The final
envelope and extra owned typed summary are reserved against the actual remainder;
no delivery stage resets the metadata cap.

The result schema is `wow-service/context-platform-result/1`; operations are
`context_platform_map` and `context_platform_inspect`. Payload retains the
original structured `selection`, exact publication/generation/candidate IDs,
configured registration and distinct native namespace ID, artifact byte length,
original `artifact` and bounded `summary`. The latter exposes original native
counts/omissions, `not_evaluated`, root conflict/unresolved-evidence/truncation
counts and `absence_authoritative=false`. Text consumes public typed result
accessors directly, without parsing serialized artifact/envelope bytes.

The acquired read and store close explicitly before delivery. A close failure
suppresses the draft; failures carry the actual `resource_closure` observation
(`not_acquired`, `closed`, `failed` or `not_evaluated`) without a payload.
Cancellation stays conservative through final closure and encoding.

Selected [map](../../../apps/wow/CONTEXT_MAP.md#selected-native-platform-map) and
[inspect](../../../apps/wow/CONTEXT_INSPECT.md#selected-native-platform-inspect)
use existing commands with explicit native host/request input and envelope-json
or typed text output. There is no native pack/render or mixed/combined route.
Original Primary contracts remain unchanged. Selected lifecycle and CLI
verification evidence is retained in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122);
full Primary/platform/combined and runtime acceptance is not asserted here.
