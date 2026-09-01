# E4-C implementation plan

**Status:** normative dependency order; implementation has not started.

## Phase 0 — prerequisite and freeze closure

Before any E4-C Rust source:

- implement/freeze all E0–E4-B prerequisite contracts and fixtures;
- freeze E3-C acquisition/lifecycle/result behavior;
- freeze exact owner ports for project/reference/store/search/graph/context;
- freeze SearchShard and LineageGraph catalogs plus retention/idempotency contracts;
- select and freeze review authorization adapter/profile without embedding credentials in fixtures;
- freeze all E4-C requests, outcomes, CLI bytes, error/status/exit maps, corpora, benchmarks and checksums.

No placeholder service modules or unavailable-as-success stubs.

## Phase 1 — request and selector primitives

Implement:

```text
E4ServiceConfiguration
closed request tagged unions
exact/current/catalog selectors
normalized request IDs
budget/privacy/profile validation
```

Tests: `S4-CONF-*`, `S4-SEL-*` excluding owner calls.

## Phase 2 — acquisition/lifecycle coordinator

Implement one reusable internal coordinator that extends E3-C semantics:

- canonical acquisition/release order;
- exact owner/shard/lineage view guards;
- stable-double-collect for strict current requests;
- partial-acquisition cleanup;
- retention admission;
- closure-before-success.

Do not create a generic callback framework; use narrow typed owner ports.

Tests: `S4-ACQ-*`, `S4-LIFE-*` acquisition/closure subsets.

## Phase 3 — search shard build/status/validation

Implement service orchestration only:

```text
search_index_status
search_index_build
search_index_validate
```

Integrate durable idempotency and response-loss receipts. E4-A owns document/shard algorithms.

Tests: `S4-SIDX-*` and security/resource cases.

## Phase 4 — search query/explain/continuation

Implement:

```text
search_query
search_continue
search_explain
```

Preserve exact owner results and continuation retention. No service ranking logic.

Tests: `S4-SEARCH-*`, continuation/status/determinism subsets.

## Phase 5 — explicit selection and context handoff

Implement:

```text
search_select
search_context
```

Reuse E3-C context operation internals through an explicit service-owned composition seam. Do not call the public transport adapter recursively.

Tests: `S4-SELECT-*`, `S4-CTX-*`.

## Phase 6 — lineage producer/build/validation/query

Implement orchestration for:

```text
lineage_status
lineage_build
lineage_validate
lineage_compare
lineage_trace
lineage_explain
```

Project/reference/search produce typed inputs; graph owns candidate/proof/publication/query behavior.

Tests: `S4-LIN-*`, build idempotency/response-loss cases.

## Phase 7 — review authorization and immutable apply

Implement the narrow authorization adapter port and:

```text
lineage_review_validate
lineage_review_apply
```

Authorization and graph semantic validation remain independent. Applying decisions publishes a new immutable snapshot.

Tests: `S4-REV-*`, key/profile/replay/scope/security fixtures.

## Phase 8 — migration and static impact

Implement:

```text
migration_candidates
migration_validate
impact_plan
impact_run
impact_continue
impact_explain
```

No edits or runtime claims.

Tests: `S4-MIG-*`, `S4-IMP-*`.

## Phase 9 — status/envelopes/serialization

Implement tagged outcomes, conservative status folding, validation payload state, nonclaims, privacy redaction and canonical JSON bytes.

Tests: `S4-RES-*`, failure/cancellation/privacy cases.

## Phase 10 — thin CLI integration

Implement `apps/wow/e4` only after service bytes/errors are frozen. The app depends on `wow-service` only and makes one service call per command.

## Phase 11 — cross-package regression and corpus evaluation

Run:

- E0 status/check and E1 Reference Pack regressions;
- E3 context operations;
- E4-A search owner tests;
- E4-B lineage owner tests;
- E4-C service/app matrices;
- synthetic, pinned `roth-ui`, pinned Blizzard UI, exact Reference transition, ambiguity/split/merge/copy, privacy/review/security, high-fanout impact corpora;
- 1/2/N workers, shuffled owner scheduling, cold/warm caches, response loss and cancellation.

## Phase 12 — freeze implementation evidence

Populate implementation commit, owner-port, profile, fixture, benchmark, canonical byte, exit-code and SHA-256 values. Update `crates/MANIFEST.json` only with exact passing evidence.

## Deferred

- applying migrations or source edits;
- runtime/client impact probes;
- E5 calibration packs;
- E6 external/Codebase Memory/model candidates;
- E7 LSP/MCP/release/signing/publication;
- CI unless explicitly requested by the owner.
