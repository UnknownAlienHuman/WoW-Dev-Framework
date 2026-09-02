# E5-B CLI command grammar

**Status:** normative.

Common explicit options:

```text
--config <PATH>
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--consumer-profile <ID>
--budget-profile <ID>
--request-id <ID>
--operation-id <ID>
```

Exact artifact inputs include `--source`, `--corpus`, `--split`, `--fact-snapshot`, `--pack`, `--run`, `--case`, `--candidate`, `--deactivation-plan`, `--review-record`, `--holdout-generation`, `--holdout-grant`, `--submission`, and optional `--expect-digest` guards.

Strict JSON transport files are accepted only where declared: `--request`, `--review-input`, `--holdout-request`, `--authorization-input`, `--submission-profile`, `--input`, or `--continuation`. Maximum one `-` stdin source.

`run retry` accepts the original run/operation/request digest only and cannot replace pack, corpus, split, profile, or budget. Review commands accept a strict review envelope—not `--approve` or local identity. Holdout execution accepts an exact grant—not a vault path/token or evaluator plugin. Promotion commands are `prepare`, `validate`, and `get` only.

There is no generic operation escape hatch and no publish/promote/activate/canary/rollout/rollback command.