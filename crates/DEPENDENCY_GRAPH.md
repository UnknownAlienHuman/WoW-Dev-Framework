# Crate dependency graph

**Status: normative implementation boundary through documentation frontier E4-C**

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

## Active E4-C operation slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-reference
        ├── wow-project
        ├── wow-graph
        ├── wow-search
        └── wow-context
```

This is the maximum active direct slice for E4-C operations. Each operation uses only the required subset.

Examples:

```text
search_query
    wow-core + wow-store + wow-reference + wow-project + wow-graph + wow-search

lineage_build
    wow-core + wow-store + wow-reference + wow-project + wow-graph
    + optional exact wow-search Candidate producer

search_context
    search query/select slice + wow-context through the existing E3-C context composition
```

Inactive direct E4-C dependencies:

```text
wow-emmy
wow-annotations
wow-recognizers
wow-rules
wow-cbm
```

Their relevant immutable facts may arrive through exact owner publications. E4-C does not invoke them directly.

## E4-A search boundary

`wow-search` owns exact-generation shards, document projections, safe query AST, retrieval lanes, rank fusion, explanations, miss classification, pagination and continuation.

It does not resolve current, call service/context/CBM, accept lineage proof or choose a candidate for the caller.

```text
wow-search
├── wow-core
├── wow-store
├── wow-reference
├── wow-project
└── wow-graph
```

E4-A `wow-graph` access is bounded generation-local graph retrieval. E4-B lineage publication does not create a reverse dependency from graph to search.

## E4-B lineage producer boundary

`wow-graph` retains only:

```text
wow-graph -> wow-core + wow-store
```

Producer facts are supplied as typed artifacts by orchestration:

```text
wow-project -> project_stable_identity / source_fingerprint / structural_change partitions
wow-reference -> explicit transition / deprecation / replacement partitions
wow-search -> search_lineage_candidate partitions capped at Candidate
review authorization boundary -> validated decision envelopes
```

`wow-graph` validates and publishes lineage assertions/change/migration/static-impact state without importing those producer crates.

## E4-C search/context boundary

```text
caller query
-> apps/wow
-> wow-service
-> wow-search returns exact candidates and explanations
-> explicit result/candidate selection receipt
-> wow-service invokes wow-context with the selected exact entity root
```

`wow-search` never calls `wow-context`. `wow-context` never performs hidden search. Rank/similarity does not enter entity or context fact confidence.

## Review boundary

```text
strict review envelope
-> apps/wow transports data
-> wow-service invokes ReviewAuthorizationPort
-> wow-graph independently validates decision semantics/proof ceiling
-> new immutable LineageGraphSnapshot
```

Authorization does not create lineage proof. Graph validity does not bypass authorization. Neither owner imports the application.

## Current-resolution boundary

```text
apps/wow
    parses symbolic current
    -> wow-service
       resolves once through owner ports
       acquires exact retained owner/shard/lineage/context views
```

The app, `wow-search`, `wow-graph`, and `wow-context` never resolve current pointers. Independent stores are not represented as a distributed atomic snapshot.

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
- `wow-reference` depending on `wow-annotations`, project, graph, search, service or apps.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing source, running recognizers/search, resolving current, authorizing reviewers or calling service.
- `wow-recognizers` depending on `wow-project`.
- `wow-project` depending on search/context/service/apps.
- `wow-search` depending on context/service/apps/CBM or implementing lineage authority.
- `wow-context` depending on store/emmy/recognizers/rules/search/cbm/service/apps.
- `wow-rules` performing persistence/network/process/source/editor/client mutation.
- `wow-service` implementing owner algorithms, changing proof ceilings, applying migration/source edits or exposing raw owner handles.
- applications importing any framework crate except `wow-service`.
- a production crate depending on an application.
- test helpers becoming runtime dependencies.

## Seam patterns

1. Stable universal identity/evidence/result primitive in `wow-core`.
2. Narrow exact generation-bound read view from the owning crate.
3. Typed producer partition or immutable generated artifact.
4. Typed operation request/result coordinated by `wow-service`.
5. Independent authorization decision plus owner semantic validation.
6. Thin application adapter over service only.

## Activation order

```text
E0 diagnostic vertical slice
-> E1 Reference Pack production
-> E2 project/graph/recognizers/ProjectStore
-> E3-A Blizzard UI source universe
-> E3-B Project Map/L0/L1/context packs
-> E3-C service/application context operations
-> E4-A exact-generation search
-> E4-B explicit lineage/migration/static impact
-> E4-C service/application search/lineage/impact operations
-> E5 calibration corpora/packs and controlled promotion
-> E6 optional Codebase Memory candidates
-> E7 LSP/MCP/release/publishing
```

## E5-A boundary

Named calibration packs remain data/rule artifacts consumed by `wow-recognizers` under existing dependency limits. They cannot create a repository-specific dependency or make project/service code branch on addon/owner/path names.

Calibration orchestration belongs above recognizers in later E5-B service tooling; immutable core-pack rollout belongs to E5-C.

## Changing the graph

A dependency change requires the exact crossing operation/data, why existing orchestration/read view/artifact is insufficient, cycle and identity-DAG analysis, security/privacy/license/feature impact, coverage/evidence impact, tests on both sides, mutation of the rejected shortcut, migration notes, and manifest/workstream updates.
