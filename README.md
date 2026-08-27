# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Status:** architecture and repository bootstrap. The implementation has not started yet.
>
> **Initial target profiles:** World of Warcraft Retail / Midnight, including Interface `120001` and `120005`. Every released reference profile will be pinned to an exact Blizzard build and source digest.

WoW Dev Framework is intended to give coding agents and addon developers a small, exact, current, and explainable technical surface for working with Blizzard UI code and addon projects. It is not another generic RAG system, not a replacement for Codebase Memory, and not a runtime injection platform.

## Mission

The framework should let an agent answer and act on questions such as:

- Does this API exist in the selected WoW build?
- What replaced an API that Blizzard removed or moved?
- Which Blizzard package owns this frame, mixin, function, or template?
- How is a project file reached through TOC and XML load order?
- Which module owns this state path, callback, registry entry, or frame?
- What will break if this function, API call, hook, or state path changes?
- Is this use of a Secret Value, protected action, or Blizzard overlay surface safe?
- Where is the correct extension point, rather than a workaround?
- How can the agent obtain the smallest sufficient context instead of reading entire repositories?

## Product shape

```text
Blizzard UI snapshot
    ├── raw APIDocumentation catalog
    ├── Ketho-compatible annotation pack
    ├── FrameXML and UI graph
    ├── restriction and Secret facets
    └── build lineage

addon repository
    ├── EmmyLua syntax and semantic facts
    ├── TOC/XML/load facts
    ├── universal framework conventions
    ├── project graph
    └── generated Project Map

optional Codebase Memory bridge
    └── broad source discovery and semantic candidates

all layers
    → exact search
    → ownership/load/call trees
    → compact skeletons
    → diagnostics and planning
    → impact analysis
    → agent verification
```

The planned implementation is divided into four operational products:

1. **`wow-reference-builder`** — compiles versioned Blizzard reference packs and Ketho-compatible annotations.
2. **`wow-emmy-ls`** — a Rust host/companion around upstream EmmyLua analysis, merging generic Lua diagnostics with WoW-specific diagnostics.
3. **`wow-index`** — project/UI graph, search, lineage, skeletons, planning, patch impact, and Secret/restriction facts.
4. **`wow-cbm-bridge`** — an optional, documented bridge to an installed Codebase Memory MCP server. It never writes directly to Codebase Memory storage.

## Core principles

- **Blizzard UI content is authoritative.** A mirror or acquisition provider is provenance, not the domain authority.
- **No EmmyLua fork.** The upstream Rust analysis crate is pinned and isolated behind one adapter.
- **Ketho-compatible, not Ketho-dependent.** Ketho is a parity oracle and format reference; editor setting mutations are not copied.
- **One correctness-path Lua parser.** WoW recognizers consume canonical Emmy syntax/semantic facts instead of introducing a second parser.
- **No addon-name branches.** Ace3, oUF, ElvUI, WeakAuras, BigWigs, Details, and Plater are calibration corpora for universal recognizers, not hardcoded product modes.
- **Exact and historical evidence precede fuzzy search.** Similar names do not prove API replacement.
- **Unknown remains unknown.** Missing coverage is never presented as a clean result.
- **Small local storage first.** SQLite B-tree indexes, FTS5, adjacency tables, and bounded in-memory graph projections are the default.
- **Skeleton-first context.** Agents read signatures and control-flow skeletons before full source.
- **Open source by default.** The project is licensed under MIT and is designed for public, unobfuscated development.

## Planned agent surface

```text
wow_status
wow_lookup
wow_search
wow_tree
wow_skeleton
wow_plan
wow_check
wow_patch_impact
wow_index_repo
wow_runtime_review
```

The exact MCP/CLI/LSP surface will remain smaller than the internal service API. Related operations should be routed internally rather than exposed as dozens of narrowly scoped tools.

## Repository roles

This repository contains the implementation, normative architecture, schemas, contracts, fixtures, tests, and release tooling for the framework.

The living research and field-note repository remains separate:

- **[WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)**

The knowledge base is referenced; it is not duplicated here. Decisions promoted from research into this repository must become explicit contracts, ADRs, fixtures, or tests.

## Documentation

- [Documentation index](docs/README.md)
- [Project vision and boundaries](docs/PROJECT_VISION.md)
- [Normative architecture](docs/ARCHITECTURE.md)
- [Reference Pack contract](docs/REFERENCE_PACK.md)
- [EmmyLua integration and diagnostics](docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [Graph, search, planning, and impact](docs/GRAPH_SEARCH_AND_PLANNING.md)
- [Codebase Memory bridge](docs/CODEBASE_MEMORY_BRIDGE.md)
- [Secret Values and restrictions](docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [Agent workflow](docs/AGENT_WORKFLOW.md)
- [Roadmap](docs/ROADMAP.md)
- [Architecture decisions](docs/DECISIONS.md)
- [Research baseline](docs/RESEARCH_BASELINE.md)
- [Candidate ideas](docs/IDEAS.md)

## Planned repository layout

```text
crates/      production Rust libraries
apps/        CLI, MCP, and LSP binaries
schemas/     versioned public data contracts
tools/       builders, evaluators, corpus and release utilities
tests/       fixtures, golden tests, evaluation tasks, and compatibility probes
docs/        normative documentation and research snapshots
```

The placeholder directory documents describe the intended boundaries. Crates will be created only when an independently testable responsibility exists; the repository will not be split into artificial microcrates by line-count targets.

## Current phase

The next executable milestone is a vertical slice:

```text
pinned upstream EmmyLua analysis
+ one generated WoW annotation fixture
+ one APIDocumentation fixture
+ one generic Lua diagnostic
+ one WoW API diagnostic
+ one Secret-local diagnostic
→ one deterministic `wow check` result
```

See [ROADMAP.md](docs/ROADMAP.md) for acceptance gates.

## License

MIT. See [LICENSE](LICENSE).

## Author

Neomorph / [UnknownAlienHuman](https://github.com/UnknownAlienHuman)
