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

## Source graph and XML handler route

`wow graph build` now composes file/load/XML/inheritance/mixin source proposals,
Main callable/call facts, and independent direct-call and XML script-assignment
and SavedVariables read/write recognizer partitions. The v7 receipt retains exact source/evidence, skipped-site
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

The frozen E2-B contract declares 26 active rule ids. Fourteen are implemented and
published: the core.lua.* triples (create_frame, create_from_mixins,
mixin_assignment) plus the eleven families above. The remaining twelve are
core.toc.* (5), core.xml.* (4) and core.state.* (3). Their upstream fact types do
not exist in Rust yet: TocPackageFact and XmlTemplateFact appear only in the
normative documents, and the contract itself marks the
wow-project/e2-toc-xml-fact-adapter as a future prerequisite for real project
integration. Recognizer code never parses source, so those rules stay unimplemented
until an owner publishes their facts.

Structural mutation fixtures per family, the apps/wow graph export lane and full E2
package acceptance remain open and are NotEvaluated.


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
[AXIS_USAGE.md](../crates/wow-graph/AXIS_USAGE.md). Full conflict/derivation/evidence
resolution and coherent ProjectStore acquisition remain open.

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

`wow graph build` result schema is now `wow-service/graph-build-result/10` and
the source graph profile is `wow-project/source-load-proposals/10`; the request
shape remains unchanged. Focused Linux/Rust 1.99.0 checkpoint
`36969801633` passed formatting, strict Clippy for the complete affected chain,
repository policy and the exact seven-product-file boundary, then published
commit `4479cf333de8d747e3f53549ea3732598d42df84`. No tests, Windows run,
real-addon acceptance or package gate is claimed.

## Retained source evidence reads

Graph-build v7 bundles now feed every graph read directly. Explain joins the
validated partition owner to project-admitted file/handle metadata and a
core-validated evidence catalog, with bounded shared derivation expansion.
See [GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md). Source bytes are not
reopened; full graph conflicts/inference-rule records, sidecar replay, runtime
and coherent ProjectStore remain open. Existing bare-snapshot encodings and
build identities are unchanged. This advances code, not acceptance gates.

## Manifested retained ProjectStore code slice

`wow-store::project` now supplies one registered WAL epoch, immutable partition
versions, complete generation membership, pinned read transactions, durable
operation phases and inactive/read-back/validation/current-CAS publication.
`wow graph publish` and `reconcile` are explicit service operations; all six graph
reads can select one current/exact stored generation. Graph and project adapters
restore and validate retained v7 bundle data without another analyzer session.
See [GRAPH_STORE.md](../apps/wow/GRAPH_STORE.md). This supersedes the earlier
blanket “no persistent ProjectStore” descriptions for this narrow storage slice,
not full E2-D acceptance: live project views, incremental invalidation, retention,
GC, backup/restore, epoch replacement and crash/power-loss acceptance remain open.
Only the existing internal project-to-store dependency is activated; no tests or
external dependency versions are changed. Full launch gates stay unchanged.

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
fixture/checksum freeze, the remaining TOC/XML/state rules, `apps/wow` real-addon
acceptance and the mandatory final WoW API Ketho MCP comparison remain open until
separately executed and recorded.
