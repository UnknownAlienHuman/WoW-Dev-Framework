# Crate dependency graph

**Status: normative implementation boundary through documentation frontier E3-B**

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
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph`, later `wow-search` |
| `wow-cbm` | `wow-core` |
| `wow-service` | production crates through reviewed public contracts |

Applications under `apps/` depend on `wow-service` and serialization/configuration adapters only. They do not reproduce domain logic.

## Active E3-B slice

```text
wow-context
├── wow-core
├── wow-reference
├── wow-project
└── wow-graph
```

Inactive direct edges in E3-B:

```text
wow-context -> wow-store
    exact persistence/generation state remains behind published project and graph views

wow-context -> wow-emmy
    analyzer facts arrive through the project view

wow-context -> wow-recognizers
    accepted/rejected recognizer records arrive through project/graph views

wow-context -> wow-rules
    existing finding evidence may be supplied as typed owner records; context does not run rules

wow-context -> wow-search
    callers supply exact roots; natural-language/fuzzy/ranking starts in E4

wow-context -> wow-cbm
    external semantic candidates are E6 and never context authority

wow-context -> wow-service
    orchestration depends on context, never the reverse
```

## E3-C planned orchestration slice

E3-C may activate `wow-service -> wow-context` plus the exact owner crates required to resolve a current request into retained exact views. It must not make `wow-context` depend on service or applications.

```text
apps/wow
    -> wow-service
        -> wow-context
        -> exact project/graph/reference/store acquisition seams
```

`wow-service` resolves symbolic current at the outer boundary exactly once, acquires coherent retained views/leases, calls context operations, closes resources, and returns canonical envelopes. It does not implement Project Map, skeleton, selection, renderer, graph, source, or store algorithms.

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

The maximum table and active work-package contracts are authoritative; the diagram is explanatory.

## Forbidden dependency patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing reference/project/graph/search/rules/context semantics.
- `wow-reference` depending on `wow-annotations`.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing Lua/TOC/XML or running recognizers.
- `wow-recognizers` depending on `wow-project`.
- `wow-project` depending on `wow-context`, `wow-search`, `wow-rules`, `wow-service`, or apps.
- `wow-search` calling Codebase Memory directly.
- `wow-rules` performing persistence, network, process, source, editor, or client mutation.
- `wow-context` owning source acquisition/parsing, analyzer internals, search ranking, rules, runtime proof, persistence, physical cache, or service orchestration.
- E3-B `wow-context` depending directly on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, or `wow-service`.
- a production crate depending on an application crate.
- test helpers becoming runtime dependencies.

## Cross-crate seam patterns

1. **Stable universal primitive:** place only universally owned identities/evidence/result primitives in `wow-core`.
2. **Exact read view:** the owner exposes a narrow generation-bound interface; consumers do not import storage/analyzer internals.
3. **Operation request/result:** `wow-service` coordinates crates instead of creating peer cycles.
4. **Generated immutable artifact:** producer owns schema/identity; consumer validates the public contract.
5. **Producer partition:** independently replaceable facts/assertions preserve owner/version/evidence/coverage.
6. **Application adapter:** CLI/LSP/MCP parse/serialize/cancel only and call service.

## Activation order

```text
E0 core/reference/emmy/project/rules/service vertical slice
-> E1 store/reference/annotations/reference-pack production
-> E2 graph/recognizers/full project/ProjectStore publication
-> E3-A Blizzard UI source universe in wow-project
-> E3-B Project Map/L0/L1/context packs in wow-context
-> E3-C service/application context orchestration
-> E4 search/lineage/impact
-> E5 calibration packs
-> E6 optional Codebase Memory candidates
-> E7 LSP/MCP/release/publishing
```

## Changing dependencies

A dependency change requires the exact operation/data crossing the seam, why an existing read view/artifact/service operation is insufficient, cycle and identity-DAG analysis, security/privacy/license/feature impact, coverage/evidence impact, both-side tests, mutation of the rejected convenience shortcut, migration notes, and manifest/workstream updates.
