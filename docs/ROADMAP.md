# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E4-A complete
next documentation package: E4-B explicit lineage/migration/impact
implementation frontier: not started
```

No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist yet.

## Milestone summary

| Milestone | Documentation outcome | Documentation | Implementation |
|---|---|---:|---:|
| E0 | Deterministic diagnostic vertical slice | Complete | Not started |
| E1 | ReferenceStore, ReferenceView, annotations, pack build/validation | Complete | Not started |
| E2 | Graph, recognizers, full project candidate, ProjectStore publication | Complete | Not started |
| E3-A | Exact Blizzard UI source universe and SkeletonInputView | Complete | Not started |
| E3-B | Project Map, L0/L1, context packs and rendering | Complete | Not started |
| E3-C | Service/application context acquisition and use cases | Complete | Not started |
| E4-A | Exact-generation search shards, retrieval lanes, ranking and explanations | Complete | Not started |
| E4-B | Explicit lineage, migration, replacement/removal/introduction and impact | Next | Not started |
| E4-C | Search/lineage/impact service and CLI orchestration | Planned | Not started |
| E5 | Named calibration packs with universal outputs | Planned | Not started |
| E6 | Optional Codebase Memory candidate bridge | Planned | Not started |
| E7 | LSP/MCP, release, signing, publication and rollback | Planned | Not started |

## E0 — executable diagnostic slice

Build `wow-core`, fixture `wow-reference`, `wow-emmy`, minimal `wow-project`, two bounded rules, service, and thin diagnostic CLI only after all E0 fixtures/profiles/checksums freeze.

Gate: one exact profile/project/reference/analyzer context; generic + WoW findings; honest negative authority and NotEvaluated; no source/editor/client mutation; byte-identical output.

## E1 — Reference Pack

Build generic storage, persistent reference ingestion/corrections/coverage, deterministic annotations, and nonrepairing pack build/validation.

Gate: exact pins; immutable validated ReferenceStore; raw unknown/correction conflicts retained; annotation loss/injection/editor gates; pack checksum/license/coverage closure; deterministic rebuild classification.

## E2 — graph, project and persistence

Build graph assertions/partitions/queries, structural recognizers, full TOC/XML/load/analyzer indexing, invalidation, and WAL manifested-partition ProjectStore publication.

Gate: producer-independent identity; atomic replacement; safe parsers; no second Lua parser; exact old/new readers; inactive read-back before current CAS; crash/response-loss/lease/GC/backup tests; logical determinism.

## E3 — source and context

### E3-A

Build a separate exact platform-source project with pinned materialization, package/TOC/XML/Lua/analyzer/recognizer/graph publication, license/coverage, incremental replacement, fingerprints, and bounded `SkeletonInputView`.

### E3-B

Build exact-universe binding, maps, L0/L1, expansion/selection/pruning, source boundaries, semantic packs, JSON/Markdown, cache identities, metrics, and evaluation.

### E3-C

Build exact/current selector resolution, retained owner view acquisition, E3-B use-case orchestration, continuation retention, result envelopes, cancellation/closure, and thin context CLI commands.

## E4-A — exact-generation search core

Documented contract:

- [`../crates/wow-search/e4/README.md`](../crates/wow-search/e4/README.md)

Build:

- one immutable SearchShard per exact user-project, Blizzard UI, or Reference generation;
- bounded typed SearchDocuments and exact field origins;
- case-sensitive exact identity/name/alias/member/prefix lanes;
- safe generation-local FTS5 text lane;
- deterministic identifier similarity and structured shape;
- seeded bounded graph reason paths;
- authority bands and integer/ordinal fusion;
- complete ranking explanations;
- exact scoped miss gates;
- immutable result-set manifests, whole-candidate pages, exact continuation;
- SearchStore integrity/privacy/security/evaluation.

Gate:

```text
no combined current/global FTS corpus
no raw cross-shard FTS score comparison
no inferred aliases
no raw query syntax or executable extension
no context dependency or hidden candidate selection
no approximate result promoted to lineage/replacement/negative authority
zero false exact/alias/authoritative-miss/lineage claims
complete owner/document/index/lane coverage accounting
deterministic ranking and pages across workers/order/cache/storage layout
exact SQLite/FTS/tokenizer/platform probes and measured thresholds
all fixture/checksum pins frozen
```

## E4-B — lineage, migration, and impact

Define explicit before/after generation models and independently validated assertions/candidates for:

```text
same entity
renamed
moved
replaced or superseded
removed
introduced
signature/type/restriction/contract change
migration compatibility
bounded static impact
```

E4-A rank and similarity are Candidate evidence only. Authoritative conclusions require exact source/reference/project/graph evidence, generation/profile closure, conflict/coverage resolution, and explicit proof ceilings.

## E4-C — service and CLI

Expose search, candidate explanation, explicit candidate selection, lineage, migration, and impact through `wow-service` and `apps/wow`.

`wow-context` continues to accept exact selected roots only. Service does not hide ranking or auto-select a replacement without an explicit reviewed policy and visible evidence.

## E5–E7

- E5: audited calibration packs over universal facts with repository/name/path mutation tests.
- E6: optional Codebase Memory bridge; candidates remain external and degradable.
- E7: thin LSP/MCP and release/signing/publication/rollback operations.

## Roadmap discipline

- Later documentation cannot bypass earlier implementation gates.
- Architecture changes require an ADR and concrete failure of the accepted design.
- Stable contracts link the current external WoW engineering KB rather than copying patch-sensitive claims.
- Missing tools/probes/benchmarks/evaluations/client tests are skipped or NotEvaluated, never pass.
- Outcomes and proof gates matter; percentages and directory counts do not.
