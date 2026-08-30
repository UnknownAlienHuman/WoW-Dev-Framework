# Crate implementation contracts

This directory contains implementation briefs for the production Rust libraries planned by WoW Dev Framework.

**Current state:** contract scaffold only. No `Cargo.toml` or Rust source file should be created merely because a directory exists.

A crate becomes active only when the current roadmap milestone needs its independently testable responsibility. Directory presence is not permission to implement a later milestone, invent a public API, or add infrastructure "for future use."

## Required reading order

Before implementing any crate:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json) — machine-readable routing and activation state
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. the target crate's `README.md`
7. the owning normative documents linked by that crate
8. the current routes in the external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)

For work involving a concrete addon, inspect that addon repository and its local instructions before using framework-wide assumptions.

## Production crate map

| Crate | Owns | First implementation milestone | Brief |
|---|---|---:|---|
| `wow-core` | IDs, profile/generation identity, evidence, coverage, findings, stable handles | E0 | [`wow-core/README.md`](wow-core/README.md) |
| `wow-store` | SQLite lifecycle, migrations, transactions, content-addressed objects | E1/E2 | [`wow-store/README.md`](wow-store/README.md) |
| `wow-reference` | Reference Pack model, APIDocumentation ingestion, corrections, exact reference view | E0 fixture / E1 full | [`wow-reference/README.md`](wow-reference/README.md) |
| `wow-annotations` | Ketho-compatible annotations and WoW dialect projections | E1 | [`wow-annotations/README.md`](wow-annotations/README.md) |
| `wow-emmy` | Upstream Emmy adapter, analyzer actor, semantic facts, generic diagnostic normalization | E0 | [`wow-emmy/README.md`](wow-emmy/README.md) |
| `wow-project` | Addon workspace, TOC/XML/load model, incremental project generations | E0 minimal / E2 full | [`wow-project/README.md`](wow-project/README.md) |
| `wow-graph` | Typed entities/relations, lineage primitives, bounded graph queries | E2/E3 | [`wow-graph/README.md`](wow-graph/README.md) |
| `wow-recognizers` | Declarative universal recognizers over normalized facts | E2 core / E5 packs | [`wow-recognizers/README.md`](wow-recognizers/README.md) |
| `wow-search` | Exact, historical, shape, FTS, graph, and candidate ranking | E4 | [`wow-search/README.md`](wow-search/README.md) |
| `wow-rules` | WoW diagnostic providers and rule-specific remediation contracts | E0 | [`wow-rules/README.md`](wow-rules/README.md) |
| `wow-cbm` | Optional Codebase Memory MCP bridge and candidate normalization | E6 | [`wow-cbm/README.md`](wow-cbm/README.md) |
| `wow-context` | L0/L1 skeletons, Project Map, context budgets, detail negotiation | E3 | [`wow-context/README.md`](wow-context/README.md) |
| `wow-service` | Transport-independent use cases and cross-component orchestration | E0 | [`wow-service/README.md`](wow-service/README.md) |

Development-only crates such as `wow-testkit` or `wow-eval` are deferred until repeated cross-crate test responsibilities justify them. E0 tests may live under `tests/` and crate-local test modules first.

## Active E0 implementation set

E0 activates only these slices:

```text
wow-core
wow-reference      fixture-backed ReferenceView only
wow-emmy           one pinned analyzer adapter path
wow-project        minimal single-workspace generation only
wow-rules          wow.api.exists + one direct Secret-local rule
wow-service        status/check orchestration only
apps/wow           minimal CLI transport
cross-crate golden fixture
```

E0 does **not** activate `wow-store`, full Reference Pack building, annotation generation, graph persistence, recognizer packs, search, Codebase Memory, LSP, MCP, or release automation.

## Contract status

The operation inventories in crate briefs define required semantics and ownership. Concrete Rust names may be adjusted during implementation only when:

- the owning semantics remain unchanged;
- no dependency cycle or responsibility leak is introduced;
- the brief and affected contract are updated in the same change;
- the change is covered by an executable fixture or compatibility test.

Do not create placeholder methods that return empty/default success. An unavailable capability must be absent, explicitly unsupported, or return `NotEvaluated`/a typed unavailable state according to the owning contract.
