# `wow-recognizers` E2-B core structural recognizer contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-recognizers/e2-b/core-structural-rules`

## Mission

Convert exact normalized source/project facts into explainable universal graph assertion proposals through a bounded, declarative, repository-independent rule engine.

```text
reviewed immutable core pack
+ exact normalized fact bundle
+ exact graph registry bundle
+ producer/generation/capability context
-> validate and compile declarative rules
-> match typed facts through bounded joins and predicates
-> retain ambiguity, missing capabilities, and truncation
-> emit proposed entity/relation assertions with derivation evidence
-> return producer-partition output for project/graph validation and publication
```

## Authority boundary

Recognizers prove only deterministic structural derivations from their input facts. They do not prove that a WoW API/event/template is valid, safe, current, unrestricted, or usable in combat. They never upgrade analyzer/source observations into platform authority.

## Direct dependencies

```text
wow-core
wow-emmy
wow-graph
```

- `wow-core`: identity, generation, source/evidence, confidence, coverage, budgets, errors.
- `wow-emmy`: normalized Lua semantic/syntax facts and read-only fact-set contracts.
- `wow-graph`: registry definitions and proposed assertion/result shapes, not persistence.

`wow-project` owns TOC/XML parsing, project generations, input-bundle assembly, invalidation, and graph publication orchestration. It may depend on recognizers and adapt its normalized facts into the recognizer input contract. No reverse dependency is allowed.

## E2-B active output

```text
universal role/entity proposals
universal relation proposals
match explanations and captured input facts
per-rule/per-partition coverage and NotEvaluated records
ambiguity/competing-match records
precision/mutation evaluation reports
deterministic producer partition manifest
```

Recognizers do not produce diagnostics, source edits, severity, autofixes, search ranking, or final graph IDs/generations.

## Core rules

The frozen E2-B pack covers:

- TOC package, load, dependency, optional dependency, LoadOnDemand, SavedVariables;
- XML template/frame/parent/inheritance/script ownership;
- `CreateFrame`, `CreateFromMixins`, and `Mixin` structure;
- native frame event registration;
- EventRegistry native frame-event bridge;
- custom registry callback only when an exact `TriggerEvent` producer is present;
- CVar callback registration;
- `SetScript`, `HookScript`, and `hooksecurefunc` structure;
- LibStub library requirement/creation/embed structure;
- SavedVariables root and literal state path reads/writes.

The pack intentionally does not infer generic module/service/lifecycle roles from arbitrary names. Named framework conventions are E5 calibration-pack data.

## Public operations

```text
parse_recognizer_pack
validate_recognizer_pack
compile_recognizer_pack
validate_recognizer_fact_bundle
match_recognizer_partition
explain_recognizer_match
build_recognizer_output_partition
build_recognizer_coverage_report
evaluate_recognizer_corpus
run_recognizer_mutation_suite
build_recognizer_precision_report
```

## Completion gate

E2-B code is complete only when every active rule operates exclusively on normalized typed facts, all outputs are explainable and generation-bound, repository/path/name mutations expose hidden overfitting, custom/native signals remain distinct, dynamic/ambiguous matches stay `Possible`, producer partition replacement removes stale outputs without changing graph semantics, missing capabilities become `NotEvaluated`, 1/2/N worker and shuffled-fact runs are byte-identical, and no source/parser/storage/editor/network/process/LLM path exists in the correctness core.
