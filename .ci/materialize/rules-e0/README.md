# `wow-rules`

`wow-rules` owns deterministic policy evaluation over exact, already-normalized project and Reference evidence. It does not discover source, run an analyzer, execute Lua, infer a current client, or treat unresolved analyzer state as API absence.

## E0-E implemented rules

- `wow.api.missing.complete-reference/1`: emits `MissingApi` only when the analyzer observation is unresolved **and** the exact target-profile Reference partition is complete and explicitly absent.
- `wow.secret.concat.require-dominating-access-guard/1`: emits a Secret-value diagnostic only when the exact initializer API is present with complete Secret restriction evidence and the exact local binding lacks a dominating `canaccessvalue` guard for the operation.

Incomplete coverage produces `NotEvaluated`, never a fabricated pass or diagnostic. Shadowed bindings, dangling relations, cross-file links, contradictory evidence, duplicate identities, and mixed target profiles fail closed.

The E0-E report is canonically ordered and content-addressed. Its observations do not certify runtime safety, secure execution, combat legality, taint behavior, replacements, or API availability outside the bound target profile.
