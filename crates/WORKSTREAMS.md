# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E4-C.

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
-> E2-B wow-recognizers
-> E2-C full wow-project candidate
-> E2-D ProjectStore coherent publication

-> E3-A wow-project Blizzard UI source universe
-> E3-B wow-context Project Map/L0/L1/context pack
-> E3-C wow-service/apps context acquisition/use cases

-> E4-A wow-search exact-generation retrieval core
-> E4-B wow-graph lineage/migration/static impact
-> E4-C wow-service/apps search/lineage/impact use cases

-> E5-A recognizer calibration corpora and named calibration packs
-> E5-B calibration orchestration/review/promotion submissions
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
- Applications import `wow-service` only.
- No CI/workflow without explicit owner instruction.

## E0 — diagnostic vertical slice

- `wow-core`: identities, evidence, coverage, conflicts, result primitives.
- fixture `wow-reference`: one exact profile and restricted lookup.
- `wow-emmy`: pinned analyzer adapter and normalized facts/diagnostics.
- minimal `wow-project`: one coherent workspace/generation.
- `wow-rules`: `wow.api.exists` plus one Secret-local rule.
- `wow-service` + `apps/wow`: `status`, `check`, exact context, canonical result/exit mapping.

## E1 — Reference Pack

- generic `wow-store` foundation and immutable ReferenceStore;
- persistent `wow-reference` ingestion/corrections/coverage/negative authority;
- deterministic `wow-annotations` with source maps/loss/parity/consumer probes;
- service/builder app build, nonrepairing validation, rebuild comparison.

## E2 — project graph and persistence

- assertion-based typed `wow-graph` with partitions/snapshots/bounded queries;
- declarative universal `wow-recognizers` over normalized facts;
- exact TOC/XML/load/analyzer project candidate and incremental invalidation;
- one SQLite WAL ProjectStore epoch with immutable partition versions, complete membership, inactive validation, and current CAS.

## E3 — Blizzard UI source and exact context

### E3-A — `wow-project`

Exact separately published `blizzard_ui_source` universe, package/TOC/XML/Lua/analyzer/recognizer/graph pipeline, source/license/coverage/fingerprint records, incremental removal closure, and bounded `SkeletonInputView`.

### E3-B — `wow-context`

Exact immutable user/platform/reference universe binding; separate/combined maps; L0/L1; deterministic expansion/selection/pruning; source/privacy/license boundaries; semantic packs; JSON/Markdown; continuation/cache identities; metrics/evaluation.

Exact roots only. No hidden search/model/parser/store internals.

### E3-C — `wow-service` and `apps/wow`

Exact/current selector resolution, retained owner view acquisition, context use-case orchestration, continuation retention, result envelopes, cancellation/closure, and thin context CLI.

## E4-A — `wow-search` exact-generation retrieval core

Primary contract:

- [`wow-search/e4/`](wow-search/e4/README.md)

```text
exact owner generation
-> immutable generation-local SearchShard

exact SearchUniverseSet + structured query
-> exact/name/alias/member/prefix/text/similarity/shape/graph lanes
-> evidence-bearing candidate signals
-> authority-banded deterministic ranking
-> complete explanations and honest miss
-> immutable result-set manifest
-> stable continuation
```

Hard stops:

- no symbolic current inside `wow-search`;
- no combined mutable global FTS corpus;
- no raw cross-shard BM25 comparison;
- no raw SQL/FTS/regex/callback/expression;
- no model/embedding/CBM lane;
- no lineage/replacement/migration/impact conclusion;
- no approximate empty result as authoritative absence;
- no hidden lane failure/coverage/conflict/truncation;
- no search-to-context dependency.

## E4-B — lineage, migration, and static impact (`wow-graph`)

Primary contract:

- [`wow-graph/e4/`](wow-graph/e4/README.md)

Producer seams:

- [`wow-project/E4_B_LINEAGE_INPUTS.md`](wow-project/E4_B_LINEAGE_INPUTS.md)
- [`wow-reference/E4_B_TRANSITION_EVIDENCE.md`](wow-reference/E4_B_TRANSITION_EVIDENCE.md)
- [`wow-search/e4/LINEAGE_CANDIDATE_HANDOFF.md`](wow-search/e4/LINEAGE_CANDIDATE_HANDOFF.md)

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

Hard stops:

- no cross-generation entity ID merge;
- no unique/top/same-name/path/fingerprint candidate promotion;
- no forced bijection for split/merge/copy ambiguity;
- no Removed/Introduced without exact complete negative-authority coverage;
- no same-lineage-to-replacement shortcut;
- no migration recipe execution;
- no static path to runtime/severity/safety claim;
- no raw source/search/store/service dependency inside `wow-graph`.

## E4-C — service and CLI orchestration

Primary contracts:

- [`wow-service/e4/`](wow-service/e4/README.md)
- [`../apps/wow/e4/`](../apps/wow/e4/README.md)

Public operations:

```text
search_index_status/build/validate
search_query/continue/explain/select/context
lineage_status/build/validate/review_validate/review_apply/compare/trace/explain
migration_candidates/validate
impact_plan/run/continue/explain
```

Architecture:

```text
symbolic/exact selectors
-> resolve current once at service boundary
-> acquire exact project/reference/search/lineage/context views in fixed order
-> validate compatibility and retention
-> invoke one owner operation plan
-> preserve owner candidates/proof/coverage/conflicts/omissions
-> require explicit candidate selection receipt
-> optionally pass exact selected entity root to E3-C context
-> release resources in reverse order
-> canonical service envelope
-> thin CLI output/exit mapping
```

### E4-C ownership

`wow-service` owns selector/profile normalization, exact acquisition, orchestration, explicit selection receipts, review authorization adapter invocation, idempotency/response-loss recovery, continuation retention, conservative status and resource closure.

`apps/wow` owns strict transport parsing, one service call, signals, output and exit codes.

### E4-C hard stops

```text
no implicit search shard build during query
no newest/first/last catalog choice
no automatic top-1 or sole-candidate selection
no search score as entity/context/lineage authority
no reviewer authorization from GitHub/OS/CLI/file identity
no review confidence above proof ceiling
no in-place lineage review mutation
no migration apply or source edit
no static impact -> runtime breakage/severity/performance/taint/combat/Secret/fixability
no path -> direct edge
no continuation current refresh or cumulative-budget reset
no public success before mandatory close
no raw SQL/FTS/source/store/model/CBM/tool access
apps import wow-service only
```

### E4-C implementation gate

```text
implemented/frozen E0–E4-B prerequisites
exact owner/shard/lineage/context/catalog/retention/idempotency ports
exact selector/acquisition/search/lineage/review/migration/impact/context profiles
review authorization adapter and synthetic public verification corpus
roth-ui, Blizzard UI, Reference transition, ambiguity/copy/split/merge corpora
search selection, review, response-loss, continuation, privacy and impact vectors
canonical service/CLI JSON, text, artifact and exit bytes
1/2/N worker and shuffled owner scheduling determinism
all member checksums populated
```

## Next — E5-A recognizer calibration corpora and packs

E5-A should define named calibration inputs without introducing named production semantics.

Required shape:

```text
pinned exact addon/framework repositories and commits
+ exact source/project/analyzer/graph publications
+ hand-reviewed expected universal structural facts
+ positive, clean-negative, near-miss and adversarial cases
+ repository/addon/owner/path/local-identifier rename mutations
-> calibration corpus manifests
-> calibration-run inputs and metrics
-> candidate named calibration packs
-> universal recognizer proposal outputs only
```

Hard boundary:

- a named pack may select which reviewed pattern family to evaluate;
- output kinds/relations/roles remain universal and graph-validatable;
- production behavior cannot branch on repository/addon/owner/path/popularity names;
- deleting a named pack removes only its producer partitions/coverage;
- no proof/authority upgrade, source execution, model inference, runtime claim, or CI.

E5-B later owns durable calibration orchestration/review/promotion submissions. E5-C owns immutable core-pack publication/canary/rollback.

## E6–E7

- E6: optional Codebase Memory bridge; external results remain Candidate and degradable.
- E7: thin LSP/MCP transports and release/signing/publication/rollback operations.

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
