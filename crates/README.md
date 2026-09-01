# Crate implementation contracts

**Documentation frontier:** E3-C complete. **Implementation frontier:** not started.

This directory contains implementation contracts for the planned production Rust libraries. No `Cargo.toml` or `.rs` file is created merely because a directory exists.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. target crate router and work-package contract
7. current external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes
8. actual addon repository/local instructions for addon-facing work

## Production crates

| Crate | Primary ownership | Documentation frontier |
|---|---|---:|
| `wow-core` | identities, evidence, coverage, conflicts, findings, canonical envelopes | E0-A |
| `wow-store` | SQLite lifecycle, ReferenceStore, ProjectStore, objects, publication, recovery/GC | E2-D |
| `wow-reference` | profiles, APIDocumentation, corrections, exact ReferenceView | E1-B |
| `wow-annotations` | annotation semantic model, rendering, source maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter, snapshots, facts, diagnostics, coordinates | E0-C |
| `wow-project` | source universes, TOC/XML/load, analyzer/recognizer orchestration, publication, Blizzard UI index | E3-A |
| `wow-graph` | registries, semantic keys/assertions, partitions, snapshots, axes/queries | E2-D seam |
| `wow-recognizers` | declarative universal structural recognition | E2-B |
| `wow-rules` | diagnostic providers, capability gates, findings/remediation tiers | E0-E |
| `wow-search` | exact/alias/FTS/shape/graph retrieval and later lineage/impact | E4-A next |
| `wow-context` | Project Map, L0/L1, context selection/budgets/source/rendering | E3-B |
| `wow-cbm` | optional external Codebase Memory candidates | E6 |
| `wow-service` | cross-owner orchestration and public use cases | E3-C |

## Documentation versus implementation

Documentation packages completed:

```text
E0-A through E0-F
E1-A through E1-D
E2-A through E2-D
E3-A through E3-C
```

Rust implementation still starts at E0-A and follows dependency/freeze order. Later contracts are not permission to start later crates first.

## Active E0 implementation set when coding begins

```text
wow-core
wow-reference fixture slice
wow-emmy pinned adapter slice
wow-project minimal generation slice
wow-rules two E0 rules
wow-service status/check
apps/wow status/check
cross-crate golden fixture
```

E0 does not activate persistent store/graph/search/context/LSP/MCP/release code unless its reviewed vertical-slice contract is revised.

## Contract discipline

- Concrete Rust names may change only with same-change contract/fixture updates.
- No empty/default success; unavailable work is typed unsupported/NotEvaluated/failure.
- Allowed dependencies are maxima; active package slices are authoritative.
- No owner algorithm in service/application convenience code.
- Search remains separate from context: service exposes candidates, then passes explicit exact roots to context.
- Missing evidence is never clean/pass.
