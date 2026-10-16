# Exact selected native Context inspection

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
Default structural inspection supports L0/L1/Both. Build/render, excerpts,
continuation and broader Context profiles remain separate operations.

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
