# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E5-A.

Documentation-ready remains `implementation_state = not-started` until executable code, probes, fixtures, checksums, corpora, benchmarks, authorization adapters and required evaluations exist.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture
-> E0-E wow-rules
-> E0-F wow-service + apps/wow diagnostics

-> E1-A wow-store foundation
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validate

-> E2-A wow-graph
-> E2-B wow-recognizers core matcher
-> E2-C full wow-project candidate
-> E2-D ProjectStore coherent publication

-> E3-A wow-project Blizzard UI source universe
-> E3-B wow-context Project Map/L0/L1/context pack
-> E3-C wow-service/apps context acquisition/use cases

-> E4-A wow-search exact-generation retrieval core
-> E4-B wow-graph lineage/migration/static impact
-> E4-C wow-service/apps search/lineage/impact use cases

-> E5-A wow-recognizers calibration corpora, shadow packs, evaluation and candidate artifacts
-> E5-B wow-service/apps calibration orchestration, review, holdout audit and promotion submissions
-> E5-C immutable core-pack publication/canary/rollback

-> E6 optional Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

## Global rules

- One agent owns one primary work package/crate.
- Shared seams are proposed before dependent implementation.
- A separate review pass validates every wave.
- Missing tools, implementations, probes, benchmarks, tokenizers, evaluations, reviewer authorization, runtime tests, or source producers are blocking/`NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact pinned source/runtime evidence.
- Repository/addon/path/provider/popularity/model names never become hidden production semantics.
- Search candidate ranking never becomes intended-entity, lineage, replacement, migration, impact, safety, or platform authority.
- Review authorization and lineage proof are independent.
- Static impact remains reason-path evidence, not runtime breakage or severity.
- A repository commit is not an admitted calibration corpus member.
- Calibration labels, split assignments, reviewer notes and expected outputs never become matcher inputs.
- Applications import `wow-service` only.
- No CI/workflow without explicit owner instruction.

## E0–E2 foundation

### E0 — diagnostic vertical slice

- `wow-core`: identities, evidence, coverage, conflicts, result primitives.
- fixture `wow-reference`: one exact profile and restricted lookup.
- `wow-emmy`: pinned analyzer adapter and normalized facts/diagnostics.
- minimal `wow-project`: one coherent workspace/generation.
- `wow-rules`: `wow.api.exists` plus one Secret-local rule.
- `wow-service` + `apps/wow`: `status`, `check`, exact context, canonical result/exit mapping.

### E1 — Reference Pack

- generic `wow-store` foundation and immutable ReferenceStore;
- persistent `wow-reference` ingestion/corrections/coverage/negative authority;
- deterministic `wow-annotations` with source maps/loss/parity/consumer probes;
- service/builder app build, nonrepairing validation, rebuild comparison.

### E2 — project graph and persistence

- assertion-based typed `wow-graph` with partitions/snapshots/bounded queries;
- declarative universal `wow-recognizers` over normalized facts;
- exact TOC/XML/load/analyzer project candidate and incremental invalidation;
- one SQLite WAL ProjectStore epoch with immutable partition versions, complete membership, inactive validation, and current CAS.

## E3 — Blizzard UI source and exact context

### E3-A — `wow-project`

Exact separately published `blizzard_ui_source` universe, package/TOC/XML/Lua/analyzer/recognizer/graph pipeline, source/license/coverage/fingerprint records, incremental removal closure, and bounded `SkeletonInputView`.

### E3-B — `wow-context`

Exact immutable user/platform/reference universe binding; separate/combined maps; L0/L1; deterministic expansion/selection/pruning; source/privacy/license boundaries; semantic packs; JSON/Markdown; continuation/cache identities; metrics/evaluation.

### E3-C — `wow-service` and `apps/wow`

Exact/current selector resolution, retained owner view acquisition, context use-case orchestration, continuation retention, result envelopes, cancellation/closure, and thin context CLI.

## E4 — search, lineage, migration and static impact

### E4-A — `wow-search`

```text
exact owner generation
-> immutable generation-local SearchShard
-> exact/name/alias/member/prefix/text/similarity/shape/graph lanes
-> evidence-bearing candidates
-> authority-banded deterministic ranking
-> complete explanations and honest miss
-> immutable result-set and stable continuation
```

Hard stops: no symbolic current inside search, no raw cross-shard BM25, no raw SQL/FTS/regex/expression, no model/CBM lane, no automatic candidate selection, no lineage/replacement/migration/impact conclusion, and no approximate empty result as authoritative absence.

### E4-B — `wow-graph`

```text
exact before/after same-universe generations
+ project stable identity/fingerprint/change partitions
+ Reference transition/deprecation/replacement partitions
+ Candidate-only search partitions
+ authorized reviewed decisions
-> bounded ambiguity components and proposals
-> proof-ceiling validation
-> immutable LineageGraphSnapshot
-> typed changes/absence/replacement/migration candidates
-> bounded static-impact reason paths
```

Hard stops: no cross-generation entity merge, no top/same-name/path/fingerprint promotion, no forced split/merge bijection, no Removed/Introduced without complete authority, no migration execution, and no static path to runtime/severity/safety claim.

### E4-C — `wow-service` and `apps/wow`

```text
symbolic/exact selectors
-> service resolves current once
-> exact project/reference/search/lineage/context acquisition
-> one owner operation plan
-> explicit search candidate selection receipt
-> independent review authorization and graph validation
-> reverse-order closure
-> canonical result envelope
-> thin CLI output/exit mapping
```

No implicit shard build, newest/first/last catalog choice, top-1 selection, review confidence above proof ceiling, in-place lineage mutation, migration apply, cumulative-budget reset, or success before mandatory close.

## E5-A — calibration corpora and named shadow packs (`wow-recognizers`)

Primary contract:

- [`wow-recognizers/e5/`](wow-recognizers/e5/README.md)

```text
exact candidate repository revisions
+ exact materialized source/project/analyzer/graph/fact publications
+ provenance/fork/copy/vendor/generated/near-duplicate group closure
+ license/privacy/notice decisions per artifact class
+ independent universal expected labels
+ atomic Train/Dev/Test/SealedHoldout/Challenge/Quarantine split
+ E2-B declarative calibration pack
+ invariance/sensitivity/near-miss/security/determinism mutations
-> shadow-only candidate-owned output partitions
-> independent graph proposal validation
-> immutable per-case results and explicit metric denominators
-> anti-overfitting and generalization report
-> immutable candidate and deactivation artifacts
```

### E5-A ownership

`wow-recognizers` owns pure validation and evaluation of exact immutable artifacts:

```text
validate candidate source/corpus/labels/splits/pack
run E2-B shadow matching
run mutation suite
evaluate per-case/per-rule/per-role/per-split/per-provenance results
build candidate/deactivation artifacts
```

Repository materialization, project publication and durable storage are supplied by existing owner seams. E5-A does not import `wow-project` or `wow-store` directly.

### E5-A hard stops

```text
no commit-pin-only corpus admission
no raw source/parser fallback or repository execution
no named repository/addon/owner/path/popularity semantic conditions
no label/split/reviewer/expected-output matcher input
no copied/forked/vendor/generated group leakage across ordinary splits
no holdout access before exact candidate/run freeze
no Unknown/Possible/NotEvaluated/Conflict/Partial/Truncated -> Negative/pass coercion
no new operator language or donor-specific graph kind
no confidence above Derived/Possible
no default/core rollout or graph publication
no hard failure hidden by aggregate weighting
no generalization claim beyond admitted independent provenance groups
no deactivation of core/foreign partitions
no CI/workflow
```

### E5-A current evidence state

Eight user-repository commits are exact candidate pins only. Tree/source inventories, project/analyzer/graph/fact publications, provenance grouping, license/privacy decisions, independent labels, split eligibility, implementations, thresholds, benchmarks, holdout infrastructure, reviewer authorization, and checksums remain blocking.

Closed synthetic fixtures exercise the contract without claiming real donor admission, measured performance, promotion eligibility, runtime behavior, or WoW API authority.

### E5-A implementation gate

```text
implemented/frozen core/emmy/graph/recognizer/project/store prerequisites
exact source tree/inventory/publication/fact identities
provenance/license/privacy/notice closure
label-review and split/holdout profiles
pack/rule/literal/mutation/evaluation/graph-validation profiles
per-case expected results and hard gates
ordinary/adversarial resource thresholds
1/2/N worker and shuffled-order determinism
candidate/deactivation artifacts
all canonical bytes and member/bundle SHA-256 values
```

## Next — E5-B calibration orchestration and review

Owner: `wow-service`; application: `apps/wow`.

Required scope:

```text
exact corpus/pack/run/candidate selectors
retained immutable artifact acquisition
fixed-order owner operation plan and reverse-order closure
durable operation ID + request digest
idempotency and response-loss recovery
reviewer authorization independent from metric/graph validity
sealed-holdout unsealing and access audit
promotion submission preparation without publication
canonical service result envelopes
thin CLI transport/output/exit behavior
```

E5-B must call E5-A owner operations and must not reproduce corpus, split, matcher, mutation, metric, graph-validation, or deactivation algorithms. E5-C remains the only core-pack publication/canary/rollback owner.

## E6–E7

- E6: optional Codebase Memory bridge; external results remain Candidate and degradable.
- E7: thin LSP/MCP transports and release/signing/publication operations.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
rejected workaround
why existing read view/artifact/orchestration is insufficient
smallest proposed seam
cycle/security/evidence/privacy/license impact
fixture/mutation proving it
implementation/freeze impact
```

Do not implement a missing seam in the wrong crate.
