# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Status:** private architecture and repository bootstrap. Implementation has not started.
>
> **Compatibility model:** every result is evaluated against one exact World of Warcraft reference profile. The first planned compatibility fixtures cover Midnight-era Interface `120001` and `120005`; newer live profiles are added only from pinned Blizzard builds and source digests.
>
> **Immediate milestone:** build the E0 vertical slice. Do not replace it with another architecture rewrite.

WoW Dev Framework is intended to give coding agents and addon developers a compact, exact, current, and explainable technical surface for Blizzard UI and addon repositories. It combines versioned API/reference data, EmmyLua analysis, a WoW-specific project graph, deterministic search, diagnostics, patch-impact analysis, and bounded agent context.

It is not a generic RAG product, not a replacement for Codebase Memory, not a VS Code configuration mutator, and not a runtime injection platform.

## What the framework should answer

- Does this API, event, widget method, template, mixin, or package exist in the selected build?
- What evidence shows that an API was removed, moved, deprecated, or replaced?
- Which package, TOC, XML template, module, registry, or state root owns a symbol?
- Can a file or symbol be reached through the actual TOC/XML load graph?
- Which project surfaces are affected by an API, hook, state-path, package, or restriction change?
- Is an operation involving a Secret Value, protected action, forbidden object, or Blizzard-managed surface statically safe?
- What is the smallest sufficient source context an agent needs before editing code?
- Which conclusion is proven, derived, possible, merely a candidate, or not evaluated because coverage is incomplete?

## Product model

```text
Blizzard UI snapshot
    ├── raw APIDocumentation catalog
    ├── Ketho-compatible annotation pack
    ├── FrameXML and package graph
    ├── restriction and Secret facets
    └── build lineage

addon repository
    ├── EmmyLua syntax and semantic facts
    ├── TOC/XML/load facts
    ├── universal framework conventions
    ├── project graph
    └── generated Project Map

optional Codebase Memory bridge
    └── broad semantic source discovery and candidate traces

all layers
    → exact lookup and search
    → ownership/load/call/state trees
    → compact source skeletons
    → diagnostics and planning
    → patch-impact analysis
    → agent verification
```

The planned implementation is divided into four operational products:

1. **`wow-reference-builder`** — compiles immutable, versioned Blizzard Reference Packs and Ketho-compatible annotations.
2. **`wow-emmy-ls`** — hosts upstream EmmyLua analysis and merges generic Lua diagnostics with WoW-specific diagnostics in one project generation.
3. **`wow-index`** — owns the project/UI graph, exact and historical search, lineage, skeletons, planning, patch impact, and restriction facts.
4. **`wow-cbm-bridge`** — optionally queries an installed Codebase Memory MCP server. It never writes directly to Codebase Memory storage.

## Non-negotiable invariants

- Blizzard UI content is the platform authority; a mirror is only an acquisition provider.
- EmmyLua is pinned behind one adapter and is not forked by default.
- Emmy syntax and semantic facts are the sole correctness-path Lua parser output.
- Ketho and Numy are differential oracles, not hidden runtime dependencies.
- Raw Blizzard metadata and generated annotations are separate projections.
- One project generation uses one active reference profile; profiles are never mixed in diagnostics.
- WoW ownership is multi-axis: lexical, package, load, object, inheritance, registration, lifecycle, state, and call relations remain distinct.
- Named framework packs contain declarative universal recognizers; production logic never branches on an addon repository name.
- Exact, migration, and lineage evidence rank before fuzzy or semantic similarity.
- Unknown remains unknown. Missing coverage never becomes a clean negative answer.
- SQLite, FTS5, adjacency tables, and bounded in-memory projections are the default storage stack.
- Agents receive skeletons and source handles before full files.

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

The public MCP/CLI/LSP surface should remain smaller than the internal service API. Related operations are routed internally instead of being exposed as dozens of narrowly scoped tools.

## Current vertical slice

```text
pinned upstream EmmyLua analysis
+ one generated WoW annotation fixture
+ one APIDocumentation fixture
+ one generic Lua diagnostic
+ one WoW API diagnostic
+ one Secret-local diagnostic
→ one deterministic `wow check` result
```

Acceptance requires merged diagnostics, exact profile identity, explicit evidence and coverage, no editor-setting mutation, and byte-identical sorted output across repeated runs.

## Documentation

Start with the [documentation index](docs/README.md).

- [Project vision and boundaries](docs/PROJECT_VISION.md)
- [Normative architecture](docs/ARCHITECTURE.md)
- [Provenance, confidence, and coverage](docs/PROVENANCE_AND_COVERAGE.md)
- [Reference Pack contract](docs/REFERENCE_PACK.md)
- [EmmyLua integration and diagnostics](docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [Graph, search, skeletons, and planning](docs/GRAPH_SEARCH_AND_PLANNING.md)
- [Codebase Memory bridge](docs/CODEBASE_MEMORY_BRIDGE.md)
- [Secret Values and restrictions](docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [Agent workflow](docs/AGENT_WORKFLOW.md)
- [Security model](docs/SECURITY_MODEL.md)
- [Test strategy](docs/TEST_STRATEGY.md)
- [Roadmap](docs/ROADMAP.md)
- [Architecture decisions](docs/DECISIONS.md)
- [Research baseline](docs/RESEARCH_BASELINE.md)
- [Candidate ideas](docs/IDEAS.md)
- [Glossary](docs/GLOSSARY.md)

## Repository layout

```text
crates/      production Rust libraries, created only at proven boundaries
apps/        CLI, MCP, LSP, and service binaries
schemas/     versioned public data contracts
tools/       builders, evaluators, corpus, migration, and release utilities
tests/       fixtures, golden tests, evaluations, and compatibility probes
docs/        normative contracts, operating guidance, research, and archive
```

Placeholder directory documents define intended boundaries. A crate is created only when it has an independently testable responsibility; line count is not a reason to split code.

## Related knowledge base

Patch-sensitive WoW engineering research, field notes, regressions, and current platform guidance live separately:

- **[WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)**

This repository references that knowledge base; it does not duplicate it. A research conclusion is promoted here only when it becomes an explicit contract, ADR, schema, fixture, test, or release input.

## Contributing

Read [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) before changing contracts or adding implementation code. Architecture changes must identify the affected invariant, decision, acceptance gate, and migration impact.

## License

MIT. See [LICENSE](LICENSE).

## Author

Neomorph / [UnknownAlienHuman](https://github.com/UnknownAlienHuman)
