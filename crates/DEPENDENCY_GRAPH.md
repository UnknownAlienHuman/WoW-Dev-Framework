# Crate dependency graph

**Status:** normative implementation boundary through documentation frontier E3-B.

Dependencies point from orchestration/domain behavior toward narrower foundations. The table lists maximum permitted direct edges, not dependencies that must be activated before a work package requires them.

| Crate | Maximum allowed direct framework dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-graph`, `wow-project` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core`, `wow-graph`, `wow-project` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-service` | all production crates through public contracts |

Applications depend on `wow-service` and serialization/configuration adapters. They do not bypass the service layer to reproduce domain logic.

## E3 sequence and active slices

### E3-A — Blizzard UI source index

```text
wow-project
├── wow-core
├── wow-emmy
├── wow-graph
├── wow-recognizers
└── wow-store
```

This publishes a separate exact `blizzard_ui_source` ProjectSnapshot/GraphSnapshot and `SkeletonInputView`. It does not depend on `wow-context`.

### E3-B — context projection

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
    storage and read leases remain behind published project/graph views

wow-context -> wow-emmy
    analyzer facts arrive through exact project views

wow-context -> wow-recognizers
    producer records arrive through project/graph views

wow-context -> wow-search
    exact roots are supplied by the caller; search begins in E4

wow-context -> wow-rules
    existing findings are supplied as exact roots/evidence by a higher layer

wow-context -> wow-cbm
    external candidates begin in E6 and remain separately scoped

wow-context -> wow-service/apps
    upward dependencies are forbidden
```

## High-level shape

```text
                         wow-core
                ___________|________________________
               /           |          |             \
         wow-store      wow-emmy   wow-cbm     wow-reference
          /    |            |                       |
   wow-graph   |       wow-recognizers       wow-annotations
       |       |          /   |
       |     wow-project -----+
       |       /   |   \
       |  wow-rules | wow-search
       |       \    |   /
       +-------- wow-context
                    |
               wow-service
                    |
                  apps
```

The table and active work-package contracts are authoritative; the diagram is illustrative.

## Forbidden patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing project/reference/graph/search/rule/context semantics.
- `wow-reference` depending on `wow-annotations`.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing Lua, TOC, or XML.
- `wow-recognizers` depending on `wow-project`.
- `wow-project` depending on `wow-context`, `wow-search`, `wow-rules`, or service/apps.
- `wow-search` calling Codebase Memory directly.
- `wow-rules` performing persistence, network, process, source, editor, or client mutation.
- `wow-context` owning source acquisition/parsing, analyzer internals, search ranking, diagnostics/remediation, runtime proof, persistence, model inference, or external side effects.
- any production crate depending on an application crate.
- test helpers becoming runtime dependencies.

## Seam rules

Before adding an edge, prefer:

1. a stable shared primitive in `wow-core` only when universally owned;
2. a narrow exact read view from the owning crate;
3. a versioned generated artifact;
4. an operation request coordinated by `wow-service`;
5. a structured result object rather than a higher-layer callback;
6. a separately scoped producer universe rather than hidden acquisition.

A dependency change must include the exact operation crossing the seam, cycle and artifact-identity analysis, security/privacy/license effects, coverage/evidence effects, two-sided tests, and work-package activation updates.
