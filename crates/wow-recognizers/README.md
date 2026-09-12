# `wow-recognizers` contract router

**Status:** the bounded E2-B matcher core is executable in Cargo. Typed owner adapters, declarative pack parsing, active structural rule families, producer partitions, mutation evaluation, and the E5-A calibration system remain incomplete.

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

The pinned user repositories are candidate inputs only. A commit pin is not an admitted corpus member. Exact source/publication/fact/provenance/license/label/split gates remain blocking until implementation.

## Executable E2-B core

The active crate currently provides:

- a closed registry for the nineteen documented observation families;
- content-addressed registry, observation, assertion, and report identities;
- exact binding to one immutable `GraphSnapshot` and existing graph endpoints;
- deterministic observation ordering and duplicate rejection;
- confidence ceilings that prevent an observation from being strengthened;
- explicit Complete, Partial, Truncated, NotEvaluated, and Failed family coverage;
- bounded assertion truncation with an explicit family blocker;
- cancellation and input/output budget checks;
- conversion of assertions to graph-owned edges;
- conversion of recognizer coverage to graph coverage without negative authority.

This is a low-level structured-observation matcher and graph handoff. It is not yet the full E2-B package described by the normative documents. In particular, callers must not treat a manually constructed `StructuredObservation` as independently established semantic evidence.

## Direct framework dependencies

```text
wow-core
wow-graph
```

The future typed owner-adapter slice also depends on frozen `wow-emmy` fact schemas. `wow-project` will supply exact TOC/XML/project fact publications through orchestration, but `wow-recognizers` must not depend on `wow-project`. `wow-store`, `wow-service`, and applications own retention, orchestration, and transport outside this crate.

## Active semantic boundary

```text
exact structured owner observations
+ closed recognizer registry
+ exact graph snapshot and endpoints
-> deterministic bounded assertions
-> graph-owned edge and coverage records
```

Future E2-B phases add typed fact adapters, declarative packs, active core families, producer partitions, and evaluation. Repository, owner, addon, path popularity, labels, splits, reviews, holdouts, canaries, providers, and model metadata cannot enter matcher semantics or confidence.

## Confidence and authority

Recognizer output cannot exceed the observation and descriptor confidence ceilings. External provider observations are always reduced to `Candidate`. Graph coverage projected by this crate always has `negative_authority = false`, including Complete recognizer coverage.

No recognizer result proves WoW API existence or absence, runtime availability, Secret state, operation safety, or graph publication success.

## Current state

```text
Cargo.toml: active
Rust source: active
bounded registry/matcher/graph handoff: implemented and tested
typed wow-emmy/project fact adapters: pending
declarative pack parser and operator DAG: pending
active structural rule families: pending
producer partition/replacement model: pending
mutation/evaluation harness and checksum freeze: pending
E5-A real corpus admission and calibration: not started
```

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.
