# Crate dependency graph

**Status: normative implementation boundary**

The graph is acyclic. Dependencies point from orchestration/domain behavior toward narrower foundations. A crate must not depend on a higher layer merely to call a convenience function.

## Allowed direct dependencies

The table lists the **maximum permitted** direct edges, not dependencies that must be added immediately. An inactive milestone edge stays absent from `Cargo.toml` until executable behavior requires it.

| Crate | Allowed direct framework dependencies |
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

The diagram is illustrative; the table is authoritative.

## Forbidden dependency patterns

- `wow-core` depending on any framework crate.
- `wow-store` importing domain entities from reference/project/search/rules.
- `wow-reference` depending on `wow-annotations`; pack assembly is orchestrated above both to avoid a cycle.
- `wow-emmy` depending on `wow-project` or `wow-rules`; it exposes analyzer and provider seams, while higher crates supply project/rule behavior.
- `wow-graph` parsing Lua, TOC, or XML.
- `wow-recognizers` depending on `wow-project`; recognizers consume normalized fact inputs and emit graph facts, while project orchestration invokes them.
- `wow-search` calling Codebase Memory directly; optional candidates arrive through `wow-service` from `wow-cbm`.
- `wow-rules` performing persistence, network, process, or editor mutations.
- `wow-context` owning search ranking or diagnostics.
- any production crate depending on an application crate.
- test helpers becoming runtime dependencies.

## Cross-crate seam rules

Use these patterns before adding an edge:

1. **Stable data contract:** place only universally owned identity/evidence primitives in `wow-core`.
2. **Read view:** the owning crate exposes a narrow read interface; consumers do not import its storage internals.
3. **Operation request:** `wow-service` coordinates two crates rather than making them depend on each other.
4. **Generated artifact:** one crate writes a versioned artifact; another reads its public contract.
5. **Event/result object:** lower crates return structured outputs; they do not call higher-layer callbacks with hidden state.

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

`wow-store`, `wow-graph`, and `wow-recognizers` may remain absent from the Cargo workspace if the E0 fixture path does not require them. Do not introduce them only to match the final diagram.

### E1

Activate `wow-store`, full `wow-reference`, and `wow-annotations` for deterministic Reference Pack production.

### E2–E3

Activate `wow-graph`, full `wow-project`, core `wow-recognizers`, and `wow-context`.

### E4–E6

Activate `wow-search`, extended recognizers, and `wow-cbm` in that order. `wow-service` exposes the new use cases only after their milestone gates pass.

## Changing the graph

A dependency change must include:

- the concrete operation crossing the seam;
- why orchestration or a read view is insufficient;
- cycle analysis;
- security and compile-time feature impact;
- test coverage for both sides;
- migration notes for downstream crates.
