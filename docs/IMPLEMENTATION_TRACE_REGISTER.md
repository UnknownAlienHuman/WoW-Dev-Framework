# Post-merge implementation trace register

**Baseline:** `main` after PR #244, commit
[`0fdc34c`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/0fdc34c4a8afb1af51ec47d2038734d02500af42).  
**Reviewed source baseline:** `c36a0581`, after #243.  
**Started:** 2026-10-24.

This is the active addendum for source traces discovered after the exhaustive
329-document reconciliation in [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md). The
baseline register remains the inventory of old documents and preserved ideas;
it is not reopened or inflated for every new implementation finding.

A row here means its named producer-to-consumer chain was read. It does **not**
mean the user workflow was executed. The implementation order remains
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md), and detailed domain behavior remains in
the existing guides.

## Status language

- **traced** — the listed source chain was followed far enough to support the
  stated decision;
- **fixed decision** — implementation ownership/boundary is decided and should
  not be reopened without contradictory source evidence;
- **unexecuted** — no end-to-end user observation established the behavior;
- **implemented / observed** — use only after the code or real workflow exists.

## Active traces

| ID | Chain | Source status | Decision | Executed status |
| --- | --- | --- | --- | --- |
| T01 | CLI check -> local input/backend -> generic Service -> output | traced | fixed decision | unexecuted |
| T02 | Ordinary configuration -> Project configuration/publication DTO | traced | fixed decision | unexecuted |
| T03 | Flavor/Interface -> TOC parser/load plan | traced | fixed decision | unexecuted |
| T04 | Disk/Library bytes -> Project input/inventory | traced | fixed decision | unexecuted |
| T05 | XML load facts -> virtual Lua units/maps | traced | fixed decision | unexecuted |
| T06 | Main/Library/virtual files -> native Emmy session/diagnostics | traced | fixed decision | unexecuted |
| T07 | Compiled Emmy pin -> workspace backend identity | traced | fixed decision | unexecuted |
| T08 | Unsaved editor text -> LSP diagnostics | traced | fixed decision | unexecuted |
| T09 | File/load/Library change -> current diagnostics/index | traced | fixed decision | unexecuted |
| T10 | Builder output -> ordinary Library reader -> typed WoW metadata/maps | traced | fixed decision | unexecuted |
| T11 | Addon chunk arguments -> transformed virtual context -> Emmy facts | traced | fixed decision | unexecuted |
| T12 | Upstream diagnostic -> adapter DTO -> LSP payload | implemented on feature branch | fixed decision | unexecuted |

## T01 — the current generic check route is not the replacement owner

**Evidence:** `apps/wow/src/args.rs` binds directly to `LocalCommand`;
`wow-service/src/local/{input,backend,projection,response}.rs`, `service.rs`,
`model.rs` and `apps/wow/src/output.rs` carry the result.

The route requires Reference, analyzer compatibility and Project publication
state; `Service::check` requires Reference/Rules component health; an empty rule
selection expands to every registry descriptor; `GenericFinding` loses the
upstream explanation; text output emphasizes internal component/capability data.

**Decision:** `wow-service` gets one direct `addon_check` use case over the lower
Project/Emmy owners. Reference/Rules exist only for explicitly selected WoW
checks. Do not fabricate empty Reference, compatibility, RuleRegistry or Project
objects to satisfy legacy constructors. Keep the old route only for transitional
callers, then remove it with them. See PR #245 and queue item 1.

## T02 — `ProjectConfiguration` is a publication contract

**Evidence:** `wow-project/src/configuration.rs` binds Project ID, profile/source/
generation identities, accepted analyzer pins/probe report, capability policy,
graph budgets and canonical digests.

**Decision:** it is not the ordinary `wow check` DTO. `wow-service::addon_check`
owns a small public configuration for root, optional TOC, flavor, optional target
Interface, Library directories, dependency roots, checks, Emmy diagnostic
configuration and failure threshold. Legacy publication callers may continue to
adapt their configuration while they exist.

## T03 — useful TOC parsing is hidden behind `ProfileIdentity`

**Evidence:** `wow-project/src/load/{mod,toc}.rs` accepts a full
`ProfileIdentity`, requires a positive target Interface, turns mismatch into a
generic configuration failure and binds the profile digest into the load plan.

**Decision:** extract a small Project load target containing flavor, optional
Interface and `TocLoadContext`. Keep exact TOC metadata and selected file order.
An explicit incompatible Interface produces a located compatibility diagnostic,
not silent target switching or loss of all usable files. Legacy profile callers
adapt to the small target.

## T04 — exact input files are reusable; publication inventory is not required

**Evidence:** `wow-project/src/disk.rs`, `disk/library.rs` and `inventory.rs`.
`ProjectInputDirectory`/Library acquisition already enforce root capabilities,
regular files, no-follow opens, UTF-8/path/case checks, deterministic ordering
and bounded bytes. `ProjectInputFile` validates exact path/content/digest and
converts directly to `LuaWorkspaceFileInput`. `ProjectInputInventory::build`
requires the publication configuration and rejects a physical-empty Main.

**Decision:** reuse the safe readers and `ProjectInputFile` handoff. The direct
check does not construct publication inventory. Selected virtual XML units can
form part or all of the language program without a dummy physical Lua file.

## T05 — XML unit preparation is coupled to publication receipts

**Evidence:** `wow-project/src/xml_lua.rs` contains useful collection of inline
units, entity/newline/discontinuous mappings and virtual workspace input, but
its current `prepare` wrapper also requires full `ProjectConfiguration`,
generation, capability budgets and backend identity.

**Decision:** extract pure preparation from selected load/package plans, exact
captured source bytes, direct limits and compiled analyzer identity. The direct
check and later source index consume those same units/maps. Keep a legacy wrapper
that adds publication identities for old callers; do not place them in the new
result.

## T06 — one shared Emmy engine exists but is fixed and private

**Evidence:** `wow-emmy/src/{session,references}.rs` and
`syntax/analyzer.rs`. `RegisteredMemberAnalysis` owns a real shared Main/Library/
virtual `EmmyLuaAnalysis`, but is private, creates `Emmyrc::default`, forces its
own diagnostic toggle, omits `init_std_lib(None)` and builds member/flow support
regardless of type-only need. Other syntax/flow routes create independent
analyses.

**Decision:** `wow-emmy::session` owns one configurable native session. Set Lua
5.1, initialize embedded STD once, register Library/Main/virtual units, collect
syntax diagnostics unconditionally and enable member/call/flow collectors only
for selected checks that need them. Do not instantiate another analyzer in
Service, CLI or LSP.

**Implementation boundary after T12:** the standalone exact-snapshot adapter now
initializes embedded STD and preserves complete diagnostic payloads. The shared
registered Main/Library/virtual owner remains fixed and still needs the
configuration/optional-collector refactor above. Do not mark T06 implemented from
the standalone change.

## T07 — report hashes are not part of runtime analyzer validation

**Evidence:** `wow-emmy/src/{workspace,references}.rs`.
`EmmyBackendIdentity` requires surface and compatibility-report SHA values, but
`validate_compiled_backend` compares only crate name, version, revision and tree.
The report hashes currently affect snapshot/probe/replay identity rather than
whether the compiled engine matches the pin.

**Decision:** the direct session derives an internal compiled backend identity
and never fabricates report hashes. Retain the report-derived identity only for
legacy replay/probe consumers until they are removed.

## T08 — LSP overlay state is useful; its analyzer path is not

**Evidence:** `wow-project/src/overlay/state.rs` correctly owns exact unsaved
text, client version, internal revision and UTF-8/UTF-16 projection, but
`analyzer_input` builds an isolated one-file `LuaWorkspaceSnapshot`.
`wow-service/src/frontend/lsp/diagnostics.rs` analyzes that input separately.
Before T12 it also replaced upstream wording with a constant sentence.

**Decision:** preserve overlay text/version/coordinate logic and pass an exact
file-text override to the same `wow-service::addon_check` owner. Unchanged Main,
Library, STD and mapped XML units remain in that owner. A stale/rejected/cancelled
candidate cannot publish over a newer document version. Issue #198 records the
frontend route.

**Implemented subset:** the transitional LSP payload now uses the exact upstream
message and upstream severity carried by the adapter. It remains an isolated
one-file analyzer route and therefore does not complete T08.

## T09 — physical deltas and load/config changes have different update rules

**Evidence:** `wow-emmy/src/session.rs`, Project update/publication code and
`wow-service/src/live_project/operations.rs`. Exact physical add/update/remove
exists with conservative full semantic reindex; the live Service route rejects
TOC/package-selected plans.

**Decision:** use an exact delta only when selected membership, load context,
Library identity and analyzer configuration remain unchanged. TOC/XML/dependency/
Library/target/config changes rebuild selection/context and the configured owner
cold, then publish atomically. Deletion removes declarations, virtual units,
maps, diagnostics and later index rows together. Issue #109 owns observation of
this behavior.

## T10 — the public builder report has no production handoff

**Evidence:** `apps/wow-reference-builder/src/lib.rs` writes ordinary `.lua`
files plus `source-report.json` with schema `wow-native-source-build/2`.
Production search found only builder tests/workflow consuming that schema.
`wow-project/src/disk/library.rs` loads the `.lua` files and does not deliver the
report. The old `wow-service/src/local/native_artifact/wire.rs` expects an
unrelated `wow-service/native-input-report/*` envelope and deserializes mappings,
metadata sidecars and limitations as `IgnoredAny`.

The current sidecars contain field names and source links; typed values remain in
raw source documents. Therefore neither retaining the report nor reading a
sidecar establishes usable WoW semantics.

**Decision:** keep one `source-report.json` boundary; do not add another artifact
envelope. `wow-annotations` owns the shared serializable generated-Library
manifest/mapping DTO, with typed API facts supplied by their existing Reference
owners. The builder writes it; Project Library acquisition returns exact optional
bounded manifest bytes without interpreting WoW semantics; `wow-service` validates
and consumes the same DTO. A plain `.lua` Library without a manifest remains
valid for type analysis and makes only affected WoW checks unavailable.

The manifest contains selected revision/flavor/API context, projection status and
limitations, generated file path/digest/length, exact source mappings, and only
the typed environment/restriction/origin relations used by implemented checks.
Do not embed the complete source corpus or require the original checkout for
ordinary checking. Retire the old native-input artifact envelope with its
callers rather than creating a schema ladder.

## T11 — the mapped addon-chunk adapter is compatible with pinned Emmy semantics

**Evidence:** pinned Emmy call/multi-return inference and assignment/table-owner
compilation, plus the existing transformed-workspace/source-map mechanisms.
A helper call can provide addon name and one stable table result; the second local
retains that table identity, and field assignments resolve against it.

**Decision:** use one transformed in-memory context view with exact maps for
supported top-level addon chunk arguments. Mark helper calls context-only so
synthetic API/call/flow facts cannot become user findings. Do not rewrite user
files, infer context from local variable spelling or reuse package path prefixes
as semantic namespace. XML handler wrappers and special Blizzard environments
remain separate adapters. This conclusion is source-derived and still requires
an executed addon observation.

## T12 — preserve the upstream diagnostic contract before frontend projection

**Evidence before implementation:** `syntax/analyzer.rs` accepted four string
codes, discarded every other upstream diagnostic, omitted `message`, and forced
normalized severity to Error. The transitional LSP then emitted one constant
sentence. The pinned analyzer exposes the original code, message, severity and
embedded STD loader.

**Implemented on `feat/direct-addon-check-2026-10-24`:** diagnostic schema `/3`
retains string, numeric and missing codes, the exact bounded upstream message,
the upstream severity and exact mapped source span. The four known diagnostic
families keep their typed kind; every other code uses `Other` without being
dropped. Code/message/count bounds remain explicit. The standalone analyzer sets
Lua 5.1 and calls `init_std_lib(None)` before user-file registration. The
transitional LSP serializes the retained message instead of a fabricated one.

**Still open:** this does not create the configured Main/Library/virtual session,
thread user `EmmyrcDiagnostic` settings, repair the old generic CLI result, or
prove behavior on an addon/editor. The shared session and direct Service/CLI
route remain T06/T01 implementation work. Build/Clippy evidence is recorded only
after the branch workflow completes; an end-to-end observation remains
`unexecuted`.

## Executed observations still required

No row above is promoted by documentation, compilation or Clippy. The following
must be observed after the corresponding product slice exists:

- builder output loaded through the ordinary reader, with valid/invalid typed
  calls preserving upstream message/code/severity/location;
- optional generated manifest enabling one implemented WoW check while a plain
  Lua Library still supports type checking;
- physical, mapped XML and XML-only programs;
- addon private table identity and one supported handler signature;
- add/edit/remove/rename plus changed TOC/XML/Library without stale results;
- unsaved LSP edit, valid version gap and stale-result rejection;
- later-process body/symbol search returning readable source-linked context.

Update this register when source evidence changes a boundary or a real
observation changes an executed status. Do not turn it into a second product
specification, acceptance-tier system or percentage dashboard.
