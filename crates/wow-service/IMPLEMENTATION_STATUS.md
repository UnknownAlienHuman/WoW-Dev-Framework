# `wow-service` implementation status

## Implemented E0-F surface

- Exact content-addressed service configuration.
- Closed operation registry with same-request replay, different-request conflict, in-progress blocking, and no retained failed attempt.
- `status` over one exact backend snapshot without implying that analysis or tests passed.
- `check` over one immutable context acquired exactly once for an explicit exact or project-scoped current selector.
- Cross-component validation for `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, and `wow-rules` identities.
- Preservation of generic findings, WoW rule findings, clean authority records, `NotEvaluated`, degradable failures, rule cancellation, component health, capability state, and exact source locations.
- Conservative semantic result ordering: failed/cancelled/partial/findings/clean, with external cancellation preventing publication.
- Deterministic presentation graph using only supplied structured `causes_or_explains`, `blocked_by`, and `exact_duplicate_of` relations. Raw findings are never deleted.
- Canonical request and result identities without timestamp, host path, process identity, credentials, or rendered message text.
- Explicit E0 deferred-operation failures for lookup, search, tree, skeleton, plan, patch impact, indexing, runtime review, LSP, MCP, release, and pack operations.
- Closed E0 acceptance coverage for full findings, structured folding, clean authority, partial output, replay/conflict, cancellation, and input-order invariance.

## Authority boundary

The service coordinates normalized owner output. It does not parse Lua, infer WoW API truth, classify Secret values, reinterpret rule evidence, select a replacement generation, mutate source, or execute analyzed code. The current `OwnedServiceBackend` is an exact in-memory owner port for immutable already-published E0 contexts; transport and source-acquisition adapters remain outside the service semantic core.

## Active operations

```text
status
check
```

All other documented operations remain typed `operation_not_implemented_for_milestone` failures.

## Next package

After E0-F is merged and the thin `apps/wow` one-shot CLI is active, implementation follows the dependency order documented in the root completion matrix. Later service packages must extend this registry rather than bypass it.
