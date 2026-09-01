# `wow-search` implementation contract

**Status:** deferred to E4; contract scaffold only.

## Mission

`wow-search` owns deterministic multi-lane lookup and ranking across exact reference facts, aliases/deprecations/replacements, historical lineage, structural shape, FTS text, and bounded graph neighborhoods. Optional semantic candidates are merged only as labeled candidate evidence by higher-layer orchestration.

## Owned responsibilities

- query intent/scope normalization;
- exact canonical and alias lookup orchestration;
- deprecation/replacement/lineage lanes;
- namespace/member/prefix and structural-shape lanes;
- FTS query construction and score normalization;
- bounded graph-neighborhood expansion;
- deterministic signal aggregation/reranking;
- negative-authority handling;
- search-hit explanations;
- result budgets, pagination, cancellation, and truncation;
- labeled search/evaluation corpus integration.

## Explicit non-responsibilities

`wow-search` does not:

- ingest source or build reference/project indexes;
- mutate graph/storage;
- call Codebase Memory directly;
- upgrade a semantic/fuzzy/name candidate to proven lineage;
- generate automatic code edits;
- decide diagnostic severity;
- return an authoritative miss without complete relevant coverage;
- use an LLM to rank correctness-path results;
- read full source bodies by default.

## Search stages

The default stage order is normative:

```text
0. normalize intent, universe, profile/generation, entity kinds, budget
1. exact active canonical name
2. exact aliases, deprecations, replacements, and build lineage
3. namespace/member/prefix
4. receiver/signature/return/restriction shape
5. bounded edit-distance/trigram candidates
6. FTS5 over docs/comments/role labels/L0-L1 skeletons
7. bounded graph-neighborhood expansion
8. optional externally supplied semantic candidates
9. deterministic reranking and explanation
```

A query may skip a lane only when its capability is unavailable or the intent excludes it. Skipped lanes appear in the result metadata.

## Ranking authority

Approximate authority order:

```text
explicit replacement/deprecation relation
exact canonical/alias match in active profile
proven/derived lineage evidence
entity kind and namespace
receiver/signature/return/restriction shape
package/load affinity
graph-neighborhood overlap
FTS documentation/skeleton score
name edit distance/trigram
external semantic score
```

Signals do not change their evidence class merely because they rank highly.

## Required operations

| Operation | Required behavior |
|---|---|
| `normalize_search_request` | Resolve explicit profile/universe/kind/scope/budget and reject floating or contradictory inputs. |
| `classify_query_intent` | Identify exact symbol, migration, structural, natural-language, or mixed intent deterministically where possible. |
| `run_exact_lane` | Query canonical names/aliases with complete evidence/coverage. |
| `run_migration_lane` | Query deprecations/replacements/lineage while preserving edge confidence. |
| `run_shape_lane` | Compare receiver/signature/return/restriction shape with explicit feature contributions. |
| `run_text_lane` | Execute bounded FTS queries and normalize text-only scores as candidate signals. |
| `run_graph_lane` | Expand only approved bounded relations from already selected seeds. |
| `merge_external_candidates` | Accept normalized candidate records from service/CBM without upgrading confidence. |
| `rerank_hits` | Apply deterministic documented signal weights/tie-breakers. |
| `explain_hit` | Return lanes/signals/evidence/coverage and why a hit outranked another. |
| `evaluate_search_miss` | Return authoritative miss, partial miss, profile unavailable, failed partition, conflict, or candidate-only. |
| `paginate_search_results` | Provide stable cursors/order under one generation and query digest. |

## Search hit contract

A hit includes:

```text
entity and canonical name/kind
active profile/reference generation
universe
owner/load chains or handles
source handle
migration status
provenance/confidence
coverage summary
lane/signal explanation
stable detail handle
generation-safe pagination identity
```

A hit does not include an entire file by default.

## Removed/missing symbol behavior

When an exact active symbol is absent:

1. verify profile and relevant complete coverage;
2. inspect explicit deprecation/replacement records;
3. inspect historical lineage;
4. compare receiver/signature/return/restriction shape;
5. inspect current source usage through handles when available;
6. return a replacement only when evidence supports it;
7. otherwise return labeled candidates plus the missing proof.

Never transform the top fuzzy hit into a migration fact.

## Determinism and calibration

- weights and tie-breakers are versioned;
- equal scores use stable entity/source keys;
- lane order is fixed;
- FTS tokenizer/configuration is part of search-version identity;
- external scores are normalized separately and cannot dominate authoritative exact evidence;
- calibration uses labeled normalized outcomes, not preferred prose answers;
- a ranking change requires before/after task metrics and false-proven analysis.

## E4 implementation sequence

1. request/result contracts and exact lane;
2. aliases/deprecations/replacements;
3. lineage lane;
4. shape lane;
5. FTS lane;
6. graph expansion;
7. explanations and stable pagination;
8. labeled task evaluation and weight calibration;
9. external-candidate merge seam, without CBM client activation.

## Required tests

- exact canonical and alias hit;
- explicit replacement outranks fuzzy candidate;
- same-name different-kind/profile separation;
- missing symbol under complete versus partial coverage;
- changed restriction/signature shape;
- lineage candidate versus proven relation;
- deterministic ties and pagination;
- FTS/name/semantic candidate cannot claim proven replacement;
- graph budget/truncation;
- stale external generation labeling;
- query injection/oversized request rejection;
- top-1/top-3 recall report and false-proven count;
- transport-independent normalized output.

## Documentation sources

- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [`../../docs/CODEBASE_MEMORY_BRIDGE.md`](../../docs/CODEBASE_MEMORY_BRIDGE.md)
- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)

## Definition of done

E4 search is complete when explicit current/historical evidence dominates similarity, every hit explains its lanes and evidence, incomplete profiles cannot produce authoritative misses, and the labeled WoW task corpus reaches the roadmap recall target without false proven replacements.
