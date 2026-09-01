# Crate implementation contracts

**Documentation frontier:** E4-A complete. **Implementation frontier:** not started.

This directory contains implementation contracts for planned production Rust libraries. No `Cargo.toml` or `.rs` file is created merely because a directory exists.

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
| `wow-search` | immutable exact-generation shards, exact/alias/text/similarity/shape/graph retrieval, ranking and explanations | E4-A |
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
E4-A
```

Next documentation package:

```text
E4-B explicit cross-generation lineage, migration and bounded static impact
```

Rust implementation still starts at E0-A and follows dependency/freeze order. A later complete contract is not permission to implement later crates first.

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

E0 does not activate persistent store, graph, search, context, LSP, MCP, or release code unless its reviewed vertical-slice contract is revised.

## Search/context boundary

```text
query -> wow-service -> wow-search ranked candidates
-> explicit exact candidate selection
-> wow-service -> wow-context exact roots
```

`wow-search` does not call `wow-context`. `wow-context` does not perform hidden retrieval. Candidate rank does not become authority.

## Contract discipline

- Concrete Rust names may change only with same-change contract/fixture updates.
- No empty/default success; unavailable work is typed unsupported/NotEvaluated/failure.
- Allowed dependencies are maxima; active package slices are authoritative.
- No owner algorithm in service/application convenience code.
- Missing evidence is never clean/pass.
- Approximate search never establishes alias, lineage, replacement, impact, or negative authority.
