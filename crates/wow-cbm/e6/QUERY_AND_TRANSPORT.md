# E6-A query and transport contract

**Status:** normative.

## Transport port

E6-A receives an already-acquired `ExternalCandidateTransportPort` whose implementation/session/authorization are owned by E6-B or a host adapter.

Allow-listed operations:

```text
provider_status
provider_capabilities
provider_generation
candidate_query
candidate_continue
candidate_explain
```

The port has no generic `call_tool`, raw MCP JSON, arbitrary method, process-start, credential, provider configuration, index/import/delete, database, filesystem, or network endpoint API.

## Query grammar

`ExternalCandidateQuery` is a closed tagged schema with operations such as:

```text
SemanticEntityCandidates
GenericTraceCandidates
RelatedSymbolCandidates
SourceLocationCandidates
```

Allowed fields are bounded typed seeds/terms/entity-kind hints/provider-side scopes/requested result fields and budgets. No natural-language text is interpreted as framework instruction or authority. Opaque search text, when a provider requires it, is untrusted provider query data under an exact field/size profile.

## Forbidden query fields

```text
raw SQL or FTS MATCH
regex program
shell/JavaScript/Lua/Wasm/native code
callback/plugin/expression
model prompt or tool policy
arbitrary MCP method/tool name
filesystem path to open
provider database operation
credential/token/private endpoint
```

## Execution

```text
validate descriptor/capability/state/query
-> normalize canonical request
-> invoke exactly one allow-listed transport operation
-> enforce request/response/time/item/depth/memory limits
-> validate response schema and state binding
-> normalize candidates/loss/coverage
-> close/cancel synchronously
```

E6-A does not retry through another provider or query lane. Idempotency/durable response-loss orchestration belongs to E6-B; the pure bridge returns typed transport/uncertain state.

## Pagination

Provider cursors are opaque untrusted transport data. Public continuation stores only a protected/digested reference under the transport owner policy. It binds exact provider/state/query/profile/cumulative budgets and cannot be substituted across sessions or providers.

## Cancellation

Cancellation is checked before/during request validation, dispatch, response parsing, item normalization, serialization, and continuation. Late responses after cancellation are rejected/discarded through the owner transport contract. No background task continues after return.

## Errors

Provider errors, unsupported operations, partial pages, malformed fields, rate limits, timeouts, cancellation, and state mismatch remain explicit. Empty response is not automatically success or authoritative zero.

## Transport nonclaims

A successful call proves only that the exact transport returned a response accepted under the schema. It does not prove provider corpus completeness, source truth, stable generation, candidate correctness, or local owner mapping.