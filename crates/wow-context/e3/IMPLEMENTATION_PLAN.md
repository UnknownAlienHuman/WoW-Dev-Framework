# E3-B implementation plan

**Status:** normative order; implementation not started.

## Phase 0 — reconcile and freeze the contract

- accept `MILESTONE_RENUMBERING.md`;
- remove any duplicate E3-A/E3-B context type or operation from the implementation design;
- implement and freeze all E0-E2 prerequisites;
- implement and freeze E3-A `wow-project` Blizzard UI source index and `SkeletonInputView`;
- freeze public project, graph, reference, and source-detail read catalogs;
- freeze context schemas, profiles, examples, expected IDs, canonical bytes, and checksums;
- update repository crate manifest and workstream activation before Rust code.

No placeholder `Cargo.toml` or `.rs` file before this gate.

## Phase 1 — value types, profiles, and canonicalization

Implement:

- `ContextUniverseSet`;
- current terminology and historical aliases as one migration layer;
- profile schemas and registries;
- exact root selectors;
- budget, token, source, privacy, boundary, and renderer profiles;
- canonical IDs, ordering, and serialization;
- typed errors and cancellation.

Tests: milestone, schema, profile, request, canonicalization, and freeze groups.

## Phase 2 — universe binding and owner view adapters

Implement narrow adapters over exact public:

```text
wow-project ProjectView and SkeletonInputView
wow-graph GraphView
wow-reference ReferenceView
```

No storage, analyzer, parser, or search access.

Tests: generation compatibility, optional platform source, partial/conflict, wrong-universe records, stable old reader while current advances.

## Phase 3 — Project Map

Implement map nodes, edges, groups, facets, exact evidence/coverage, ordering, budget, omission, and continuation over synthetic fixtures.

No source excerpts.

Tests: map identity, grouping, path/direct distinction, multi-universe separation, fanout, determinism.

## Phase 4 — L0 skeletons

Implement bounded container scopes, typed sections, declaration summaries, direct owner/load/dependency relations, role evidence, counts, member pages, and expansion routes.

Tests: body exclusion, name/path mutations, partial counts, mandatory closure, pagination.

## Phase 5 — L1 and control/effect projections

Implement exact entity records, signatures, types, spans, direct relations, reason paths, API/reference enrichment, event/hook/state distinctions, source-excerpt candidates, and the inherited closed control/effect projection over published facts.

Tests: possible/dynamic relations, no second parser/CFG, no runtime-safety claims, direct/path distinction.

## Phase 6 — expansion, selection, budgets, and continuation

Implement:

- deterministic stage planner and frontier;
- candidate dependency graph;
- mandatory closure;
- optional tier/tie selection;
- semantic byte costs;
- exact/estimated token accounting interfaces;
- no-new-evidence and all stop states;
- exact continuation chain.

Tests: cycles, high fanout, tight budgets, mandatory overflow, continuation changes, cancellation, 1/2/N workers.

## Phase 7 — source excerpts and trust boundaries

Implement exact selected-source retrieval, source-map handling, privacy/license/consumer decisions, deterministic transformations, structural JSON/Markdown data boundaries, and excerpt budgets/continuation.

Tests: hostile structured source strings, range/digest mismatch, private data, denied source, virtual units, boundary round trips.

## Phase 8 — semantic pack and renderers

Implement `ContextSemanticPack`, nonrepairing validation, canonical JSON, deterministic Markdown, item/output mapping, rendering loss, exact byte measurement, and optional exact tokenizer integration.

Tests: identity DAG, JSON round trip, semantic/render separation, token failure, renderer overflow, source boundaries.

## Phase 9 — cache keys, comparison, metrics, and evaluation

Implement storage-independent cache keys and validation, exact pack comparison, deterministic metrics, and the frozen evaluation harness. Physical cache and orchestration remain external.

Tests: stale, corrupt, cross-privacy, and cross-generation cache entries; cold/warm equivalence; consumer evaluation hard gates.

## Phase 10 — integration corpora

Run:

- tiny synthetic project;
- complex synthetic TOC/XML/load/event/hook/state/cycle corpus;
- pinned `roth-ui` published project fixture;
- pinned E3-A Blizzard UI source publication fixture;
- exact ReferenceView fixture;
- repository/package/path/name mutation corpus;
- privacy/license/source-boundary corpus;
- 1/2/N workers and shuffled owner results;
- tight, normal, and large context profiles;
- canonical JSON and Markdown;
- exact tokenizer and fallback estimator profiles where active.

## Phase 11 — freeze implementation outputs

Populate every null prerequisite, profile, corpus, artifact, expected error/result, benchmark, and SHA-256 value. Tests verify committed bytes and never rewrite fixtures automatically.

## Deferred

- fuzzy and natural-language search;
- lineage, migration, patch impact, and candidate similarity;
- named calibration packs;
- Codebase Memory;
- runtime/client observations;
- model-generated summaries;
- diagnostics, fixes, edits, and planning;
- physical cache storage;
- service/application transport and CI.

## Definition of Done

- one context implementation and one terminology model;
- exact universe/generation binding;
- map, L0, L1, pack, and render operations implemented from public views only;
- origin, evidence, coverage, conflict, and omission closure;
- deterministic selection and bytes;
- exact token claims only with a frozen tokenizer;
- mandatory trust metadata never pruned;
- source data structurally isolated;
- no side effects or background work;
- all nondeferred tests and frozen checksums pass.
