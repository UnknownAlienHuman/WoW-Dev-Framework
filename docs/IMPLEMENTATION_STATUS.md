# Implementation status and update policy

**Current census:** [PROJECT_COMPLETION_MATRIX.md](PROJECT_COMPLETION_MATRIX.md).
**Execution findings:** [AUDIT_2026-09-19.md](AUDIT_2026-09-19.md).
**Normative order:** [IMPLEMENTATION_HANDOFF.md](IMPLEMENTATION_HANDOFF.md), I0–I7.

This ledger describes executable scope, not completion of the E0–E7 architecture.
Do not use old bootstrap or owner README status lines as evidence that existing
Rust implementations must be written again.

## Active workspace and unaccepted scope

The root Cargo workspace has **16 members**: `apps/wow`,
`apps/wow-reference-builder`, `wow-core`, `wow-store`, `wow-reference`,
`wow-annotations`, `wow-emmy`, `wow-project`, `wow-rules`, `wow-service`,
`wow-graph`, `wow-recognizers`, `xtask`, `wow-render-contract`,
`wow-ketho-literals` and the guest in `modules/ketho-literals`.
The separate `bridges/literal-host` workspace has dedicated CI and is deliberately
excluded from the root workspace.

Real analyzer, project-generation, diagnostic, service, persistence, graph and
recognizer slices exist. E0-B–E0-F normative checksum manifests nevertheless
retain required pending/null fields. Partial executable state is not complete
package acceptance. This correction does not manufacture missing evidence or
waive the earlier fixture-freeze policy.

`apps/wow` implements one-shot `status/check` over explicit materialized input
through real project/analyzer/rule owners; see
[local input](../apps/wow/LOCAL_INPUT.md). `apps/wow-reference-builder` is now an
active service-only workspace frontend with confined staging, independent read-back
and durable filesystem-effect reconciliation. Neither focused implementation is
full package acceptance. Search, context, optional external bridge and supported
release owners remain planned.

The managed-checkout and checkout-free GitHub API lanes now implement exact
`auto`/`prompt`/`never` plans, immutable manifests/snapshots and durable replay.
The API lane resolves one branch once and reads the exact commit, recursive tree and
selected blobs under finite request and admitted-body limits. Remaining W08 work is
live fault/platform acceptance and lower-layer hostile-network qualification;
W09 load closure is now an input to the W10 virtual-semantic path. Functional
development continues through bounded W10 consumers; existing full acceptance and
launch gates remain open.

The physical update path retains native parser state and publishes exact final
Main deltas through the complete graph producer chain. Bounded retention/GC,
recovery, portable original-source authority and guarded cross-epoch selection
are executable. The current functional frontier is W17: exact local platform-source
byte admission and retained-byte package/TOC/XML/Main specialization are implemented;
exact platform configuration/project/analyzer/graph binding follows. Broad package
acceptance remains open.

## W14 retained physical native analysis (2026-10-09)

`AnalyzerUpdateBatch` binds an exact previous Main snapshot and target generation
to a complete derived Add/Update/Remove set. `ProjectPublisher` retains private
native syntax and Main/Library semantic engines. Only changed texts re-enter the
parser; unchanged trees survive. Semantic indexes and all reports are rebuilt
conservatively from that exact retained corpus. No dependency-specific fact reuse
or performance bound is claimed. Library/configuration replacement and
standalone/package/virtual inputs use cold owners.

Failure after native mutation poisons the session; any project candidate failure
or cancellation discards it and retains the last immutable publication. NoChange
keeps its exact Arc without native work. Green allocation/mapping checks,
independent full report parity, same-session remove/readd, exact declaration
removal and original frozen replay IDs pass. Workspace policy, fmt, check,
strict Clippy, tests (859 passed, 1 ignored, 104 targets), rustdoc and build passed;
the strengthened declaration-removal proof passed separately afterward.
Durable full-graph update/removal closure, retention/recovery and complete W14/E2
or source/Ketho/runtime acceptance remain open.

## Source graph and XML handler route

`wow graph build` now composes file/load/XML/inheritance/mixin source proposals,
Main callable/call facts, and independent direct-call and XML script-assignment
and SavedVariables read/write recognizer partitions. The v16 receipt retains exact source/evidence, skipped-site
outcomes and final node/edge crosswalks. Inline XML handlers reuse existing syntax
units; named handlers use concrete Emmy signatures. Method and inherited handler
associations are Possible, not effective dispatch. The selected TOC seeds
account/character state roots; exact unshadowed Main global-slot accesses feed
the existing state recognizers and State axis. Typed string/integer/boolean
keys and transparent parentheses preserve distinct slot identities. Exact lexical
alias chains retain initializer evidence and Possible edges; any local rebinding
blocks the chain. Paths remain symbolic, not runtime values; ambiguous declarations
and unsupported access forms retain explicit outcomes. See
[GRAPH_BUILD.md](../apps/wow/GRAPH_BUILD.md). Existing graph reads cover entities,
neighbors, paths, subgraphs, axes and retained-support explanations. Remaining
recognizer families, effective XML receiver/runtime dispatch semantics and coherent ProjectStore
publication are not completed by this source slice. Full E2 acceptance remains open.

## W11 signal, hook and library recognizer families

`wow-recognizers` now implements all eleven core structural families declared by the frozen E2-B rule profile: `core.signal.native_frame_event@1`, `core.signal.native_event_registry_bridge@1`, `core.signal.custom_registry_producer@1`, `core.signal.custom_registry_subscription@1`, `core.signal.cvar_callback@1`, `core.hook.set_script@1`, `core.hook.hook_script@1`, `core.hook.secure_posthook@1`, `core.library.libstub_require@1`, `core.library.libstub_new@1` and `core.library.embed@1`.

Each family is an independent declarative producer partition that consumes only generation-bound Emmy owner facts. No source text is reparsed, and no client build, Interface value, source revision, provider revision or toolchain version is asserted anywhere in these adapters. `wow-project` gained the matching universal graph registry entries (`native_event`, `custom_signal`, `cvar_key`, `library` entities and the registration/bridge/callback/script-hook/secure-hook/library relations) so producer batches pass registry validation.

Structural limits are preserved: an exact resolved receiver and an exact literal key are required for a confirmed relation, a dynamic receiver or key keeps the outcome Possible, and absent evidence is never reported as a clean negative. The library version string is retained as exact evidence, not as proof of a loaded revision. `Libs/` folder names alone never prove an embed relation.

Validated locally with `cargo check --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, `cargo test --workspace --all-targets --all-features` and `cargo xtask check`. These are focused owner checks; service-side partition publication, the `apps/wow` graph export lane, structural mutation fixtures and full E2 package acceptance remain open and are NotEvaluated.

Service-side publication of these eleven families is now executable.
`crates/wow-service/src/graph/build/signals.rs` publishes all six groups inside the
established owner order: native frame events, EventRegistry bridges, custom registry
producers and subscriptions, CVar callbacks, the script and secure-posthook families,
and the LibStub require, new and embed families. Each adapter crosswalks against the
accepted source graph that precedes it, so an unrecognized binding is an error rather
than a silent skip. The published partitions are projected into node and edge
crosswalks and bound into `BuiltGraph` and `GraphBuildResult` alongside the existing
owners.

The frozen E2-B contract declares 26 active rule ids. All are implemented and
published as functional core rule service ids: the core.lua.* triples (create_frame,
create_from_mixins, mixin_assignment), the eleven signal/hook/library families above and
the five selected-TOC, four XML and three state families described below. The CVar adapter now publishes the
exact matcher predicates it consumes (`has_cvar_key` and `exact_cvar_key`) and retains
an exact callback declaration in both fact support and the recognition receipt when
one resolves; a dynamic callback does not fabricate an endpoint. The `core.state.*`
adapters reuse selected TOC facts and the admitted legacy state partition, retaining
the original analyzer/span/support validation rather than running another analysis.

Twenty-six published rule ids are not full acceptance. E2 acceptance, the E0
fixtures/identity/checksum gates, real-addon runtime validation, the Ketho parity
baseline, the complete `apps/wow` CLI surface and coherent ProjectStore publication
remain open and are NotEvaluated.

A subsequent W11 semantic-closure audit tightened three owner boundaries without
widening rule authority: native-frame-event support now validates exact source handles,
digests, generations and evidence provenance; custom-signal graph proposals preserve
the matcher support/coverage closure and include the unique compatible producer support;
script/hook facts and graph relations retain every exact resolved receiver, target,
handler and callback declaration used by the claim. Dynamic or unresolved endpoints
remain unprojected/Possible rather than receiving fabricated support. The affected
producer/fact/evaluation profiles are versioned. These functional checkpoints use
formatting, affected-chain `cargo check`, strict Clippy and repository policy only;
the planned full rule-specific test and fixture-freeze phase remains NotEvaluated.

## W11 selected-TOC core families

The service now publishes `core.toc.package`, `core.toc.file_order`,
`core.toc.dependencies`, `core.toc.load_on_demand` and `core.toc.saved_variables`,
all at rule version 1, in independent partitions. `wow-project` owns parsing,
selection and exact generation-bound facts; recognizers use data-only shadow
packs through the existing compiler/matcher. Missing acceptance fixture categories
stay empty in shadow packs. Default rollout still requires all four categories.

Each evaluation retains the normalized fact bundle, coverage records and complete
matcher output. Receipts map original facts to graph proposals; the service reads
back final node/edge identities after all producers. Equivalent entity assertions
merge all evidence. Unresolved dependencies and uncertain selections retain
omissions; repeated files do not manufacture an ordering DAG or runtime loading
claims. Package ownership deduplicates support when the package and TOC file
share the same whole-file observation.

The source projection profile is `wow-project/source-load-proposals/19`, registry
version 15; the build result is `wow-service/graph-build-result/16` and request
shape remains `/9`. A named-package fixture verifies all five TOC families through
real project publication, matcher, partition replacement and final crosswalk.
See [PROJECT_WORK_MAP.md](PROJECT_WORK_MAP.md) for exact local verification and
the remaining acceptance gates.

## W11 XML core families

The service publishes `core.xml.template`, `core.xml.object`,
`core.xml.inherits` and `core.xml.script`, all at rule version 1. Object declaration
and explicit parentage have separate partitions; parentage consumes already
materialized Object nodes. TOC package/variant ownership precedes these phases.
Typed `wow-project` records feed data-only shadow packs through the existing
compiler/matcher, with exact support, bundles, outcomes, receipts and final
`xml_topology` crosswalks. Default rollout still requires every fixture category.

Explicit ParentOf is parent to child and never inferred from lexical nesting.
Inherits and ReferencesTemplate preserve separate meanings. Script bindings reuse
accepted source proposals and the same Emmy lookup session. Source-only Script
chunks stay analyzed without callback bindings. Missing or ambiguous targets
remain omissions. Partial captured-structure coverage keeps matcher assertions
Possible; the Object axis requires explicit IncludePossible traversal. Generic
XML elements do not certify runtime classes or effective callback dispatch.

Fresh whole-workspace checks, strict Clippy, rustdoc, formatting, native repository
policy and tests passed on 2026-10-09: 837 passed, 1 ignored, 102 test targets.
The integration fixture exercises all five publication phases and final IDs,
explicit versus lexical parentage, confidence filtering, inheritance, named/inline
handlers, unresolved/ambiguous targets and source-only chunks. This is functional
pipeline verification; full E2-B and per-rule fixture freeze remain open.


## W11 state core families

`core.state.saved_variable_root`, `core.state.literal_path_read` and
`core.state.literal_path_write` publish at rule version 1. Root publication reuses
the selected TOC adapter. Read/write publication consumes exact admitted legacy
accesses and binds their complete source-handle/evidence vectors before executing
the existing matcher. Path keys remain the owner's Identifier root plus String
path; equivalent paths merge support. Recognition retains bundles, outputs,
admission partition digest and final proposal crosswalks. All source-level
associations remain Partial/Possible; empty target shapes are NotEvaluated.

Fresh whole-workspace build, check, strict Clippy, rustdoc, format, native policy
and tests passed on 2026-10-09 (838 passed, 1 ignored, 102 targets). The integration
fixture verifies exact account/character roots, alias support, dynamic omissions,
local shadowing, merged paths and rejection of a substituted source handle at
unchanged endpoints/evidence. No runtime persistence, public CLI acceptance,
per-rule fixture freeze or full E2-B acceptance follows.

## Explicit graph source read-back

`wow graph explain --bundle ... --source-root <Main-root>` joins the existing
resolved evidence to exact local bytes through `wow-project`'s confined reader.
Whole-file hashes precede UTF-8 span/excerpt output; drift, missing/unsafe sources
and all record/read/output limits remain explicit. The root never enters semantic
output and no analyzer is opened. Metadata-only modes keep their prior encodings.
See [GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md). This is selected-source
read-back, not authenticated provenance, an atomic filesystem snapshot, complete
context generation or coherent ProjectStore publication. Full acceptance is open.

## Native source and annotation boundary

Ketho/vscode-wow-api remains the annotation implementation donor; the
[port map](KETHO_RUST_PORT.md) is the route for changes. Current Gethe source,
resolved once per operation, supplies Blizzard facts. Source and generated Lua
are data and must not be executed.

Native source/model/scalar/correction/alias handling, library projection,
receiver/type/catalog/inheritance/navigation slices and bounded literal rendering
already exist. Exact native scope is in their code, focused tests and port/usage
contracts. Scoped dual-consumer tests, source-bundle validation and a Wasm swap
probe do not establish universal corpus parity, platform truth or E1 acceptance.

The development driver is `crates/wow-annotations/examples/native_library.rs`;
use its documented explicit inputs rather than inventing a public `wow` command.
Raw source observations remain unchanged by reviewed projections. Missing,
conflicting, failed, omitted or unsupported inputs remain partial/NotEvaluated,
never `any`, a clean negative, or authoritative runtime safety.

Legacy v1 API/topology readers remain compatibility boundaries. The retired
interpreter producers are not a fallback, and the annotation TOC loader is not a
full E2 TOC/XML/load/project index. Do not restore Python, embedded interpreters,
wrappers or interpreter-based project tests.

## Annotation artifact persistence boundary

`wow-annotations::artifact` now owns only canonical artifact bytes, identity and a
store-neutral publication selector. The crate no longer has a regular dependency
on `wow-store`. `wow-service::annotation_admin` preserves the existing
`wow.annotation.artifact` object kind/schema and `annotation.current` catalog while
owning exact operation/request identity, CAS publication, mandatory read-back,
replay, retention, integrity status and bounded GC. No raw SQL or mutable Store
handle enters the annotation or application contract.

This is a boundary repair and durable publication slice, not full E1-C/E1-D
Reference Pack assembly or package acceptance. Existing tests were split between
pure artifact ownership and service orchestration; broad execution remains subject
to the repository's later acceptance stage.

## Implemented maintenance commands

See [xtask commands and limits](../tools/xtask/README.md):

```sh
cargo xtask check
cargo xtask sync-skill --check
cargo xtask sync-skill --write
cargo xtask check-source /path/to/checkout live
cargo xtask update-source /path/to/checkout live --expected-head <observed-SHA>
cargo xtask materialize-source /path/to/request.json
cargo xtask materialize-source-api /path/to/request.json
cargo xtask manifest /path/to/checkout HEAD live /path/to/new-manifest.json
cargo xtask verify-manifest /path/to/manifest.json /path/to/checkout HEAD
cargo xtask verify-library /path/to/native-output --require-input-complete
```

These are internal maintenance/development operations, not a replacement product
service or public CLI. Repository checks enforce native-only assets/invocations,
JSON syntax, **unique decoded object keys**, and synchronized skill copies.
They do not yet prove full contract-schema/ID/dependency/fixture/Markdown closure.
Eight duplicate-key regression groups were added at implementation checkpoint
`c10579d359b5f0044fc6fcdfef3b353c5caa65dc` without changing dependencies.

## Source update and provenance policy

Choose an explicit flavor/moving source selector at operation start and record
one exact source revision, version and manifest. Re-resolve on a new operation.
A recorded commit/build/toolchain is evidence for a run, not permanently embedded
current truth. Offline or unavailable remote observation is `unverified-current`.

`check-source` is read-only. `update-source` is an explicitly authorized guarded
fast-forward for an existing exclusively owned standalone checkout; see
[SOURCE_CHECKOUT_UPDATES.md](SOURCE_CHECKOUT_UPDATES.md). The separate
`materialize-source` implements exact `auto`/`prompt`/`never` plans for a managed
Git checkout, guarded updates, immutable manifests and durable recovery; see
[SOURCE_MATERIALIZATION.md](SOURCE_MATERIALIZATION.md). The separate
`materialize-source-api` fallback resolves one public GitHub branch once, then reads
the exact commit, recursive tree and selected blobs by object ID under finite
request/admitted-body limits; see
[SOURCE_API_MATERIALIZATION.md](SOURCE_API_MATERIALIZATION.md). Neither path may
reset/stash operator changes, mutate an unexpected root or blindly retry an
uncertain effect.

Both lanes still trust an operator-approved GitHub HTTPS origin. Their body limits
do not prove a hard bound on all lower-layer network traffic, authorship, license
or semantic compatibility. Background update scheduling remains unimplemented.
Optional operator-only context stays advisory and disabled by default.

## Checkout-free API snapshot checkpoint

W08 also provides `materialize-source-api`: an explicit GitHub REST fallback that
binds one observed branch to an exact commit/tree, independently admits each
selected blob, re-hashes Git object identity and publishes an immutable snapshot
plus the ordinary source manifest. Commit `b6ea5981b2372468e332b30c0bdd697ce3b52301`
has checked tree `312caed69e65a5b907dfa219e2fb3aa739837ece`; focused run
`36826290606` passed formatting, strict `xtask` Clippy and repository policy on
Linux/Rust 1.98.1. Tests, a live donor invocation, Windows and network/process
fault injection were not run.

## CI and exact evidence

Root CI exercises Linux/Windows repository policy, formatting, locked workspace
check, strict Clippy, debug/release tests and rustdoc. Other lanes exercise updated
dependencies, rolling parser consumers, isolated literal-host/guest swap/rollback
and semantic annotation consumers. The rolling parser lane deliberately excludes
semantic analyzer consumers; it is not proof that the entire analyzer adapter was
tested against a new upstream revision.

The current-source workflow resolves explicit Gethe/Ketho inputs and retains
source identities, counts, output bytes/hashes and reports. Source admission
failures are errors; explicit projection omissions remain partial, not a semantic
pass or an R0 product evaluation.

Baseline source `e2c74314bb7ccde4a9b48c049dfeacc157259960` passed
[CI 34735568141](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/34735568141).
The duplicate-key implementation is associated with
[CI 35487232004](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/35487232004).
The focused managed-source checkpoint passed formatting, strict xtask Clippy and
repository policy in
[CI 36821527175](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36821527175)
and was published from its exact retained tree by
[CI 36821677002](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36821677002).
No tests, Windows runtime or live source acquisition were executed in that checkpoint.
Use each run's exact head and actual conclusions; later documentation commits have
their own CI. A source archive and successful CI are not a release signature,
full fixture-freeze acceptance, real-addon runtime test or supported installation.

## Updating this ledger

Work sequentially in `main`, one owned slice at a time, without task branches or
worktrees. Read owner contracts and prerequisites; execute applicable checks;
publish without force and read back exact remote HEAD/blob identities. Record
failed, skipped and unavailable checks separately. Do not claim a local cargo
run when validation came from Actions.

Update the machine inventory, current matrix and affected owner status together
when executable scope changes. An Implemented/Complete gate requires its full
reviewed fixtures and evidence, not only code presence. Historical annotation
checkpoint details remain available in Git history and their focused port/usage
documents; they must not be recycled as a current backlog without checking code.

## Verified code checkpoint

All eight jobs in CI run `35487232004` concluded success for
`c10579d359b5f0044fc6fcdfef3b353c5caa65dc`: Linux/Windows locked checks,
strict Clippy, debug/release tests and rustdoc; updated dependencies; rolling
parser; Linux/Windows semantic consumers; Linux/Windows Wasm swap/rollback.
This is code-checkpoint evidence, not a full package or public launch certificate.

## Explicit disk inputs

The one-shot CLI also accepts `wow-service/local-project-files/1`: pinned metadata
artifacts plus explicitly listed Main/Library files below the config directory.
`wow-project::disk` owns bounded read-only acquisition; both input modes use the
same owner composition. See [FILES_INPUT.md](../apps/wow/FILES_INPUT.md).
The separate `wow-service/local-project-toc/1` mode adds selected TOC/XML external-
file expansion, a generation-bound load receipt and explicit partial blockers;
see [TOC_INPUT.md](../apps/wow/TOC_INPUT.md). There is no source scan, live-rule expansion, full E2-C or R0 acceptance.
XML now has a source-backed syntax index and exact inline units analyzed in the same pinned
EmmyLua Main/Library session. Check returns mapped syntax/semantic diagnostics and direct
member/call facts. `wow.api.exists@1` consumes only exact-static XML facts with complete unit
coverage; implicit receivers, inheritance materialization and runtime dispatch remain partial.
See [XML_ANALYSIS.md](../apps/wow/XML_ANALYSIS.md). Local XML parent/inheritance references are
linked as described below, without inherited receiver materialization.

Selected-TOC acquisition now preflights package-wide target filters before any
Lua/XML descendant read. Included declarations enter the v3 load receipt; excluded
and unresolved packages return distinct typed local-operation outcomes without an
analyzer snapshot. See [TOC_CONTEXT.md](../apps/wow/TOC_CONTEXT.md#package-filters-before-source-acquisition).

## XML local references

Selected-TOC input links XML parent/inheritance names across the captured closure,
retaining duplicate candidates, load order, source anchors and separate parent/
inheritance cycle components. Link issues enter ordinary XML-scoped findings.
See [XML_REFERENCES.md](../apps/wow/XML_REFERENCES.md). This is local source linking,
not XSD object validation, Lua binding, inheritance flattening or runtime acceptance.

## XML to Lua declarations

Selected-TOC checks query `mixin`, `function` and direct/inherited-mixin `method`
candidates through the existing Emmy member-call session. Inherited handler sources
also have consuming-declaration contexts; method queries use that consumer's mixins.
Source provenance and incomplete ancestry remain explicit. Exact Main/Library declaration
locations and unresolved/ambiguous results enter owner receipts and ordinary
XML-scoped findings. See [XML_BINDINGS.md](../apps/wow/XML_BINDINGS.md).
No new compiler session, synthetic source or callable/runtime proof is introduced;
receiver construction and inherited method precedence remain partial.

## W10 XML rule consumer checkpoint

Commit [`260cf430`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/260cf430a22fdc3f86da33a9c363946cc31aca00)
(tree `df3db6e01a804f44f57cbfe11f1a15cec1face60`) extends the existing W10 semantic
facts into `wow.api.exists@1`. A mixed `RuleScope` now selects physical file IDs and
normalized captured XML documents. The rule validates exact script-site authority,
document digest, mapped nonempty XML pieces, unique closed reference/call links and
unit-specific complete fact coverage before exact reference lookup. Authoritative
absence produces an XML-located finding; partial/conflicting/ambiguous evidence remains
`NotEvaluated`. `ProjectView` supplies generation-bound handles for retained XML sources,
and the service projects those handles without treating virtual URIs as physical files.

Focused Linux run
[`36966297105`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36966297105)
used Rust 1.99.0 and passed formatting, strict Clippy (`wow-project`, `wow-rules`,
`wow-service`, `wow-cli`, all targets/features, `-D warnings`), existing tests for
`wow-project`/`wow-rules`/`wow-service`, `cargo +stable xtask check`, exact file-set validation
and exact-tree fast-forward. Evidence artifact `11209657936` has ZIP digest
`fa3286abf5e39d03124ab7223d7bf4d5bc6857e987a739576378af79eb3631b1`; its embedded
checked tree and both retained SHA-256 checks were read back successfully. No Windows,
WoW runtime, real-addon command fixture or full workspace acceptance is claimed.

## Bounded graph projection

The graph owner now exposes `GraphSubgraphQuery` for exact-snapshot, multi-root
breadth-first neighborhoods with original evidence and explicit coverage and
truncation. See [SUBGRAPH_USAGE.md](../crates/wow-graph/SUBGRAPH_USAGE.md).
This completes the bounded subgraph code slice of E2-01, not its full acceptance.
Exact retained-support explanations now use `GraphExplainQuery`; see
[EXPLANATION_USAGE.md](../crates/wow-graph/EXPLANATION_USAGE.md). Registry-bound axes are described in
[AXIS_USAGE.md](../crates/wow-graph/AXIS_USAGE.md). Exact retained assertion
records and evidence resolution now extend this route; complete producer-chain
coverage, automatic conflict assessment and coherent live ProjectStore acquisition
remain open.

## Retained graph CLI

`wow graph entity|neighbors|subgraph|axis|explain|path` routes explicit retained partition-snapshot
and query artifacts through `wow-service` to the existing graph owner. It has
bounded JSON/file admission, exact identity guards and faithful typed status.
See [GRAPH_INPUT.md](../apps/wow/GRAPH_INPUT.md). It does not acquire ProjectStore,
produce a graph itself, or advance full E2/E3/E7 acceptance. Source construction
is the separate `wow graph build` route below.

## Source graph construction and export

`wow graph build` now materializes the same explicit project input as `wow check`
once, asks `wow-project` for exact source-file/direct-load proposals, and uses
`wow-graph` to validate/materialize an exportable partition snapshot. JSON retains
source handles, evidence, the source context/load receipt and path-to-node IDs;
snapshot output feeds existing graph-read commands. See [GRAPH_BUILD.md](../apps/wow/GRAPH_BUILD.md).
The source projection is Partial with no negative authority. Dependency packages,
XML runtime objects, non-call recognizers and coherent ProjectStore remain
unimplemented by this route. This is not full E2-C/E2-D or runtime acceptance.


Source graph export also includes distinct XML declarations, document ownership,
admitted local template inheritance and exact Main Lua mixin source links, with
materialized XML/Lua node maps and retained unresolved lookup receipts. The
Inheritance axis now accepts this registry. Library targets remain separate;
table construction, callback dispatch and runtime are not evaluated.


## Lua callable graph pipeline

`wow graph build` now requests bounded callable/call facts from the existing Emmy
session, projects source-owned chunk/closure nodes and real evidence, executes the
existing `wow.direct-call` recognizer, and publishes its proposals into a separate
in-memory producer partition. Final function/edge ID crosswalks and original fact,
recognition and skipped-target receipts are exported together. Ordinary check/status
keep the previous analyzer mode. See [GRAPH_BUILD.md](../apps/wow/GRAPH_BUILD.md).
Dynamic/Library/inline-XML calls, direct self-edges, non-call recognizers, persistent
ProjectStore and full E2 acceptance remain open. No acceptance/checksum gate moves.

## Historical W11 first construction checkpoint

The first W11 family slice is executable on `main`. The Emmy function-call sidecar
profile is now `wow-emmy/function-call-facts/5`: each physical Main call retains
ordered exact argument spans, bounded literal values, exact source-backed reference
keys where available, and an optional exact callable key derived from the existing
global-access owner facts. The recognizer does not reread or reparse Lua.

`core.lua.create_frame@1` is routed through the existing declarative
pack/parser/compiler/matcher into the independent
`wow-recognizers.lua-construction` producer partition. It emits static `frame`
entities keyed by the exact call occurrence and `FactoryCreates` relations from the
captured caller function. Dynamic/unresolved arguments retain Possible or
NotEvaluated authority; no runtime frame existence, lifecycle, parent/template
application, protection, taint or execution claim is made. At that checkpoint,
`CreateFromMixins`, mixin assignment and the event/callback/hook/library families
were still unimplemented; the later W11 sections supersede that historical state.

At that checkpoint, `wow graph build` used `wow-service/graph-build-result/10` and
the source graph profile was `wow-project/source-load-proposals/10`; the request
shape remains unchanged. Focused Linux/Rust 1.99.0 checkpoint
`36969801633` passed formatting, strict Clippy for the complete affected chain,
repository policy and the exact seven-product-file boundary, then published
commit `4479cf333de8d747e3f53549ea3732598d42df84`. No tests, Windows run,
real-addon acceptance or package gate is claimed.

## Retained source evidence reads

Current graph-build v16 bundles feed every graph read directly. Explain joins the
validated partition owner to project-admitted file/handle metadata and a
core-validated evidence catalog, with bounded shared derivation expansion.
See [GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md). Source bytes are not
reopened. W12 adds producer derivation and reported conflict records; complete
automatic conflict assessment, sidecar replay, runtime and coherent live
ProjectStore remain open. Record-free v1 snapshots retain their encodings and
identities; record-bearing snapshots use v2. Older build receipts can still supply
their extracted bare snapshot, without relabeling the receipt as v16.

## Manifested retained ProjectStore code slice

`wow-store::project` now supplies one registered WAL epoch, immutable partition
versions, complete generation membership, pinned read transactions, durable
operation phases and inactive/read-back/validation/current-CAS publication.
`wow graph publish` and `reconcile` are explicit service operations; all six graph
reads can select one current/exact stored generation. Graph and project adapters
restore and validate current retained bundle data without another analyzer session.
See [GRAPH_STORE.md](../apps/wow/GRAPH_STORE.md). This supersedes the earlier
blanket “no persistent ProjectStore” descriptions for this narrow storage slice,
not full E2-D acceptance: live project views, incremental invalidation, retention,
GC, backup/restore, epoch replacement and crash/power-loss acceptance remain open.
Only the existing internal project-to-store dependency is activated; no tests or
external dependency versions are changed. Full launch gates stay unchanged.

## W12 retained assertion records

`GraphProposalBatch::with_assertion_records` binds exact local/cross-producer
inputs, rebuttals, missing prerequisites and reported unresolved conflicts.
Partition publication validates exact batch identities, input scope, acyclic
closure, bounded depth and confidence monotonicity. It retains all participants
and conservatively downgrades incident relation coverage. It does not certify the
authenticity or completeness of a producer's conflict judgment.

Source19 and legacy/core state producers attach real graph prerequisites without
another parser or analyzer. The service state explanation closes through source
entities and captured files. Explanations return additional assertion supports,
actual derivation/conflict observations and an explicit `derivation_complete`
flag under scan/support/output/depth budgets. Missing legacy records and truncated
chains retain boundaries; conflict assessment remains unavailable unless provided
by a future complete owner assessment.

Record-free v1 batch/snapshot identities remain unchanged. Record-bearing batches
and snapshots use v2, with matching stored header/producer schemas; the loader
accepts and validates both. Explanation and resolved-explanation payloads use v2.
Current bundle import requires result16/source19/request9; legacy receipts remain
available through extracted bare snapshots. See
[GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md).

Fresh workspace gates on 2026-10-09 passed: policy, fmt, check, strict Clippy,
842 tests passed (1 ignored, 103 targets), rustdoc and build. Actual publication
regressions cover stale batches, cycles, confidence promotion, conflict retention,
canonical identities, depth truncation and v1/v2 stored read-back. Full W12/E2
acceptance and W13 live acquisition remain open.

## W13 physical-input live pair

`ProjectReplay` and `ProjectPublicationBundle` now preserve exact Main/Library
inputs and configuration independently of serialized analyzer state. Native
read-back runs the approved existing ProjectPublisher, compares original semantic
IDs and recomputes source19 proposals against the stored graph. Logical member
versions derive a publication set before the store generation; store identities
cannot enter project/graph recipes.

`wow-service::live_project::LiveProjectStore` publishes and acquires an actual
ProjectView/GraphPartitionSnapshot pair through a distinct registered epoch,
inactive read-back validation and exact current CAS. A `LiveProjectRead` retains
one transaction and generation lease through all owner checks. Missing/mutated
records and mixed project/graph inputs fail; old readers survive current advancement.

The first admitted profile is physical Lua configuration. Loader-plan inputs
return explicit DeferredCapability, and public CLI wiring remains open. Other
recognizer sidecars are not independently replayed or certified. See
[REPLAY_PUBLICATION.md](../crates/wow-project/REPLAY_PUBLICATION.md).
Fresh whole-workspace policy, fmt, check, strict Clippy, tests (844 passed,
1 ignored, 103 targets), rustdoc and build passed on 2026-10-09. Full W13/E2,
incremental, crash, backup/GC, source-parity and runtime acceptance remain open.

## W13 CLI / W14 controls checkpoint (2026-10-09)

`wow project publish/read/reconcile` now exposes the physical Lua live-pair service
through one service operation per command. The original native publisher is
retained through the full existing graph chain. Public reads project exact IDs
and counts after leased native replay; reconciliation never repeats effects.
See [LIVE_PROJECT.md](../apps/wow/LIVE_PROJECT.md). The service smoke repaired
noncanonical constructors in bridge4/hooks5/library3 without loosening validators.

`apply_update_cancellable` threads caller cancellation through file operations,
analysis and publication, including NoChange. Explicit Keep/Replace/Clear preserves
legacy empty-as-Keep conversion. Current E0 policy rejects Clear/empty replacement
because an explicit Library remains mandatory. Full rebuild, generation Library
binding, real incremental reuse and durable update/removal closure remain open.

Fresh workspace check, strict Clippy, tests (847 passed, 1 ignored, 103 targets),
rustdoc and build passed; final transport lint/build passed separately. Loader-plan
replay, fixture freeze, genuine backend probes, Ketho/source and runtime acceptance
remain open. See [PROJECT_WORK_MAP.md](PROJECT_WORK_MAP.md) for the dependency order.

## W13 standalone TOC/XML replay checkpoint (2026-10-09)

Selected-TOC configurations now pass the existing public live-project service/CLI
path. Native replay v2 captures selection context and exact consumed TOC/XML bytes
alongside Main/Library. A typed retained source port runs the same bounded loader,
then requires the original load-plan digest and project/analyzer identities.
Missing, excluded and unresolved decisions remain explicit; surplus or substituted
bytes/context/schema reject. No loader receipt or analyzer session is deserialized.

Physical replay keeps v1 encoding. Strict legacy-catalog reopen preserves existing
physical epoch and membership IDs; v2 loader archives require a newly initialized
store, without migration. Full service TOC/XML composition, source-directory
removal, native hydrate and Exact reopen are exercised by the new regression.
Fresh workspace policy, fmt, check, strict Clippy, tests (849 passed, 1 ignored,
103 targets), rustdoc and build passed. Package replay, generation Library binding,
incremental reuse, retention/backup and full W13/E2/source/runtime acceptance remain
open. See [REPLAY_PUBLICATION.md](../crates/wow-project/REPLAY_PUBLICATION.md).

## W13 declared-package replay checkpoint (2026-10-09)

Native/storage replay v3 captures every selected package closure, all declared TOC
variants and Main fixture references. Unreachable Lua remains retained independently
of analyzer Main. Package bytes are stored once; the original bounded package
loader rebuilds dependencies, reachability, order and collision-free Main. Every
selected, package/Main and project/analyzer identity must reproduce after source
removal. Old v1/v2 encodings and exact epoch catalogs remain compatible; v3 writes
require an epoch that admits v3, without migration or widening an older catalog.

The complete service package path also repairs source ownership/dependency/load
crosswalks: accepted edge endpoints are rebound with the existing helper to the
final graph generation, preserving evidence/confidence and checking final presence.
Fresh policy, fmt, workspace check, strict Clippy, tests (850 passed, 1 ignored,
103 targets), rustdoc and build passed. Native full-service publication and stored
Exact reopen run after source deletion; corpus, variant, root and foreign-Main
substitution guards pass. Full W13/E2 acceptance, exact Library generation binding,
incremental reuse, retention/recovery and source/runtime gates remain open.

## W11 semantic-repair checkpoint

A read-through after the first W11 publication found positive-path defects that
ordinary workspace CI did not exercise: colon receivers were reused as positional
arguments, `RegisterUnitEvent` argument order was reversed, hook callable keys did
not match the production analyzer profile, library matcher fact IDs were confused
with call IDs, and library graph proposals lacked mandatory support. The bounded
repair route and remaining acceptance order are recorded in
[W11_SEMANTIC_REPAIR_MAP_2026-10-09.md](W11_SEMANTIC_REPAIR_MAP_2026-10-09.md).

The repaired slice remains structural static evidence. It does not establish
runtime event delivery, frame lifecycle, callback safety, taint/combat legality,
loaded library revisions or complete E2 acceptance. Full per-rule pipeline cases,
fixture/checksum freeze, the remaining XML/state rules, `apps/wow` real-addon
acceptance and the mandatory final WoW API Ketho MCP comparison remain open until
separately executed and recorded.

## W14 exact analyzer-input generation checkpoint (2026-10-09)

New generation recipe v2 binds sorted exact Library snapshot IDs and the
function-call-facts flag. Publisher and service target derivation use the same
inputs; analyzer and snapshot validators require their equality. Library-only
replacement now changes generation and rebinds source handles while retaining
Main bytes, matching independent final-state publication. Clear/empty replacement
still rejects through the E0 mandatory-Library validator and preserves current.

New archives use native/storage replay v4 with explicit generation version 2.
Frozen v1/v2/v3 archives reproduce their original v1 recipe and semantic IDs.
Exact legacy catalogs reopen without migration and reject v4 writes without
current or epoch mutation. Policy, fmt, check, strict Clippy, tests (852 passed,
1 ignored, 103 targets), rustdoc and build passed. Incremental analyzer batches,
reuse, durable updates/removal closure, retention/recovery and full W14/E2 or
Gethe/Ketho/runtime acceptance remain open.

## W14 durable full-graph physical update (2026-10-09)

The service now reads the exact retained expected publication, retains its native
publisher under the original lease, derives explicit final file operations and
Library intent, and composes every graph producer before validated current CAS.
The public `wow project update` command is a single service transport. NoChange
validates project/generation selection and has no publication effects or consumed
operation ID. A changed request's original canonical fingerprint controls retries,
including after a later activation; contradictory targets conflict.

Independent final-state full-graph equality covers Add/Update/Remove and exact
old-reader/Exact/historical identities. Keep/Replace/Clear remain distinct; empty
Library rejection preserves current. The Main-call path fixes the observation-ID
versus source-EvidenceId mismatch through source-function-calls v2 and exact
retained typed supports, without promoting confidence or coverage.

Workspace policy, fmt, check, strict Clippy, tests (860 passed, 1 ignored,
104 targets), rustdoc and build passed. One additional focused service Library
intent test passed afterward; CLI guard/help smoke also passed. Full W14/E2,
loader/package durable updates, dependency-specific fact reuse, retention/recovery
and source/runtime acceptance remain open. Current Gethe materialization still
follows the product implementation/build stage.

## W15 persistent generation roots (2026-10-09)

New live stores select physical profile v2 with SQL-authoritative, attributable
generation holds. Put/list/digest-guarded removal bind exact epoch and generation,
reject same-ID substitution, remain bounded and cancellable, and survive reopen.
Frozen v1 schemas, payload catalogs and epoch bytes are preserved; old service
stores remain readable and refuse the unsupported retention API without migration.
Commit observation rejects an active connection transaction as `OutcomeUnknown`,
including the existing publication path. See
[PROJECT_RETENTION.md](../crates/wow-store/PROJECT_RETENTION.md).

The store lifecycle and nine live-project service regressions passed, including
real native publication/reopen and all frozen legacy catalogs. Final workspace
policy, fmt, check, strict Clippy, tests (864 passed, 1 ignored, 105 targets),
rustdoc and build passed. This slice provides
no deletion. Operation-root release/tombstones, complete GC closure and plans,
backup/recovery, Windows/crash and full W15/E2 acceptance remain open.

## W15 explicit release and inline SQL GC (2026-10-09)

Physical v3 extends the frozen v1/v2 profiles with authoritative GC policy and
durable batch receipts. Explicit operation release preserves original request,
manifest and activation identities while stopping resumability. Planning validates
complete bounded inline closure, protects current/policy/pins/leases/operations
and their bases, then selects only unreachable generations/versions. Policy CAS,
lease revisions, writer changes and exact recheck reject stale/ABA plans. Deletions
and receipt share one immediate transaction; response loss reconciles honestly.
Store and native Project/Graph service lifecycle regressions pass. Final workspace
policy, fmt, check, strict Clippy, tests (866 passed, 1 ignored, 106 targets),
rustdoc and build passed. Object/epoch deletion, backup/recovery, process/power-loss and
Windows fault acceptance remain open; see
[PROJECT_GC.md](../crates/wow-store/PROJECT_GC.md).

## W16 recovery, verified backup and isolated restore (2026-10-09)

Exact admitted physical v1/v2/v3 stores now expose one held read-only recovery
snapshot with current/scope classifications, canonical operation dispositions and
unknown acknowledgment. Complete manifest-to-seal descriptors, payload digests,
validation/history, applicable pins/policy/GC receipts and SQLite/FK integrity are
checked. Missing legacy tables are not applicable; an unselected v3 policy is valid.

Native bounded SQLite backup includes committed WAL, retains original identities,
reopens independently and binds exact snapshot/body digests in a canonical manifest.
An explicit new private restore path receives its registry only after exact owner
capabilities for every generation. Native service replay checks all Project/Graph
pairs. Source current, prior receipts and older leased readers survive. The public
`wow project recover` command reports physical coverage without implicit effects.

Workspace policy, fmt, check, strict Clippy, tests (877 passed, 1 ignored,
106 targets), rustdoc and build pass. Quarantine,
same-root epoch replacement, supported migration, process termination, power loss,
sharing/cleanup faults and full W16/E2 acceptance remain open. See
[PROJECT_RECOVERY.md](../crates/wow-store/PROJECT_RECOVERY.md).

## W16 guarded physical-instance replacement (2026-10-09)

The outer registry now admits legacy unchanged EpochManifest bytes and a separate
v2 physical selector. A new operation-derived confined instance preserves exact
semantic epoch/generation/partition and native owner identities. Explicit
selector/current guards, native whole-snapshot copy, all owner replay, synced
staging, one selector replacement and independent read-back precede selection.
Root/instance locks, old files/readers and aggregate reader admission survive;
normal publication inside the new instance preserves the original replacement
receipt. Explicit staged or unknown-result reconciliation never repeats copying
or a committed selector replacement. Service wrappers compose the actual owners.

Seven store regressions, native Project/Graph replacement, Windows selector-sharing
failure and native termination after four completed durable boundaries pass.
Workspace policy, fmt, check, strict Clippy, tests (887 passed, 1 ignored,
107 targets), rustdoc and build pass. Quarantine, incompatible-epoch/schema
migration, interruption inside writes/OS calls, power loss, cleanup faults and full
W16/E2 or Gethe/Ketho/runtime acceptance remain open. See
[PROJECT_REGISTRY.md](../crates/wow-store/PROJECT_REGISTRY.md).

## W16 authoritative physical-instance quarantine (2026-10-09)

Outer registry schema 3 now holds the selected physical instance while preserving
frozen SQL schemas, Epoch/Generation/Partition and native owner IDs. Exact normal
selector, raw Current observation and bounded immutable evidence bind an explicit
operation. Damaged or unreadable Current never becomes absence. Normal new reads,
publication/activation, GC and backup reject the hold; already held native pairs
and root/instance locks survive. A standalone typed readonly inspection admits
the canonical registry/files without a writable SQL open and can explicitly hold
a damaged body/header. Exact selected receipts reconcile without another rename.

Thin native service wrappers expose inspection and readonly recovery. Seven store
regressions and one actual Project/Graph lifecycle pass, including damaged
pointer/history/payload/header, stale evidence, archive substitution, pointer
shape/budgets, old leases, absent Current in a selected instance and real Windows
sharing refusal with exact reconciliation. The nullable-field repair preserves
all valid legacy/Some encodings. Workspace policy, fmt, check, strict Clippy,
tests (895 passed, 1 ignored, 107 targets), strict rustdoc and build pass; final CLI
diagnostics passed focused strict Clippy/build. This initial hold checkpoint left
guarded restore to the follow-up below. Fine-grained/domain quarantine,
incompatible-epoch migrations, interruption inside
writes/OS calls, power loss, cleanup and full W16/E2/source/runtime acceptance
remain open. See [PROJECT_QUARANTINE.md](../crates/wow-store/PROJECT_QUARANTINE.md).

## W16 guarded quarantine restore and portable hold authority (2026-10-09)

`QuarantinedStore` stages/reopens an explicit independently verified backup and
activates only after compiled owners validate every included generation. A
versioned held-source intent binds the exact schema-3 selector/archive and raw
Current/evidence observations. Unreadable and malformed Pointer never become
absence; observational equality does not certify unchanged corrupt SQL bytes.
The native service replays actual Project/Graph pairs before selection.

Registry schemas 4/5 bind a sorted flat hold closure with exact archive records,
dependency subsets/revisions and finite count/byte admission. Shallow historical
selectors avoid recursion and historical SQL dependencies. Backup manifest 2
binds and transports the closure through independent reopen, re-copy and isolated
restore; empty-reference v1 encodings and original semantic IDs stay unchanged.
Normal replacement persists its original selector as `replacement-source.json`
and reconstructs inherited authority from it during exact adoption. Admission
rejects removed source refs, including an empty/schema2 rewrite. Archive capacity
is checked before writing a new hold or dispatching its selector.

Five new store regressions and one native service regression cover explicit
pre-incident targets, damaged pointer/header, omitted owner checks, archive/source
substitution, successive holds, source-independent portable backup, old readers
and real Windows sharing refusal with staged/selected reconciliation. Workspace
policy, fmt, check, strict Clippy, 901 tests (1 ignored, 107 targets), rustdoc and
build passed. Final capacity preflight passed 40 store tests and refreshed
workspace gates; the broad suite was not repeated.
Budget-exhaustion fault fixtures, domain quarantine, inside-write interruption, power loss,
deletion and full W16/E2/Gethe/Ketho/runtime acceptance remain open. See
[PROJECT_QUARANTINE_RESTORE.md](../crates/wow-store/PROJECT_QUARANTINE_RESTORE.md).

## W16 supported inactive physical epoch migration (2026-10-09)

Store/service APIs now transform an explicit verified physical v1/v2 backup into
a new unselected v3 with its original record catalog. The complete original
history, operations, validations, pins and holds remain in an independent source
archive. Original partition and native Project/Graph/analyzer/publication IDs are
preserved. New epoch/generation/validation/request IDs and complete source-to-target
maps retain aliases through one deterministic representative per target.

Every unique target receives fresh compiled owner replay before completion at
`ValidatedInactive`; no target Current, history, registry or roots are selected.
Exact continuation admits the entire present subset before resumed writes and
reconciles a completed canonical record without treating it as an owner verdict.
Both target epoch manifests are rechecked before validation mutation.

Four store and one native service regressions pass. Workspace policy, fmt, check,
strict Clippy, tests (906 passed, 1 ignored, 107 targets), strict rustdoc and build
passed. Exact cross-epoch activation, mapped target retention,
payload/old-runtime transformation, domain quarantine, pre-intent
staging recovery, inside-write/power-loss/deletion/platform and full W16/E2 or
Gethe/Ketho/runtime acceptance remain open. See
[PROJECT_MIGRATION.md](../crates/wow-store/PROJECT_MIGRATION.md).

## W16 exact Current-domain recovery observation (2026-10-09)

The service now retains the physical RecoveryReport beside a versioned observation
of only its exact Current publication. A held native Project/Graph replay returns
actual pair IDs or typed acquisition/replay Failed, Incomplete or Cancelled outcomes.
Corrupt/Unverified physical state never becomes Absent, and later Current cannot
replace the selected history record. Native store availability errors retain their
Incomplete classification. Existing physical-only recovery stays available; explicit
`wow project recover --domain-current` dispatches one service operation and writes
completed physical evidence even after replay cancellation.

Three native cases and workspace policy, fmt, check, strict Clippy, tests
(909 passed, 1 ignored, 107 targets), strict rustdoc and build pass. Actual CLI help,
strict flags and missing-root routing pass. Positive published-root CLI smoke and
injected late cancellation/availability remain NotEvaluated. No repair/activation
authority, all-generation domain validation or full W16/E2 acceptance follows.
Cross-epoch activation with mapped retention, domain quarantine, payload/runtime
migration and platform/source/runtime gates remain open. See
[PROJECT_CURRENT_RECOVERY.md](../crates/wow-service/PROJECT_CURRENT_RECOVERY.md).

## W16 guarded inactive staging and immutable target export (2026-10-09)

Store/service live staging now checks the exact source epoch, selector, optional
Current and complete fresh SQL/pin/hold closure against its verified backup before
and after native physical staging. A pin-only stale source rejects before target
creation. Native owner validation follows; the target remains inactive.

A completed migration can independently export its frozen target through the native
SQLite backup owner. Exact baseline receipt, inventory, source archive and manifests
are revalidated before export; the returned backup must match the target epoch and
snapshot. The service replays every exported Project/Graph pair. The separate source
history/holds and migration metadata remain in the original baseline, outside this
target-only export.

Six store migration cases and the extended native service lifecycle pass, including
independent reopen and owner-checked private copy mutation with old readers and
baseline evidence preserved. Workspace policy, fmt, check, strict Clippy, tests
(911 passed, 1 ignored, 107 targets), strict rustdoc and build pass. Automatic mapped
retention/activation-ready preparation, exact cross-epoch registry selection/retry,
portable full migration history, interrupted-copy/power-loss and full W16/E2 or
source/runtime acceptance remain open. See
[PROJECT_MIGRATION.md](../crates/wow-store/PROJECT_MIGRATION.md).

## W16 immutable migration-ready preparation (2026-10-09)

Store/service now prepare a separate private native copy with every mapped source
pin and the exact target-local Current, then freeze it into an independently
verified immutable artifact. Source pin IDs, kinds and holders stay unchanged;
epoch/generation and pin digests are reconstructed. The target Current has no SQL
predecessor, while original source history remains archived.

A bounded digest-bound intent precedes mutable handoff. Exact reopen admits only
planned root subsets, the selected operation's exact activation and complete
baseline generation/partition/validation/operation equality. Fresh compiled owners
replay every retained Project/Graph pair. Missing Working epoch metadata or
foreign/incomplete final output refuses before effects. Completed artifact/receipt
retry verifies and reconciles the same output without recopy.

Three new store cases and the extended native service lifecycle preserve original
source Current, old readers and immutable migration evidence. Workspace policy,
fmt, check, strict Clippy, tests (914 passed, 1 ignored, 107 targets), strict rustdoc
and build pass. Live cross-epoch selection, portable full migration history,
arbitrary interrupted-copy recovery, power loss, domain quarantine and full W16/E2
or Gethe/Ketho/runtime acceptance remain separate open gates. See
[PROJECT_MIGRATION_READY.md](../crates/wow-store/PROJECT_MIGRATION_READY.md).

## W16 portable original-source selector and hold authority (2026-10-09)

`ProjectStore::export_ready_migration_to_new` and its `LiveProjectStore` wrapper
export a new immutable READY-target backup under exact live selector, optional
Current and complete source snapshot guards before and after copying. Held
migration/READY receipts and closed inventories are revalidated; every exported
native Project/Graph pair is replayed. Original live and immutable evidence stays
unchanged. [PROJECT_SOURCE_AUTHORITY.md](PROJECT_SOURCE_AUTHORITY.md) describes the owner.

Canonical original selectors and native hold archives retain their own epoch/catalog
in a complete flat dependency DAG. Full admission rejects missing dependencies,
cycles, substituted bytes and unsafe paths. Combined source/target context is
admitted before staging files and freshly before activation, within 32 authority
manifests, 32 distinct epoch/hold-operation identities and 64 MiB. Backup/open/verify,
isolated restore, replacement and quarantine preserve the closure and inherited
source references. Source-bearing backups use schema 3, restored registries 7 and
replacement registries 8; empty sources preserve existing canonical bytes/digests.

Three native store cases and the extended native Project/Graph service lifecycle
pass. Workspace policy, fmt, all-target/all-feature check, strict Clippy, tests
(917 passed, 1 ignored, 107 targets), strict rustdoc and build pass. Exact live
cross-epoch selection/reconciliation remains next. Original source SQL/history
hydration, portable full migration history, payload/runtime transformation, domain
quarantine, arbitrary interrupted-copy recovery, inside-write/power-loss/deletion/
platform and full W16/E2/Gethe/Ketho/runtime acceptance remain open. The following
checkpoint supplies guarded cross-epoch selection.

## W16 guarded cross-epoch selection (2026-10-09)

Store/service APIs `stage_ready_selection`, `reopen_ready_selection`,
`activate_ready_selection` and `migration_selection_receipt` install one verified
READY target under exact original selector, optional Current and complete source
snapshot guards. A separately bound intent/evidence ledger precedes the native
copy. Exact epoch metadata and a durable Working marker precede writable handoff;
reopen verifies the complete SQL snapshot before writer configuration.

Fresh native Project/Graph owners validate every actual target generation before
one outer schema-6 selector rename. Selected retries use the held target owner and
dispatch no second selector or SQL activation. Adoption preserves the entire target
lifetime, staged leases, inherited locks and shared reader admissions. Original
source selectors/holds remain archival authority under their own epochs. Independent
readback reconciles a lost response without promoting Unknown acknowledgment.
Historical installation receipts survive later legitimate publication; quarantine
rejects their lookup. See
[PROJECT_MIGRATION_SELECTION.md](PROJECT_MIGRATION_SELECTION.md).

Two native store cases and the extended native service lifecycle pass. Workspace
policy, fmt, all-target/all-feature check, strict Clippy, tests (919 passed,
1 ignored, 107 targets), strict rustdoc and build pass. Portable full migration
history, payload/runtime transformation, domain quarantine, arbitrary partial-copy/
inside-write/power-loss/deletion/platform and full W16/E2/Gethe/Ketho/runtime
acceptance remain open. Next functional work is W17 exact materialized source
profile/inventory admission. W18-W26 remain open; Gethe materialization follows
product implementation/build.

## W17 explicit local platform-source byte admission (2026-10-09)

`ProjectInputDirectory::admit_platform_source` accepts a validated
`BlizzardUiSourceProfile` and typed `PlatformSourceInventory`. Full target/profile,
revision/class, root/path/case/extension, reviewed exclusions and count/byte/metadata
admission precedes source IO. The existing confined no-follow raw reader verifies
every declared included member against mandatory digest/length, retaining unknown
and undecodable bytes. Standard unresolved LFS pointers reject. Source execution,
network acquisition and directory scanning are absent.

`AdmittedPlatformSource` privately retains exact immutable bytes. Its serialize-only
receipt binds scoped content/snapshot identities separately from the entire caller
evidence. Provider display labels and host directories are not content identity.
All declared omissions and original provenance/materializer/compatibility/license
assertions survive; included-byte verification never attests these assertions.
Inventory coverage stays Partial; root/Git completeness, client/security/license
and decoding/package/analyzer/graph/API/runtime authority remain unevaluated. See
[PLATFORM_SOURCE_ADMISSION.md](PLATFORM_SOURCE_ADMISSION.md).

Two native cases and workspace policy, fmt, all-target/all-feature check, strict
Clippy, tests (921 passed, 1 ignored, 108 targets), strict rustdoc and build pass.
The next owner consumes retained bytes through existing package/TOC/XML loaders
and binds the exact platform source generation. Full W17 project/graph/publication,
SkeletonInputView, real mirror/performance/runtime and E0/E2 acceptance remain open.
Gethe materialization remains deferred until product implementation/build is complete.

## W17 retained-byte package/load specialization (2026-10-10)

`AdmittedPlatformSource::specialize_packages` consumes explicit native package,
root and variant declarations from its held source Arc. Full native declaration
preflight, configured-root admission, exact equality with profile-selected TOCs
and observed variant digest/length pinning precede parsing. New private Admitted
branches reuse the existing TOC/XML/package loaders, dependency closure and Main
namespace. Only requested bytes are decoded with native UTF-8/NUL/size/cancellation
checks; unrelated binary inventory is retained untouched.

Requested Excluded, Unsupported and External/Conflict/Failed records yield distinct
typed refusal and logical paths. No member is reacquired from disk. Strict Retained
replay surplus rejection and frozen native load/package recipes remain unchanged.
`PlatformPackageSpecialization` holds the actual source, load/Main owners and a
serialize-only `PlatformPackageBinding` linking exact source/profile/target/evidence
and native plan identities. Original Partial inventory and unevaluated authority
survive. See [PLATFORM_SOURCE_PACKAGES.md](PLATFORM_SOURCE_PACKAGES.md).

Three focused native cases and workspace policy, fmt, all-target/all-feature check,
strict Clippy, tests (922 passed, 1 ignored, 108 targets), strict rustdoc and build
pass. The new case loads after source-directory removal and checks exact caller pin,
cancellation and excluded/unsupported demands. Platform configuration/project kind,
additive Main universe, native replay, graph/publication/SkeletonInputView and full
W17 or real mirror/performance/runtime acceptance remain open. The existing
`BlizzardUi` workspace remains Library-only; platform implementation is Main.
Gethe materialization follows product implementation/build.
