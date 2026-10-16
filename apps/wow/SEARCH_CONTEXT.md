# Exact selected native Context inspection and Build

```text
wow search context --context-json <JSON> --context-host <ABSOLUTE-FILE|->
  --search-store <EXISTING-SEARCH-STORE> --operation-id <ID>
  --allow-local-metadata-only <Store/output limits>
```

One strict semantic input references an already completed explicit selection:

```json
{
  "schema": "wow-service/search-context-input/1",
  "selection_operation_id": "original-selection",
  "selection_request_digest": "sha256:...",
  "selection_result_id": "wow-service/search-durable-result/1:...",
  "receipt_id": "search-selection-receipt:...",
  "project_store_id": "explicit-native-registration",
  "level": "l1"
}
```

`level` is `l0`, `l1` or `both`. Store/output limits are the existing Search
options. Unknown fields and root, generation, Current, query, profile, Search
bounds or lease overrides fail. Selection origin and exact candidate/root come
from the immutable receipt; rank, name and sole-candidate inference are forbidden.

The host file/stdin uses the existing
[`wow-service/context-map-native-host/1`](CONTEXT_INSPECT.md) input, bounded by
`CONTEXT_MAP_HOST_INPUT_MAX_BYTES`. It must contain a genuine ReferenceView and
profile supplied through their existing owner route, never reconstructed from
Search output. The CLI admits the bounded inert host and invokes exactly one
`SearchOperationService::context_for_selection` operation. It opens no owner or
database and implements no Context semantics.

Service checks the completed selection/result/receipt and revalidates the exact
original explanation through the shared selection materializer. For a Primary
GraphEntity it verifies original node/universe/generation, selects the original
exact retained StoreGenerationId, and invokes the existing E3
`ContextInspectRequest::primary` / `execute_context_inspect` lifecycle once.
Actual native Project/Graph/Reference admission proves membership and compatibility.
The unchanged native result is returned as `payload.context`; original Search
explanation and selection receipt remain outer provenance, not Context truth.

E3 may include its transactional `current_at_acquisition` observation even for an
Exact read. It remains native observational data, not a selector: advancing Current
does not retarget the selected generation or change its semantic artifacts. Fresh
operations preserve E3's current observation; historical replay preserves the
original recorded response.

Ordinary Entity inspection retains E3's original Reference generation/profile
caller-association ceiling; it is not a new Reference publication attestation.
Federation additionally checks its originally captured exact ReferenceView digest.
Default structural inspection supports L0/L1/Both. The explicit Build operation
below uses the same exact selection and native admission. Excerpts, continuation
and broader Context profiles remain separate operations.

## Selected semantic pack and rendering

```text
wow search context build --build-json <JSON> --context-host <ABSOLUTE-FILE|->
  --search-store <EXISTING-SEARCH-STORE> --operation-id <ID>
  --allow-local-metadata-only <Store/output limits>
```

The strict Build input has all the selection/registration fields shown above,
`schema="wow-service/search-context-build-input/1"`, explicit `level` and required
`render`: `none`, `json`, `markdown` or `both`. Omitted, null, unknown and duplicate
fields refuse. Inspection and Build inputs/options cannot be mixed. Root,
generation, Current, profile, frontier, source and continuation overrides refuse.

The CLI invokes one `SearchOperationService::build_context_for_selection`
operation. The shared selection plan invokes the existing
`ContextBuildRequest::primary` / `execute_context_build` with the original exact
generation and GraphEntity, default structural pack and selected native renderers.
It supplies no source-excerpt capability. The unchanged native result remains
`payload.context`; `context_admission="exact_native_build"` identifies this route.
The outer durable operation is `search_context_build`, with a separate request
identity binding explicit render selection. Existing inspection request/result
identities and bytes remain unchanged.

The existing [`wow-context/primary-structural-pack/1`](CONTEXT_BUILD.md) builds
L0/L1/Both with native level/root admission and the continuing construction
Budget. `none` returns a validated semantic pack and no rendering. `json` and
`markdown` select the existing `wow-context/primary-pack-json/1` and
`wow-context/primary-pack-markdown/1`; `both` uses JSON then Markdown. Renderer
selection preserves semantic pack bytes/identity. Native rendered content,
artifact IDs, traces, omissions and budgets remain unchanged. A requested
renderer failure suppresses delivery; there is no implicit pack-only fallback.

Build preserves the inspection route's Reference association ceiling, explicit
federation View guard, metadata-only privacy, original generation and native
Current observation. Replaying a completed Build returns its recorded bytes
before acquiring native owners. Changing render under the same operation ID
refuses.

## Explicit progressive Build and continuation

Build may include one non-null `progressive` object:

```json
{
  "context_profile_id": "wow-context/primary-progressive-pack/1",
  "expansion_profile_id": "wow-context/primary-native-progressive/1",
  "continuation_retention_profile_id": "wow-service/context-continuation-retention/1",
  "frontier_rounds": [["graph-node:sha256:..."], ["graph-node:sha256:..."]]
}
```

This explicit selection requires L1, two to four caller-ordered batches of one
to 32 exact GraphEntity IDs and the existing continuation-capable native host
`wow-service/context-map-native-host/2`. The input remains bounded to 8192 bytes.
E3 validates distinct roots, original retained membership and every later batch;
Search supplies no inferred frontier. The existing progressive pack/render family
and retention profile execute the first batch and return the native continuation
ID under `payload.context.payload.continuation_id`. Omission of `progressive`
preserves default Build request identity and native bytes. Explicit null, unknown
profiles, invalid levels/batches or missing native continuation configuration
refuse.

```text
wow search context continue --continue-json <JSON>
  --context-host <ABSOLUTE-FILE|-> --search-store <EXISTING-SEARCH-STORE>
  --operation-id <ID> --allow-local-metadata-only <Store/output limits>
```

The strict `wow-service/search-context-continue-input/1` document requires
`prior_context_operation_id`, `prior_context_request_digest`,
`prior_context_result_id` and `continuation_id`. It references the exact completed
outer progressive Build or Continue response whose native payload issued that
token. It accepts no root, selector, frontier, render, profile or native budget
override. Host semantic binding must equal that prior response. The original
selection receipt and Search explanation are carried unchanged through each
outer response; they never become native Context authority.

One `SearchOperationService::continue_context_for_selection` call invokes
`ContextContinueRequest` / `execute_context_continue` with the exact token and
guards for its original native request and latest native pack. E3 verifies the
completed issuer, actual owner binding and retained original generation, replays
the issued prefix with its cumulative private Budget, and atomically records the
next result/receipt before releasing a terminal generation hold. Native output
remains unchanged; the outer operation is `search_context_continue` with admission
`exact_native_continuation`. The next token is absent after the final batch.

Exact outer replay returns recorded bytes before native acquisition. Once native
issuance or continuation has dispatched, uncertain native closure or failure to
finalize the outer response records `OutcomeUnknown`, never retryable no-effect.
Blind repetition is blocked; the exact native journal remains the reconciliation
owner. Pre-dispatch guards preserve their typed failure. Once issuance/continuation
is dispatched, a typed native failure is also fenced as `OutcomeUnknown`: checked
closure alone does not certify that its durable journal was unchanged. Continuation
does not refresh Current, reset budgets, deserialize a native expansion step,
read source or authorize edits. Broader owner mappings and acceptance remain open.

Reference document keys, Source fact IDs and unqualified platform roots are not
converted into GraphEntities. Unsupported mappings return `not_evaluated` with
no native Context success or empty fake artifact. Unknown schema/universe values
remain strict identity/schema refusals.

Native closure and failure take precedence over output limits. No public Context
success precedes checked native closure; uncertain cleanup records OutcomeUnknown
and forbids blind retry. Original partial/truncated/conflict and cancellation
states remain conservative. A smaller delivery cap cannot conceal a close failure.

The durable request binds host registration, project, actual ReferenceView,
profile, schema and budgets. Host filesystem/continuation routing locations are
excluded from semantic identity. Completed replay reads the recorded response
before native acquisition, including after Current advances or source resources
become unavailable; it does not claim present liveness. A new operation still
requires the original exact native owners. Missing SearchStore is never created.

`envelope-json` is emitted once with the standard final LF. Original partial or
truncated Context is exit2; typed refusals are exit3/4, invalid transport/schema
exit64, cancellation130. This functional route does not accept full E4, external
current-source parity, a WoW runtime or launch qualification.
