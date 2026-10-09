# Current project work map

Updated 2026-10-09, America/New_York. This map routes implementation work; it
does not replace the package contracts or certify their acceptance.

The verified starting checkpoint is `a248dadb89eb964e2f0167c762d633800f70d200`,
published and read back from `main`. It follows `e4d8ee7`, including the checked W11
semantic-repair product tree recorded in
[W11_SEMANTIC_REPAIR_MAP_2026-10-09.md](W11_SEMANTIC_REPAIR_MAP_2026-10-09.md).
The root workspace contains 16 real Rust members. Work continues sequentially
in `main`, with bounded owner responsibilities and verified remote publication.

## Current functional checkpoint

W11 has 14 of 26 declared core rule IDs in its service publication path. The
remaining work starts with the five TOC rules, followed by four XML and three
state rules. Existing TOC/XML/load/analyzer owners supply the records; recognizers
must consume typed facts rather than parsing source again.

The active TOC slice has three responsibilities:

| Owner | Responsibility | Current state |
|---|---|---|
| `wow-project` | Retain normalized TOC metadata and expose exact package, file-order, dependency, LOD and SavedVariables facts | Executable owner projection; integrated recognizer acceptance remains open |
| `wow-recognizers` | Compile and match five declarative core TOC families with source support, omissions and coverage | In progress |
| `wow-service` / `apps/wow` | Replace each partition in owner order and expose final graph crosswalks through the existing build command | In progress |

The TOC owner now retains normalized key/value pairs after conditional selection.
Package dependency projection consumes these retained records, preserving
conditional directives that the previous raw-span reparse lost. Generation-bound
facts retain source order, excluded/unresolved selections, repeated targets,
declaration occurrences and exact span/content/evidence identities. Missing
fields are omitted from strict canonical JSON rather than serialized as null.
This is producer code, not publication of the five declarative TOC rules.

The graph registry includes the distinct TOC entity/relation meanings. The
versioned Load recipe preserves v1 identities for old registries and requires
Loads, DependsOn, LoadsBefore and OptionalDependsOn for an extended registry.
It still rejects ambiguous registered definition IDs rather than silently
narrowing an axis.

The source-graph query-budget defect is repaired in the starting checkpoint and
has a local executable regression: graph
capacity and traversal budget are now separate, so source graph construction
does not supply a traversal limit above the graph owner's admitted ceiling.
The regression performs real project publication and graph partition materialization.
The local regression, focused checks and remote publication are separate evidence;
full graph acceptance and the checkpoint's CI conclusions remain open.

## Dependency order after the active slice

| Queue / reference | Next responsibility | Boundary |
|---|---|---|
| [W11 / PR 102](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/102) | TOC, XML, state facts/rules and application crosswalk closure | Full E2-B acceptance remains open |
| [Issue 103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103) | Executable structural mutations and admitted per-rule fixtures | After functional implementation; frozen fixtures are not rewritten by tests |
| [W12 / PR 80](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/80) | Conflict retention and complete derivation explanations | Existing graph query code is reused |
| [W13 / PR 81](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/81) | Coherent live ProjectView/GraphView publication | A graph-only retained snapshot is insufficient |
| [W14 / PR 82](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/82), [W15 / PR 83](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/83), [W16 / PR 84](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/84) | Incremental invalidation, retained roots/GC, backup/recovery | Exact generation and durable reconciliation gates |
| [W17 / PR 85](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/85), [W18 / PR 86](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/86), [W19 / PR 87](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/87) | Blizzard source universe, Project Map/skeletons, context packs | Real prerequisite views before context |
| [W20 / PR 88](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/88), [W21 / PR 89](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/89) | Search, lineage and static impact | Exact immutable generation inputs |
| [W22 / PR 90](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/90), [W23 / PR 91](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/91) | Sessions/MCP and private LSP overlays | Implemented capabilities only |
| [W24 / PR 92](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/92), [W25 / PR 93](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/93) | Calibration governance and selected Windows build/release tooling | No inferred review, signing or installed state |
| [W26 / PR 94](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/94) | Contract, dependency, fixture and queue validation | Reports missing evidence without blessing it |

E0/E1 fixture identity, checksum and Reference Pack acceptance remain separate
open requirements in [PROJECT_COMPLETION_MATRIX.md](PROJECT_COMPLETION_MATRIX.md).
Specification PRs supply scope, not evidence that code is implemented.

## Verification and source preparation

The current local compiler is Rust 1.99.0, observed on 2026-10-09. Workspace
all-target/all-feature compilation, strict Clippy and warnings-as-errors rustdoc
passed after the TOC owner changes. All 52 existing/new graph and project tests
passed, including a real TOC acquisition -> ProjectView -> source
facts -> graph partition regression passed locally. The regression covers
normalized conditions, repeated files, duplicate declarations and exact support.
Five Load recipe checks cover original/extended families, incomplete/ambiguous
registries and preservation of other axes.
Full TOC recognizer/service pipeline, package acceptance and named-client runtime
checks remain NotEvaluated until their actual results are recorded. Whole-workspace
tests remain separate from the focused graph/project test run.

Gethe `live` resolved at operation start to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`. This is one observation, not a
permanent dependency. Source materialization and annotation parity follow product
implementation/build. The WoW API MCP endpoint was successfully queried; that
alone does not verify a shared source revision or a complete comparative corpus.

Use [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) for annotation behavior and
[WASM_BRIDGES.md](WASM_BRIDGES.md) for narrow algorithm boundaries. Report complete,
partial, conflict, skipped and NotEvaluated outcomes at their actual scope.
