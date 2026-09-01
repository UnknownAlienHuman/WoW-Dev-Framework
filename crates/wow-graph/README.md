# `wow-graph` contract router

**Status:** E2-A typed graph and E4-B cross-generation lineage/migration/static-impact contracts are implementation-ready documentation; E4-C public orchestration is defined in `wow-service`. Rust implementation has not started.

`wow-graph` owns versioned graph schemas, semantic entity/relation assertions, producer partitions, immutable graph snapshots, bounded graph queries, and E4-B cross-generation lineage/change/impact overlays. It does not parse source, run recognizers/search, infer platform facts, rank candidates, resolve current publications, authorize reviewers, or expose raw storage internals.

## E2-A — generation-local typed graph

Read [`e2/README.md`](e2/README.md) and its complete route.

E2-A defines:

- entity/relation/attribute/axis registries;
- generation-scoped semantic keys and assertions;
- producer partition replacement;
- conflicts, coverage and provenance;
- immutable graph publication;
- bounded exact entity/neighbor/axis/path/subgraph/explanation queries;
- logical `wow-store` persistence boundary.

## E4-B — cross-generation lineage, migration and static impact

Read [`e4/README.md`](e4/README.md) and its complete route.

E4-B adds a separate immutable comparison overlay:

```text
exact before/after owner generations
+ explicit owner/reference transition evidence
+ bounded project fingerprints/change facts
+ Candidate-only search retrieval signals
+ authorized review decisions under graph proof ceilings
-> producer-separated lineage proposals
-> proof-ceiling and ambiguity validation
-> immutable LineageGraphSnapshot
-> change classification, migration candidates and bounded static impact paths
```

Generation-local E2 graph identity remains unchanged. Cross-generation relationships are explicit assertions in a separate `LineageGraph`; they never merge or rewrite source entity keys.

## Direct dependency boundary

```text
wow-core
wow-store
```

`wow-project`, `wow-reference`, and `wow-search` produce typed E4-B inputs through their own public contracts. [`wow-service/e4`](../wow-service/e4/README.md) coordinates exact acquisition, producer execution, review authorization, lineage publication/query, migration validation and static-impact use cases. `wow-graph` never depends directly on those higher/domain producers or service.

## E4-C orchestration boundary

```text
exact before/after selectors and producer profiles
-> wow-service acquires exact project/reference/search views
-> ReviewAuthorizationPort validates review authorization when present
-> wow-graph independently validates proposal/review semantics and proof ceilings
-> immutable LineageGraphSnapshot and exact query results
-> wow-service canonical envelope
-> apps/wow thin CLI
```

Review authorization does not create lineage proof. Graph validity does not authorize a reviewer. Applying review produces a new immutable snapshot. Search rank cannot be auto-selected as lineage truth. Migration remains advisory and static impact remains static reason-path evidence.

## Authority boundary

- Stable owner identity or explicit transition evidence can support a higher lineage proof ceiling under the exact E4-B profile.
- Structural fingerprint, same name/path, text/fuzzy/shape/graph proximity, and E4-A rank remain Candidate evidence.
- Replacement/migration is not the same relation as identity continuity.
- Missing target entities are not `Removed` without complete relevant before/after coverage.
- Impact is a bounded reason-path result, not proof of runtime breakage or severity.
- One-to-many split and many-to-one merge remain explicit; no forced bijection or greedy first match.

## Implementation state

```text
documentation frontier: E4-B owner contract; E4-C service boundary linked
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

No E4-B Rust work starts before the E2 graph implementation, E4-A search implementation, exact comparison inputs, candidate/proof/review profiles, paired corpora, and all fixture/checksum gates are frozen.
