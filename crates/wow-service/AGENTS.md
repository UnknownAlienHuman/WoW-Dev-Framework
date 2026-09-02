# AGENTS.md — `wow-service`

## Required routing

Read repository/crate instructions, `README.md`, `../DEPENDENCY_GRAPH.md`, `../WORKSTREAMS.md`, and exactly one package:

```text
E0-F -> root E0 files
E1-D -> e1/
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
```

Read current external WoW engineering KB routes for patch-sensitive work and actual addon instructions for addon-facing operations.

## Common rules

- Coordinate owners; never reproduce owner algorithms.
- Resolve permitted symbolic selectors once and replace them with exact IDs.
- Never select latest, best, highest score/metric, first, last, sole, same-name, or nearest artifact.
- Use narrow typed ports; no raw SQL, parser/session objects, filesystem roots, or mutable graph/project handles.
- Register `OperationId + CanonicalRequestDigest` before effects.
- Response loss is not effect absence; `OutcomeUnknown` blocks blind repetition.
- Same operation ID with a different digest is rejected.
- No public success before retention and reverse-order resource closure.
- Authorization is independent of semantic proof and is never inferred from GitHub/OS/CLI/file/commit identity.
- Review, holdout access, disclosure, publication/signing, and runtime proof remain separate authorities.
- Preserve partial, candidate, blocked, conflict, truncated, `OutcomeUnknown`, `NotEvaluated`, cancelled, and failed states.
- Applications depend on `wow-service` only and make one service call per command.
- No Cargo/Rust/workflow/placeholder/fake owner/reviewer/vault/run evidence during documentation phase.