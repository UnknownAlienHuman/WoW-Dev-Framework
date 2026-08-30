# `wow-service` implementation contract

**Status:** E0-active contract scaffold; no Rust code yet.

## Mission

`wow-service` owns transport-independent use cases and orchestration across reference, analyzer, project, graph, rules, search, context, and optional external candidate components. It is the only production layer allowed to coordinate multiple domain crates into one public operation/result generation.

## Owned responsibilities

- service configuration and component capability assembly;
- `status`, `lookup`, `search`, `tree`, `skeleton`, `plan`, `check`, `patch_impact`, `index_repo`, and `runtime_review` use cases;
- profile/reference/project/external generation coherence;
- operation-level capability checks and partial-degradation policy;
- transaction/orchestration order across components;
- diagnostic provider execution and root-cause folding;
- optional CBM candidate lane merge;
- result envelope construction, budgets, cancellation, and deterministic ordering;
- transport-neutral request/response contracts;
- application-facing errors and health/status;
- last-known-good component activation policy.

## Explicit non-responsibilities

`wow-service` does not:

- implement parser, storage, graph, rule, ranking, or skeleton algorithms;
- expose raw database/analyzer/MCP handles to applications;
- duplicate CLI/LSP/MCP serialization behavior;
- select a floating current profile;
- hide `NotEvaluated`, partial coverage, conflicts, or stale external generations;
- execute addon/external repository code;
- mutate editor settings;
- turn optional component failure into false local failure;
- perform automatic edits without an owning proven remediation contract.

## Public use-case surface

The long-term public surface remains compact:

```text
wow_status
wow_lookup
wow_search
wow_tree
wow_skeleton
wow_plan
wow_check
wow_patch_impact
wow_index_repo
wow_runtime_review
```

CLI, MCP, and LSP frontends call these same use cases. A transport-specific convenience is not a reason to add a new domain operation.

## Required operations

| Operation | Required behavior |
|---|---|
| `status` | Report exact component/profile/generation/capability state without implying validation success. |
| `lookup` | Perform exact profile/universe-scoped lookup with evidence and negative-authority state. |
| `search` | Coordinate deterministic local lanes and optional candidate lanes without confidence upgrades. |
| `tree` | Return one bounded explicit graph-axis projection. |
| `skeleton` | Resolve L0/L1/L2 detail through validated handles and budgets. |
| `plan` | Build an evidence-backed implementation/study/test plan. |
| `check` | Merge generic and WoW diagnostics for one coherent generation. |
| `patch_impact` | Intersect reference deltas with project facts and return a bounded impact/test plan. |
| `index_repo` | Index one explicit repository/universe/revision under security and license policy. |
| `runtime_review` | Validate/import scenario-scoped runtime evidence without globalizing it. |

### `status`

Return:

```text
service/tool versions
selected/available profiles and reference generations
project generation and indexed roots
component capabilities/coverage
analyzer compatibility state
optional CBM status/generation
last-known-good/failed components
budgets and public schema versions
```

No status field may imply a check passed merely because a component is installed.

### `lookup`

Perform exact entity/reference/project lookup under an explicit profile/universe and return evidence/coverage/source handles. No fuzzy fallback is hidden inside exact lookup.

### `search`

Coordinate local exact/historical/text/graph lanes and optionally merge CBM candidates. Preserve lane/evidence separation and negative-authority semantics.

### `tree`

Request one explicit graph axis/view, generation, root, relation filter, and budget. Return bounded evidence-bearing projection.

### `skeleton`

Resolve entity/source handles through `wow-context` at explicit L0/L1/L2 detail and budget.

### `plan`

Combine target entities, owner/load chains, exact contracts, known restrictions, smallest source handles, files likely to change, required checks, and runtime scenarios. Plans distinguish facts from candidates.

### `check`

Assemble one coherent project/analyzer/reference context, select runnable rules, execute generic and WoW diagnostics, record `NotEvaluated`, fold known root causes, sort deterministically, and return one result envelope.

### `patch_impact`

Coordinate reference delta/lineage with project uses/hooks/templates/state/load facts, then return direct/derived/possible/candidate/not-evaluated impact and a bounded study/test plan.

### `index_repo`

Index only an explicitly configured repository/universe/revision with security/license/root/budget policy. External repositories remain read-only candidate evidence.

### `runtime_review`

Validate/import structured runtime evidence tied to build/profile/addon revision/scenario and relate it to static findings without generalizing beyond the observed context.

## Orchestration rules

1. Resolve explicit profile/reference generation before project/query/check work.
2. Acquire coherent immutable component snapshots/leases before executing a request.
3. Reject cross-generation inputs rather than retrying silently against a different generation.
4. Check capabilities before invoking dependent rules/lanes.
5. Optional component failure degrades only its lane.
6. Cancellation propagates; unpublished writes are aborted and late responses discarded.
7. Final ordering/folding is deterministic and based on structured IDs/causes, not message text.
8. Every result reports used/skipped/failed lanes and coverage.
9. Service orchestration cannot upgrade evidence confidence.
10. Applications never bypass service to reconstruct a richer answer.

## E0 service scope

E0 implements only:

```text
status
check
```

### E0 `status`

Reports exact fixture profile/reference generation, minimal project generation, pinned analyzer compatibility state, active rules/capabilities, and unsupported later capabilities.

### E0 `check` sequence

```text
validate request/profile
acquire fixture ReferenceView
acquire minimal ProjectGeneration + Emmy snapshot
normalize built-in Emmy diagnostics
select E0 rules
emit NotEvaluated for missing rule capabilities
run wow.api.exists
run wow.secret.local_operation
fold only explicitly known root causes
canonical sort
validate/build one result envelope
hand to apps/wow serializer
```

E0 excludes lookup/search/tree/skeleton/plan/impact/index/runtime APIs except explicit typed unavailable status in `status`. Do not implement fake empty success responses.

## Root-cause folding

Service may fold descendants only when a deterministic causal relation is supplied, for example:

```text
profile unavailable -> dependent API rules NotEvaluated
annotation library failed -> downstream unknown globals grouped
TOC partition failed -> reachability rules NotEvaluated
unknown restriction facet -> dependent Secret rules NotEvaluated
```

Raw findings remain inspectable. Independent errors are never hidden merely because their messages look similar.

## Application boundary

Applications under `apps/` own:

- CLI arguments/stdin/stdout/exit codes;
- MCP/LSP transport/session protocols;
- user-facing serialization format selection;
- process startup/config loading.

They call `wow-service` and do not import lower crates to perform richer logic.

## Failure taxonomy

Service-level states include:

```text
profile_required_or_unavailable
reference_generation_unavailable
project_generation_mismatch
component_capability_unavailable
operation_not_implemented
request_budget_exceeded
request_cancelled
partial_result
optional_lane_unavailable
result_contract_violation
```

A partial result carries useful completed lanes plus explicit failures. It is not automatically an overall success.

## Required tests

### E0

- status exposes exact fixture/analyzer/project/rule capability state;
- check returns generic + API + Secret findings under one context;
- clean fixture remains clean;
- partial capability produces `NotEvaluated` and no false pass;
- profile/generation mismatch rejected;
- root-cause folding preserves raw descendants;
- repeated output byte-identical after canonical serialization;
- cancellation and failed project update preserve last-known-good;
- unimplemented operations return typed unavailable, not empty success;
- application CLI uses service only.

### Later

- optional CBM failure isolation;
- bounded tree/context/search/impact;
- stale external generation;
- runtime evidence scenario scoping;
- multi-transport normalized equivalence;
- operation authorization/root path/security constraints;
- last-known-good dependency update rollback.

## Documentation sources

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/AGENT_WORKFLOW.md`](../../docs/AGENT_WORKFLOW.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/ROADMAP.md`](../../docs/ROADMAP.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [Current WoW addon agent workflow](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_DevWorkflow.md)
- [Current KB task router](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md)

## Definition of done

The E0 service is complete when one `status` and one `check` path produce coherent, deterministic, evidence-bearing output through a thin CLI, with honest unsupported/NotEvaluated states and no domain algorithm duplicated in the orchestration or transport layer.
