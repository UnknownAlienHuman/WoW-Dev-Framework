# AGENTS.md — `wow-service`

Read repository/crate instructions, `README.md`, `../DEPENDENCY_GRAPH.md`, `../WORKSTREAMS.md`, and exactly one package:

```text
E0-F -> root E0 files
E1-D -> e1/
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
E6-B -> e6/
```

Read current external WoW engineering KB routes for patch-sensitive work and actual addon instructions for addon-facing operations.

Common rules:

- Coordinate owners; never reproduce owner algorithms.
- Resolve permitted symbolic selectors once and replace them with exact IDs.
- Never select latest, best, highest score/metric, previous, first, last, sole, same-name, nearest, or default artifact/provider/mapping.
- Use narrow typed ports; no raw SQL, parser/session/process objects, filesystem roots, provider databases, or mutable graph/project handles.
- Register `OperationId + CanonicalRequestDigest` before effects.
- Response loss is not effect absence; `OutcomeUnknown` blocks blind repetition.
- Same operation ID with a different digest is rejected.
- No public success before retention and reverse-order closure.
- Authorization is independent of semantic proof and never inferred from GitHub/OS/CLI/file/commit identity.
- Review, holdout, signing, publication, canary, activation, rollout, rollback, external-provider use, mapping, selection, context, distribution, and runtime proof remain separate authorities.
- External provider results remain `semantic_candidate + Candidate`; mapping validates locator identity only; selection is not verification.
- Provider failure is lane-local; never hide fallback to another provider, stale cache, model, web, local search, or broader query.
- Preserve partial, candidate, blocked, conflict, truncated, `OutcomeUnknown`, `NotEvaluated`, cancelled, failed, revoked, rolled-back, and deactivated states exactly.
- Applications depend on `wow-service` only and invoke one service operation per command.
- No Cargo/Rust/workflow/placeholder/fake owner/reviewer/vault/signing/provider/session/mapping/canary evidence during documentation phase.