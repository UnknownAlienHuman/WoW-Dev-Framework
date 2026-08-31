# Crate dependency graph

**Status: normative implementation boundary through documentation frontier E3-A**

The graph is acyclic. Dependencies point from orchestration/domain behavior toward narrower foundations. A crate must not depend on a higher layer merely to call a convenience function.

## Maximum allowed direct dependencies

The table lists the **maximum permitted** future direct edges, not dependencies that must be activated immediately. An inactive milestone edge stays absent from `Cargo.toml` until an exact executable contract requires it.

| Crate | Maximum allowed direct framework dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` when persistent pack storage is activated |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-graph` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph`, `wow-search` |
| `wow-service` | all production crates through their public contracts |

Applications under `apps/` may depend on `wow-service` and serialization/configuration adapters. They must not bypass the service layer to reproduce domain logic.

## E3-A active dependency slice

E3-A intentionally activates only:

```text
wow-context
├── wow-core
├── wow-reference
├── wow-project
└── wow-graph
```

The following maximum edges remain inactive in E3-A:

```text
wow-context -> wow-emmy
    analyzer facts arrive through the coherent published ProjectView

wow-context -> wow-search
    exact roots are supplied by the caller; search/ranking begins in E4

wow-context -> wow-store
    epoch/store/publication IDs are carried for coherence, while SQLite snapshots and leases remain behind ProjectView/GraphView
```

An E3-A implementation adding any of those direct dependencies fails the architecture gate unless a reviewed contract revision changes the slice.

## Dependency shape

```text
                         wow-core
                 _________|________________________________
                /         |          |          |          \
          wow-store   wow-emmy   wow-cbm   wow-reference   ...
              |          |                         |
          wow-graph      |                  wow-annotations
            /   \        |
 wow-recognizers \       |
          \       \      |
           \     wow-project
            \      /   \
             wow-rules   wow-search
                  \       /  \
                   \ wow-context
                    \   /
                  wow-service
                       |
                     apps
```

The diagram is illustrative; the maximum-edge table plus active work-package contracts are authoritative.

## Forbidden dependency patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing domain entities from reference/project/search/rules/context.
- `wow-reference` depending on `wow-annotations`; pack assembly is orchestrated above both.
- `wow-emmy` depending on `wow-project` or `wow-rules`.
- `wow-graph` parsing Lua, TOC, or XML.
- `wow-recognizers` depending on `wow-project`; project supplies normalized facts and consumes proposals.
- `wow-search` calling Codebase Memory directly; optional candidates arrive through `wow-service` from `wow-cbm`.
- `wow-rules` performing persistence, network, process, source, editor, or client mutation.
- `wow-context` owning source acquisition/parsing, analyzer internals, search ranking, diagnostics, remediation, runtime proof, or persistence.
- E3-A `wow-context` depending directly on `wow-store`, `wow-emmy`, `wow-search`, `wow-rules`, or `wow-cbm`.
- any production crate depending on an application crate.
- test helpers becoming runtime dependencies.

## Cross-crate seam rules

Use these patterns before adding an edge:

1. **Stable data contract:** place only universally owned identity/evidence primitives in `wow-core`.
2. **Read view:** the owning crate exposes a narrow exact read interface; consumers do not import storage/analyzer internals.
3. **Operation request:** `wow-service` coordinates two crates rather than making them depend on each other.
4. **Generated artifact:** one crate writes a versioned artifact; another reads its public contract.
5. **Event/result object:** lower crates return structured outputs; they do not call higher-layer callbacks with hidden state.
6. **Producer universe:** source acquisition/indexing publishes a separately identified universe; a projection crate never performs hidden acquisition.

## Activation by milestone

### E0

```text
wow-core
wow-reference      in-memory/fixture view; no persistent pack builder
wow-emmy
wow-project        minimal workspace/generation slice
wow-rules
wow-service
```

`wow-store`, `wow-graph`, and `wow-recognizers` may remain absent from the Cargo workspace if the E0 fixture path does not require them.

### E1

Activate `wow-store`, full `wow-reference`, and `wow-annotations` for deterministic Reference Pack production.

### E2

Activate `wow-graph`, full `wow-project`, core `wow-recognizers`, and ProjectStore coherent publication.

### E3-A

Activate only the four-edge `wow-context` slice above after E0-E2 implementation/freeze gates pass. Build Project Map, L0/L1/control-effect projections, source excerpts, bundles, continuation, rendering, and evaluation over exact published views.

### E3-B

Add a reviewed pinned Blizzard UI source producer through existing project/analyzer/graph owners or a separately contracted producer boundary. Do not add source extraction logic to `wow-context`.

### E4–E6

Activate `wow-search`, extended recognizers, and `wow-cbm` in that order. A later context contract may then activate the maximum `wow-search` edge for selected exact search results, never ranking authority.

## Changing the graph

A dependency change must include:

- the concrete operation crossing the seam;
- why orchestration, a read view, or a producer artifact is insufficient;
- cycle and artifact-identity-DAG analysis;
- security, privacy, license, and compile-time feature impact;
- coverage/evidence/provenance impact;
- tests for both sides and mutation of the proposed convenience shortcut;
- migration notes and work-package activation updates.
