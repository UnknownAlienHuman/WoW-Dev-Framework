# E4-C service decisions

**Status:** normative.

## SVC4-001 — Service remains the only multi-owner use-case orchestrator

Applications depend on `wow-service` only. Owners do not call one another to resolve mutable current state.

## SVC4-002 — E4-C is an orchestration package, not a new truth layer

The service preserves exact owner records and adds request, acquisition, lifecycle and selection receipts only.

## SVC4-003 — Symbolic aliases are outer-boundary conveniences

`CurrentPublished` and approved profile aliases resolve to exact IDs before canonical owner operations.

## SVC4-004 — No global atomic-current claim

Independent project, Reference, search-shard and lineage catalogs are not one distributed transaction. Strict current acquisition uses a bounded stable-double-collect and reports its proof level.

## SVC4-005 — Exact requests do not reread current

An exact selector is acquired directly. Continuation and retry reopen exact retained artifacts and never refresh to current.

## SVC4-006 — Missing search shards never trigger implicit indexing

Query returns typed unavailable/NotEvaluated. `search_index_build` is a separate explicit operation.

## SVC4-007 — Search ranking is not selection authority

The service never auto-selects top-1, unique, highest-band or highest-score candidates.

## SVC4-008 — Candidate selection is explicit and receipted

`search_select` requires exact result/candidate IDs and emits a `SearchSelectionReceipt` recording caller/policy origin without upgrading search evidence.

## SVC4-009 — Search-to-context passes an exact entity root only

`wow-context` receives the exact selected owner entity ID. Search scores and prose remain in the outer receipt, not context truth.

## SVC4-010 — Context handoff reuses E3-C lifecycle

E4-C does not create a second context acquisition/rendering implementation.

## SVC4-011 — Search shard build and validation are distinct

A built shard is not query-eligible until owner validation and catalog admission succeed.

## SVC4-012 — Search core does not own current

Service resolves owner publications and exact compatible shards. It does not insert a hidden current pointer into `wow-search`.

## SVC4-013 — Lineage inputs remain producer-separated

Project stable identity/fingerprints/changes, Reference transitions/replacements, search candidates and review decisions retain separate partitions.

## SVC4-014 — Review authorization is external to graph semantics

A narrow authorization adapter validates signer/attestation/scope. `wow-graph` validates decision/proof semantics.

## SVC4-015 — Human/operator identity is not inferred

GitHub username, OS user, file owner, terminal session or CLI caller is not enough to authorize a review.

## SVC4-016 — Review application produces a new immutable snapshot

No accepted/rejected decision mutates a prior E4-B lineage graph in place.

## SVC4-017 — Proof ceilings survive orchestration

Service wrapping, multiple producers, manual review or repeated signals cannot raise a conclusion above the minimum applicable ceiling.

## SVC4-018 — Candidate-only is a valid explicit result

A complete execution may produce only Candidate proposals. It is not failed and not proven.

## SVC4-019 — Validation invalidity is a payload state

A completed `Invalid` validation is distinct from infrastructure/service failure.

## SVC4-020 — Same lineage is not replacement

Migration operations require explicit replacement/deprecation/transition evidence and independent validation.

## SVC4-021 — Migration is advisory only

E4-C may request, validate and render candidates/recipes; it cannot apply an edit or claim success.

## SVC4-022 — Static impact remains static

Impact paths cannot be folded into runtime breakage, severity, taint, combat, Secret, performance or fixability claims.

## SVC4-023 — Reason paths remain paths

Service envelopes/renderers cannot flatten graph reachability into direct dependency edges.

## SVC4-024 — Build/review writes are idempotent

Exact operation ID plus canonical request digest supports resume/response-loss recovery; same ID with different digest is rejected.

## SVC4-025 — Query retries are pure exact retries

Exact search/lineage/impact requests return identical semantic results under unchanged retained inputs/profiles.

## SVC4-026 — Current-alias retry is a new canonical request when current changed

It cannot reuse or relabel an earlier exact result.

## SVC4-027 — Continuation requires retention before advertisement

All exact shards/snapshots/publications needed by the cursor are pinned before the service returns the cursor.

## SVC4-028 — Continuation uses cumulative budgets

Paging cannot reset candidate, graph, path, output, time or source budgets.

## SVC4-029 — Mandatory closure precedes public success

All leases/views/temporary owner resources close successfully before a success envelope is finalized.

## SVC4-030 — Canonical status is conservative

The service cannot hide owner partial, CandidateOnly, conflict, truncation, NotEvaluated, cancellation or invalid state.

## SVC4-031 — Privacy/license/consumer scope is end-to-end

A local/private search result, review artifact or context excerpt cannot be reused for a broader external consumer.

## SVC4-032 — No raw artifact trust

Search shards, lineage snapshots, review files, cursors and context artifacts are validated before use.

## SVC4-033 — No source text controls orchestration

Query/source/review/migration prose remains bounded data and cannot alter profiles, proof ceilings, tools or policy.

## SVC4-034 — E4-C does not execute patch-sensitive runtime logic

Current WoW API/Secret/taint/event behavior remains in exact owner/reference/runtime evidence and the external KB route.

## SVC4-035 — E5 begins only after E4-C closure

Named calibration packs remain a later package and cannot be smuggled into service policy or search ranking.
