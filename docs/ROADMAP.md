# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E5-A complete
next documentation package: E5-B calibration orchestration, review, holdout audit and promotion submissions
implementation frontier: not started
```

No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist yet.

## Milestone summary

| Milestone | Documentation outcome | Documentation | Implementation |
|---|---|---:|---:|
| E0 | Deterministic diagnostic vertical slice | Complete | Not started |
| E1 | ReferenceStore, ReferenceView, annotations, pack build/validation | Complete | Not started |
| E2 | Graph, core recognizers, full project candidate, ProjectStore publication | Complete | Not started |
| E3-A | Exact Blizzard UI source universe and SkeletonInputView | Complete | Not started |
| E3-B | Project Map, L0/L1, context packs and rendering | Complete | Not started |
| E3-C | Service/application context acquisition and use cases | Complete | Not started |
| E4-A | Exact-generation search shards, retrieval lanes, ranking and explanations | Complete | Not started |
| E4-B | Explicit lineage, change, migration records and bounded static impact | Complete | Not started |
| E4-C | Search/lineage/review/migration/impact service and CLI orchestration | Complete | Not started |
| E5-A | Calibration corpora, named shadow packs, anti-overfitting evaluation, candidate/deactivation artifacts | Complete | Not started |
| E5-B | Durable calibration runs, reviewer authorization, sealed-holdout audit, promotion submissions | Next | Not started |
| E5-C | Immutable core-pack publication, canary, rollout and rollback | Planned | Not started |
| E6 | Optional Codebase Memory candidate bridge | Planned | Not started |
| E7 | LSP/MCP, release, signing, publication and rollback | Planned | Not started |

## E0–E2 implementation foundation

### E0 — executable diagnostic slice

Build `wow-core`, fixture `wow-reference`, `wow-emmy`, minimal `wow-project`, two bounded rules, service, and thin diagnostic CLI only after all E0 fixtures/profiles/checksums freeze.

Gate: one exact profile/project/reference/analyzer context; generic + WoW findings; honest negative authority and NotEvaluated; no source/editor/client mutation; byte-identical output.

### E1 — Reference Pack

Build generic storage, persistent reference ingestion/corrections/coverage, deterministic annotations, and nonrepairing pack build/validation.

Gate: exact pins; immutable validated ReferenceStore; raw unknown/correction conflicts retained; annotation loss/injection/editor gates; pack checksum/license/coverage closure; deterministic rebuild classification.

### E2 — graph, project and persistence

Build graph assertions/partitions/queries, E2-B structural recognizers, full TOC/XML/load/analyzer indexing, invalidation, and WAL manifested-partition ProjectStore publication.

Gate: producer-independent identity; atomic replacement; safe parsers; no second Lua parser; exact old/new readers; inactive read-back before current CAS; crash/response-loss/lease/GC/backup tests; logical determinism.

## E3 — source and context

- **E3-A:** separate exact Blizzard UI source project with pinned materialization, package/TOC/XML/Lua/analyzer/recognizer/graph publication, license/coverage, fingerprints and bounded `SkeletonInputView`.
- **E3-B:** exact-universe binding, Project Map/L0/L1, deterministic expansion/selection/pruning, source/privacy/license boundaries, semantic packs, rendering, continuation/cache identities and metrics.
- **E3-C:** exact/current selector resolution, retained owner acquisition, context orchestration, continuation retention, result envelopes, cancellation/closure and thin context CLI.

## E4 — search, lineage and impact

### E4-A — exact-generation search core

Contract: [`../crates/wow-search/e4/README.md`](../crates/wow-search/e4/README.md)

Build immutable SearchShards per exact owner generation, bounded typed documents, exact/alias/member/prefix/text/similarity/shape/graph lanes, authority bands, deterministic integer/ordinal fusion, complete explanations, honest miss, immutable result sets, continuation, integrity/privacy/security/evaluation.

Gate:

```text
no combined current/global FTS corpus
no raw cross-shard FTS score comparison
no inferred aliases or hidden candidate selection
no raw query syntax/executable extension/model/CBM lane
no approximate result promoted to lineage/replacement/negative authority
complete owner/document/index/lane coverage accounting
deterministic ranking/pages across workers/order/cache/storage layout
exact SQLite/FTS/tokenizer/platform probes and measured thresholds
all fixture/checksum pins frozen
```

### E4-B — lineage, migration records and static impact

Contract: [`../crates/wow-graph/e4/README.md`](../crates/wow-graph/e4/README.md)

Build exact before/after same-universe bindings, independent project/reference/search/review producer partitions, bounded ambiguity components, proof-ceiling validation, immutable lineage snapshot, typed changes/absence/replacement/migration records and bounded static-impact reason paths.

Gate:

```text
no cross-generation entity merge
no name/path/signature/fingerprint/rank/uniqueness promotion
no unrestricted all-pairs or forced split/merge bijection
no search candidate above Candidate
no review confidence above minimum proof ceiling
no Removed/Introduced without complete exact negative authority
no same-lineage -> replacement shortcut
no migration execution
no static path -> runtime/severity/performance/taint/combat/Secret/fixability
immutable publication and exact retained continuation
```

### E4-C — service and CLI orchestration

Contracts:

- [`../crates/wow-service/e4/README.md`](../crates/wow-service/e4/README.md)
- [`../apps/wow/e4/README.md`](../apps/wow/e4/README.md)

Build explicit shard/lineage status/build/validation, search/query/explain/continuation, exact current acquisition, explicit candidate selection, search-to-context handoff, review authorization plus independent graph validation, immutable lineage publication/query, advisory migration validation, bounded impact operations, idempotency/response-loss/retention/closure, and thin CLI transport.

Gate:

```text
no implicit shard build or newest/first/last selection
no top-1/sole/rank/name candidate selection
no rank/query text as context or lineage authority
no reviewer authorization from GitHub/OS/CLI/file identity
no in-place review mutation or migration apply
no static impact overclaim or path flattening
no current refresh/budget reset on continuation
no public success before reverse resource closure
apps depend on wow-service only
```

## E5-A — calibration corpora and named packs

Contract: [`../crates/wow-recognizers/e5/README.md`](../crates/wow-recognizers/e5/README.md)

Documentation now defines:

- exact candidate-source revision, tree/inventory/publication/fact closure;
- conservative upstream/fork/copy/vendor/generated/near-duplicate provenance groups;
- license/privacy/notice decisions per artifact class;
- immutable independent label and reviewer-evidence records;
- atomic Train/Dev/Test/SealedHoldout/Challenge/Quarantine splits;
- leakage detection, consumed-generation history, and holdout visibility rules;
- E2-B-compatible `calibration` + `shadow_only` pack schema;
- universal registered graph outputs at `Derived`/`Possible` only;
- repository/owner/addon/path/local-name/prose invariance;
- decisive literal/structure/resolution/coverage sensitivity and near-miss tests;
- independent graph validation and immutable per-case-first metrics;
- candidate artifact, supersession and partition-local deactivation;
- typed errors, security/resource ceilings, determinism, implementation plan, fixtures and pending checksum gate.

Current real-source state:

```text
8 exact user-repository commits pinned as candidate inputs
0 real sources admitted
0 real measured calibration runs
0 sealed holdout generations
0 promotion submissions
```

The eight pins cannot advance until exact tree/source inventories, project/analyzer/graph/fact publications, provenance grouping, license/privacy decisions, independent labels and split eligibility close.

E5-A gate before Rust:

```text
implemented/frozen E0/E2 prerequisite crates and owner seams
exact candidate source and publication identities
provenance/license/privacy/label/split/holdout profiles
pack/rule/literal/mutation/evaluation/graph-validation profiles
per-case expected outcomes and hard failure gates
ordinary/adversarial benchmarks and quantitative thresholds
1/2/N worker and shuffled-order determinism
candidate/deactivation canonical bytes
all member and bundle SHA-256 values
```

Hard stops:

```text
no commit-pin-only corpus admission
no repository source execution or raw-source matcher fallback
no repository/addon/owner/path/popularity/label/split/reviewer/model condition
no copied/forked/vendor/generated leakage across ordinary splits
no holdout access before exact candidate/run freeze
no Unknown/Possible/NotEvaluated/Conflict/Partial/Truncated -> Negative/pass
no donor-specific graph kind or new E5 operator language
no confidence above Derived/Possible
no default/core rollout or graph publication
no hard failure hidden by aggregate weighting
no generalization claim beyond admitted independent provenance groups
no core/foreign partition deletion during deactivation
```

## Next — E5-B calibration orchestration, review and promotion submissions

Owner: `wow-service`; thin application: `apps/wow`.

Define:

```text
exact corpus/pack/run/candidate selectors
retained artifact catalogs and fixed-order acquisition
owner compatibility validation and reverse-order closure
durable operation ID + canonical request digest
idempotency and response-loss recovery
reviewer authorization independent from metrics and graph validity
sealed-holdout unsealing authorization and access audit
promotion submission preparation without core publication
conservative status/result envelopes
strict CLI inputs, output modes, cancellation and exit codes
```

E5-B must invoke `wow-recognizers` E5-A operations rather than reimplement corpus admission, provenance grouping, split/leakage, labeling, matcher, mutation, metrics, graph validation, or deactivation. Missing reviewer authorization or holdout infrastructure remains blocked/NotEvaluated. E5-C remains the sole publication/canary/rollback owner.

## E5-C

Implement immutable core-pack publication, signatures when a signing profile exists, staged canary, rollout, exact last-known-good, rollback, stale partition closure and publication/read-back validation.

## E6–E7

- E6: optional Codebase Memory bridge; candidates remain external and degradable.
- E7: thin LSP/MCP and release/signing/publication operations.

## Roadmap discipline

- Later documentation cannot bypass earlier implementation gates.
- Architecture changes require an ADR and concrete failure of the accepted design.
- Stable contracts link the current external WoW engineering KB rather than copying patch-sensitive claims.
- Missing tools/probes/benchmarks/evaluations/authorization/client tests are skipped, blocked or NotEvaluated, never pass.
- Outcomes and proof gates matter; percentages and directory counts do not.
