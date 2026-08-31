# E3-A implementation plan

**Status:** normative order; Rust implementation has not started.

## Phase 0 — prerequisites and freeze

- implement and freeze `wow-core`, persistent `wow-reference`, E2-A `wow-graph`, E2-C `wow-project`, and E2-D coherent publication/read views;
- verify the selected E2-D WAL/partition contract and remove every `StoreImageId` assumption;
- freeze exact project/graph/reference read catalogs and input-snapshot seam;
- freeze context, Project Map, L0/L1, control/effect, expansion, source, budget, tokenizer, security, renderer, and evaluation profiles;
- freeze synthetic project/reference/platform-source fixture and pinned real user-addon fixture;
- populate every required fixture/checksum pin before the first Rust commit.

## Phase 1 — bounded profile/value primitives

Implement profile registries, exact input snapshot, request, budget, coverage/loss/omission/stopping/status value types, canonicalization, and validation. No graph/project queries yet.

Tests: profile, input identity, budget, canonical JSON, enum/unknown-field, generation mismatch.

## Phase 2 — read-view adapters

Implement narrow adapters over exact `ProjectView`, `GraphView`, and optional `ReferenceView`. No raw store/analyzer/source path access. Validate universe/query catalog/capability closure.

Tests: coherent snapshot, old retained view, mixed generation, missing capability, root statuses, project-use versus platform evidence.

## Phase 3 — L0 semantic skeletons

Implement exact identity/owner/load/role/public-surface/direct-relation skeleton records, evidence sidecars, blocker fields, detail routes, ordering, and canonical IDs.

Tests: all supported subject families, unsupported kinds, partial/conflict, no body/prose inference, deterministic grouping.

## Phase 4 — L1 signatures and control/effects

Implement exact signatures/members/direct reason paths and the closed control/effect node registry over published facts. Unknown/collapsed/omitted regions remain explicit. No second parser/CFG/data-flow engine.

Tests: branches/loops/calls/registrations/hooks/state/API uses, dynamic/possible facts, source spans, tight budget, no safety overclaim.

## Phase 5 — Project Map

Implement principal-root selection, frozen sections, exact grouping, mandatory budget reserve, next-detail routes, evidence/coverage/conflict summary, and strict default byte profile.

Tests: exact 2 KiB default renderer target/gate policy, selected TOC variant, load/static nonclaims, signal/state/API separation, repository-name mutations.

## Phase 6 — progressive expansion

Implement context plan/frontier/work items, lane-specific exact queries, deterministic merge/dedup, cycles, stopping, no-new-evidence, continuation, and total-budget inheritance.

Tests: direct/path distinction, every lane, high fanout, cycles, partial/conflict boundaries, cursor tampering/generation switch, 1/2/N order.

## Phase 7 — source excerpts and security

Implement exact source-handle resolution through owning views, faithful spans, deterministic surrounding context, license/privacy/security checks, prompt/container/terminal escaping, redaction/loss, and source budgets.

Tests: stale digest/path, virtual Lua, malicious comments, invalid UTF-8/control bytes, private/runtime data, unlicensed source, no mutation.

## Phase 8 — bundle/rendering/tokenizer

Implement semantic bundle assembly/validation, canonical semantic JSON, Markdown/compact renderers, renderer loss sidecars, exact bytes/scalars/lines, optional pinned-tokenizer adapter, and estimate separation.

Tests: semantic equivalence across renderers, canonical bytes, escaping, exact tokenizer vectors, estimate labeling, renderer budget.

## Phase 9 — metrics and evaluation

Implement structural/evidence recall, honesty, relevance, redundancy, compression, detail efficiency, budget/continuation/source metrics, frozen evaluation reports, and supplemental external consumer harness interface outside the crate.

Tests: hard-gate failures, false dedup, smaller-but-wrong context, malicious prompt task, profile tuning comparison, deterministic report.

## Phase 10 — integrated fixtures

Run:

```text
synthetic closed project/reference/graph/platform-source fixture
pinned UnknownAlienHuman/roth-ui publication fixture
repository/package/path/local-name mutations
partial/conflict/high-fanout/cycle/security fixtures
```

Compare Project Map/L0/L1/bundle/continuation under 1/2/N workers and shuffled inputs.

## Phase 11 — freeze outputs

Populate implementation commits, query/profile IDs, all expected semantic artifact IDs, canonical bytes/digests, tokenizer vectors, evaluation baselines, and member/bundle SHA-256. Tests verify committed fixtures and never rewrite them.

## Explicitly deferred

- real Blizzard UI source acquisition/index producer (E3-B);
- service/API orchestration for context use cases (E3-C or later);
- search/FTS/lineage/impact (E4);
- named calibration packs (E5);
- Codebase Memory candidates (E6);
- LSP/MCP/network/release/CI (E7);
- context persistence/cache unless separately contracted.
