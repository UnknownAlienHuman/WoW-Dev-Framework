# Post-merge implementation trace register

**Documentation baseline:** PR #244, merge `0fdc34c`.  
**Implementation work started:** 2026-10-24.

This is the active implementation addendum to
[AUDIT_REGISTRY.md](AUDIT_REGISTRY.md). The old register accounts for every
baseline document and preserved idea. This file records only new
producer-to-consumer traces and implementation decisions discovered after that
merge.

A source trace is not an executed user workflow. Documentation, compilation and
Clippy may move a row to **implemented**, but only an exact command/input/result
observation moves it to **observed**.

## Status language

- **traced** — the named source chain was followed far enough to support the
  decision;
- **implemented** — the code exists and passed the stated build gate;
- **observed** — the advertised user workflow was actually exercised;
- **unexecuted** — no end-to-end observation exists yet.

## Active traces

| ID | Chain | Code status | User observation |
| --- | --- | --- | --- |
| T01 | CLI check -> old local backend -> generic Service -> output | traced | unexecuted |
| T02 | ordinary config -> publication `ProjectConfiguration` | traced | unexecuted |
| T03 | flavor/Interface -> TOC parser/load plan | traced | unexecuted |
| T04 | disk/Library bytes -> exact analyzer snapshots | traced | unexecuted |
| T05 | XML load facts -> virtual Lua units/maps | traced | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented in `main` by PR #247 | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented on feature branch | unexecuted |
| T08 | unsaved editor text -> project diagnostics | partially implemented | unexecuted |
| T09 | source/config changes -> current diagnostics/index | traced | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced | unexecuted |
| T11 | addon chunk arguments -> mapped analysis context | traced design | unexecuted |
| T12 | upstream diagnostic -> adapter DTO -> LSP payload | implemented in `main` by PR #246 | unexecuted |
| T13 | caller diagnostic config -> optional semantic collectors | implemented in `main` by PR #247 | unexecuted |
| T14 | exact snapshots -> direct Service result | implemented in `main` by PR #248 | unexecuted |
| T15 | compiled pin/report evidence -> workspace identity -> legacy admission | implemented on feature branch | unexecuted |

## Fixed decisions from T01–T11

### T01 — do not extend the old generic check platform

`apps/wow/src/args.rs` routes `wow check` through `LocalCommand`,
`LocalProjectBackend`, generic `Service::check`, component capability reduction
and presentation graphs. That path requires Reference, Rules, compatibility and
Project-publication objects even for plain Lua/type diagnostics; empty rule
selection expands to every rule; the old generic finding loses the upstream
message.

**Decision:** the replacement is `wow-service::addon_check`, composed directly
over Project acquisition and the configured Emmy owner. Reference/Rules are
created only for explicitly requested WoW checks. Never fabricate empty
platform objects to satisfy legacy constructors. Remove the old route with its
last callers.

### T02 — publication configuration is not user configuration

`wow-project::ProjectConfiguration` binds project IDs, generations, probe/report
hashes, capability policy, graph budgets and canonical publication identities.

**Decision:** ordinary `wow check` gets a small config: root, optional TOC,
flavor, optional Interface target, Library directories, dependency roots,
selected checks, Emmy diagnostic settings and failure threshold. Legacy
publication callers may adapt while they exist; the public CLI never asks users
for those internal identities.

### T03 — separate load target from `ProfileIdentity`

Current TOC parsing takes a full `ProfileIdentity`, requires a positive target
Interface and turns mismatch into a generic configuration failure.

**Decision:** extract a small load target: flavor, optional Interface and load
context. Preserve selected files/order and report an explicit Interface mismatch
without silently switching TOC/flavor or discarding independently usable input.

### T04 — reuse safe acquisition, not publication inventory

`ProjectInputDirectory`, Library acquisition and `ProjectInputFile` already
enforce root capabilities, no-follow regular-file reads, path/case/UTF-8 checks,
bounds, deterministic order and exact digests. `ProjectInputInventory::build`
adds publication configuration and rejects physical-empty Main.

**Decision:** the direct check reuses safe readers and converts exact files to
`LuaWorkspaceFileInput`; it does not build publication inventory. Virtual XML
units may form part or all of the language program without a dummy `.lua` file.

### T05 — split pure XML preparation from publication receipts

`wow-project::xml_lua` already owns inline-unit collection, entity/newline and
discontinuous source mappings, but its public preparation wrapper also requires
configuration/generation/capability/report identities.

**Decision:** extract pure preparation from selected load facts, captured bytes,
direct limits and the compiled analyzer identity. Keep one legacy wrapper for
old publication callers. The direct check and later index consume the same
virtual units/maps.

### T07 — compatibility-report hashes are not runtime analyzer validation

The original `EmmyBackendIdentity` required surface/report SHA values even though
compiled-backend validation compared only crate, version, revision and tree.
That forced new direct callers either to fabricate evidence or to depend on the
old probe/report pipeline.

**Decision:** one identity records exact compiled provenance; compatibility
reports remain optional evidence for legacy publication/replay. Direct code must
never invent report hashes. Legacy report-only callers are transitional and
remain report-backed until their platform is removed.

### T08 — retain overlay state, replace its one-file analyzer

Project overlay code correctly owns exact unsaved text, client version,
internal revision and UTF-8/UTF-16 conversion. Its current analyzer input is an
isolated one-file snapshot; the transitional LSP still bypasses project
Main/Library/XML context.

**Decision:** keep overlay text/version/coordinate logic and pass exact text
overrides to the same direct addon-check owner. A stale/rejected/cancelled
candidate cannot publish over a newer document. PR #246 fixed message/severity
projection only; full project LSP reuse remains open.

### T09 — file deltas and load/config changes are different operations

Exact physical add/update/remove exists with conservative semantic reindexing.
TOC/package-selected live updates are currently refused.

**Decision:** use a file delta only when selected membership, load context,
Library identity and Emmy configuration remain unchanged. TOC/XML/dependency/
Library/target/config changes rebuild selection/context and the owner cold, then
publish atomically. Deletion removes declarations, virtual units, maps,
diagnostics and index rows together.

### T10 — one generated-Library manifest, not another envelope

The builder writes `.lua` files plus `source-report.json` schema
`wow-native-source-build/2`; the ordinary Library reader only reads Lua. The old
native-artifact reader expects an unrelated envelope and ignores mappings,
metadata and limitations. Existing sidecars contain names/source links, while
typed values remain in raw source documents.

**Decision:** keep one `source-report.json`. `wow-annotations` owns the shared
serializable manifest/mapping DTO; Project returns optional exact bounded bytes;
Service validates and consumes the same DTO. Plain Lua Libraries remain valid
for typing and only disable affected WoW checks. The manifest contains selected
revision/flavor/context, projection limitations, generated file identities,
source mappings and only typed facts used by implemented checks—not the complete
source corpus or old Reference snapshot.

### T11 — mapped helper context is compatible with pinned Emmy semantics

Pinned Emmy keeps final-position multi-return values and table identity, while
existing transformed-workspace/source-map machinery can map a replacement
expression to original bytes.

**Decision:** create one analysis-only helper/private table per addon, replace
only chunk-owned ellipsis expressions in an in-memory mapped view, and mark
helper declarations/calls context-only so they never become user API/flow facts.
Do not rewrite files, infer semantics from variable spelling, or treat package
path prefixes as namespaces. XML handler and secure/bootstrap contexts remain
separate adapters. This is source-derived and still requires execution.

## Implemented slices

### T12 — preserve the upstream diagnostic contract

Before PR #246 the adapter admitted four codes, dropped all other diagnostics,
omitted the message and forced severity to Error; LSP emitted one constant
sentence.

PR #246 (`b56c346`) now retains string/numeric/missing codes, exact bounded
messages, upstream severity and mapped source spans. Known families keep typed
kinds; unknown codes become `Other` without being discarded. The standalone
snapshot analyzer sets Lua 5.1 and initializes embedded STD. Transitional LSP
publishes the retained message/severity. Exact-head repository policy,
formatting, full compilation and strict Clippy passed. Addon/editor behavior is
still unobserved.

### T06/T13 — one configured Main/Library/virtual owner

PR #247 (`b40f701`) added `EmmyAnalysisConfiguration` and
`ConfiguredAnalyzerSession`:

- caller-supplied pinned `EmmyrcDiagnostic`, fixed Lua 5.1, no `.emmyrc`
  discovery;
- exact Main, explicit Library and optional virtual-unit registration in one
  retained upstream owner;
- embedded STD initialized once;
- diagnostics-only collection does not invoke member/call/local-flow;
- semantic collectors run only through an explicit method in the same owner;
- compatibility includes Main identity, Library set, virtual
  snapshot/generation and structural diagnostic configuration equality.

The legacy registration constructor still installs its default diagnostic config
while registering files; the new owner replaces the supported diagnostics policy
before collection and then adds STD. Since the new public config cannot alter
parser/runtime features, this does not create mixed parsing semantics. Move
configuration/STD initialization into the constructor when legacy entry points
are adapted or removed. Exact-head policy, formatting, compilation and strict
Clippy passed. No Service/CLI workflow was observed.

### T14 — direct Service result without the old platform

PR #248 (`d96c2fb`) added `wow-service::addon_check` over exact already-acquired
snapshots. It:

- creates only `ConfiguredAnalyzerSession`; no Reference, Rules, compatibility
  report, Project publication, graph, store or presentation graph;
- keeps code, exact message, kind, severity, path, content digest and byte span;
- separates physical Main and virtual diagnostics;
- separates findings from limitations and records exact analysis/snapshot IDs;
- keeps deterministic Library identities and output ordering;
- maps typed cancellation/input/native failures to existing Service error
  families.

The result is intentionally `Partial`: addon-name/private-table chunk arguments
are not yet bound. Virtual input additionally reports missing exact XML source
projection and handler parameters instead of fabricating locations. The function
accepts exact snapshots; root/TOC/Library acquisition and CLI wiring remain open.
Exact-head policy, formatting, compilation and strict Clippy passed. No addon or
editor workflow was observed.

### T15 — distinguish compiled provenance from compatibility evidence

`feat/compiled-emmy-identity-2026-10-24` changes `EmmyBackendIdentity` without
weakening legacy report-backed identity:

- `EmmyBackendIdentitySource` distinguishes `compiled_pin` from
  `compatibility_report`;
- `compiled` and `compiled_pinned` create exact crate/version/revision/tree
  identities with no report digests;
- report-backed `new` still validates both SHA-256 values;
- report-backed serialization preserves its historical field shape; compiled
  serialization emits an explicit source and omits report fields;
- safe optional evidence accessors let admission code distinguish the two;
- old report-only getters remain solely for existing legacy callers and fail
  closed if misused, rather than returning a sentinel or fabricated digest.

`AnalyzerBindingDeclaration` and replay/publication still require report-backed
inputs through their existing construction path. The direct snapshot builder can
now bind the actual compiled pin without running or pretending to have run the
compatibility probe. Build status must be recorded after the exact-head gate;
there is still no user observation.

## Executed observations still required

No build-only slice above is a release claim. Record the exact command, fixture,
input identities and result before marking any of these observed:

- builder output loaded through the ordinary reader, with valid/invalid typed
  calls preserving message/code/severity/location;
- optional manifest enabling one implemented WoW check while plain Lua still
  supports type checking;
- physical, mapped XML and XML-only programs;
- addon private-table identity and one supported handler signature;
- add/edit/remove/rename plus changed TOC/XML/Library without stale results;
- unsaved LSP edit, valid client-version gap and stale-result rejection;
- later-process body/symbol search returning readable source-linked context.

Update this register when evidence changes a boundary or a real observation
changes status. Do not turn it into a second product specification, acceptance
tier system or percentage dashboard.
