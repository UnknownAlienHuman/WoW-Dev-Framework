# `wow-recognizers` contract router

**Status:** E2-B core structural recognizer contract is implementation-ready; Rust implementation has not started. Named calibration packs remain deferred to E5.

`wow-recognizers` deterministically matches reviewed structural conventions over normalized fact bundles and emits proposed universal graph assertions. It never reparses source, branches on repository or addon identity, executes code, decides platform truth, runs diagnostics, or publishes graph generations.

## Canonical E2-B route

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

Also read [`../AGENTS.md`](../AGENTS.md), [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md), [`../WORKSTREAMS.md`](../WORKSTREAMS.md), the E2-A [`wow-graph` contract](../wow-graph/e2/README.md), the normalized [`wow-emmy` fact model](../wow-emmy/FACT_MODEL.md), and the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes.

## Direct framework dependencies

```text
wow-core
wow-emmy
wow-graph
```

`wow-project` supplies TOC/XML/project fact bundles and invokes recognizers, but `wow-recognizers` does not depend on `wow-project`. The recognizer crate returns proposed graph assertion batches; publication remains owned by project/graph orchestration.

## E2-B active families

```text
TOC package/load/dependency/LoadOnDemand/SavedVariables
XML template/frame/parent/inherits/script ownership
CreateFrame/CreateFromMixins/Mixin assignment
native frame event registration
native EventRegistry frame-event bridge
custom registry callback only with an exact TriggerEvent producer
CVar callback registration
SetScript/HookScript/hooksecurefunc structural hooks
LibStub/library requirement and embed structure
SavedVariables roots and literal state paths
```

Framework-specific module/lifecycle factories, plugin/style/element ecosystems, message buses, Secret guard/sink recognizers, and named calibration packs remain deferred until their E5 corpus and mutation gates are defined.

## Current state

```text
documentation contract: complete
closed fixture shapes: complete
implementation-dependent pins and SHA-256 freeze: pending
Cargo workspace activation: not started
Rust source: absent
```
