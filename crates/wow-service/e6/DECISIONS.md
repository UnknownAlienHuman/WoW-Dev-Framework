# E6-B service decisions

**Status:** normative.

## S6B-001 — E6-B orchestrates E6-A; it does not reproduce it

Provider descriptor validation, external-state classification, query normalization, transport adaptation, Candidate normalization, result validation, explanation, comparison, and cache-entry validation remain owned by `wow-cbm`.

## S6B-002 — Provider access is configured and allow-listed

A provider adapter is selected by exact reviewed profile. No arbitrary tool name, endpoint, command, model, or MCP method enters a public request.

## S6B-003 — Credentials remain inside a narrow authorization/session boundary

Canonical requests and results contain nonsecret references and authorization receipts only. Raw tokens, cookies, keys, private endpoints, database paths, and vault material never cross the service API.

## S6B-004 — External state is never silently upgraded

`StableExternalGeneration`, `ObservedMutableGeneration`, and `OpaqueExternalState` remain distinct through service, persistence, continuation, mapping, and context operations.

## S6B-005 — Durable operation identity precedes effects

Every provider/store effect is registered under exact `OperationId + CanonicalRequestDigest` before dispatch.

## S6B-006 — Response loss does not prove no effect

Timeout, disconnect, cancellation, or serialization failure after dispatch produces `OutcomeUnknown` until exact provider/store reconciliation.

## S6B-007 — Candidate artifacts are immutable and authority-capped

Persistence, validation, mapping, selection, context use, cache, and repeated retrieval never raise E6-A Candidate authority.

## S6B-008 — Provider locators are unverified

Provider path/URI/repository/symbol/span/digest fields remain data until an exact owner port returns a mapping receipt.

## S6B-009 — Mapping belongs to exact project/reference owners

`wow-service` coordinates owner mapping but defines no path, symbol, digest, alias, fuzzy, or repository mapping algorithm.

## S6B-010 — Mapping is one exact owner universe at a time

A request declares project or reference owner target plus exact retained publication. Cross-owner results remain separate. The service never merges them because names match.

## S6B-011 — Exact mapping proves identity only

`ExactMapped` proves locator correspondence under one profile and generation. It does not prove the provider's prose, inferred relationship, confidence, recommendation, or absence claim.

## S6B-012 — No mapping is not automatically global absence

`NoMappingWithOwnerAuthority` is scoped to the exact locator, owner generation, mapping profile, and complete relevant owner coverage. Partial or unavailable owner state remains nonauthoritative.

## S6B-013 — Candidate selection is explicit

No automatic top-1, first, best, highest-score, sole-result, same-name, same-path, or provider-labelled exact selection exists.

## S6B-014 — Selection is not evidence

A selection receipt records an explicit choice and mapped root. It does not raise confidence, authorize tools/edits, or establish lineage/replacement/runtime/platform truth.

## S6B-015 — Search and external candidates remain separate

E4 search shards/ranking are not invoked or fused in E6-B. Any future fusion requires a new owner contract and cannot compare provider-local scores as authority.

## S6B-016 — Context receives exact mapped owner roots only

The existing context operation receives the mapped project/reference root and exact views. Provider snippets, summaries, ranks, scores, and inferred relations are excluded from `ContextSemanticPack` framework facts.

## S6B-017 — External evidence remains visible outside context truth

The outer service envelope can reference the external result, mapping, and selection receipts as Candidate evidence with explicit nonclaims.

## S6B-018 — Optional provider failure does not break exact local workflows

Provider/session/capability/query failure is explicit degradation. It does not disable project/reference/graph/search/context/diagnostic operations that do not require the provider.

## S6B-019 — No hidden fallback

E6-B does not switch provider, generation, state class, cache entry, query, mapping owner, or context root without a new exact request.

## S6B-020 — Continuation is state- and budget-bound

Continuation reuses the exact provider/session/external-state/query/profile/result chain and cumulative budgets; it never refreshes mutable state or resets limits.

## S6B-021 — Cache is validation only

`wow-service` coordinates exact cache reads through owner/store ports. A cache hit cannot raise authority, freshness, negative authority, or state reproducibility.

## S6B-022 — Resource acquisition and closure are deterministic

Resources open in frozen order and close in exact reverse order. Public success is impossible before mandatory retention and close receipts.

## S6B-023 — Applications remain transport-only

`apps/wow` parses explicit input, invokes exactly one service operation, and projects output/exit codes. It never opens providers, owner views, stores, source, or credential systems.

## S6B-024 — No provider management surface

E6-B has no provider install/start/stop/update/configure/index/import/delete/rebuild operation.

## S6B-025 — No runtime or platform truth

External candidate output and exact mapping do not prove WoW runtime behavior, API contract, Secret/taint/combat/protected safety, performance, migration correctness, or impact.

## S6B-026 — Audit records are immutable and privacy-scoped

Provider authorization, session acquisition, dispatch, response, reconciliation, mapping, selection, context handoff, retention, cache, and close events are auditable without leaking raw credentials or private provider/source data.

## S6B-027 — Physical provider/session identity is nonsemantic unless explicitly bound

Process IDs, sockets, local paths, connection handles, timing, and transport retries do not enter semantic Candidate identities. Exact adapter/session/external-state receipts do where required.

## S6B-028 — Empty result and no mapping remain distinct

Provider zero-result, candidate not selected, locator not mapped, context omitted, and owner-authoritative absence are separate records and cannot be collapsed.

## S6B-029 — No current/latest inside owner calls

A permitted symbolic project/reference selector is resolved once at the service boundary. Provider state is acquired/classified explicitly. All downstream calls use exact identities.

## S6B-030 — Missing reconciliation capability blocks retry

When a potentially effecting provider adapter cannot reconcile by exact operation/request identity, the operation remains `OutcomeUnknown`; the service does not guess or replay.
