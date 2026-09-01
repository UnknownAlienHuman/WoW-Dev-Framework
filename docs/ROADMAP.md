# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E3-C complete
next documentation package: E4-A wow-search core retrieval
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
| E4-A | Exact/alias/FTS/shape/graph search and ranking explanations | Next | Not started |
| E4-B | Explicit lineage, migration, replacement candidates and impact | Planned | Not started |
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

## E3-A — Blizzard UI source universe

Build a separate exact platform-source project with pinned materialization, package/TOC/XML/Lua/analyzer/recognizer/graph publication, license/coverage, incremental replacement, fingerprints, and bounded `SkeletonInputView`.

Gate: exact source/build/profile; no flavor merge or source execution; implementation source not promoted to API/runtime authority; separate store/snapshots; removal closure; bounded reads; redistribution explicit.

## E3-B — Project Map and context

Build exact-universe binding, maps, L0/L1, closed control/effect projection, expansion/selection/pruning, source boundaries, semantic packs, JSON/Markdown, cache identities, metrics, and evaluation.

Gate: exact roots/views; separate universes; origin/evidence/coverage/conflict closure; mandatory records never pruned; omissions explicit; exact tokens only with frozen tokenizer; source remains untrusted data; byte-identical deterministic outputs.

## E3-C — service and application context use cases

Build exact current/exact selector resolution, retained owner view acquisition, E3-B use-case orchestration, continuation retention, result envelopes, cancellation/closure, and thin context CLI commands.

Gate:

```text
current resolved once by service only
independent stores not misrepresented as globally atomic
no hidden retry/fallback/LKG substitution
exact ReferenceView and compatibility validation
fixed acquire/reverse-release order
no public success before close
continuation reopens exact retained generations
invalid artifact stays an Invalid validation payload, not internal failure
service/app do not reimplement owner algorithms
app imports service only
canonical JSON/artifact bytes and exit codes frozen
no implicit config/source/client discovery or background work
```

## E4-A — search core

Define and implement:

- exact ID/name/symbol lookup;
- reviewed aliases/corrections;
- FTS over bounded indexed text fields;
- structural shape retrieval;
- bounded graph neighborhood/path-assisted retrieval;
- per-lane candidates, scores, evidence, coverage and conflicts;
- deterministic fusion/ranking/tie-breaking;
- explanations and explicit query normalization;
- snapshot-bound pagination/continuation;
- service operation and thin CLI projection.

`wow-search` does not call `wow-context`; service passes an explicitly selected exact result to context. Search result ranking never establishes lineage, replacement, safety, or platform truth.

## E4-B — lineage, migration and impact

Add explicit cross-generation assertions/candidates for same entity, moved/renamed/replaced/removed/introduced states, migration recipes, and bounded impact plans. Similarity is candidate evidence only. Every authoritative conclusion needs exact profile/generation/evidence/coverage/conflict closure.

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
