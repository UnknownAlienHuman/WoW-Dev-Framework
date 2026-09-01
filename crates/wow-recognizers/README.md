# `wow-recognizers` contract router

**Status:** E2-B core structural recognizers and E5-A calibration-corpus/named-pack contracts are implementation-ready documentation. Rust implementation has not started.

`wow-recognizers` deterministically matches reviewed structural conventions over normalized facts and emits proposed universal graph assertions. It never reparses source, branches on repository/addon identity, executes code, decides platform truth, runs diagnostics, publishes graph generations, or authorizes pack promotion.

## Canonical routes

### E2-B — core structural recognizers

Read in order:

1. [`e2/README.md`](e2/README.md)
2. [`e2/AGENTS.md`](e2/AGENTS.md)
3. [`e2/DECISIONS.md`](e2/DECISIONS.md)
4. [`e2/DATA_MODEL.md`](e2/DATA_MODEL.md)
5. [`e2/FACT_INPUT_MODEL.md`](e2/FACT_INPUT_MODEL.md)
6. [`e2/PACK_SCHEMA.md`](e2/PACK_SCHEMA.md)
7. [`e2/MATCH_ENGINE.md`](e2/MATCH_ENGINE.md)
8. [`e2/RULE_FAMILIES.md`](e2/RULE_FAMILIES.md)
9. [`e2/OUTPUT_AND_GRAPH_HANDOFF.md`](e2/OUTPUT_AND_GRAPH_HANDOFF.md)
10. [`e2/CONFIDENCE_AMBIGUITY_AND_COVERAGE.md`](e2/CONFIDENCE_AMBIGUITY_AND_COVERAGE.md)
11. [`e2/PARTITIONS_AND_VERSIONING.md`](e2/PARTITIONS_AND_VERSIONING.md)
12. [`e2/MUTATION_AND_EVALUATION.md`](e2/MUTATION_AND_EVALUATION.md)
13. [`e2/SECURITY_AND_BUDGETS.md`](e2/SECURITY_AND_BUDGETS.md)
14. [`e2/ERROR_MODEL.md`](e2/ERROR_MODEL.md)
15. [`e2/TEST_MATRIX.md`](e2/TEST_MATRIX.md)
16. [`e2/IMPLEMENTATION_PLAN.md`](e2/IMPLEMENTATION_PLAN.md)
17. [`e2/CONTRACT.json`](e2/CONTRACT.json) and [`e2/examples/`](e2/examples/README.md)

E2-B owns the bounded declarative operator language, typed fact inputs, deterministic matcher, universal proposal output, producer partitions, and core-rule evaluation.

### E5-A — calibration corpora and named packs

Read in order:

1. [`e5/README.md`](e5/README.md)
2. [`e5/AGENTS.md`](e5/AGENTS.md)
3. [`e5/DECISIONS.md`](e5/DECISIONS.md)
4. [`e5/DATA_MODEL.md`](e5/DATA_MODEL.md)
5. [`e5/CORPUS_ADMISSION_AND_PROVENANCE.md`](e5/CORPUS_ADMISSION_AND_PROVENANCE.md)
6. [`e5/CORPUS_SPLITS_AND_LEAKAGE.md`](e5/CORPUS_SPLITS_AND_LEAKAGE.md)
7. [`e5/LABELING_AND_REVIEW.md`](e5/LABELING_AND_REVIEW.md)
8. [`e5/CALIBRATION_PACK_SCHEMA.md`](e5/CALIBRATION_PACK_SCHEMA.md)
9. [`e5/OPERATIONS.md`](e5/OPERATIONS.md)
10. [`e5/MUTATION_AND_ANTI_OVERFITTING.md`](e5/MUTATION_AND_ANTI_OVERFITTING.md)
11. [`e5/EVALUATION_AND_GATES.md`](e5/EVALUATION_AND_GATES.md)
12. [`e5/PARTITIONS_AND_DEACTIVATION.md`](e5/PARTITIONS_AND_DEACTIVATION.md)
13. [`e5/SECURITY_AND_BUDGETS.md`](e5/SECURITY_AND_BUDGETS.md)
14. [`e5/ERROR_MODEL.md`](e5/ERROR_MODEL.md)
15. [`e5/TEST_MATRIX.md`](e5/TEST_MATRIX.md)
16. [`e5/IMPLEMENTATION_PLAN.md`](e5/IMPLEMENTATION_PLAN.md)
17. [`e5/CONTRACT.json`](e5/CONTRACT.json) and [`e5/examples/`](e5/examples/README.md)

E5-A defines exact candidate-source admission, immutable corpora/labels/provenance/splits, leakage and sealed-holdout semantics, shadow-only calibration packs, independent graph validation, anti-overfitting mutations, per-case-first metrics, candidate artifacts, and partition-local deactivation.

The eight pinned user repositories are candidate inputs only. A commit pin is not an admitted corpus member. Exact source/publication/fact/provenance/license/label/split gates remain blocking until implementation.

## Direct framework dependencies

```text
wow-core
wow-emmy
wow-graph
```

`wow-project` supplies exact TOC/XML/project fact publications through orchestration, but `wow-recognizers` does not depend on `wow-project`. `wow-store`, `wow-service`, and applications own retention/orchestration/transport outside this crate.

## Active semantic boundary

```text
exact normalized fact partitions
+ bounded E2-B declarative pack
+ exact profiles and graph registry
-> deterministic matches and ambiguity
-> universal entity/relation proposals
-> graph validation receipts
-> exact producer-owned output partitions
```

E5-A adds audit/evaluation artifacts around this path. Repository, owner, addon, path, popularity, split, expected label, reviewer, search, model, and prompt metadata cannot enter matcher clauses, captures, semantic keys, confidence, coverage, ordering, or budgets.

## Confidence and rollout

Recognizer proposal confidence is limited to:

```text
Derived
Possible
```

E5-A packs use:

```text
trust_class = calibration
rollout_state = shadow_only
```

`ShadowValidated` and `PromotionEligibleByMetrics` are evaluation states, not default activation. E5-B owns durable orchestration, reviewer authorization, holdout unsealing audit, and promotion submissions. E5-C owns immutable core-pack publication, canary, rollout, rollback, and last-known-good.

## Current state

```text
documentation frontier: E5-A
implementation frontier: not-started
E2-B implementation/checksum freeze: pending
E5-A real corpus admission: pending
sealed holdout/reviewer authorization: deferred to E5-B
core-pack publication/rollback: deferred to E5-C
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Also read [`../AGENTS.md`](../AGENTS.md), [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md), [`../WORKSTREAMS.md`](../WORKSTREAMS.md), the E2-A [`wow-graph` contract](../wow-graph/e2/README.md), the normalized [`wow-emmy` fact model](../wow-emmy/FACT_MODEL.md), and the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes.
