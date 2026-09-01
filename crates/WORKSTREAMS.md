# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E4-A.

Documentation-ready remains `implementation_state = not-started` until executable code, probes, fixtures, checksums, and required evaluations exist.

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
-> E4-B explicit lineage/migration/impact
-> E4-C search/lineage/impact service and CLI

-> E5 calibration packs
-> E6 optional Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

## Global rules

- One agent owns one primary work package/crate.
- Shared seams are proposed before dependent implementation.
- A separate review pass validates every wave.
- Missing tools, implementations, probes, benchmarks, tokenizers, evaluations, runtime tests, or source producers are blocking/`NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact pinned source/runtime evidence.
- Repository/addon/path/provider/popularity names never become hidden production semantics.
- Search candidate ranking never becomes lineage, replacement, intent, safety, or platform authority.
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

Active direct dependencies:

```text
wow-search
├── wow-core
├── wow-store
├── wow-reference
├── wow-project
└── wow-graph
```

Public owner operations:

```text
validate_search_profiles
build_search_document_partition
plan_search_shard_build
build_search_shard
validate_search_shard
open_search_shard_view
bind_search_universe_set
validate_search_request
normalize_search_request
plan_search_query
run_exact_identity_lane
run_exact_name_lane
run_exact_alias_lane
run_member_prefix_lane
run_text_lane
run_identifier_similarity_lane
run_shape_lane
run_graph_lane
fuse_and_rank_search_candidates
explain_search_candidate
evaluate_search_miss
materialize_search_result_set
paginate_search_results
continue_search_results
validate_search_result
```

### E4-A architecture

```text
exact user-project owner generation
-> immutable project SearchShard

exact Blizzard UI source generation
-> immutable platform-source SearchShard

exact ReferenceView generation
-> immutable reference SearchShard

exact SearchUniverseSet + structured query
-> exact/alias/member/prefix/text/similarity/shape/graph lanes
-> candidate signals with origins and coverage
-> authority-banded deterministic ranking
-> complete explanations
-> immutable result-set manifest
-> stable pages/continuation
```

### E4-A hard stops

```text
no symbolic current/latest inside wow-search
no combined mutable global FTS corpus
no raw cross-shard BM25/FTS comparison
no case-folded exact identifier match
no inferred alias
no raw FTS/SQL/regex/callback/expression/tokenizer input
no full source body or context artifact indexing
no search -> context dependency
no model/embedding/CBM lane
no lineage/replacement/migration/impact conclusion
no approximate empty result as authoritative absence
no hidden lane failure, coverage loss, conflict, truncation, or omission
no result page before immutable result-set ordering exists
no continuation generation refresh or budget reset
no raw SQLite/storage handle or loadable extension
```

### E4-A implementation gate

```text
implemented/frozen E0–E3 prerequisites
exact owner read catalogs
exact SQLite/Rust binding/FTS5/tokenizer/platform probes
document/field/normalization/query/lane/ranking/miss/privacy/budget profiles
synthetic, roth-ui, Blizzard UI, Reference, collision, adversarial and update corpora
zero false exact/alias/authoritative-miss/lineage claims
deterministic 1/2/N and shuffled-order results
accepted measured build/query/resource thresholds
all fixture and member checksums populated
```

## Next — E4-B lineage, migration, and impact

E4-B must define independently validated cross-generation assertions/candidates for:

```text
same entity
renamed
moved
replaced/superseded
removed
introduced
signature/restriction/behavioral contract change
migration compatibility
bounded static impact
```

E4-A may supply `LineageCandidateInput` with exact entities, retrieval signals, shape differences, graph paths, and coverage. Rank is Candidate evidence only.

Authoritative E4-B conclusions require exact before/after generations, source/reference/project/graph evidence, conflict and coverage closure, proof ceilings, and reviewed resolution. No top fuzzy result becomes a replacement.

## E4-C and later

- E4-C: `wow-service` and `apps/wow` search/lineage/impact operations plus explicit candidate-to-context root handoff.
- E5: audited named calibration packs over universal facts with rename/path/name mutations.
- E6: optional Codebase Memory bridge; external results remain Candidate.
- E7: thin LSP/MCP and release/signing/publication/rollback operations.

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
