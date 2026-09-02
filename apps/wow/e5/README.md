# `apps/wow` E5-B calibration CLI contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow/e5-b/calibration-review-holdout-promotion-cli`

`apps/wow` is a thin adapter over `wow-service`; its only framework dependency is `wow-service`.

## Commands

```text
wow calibration status
wow calibration source validate
wow calibration corpus validate
wow calibration corpus admit
wow calibration split validate
wow calibration run submit|get|list|cancel|retry
wow calibration case explain
wow calibration candidate build|validate
wow calibration review validate|record
wow calibration holdout request|execute|audit
wow calibration promotion prepare|validate|get
wow calibration deactivation validate
```

Each valid command constructs exactly one typed service request and invokes service exactly once. The app never resolves catalogs/current, chooses latest/best/highest-metric/sole artifacts, opens project/store/graph/recognizer/vault resources, authorizes reviewers, inspects hidden holdout data, executes E5-A algorithms, mutates packs/labels/splits/source/graphs, or publishes/activates a core pack.

Read `AGENTS.md`, `CLI_COMMANDS.md`, `OUTPUT_EXIT_AND_STREAMS.md`, `SECURITY_AND_INPUTS.md`, `TEST_MATRIX.md`, `CONTRACT.json`, and `examples/`.