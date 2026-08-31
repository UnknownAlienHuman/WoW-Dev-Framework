# `wow-project` contract router

**Status:** E0-D minimal generation and E2-C full source/TOC/XML/load/incremental-index contracts are implementation-ready documentation; Rust implementation has not started.

## Contract packages

### E0-D — minimal project generation

The original E0-D overview is preserved in [`E0_OVERVIEW.md`](E0_OVERVIEW.md). Its normative root-level companions remain:

```text
AGENTS.md
DECISIONS.md
DATA_MODEL.md
GENERATION_AND_PUBLICATION.md
UPDATE_MODEL.md
SOURCE_REGISTRY.md
ERROR_MODEL.md
TEST_MATRIX.md
IMPLEMENTATION_PLAN.md
CONTRACT.json
examples/
```

E0-D owns one exact closed Lua workspace, generation-bound analyzer update, first-party source registry, and immutable in-memory project snapshot. It does not parse TOC/XML or build graph facts.

### E2-C — full project index candidate

Read [`e2/README.md`](e2/README.md), [`e2/CONTRACT.json`](e2/CONTRACT.json), and the full [`e2/`](e2/) package.

E2-C adds:

```text
closed materialized project source snapshot
root/universe/package model
one selected TOC variant per package
bounded TOC directives/files/dependencies/LOD/bootstrap/SavedVariables parser
bounded streaming XML includes/templates/objects/inheritance/scripts parser
source-mapped XML inline Lua virtual units
static load order and reachability model
exact wow-emmy physical/virtual workspace binding
E2-B recognizer fact bundles and output partitions
E2-A graph proposal validation
incremental invalidation/reuse/stale-removal
immutable NotPublishedE2C ProjectIndexCandidate
```

Persistent ProjectStore, WAL, current pointers, final GraphGeneration, and atomic ProjectSnapshot/GraphSnapshot publication remain E2-D.

## Direct dependency activation

Maximum graph:

```text
wow-core
wow-store
wow-emmy
wow-graph
wow-recognizers
```

Active in E0-D:

```text
wow-core
wow-emmy
```

Active in E2-C:

```text
wow-core
wow-emmy
wow-graph
wow-recognizers
```

`wow-store` remains inactive until E2-D.

## Hard rules

- `wow-emmy` is the only Lua parser/analyzer.
- TOC variants never merge across flavor/profile boundaries.
- XML DTD, external entities, network access, and source execution are forbidden.
- LOD/bootstrap/static load order do not prove runtime readiness or success.
- SavedVariables contents are never read; roots require TOC declarations.
- First-party, dependency, analyzer-library, reference, external, and runtime universes remain separate.
- Recognizers emit proposals; graph validates them; project does not rewrite either contract.
- E2-C candidates are not persisted or current.
- Live patch-sensitive WoW guidance comes from the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb), not duplicated static assumptions.
