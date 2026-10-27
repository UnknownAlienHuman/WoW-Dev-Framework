# Current implementation status

**Documentation/source audit base:** `main` after PR #268
(`28302632599bed087394dee83751815d13041579`)
**Baseline register:** PR #244 / `c36a058`
**Active traces:** T01–T35 in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)

This is the only live delivery checkpoint. The queue orders work; owner/domain
guides define stable contracts; the trace register keeps source decisions and
exact observations. README, AGENTS, skills and Issues must link here rather than
copy a moving delivery list.

## Verified documentation boundary

- all 329 baseline Markdown files are represented exactly once in the baseline
  register: 287 `read / traced`, 42 `read / not_applicable`, no partial or
  unchecked row;
- all 60 current Markdown files and all 21 open implementation Issues were read
  again against current Project/Emmy/Service owners;
- current local Markdown paths/anchors and the four synchronized skill copies are
  valid;
- obsolete E1/E2/E3/platform specifications remain outside the active tree;
- audit history and the supplied Claude review remain evidence, not live product
  instructions. See [AUDIT.md](AUDIT.md),
  [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md) and the trace register for details.

`traced` means a producer/handoff/reader chain was inspected. `implemented` means
code exists. `observed` requires an ordinary command or protocol sequence with
its visible result. Compilation, Clippy and source review never make a workflow
observed.

## Implemented internal owners

| Boundary | State |
| --- | --- |
| configured Emmy diagnostics preserve upstream code, exact message, severity and byte range; Lua 5.1 and embedded STD share one owner | implemented, user workflow unexecuted |
| exact Main + explicit Library + optional virtual `ConfiguredAnalyzerSession`; diagnostics and selected semantic collection share it | implemented, user workflow unexecuted |
| direct `wow-service::addon_check` result without Reference, Rules, Project publication, graph, store or presentation construction | implemented, user workflow unexecuted |
| compiled Emmy identity is distinct from compatibility-report evidence | implemented, user workflow unexecuted |
| exact Main/canonical Library set/virtual/config compatibility gates owner reuse | implemented, user workflow unexecuted |
| finite located limitation categories and reachable package-plan issue mapping | implemented, ordinary standalone TOC path open |
| one three-valued TOC predicate reduction | implemented, ordinary workflow unexecuted |

The Library caller-order concern is closed: Library snapshots are sorted by exact
snapshot ID before registration and the same canonical set is compared for reuse.

## Exact open boundary: Slice 1A

There is still no ordinary public addon checker. The executable uses the
transitional `LocalCommand -> LocalProjectBackend -> generic Service` route.
Slice 1A must connect normal input directly without preserving that platform.

Four source boundaries are mandatory:

1. **Profile-free plan identity.** Current TOC readers and retained plans bind a
   mandatory positive Interface and full `ProfileIdentity` digest. Add one small
   Project-owned `AddonLoadTarget` containing explicit flavor, optional positive
   target Interface and optional `TocLoadContext`. The plan stores/validates that
   target directly. Legacy callers adapt profiles into the target; the direct
   route never fabricates a profile, report/source digest, schema identity or
   generation.
2. **Physical projection mode.** The current loader eagerly opens and parses XML,
   so malformed XML aborts physical Lua acquisition. Slice 1A needs an identity-
   bound `physical_lua` projection: retain TOC records and direct `.lua` bytes,
   do not open XML, and emit a located source-selection limitation for each
   included XML branch deferred to Slice 1B. An XML-only addon is not reported
   clean by Slice 1A.
3. **Scope-aware standalone limitations.** The existing mapper accepts only a
   package plan and deliberately drops parser dependency placeholders. Add a
   mapper for one `ProjectLoadPlan`. Required dependencies, unresolved selected
   conditions, missing direct source and deferred XML make `LuaTypes` partial;
   normal optional-dependency absence and Interface metadata with no positive
   target do not. Nonblocking observations stay in Project and never enter
   `AddonCheckLimitation`. Do not derive the direct result from global
   `external_files_complete()` alone.
4. **Exact snapshot/input boundary.** Use `EmmyBackendIdentity::compiled_pinned()`;
   Main uses universe `Project`; each explicit annotation-Library directory is a
   separate `BlizzardUi` snapshot under existing Main/Library limits. Preserve
   canonical root-relative paths in identities/results; the frontend may join the
   configured root for text display. Reject duplicate resolved Library roots or
   duplicate snapshot IDs rather than silently merging them. A malformed caller
   Library is invalid input, not internal `ComponentUnavailable`; its error names
   the configured Library root and root-relative file.

`check_addon` must then derive `Complete` only when no limitation required by the
selected `LuaTypes` scope remains. It must stop adding an unconditional chunk-
context limitation and hard-coded `Partial` for files that do not consume that
context.

Slice 1A accepts exactly flavor `retail`. If selected deferred XML exists but no
physical Lua does, use the supported empty `Project` snapshot and return a located
partial result; do not fabricate a dummy file or report clean.

## Ordered work

1. **Slice 1A:** explicit root + explicit TOC + `retail` + one or more Library
   directories -> profile-free physical plan -> scope-aware limitations -> exact
   snapshots -> existing `check_addon` -> root-relative text/JSON diagnostics and
   exit status.
2. **Slice 1B:** dependencies, exact selected order, mapped XML/chunk/handler
   context and coherent updates.
3. **Slice 1C:** channel, `LoadWith`, implicit bindings, positive target Interface
   and secure/special contexts.
4. Generated-Library metadata and selected WoW checks.
5. Thin LSP/MCP adapters, then the direct SQLite body/symbol index and readable
   context.

The exact acceptance conditions are in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Readiness verdict

- baseline and current documentation reconciliation: **complete at the named
  base**;
- canonical plan for Slice 1A: **ready after the T32–T35 source corrections above**;
- first ordinary `wow check`: **not implemented**;
- any addon/editor/search workflow observed: **no**;
- guarantee of no future runtime defect: **no**. The documentation now prevents
  known profile/publication, eager-XML, global-completeness and snapshot-identity
  traps without pretending implementation has already proved itself.
