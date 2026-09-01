# Crate dependency graph

**Status: normative implementation boundary through documentation frontier E4-A**

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
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-service` | production crates through reviewed public contracts |

Applications depend on `wow-service` only among framework crates.

## Active E4-A search slice

```text
wow-search
├── wow-core
├── wow-store
├── wow-reference
├── wow-project
└── wow-graph
```

Inactive direct dependencies for E4-A:

```text
wow-context
wow-cbm
wow-service
wow-rules
wow-emmy
wow-recognizers
wow-annotations
```

Relevant analyzer/recognizer state reaches search only through exact published project/graph owner views. Search does not run those components.

## SearchStore boundary

```text
wow-search
    owns logical documents, fields, indexes, query lanes,
    ranking, explanations, misses and result manifests

wow-store
    owns SQLite/runtime/VFS/transactions, immutable file/object lifecycle,
    read-only reopening, integrity plumbing, retention and GC
```

No raw SQL, table, rowid, PRAGMA, extension, connection, VFS, transaction callback, or path crosses the public seam.

## Search/context boundary

`wow-context` remains exact-root-only and does not depend on `wow-search`.

```text
caller structured/text query
-> wow-service [E4-C]
-> wow-search returns ranked exact candidates with explanations/evidence
-> caller or explicit service policy selects exact candidate IDs
-> wow-service
-> wow-context exact roots
```

Search ranking never becomes context authority, and context never performs hidden search.

## Lineage boundary

E4-A search can emit only Candidate evidence for a later lineage owner.

```text
wow-search retrieval signals
-> E4-B lineage evaluation in owning project/reference/graph contracts
-> accepted/rejected explicit lineage assertions
```

No dependency shortcut permits search to write graph lineage or Reference replacement facts directly.

## Current-resolution boundary

Applications may accept a symbolic current selector, but only `wow-service` resolves it. `wow-search`, `wow-context`, owner views, and continuations use exact generations.

## Forbidden patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing domain semantics.
- `wow-reference` depending on `wow-annotations`.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing source or running recognizers/search.
- `wow-recognizers` depending on `wow-project`.
- `wow-project` depending on search/context/service/apps.
- `wow-search` depending on context/service/apps/CBM/analyzer/recognizer/rules or owning their algorithms.
- `wow-search` writing owner graph/reference/project facts or resolving current.
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
6. Rebuildable derived sidecar bound to exact owner generations.
7. Thin application adapter over service only.

## Activation order

```text
E0 diagnostic vertical slice
-> E1 Reference Pack production
-> E2 project/graph/recognizers/ProjectStore
-> E3-A Blizzard UI source universe
-> E3-B Project Map/L0/L1/context packs
-> E3-C service/application context operations
-> E4-A exact-generation search core
-> E4-B explicit lineage/migration/impact
-> E4-C search/lineage/impact service and CLI
-> E5 calibration packs
-> E6 optional Codebase Memory candidates
-> E7 LSP/MCP/release/publishing
```

## Changing the graph

A dependency change requires the exact crossing operation/data, why existing orchestration/read view/artifact is insufficient, cycle and identity-DAG analysis, security/privacy/license/feature impact, coverage/evidence impact, tests on both sides, mutation of the rejected shortcut, migration notes, and manifest/workstream updates.
