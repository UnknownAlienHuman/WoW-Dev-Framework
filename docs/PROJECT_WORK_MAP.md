# Current project work map

Updated 2026-10-09, America/New_York. This map routes implementation work; it
does not replace the package contracts or certify their acceptance.

The verified W11 predecessor is `342359b41ba3b6ad6ec3b6ee7654408c0fe01a78`,
published and read back from `main`. It includes the checked W11
semantic-repair product tree recorded in
[W11_SEMANTIC_REPAIR_MAP_2026-10-09.md](W11_SEMANTIC_REPAIR_MAP_2026-10-09.md).
The root workspace contains 16 real Rust members. Work continues sequentially
in `main`, with bounded owner responsibilities and verified remote publication.

## Current functional checkpoint

W11 has all 26 declared core rule IDs in its functional service publication path.
The five TOC, four XML and three state families publish alongside the earlier W11
families. Existing
TOC/XML/load/analyzer owners supply the records; recognizers
must consume typed facts rather than parsing source again.

The completed TOC slice has three responsibilities:

| Owner | Responsibility | Current state |
|---|---|---|
| `wow-project` | Retain normalized TOC metadata and expose exact package, file-order, dependency, LOD and SavedVariables facts | Executable owner projection; service integration verified, full acceptance open |
| `wow-recognizers` | Compile and match five declarative core TOC families with source support, omissions and coverage | Executable; service-published through the TOC pipeline |
| `wow-service` / `apps/wow` | Replace each partition in owner order and expose final graph crosswalks through the existing build command | Executable for the TOC slice |

The TOC owner now retains normalized key/value pairs after conditional selection.
Package dependency projection consumes these retained records, preserving
conditional directives that the previous raw-span reparse lost. Generation-bound
facts retain source order, excluded/unresolved selections, repeated targets,
declaration occurrences and exact span/content/evidence identities. Missing
fields are omitted from strict canonical JSON rather than serialized as null.
The service publishes five independent TOC partitions in dependency order. Each
evaluation retains its exact normalized fact bundle, coverage witnesses and full
matcher output; receipts bind original fact IDs to final graph proposals.
Equivalent entity assertions retain every source/evidence witness. Repeated or
unselected file occurrences cannot manufacture an ordering DAG, and unresolved
dependencies remain explicit omissions.

The graph registry includes the distinct TOC entity/relation meanings. The
versioned Load recipe preserves v1 identities for old registries and requires
Loads, DependsOn, LoadsBefore and OptionalDependsOn for an extended registry.
It still rejects ambiguous registered definition IDs rather than silently
narrowing an axis.

The XML slice consumes retained declaration, explicit parent, inheritance and
script facts. Five ordered partitions implement four unique rule IDs:
`core.xml.template`, `core.xml.object` (declaration and parentage phases),
`core.xml.inherits` and `core.xml.script`. Object parentage reads materialized
Object nodes; TOC publication supplies package/variant ownership first. Each
evaluation retains its normalized fact bundle, matcher output and exact receipts.
The final `xml_topology` crosswalk reads materialized node/edge identities.

Explicit `ParentOf` runs parent to child; lexical nesting cannot manufacture it.
The Object axis uses a versioned MultiParent recipe. `Inherits` and
`ReferencesTemplate` retain distinct relations. Named and inline handlers reuse
the same Emmy session and exact source support; unresolved and ambiguous targets
remain omissions. Script chunks remain analyzed source units without callback
bindings. Captured XML coverage stays Partial, so matcher outputs remain Possible;
Object traversal requires `IncludePossible` to follow these edges. No runtime
frame class, implicit receiver, effective dispatch or negative authority follows.

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
| [W11 / PR 102](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/102) | Public CLI and per-rule acceptance closure | All 26 functional rule IDs publish; full E2-B acceptance remains open |
| [Issue 103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103) | Executable structural mutations and admitted per-rule fixtures | After functional implementation; frozen fixtures are not rewritten by tests |
| [W12 / PR 80](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/80) | Extend real producer chains and conflict assessment | Exact retained records, publication validation and bounded explanations are executable; full acceptance remains open |
| [W13 / PR 81](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/81) | CLI and loader-plan live publication | Physical Lua native replay and coherent leased pair acquisition are executable; full acceptance remains open |
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

The current local compiler is Rust 1.99.0, observed on 2026-10-09. Native repository
policy checks also passed at the recorded native policy checkpoint.

The XML checkpoint was verified with whole-workspace `cargo check --locked
--all-targets --all-features`, strict Clippy under `-D warnings`,
`RUSTDOCFLAGS="-D warnings" cargo doc`, `cargo fmt --all --check`,
`cargo xtask check` and the full workspace test suite: 837 passed, 1 ignored,
across 102 test targets. Native policy checked 1,391 distributable files.

The single ignored test is `both_consumers_interpret_generated_library`. It
requires two explicitly approved consumer executables, so it stays ignored rather than
being counted as a pass; it remains mandatory in the consumer CI job.
Current Gethe materialization and the native annotation driver comparison remain
NotEvaluated and follow the product implementation/build stage.

The TOC pipeline ran end to end through the named package loader,
ProjectPublisher, all five declarative TOC matcher families, service partition
replacement and the final node/edge crosswalk, including repeated LoadOnDemand and
SavedVariable support. That is a functional path over fixture and synthetic project
inputs, not package acceptance.

The XML pipeline additionally verifies explicit versus lexical parentage, Object
traversal confidence filtering, inheritance/template references, exact named and
inline handler bindings, unresolved/ambiguous omissions and source-only chunks.

The source-graph projection profile is `wow-project/source-load-proposals/19`
(registry version 15),
the graph-build result is `wow-service/graph-build-result/16`, and the unchanged
request shape is `wow-service/graph-build-request/9`. Legacy record-free batches
and snapshots retain v1 identities; record-bearing ones use explicit v2 schemas.
Explanation payloads use v2; graph-read request shapes remain compatible.

The state checkpoint publishes `core.state.saved_variable_root`,
`core.state.literal_path_read` and `core.state.literal_path_write`. Root facts
reuse the selected TOC owner; access facts consume the exact admitted legacy
state partition after its analyzer/span/support validation. Read/write packs
execute the existing matcher. Typed path identities match the source owner;
equivalent paths merge every access witness, and final crosswalks retain exact
materialized IDs. Partial coverage keeps outputs Possible; empty target shapes
are NotEvaluated. Dynamic keys, ambiguous roots and local shadowing cannot
manufacture exact state paths. These are static associations, not runtime values
or persistence claims.

Fresh state checks on 2026-10-09 passed: native policy (1,399 distributable files),
formatting, whole-workspace check, strict Clippy, rustdoc, whole-workspace build
and tests (838 passed, 1 ignored, 102 targets). The focused pipeline also rejects
substituted source handles at unchanged edge endpoints/evidence and checks exact
alias support, dynamic-key omissions and local shadowing. Full rule fixture freeze,
public CLI and named-client acceptance remain open.

Full W11 fixture freeze, the `apps/wow` CLI acceptance lane, real-addon
graph-build acceptance, the Ketho MCP comparative gate, Windows and named-client WoW
runtime checks remain open, as does full E2-B package acceptance. The earlier focused
graph/project test run remains part of this evidence and is not a substitute for the
whole-workspace suite.

## W12 assertion records checkpoint

Graph publication now admits producer-local and exact cross-producer assertion
references, derivation inputs/rebuttals and reported unresolved conflicts. It
rejects cycles, stale batches, mixed scopes and confidence promotion before
publication. Conflicting participants remain intact; incident relation coverage
is downgraded conservatively, without a selected winner or negative authority.

Explanations return exact additional supports, derivation/conflict observations,
producer versions and missing/truncated boundaries under scan, support, byte and
depth limits. Source19 and legacy/core state producers retain real prerequisites;
the service state chain closes through captured file assertions. Other producers
may still lack derivation records. Reported conflicts are not a complete automatic
conflict assessment. See [GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md).

Fresh checks on 2026-10-09 passed: native policy (1,409 distributable files),
formatting, whole-workspace check, strict Clippy, tests (842 passed, 1 ignored,
103 targets), rustdoc and whole-workspace build. Focused regressions exercise
actual publication, stale/cyclic/promoted-input rejection, canonical identities,
conflict retention, exact state chains and stored v1/v2 read-back. These checks
do not close W12/E2 package acceptance, W13 live views or source/runtime gates.

## W13 physical-input live pair checkpoint

The W12 predecessor `8ef387c3534b275664f9307dc2647278e3ef7fae` was published
and read back with all 27 changed blob identities. W13 now has an owned native
replay archive, a project/graph publication bundle and a separate service store
profile. Read-back runs the real ProjectPublisher over exact captured Main and
Library inputs, compares original semantic IDs and validates the source graph
against that live view before activation. A returned pair holds one store read
transaction/lease; Current/Exact reads cannot mix generations.

The admitted profile is physical Lua configuration. Loader plans reject explicitly;
CLI wiring, TOC/XML/package replay and remaining W13 acceptance are next work.
This path does not relabel a retained graph receipt as a live project or certify
all recognizer sidecars. See
[REPLAY_PUBLICATION.md](../crates/wow-project/REPLAY_PUBLICATION.md).

Fresh workspace gates on 2026-10-09 passed: policy, fmt, check, strict Clippy,
tests (844 passed, 1 ignored, 103 targets), rustdoc and build. Actual store tests
cover close/reopen, old leased readers during current advancement, CAS rejection,
Main/Library separation, cancellation and missing/mutated/mixed input rejection.
Gethe `live` was re-resolved to `09b9db7948abc9b9648dedaab51eb0cf3ee67b31`;
materialization and native annotation parity remain NotEvaluated.

Gethe `live` resolved at operation start to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`. This is one observation, not a
permanent dependency. Source materialization and annotation parity follow product
implementation/build. The WoW API MCP endpoint was successfully queried; that
alone does not verify a shared source revision or a complete comparative corpus.

Use [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) for annotation behavior and
[WASM_BRIDGES.md](WASM_BRIDGES.md) for narrow algorithm boundaries. Report complete,
partial, conflict, skipped and NotEvaluated outcomes at their actual scope.
