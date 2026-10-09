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

W14 now publishes explicit final physical Main deltas through the retained native
owner and the complete graph producer chain. It reuses unchanged parsed trees
and conservatively reindexes semantics before extracting target-bound reports.
The exact retained publication record remains the base for idempotent retries;
inactive validation and current CAS preserve older leased readers. Exact Library
and configuration replacement retain cold analysis paths. Standalone/package
durable updates and dependency-specific fact reuse remain open. W15 has executable
persistent exact generation pins in a separately selected physical v2 profile,
with frozen v1 reopening unchanged. A separately selected v3 slice now supplies
operation release, complete bounded inline root closure, exact policy CAS and
guarded transactional GC through store/service APIs. Shared/current/leased/pinned
data survives; released operation evidence and GC receipts survive collection and
reopen. W16 recovery and object/epoch/platform gates remain next; full W14/E2 and
W15 acceptance remain open. See
[PROJECT_GC.md](../crates/wow-store/PROJECT_GC.md).

The roots checkpoint passed workspace policy, fmt, check, strict Clippy, tests
(864 passed, 1 ignored, 105 targets), rustdoc and build on 2026-10-09. The ignored
consumer check and Gethe/Ketho/runtime gates remain open.

The subsequent release/inline-GC checkpoint passed workspace policy, fmt, check,
strict Clippy, tests (866 passed, 1 ignored, 106 targets), rustdoc and build on
2026-10-09. Gates use an isolated build target outside the workspace; another
project's shared Cargo target remains independent. Platform faults and full W15
acceptance stay open.

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
| [W13 / PR 81](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/81) | Full live-pair acceptance | Physical Lua, standalone TOC/XML and declared-package native replay plus coherent leased acquisition are executable; full acceptance remains open |
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
this predecessor left CLI wiring, TOC/XML/package replay and W13 acceptance open.
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

## W13 CLI and W14 update controls checkpoint

The physical-input predecessor `7c9f66ef068558974da441c29d0bd014069c8550`
was published and read back with all 13 changed blob identities. The public
`wow project publish/read/reconcile` transport now calls one service operation
per command. Publication retains the original native publisher through the same
complete graph producer chain, then validates inactive read-back and exact CAS.
Read restores and projects one actual pair under a held lease; reconciliation
observes the original journal without repeating effects. See
[LIVE_PROJECT.md](../apps/wow/LIVE_PROJECT.md).

The full service regression found noncanonical bridge fixture lists and
hook/library rule ordering in earlier constructors. The strict validators remain
unchanged; fixed constructors use bridge4, hooks5 and library3 profiles. All three
partitions are retained, with no negative authority.

W14 adds explicit Keep/Replace/Clear intent and caller cancellation through
file-operation/analyzer/publication boundaries. Empty legacy vectors retain
Library; explicit Clear/empty replacement reaches the current E0 mandatory-Library
validator and rejects, preserving current. Nonempty replacement matches an
independently built final-state snapshot. Full incremental reuse, exact Library
binding in project generation, graph removal closure and durable update routing
remain open; see [UPDATE_MODEL.md](../crates/wow-project/UPDATE_MODEL.md).

Fresh workspace check, strict Clippy, tests (847 passed, 1 ignored, 103 targets),
rustdoc and build passed on 2026-10-09. CLI transport lint/build also passed after
the final help guard. Remaining W13 loader-plan replay, fixture freeze, genuine
backend probes, source/Ketho parity, real-addon/runtime and full E2 acceptance
stay open. Gethe materialization follows the product implementation/build stage.

## W13 standalone TOC/XML replay checkpoint

The CLI/update-controls predecessor `6e864fc272150913ef5e96d9bdde75eabd27d4a7`
was published and read back with all 24 changed blob identities. Standalone replay
now retains selected TOC/context and all consumed TOC/XML documents in explicit
native replay v2. A typed retained source port reuses the existing loader, including
bounded parsing, missing/excluded/unresolved decisions, inline Lua and XML indexes.
Read-back requires the original plan digest and exact project/analyzer identities.
Surplus and substituted archives reject; physical v1 encoding remains unchanged.
Legacy physical store epochs reopen against their exact original catalog, while
v2 loader writes require a newly initialized store. No migration is performed.

Fresh policy, fmt, workspace check, strict Clippy, tests (849 passed, 1 ignored,
103 targets), rustdoc and build passed on 2026-10-09. The regression removes source
files before hydrate and the complete service producer chain, then reopens the
actual stored pair under Exact. Multi-package replay remains next: Main flattening
currently omits unreachable packages, so replay must retain their actual captured
bytes and variant sources before reconstructing the package closure. W13/E2
acceptance, generation Library binding, retention/backup and source/runtime gates
remain open. Gethe preparation still follows product implementation/build.

## W13 declared-package replay checkpoint

Standalone predecessor `c317f9aaa59feb4b0e74665e5a69fd14f029ac48` was published
and read back with all 12 changed blob identities. Native replay v3 now retains
all selected package closures and variant TOC bytes, including unreachable Lua
that Main flattening omits. The original package loader rebuilds dependencies,
reachability, order and namespaced Main; all original plan and semantic IDs must
match. Package bytes are archived once and never executed. Prior v1/v2 epochs
retain exact catalogs and reject unsupported v3 writes without current mutation.

The full service regression exposed a stale source-edge crosswalk: accepted
edges belonged to the input generation, while later publication rebound endpoints
to the final generation. The service now uses its existing node/edge rebinder for
package ownership, dependency and load edges, preserving relation/confidence/evidence
and verifying final presence. No graph validator was relaxed.

Policy, fmt, workspace check, strict Clippy, tests (850 passed, 1 ignored,
103 targets), rustdoc and build passed on 2026-10-09. The native regression removes
source directories before full publication and Exact reopen and rejects variant,
unreachable-corpus, root-selection and foreign-Main substitutions. Full W13/E2
acceptance and external source/runtime checks remain open. The next functional
slice at this predecessor was W14 exact Library binding with explicit legacy
replay compatibility, before incremental invalidation and W15 retention/GC.

## W14 exact analyzer-input generation checkpoint

Package predecessor `144761fa45c4ffa17393d1ac40caacc51c55730b` was published
and read back with all 15 changed blobs. New publishers now use generation recipe
v2 with exact sorted Library snapshot IDs and the function-call-facts flag. Both
inputs validate before analysis and against the final analyzer binding. Library
replacement reaches the same IDs as an independent final-state build; Main bytes
remain unchanged while generation-bound file records and handles rebind.

Native replay v4 covers physical, standalone and package inputs. Frozen native
v1/v2/v3 fixtures were captured before this change and reproduce their original
project/analyzer/generation IDs. Old epochs open only under exact V1/V2/V3
catalogs and reject v4 writes without changing current or epoch identity.
No migration or legacy recipe widening is provided.

Policy, fmt, workspace check, strict Clippy, tests (852 passed, 1 ignored,
103 targets), rustdoc and build passed on 2026-10-09. The next functional slice
is an actual native analyzer update batch and exact unchanged-input reuse,
followed by durable W13 update publication/removal closure, W15 retention and W16
recovery. Full W14/E2 acceptance and the deferred Gethe/Ketho/runtime gates remain
open. Current Gethe `live` was re-resolved to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`; it was not materialized.

## W14 retained physical native-analysis checkpoint

Predecessor `3a7ed0fb83d984d0c43ab896d23fe635ef33eb89` was published and read
back with all 22 changed blobs. Physical updates now derive a complete native
Main delta under an exact previous-snapshot precondition. Only changed files
are reparsed in the existing syntax and semantic lanes. Unchanged Main/Library
trees remain retained, while complete semantic reindexing recomputes dependent
outputs and existing collectors extract fresh target-bound identities.

Safe green-node pointer checks prove unchanged allocation retention; full cold
report parity covers add/update/remove, same-session remove/readd and optional
function facts. Exact symbol lookup observes the original declaration and its
removal from the unchanged consumer's session. Project update output matches an
independent final-state publication. Errors/cancellation discard the mutable
cache and retain current; NoChange returns the same immutable Arc. Library or
configuration changes reopen the owner; standalone/package/virtual replay stays
on the existing cold path and frozen original IDs remain valid.

Policy, fmt, workspace check, strict Clippy, tests (859 passed, 1 ignored,
104 targets), rustdoc and build passed on 2026-10-09. The final declaration-removal
proof also passed separately. Dependency-specific fact reuse, durable full-graph
update/removal closure, retention/recovery and full W14/E2 acceptance remain
open. Gethe materialization, Ketho parity and named-client runtime gates remain
NotEvaluated until the product implementation/build stage is complete.

## W14 durable full-graph physical update checkpoint

Predecessor `a08ab335e2978f64e8a8c8032dd3524ebc9449a2` was published and read
back with all 15 changed blobs. `wow project update` now transports explicit
final physical inputs, exact expected-current/operation guards and mandatory
Library Keep/Replace/Clear intent to the service. NoChange validates the graph
project/generation selector and consumes no operation ID. Changed operations use
the existing prepare/read-back/validate/activate pipeline and its canonical
fingerprint. A replay after later activation returns the original receipt;
substitution of its target conflicts, without silently rebasing the original base.

Full graph equality against independently built final inputs verifies removal
closure over all producer partitions, not only source files. Old leased readers
and Exact/historical acquisition retain their original project/graph identities.
Library Keep rejects contradictory final inputs; Replace binds the exact new set;
Clear still rejects through the current mandatory-Library policy.

The real Main-call scenario also repaired a previously unexercised producer type
mismatch: generic recognizer assertions include an observation ID, which is not
a source EvidenceId. Source-function-calls v2 verifies that exact assertion
handoff, forwards original typed call/caller/target supports and retains the
observation receipt. Coverage and confidence are not promoted.

Fresh 2026-10-09 workspace policy, fmt, check, strict Clippy, tests (860 passed,
1 ignored, 104 targets), rustdoc and build passed. The additional service Library
intent test passed separately after the full suite. CLI smoke verified help,
mandatory Library intent, duplicate/initialization rejection and no store creation
for invalid input. Full W14/E2, loader/package updates, dependency-specific fact
reuse, retention/recovery and Gethe/Ketho/runtime acceptance remain open.
