# `wow-context` implementation contract

**Status:** deferred to E3; contract scaffold only.

## Mission

`wow-context` builds compact, deterministic, evidence-bearing representations for agents and users: L0/L1 source skeletons, Project Maps, source detail bundles, and context-budget plans. It minimizes source reads without hiding coverage gaps or turning summaries into authority.

## Owned responsibilities

- L0 identity/role skeleton generation;
- L1 collapsed control/effect skeleton generation;
- Project Map generation;
- source-handle detail negotiation (L0/L1/exact span/surrounding declaration/full file);
- context request planning and byte/node/item budgets;
- deterministic rendering/serialization of context bundles;
- explicit truncation and omitted-capability reporting;
- redaction and untrusted-source quotation boundaries;
- context telemetry inputs for evaluation;
- cache keys tied to profile/generation/source digests.

## Explicit non-responsibilities

`wow-context` does not:

- parse source independently;
- rank search hits;
- decide diagnostic correctness;
- infer graph relations;
- read arbitrary filesystem paths;
- include full Blizzard/external repositories by default;
- convert summaries into proven facts without source handles;
- generate model-authored architecture prose in the correctness path;
- mutate project files or editor state.

## Detail levels

### L0 — identity and role

Required fields as available:

```text
signature/identity
entity kind and universe
profile/generation
owner/load chains
important registration/event/state/API roles
restriction/migration status
caller/callee/edge counts
source handle
coverage gaps
```

### L1 — control and effects

Required fields as available:

```text
signature
ordered branches/loops/early returns
calls/callbacks/hooks
access guards
state reads/writes
API/restriction interactions
collapsed implementation bodies
source handles for each retained item
coverage/truncation
```

### L2 — exact source

L2 is not generated prose. It resolves the exact source span, surrounding declaration, or explicitly budgeted full file from a validated stable handle.

## Project Map contract

Default target size is approximately 2 KB, with an explicit configurable byte cap. It includes only high-value generation state:

```text
project/profile/reference identity
TOC/load skeleton
first-party module/service owners
registries/extension points
state roots
critical invariants
known failed/partial partitions
runtime-required gaps
open workaround debt when project-confirmed
handles for deeper detail
```

It excludes full file lists, copied documentation, third-party source bodies, volatile timestamps, and unverified model conclusions.

## Required operations

| Operation | Required behavior |
|---|---|
| `build_l0_skeleton` | Render identity/role/chain/effect counts from normalized facts and handles. |
| `build_l1_skeleton` | Render bounded ordered control/effect structure without source-body duplication. |
| `build_project_map` | Select generation-level architecture facts under the configured byte budget. |
| `plan_context_bundle` | Choose the smallest handles/detail levels required by a task/plan/result. |
| `negotiate_detail_level` | Validate requested L0/L1/span/declaration/file level and byte budget. |
| `resolve_detail_handle` | Resolve through owning project/reference/external views; never arbitrary paths. |
| `render_context_bundle` | Produce deterministic transport-neutral structured output and optional Markdown. |
| `report_context_coverage` | List omitted/truncated/failed capabilities and source partitions. |
| `redact_context_metadata` | Remove private host paths, credentials, user data, and disallowed runtime payloads. |
| `record_context_metrics` | Count handles, files, bytes, graph nodes, detail levels, and external candidates without storing source contents by default. |

## Selection rules

1. Search/plan selects entities; context chooses detail, not ranking.
2. L0 precedes L1; L1 precedes L2 unless caller explicitly proves smaller direct span is sufficient.
3. Each material statement retains at least one source/evidence handle.
4. Missing facts are marked, not filled with plausible prose.
5. Source comments/docs are quoted as untrusted evidence.
6. Full file requires explicit request and budget; full repository is unavailable.
7. External source is included only after exact current platform/project facts and license/revision checks.
8. Truncation is deterministic and reported in coverage.
9. Equal-priority items use stable keys.
10. Context cache is invalidated by profile/generation/source digest changes.

## Skeleton generation boundaries

Skeletons consume normalized syntax/semantic/graph facts. They must not reconstruct control flow by regex or a second parser. Unsupported dynamic constructs remain summarized as unknown/possible with a source handle.

A skeleton cannot omit a branch/guard/state effect that changes the selected rule or plan conclusion merely to meet a byte target. It must instead truncate explicitly or request a larger budget.

## E3 implementation sequence

1. L0 contract and fixture;
2. L1 ordered control/effect representation;
3. detail-handle resolution;
4. Project Map selection/budget;
5. deterministic rendering;
6. truncation/coverage/redaction;
7. task context planner;
8. context-budget telemetry and agent evaluation.

## Required tests

- L0/L1 stable output under shuffled fact order;
- source handle present for every material item;
- exact span/declaration/file negotiation;
- path escape/digest mismatch rejection;
- byte/node/item budget and deterministic truncation;
- branch/guard/state effect preservation;
- Project Map size cap and priority selection;
- private path/token/runtime data redaction;
- source prompt-injection text quoted as data;
- generation/profile cache invalidation;
- fewer files/bytes in labeled agent tasks without reduced correctness.

## Documentation sources

- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/AGENT_WORKFLOW.md`](../../docs/AGENT_WORKFLOW.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## Definition of done

E3 context is complete when an agent can reach the correct owner/load/control/effect information with bounded L0/L1 reads, every conclusion remains resolvable to exact source, truncation is honest, and measured context size falls without hiding necessary facts.
