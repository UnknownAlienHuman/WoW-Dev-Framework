# `wow-service` E6-B external-candidate orchestration, mapping, and context handoff

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `wow-service/e6-b/external-candidate-orchestration-mapping-context`

## Mission

Provide one durable transport-independent boundary around the optional E6-A `wow-cbm` lane without upgrading external Candidate evidence or making exact local workflows depend on an external provider.

```text
exact provider configuration/profile selector
+ credential-use authorization reference
+ provider descriptor and external-state policy
+ ExternalCandidateQuery
+ OperationId + CanonicalRequestDigest
-> resolve exact configuration once
-> authorize credential/session use without exposing credentials
-> acquire one narrow provider session/transport
-> invoke E6-A descriptor/state/query validation and candidate normalization
-> publish immutable query/result/artifact records
-> optionally map UnverifiedProviderLocator through exact project/reference owners
-> optionally record one explicit candidate selection
-> optionally build exact-root context through existing context owners
-> retain/audit/reconcile effects
-> close resources in reverse order
-> emit one conservative canonical service envelope
```

## Public operations

```text
external_candidate_status
external_candidate_provider_validate
external_candidate_query_submit
external_candidate_query_get
external_candidate_query_list
external_candidate_query_cancel
external_candidate_query_continue
external_candidate_operation_reconcile
external_candidate_result_validate
external_candidate_result_explain
external_candidate_result_compare
external_candidate_artifact_build
external_candidate_artifact_get
external_candidate_mapping_resolve
external_candidate_mapping_get
external_candidate_selection_record
external_candidate_selection_get
external_candidate_context_build
external_candidate_context_continue
external_candidate_cache_validate
```

## Active dependency slice

```text
wow-core
wow-store
wow-project
wow-reference
wow-graph
wow-context
wow-cbm
```

`wow-service` coordinates exact owner ports only. It does not parse provider results, map paths itself, reproduce project/reference/context logic, or access raw storage.

## Authority separation

```text
external result              = semantic_candidate + Candidate
provider locator             = UnverifiedProviderLocator
owner mapping                = exact locator-to-owner-record identity only
selection receipt            = explicit caller choice, not proof
context artifact             = exact owner-root context
provider metadata sidecar    = external Candidate evidence only
```

A mapped locator does not verify a provider summary, relation, trace, ranking, replacement, impact, or runtime claim. A selected candidate does not become correct because it was selected. Provider text never enters a `ContextSemanticPack` as an exact framework fact.

## Provider/session boundary

Configuration, credential authorization, session acquisition, and transport construction are owned by narrow host/provider ports. Requests carry stable nonsecret references, never tokens, keys, cookies, passwords, private endpoints, shell commands, or provider database paths.

E6-B does not install, update, start, configure, index, import, delete, or mutate an external provider. A session exposes only the reviewed E6-A `ExternalCandidateTransportPort` allowlist.

## Durable query/result lifecycle

Every query operation is registered before provider dispatch. Immutable records distinguish:

```text
Registered
Authorized
SessionAcquired
Dispatched
ResultRecorded
Validated
Partial
Truncated
OutcomeUnknown
Cancelled
Failed
```

Provider response delivery, result catalog publication, mapping, selection, and context handoff are separate effects. Response loss is reconciled by exact operation/request/provider-session identity. Blind repeat is forbidden.

## Exact mapping

Mapping requires one exact retained project or reference generation and one exact `UnverifiedProviderLocator`. Owners return only:

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

Service never resolves by same name, nearest path, top rank, newest/current generation, repository popularity, or provider confidence. `NoMappingWithOwnerAuthority` is allowed only when the owner proves complete relevant coverage.

## Explicit selection

Selection requires an exact result, candidate, mapping, intended use, consumer, and authorization/audit profile. Service records the supplied decision; it never chooses top-1, sole, highest-score, first, newest, same-name, or most frequently returned candidates.

## Context handoff

Context build requires `ExactMapped` plus an explicit valid selection receipt. Service reacquires exact retained project/reference/graph views, constructs the normal E3 context universe, and invokes one `wow-context` operation with the exact mapped root. External candidate metadata remains a separate sidecar in the service envelope.

## Optional degradation

Provider configuration/session/query failure disables only the external lane. Exact local reference, project, graph, search, context, diagnostics, and rules remain independently usable. E6-B performs no hidden fallback to another provider, stale cache, model, web search, or local search.

## Deferred

- LSP/MCP/HTTP/daemon transports;
- public provider tool discovery;
- provider install/index/database lifecycle;
- model invocation or generic agent tools;
- automatic candidate selection or source edits;
- external evidence promotion into core recognizers;
- public release/update distribution;
- CI.

## Completion gate

E6-B implementation is complete only when exact provider configuration/session authorization, durable query/result/artifact catalogs, response-loss reconciliation, owner mapping, explicit selection, exact-root context handoff, optional degradation, retention/audit/closure, privacy/license/security, canonical envelopes, and CLI mappings pass all frozen fixtures and measured limits.