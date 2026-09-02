# E6-B implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite and freeze closure

Before E6-B Rust source:

- implement/freeze E0 through E6-A prerequisites and fixtures;
- freeze `wow-cbm` provider descriptor/state/query/result/cache contracts;
- freeze provider authorization/session adapter profiles without credentials;
- freeze durable operation, response-loss, artifact catalog/store, retention, audit, project/reference mapping, selection, context, privacy/license, result, error, and canonicalization profiles;
- freeze synthetic stable/mutable/opaque providers and one configured real provider integration corpus;
- freeze request/result/receipt/CLI vectors and all SHA-256 manifests.

No fake provider, fake token, fake exact mapping, fake result, or fake passing benchmark.

## Phase 1 — E6-B request/result primitives

Implement closed tagged request/result/error/status types, exact selectors, profile validation, budget/cancellation, and canonical IDs.

Tests: `S6B-CONF-*`, request/schema/security subsets.

## Phase 2 — provider authorization/session boundary

Implement narrow configured ports and receipt validation:

```text
ProviderCredentialAuthorizationPort
ProviderSessionAcquirePort
ExternalCandidateProviderCatalogPort
```

No credential bytes in public types.

Tests: `S6B-PROV-*`, credential/replay/substitution/close cases.

## Phase 3 — durable operations and reconciliation

Implement operation registration, state transitions, dispatch receipts, provider reconciliation, local artifact idempotency, cancellation, and `OutcomeUnknown`.

Tests: `S6B-IDEM-*`, response-loss injection at every boundary.

## Phase 4 — E6-A query/result orchestration

Implement status/provider/generation validation, query, continuation, result get/list/validate, explain, artifact build, and cache validate by invoking `wow-cbm` only.

Tests: `S6B-QUERY-*`, `S6B-CONT-*`, E6-A regression suite.

## Phase 5 — owner mapping seams

Implement typed orchestration over:

```text
ProjectExternalLocatorMappingPort
ReferenceExternalLocatorMappingPort
```

Validate exact receipts; no heuristics.

Tests: `S6B-MAP-*` and owner contract fixtures.

## Phase 6 — explicit selection

Implement validate/record under durable idempotent receipt semantics. Prohibit rank/position/name/path shortcuts and authority/permission upgrades.

Tests: `S6B-SEL-*`.

## Phase 7 — exact-root context composition

Invoke existing E3-C context use case through `ContextUseCasePort`; keep external evidence in outer envelope and validate excluded provider metadata.

Tests: `S6B-CTX-*` and E3-B/E3-C regressions.

## Phase 8 — retention, cache, audit, and optional degradation

Implement exact retention closure, GC-race handling, catalog snapshots, cache validation, audit chains, reverse close, and provider-scoped degradation.

Tests: `S6B-RET-*`, security/privacy subsets.

## Phase 9 — canonical envelopes and serialization

Implement conservative status folding, authority/nonclaim records, redaction, canonical JSON, and deterministic bytes.

Tests: result/error golden vectors, `S6B-DET-*`.

## Phase 10 — thin CLI

Activate `apps/wow/e6` only after service request/result/error/operation bytes and exit mappings freeze. App depends only on `wow-service` and makes one service call per valid command.

## Phase 11 — cross-package and adversarial evaluation

Run:

- E0–E5 regression suites;
- E6-A owner suite;
- E6-B service/app matrices;
- stable/mutable/opaque synthetic provider corpus;
- configured real provider integration under nonsecret deployment credentials;
- forged/revoked/replayed authorization/session receipts;
- mapping ambiguity/partial/no-authority cases;
- rank/top/sole selection mutations;
- context-injection/privacy/license cases;
- response loss/cancellation/close failures at every effect boundary;
- 1/2/N workers, shuffled scheduling, cold/warm cache;
- ordinary/adversarial resource benchmarks.

## Phase 12 — freeze implementation evidence

Populate prerequisite commits, adapters/ports, profiles, vectors, thresholds, canonical bytes, and checksums. Update `crates/MANIFEST.json` only with fresh evidence.

## Deferred to E7+

- generic MCP/LSP/session/daemon protocol exposure;
- arbitrary provider plugin management;
- provider index/import/delete lifecycle;
- provider privacy deletion/anti-resurrection workflow;
- external Candidate fusion with E4 search;
- external Candidate graph publication or lineage;
- source edits/tool execution/runtime validation;
- release/public distribution and CI.
