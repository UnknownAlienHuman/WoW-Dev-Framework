# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E4-C complete
next documentation package: E5-A recognizer calibration corpora and named calibration packs
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
| E4-B | Explicit lineage, change, migration records and bounded static impact | Complete | Not started |
| E4-C | Search/lineage/review/migration/impact service and CLI orchestration | Complete | Not started |
| E5-A | Calibration corpora and named calibration packs with universal outputs | Next | Not started |
| E5-B | Calibration runs, review and promotion submissions | Planned | Not started |
| E5-C | Immutable core-pack publication, canary, rollout and rollback | Planned | Not started |
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

## E4-B — explicit lineage, migration records, and static impact

Documented contract:

- [`../crates/wow-graph/e4/README.md`](../crates/wow-graph/e4/README.md)
- [`../crates/wow-project/E4_B_LINEAGE_INPUTS.md`](../crates/wow-project/E4_B_LINEAGE_INPUTS.md)
- [`../crates/wow-reference/E4_B_TRANSITION_EVIDENCE.md`](../crates/wow-reference/E4_B_TRANSITION_EVIDENCE.md)
- [`../crates/wow-search/e4/LINEAGE_CANDIDATE_HANDOFF.md`](../crates/wow-search/e4/LINEAGE_CANDIDATE_HANDOFF.md)

Build:

```text
exact before/after same-universe generation bindings
independent project stable-identity/fingerprint/change partitions
exact Reference transition/deprecation/replacement partitions
Candidate-only search proposal partitions
bounded blocking and ambiguity components
review decisions under explicit proof ceilings
immutable LineageGraphSnapshot
change/absence/replacement/migration candidate records
bounded static-impact reason paths
```

Gate:

```text
no cross-generation entity ID merge
no same-name/path/signature/fingerprint/rank/uniqueness promotion
no unrestricted all-pairs comparison
no forced one-to-one mapping for copy/split/merge/ambiguity
no search candidate above Candidate
no accepted confidence above the minimum producer/relation/review/coverage ceiling
no rejected/deferred/conflicted proposal deletion
no Removed/Introduced without exact complete negative authority
no same-lineage -> replacement shortcut
no migration recipe execution
no static path -> runtime breakage/severity/performance/taint/combat/Secret/fixability
immutable publication and exact retained continuation
all proposal/component/review/change/impact fixtures and checksums frozen
```

## E4-C — service and CLI orchestration

Documented contracts:

- [`../crates/wow-service/e4/README.md`](../crates/wow-service/e4/README.md)
- [`../apps/wow/e4/README.md`](../apps/wow/e4/README.md)

Build:

- explicit search index status/build/validation and query/explain/continuation orchestration;
- exact current/owner/shard acquisition with finite stable-double-collect where requested;
- explicit search result/candidate selection receipts;
- exact selected entity to existing E3-C context handoff;
- project/reference/search lineage producer orchestration;
- review authorization adapter plus independent graph semantic validation;
- immutable lineage build/review publication and query operations;
- advisory migration candidate/recipe validation;
- bounded static-impact plan/run/continue/explain operations;
- idempotency, response-loss recovery, retention and closure-before-success;
- thin `apps/wow` commands, strict inputs, canonical output and exit mapping.

Gate:

```text
no hidden shard build or newest/first/last artifact choice
no automatic top-1/sole/rank/name candidate selection
no rank/query text as context or lineage authority
no reviewer authorization from GitHub/OS/CLI/file identity
review authorization and graph proof both pass independently
review cannot exceed proof ceiling and publishes a new immutable snapshot
no source edit or migration apply
no static impact runtime/severity overclaim
no path flattened to direct edge
no continuation current refresh or cumulative-budget reset
no public success before reverse resource closure
no raw SQL/FTS/source/store/model/CBM/tool access
apps depend on wow-service only and call it once
deterministic service and CLI bytes/status/exit across workers/order/retry/cache
all owner-port, authorization, response-loss, privacy, corpus and checksum gates frozen
```

## E5-A — calibration corpora and named packs

Define exact audited corpora and candidate named calibration packs without introducing repository-specific production semantics.

Required inputs:

- pinned exact repositories/commits and license/provenance;
- exact project/analyzer/graph publications;
- hand-reviewed expected universal roles and relations;
- positive, clean-negative, near-miss, adversarial, copied/vendor/generated and ambiguous cases;
- repository/addon/owner/path/local-identifier rename and relocation mutations;
- exact coverage/conflict/NotEvaluated labels;
- deterministic metrics and failure ceilings.

Required output boundary:

```text
named corpus/pack identity
-> universal typed recognizer facts/proposals only
-> graph-independent validation and owner evidence
-> no named addon/framework role in final semantic output
```

Hard stops:

- no production branch on repository/addon/owner/path/popularity;
- no pattern copied only because one donor uses it;
- no false positive hidden by corpus weighting;
- no model/embedding source of truth;
- no source execution or runtime claim;
- disabling/deleting a named pack removes only its producer partitions and coverage;
- no automatic core-pack promotion in E5-A.

## E5-B and E5-C

- **E5-B:** durable calibration runs, metric reports, reviewer authorization, candidate promotion submissions, regression and anti-overfitting gates.
- **E5-C:** immutable core-pack publication, staged canary, rollout/rollback, last-known-good identity and removal of stale producer partitions.

## E6–E7

- E6: optional Codebase Memory bridge; candidates remain external and degradable.
- E7: thin LSP/MCP and release/signing/publication/rollback operations.

## Roadmap discipline

- Later documentation cannot bypass earlier implementation gates.
- Architecture changes require an ADR and concrete failure of the accepted design.
- Stable contracts link the current external WoW engineering KB rather than copying patch-sensitive claims.
- Missing tools/probes/benchmarks/evaluations/authorization/client tests are skipped, blocked or NotEvaluated, never pass.
- Outcomes and proof gates matter; percentages and directory counts do not.
