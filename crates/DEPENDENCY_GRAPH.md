# Crate dependency graph

**Status: normative implementation boundary through documentation frontier E3-C**

Dependencies point from orchestration/domain behavior toward narrower foundations. The graph must remain acyclic. Maximum permitted edges are not instructions to activate every dependency.

## Maximum allowed direct dependencies

| Crate | Maximum permitted direct framework dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-graph`, `wow-project` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-service` | production crates through reviewed public contracts |

Applications depend on `wow-service` only among framework crates.

## Active E3-C context-service slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-reference
        ├── wow-project
        ├── wow-graph
        └── wow-context
```

This is the operation-path slice for E3-C context commands. Existing E0/E1 service operations retain their own contracts/dependencies.

Inactive context-operation dependencies:

```text
wow-emmy
wow-annotations
wow-recognizers
wow-rules
wow-search
wow-cbm
```

Analyzer/recognizer/finding records may arrive through exact published project/graph artifacts. E3-C does not run those owners.

## Search/context boundary

`wow-context` remains exact-root-only and does not depend on `wow-search`.

Planned E4 shape:

```text
caller text/structured query
-> wow-service
-> wow-search returns ranked candidates with explanations/evidence
-> caller or explicit service policy selects exact candidate IDs
-> wow-service invokes wow-context with exact roots
```

Search ranking never becomes context authority, and context never performs hidden search. This avoids a search/context dependency and keeps candidate selection visible.

## Current-resolution boundary

```text
apps/wow
    parses symbolic current
    -> wow-service
       resolves once through public store/project ports
       acquires exact project/graph/reference views
       binds wow-context ContextUniverseSet
```

The app and `wow-context` never read current pointers. `wow-service` does not access raw SQL/SQLite; physical acquisition remains behind owner ports.

## Illustrative shape

```text
                         wow-core
              _____________|_________________________
             /       /      |        |               \
       wow-store wow-emmy wow-cbm wow-reference      ...
          |        |                    |
       wow-graph   |              wow-annotations
       /      \    |
wow-recognizers \ |
        \       wow-project
         \       /     \
          wow-rules   wow-search
               \       /  \
                \ wow-context
                 \    /
               wow-service
                    |
                  apps
```

The table and active work-package contracts are authoritative; this diagram is explanatory.

## Forbidden patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing domain semantics.
- `wow-reference` depending on `wow-annotations`.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing source or running recognizers.
- `wow-recognizers` depending on `wow-project`.
- `wow-project` depending on search/context/service/apps.
- `wow-search` depending on `wow-context`, `wow-service`, apps, or Codebase Memory.
- `wow-context` depending on store/emmy/recognizers/rules/search/cbm/service/apps.
- `wow-rules` performing persistence/network/process/source/editor/client mutation.
- `wow-service` implementing owner algorithms or exposing raw owner handles.
- applications importing any framework crate except `wow-service`.
- a production crate depending on an application.
- test helpers becoming runtime dependencies.

## Seam patterns

1. Stable universal identity/evidence/result primitive in `wow-core`.
2. Narrow exact generation-bound read view from the owning crate.
3. Typed operation request/result coordinated by `wow-service`.
4. Immutable generated artifact with owner schema/identity.
5. Independently replaceable producer partition with evidence/coverage.
6. Thin application adapter over service only.

## Activation order

```text
E0 diagnostic vertical slice
-> E1 Reference Pack production
-> E2 project/graph/recognizers/ProjectStore
-> E3-A Blizzard UI source universe
-> E3-B Project Map/L0/L1/context packs
-> E3-C service/application context operations
-> E4-A exact/alias/FTS/shape/graph search lanes
-> E4-B explicit lineage/migration/impact
-> E5 calibration packs
-> E6 optional Codebase Memory candidates
-> E7 LSP/MCP/release/publishing
```

## Changing the graph

A dependency change requires the exact crossing operation/data, why existing orchestration/read view/artifact is insufficient, cycle and identity-DAG analysis, security/privacy/license/feature impact, coverage/evidence impact, tests on both sides, mutation of the rejected shortcut, migration notes, and manifest/workstream updates.
